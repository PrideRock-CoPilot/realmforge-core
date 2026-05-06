// ─────────────────────────────────────────────
// error.rs — Intake Engine Error Types
// ─────────────────────────────────────────────
// Typed errors for all intake engine operations.
// No stringly-typed errors — every failure mode
// has a named variant with context.
// ─────────────────────────────────────────────

use thiserror::Error;

/// Errors that can occur during intake engine operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum IntakeEngineError {
    #[error("Tree definition parse error: {0}")]
    ParseError(String),

    #[error("Tree validation error: {0}")]
    ValidationError(String),

    #[error("Condition evaluation error: {0}")]
    ConditionError(String),

    #[error("Unknown question referenced: {0}")]
    UnknownQuestion(String),

    #[error("Unknown answer value: {0}")]
    UnknownAnswer(String),

    #[error("Tree exceeds maximum depth of {max_depth}: reached {actual}")]
    MaxDepthExceeded { max_depth: usize, actual: usize },

    #[error("Tree has no question with id '{0}'")]
    QuestionNotFound(String),

    #[error(
        "Answer type mismatch for question '{question_id}': expected {expected}, got {actual}"
    )]
    AnswerTypeMismatch {
        question_id: String,
        expected: String,
        actual: String,
    },

    #[error("Feature mapping condition references unknown question '{0}'")]
    MappingConditionUnknown(String),

    #[error("Feature mapping condition references unknown answer '{0}'")]
    MappingConditionAnswer(String),

    #[error("Meta mapping condition evaluation failed: {0}")]
    MetaMappingError(String),

    #[error("Tree version is not compatible: {0}")]
    VersionIncompatible(String),

    #[error("Tree must have at least one question")]
    EmptyTree,

    #[error("Tree requires root question to be marked with is_root: true")]
    MissingRootQuestion,

    #[error("Question '{0}' references condition with unknown question_id")]
    ConditionQuestionNotFound(String),
}
