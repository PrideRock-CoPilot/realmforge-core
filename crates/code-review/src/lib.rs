//! # Code Review Module
//!
//! Automated code review engine for RealmForge.
//!
//! This crate provides:
//! - A **standards registry** that loads language-specific and language-agnostic
//!   code review standards from YAML files.
//! - **Automated check execution** supporting three check types:
//!   - `ExecutableCheck` — runs external CLI tools (e.g., `cargo clippy`, `ruff`)
//!   - `RegexCheck` — scans source files for regex patterns (e.g., forbidden patterns)
//!   - `FilePropertyCheck` — validates file properties (e.g., line count limits)
//! - A **review report** aggregator that produces structured output with
//!   findings, severity classification, and pass/fail status.
//!
//! ## Architecture
//!
//! ```text
//! .code-review.yaml  ──►  ReviewConfig
//!                              │
//!                              ▼
//! code-review-standards/  ──►  StandardsRegistry  ──►  Vec<CheckConfig>
//!   ├── language-agnostic.yaml                           │
//!   ├── rust.yaml                                        │
//!   └── python.yaml                                      ▼
//!                                                  Check implementations
//!                                                   (exec, regex, prop)
//!                                                        │
//!                                                        ▼
//!                                                  ReviewReport
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use code_review::{ReviewConfig, StandardsRegistry};
//! use code_review::ReviewReport;
//! use std::path::Path;
//!
//! // Load config
//! let config = ReviewConfig::load(Path::new(".code-review.yaml"))
//!     .unwrap()
//!     .unwrap_or_default();
//!
//! // Load standards
//! let registry = StandardsRegistry::load(&config, Path::new(".")).unwrap();
//!
//! // Execute checks...
//! ```
//!
//! ## Security
//!
//! The `ExecutableCheck` enforces a command whitelist to prevent arbitrary
//! code execution through configuration files. Only approved binaries
//! (cargo, rustc, ruff, eslint, etc.) can be launched.

pub mod check;
pub mod config;
pub mod error;
pub mod executable;
pub mod file_property;
pub mod regex;
pub mod report;
pub mod severity;
pub mod standards;

// Re-exports for public API
pub use check::{Check, CheckConfig, CheckResult, CheckType, FileLocation, Finding};
pub use config::ReviewConfig;
pub use error::{CodeReviewError, Result};
pub use executable::ExecutableCheck;
pub use file_property::FilePropertyCheck;
pub use regex::RegexCheck;
pub use report::{ConfigSummary, FindingsBySeverity, ReviewReport, ReviewStatus, ReviewSummary};
pub use severity::Severity;
pub use standards::{StandardsFile, StandardsRegistry};

use std::path::Path;
use std::time::Instant;

