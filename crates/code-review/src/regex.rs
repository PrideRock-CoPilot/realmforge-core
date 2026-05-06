use crate::check::{Check, CheckConfig, CheckResult, CheckType, FileLocation, Finding};
use crate::error::{CodeReviewError, Result};
use crate::severity::Severity;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::path::Path;
use std::time::Instant;

/// A check that scans source file contents using regex patterns.
///
/// Supports three modes:
/// - **Ban pattern**: Flags any file matching the `pattern` that does NOT have
///   a preceding line matching `allowed_pattern` (e.g., unwrap() OK if preceded by SAFETY comment).
/// - **Required pattern**: Flags any occurrence of `pattern` that does NOT have
///   a `required_above` pattern on a preceding line.
/// - **Simple presence**: Flags any file containing `pattern`.
pub struct RegexCheck {
    config: CheckConfig,
    compiled_pattern: Regex,
    allowed_regex: Option<Regex>,
    required_above_regex: Option<Regex>,
    file_glob: Option<GlobSet>,
}

impl RegexCheck {
    /// Create a new regex check from configuration.
    pub fn new(config: CheckConfig) -> Result<Self> {
        let pattern_str = config.pattern.as_ref().ok_or_else(|| {
            CodeReviewError::invalid_check(&config.id, "Regex check must have a 'pattern' field")
        })?;

        let compiled_pattern =
            Regex::new(pattern_str).map_err(|e| CodeReviewError::InvalidCheck {
                check_id: config.id.clone(),
                reason: format!("Invalid regex pattern '{}': {}", pattern_str, e),
            })?;

        let allowed_regex = if let Some(ref allowed) = config.allowed_pattern {
            Some(
                Regex::new(allowed).map_err(|e| CodeReviewError::InvalidCheck {
                    check_id: config.id.clone(),
                    reason: format!("Invalid allowed_pattern regex '{}': {}", allowed, e),
                })?,
            )
        } else {
            None
        };

        let required_above_regex = if let Some(ref required) = config.required_above {
            Some(
                Regex::new(required).map_err(|e| CodeReviewError::InvalidCheck {
                    check_id: config.id.clone(),
                    reason: format!("Invalid required_above regex '{}': {}", required, e),
                })?,
            )
        } else {
            None
        };

        let file_glob = if let Some(ref pattern) = config.file_pattern {
            Some(
                compile_glob(pattern)
                    .map_err(|e| CodeReviewError::glob_pattern(pattern.clone(), e))?,
            )
        } else {
            None
        };

        Ok(Self {
            config,
            compiled_pattern,
            allowed_regex,
            required_above_regex,
            file_glob,
        })
    }

    /// Check if a file path matches the file glob pattern.
    fn matches_glob(&self, file_path: &Path) -> bool {
        match &self.file_glob {
            Some(glob_set) => glob_set.is_match(file_path),
            None => true, // No filter means match all
        }
    }

    /// Check if the line above matches the allowed_pattern (safety comment check).
    fn is_allowed(&self, lines: &[&str], current_idx: usize) -> bool {
        match &self.allowed_regex {
            Some(re) => {
                if current_idx == 0 {
                    return false;
                }
                re.is_match(lines[current_idx - 1])
            }
            None => false,
        }
    }

    /// Check if a required pattern appears on the line above the current line index.
    fn has_required_above(&self, lines: &[&str], current_idx: usize) -> bool {
        match &self.required_above_regex {
            Some(re) => {
                if current_idx == 0 {
                    return false;
                }
                // Check the line immediately above
                re.is_match(lines[current_idx - 1])
            }
            None => false,
        }
    }
}

impl Check for RegexCheck {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn severity(&self) -> Severity {
        self.config.severity
    }

    fn check_type(&self) -> CheckType {
        CheckType::Regex
    }

