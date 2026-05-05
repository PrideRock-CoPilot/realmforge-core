// Cross-crate integration test: propose command → audit → snapshot → rollback → verify.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{
    ActorId, ActorScope, ApprovalState, CommandStatus, ExecutionMode, ProjectId, SessionId,
    TenantId,
};
use control_service::{AuditService, CommandService, RollbackService, SnapshotService};
use control_store::CoreStore;
use serde_json::json;
use sqlx::PgPool;

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

async fn seed_test_env(pool: &PgPool, scope: &ActorScope) {
    sqlx::query(
        "INSERT INTO tenants (id, name, status, created_at) \
         VALUES ($1, $2, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(scope.tenant_id.as_str())
    .bind(scope.tenant_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed tenant");

    sqlx::query(
        "INSERT INTO projects (id, tenant_id, name, status, created_at) \
         VALUES ($1, $2, $3, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(scope.project_id.as_str())
    .bind(scope.tenant_id.as_str())
    .bind(scope.project_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed project");

    sqlx::query(
        "INSERT INTO actors (id, tenant_id, display_name, actor_type, status, created_at) \
         VALUES ($1, $2, $3, 'human', 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(scope.actor_id.as_str())
    .bind(scope.tenant_id.as_str())
    .bind(scope.actor_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed actor");

    sqlx::query("DELETE FROM rollback_previews WHERE project_id = $1")
        .bind(scope.project_id.as_str())
        .execute(pool)
        .await
        .expect("failed to clear rollback previews");

    sqlx::query("DELETE FROM snapshot_manifests WHERE project_id = $1")
        .bind(scope.project_id.as_str())
        .execute(pool)
        .await
        .expect("failed to clear snapshot manifests");

    sqlx::query("DELETE FROM core_audit_events WHERE project_id = $1")
        .bind(scope.project_id.as_str())
        .execute(pool)
        .await
        .expect("failed to clear audit events");

    sqlx::query("DELETE FROM bounded_commands WHERE project_id = $1")
        .bind(scope.project_id.as_str())
        .execute(pool)
        .await
        .expect("failed to clear bounded commands");
}

#[tokio::test]
async fn test_cross_crate_happy_path() {
    let store: CoreStore = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let project_id = ProjectId::new("x-crate-test-project").unwrap();
    let scope = full_scope(project_id.clone());
    seed_test_env(store.pool(), &scope).await;

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
    assert_eq!(command.status, CommandStatus::Proposed);

    // 2. Authorize the command
    let authorized = cmd_svc
        .authorize_command(&command.id, &scope)
        .await
        .expect("authorize should succeed");
    assert_eq!(authorized.status, CommandStatus::Authorized);

    // 3. Apply the command
    let applied = cmd_svc
        .apply_command(&command.id, &scope)
        .await
        .expect("apply should succeed");
    assert_eq!(applied.status, CommandStatus::Applied);

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
