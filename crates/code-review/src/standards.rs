use crate::check::CheckConfig;
use crate::config::ReviewConfig;
use crate::error::{CodeReviewError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A standards file containing a set of checks for a specific language or context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardsFile {
    /// The language this standards file applies to ("*" for language-agnostic).
    pub language: String,
    /// Version of the standards.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// List of check definitions.
    pub checks: Vec<CheckConfig>,
}

/// Registry of all loaded standards for a given project configuration.
#[derive(Debug, Clone)]
pub struct StandardsRegistry {
    /// All loaded check configurations, with overrides applied.
    pub checks: Vec<CheckConfig>,
    /// The language-agnostic base checks.
    pub base_checks: Vec<CheckConfig>,
    /// Language-specific checks.
    pub language_checks: Vec<CheckConfig>,
    /// Framework-specific checks (if any).
    pub framework_checks: Vec<CheckConfig>,
    /// The config that produced this registry.
    pub config: ReviewConfig,
}

impl StandardsRegistry {
    /// Load all standards for the given project configuration.
    ///
    /// Loads in this order:
    /// 1. Language-agnostic base standards (`language-agnostic.yaml`)
    /// 2. Per-language standards files (e.g., `rust.yaml`, `python.yaml`)
    /// 3. Per-framework standards files (e.g., `axum.yaml`, `django.yaml`)
    ///
    /// Overrides from the config are applied to each check.
    pub fn load(config: &ReviewConfig, workspace_root: &Path) -> Result<Self> {
        let standards_dir = workspace_root.join(&config.standards_dir);

        if !standards_dir.exists() {
            return Err(CodeReviewError::StandardsDirNotFound {
                path: standards_dir.display().to_string(),
            });
        }

        // 1. Load language-agnostic base
        let base_path = standards_dir.join("language-agnostic.yaml");
        let base_checks = if base_path.exists() {
            let sf = load_single_standards_file(&base_path)?;
            sf.checks
        } else {
            Vec::new()
        };

        // 2. Load language-specific standards
        let mut language_checks = Vec::new();
        for lang in &config.languages {
            let lang_path = standards_dir.join(format!("{}.yaml", lang));
            if lang_path.exists() {
                let sf = load_single_standards_file(&lang_path)?;
                language_checks.extend(sf.checks);
            } else {
                return Err(CodeReviewError::StandardsNotFound {
                    language: lang.clone(),
                });
            }
        }

        // 3. Load framework-specific standards
        let mut framework_checks = Vec::new();
        for fw_file in config.framework_files() {
            let fw_path = standards_dir.join(&fw_file);
            if fw_path.exists() {
                let sf = load_single_standards_file(&fw_path)?;
                framework_checks.extend(sf.checks);
            }
            // Framework files are optional — no error if missing
        }

        // 4. Merge all checks, applying overrides and filtering skipped
        let mut all_checks: Vec<CheckConfig> = Vec::new();
        all_checks.extend(base_checks.clone());
        all_checks.extend(language_checks.clone());
        all_checks.extend(framework_checks.clone());

        // Apply overrides
        all_checks = all_checks
            .into_iter()
            .map(|c| config.apply_overrides(c))
            .collect();

        // Filter skipped
        all_checks.retain(|c| !config.should_skip(&c.id));

        if all_checks.is_empty() {
            return Err(CodeReviewError::NoApplicableChecks {
                language: config.languages.join(", "),
            });
        }

        Ok(Self {
            checks: all_checks,
            base_checks,
            language_checks,
            framework_checks,
            config: config.clone(),
        })
    }

    /// Returns all checks of a specific type.
    pub fn checks_of_type(&self, check_type: &str) -> Vec<&CheckConfig> {
        let check_type = check_type.to_lowercase();
        self.checks
            .iter()
            .filter(|c| {
                let ct = format!("{:?}", c.check_type).to_lowercase();
                ct == check_type || ct.starts_with(&check_type)
            })
            .collect()
    }

    /// Returns the total number of checks loaded.
    pub fn check_count(&self) -> usize {
        self.checks.len()
    }
}

/// Load and parse a single standards YAML file.
fn load_single_standards_file(path: &Path) -> Result<StandardsFile> {
    let content = std::fs::read_to_string(path).map_err(CodeReviewError::Io)?;

    let sf: StandardsFile = serde_yaml::from_str(&content)
        .map_err(|e| CodeReviewError::yaml_parse(path.display().to_string(), e.to_string()))?;

    // Validate checks have IDs
    for check in &sf.checks {
        if check.id.is_empty() {
            return Err(CodeReviewError::invalid_check(
                "(unknown)",
                "Check ID must not be empty",
            ));
        }
    }

    Ok(sf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_standards(dir: &TempDir) {
        // Language-agnostic
        let agnostic = r#"
language: "*"
version: "1.0"
description: "Base checks"
checks:
  - id: ALL-PROP-001
    name: "File Size"
    description: "No file exceeds 500 lines"
    check_type: file_property
    property: line_count
    max: 500
    warning_at: 300
    file_pattern: "**/*"
    severity: REQUIRED_CHANGE
"#;

        // Rust
        let rust = r#"
language: rust
version: "1.0"
description: "Rust checks"
checks:
  - id: RUST-EXEC-001
    name: "Clippy"
    description: "Run clippy"
    check_type: executable
    command: ["cargo", "clippy"]
    working_dir: workspace_root
    timeout_secs: 120
    severity: BLOCKER
"#;

        let standards_dir = dir.path().join("code-review-standards");
        std::fs::create_dir_all(&standards_dir).unwrap();
        std::fs::write(standards_dir.join("language-agnostic.yaml"), agnostic).unwrap();
        std::fs::write(standards_dir.join("rust.yaml"), rust).unwrap();
    }

    #[test]
    fn test_load_standards_success() {
        let dir = TempDir::new().unwrap();
        create_test_standards(&dir);

        let config = ReviewConfig {
            languages: vec!["rust".to_string()],
            ..Default::default()
        };

        let registry = StandardsRegistry::load(&config, dir.path()).unwrap();
        assert_eq!(registry.check_count(), 2);
    }

    #[test]
    fn test_load_standards_dir_not_found() {
        let dir = TempDir::new().unwrap();
        let config = ReviewConfig::default();
        let result = StandardsRegistry::load(&config, dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_load_standards_language_not_found() {
        let dir = TempDir::new().unwrap();
        let standards_dir = dir.path().join("code-review-standards");
        std::fs::create_dir_all(&standards_dir).unwrap();
        // Only language-agnostic, no rust.yaml
        let agnostic = r#"
language: "*"
version: "1.0"
description: "Base"
checks: []
"#;
        std::fs::write(standards_dir.join("language-agnostic.yaml"), agnostic).unwrap();

        let config = ReviewConfig {
            languages: vec!["rust".to_string()],
            ..Default::default()
        };

        let result = StandardsRegistry::load(&config, dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            CodeReviewError::StandardsNotFound { language } => {
                assert_eq!(language, "rust");
            }
            other => panic!("Expected StandardsNotFound, got: {}", other),
        }
    }
}
