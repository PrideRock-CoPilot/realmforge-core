//! Intake Engine - Pure logic for structured intake processing
//!
//! This crate provides the core logic for the RealmForge structured intake system.
//! It is a pure logic crate with no IO dependencies - all persistence and external
//! integrations are handled by the control-service layer.
//!
//! # Architecture
//!
//! The intake engine follows ADR-001 Decision 1: it is a separate crate with pure
//! logic and no IO. The layers are:
//!
//! ```text
//! intake-engine (this crate) → control-service → control-store → PostgreSQL
//! ```
//!
//! # Core Components
//!
//! - **Types**: All data structures (Template, Question, Answer, Session)
//! - **Evaluator**: Conditional logic evaluation
//! - **Navigator**: Decision tree traversal
//! - **Validator**: Template validation
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use intake_engine::*;
//!
//! // Create a session
//! let template_id = TemplateId(uuid::Uuid::new_v4());
//! let root_question = QuestionId::from("Q1");
//! let mut session = IntakeSession::new(template_id, root_question);
//!
//! // Get the next question
//! let template = /* load from store */
//! # Template {
//! #     id: template_id,
//! #     name: "Test".to_string(),
//! #     description: "Test".to_string(),
//! #     version: "1.0".to_string(),
//! #     root_question: root_question.clone(),
//! #     questions: std::collections::HashMap::new(),
//! #     created_at: chrono::Utc::now(),
//! #     updated_at: chrono::Utc::now(),
//! #     is_ai_generated: false,
//! #     admin_reviewed: true,
//! # };
//! let next_question = get_next_question(&template, &session)?;
//!
//! // Validate a template
//! let validation_result = validate_template(&template)?;
//! assert!(validation_result.is_valid);
//! # Ok::<(), intake_engine::IntakeError>(())
//! ```

// Module declarations
pub mod error;
pub mod evaluator;
pub mod navigator;
pub mod types;
pub mod validator;

// Re-export primary types and functions for convenience
pub use error::{IntakeError, Result};
pub use evaluator::evaluate_condition;
pub use navigator::{derive_modules_and_personas, get_next_question};
pub use types::{
    Answer, Branch, Condition, IntakeSession, Question, QuestionId, QuestionType, Response,
    SessionId, SessionStatus, Template, TemplateId,
};
pub use validator::{validate_template, ValidationResult};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