    fn execute(&self, workspace_root: &Path) -> Result<CheckResult> {
        let start = Instant::now();
        let mut findings = Vec::new();

        // Walk all files in the workspace
        let walk_root = workspace_root.to_path_buf();
        for entry in walkdir::WalkDir::new(&walk_root)
            .into_iter()
            .filter_entry(move |e| {
                // Always include the root directory
                if e.path() == walk_root {
                    return true;
                }
                // Skip hidden dirs and node_modules/target
                let name = e.file_name().to_str().unwrap_or("");
                !name.starts_with('.')
                    && name != "target"
                    && name != "node_modules"
                    && name != ".git"
            })
        {
            let entry = entry.map_err(|e| {
                CodeReviewError::Io(std::io::Error::other(format!("Walkdir error: {}", e)))
            })?;

            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            if !self.matches_glob(file_path) {
                continue;
            }

            // Try to read the file as text
            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary files
            };

            let lines: Vec<&str> = content.lines().collect();

            // Scan each line
            for (line_idx, line) in lines.iter().enumerate() {
                for mat in self.compiled_pattern.find_iter(line) {
                    // Check allowed pattern (e.g., preceding safety comment)
                    if self.is_allowed(&lines, line_idx) {
                        continue;
                    }

                    // Check required above pattern — if the required pattern IS
                    // present on the preceding line, this usage is valid
                    if self.has_required_above(&lines, line_idx) {
                        continue;
                    }

                    let relative_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);

                    let finding = Finding::new(
                        &self.config.id,
                        &self.config.name,
                        self.config.severity,
                        format!("Match '{}' found at line {}", mat.as_str(), line_idx + 1,),
                    )
                    .with_location(FileLocation {
                        file: relative_path.display().to_string(),
                        line: Some(line_idx + 1),
                        column: Some(mat.start() + 1),
                    });

                    findings.push(finding);
                }
            }
        }

        let elapsed = start.elapsed();

        if findings.is_empty() {
            Ok(CheckResult::passed(&self.config.id, elapsed))
        } else {
            Ok(CheckResult::failed(&self.config.id, findings, elapsed))
        }
    }
}

/// Compile a glob pattern string into a GlobSet.
fn compile_glob(pattern: &str) -> std::result::Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    let glob =
        Glob::new(pattern).map_err(|e| format!("Invalid glob pattern '{}': {}", pattern, e))?;
    builder.add(glob);
    builder
        .build()
        .map_err(|e| format!("Failed to build glob set: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_regex_config(
        id: &str,
        pattern: &str,
        file_pattern: Option<&str>,
        allowed: Option<&str>,
        required_above: Option<&str>,
    ) -> CheckConfig {
        CheckConfig {
            id: id.to_string(),
            name: format!("Check {}", id),
            description: "Regex test".to_string(),
            check_type: CheckType::Regex,
            severity: Severity::RequiredChange,
            file_pattern: file_pattern.map(|s| s.to_string()),
            command: None,
            working_dir: None,
            timeout_secs: None,
            pattern: Some(pattern.to_string()),
            allowed_pattern: allowed.map(|s| s.to_string()),
            required_above: required_above.map(|s| s.to_string()),
            property: None,
            max: None,
            warning_at: None,
        }
    }

    #[test]
    fn test_detect_unwrap_without_safety() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn example() {\n    let x = some_result.unwrap();\n}\n",
        )
        .unwrap();

        let config =
            make_regex_config("RUST-REGEX-001", r"unwrap\(\)", Some("**/*.rs"), None, None);
        let check = RegexCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_unwrap_with_safety_comment_allowed() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn example() {\n    // SAFETY: invariant guaranteed by caller\n    let x = some_result.unwrap();\n}\n",
        ).unwrap();

        let config = make_regex_config(
            "RUST-REGEX-001",
            r"unwrap\(\)",
            Some("**/*.rs"),
            Some(r"//\s*SAFETY:"),
            None,
        );
        let check = RegexCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(result.passed, "Should pass when unwrap has SAFETY comment");
    }

    #[test]
    fn test_unsafe_block_without_safety_comment() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn example() {\n    unsafe {\n        // dangerous stuff\n    }\n}\n",
        )
        .unwrap();

        let config = make_regex_config(
            "RUST-REGEX-002",
            r"unsafe\s*\{",
            Some("**/*.rs"),
            None,
            Some(r"//\s*SAFETY:"),
        );
        let check = RegexCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_unsafe_block_with_safety_comment() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "fn example() {\n    // SAFETY: this is safe because...\n    unsafe {\n        // dangerous stuff\n    }\n}\n",
        ).unwrap();

        let config = make_regex_config(
            "RUST-REGEX-002",
            r"unsafe\s*\{",
            Some("**/*.rs"),
            None,
            Some(r"//\s*SAFETY:"),
        );
        let check = RegexCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(
            result.passed,
            "Should pass when SAFETY comment precedes unsafe"
        );
    }

    #[test]
    fn test_skip_non_matching_glob() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.py"), "x = unsafe_function()").unwrap();
        std::fs::write(src.join("lib.rs"), "x = unsafe_function()").unwrap();

        // Only scan .rs files
        let config = make_regex_config("TEST-001", r"unsafe_function", Some("**/*.rs"), None, None);
        let check = RegexCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
    }
}
