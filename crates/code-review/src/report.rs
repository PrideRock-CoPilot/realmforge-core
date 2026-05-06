use crate::check::{CheckResult, Finding};

use crate::severity::Severity;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Aggregated report from running a set of code review checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    /// Overall status of the review.
    pub status: ReviewStatus,
    /// All findings from all executed checks.
    pub findings: Vec<Finding>,
    /// Summary statistics.
    pub summary: ReviewSummary,
    /// Per-check results.
    pub check_results: Vec<CheckResult>,
    /// Configuration that was used for this review.
    pub config_summary: ConfigSummary,
    /// Total duration of the review.
    pub duration: Duration,
}

/// Overall review status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    /// All checks passed — no blocking findings.
    Passed,
    /// Some checks failed with findings at or above the fail threshold.
    Failed,
    /// Some checks could not be executed (errors).
    Error,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passed => write!(f, "PASSED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Summary statistics for a review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    /// Total number of checks executed.
    pub total_checks: usize,
    /// Number of checks that passed.
    pub passed_checks: usize,
    /// Number of checks that failed.
    pub failed_checks: usize,
    /// Number of checks that encountered errors.
    pub error_checks: usize,
    /// Total number of findings.
    pub total_findings: usize,
    /// Findings broken down by severity.
    pub findings_by_severity: FindingsBySeverity,
    /// Number of files checked (approximately).
    pub files_scanned: usize,
}

/// Breakdown of findings by severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingsBySeverity {
    pub blockers: usize,
    pub required_changes: usize,
    pub should_fixes: usize,
    pub notes: usize,
}

impl FindingsBySeverity {
    fn new() -> Self {
        Self {
            blockers: 0,
            required_changes: 0,
            should_fixes: 0,
            notes: 0,
        }
    }

    fn increment(&mut self, severity: Severity) {
        match severity {
            Severity::Blocker => self.blockers += 1,
            Severity::RequiredChange => self.required_changes += 1,
            Severity::ShouldFix => self.should_fixes += 1,
            Severity::Note => self.notes += 1,
        }
    }
}

/// Summary of the configuration used for the review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Languages checked.
    pub languages: Vec<String>,
    /// Frameworks checked (if any).
    pub frameworks: Vec<String>,
    /// Number of standards files loaded.
    pub standards_count: usize,
    /// Fail threshold used.
    pub fail_threshold: String,
}

impl ReviewReport {
    /// Build a report from a set of check results and configuration.
    pub fn build(
        check_results: Vec<CheckResult>,
        config: &crate::config::ReviewConfig,
        registry: &crate::standards::StandardsRegistry,
        duration: Duration,
    ) -> Self {
        let mut all_findings = Vec::new();
        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut errors = 0usize;

        for result in &check_results {
            if result.error.is_some() {
                errors += 1;
            } else if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }
            all_findings.extend(result.findings.clone());
        }

        // Calculate files scanned (approximate — count unique files from findings)
        let mut files = std::collections::BTreeSet::new();
        for finding in &all_findings {
            if let Some(ref loc) = finding.location {
                files.insert(loc.file.clone());
            }
        }

        let mut findings_by_severity = FindingsBySeverity::new();
        for finding in &all_findings {
            findings_by_severity.increment(finding.severity);
        }

        let status = if errors > 0 {
            ReviewStatus::Error
        } else if failed > 0 {
            // Check if blocking findings exist at or above fail threshold
            let threshold = match config.fail_threshold.as_str() {
                "BLOCKER" => Severity::Blocker,
                "REQUIRED_CHANGE" => Severity::RequiredChange,
                "SHOULD_FIX" => Severity::ShouldFix,
                "NOTE" => Severity::Note,
                _ => Severity::RequiredChange,
            };

            let has_blocking = all_findings.iter().any(|f| f.severity >= threshold);

            if has_blocking {
                ReviewStatus::Failed
            } else {
                ReviewStatus::Passed
            }
        } else {
            ReviewStatus::Passed
        };

        let framework_names: Vec<String> =
            config.frameworks.iter().map(|f| f.name.clone()).collect();

