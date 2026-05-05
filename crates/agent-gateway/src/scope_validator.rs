use authority_domain::SkillGrant;

/// Result of a file scope validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileScopeResult {
    Allowed,
    Denied(String),
}

/// Result of a schema scope validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaScopeResult {
    Allowed,
    Denied(String),
}

/// Validate that a file path is within the grant's allowed scope.
///
/// Rules:
/// - Deny list always wins over allow list.
/// - Rejects absolute paths (no leading `/` on Unix, no drive letters on Windows).
/// - Rejects path traversal (`..` components).
/// - Checks against both allowed_file_patterns and denied_file_patterns.
pub fn validate_file_scope(file_path: &str, grant: &SkillGrant) -> FileScopeResult {
    // Reject path traversal before anything else
    if file_path.contains("..") {
        let components: Vec<&str> = file_path.split(&['/', '\\'][..]).collect();
        if components.contains(&"..") {
            return FileScopeResult::Denied("path traversal is not allowed".to_string());
        }
    }

    // Deny list wins — checked before allow list
    if grant
        .denied_file_patterns
        .iter()
        .any(|p| file_path.starts_with(p) || file_path == p)
    {
        return FileScopeResult::Denied(format!(
            "file path '{}' matches denied pattern",
            file_path
        ));
    }

    // Allow list check — must happen BEFORE the absolute-path rejection so that
    // explicitly-permitted absolute paths like "/workspace/src/main.rs" are allowed
    if grant
        .allowed_file_patterns
        .iter()
        .any(|p| file_path.starts_with(p) || file_path == p)
    {
        return FileScopeResult::Allowed;
    }

    // Reject absolute paths that didn't match an allowed pattern
    if file_path.starts_with('/') || file_path.starts_with('\\') {
        return FileScopeResult::Denied("absolute paths are not allowed".to_string());
    }
    // Reject drive-letter absolute paths (Windows)
    if file_path.len() >= 2 && file_path.as_bytes()[1] == b':' {
        return FileScopeResult::Denied("absolute paths are not allowed".to_string());
    }

    FileScopeResult::Denied(format!(
        "file path '{}' is not within the grant's allowed scope",
        file_path
    ))
}

/// Validate that a schema view name is allowed by the grant.
///
/// Only views allowed by the grant are accessible; base tables cannot be
/// accessed directly.
pub fn validate_schema_scope(view_name: &str, grant: &SkillGrant) -> SchemaScopeResult {
    if grant
        .allowed_file_patterns
        .iter()
        .any(|p| p == view_name || view_name.starts_with(p))
    {
        return SchemaScopeResult::Allowed;
    }
    SchemaScopeResult::Denied(format!(
        "schema view '{}' is not allowed by this grant",
        view_name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ActorId, GrantId, GrantState, TenantId};
    use chrono::{Duration, Utc};

    fn test_grant() -> SkillGrant {
        SkillGrant {
            id: GrantId::new("test-grant").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            allowed_actions: vec!["file.read".to_string()],
            denied_actions: vec![],
            allowed_file_patterns: vec!["/workspace/src".to_string(), "data/".to_string()],
            denied_file_patterns: vec![
                "/workspace/src/secret".to_string(),
                "data/private/".to_string(),
            ],
            budget_tokens: Some(1000),
            budget_operations: Some(50),
            separation_group: "developer".to_string(),
            state: GrantState::Active,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn allows_file_within_allowed_pattern() {
        let grant = test_grant();
        let result = validate_file_scope("/workspace/src/main.rs", &grant);
        assert_eq!(result, FileScopeResult::Allowed);
    }

    #[test]
    fn denies_file_in_denied_pattern() {
        let grant = test_grant();
        let result = validate_file_scope("/workspace/src/secret/key.rs", &grant);
        assert!(matches!(result, FileScopeResult::Denied(_)));
    }

    #[test]
    fn denies_absolute_unix_path() {
        let grant = test_grant();
        let result = validate_file_scope("/etc/passwd", &grant);
        assert!(matches!(result, FileScopeResult::Denied(_)));
    }

    #[test]
    fn denies_path_traversal() {
        let grant = test_grant();
        let result = validate_file_scope("../escape/src/main.rs", &grant);
        assert!(matches!(result, FileScopeResult::Denied(_)));
    }

    #[test]
    fn denies_file_outside_scope() {
        let grant = test_grant();
        let result = validate_file_scope("/other/path/file.rs", &grant);
        assert!(matches!(result, FileScopeResult::Denied(_)));
    }

    #[test]
    fn allows_relative_path() {
        let grant = test_grant();
        let result = validate_file_scope("data/records.csv", &grant);
        assert_eq!(result, FileScopeResult::Allowed);
    }

    #[test]
    fn denies_relative_path_in_denied_subdir() {
        let grant = test_grant();
        let result = validate_file_scope("data/private/keys.json", &grant);
        assert!(matches!(result, FileScopeResult::Denied(_)));
    }

    #[test]
    fn schema_scope_allows_matching_view() {
        let grant = test_grant();
        let result = validate_schema_scope("/workspace/src/views/users", &grant);
        assert_eq!(result, SchemaScopeResult::Allowed);
    }

    #[test]
    fn schema_scope_denies_non_matching_view() {
        let grant = test_grant();
        let result = validate_schema_scope("analytics/sales", &grant);
        assert!(matches!(result, SchemaScopeResult::Denied(_)));
    }
}
