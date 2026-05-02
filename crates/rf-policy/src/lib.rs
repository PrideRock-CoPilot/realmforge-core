use chrono::{DateTime, Utc};
use rf_domain::{ActorScope, ApprovalState, CommandStatus};
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
    CommandAlreadyApplied,
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

/// Run all standard policy checks against a scope.
pub fn authorize_action(
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
pub fn evaluate_policy_chain(
    checks: &[Box<dyn FnOnce() -> PolicyDecision + '_>],
) -> PolicyDecision {
    for check in checks {
        let decision = check();
        if !decision.allowed {
            return decision;
        }
    }
    PolicyDecision::allow()
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
    // A command that's already been applied cannot be authorized again
    if *current_status == CommandStatus::Applied {
        return PolicyDecision::deny(
            DenialCode::CommandAlreadyApplied,
            "command has already been applied",
            false,
            Some(CorrectiveAction::VerifyCommandStatus),
        );
    }

    // A command that's been denied stays denied
    if *current_status == CommandStatus::Denied {
        return PolicyDecision::deny(
            DenialCode::CommandAlreadyApplied,
            "command has already been denied",
            false,
            Some(CorrectiveAction::VerifyCommandStatus),
        );
    }

    // Run standard scope-level checks
    authorize_action(scope, action, mutating, approval_required, now, max_context_age_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rf_domain::*;

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
        assert!(authorize_action(
            &scope(),
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300
        )
        .allowed);
    }

    #[test]
    fn denies_wrong_skill() {
        let mut scope = scope();
        scope.active_skill_id = Some(SkillId::new("other").unwrap());
        let decision = authorize_action(
            &scope,
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(decision.denial.unwrap().code, DenialCode::WrongSkillActive);
    }

    #[test]
    fn denies_expired_session() {
        let mut scope = scope();
        scope.expires_at = Utc::now() - Duration::seconds(1);
        let decision = authorize_action(
            &scope,
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(decision.denial.unwrap().code, DenialCode::SessionExpired);
    }

    #[test]
    fn denies_stale_context() {
        let mut scope = scope();
        scope.context_updated_at = Utc::now() - Duration::seconds(600);
        let decision = authorize_action(
            &scope,
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300,
        );
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
        let decision = authorize_action(
            &scope,
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300,
        );
        assert_eq!(decision.denial.unwrap().code, DenialCode::ProposalOnly);
    }

    #[test]
    fn denies_missing_approval() {
        let mut scope = scope();
        scope.approval_state = ApprovalState::Required;
        let decision = authorize_action(
            &scope,
            "core.create_snapshot",
            true,
            true,
            Utc::now(),
            300,
        );
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
            DenialCode::CommandAlreadyApplied
        );
    }
}
