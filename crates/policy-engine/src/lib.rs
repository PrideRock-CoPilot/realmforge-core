//! Policy engine for RealmForge governance.
//!
//! Enforces authorization rules for actors, sessions, skills, and commands.
//! Policy decisions are logged for observability and include corrective actions
//! when denials occur.

mod checks;
mod decision;
mod evaluator;

// Re-export public API
pub use checks::{authorize_command, authorize_command_action};
pub use decision::{
    CorrectiveAction, DenialCode, PolicyChainResult, PolicyDecision, PolicyDenial,
};
pub use evaluator::{authorize_action, evaluate_policy_chain, PolicyChainEvaluator};
