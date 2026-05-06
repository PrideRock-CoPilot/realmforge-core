use crate::error::Result;

use crate::severity::Severity;
use std::path::Path;
use std::time::Duration;

/// The type of automated check to perform.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    /// An external command-line tool (e.g., cargo clippy, ruff).
    Executable,
    /// A regex pattern match against source file content.
    Regex,
    /// A property check on files (e.g., file size, encoding).
    FileProperty,
}

impl std::fmt::Display for CheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executable => write!(f, "executable"),
            Self::Regex => write!(f, "regex"),
            Self::FileProperty => write!(f, "file_property"),
        }
    }
}

/// Serialisable configuration for a single check as defined in standards YAML.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckConfig {
    /// Unique check identifier (e.g., RUST-EXEC-001).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what the check validates.
    pub description: String,
    /// The type of check.
    pub check_type: CheckType,
    /// Severity level when the check fails.
    pub severity: Severity,
    /// File glob pattern to scope this check.
    #[serde(default)]
    pub file_pattern: Option<String>,
    // --- Executable-specific ---
    /// Command and arguments (executable checks).
    #[serde(default)]
    pub command: Option<Vec<String>>,
    /// Working directory hint (workspace_root or repo_root).
    #[serde(default)]
    pub working_dir: Option<String>,
    /// Timeout in seconds (executable checks).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    // --- Regex-specific ---
    /// Regex pattern to find (regex checks).
    #[serde(default)]
    pub pattern: Option<String>,
    /// Optional regex pattern that, if found on a preceding line, makes the finding pass.
    #[serde(default)]
    pub allowed_pattern: Option<String>,
    /// Optional pattern that must appear on a line immediately preceding the match.
    #[serde(default)]
    pub required_above: Option<String>,
    // --- File property-specific ---
    /// Property name to check (e.g., line_count).
    #[serde(default)]
    pub property: Option<String>,
    /// Maximum allowed value (e.g., max lines).
    #[serde(default)]
    pub max: Option<usize>,
    /// Warning threshold value.
    #[serde(default)]
    pub warning_at: Option<usize>,
}

/// A single finding produced by a check.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckResult {
    /// The check identifier that produced this result.
    pub check_id: String,
    /// Whether the check passed (no findings) or failed.
    pub passed: bool,
    /// Individual findings from the check.
    pub findings: Vec<Finding>,
    /// How long the check took to execute.
    pub duration: Duration,
    /// Any error that occurred during execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CheckResult {
    /// Create a passing result with no findings.
    pub fn passed(check_id: impl Into<String>, duration: Duration) -> Self {
        Self {
            check_id: check_id.into(),
            passed: true,
            findings: Vec::new(),
            duration,
            error: None,
        }
    }

    /// Create a failing result with findings.
    pub fn failed(check_id: impl Into<String>, findings: Vec<Finding>, duration: Duration) -> Self {
        Self {
            check_id: check_id.into(),
            passed: false,
            findings,
            duration,
            error: None,
        }
    }

    /// Create an error result.
    pub fn error(
        check_id: impl Into<String>,
        error: impl Into<String>,
        duration: Duration,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            passed: false,
            findings: Vec::new(),
            duration,
            error: Some(error.into()),
        }
    }
}

/// Location within a source file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileLocation {
    /// File path relative to workspace root.
    pub file: String,
    /// Optional line number (1-based).
    pub line: Option<usize>,
    /// Optional column number (1-based).
    pub column: Option<usize>,
}

/// A single code review finding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    /// Check identifier that produced this finding.
    pub check_id: String,
    /// Human-readable check name.
    pub check_name: String,
    /// Severity of this finding.
    pub severity: Severity,
    /// Description of the finding.
    pub message: String,
    /// Location in the source code (if applicable).
    pub location: Option<FileLocation>,
    /// Optional suggestion for fixing the finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl Finding {
    /// Create a new finding.
    pub fn new(
        check_id: impl Into<String>,
        check_name: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            check_name: check_name.into(),
            severity,
            message: message.into(),
            location: None,
            suggestion: None,
        }
    }

    /// Attach a file location to this finding.
    pub fn with_location(mut self, location: FileLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Attach a suggestion to this finding.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// The core trait that all check implementations must implement.
pub trait Check: Send + Sync {
    /// Returns the unique identifier for this check.
    fn id(&self) -> &str;
    /// Returns the human-readable name of this check.
    fn name(&self) -> &str;
    /// Returns the severity level when this check fails.
    fn severity(&self) -> Severity;
    /// Returns the type of check.
    fn check_type(&self) -> CheckType;
    /// Execute the check against the given workspace root.
    fn execute(&self, workspace_root: &Path) -> Result<CheckResult>;
}
