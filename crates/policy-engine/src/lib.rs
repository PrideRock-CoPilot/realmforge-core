use authority_domain::{ActorScope, ApprovalState, CommandStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DenialCode {
    ActorUnauthorized,
    SkillNotUsable,
    WrongSkillActive,
    ApprovalRequired,
    ProposalOnly,
    SessionStale,
    SessionExpired,
    /// The command has already reached the `Applied` terminal state.
    CommandAlreadyApplied,
    /// The command has already been denied and cannot be re-authorized.
    CommandAlreadyDenied,
    /// The command is already `Authorized` and cannot be authorized again.
    CommandAlreadyAuthorized,
    RateLimited,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectiveAction {
    RefreshSession,
    RebindSkill,
    RequestApproval,
    SwitchExecutionMode,
    VerifyCommandStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDenial {
    pub code: DenialCode,
    pub message: String,
    pub retryable: bool,
    pub corrective_action: Option<CorrectiveAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub denial: Option<PolicyDenial>,
}

impl PolicyDecision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            denial: None,
        }
    }

    pub fn deny(
        code: DenialCode,
        message: impl Into<String>,
        retryable: bool,
        corrective_action: Option<CorrectiveAction>,
    ) -> Self {
        Self {
            allowed: false,
            denial: Some(PolicyDenial {
                code,
                message: message.into(),
                retryable,
                corrective_action,
            }),
        }
    }
}

impl std::fmt::Display for PolicyDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.allowed {
            write!(f, "allowed")
        } else if let Some(d) = &self.denial {
            write!(f, "denied({:?}): {}", d.code, d.message)
        } else {
            write!(f, "denied")
        }
    }
}

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
            format!("target type '{target_type}' is not allowed; allowed: {:?}", allowed_targets),
            false,
            None,
        );
    }

    PolicyDecision::allow()
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::*;
    use chrono::Duration;

    fn scope() -> ActorScope {
        let now = Utc::now();
        ActorScope {
            tenant_id: TenantId::new("tenant").unwrap(),
            project_id: ProjectId::new("project").unwrap(),
            actor_id: ActorId::new("actor").unwrap(),
            roles: vec![RoleId::new("builder").unwrap()],
            session_id: SessionId::new("session").unwrap(),
            skill_session_id: Some(SkillSessionId::new("skill-session").unwrap()),
            requested_skill_id: Some(SkillId::new("skill").unwrap()),
            active_skill_id: Some(SkillId::new("skill").unwrap()),
            allowed_actions: vec!["core.create_snapshot".to_string()],
            approval_id: None,
            approval_state: ApprovalState::Approved,
            execution_mode: ExecutionMode::Approved,
            context_updated_at: now,
            expires_at: now + Duration::minutes(5),
        }
    }

    #[test]
    fn allows_valid_scope() {
        assert!(
            authorize_action(
                &scope(),
                "core.create_snapshot",
                true,
                true,
                Utc::now(),
                300
            )
            .allowed
        );
    }

    #[test]
    fn denies_wrong_skill() {
        let mut scope = scope();
        scope.active_skill_id = Some(SkillId::new("other").unwrap());
        let decision =
            authorize_action(&scope, "core.create_snapshot", true, true, Utc::now(), 300);
        assert_eq!(decision.denial.unwrap().code, DenialCode::WrongSkillActive);
    }

    #[test]
    fn denies_expired_session() {
        let mut scope = scope();
        scope.expires_at = Utc::now() - Duration::seconds(1);
        let decision =
            authorize_action(&scope, "core.create_snapshot", true, true, Utc::now(), 300);
        assert_eq!(decision.denial.unwrap().code, DenialCode::SessionExpired);
    }

    #[test]
    fn denies_stale_context() {
        let mut scope = scope();
        scope.context_updated_at = Utc::now() - Duration::seconds(600);
        let decision =
            authorize_action(&scope, "core.create_snapshot", true, true, Utc::now(), 300);
        assert_eq!(decision.denial.unwrap().code, DenialCode::SessionStale);
    }

    #[test]
    fn denies_unauthorized_action() {
        let decision = authorize_action(
            &scope(),
            "core.delete_everything",
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(decision.denial.unwrap().code, DenialCode::ActorUnauthorized);
    }

    #[test]
    fn denies_proposal_only_for_mutation() {
        let mut scope = scope();
        scope.execution_mode = ExecutionMode::Proposal;
        let decision =
            authorize_action(&scope, "core.create_snapshot", true, true, Utc::now(), 300);
        assert_eq!(decision.denial.unwrap().code, DenialCode::ProposalOnly);
    }

    #[test]
    fn denies_missing_approval() {
        let mut scope = scope();
        scope.approval_state = ApprovalState::Required;
        let decision =
            authorize_action(&scope, "core.create_snapshot", true, true, Utc::now(), 300);
        assert_eq!(decision.denial.unwrap().code, DenialCode::ApprovalRequired);
    }

    #[test]
    fn command_already_applied_cannot_authorize() {
        let decision = authorize_command_action(
            &scope(),
            "core.create_snapshot",
            &CommandStatus::Applied,
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(
            decision.denial.unwrap().code,
            DenialCode::CommandAlreadyApplied
        );
    }

    #[test]
    fn command_already_denied_cannot_authorize() {
        let decision = authorize_command_action(
            &scope(),
            "core.create_snapshot",
            &CommandStatus::Denied,
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(
            decision.denial.unwrap().code,
            DenialCode::CommandAlreadyDenied
        );
    }

    #[test]
    fn command_already_authorized_cannot_reauthorize() {
        let decision = authorize_command_action(
            &scope(),
            "core.create_snapshot",
            &CommandStatus::Authorized,
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(
            decision.denial.unwrap().code,
            DenialCode::CommandAlreadyAuthorized
        );
    }

    #[test]
    fn policy_chain_evaluator_collects_all_denials() {
        let check1 = || {
            PolicyDecision::deny(
                DenialCode::SessionExpired,
                "scope expired",
                true,
                Some(CorrectiveAction::RefreshSession),
            )
        };
        let check2 = || {
            PolicyDecision::deny(
                DenialCode::ApprovalRequired,
                "approval needed",
                true,
                Some(CorrectiveAction::RequestApproval),
            )
        };

        let result = PolicyChainEvaluator::evaluate(vec![
            Box::new(check1) as Box<dyn FnOnce() -> PolicyDecision>,
            Box::new(check2),
        ]);

        assert!(!result.allowed);
        assert_eq!(result.denials.len(), 2);
        assert_eq!(result.denials[0].code, DenialCode::SessionExpired);
        assert_eq!(result.denials[1].code, DenialCode::ApprovalRequired);
    }

    #[test]
    fn policy_chain_evaluator_allows_when_no_denials() {
        let check = || PolicyDecision::allow();
        let result = PolicyChainEvaluator::evaluate(vec![
            Box::new(check) as Box<dyn FnOnce() -> PolicyDecision>
        ]);
        assert!(result.allowed);
        assert!(result.denials.is_empty());
    }
}
