use authority_domain::{ActorScope, ApprovalState};
use chrono::{DateTime, Utc};

use crate::types::{CorrectiveAction, DenialCode, PolicyDecision};

/// Run all standard policy checks against a scope.
/// Logs every decision with structured context for observability.
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
