use authority_domain::*;
use chrono::{Duration, Utc};
use policy_engine::*;

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
