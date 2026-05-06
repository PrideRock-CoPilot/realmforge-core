//! Core policy evaluation functions.
//!
//! Contains the main policy evaluation logic for checking scopes against
//! actions, including expiry, staleness, and authorization checks.

use authority_domain::{ActorScope, ApprovalState};
use chrono::{DateTime, Utc};

use crate::decision::{CorrectiveAction, DenialCode, PolicyChainResult, PolicyDecision};

/// Run all standard policy checks against a scope.
/// Logs every decision with structured context for observability.
///
/// # Arguments
/// * `scope` - Actor scope containing session, role, and permission context
/// * `action` - Action being requested (e.g., "core.create_snapshot")
/// * `mutating` - Whether the action modifies state
/// * `approval_required` - Whether this action requires approval
/// * `now` - Current timestamp for expiry/staleness checks
/// * `max_context_age_seconds` - Maximum age before context is considered stale
pub fn authorize_action(
    scope: &ActorScope,
    action: &str,
    mutating: bool,
    approval_required: bool,
    now: DateTime<Utc>,
    max_context_age_seconds: i64,
) -> PolicyDecision {
    let decision = inner_authorize_action(
        scope,
        action,
        mutating,
        approval_required,
        now,
        max_context_age_seconds,
    );
    tracing::info!(
        policy.action = action,
        policy.mutating = mutating,
        policy.approval_required = approval_required,
        policy.allowed = decision.allowed,
        policy.denial_code = ?decision.denial.as_ref().map(|d| &d.code),
        policy.denial_message = decision.denial.as_ref().map(|d| &d.message),
        actor.id = %scope.actor_id,
        session.id = %scope.session_id,
        "policy decision"
    );
    decision
}

fn inner_authorize_action(
    scope: &ActorScope,
    action: &str,
    mutating: bool,
    approval_required: bool,
    now: DateTime<Utc>,
    max_context_age_seconds: i64,
) -> PolicyDecision {
    if now > scope.expires_at {
        return PolicyDecision::deny(
            DenialCode::SessionExpired,
            "scope has expired",
            true,
            Some(CorrectiveAction::RefreshSession),
        );
    }

    let age_seconds = now
        .signed_duration_since(scope.context_updated_at)
        .num_seconds();
    if age_seconds > max_context_age_seconds {
        return PolicyDecision::deny(
            DenialCode::SessionStale,
            "scope context is stale",
            true,
            Some(CorrectiveAction::RefreshSession),
        );
    }

    if !scope.allows_action(action) {
        return PolicyDecision::deny(
            DenialCode::ActorUnauthorized,
            format!("{action} is not allowed for this scope"),
            false,
            None,
        );
    }

    if scope.requested_skill_id.is_some()
        && scope.active_skill_id.is_some()
        && scope.requested_skill_id != scope.active_skill_id
    {
        return PolicyDecision::deny(
            DenialCode::WrongSkillActive,
            "requested skill does not match active skill",
            true,
            Some(CorrectiveAction::RebindSkill),
        );
    }

    if mutating && !scope.execution_mode.may_mutate() {
        return PolicyDecision::deny(
            DenialCode::ProposalOnly,
            "scope execution mode cannot mutate state",
            true,
            Some(CorrectiveAction::SwitchExecutionMode),
        );
    }

    if approval_required && scope.approval_state != ApprovalState::Approved {
        return PolicyDecision::deny(
            DenialCode::ApprovalRequired,
            "approval is required before this action can run",
            true,
            Some(CorrectiveAction::RequestApproval),
        );
    }

    PolicyDecision::allow()
}

/// Evaluate a chain of policy checks in order. Short-circuits on first denial.
///
/// For collecting ALL denials, use `PolicyChainEvaluator::evaluate` instead.
pub fn evaluate_policy_chain(checks: Vec<Box<dyn FnOnce() -> PolicyDecision>>) -> PolicyDecision {
    for check in checks {
        let decision = check();
        if !decision.allowed {
            return decision;
        }
    }
    PolicyDecision::allow()
}

/// Evaluator that runs all policy checks and collects every denial.
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