        Self {
            status,
            findings: all_findings,
            summary: ReviewSummary {
                total_checks: check_results.len(),
                passed_checks: passed,
                failed_checks: failed,
                error_checks: errors,
                total_findings: findings_by_severity.blockers
                    + findings_by_severity.required_changes
                    + findings_by_severity.should_fixes
                    + findings_by_severity.notes,
                findings_by_severity,
                files_scanned: files.len(),
            },
            check_results,
            config_summary: ConfigSummary {
                languages: config.languages.clone(),
                frameworks: framework_names,
                standards_count: registry.check_count(),
                fail_threshold: config.fail_threshold.clone(),
            },
            duration,
        }
    }

    /// Returns `true` if the review passed (no blocking findings above threshold).
    pub fn passed(&self) -> bool {
        self.status == ReviewStatus::Passed
    }

    /// Returns `true` if any check had an execution error.
    pub fn has_errors(&self) -> bool {
        self.status == ReviewStatus::Error
    }

    /// Returns a human-readable one-line summary.
    pub fn one_line_summary(&self) -> String {
        format!(
            "{} — {} checks, {} passed, {} failed, {} errors, {} findings",
            self.status,
            self.summary.total_checks,
            self.summary.passed_checks,
            self.summary.failed_checks,
            self.summary.error_checks,
            self.summary.total_findings,
        )
    }

    /// Format the report as a Markdown string suitable for PR comments or console output.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Code Review Report\n\n");
        md.push_str(&format!("**Status:** {}\n\n", self.status));
        md.push_str(&format!("**Duration:** {:?}\n\n", self.duration));
        md.push_str(&format!(
            "**Languages:** {}\n\n",
            self.config_summary.languages.join(", ")
        ));

        if !self.config_summary.frameworks.is_empty() {
            md.push_str(&format!(
                "**Frameworks:** {}\n\n",
                self.config_summary.frameworks.join(", ")
            ));
        }

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "| Metric | Value |\n|--------|-------|\n\
             | Total Checks | {} |\n\
             | Passed | {} |\n\
             | Failed | {} |\n\
             | Errors | {} |\n\
             | Total Findings | {} |\n\
             | Files Scanned | {} |\n\n",
            self.summary.total_checks,
            self.summary.passed_checks,
            self.summary.failed_checks,
            self.summary.error_checks,
            self.summary.total_findings,
            self.summary.files_scanned,
        ));

        md.push_str("## Findings by Severity\n\n");
        md.push_str(&format!(
            "| Severity | Count |\n|----------|-------|\n\
             | BLOCKER | {} |\n\
             | REQUIRED CHANGE | {} |\n\
             | SHOULD FIX | {} |\n\
             | NOTE | {} |\n\n",
            self.summary.findings_by_severity.blockers,
            self.summary.findings_by_severity.required_changes,
            self.summary.findings_by_severity.should_fixes,
            self.summary.findings_by_severity.notes,
        ));

        if !self.findings.is_empty() {
            md.push_str("## Findings\n\n");
            for finding in &self.findings {
                let location = match &finding.location {
                    Some(loc) => format!("{}:{}", loc.file, loc.line.map_or(0, |l| l)),
                    None => "(general)".to_string(),
                };
                md.push_str(&format!(
                    "- **{}** [{}] {} — {}\n",
                    finding.severity, finding.check_id, finding.message, location,
                ));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ReviewConfig;
    use crate::standards::StandardsRegistry;

    // Mock a simple standards registry for testing
    fn mock_registry(config: &ReviewConfig) -> StandardsRegistry {
        StandardsRegistry {
            checks: Vec::new(),
            base_checks: Vec::new(),
            language_checks: Vec::new(),
            framework_checks: Vec::new(),
            config: config.clone(),
        }
    }

    #[test]
    fn test_report_passed_no_findings() {
        let config = ReviewConfig::default();
        let registry = mock_registry(&config);

        let results = vec![
            CheckResult::passed("RUST-EXEC-001", Duration::from_secs(1)),
            CheckResult::passed("RUST-REGEX-001", Duration::from_millis(200)),
        ];

        let report = ReviewReport::build(results, &config, &registry, Duration::from_secs(2));
        assert_eq!(report.status, ReviewStatus::Passed);
        assert!(report.passed());
    }

    #[test]
    fn test_report_failed_with_blocking_findings() {
        let config = ReviewConfig::default();
        let registry = mock_registry(&config);

        let finding = Finding::new(
            "RUST-EXEC-001",
            "Clippy",
            Severity::Blocker,
            "Clippy found warnings",
        );

        let results = vec![CheckResult::failed(
            "RUST-EXEC-001",
            vec![finding],
            Duration::from_secs(30),
        )];

        let report = ReviewReport::build(results, &config, &registry, Duration::from_secs(30));
        assert_eq!(report.status, ReviewStatus::Failed);
    }

    #[test]
    fn test_report_error_status() {
        let config = ReviewConfig::default();
        let registry = mock_registry(&config);

        let results = vec![CheckResult::error(
            "RUST-EXEC-001",
            "Command not found",
            Duration::from_secs(1),
        )];

        let report = ReviewReport::build(results, &config, &registry, Duration::from_secs(1));
        assert_eq!(report.status, ReviewStatus::Error);
        assert!(report.has_errors());
    }

    #[test]
    fn test_report_non_blocking_findings_pass() {
        let config = ReviewConfig::default();
        let registry = mock_registry(&config);

        let finding = Finding::new(
            "ALL-REGEX-001",
            "TODOs",
            Severity::Note,
            "Found a TODO comment",
        );

        let results = vec![CheckResult::failed(
            "ALL-REGEX-001",
            vec![finding],
            Duration::from_secs(1),
        )];

        let report = ReviewReport::build(results, &config, &registry, Duration::from_secs(1));
        // NOTE is below REQUIRED_CHANGE threshold, so should pass
        assert_eq!(report.status, ReviewStatus::Passed);
    }

    #[test]
    fn test_one_line_summary_format() {
        let config = ReviewConfig::default();
        let registry = mock_registry(&config);

        let results = vec![CheckResult::passed("TEST-001", Duration::from_secs(1))];

        let report = ReviewReport::build(results, &config, &registry, Duration::from_secs(1));
        let summary = report.one_line_summary();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("1 checks"));
    }
}
