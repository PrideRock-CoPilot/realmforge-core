use serde::{Deserialize, Serialize};

use crate::types::{PolicyDecision, PolicyDenial};

/// Result of evaluating a policy chain — collects all denials.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyChainResult {
    pub allowed: bool,
    pub denials: Vec<PolicyDenial>,
}

/// Evaluate a chain of policy checks in order. Short-circuits on first denial.
pub fn evaluate_policy_chain(checks: Vec<Box<dyn FnOnce() -> PolicyDecision>>) -> PolicyDecision {
    for check in checks {
        let decision = check();
        if !decision.allowed {
            return decision;
        }
    }
    PolicyDecision::allow()
}

/// Runs all policy checks in sequence and collects every denial.
/// Unlike `evaluate_policy_chain`, this does NOT short-circuit on the first denial.
pub struct PolicyChainEvaluator;

impl PolicyChainEvaluator {
    /// Evaluate all checks, collecting every denial encountered.
    /// The result is `allowed: false` if any check was denied, with all denials listed.
    pub fn evaluate(checks: Vec<Box<dyn FnOnce() -> PolicyDecision>>) -> PolicyChainResult {
        let mut denials = Vec::new();
        for check in checks {
            let decision = check();
            if let Some(denial) = decision.denial {
                denials.push(denial);
            }
        }
        PolicyChainResult {
            allowed: denials.is_empty(),
            denials,
        }
    }
}
