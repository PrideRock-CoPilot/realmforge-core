use crate::{ActorId, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of a skill grant.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantState {
    Active,
    Expired,
    Revoked,
    Suspended,
}

impl std::fmt::Display for GrantState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Expired => write!(f, "expired"),
            Self::Revoked => write!(f, "revoked"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

/// A skill grant binds an actor to a set of allowed actions with constraints.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillGrant {
    pub id: GrantId,
    pub actor_id: ActorId,
    pub tenant_id: TenantId,
    pub allowed_actions: Vec<String>,
    pub denied_actions: Vec<String>,
    pub allowed_file_patterns: Vec<String>,
    pub denied_file_patterns: Vec<String>,
    pub budget_tokens: Option<u64>,
    pub budget_operations: Option<u64>,
    pub separation_group: String,
    pub state: GrantState,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl SkillGrant {
    /// Check if the grant is in a usable (Active) state.
    pub fn is_active(&self) -> bool {
        self.state == GrantState::Active
    }

    /// Check if the grant is expired.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }

    /// Check if a specific action is allowed by this grant.
    /// Deny list wins over allow list.
    pub fn allows_action(&self, action: &str) -> bool {
        if self.denied_actions.iter().any(|d| d == action) {
            return false;
        }
        if self.allowed_actions.iter().any(|a| a == action) {
            return true;
        }
        false
    }

    /// Check if a file path is allowed by this grant.
    /// Deny list wins over allow list.
    pub fn allows_file(&self, file_path: &str) -> bool {
        if self
            .denied_file_patterns
            .iter()
            .any(|p| file_path.starts_with(p) || file_path == p)
        {
            return false;
        }
        if self
            .allowed_file_patterns
            .iter()
            .any(|p| file_path.starts_with(p) || file_path == p)
        {
            return true;
        }
        false
    }
}

/// A grant ID type.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GrantId(String);

impl GrantId {
    pub fn new(value: impl Into<String>) -> Result<Self, crate::IdError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(crate::IdError::Empty { kind: "GrantId" });
        }
        Ok(Self(value))
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for GrantId {
    type Err = crate::IdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_grant() -> SkillGrant {
        SkillGrant {
            id: GrantId::new("grant-1").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            allowed_actions: vec!["file.read".to_string(), "file.write".to_string()],
            denied_actions: vec!["file.delete".to_string()],
            allowed_file_patterns: vec!["/workspace/src".to_string()],
            denied_file_patterns: vec!["/workspace/src/secret".to_string()],
            budget_tokens: Some(1000),
            budget_operations: Some(50),
            separation_group: "developer".to_string(),
            state: GrantState::Active,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn grant_allows_explicitly_allowed_action() {
        let grant = active_grant();
        assert!(grant.allows_action("file.read"));
        assert!(grant.allows_action("file.write"));
    }

    #[test]
    fn grant_denies_explicitly_denied_action() {
        let grant = active_grant();
        assert!(!grant.allows_action("file.delete"));
    }

    #[test]
    fn grant_denies_unspecified_action() {
        let grant = active_grant();
        assert!(!grant.allows_action("core.create_snapshot"));
    }

    #[test]
    fn grant_allows_allowed_file_path() {
        let grant = active_grant();
        assert!(grant.allows_file("/workspace/src/main.rs"));
    }

    #[test]
    fn grant_denies_denied_file_pattern() {
        let grant = active_grant();
        assert!(!grant.allows_file("/workspace/src/secret/key.rs"));
    }

    #[test]
    fn grant_is_expired_check() {
        let grant = active_grant();
        assert!(!grant.is_expired(Utc::now()));
        let far_future = Utc::now() + chrono::Duration::hours(2);
        assert!(grant.is_expired(far_future));
    }

    #[test]
    fn grant_state_active_check() {
        let grant = active_grant();
        assert!(grant.is_active());

        let mut revoked = grant.clone();
        revoked.state = GrantState::Revoked;
        assert!(!revoked.is_active());
    }

    #[test]
    fn grant_state_display() {
        assert_eq!(GrantState::Active.to_string(), "active");
        assert_eq!(GrantState::Revoked.to_string(), "revoked");
    }

    #[test]
    fn grant_id_rejects_empty() {
        assert!(GrantId::new("").is_err());
    }

    #[test]
    fn grant_id_generates_non_empty() {
        let id = GrantId::generate();
        assert!(!id.as_str().is_empty());
    }
}
