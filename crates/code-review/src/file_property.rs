use crate::check::{Check, CheckConfig, CheckResult, CheckType, FileLocation, Finding};
use crate::error::{CodeReviewError, Result};
use crate::severity::Severity;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use std::time::Instant;

/// A check that validates properties of files (e.g., line count, encoding).
///
/// Currently supported properties:
/// - `line_count`: Validates the number of lines in a file.
pub struct FilePropertyCheck {
    config: CheckConfig,
    file_glob: Option<GlobSet>,
}

impl FilePropertyCheck {
    /// Create a new file property check from configuration.
    pub fn new(config: CheckConfig) -> Result<Self> {
        let property = config.property.as_deref().unwrap_or("");
        match property {
            "line_count" => {} // supported
            other => {
                return Err(CodeReviewError::invalid_check(
                    &config.id,
                    format!(
                        "Unsupported file property '{}'. Supported: line_count",
                        other
                    ),
                ));
            }
        }

        let file_glob = if let Some(ref pattern) = config.file_pattern {
            let mut builder = GlobSetBuilder::new();
            let glob = Glob::new(pattern)
                .map_err(|e| CodeReviewError::glob_pattern(pattern.clone(), e.to_string()))?;
            builder.add(glob);
            let glob_set = builder
                .build()
                .map_err(|e| CodeReviewError::glob_pattern(pattern.clone(), e.to_string()))?;
            Some(glob_set)
        } else {
            None
        };

        Ok(Self { config, file_glob })
    }

    /// Check if a file path matches the file glob pattern.
    fn matches_glob(&self, file_path: &Path) -> bool {
        match &self.file_glob {
            Some(glob_set) => glob_set.is_match(file_path),
            None => true,
        }
    }

    /// Count the number of lines in a file.
    fn count_lines(path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let count = content.lines().count();
        // If the file is empty but content exists, at minimum 1 line
        Ok(if content.is_empty() { 0 } else { count })
    }
}

impl Check for FilePropertyCheck {
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
        CheckType::FileProperty
    }

    fn execute(&self, workspace_root: &Path) -> Result<CheckResult> {
        let start = Instant::now();
        let mut findings = Vec::new();

        let max_val = self.config.max.unwrap_or(usize::MAX);
        let warning_at = self.config.warning_at.unwrap_or(max_val);

        // Walk all files
        let walk_root = workspace_root.to_path_buf();
        for entry in walkdir::WalkDir::new(&walk_root)
            .into_iter()
            .filter_entry(move |e| {
                // Always include the root directory
                if e.path() == walk_root {
                    return true;
                }
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

            let property = self.config.property.as_deref().unwrap_or("");

            match property {
                "line_count" => {
                    let line_count = match Self::count_lines(file_path) {
                        Ok(count) => count,
                        Err(_) => continue, // Skip binary/unreadable files
                    };

                    if line_count > max_val {
                        let relative_path =
                            file_path.strip_prefix(workspace_root).unwrap_or(file_path);

                        let severity = if line_count > max_val {
                            self.config.severity
                        } else {
                            Severity::ShouldFix
                        };

                        let finding = Finding::new(
                            &self.config.id,
                            &self.config.name,
                            severity,
                            format!(
                                "File has {} lines, which exceeds the maximum of {}",
                                line_count, max_val,
                            ),
                        )
                        .with_location(FileLocation {
                            file: relative_path.display().to_string(),
                            line: None,
                            column: None,
                        })
                        .with_suggestion(format!(
                            "Consider refactoring this file into smaller modules. Target: ≤{} lines.",
                            warning_at,
                        ));

                        findings.push(finding);
                    } else if line_count > warning_at {
                        let relative_path =
                            file_path.strip_prefix(workspace_root).unwrap_or(file_path);

                        let finding = Finding::new(
                            &self.config.id,
                            &self.config.name,
                            Severity::ShouldFix,
                            format!(
                                "File has {} lines (warning threshold: {})",
                                line_count, warning_at,
                            ),
                        )
                        .with_location(FileLocation {
                            file: relative_path.display().to_string(),
                            line: None,
                            column: None,
                        })
                        .with_suggestion(format!(
                            "File is approaching the maximum of {} lines.",
                            max_val,
                        ));

                        findings.push(finding);
                    }
                }
                _ => {
                    // Should not happen — validated in constructor
                    return Err(CodeReviewError::invalid_check(
                        &self.config.id,
                        format!("Unknown property '{}'", property),
                    ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_property_config(
        id: &str,
        property: &str,
        max: usize,
        warning_at: usize,
        file_pattern: Option<&str>,
    ) -> CheckConfig {
        CheckConfig {
            id: id.to_string(),
            name: format!("Check {}", id),
            description: "File property test".to_string(),
            check_type: CheckType::FileProperty,
            severity: Severity::RequiredChange,
            file_pattern: file_pattern.map(|s| s.to_string()),
            command: None,
            working_dir: None,
            timeout_secs: None,
            pattern: None,
            allowed_pattern: None,
            required_above: None,
            property: Some(property.to_string()),
            max: Some(max),
            warning_at: Some(warning_at),
        }
    }

    #[test]
    fn test_file_size_within_limit() {
        let dir = TempDir::new().unwrap();
        let content = "line1\nline2\nline3\n";
        let file_path = dir.path().join("small.rs");
        std::fs::write(&file_path, content).unwrap();

        let config = make_property_config("ALL-PROP-001", "line_count", 10, 5, Some("**/*.rs"));
        let check = FilePropertyCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_file_size_exceeds_max() {
        let dir = TempDir::new().unwrap();
        let content = (0..600)
            .map(|i| format!("line_{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let file_path = dir.path().join("large.rs");
        std::fs::write(&file_path, content).unwrap();

        let config = make_property_config("ALL-PROP-001", "line_count", 500, 300, Some("**/*.rs"));
        let check = FilePropertyCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(!result.passed);
        assert!(result.findings[0].message.contains("600"));
    }

    #[test]
    fn test_file_size_warning_threshold() {
        let dir = TempDir::new().unwrap();
        let content = (0..400)
            .map(|i| format!("line_{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let file_path = dir.path().join("medium.rs");
        std::fs::write(&file_path, content).unwrap();

        let config = make_property_config("ALL-PROP-001", "line_count", 500, 300, Some("**/*.rs"));
        let check = FilePropertyCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(!result.passed);
        // Should find it at warning level (SHOULD_FIX)
        assert_eq!(result.findings[0].severity, Severity::ShouldFix);
    }

    #[test]
    fn test_skip_non_matching_glob() {
        let dir = TempDir::new().unwrap();
        let content = (0..600)
            .map(|i| format!("line_{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let py_file = dir.path().join("large.py");
        std::fs::write(&py_file, content).unwrap();
        let rs_file = dir.path().join("small.rs");
        std::fs::write(&rs_file, "fn main() {}").unwrap();

        // Only check .rs files
        let config = make_property_config("ALL-PROP-001", "line_count", 500, 300, Some("**/*.rs"));
        let check = FilePropertyCheck::new(config).unwrap();
        let result = check.execute(dir.path()).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_unsupported_property_fails() {
        let config = make_property_config("TEST-001", "encoding", 0, 0, None);
        let result = FilePropertyCheck::new(config);
        assert!(result.is_err());
    }
}
