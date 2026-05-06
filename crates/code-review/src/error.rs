use thiserror::Error;

/// Errors that can occur during code review execution.
#[derive(Error, Debug)]
pub enum CodeReviewError {
    /// Configuration parsing or validation error.
    #[error("Config error: {0}")]
    Config(String),

    /// I/O error reading files or running commands.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parse error in standards or config files.
    #[error("YAML parse error in {file}: {message}")]
    YamlParse { file: String, message: String },

    /// Regex compilation error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Glob pattern compilation error.
    #[error("Glob pattern error: {pattern}: {message}")]
    GlobPattern { pattern: String, message: String },

    /// External command execution failure.
    #[error("Execution error for '{command}': {details}")]
    Execution { command: String, details: String },

    /// Command timed out.
    #[error("Check '{check_id}' timed out after {seconds}s")]
    Timeout { check_id: String, seconds: u64 },

    /// Invalid check definition in standards.
    #[error("Invalid check definition for '{check_id}': {reason}")]
    InvalidCheck { check_id: String, reason: String },

    /// Standards not found for the specified language.
    #[error("Standards not found for language '{language}'")]
    StandardsNotFound { language: String },

    /// Standards directory not found.
    #[error("Standards directory not found: {path}")]
    StandardsDirNotFound { path: String },

    /// No checks matched the requested scope.
    #[error("No applicable checks found for language '{language}'")]
    NoApplicableChecks { language: String },
}

impl CodeReviewError {
    /// Create a YAML parse error.
    pub fn yaml_parse(file: impl Into<String>, message: impl Into<String>) -> Self {
        Self::YamlParse {
            file: file.into(),
            message: message.into(),
        }
    }

    /// Create a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create an execution error.
    pub fn execution(command: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Execution {
            command: command.into(),
            details: details.into(),
        }
    }

    /// Create a timeout error.
    pub fn timeout(check_id: impl Into<String>, seconds: u64) -> Self {
        Self::Timeout {
            check_id: check_id.into(),
            seconds,
        }
    }

    /// Create an invalid check error.
    pub fn invalid_check(check_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidCheck {
            check_id: check_id.into(),
            reason: reason.into(),
        }
    }

    /// Create a glob pattern error.
    pub fn glob_pattern(pattern: impl Into<String>, message: impl Into<String>) -> Self {
        Self::GlobPattern {
            pattern: pattern.into(),
            message: message.into(),
        }
    }
}

/// Convenience alias for `Result<T, CodeReviewError>`.
pub type Result<T> = std::result::Result<T, CodeReviewError>;
