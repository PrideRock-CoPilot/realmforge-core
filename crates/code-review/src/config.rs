use crate::check::CheckConfig;
use crate::error::{CodeReviewError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The project-specific `.code-review.yaml` configuration file.
///
/// This file sits at the repository root and controls which languages
/// and framework-specific rules apply to this project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// The programming language(s) used in this project.
    /// Maps to standards YAML files by `language` field.
    pub languages: Vec<String>,

    /// Optional framework configurations for framework-specific rules.
    #[serde(default)]
    pub frameworks: Vec<FrameworkConfig>,

    /// Optional per-check overrides (e.g., severity bumps, exclusions).
    #[serde(default)]
    pub overrides: Vec<CheckOverride>,

    /// Optional path to the standards directory.
    /// Defaults to `code-review-standards/` relative to workspace root.
    #[serde(default = "default_standards_dir")]
    pub standards_dir: String,

    /// Optional list of check IDs to skip entirely.
    #[serde(default)]
    pub skip_checks: Vec<String>,

    /// Optional maximum severity level that will fail the review.
    /// Defaults to `REQUIRED_CHANGE` (so NOTE and SHOULD_FIX do not fail).
    #[serde(default = "default_fail_threshold")]
    pub fail_threshold: String,
}

fn default_standards_dir() -> String {
    "code-review-standards".to_string()
}

fn default_fail_threshold() -> String {
    "REQUIRED_CHANGE".to_string()
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            languages: vec!["rust".to_string()],
            frameworks: Vec::new(),
            overrides: Vec::new(),
            standards_dir: default_standards_dir(),
            skip_checks: Vec::new(),
            fail_threshold: default_fail_threshold(),
        }
    }
}

/// Framework-specific configuration (e.g., "axum", "actix", "django").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    /// Framework name — maps to a standards file named `<framework>.yaml`.
    pub name: String,
    /// Optional version constraint.
    #[serde(default)]
    pub version: Option<String>,
}

/// Per-check override for severity, exclusions, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOverride {
    /// The check ID to override.
    pub check_id: String,
    /// Optional severity override.
    #[serde(default)]
    pub severity: Option<String>,
    /// Optional reason for the override (documented in review output).
    #[serde(default)]
    pub reason: Option<String>,
    /// Optional file glob patterns to exclude from this check.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl ReviewConfig {
    /// Load and parse a `.code-review.yaml` file from the given path.
    ///
    /// Returns `None` if the file does not exist (configuration is optional).
    /// Returns an error if the file exists but is malformed.
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path).map_err(CodeReviewError::Io)?;

        let config: ReviewConfig = serde_yaml::from_str(&content)
            .map_err(|e| CodeReviewError::yaml_parse(path.display().to_string(), e.to_string()))?;

        config.validate()?;

        Ok(Some(config))
    }

    /// Validate the configuration for internal consistency.
    fn validate(&self) -> Result<()> {
        if self.languages.is_empty() {
            return Err(CodeReviewError::config(
                "At least one language must be specified in .code-review.yaml",
            ));
        }

        for lang in &self.languages {
            if lang.trim().is_empty() {
                return Err(CodeReviewError::config("Language names must not be empty"));
            }
        }

        // Validate fail_threshold
        match self.fail_threshold.as_str() {
            "BLOCKER" | "REQUIRED_CHANGE" | "SHOULD_FIX" | "NOTE" => {}
            other => {
                return Err(CodeReviewError::config(format!(
                    "Invalid fail_threshold '{}' — must be one of: BLOCKER, REQUIRED_CHANGE, SHOULD_FIX, NOTE",
                    other,
                )));
            }
        }

        Ok(())
    }

    /// Returns the list of language standards files to load.
    pub fn language_files(&self) -> Vec<String> {
        self.languages
            .iter()
            .map(|lang| format!("{}.yaml", lang))
            .collect()
    }

    /// Returns the list of framework standards files to load.
    pub fn framework_files(&self) -> Vec<String> {
        self.frameworks
            .iter()
            .map(|fw| format!("{}.yaml", fw.name))
            .collect()
    }

    /// Apply overrides to a check configuration, returning the modified config.
    pub fn apply_overrides(&self, config: CheckConfig) -> CheckConfig {
        let mut config = config;
        for override_ in &self.overrides {
            if override_.check_id == config.id {
                if let Some(ref severity_str) = override_.severity {
                    let new_severity: std::result::Result<crate::severity::Severity, _> =
                        serde_yaml::from_str(severity_str);
                    if let Ok(sev) = new_severity {
                        config.severity = sev;
                    }
                }
            }
        }
        config
    }

    /// Returns `true` if the given check ID should be skipped.
    pub fn should_skip(&self, check_id: &str) -> bool {
        self.skip_checks.iter().any(|c| c == check_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-review.yaml");
        let yaml = r#"
languages:
  - rust
  - python
standards_dir: "code-review-standards"
fail_threshold: "BLOCKER"
"#;
        std::fs::write(&config_path, yaml).unwrap();
        let config = ReviewConfig::load(&config_path).unwrap().unwrap();
        assert_eq!(config.languages, vec!["rust", "python"]);
        assert_eq!(config.fail_threshold, "BLOCKER");
    }

    #[test]
    fn test_load_missing_config_returns_none() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-review.yaml");
        let result = ReviewConfig::load(&config_path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_empty_languages_fails() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-review.yaml");
        let yaml = "languages: []\n";
        std::fs::write(&config_path, yaml).unwrap();
        let result = ReviewConfig::load(&config_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one language"));
    }

    #[test]
    fn test_load_invalid_threshold_fails() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".code-review.yaml");
        let yaml = "languages:\n  - rust\nfail_threshold: \"INVALID\"\n";
        std::fs::write(&config_path, yaml).unwrap();
        let result = ReviewConfig::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config() {
        let config = ReviewConfig::default();
        assert_eq!(config.languages, vec!["rust"]);
        assert_eq!(config.fail_threshold, "REQUIRED_CHANGE");
        assert!(config.skip_checks.is_empty());
    }
}
