// Cross-crate integration test: propose command → audit → snapshot → rollback → verify.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{
    ActorId, ActorScope, ApprovalState, ExecutionMode, ProjectId, SessionId, TenantId,
};
use control_service::{AuditService, CommandService, RollbackService, SnapshotService};
use control_store::CoreStore;
use serde_json::json;

/// Build a full scope for testing.
fn full_scope(project_id: ProjectId) -> ActorScope {
    ActorScope {
        tenant_id: TenantId::new("x-crate-tenant").unwrap(),
        project_id,
        actor_id: ActorId::new("x-crate-actor").unwrap(),
        roles: vec![],
        session_id: SessionId::generate(),
        skill_session_id: None,
        requested_skill_id: None,
        active_skill_id: None,
        allowed_actions: vec!["*".to_string()],
        approval_id: None,
        approval_state: ApprovalState::NotRequired,
        execution_mode: ExecutionMode::Approved,
        context_updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
    }
}

#[tokio::test]
async fn test_cross_crate_happy_path() {
    let store: CoreStore = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let project_id = ProjectId::new("x-crate-test-project").unwrap();
    let scope = full_scope(project_id.clone());

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store.clone(), audit_svc.clone());
    let snapshot_svc = SnapshotService::new(store.clone());
    let rollback_svc = RollbackService::new(store.clone(), audit_svc.clone());

    // 1. Propose a command
    let command = cmd_svc
        .propose_command(
            &scope,
            "x_crate.test_action",
            "resource",
            "x-res-1",
            json!({"key": "value"}),
        )
        .await
        .expect("propose should succeed");
    assert_eq!(command.status.to_string(), "Proposed");

    // 2. Authorize the command
    let authorized = cmd_svc
        .authorize_command(&command.id, &scope)
        .await
        .expect("authorize should succeed");
    assert_eq!(authorized.status.to_string(), "Authorized");

    // 3. Apply the command
    let applied = cmd_svc
        .apply_command(&command.id, &scope)
        .await
        .expect("apply should succeed");
    assert_eq!(applied.status.to_string(), "Applied");

    // 4. Verify audit chain integrity
    let anchor = audit_svc
        .verify_chain(&project_id)
        .await
        .expect("verify chain should succeed");
    assert!(anchor.chain_integrity);
    assert!(
        anchor.event_count >= 1,
        "should have at least one audit event"
    );

    // 5. Create a snapshot
    let snapshot = snapshot_svc
        .create_snapshot(
            &scope.tenant_id,
            &project_id,
            "x-crate-test",
            None,
            vec![], // empty object_refs
            vec![], // empty table_exports
            None,   // no previous_manifest_hash
        )
        .await
        .expect("snapshot creation should succeed");
    assert!(!snapshot.id.as_str().is_empty());

    // 6. Preview a rollback (to itself — no-op but verifies the API)
    let preview = rollback_svc
        .preview_rollback(&snapshot.id, &snapshot.id)
        .await;
    assert!(
        preview.is_ok(),
        "preview should succeed: {:?}",
        preview.err()
    );

    // 7. Query audit events
    let (events, total) = audit_svc
        .query_events(&project_id, None, None, None, None, None, 10, 0)
        .await
        .expect("query should succeed");
    assert!(!events.is_empty());
    assert!(total >= 1);

    tracing::info!(
        "Cross-crate integration test passed: {} events, snapshot={}, command={}",
        total,
        snapshot.id.as_str(),
        command.id.as_str()
    );
}