/// Run a complete code review for the given workspace.
///
/// This is the main entry point for the code review module. It:
/// 1. Loads the project configuration from `.code-review.yaml`
/// 2. Loads all applicable standards from `code-review-standards/`
/// 3. Instantiates and runs all applicable checks
/// 4. Aggregates results into a `ReviewReport`
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory.
/// * `config_path` - Optional explicit path to `.code-review.yaml`. If `None`,
///   defaults to `workspace_root/.code-review.yaml`.
///
/// # Returns
///
/// A `ReviewReport` with all findings and status.
pub fn run_review(workspace_root: &Path, config_path: Option<&Path>) -> ReviewReport {
    let start = Instant::now();

    // 1. Load config
    let config_path = config_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| workspace_root.join(".code-review.yaml"));

    let config = match ReviewConfig::load(&config_path) {
        Ok(Some(c)) => c,
        Ok(None) => ReviewConfig::default(),
        Err(e) => {
            // Return an error report
            return ReviewReport {
                status: report::ReviewStatus::Error,
                findings: vec![Finding::new(
                    "SYSTEM-CONFIG",
                    "Config Load",
                    Severity::Blocker,
                    format!("Failed to load config: {}", e),
                )],
                summary: report::ReviewSummary {
                    total_checks: 0,
                    passed_checks: 0,
                    failed_checks: 0,
                    error_checks: 1,
                    total_findings: 1,
                    findings_by_severity: report::FindingsBySeverity {
                        blockers: 1,
                        required_changes: 0,
                        should_fixes: 0,
                        notes: 0,
                    },
                    files_scanned: 0,
                },
                check_results: Vec::new(),
                config_summary: report::ConfigSummary {
                    languages: Vec::new(),
                    frameworks: Vec::new(),
                    standards_count: 0,
                    fail_threshold: "REQUIRED_CHANGE".to_string(),
                },
                duration: start.elapsed(),
            };
        }
    };

    // 2. Load standards
    let registry = match StandardsRegistry::load(&config, workspace_root) {
        Ok(r) => r,
        Err(e) => {
            return ReviewReport {
                status: report::ReviewStatus::Error,
                findings: vec![Finding::new(
                    "SYSTEM-STANDARDS",
                    "Standards Load",
                    Severity::Blocker,
                    format!("Failed to load standards: {}", e),
                )],
                summary: report::ReviewSummary {
                    total_checks: 0,
                    passed_checks: 0,
                    failed_checks: 0,
                    error_checks: 1,
                    total_findings: 1,
                    findings_by_severity: report::FindingsBySeverity {
                        blockers: 1,
                        required_changes: 0,
                        should_fixes: 0,
                        notes: 0,
                    },
                    files_scanned: 0,
                },
                check_results: Vec::new(),
                config_summary: report::ConfigSummary {
                    languages: config.languages.clone(),
                    frameworks: Vec::new(),
                    standards_count: 0,
                    fail_threshold: config.fail_threshold.clone(),
                },
                duration: start.elapsed(),
            };
        }
    };

    // 3. Execute all checks
    let mut check_results = Vec::new();

    for check_config in &registry.checks {
        let result = match check_config.check_type {
            check::CheckType::Executable => {
                match executable::ExecutableCheck::new(check_config.clone()) {
                    Ok(check) => check.execute(workspace_root),
                    Err(e) => Ok(CheckResult::error(
                        &check_config.id,
                        format!("Failed to initialize check: {}", e),
                        Instant::now().duration_since(start),
                    )),
                }
            }
            check::CheckType::Regex => match regex::RegexCheck::new(check_config.clone()) {
                Ok(check) => check.execute(workspace_root),
                Err(e) => Ok(CheckResult::error(
                    &check_config.id,
                    format!("Failed to initialize check: {}", e),
                    Instant::now().duration_since(start),
                )),
            },
            check::CheckType::FileProperty => {
                match file_property::FilePropertyCheck::new(check_config.clone()) {
                    Ok(check) => check.execute(workspace_root),
                    Err(e) => Ok(CheckResult::error(
                        &check_config.id,
                        format!("Failed to initialize check: {}", e),
                        Instant::now().duration_since(start),
                    )),
                }
            }
        };

        match result {
            Ok(r) => check_results.push(r),
            Err(e) => {
                check_results.push(CheckResult::error(
                    &check_config.id,
                    e.to_string(),
                    Instant::now().duration_since(start),
                ));
            }
        }
    }

    // 4. Build and return report
    ReviewReport::build(check_results, &config, &registry, start.elapsed())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_workspace() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create .code-review.yaml
        let config_yaml = r#"
languages:
  - rust
standards_dir: "code-review-standards"
fail_threshold: "REQUIRED_CHANGE"
"#;
        std::fs::write(dir.path().join(".code-review.yaml"), config_yaml).unwrap();

        // Create standards directory
        let standards_dir = dir.path().join("code-review-standards");
        std::fs::create_dir_all(&standards_dir).unwrap();

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
    file_pattern: "**/*.rs"
    severity: REQUIRED_CHANGE
"#;
        std::fs::write(standards_dir.join("language-agnostic.yaml"), agnostic).unwrap();

        // Rust
        let rust = r#"
language: rust
version: "1.0"
description: "Rust checks"
checks:
  - id: RUST-REGEX-001
    name: "No Bare Unwrap"
    description: "Detect unwrap()"
    check_type: regex
    pattern: "unwrap\\(\\)"
    file_pattern: "**/*.rs"
    severity: REQUIRED_CHANGE
"#;
        std::fs::write(standards_dir.join("rust.yaml"), rust).unwrap();

        // Create a small source file
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn hello() -> &'static str { \"hello\" }",
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_run_review_no_findings_clean_code() {
        let dir = setup_test_workspace();
        let report = run_review(dir.path(), None);
        // PROP check should pass (small file), REGEX should pass (no unwrap)
        assert_eq!(
            report.status,
            ReviewStatus::Passed,
            "Report: {}",
            report.one_line_summary()
        );
    }

    #[test]
    fn test_run_review_finds_unwrap() {
        let dir = setup_test_workspace();
        let src_dir = dir.path().join("src");

        // Add unwrap to the source
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn example() { let x = Ok(42).unwrap(); }",
        )
        .unwrap();

        let report = run_review(dir.path(), None);
        assert!(!report.passed(), "Should fail with unwrap finding");
        assert!(report
            .findings
            .iter()
            .any(|f| f.check_id == "RUST-REGEX-001"));
    }

    #[test]
    fn test_run_review_missing_config_defaults() {
        let dir = TempDir::new().unwrap();

        // No .code-review.yaml — will fail because standards dir doesn't exist
        let report = run_review(dir.path(), None);
        assert_eq!(report.status, ReviewStatus::Error);
        assert!(report.findings[0].message.contains("not found"));
    }
}
