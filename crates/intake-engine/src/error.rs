//! Error types for the intake engine.
//!
//! This module defines all error conditions that can occur during intake processing,
//! including validation errors, logic errors, and template issues.

use thiserror::Error;

/// Errors that can occur during intake processing.
#[derive(Debug, Error)]
pub enum IntakeError {
    /// Template validation failed
    #[error("Template validation failed: {0}")]
    ValidationError(String),

    /// Conditional logic evaluation error
    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),

    /// Question not found in template
    #[error("Question not found: {0}")]
    QuestionNotFound(String),

    /// Invalid answer for question type
    #[error("Invalid answer for question {question_id}: {reason}")]
    InvalidAnswer {
        question_id: String,
        reason: String,
    },

    /// Circular dependency detected in decision tree
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Invalid template structure
    #[error("Invalid template structure: {0}")]
    InvalidTemplate(String),

    /// Session state error
    #[error("Session state error: {0}")]
    SessionState(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Result type for intake engine operations
pub type Result<T> = std::result::Result<T, IntakeError>;
