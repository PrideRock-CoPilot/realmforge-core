// ─────────────────────────────────────────────
// intake-engine — Decision Tree Evaluation Engine
// ─────────────────────────────────────────────
// A pure, stateless engine for walking decision
// trees during the intake process. Has zero
// dependencies on RealmForge crate types.
//
// # Public API
//
// - `tree::TreeDefinition` — Decision tree definition
// - `tree::Question` — A single question node
// - `tree::AnswerValue` — User answer values
// - `tree::TreeWalkResult` — Walk completion result
// - `engine::next_question()` — Get next question
// - `engine::walk_tree()` — Walk entire tree
// - `condition::evaluate_condition()` — Evaluate expression
// - `mapper::apply_all_mappings()` — Map answers to modules
// - `validate::validate_tree()` — Validate tree definition
// - `error::IntakeEngineError` — Typed errors
// ─────────────────────────────────────────────

pub mod condition;
pub mod engine;
pub mod error;
pub mod mapper;
pub mod tree;
pub mod validate;

// Re-export public types at crate level for convenience.
pub use engine::{next_question, walk_tree};
pub use error::IntakeEngineError;
pub use mapper::apply_all_mappings;
pub use tree::{
    AnswerValue, ConditionExpression, FeatureMapping, MetaMapping, PlanStageHandoff, Question,
    QuestionCondition, QuestionOption, QuestionType, TreeDefinition, TreeWalkResult,
};
pub use validate::validate_tree;
