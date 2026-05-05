use authority_domain::{ActorScope, CommandStatus};
use chrono::{DateTime, Utc};

use crate::authorizer::authorize_action;
use crate::types::{CorrectiveAction, DenialCode, PolicyDecision};

/// Command-specific policy: check if a command can be authorized given its current status.
pub fn authorize_command_action(
    scope: &ActorScope,
    action: &str,
    current_status: &CommandStatus,
    mutating: bool,
    approval_required: bool,
    now: DateTime<Utc>,
    max_context_age_seconds: i64,
) -> PolicyDecision {
    // Applied is a terminal state — cannot authorize or re-run.
    if *current_status == CommandStatus::Applied {
        return PolicyDecision::deny(
            DenialCode::CommandAlreadyApplied,
            "command has already been applied and is in a terminal state",
            false,
            Some(CorrectiveAction::VerifyCommandStatus),
        );
    }

    // Denied is a terminal state — propose a new command instead.
    if *current_status == CommandStatus::Denied {
        return PolicyDecision::deny(
            DenialCode::CommandAlreadyDenied,
            "command has already been denied; propose a new command to retry",
            false,
            Some(CorrectiveAction::VerifyCommandStatus),
        );
    }

    // Already-authorized commands must be applied or denied, not re-authorized.
    if *current_status == CommandStatus::Authorized {
        return PolicyDecision::deny(
            DenialCode::CommandAlreadyAuthorized,
            "command is already authorized; call apply_command or deny_command",
            false,
            Some(CorrectiveAction::VerifyCommandStatus),
        );
    }

    // Run standard scope-level checks
    authorize_action(
        scope,
        action,
        mutating,
        approval_required,
        now,
        max_context_age_seconds,
    )
}

/// Authorize a proposed action/target against a scope without an existing command.
/// Checks: allowed_actions, target_type match, scope execution mode, expiry.
///
/// Returns `PolicyDecision::allow()` if all checks pass, otherwise the first
/// denial encountered.
#[allow(clippy::too_many_arguments)]
pub fn authorize_command(
    scope: &ActorScope,
    action: &str,
    target_type: &str,
    allowed_targets: &[&str],
    mutating: bool,
    approval_required: bool,
    now: DateTime<Utc>,
    max_context_age_seconds: i64,
) -> PolicyDecision {
    // 1. Scope-level checks (expiry, staleness, action match, skill match, etc.)
    let scope_decision = authorize_action(
        scope,
        action,
        mutating,
        approval_required,
        now,
        max_context_age_seconds,
    );
    if !scope_decision.allowed {
        return scope_decision;
    }

    // 2. Target type check: ensure target_type is in allowed_targets
    if !allowed_targets.is_empty() && !allowed_targets.contains(&target_type) {
        return PolicyDecision::deny(
            DenialCode::ActorUnauthorized,
            format!(
                "target type '{target_type}' is not allowed; allowed: {:?}",
                allowed_targets
            ),
            false,
            None,
        );
    }

    PolicyDecision::allow()
}
