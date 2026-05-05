// Integration tests for command lifecycle: Propose → Authorize → Apply → Audit → Verify.
//
// Validates the full CommandService workflow with audit event emission
// and policy enforcement at every step.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{ActorId, ActorScope, CommandStatus, ExecutionMode, ProjectId, TenantId};
use control_service::{error::ServiceError, AuditService, CommandService};
use serde_json::json;
use sqlx::PgPool;

fn test_scope() -> ActorScope {
    ActorScope {
        allowed_actions: vec!["*".to_string()],
        execution_mode: ExecutionMode::ReadWrite,
        ..Default::default()
    }
}

/// Ensure required FK targets (tenant, project, actor) exist in the database.
async fn seed_test_env(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO tenants (id, name, status, created_at) \
         VALUES ('default', 'Test Tenant', 'active', now()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed_test_env: failed to insert tenant");
    sqlx::query(
        "INSERT INTO projects (id, tenant_id, name, status, created_at) \
         VALUES ('default', 'default', 'Test Project', 'active', now()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed_test_env: failed to insert project");
    sqlx::query(
        "INSERT INTO actors (id, tenant_id, display_name, actor_type, status, created_at) \
         VALUES ('default', 'default', 'Test Actor', 'human', 'active', now()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed_test_env: failed to insert actor");
}

#[tokio::test]
async fn test_command_propose() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(store.pool()).await;

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store, audit_svc);

    let scope = test_scope();
    let command = cmd_svc
        .propose_command(
            &scope,
            "core.create_snapshot",
            "snapshot",
            "obj-1",
            json!({"key": "value"}),
        )
        .await
        .unwrap();

    assert_eq!(command.status, CommandStatus::Proposed);
    assert_eq!(command.action, "core.create_snapshot");
    assert_eq!(command.target_id, "obj-1");
}

#[tokio::test]
async fn test_command_propose_authorize_apply() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(store.pool()).await;

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store, audit_svc);

    let scope = test_scope();

    // 1. Propose
    let command = cmd_svc
        .propose_command(
            &scope,
            "core.create_snapshot",
            "snapshot",
            "obj-1",
            json!({"key": "value"}),
        )
        .await
        .unwrap();
    assert_eq!(command.status, CommandStatus::Proposed);

    // 2. Authorize
    let authorized = cmd_svc
        .authorize_command(&command.id, &scope)
        .await
        .unwrap();
    assert_eq!(authorized.status, CommandStatus::Authorized);

    // 3. Apply
    let applied = cmd_svc.apply_command(&command.id, &scope).await.unwrap();
    assert_eq!(applied.status, CommandStatus::Applied);
}

#[tokio::test]
async fn test_command_not_found() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(store.pool()).await;

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store, audit_svc);
    let fake_id = authority_domain::CommandId::generate();

    let result = cmd_svc.authorize_command(&fake_id, &test_scope()).await;
    assert!(matches!(result.unwrap_err(), ServiceError::CommandNotFound));
}

#[tokio::test]
async fn test_command_audit_events_emitted() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(store.pool()).await;

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store.clone(), audit_svc.clone());
    let scope = test_scope();

    // Propose → Authorize → Apply
    let command = cmd_svc
        .propose_command(&scope, "core.test", "test", "t-1", json!({"n": 1}))
        .await
        .unwrap();
    cmd_svc
        .authorize_command(&command.id, &scope)
        .await
        .unwrap();
    cmd_svc.apply_command(&command.id, &scope).await.unwrap();

    // Query audit events — should see proposed, authorized, applied
    let (events, _total) = audit_svc
        .query_events(&scope.project_id, None, None, None, None, None, 100, 0)
        .await
        .unwrap();

    let event_types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert!(
        event_types.contains(&"command.proposed"),
        "expected command.proposed in events: {:?}",
        event_types
    );
    assert!(
        event_types.contains(&"command.authorized"),
        "expected command.authorized in events: {:?}",
        event_types
    );
    assert!(
        event_types.contains(&"command.applied"),
        "expected command.applied in events: {:?}",
        event_types
    );
}

/// Each chain test uses unique IDs so tests can run in parallel without conflict,
/// AND cleans up any events for that project from previous runs.
fn chain_ids(label: &str) -> (TenantId, ProjectId, ActorId) {
    (
        TenantId::new(format!("chain-t{label}")).unwrap(),
        ProjectId::new(format!("chain-p{label}")).unwrap(),
        ActorId::new(format!("chain-a{label}")).unwrap(),
    )
}

/// Seed tenant, project, and actor for chain integrity tests (unique IDs).
async fn seed_chain_env(pool: &PgPool, t: &TenantId, p: &ProjectId, a: &ActorId) {
    sqlx::query(
        "INSERT INTO tenants (id, name, status, created_at) \
         VALUES ($1, $2, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(t.as_str())
    .bind(t.as_str())
    .execute(pool)
    .await
    .expect("seed_chain_env: failed to insert tenant");
    sqlx::query(
        "INSERT INTO projects (id, tenant_id, name, status, created_at) \
         VALUES ($1, $2, $3, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(p.as_str())
    .bind(t.as_str())
    .bind(p.as_str())
    .execute(pool)
    .await
    .expect("seed_chain_env: failed to insert project");
    sqlx::query(
        "INSERT INTO actors (id, tenant_id, display_name, actor_type, status, created_at) \
         VALUES ($1, $2, $3, 'human', 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(a.as_str())
    .bind(t.as_str())
    .bind(a.as_str())
    .execute(pool)
    .await
    .expect("seed_chain_env: failed to insert actor");
}

/// Clear audit events for the given project so chain starts fresh.
async fn clear_chain_events(pool: &PgPool, project_id: &ProjectId) {
    sqlx::query("DELETE FROM core_audit_events WHERE project_id = $1")
        .bind(project_id.as_str())
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_command_chain_integrity() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let (tenant, project, actor) = chain_ids("int");
    seed_chain_env(store.pool(), &tenant, &project, &actor).await;
    clear_chain_events(store.pool(), &project).await;

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store, audit_svc.clone());

    // Build scope with unique IDs, allowing read-write execution
    let scope = ActorScope {
        tenant_id: tenant,
        project_id: project,
        actor_id: actor,
        allowed_actions: vec!["*".to_string()],
        execution_mode: ExecutionMode::ReadWrite,
        ..Default::default()
    };

    // Run a command lifecycle
    let cmd = cmd_svc
        .propose_command(&scope, "core.chain_test", "test", "chain-1", json!({}))
        .await
        .unwrap();
    cmd_svc.authorize_command(&cmd.id, &scope).await.unwrap();
    cmd_svc.apply_command(&cmd.id, &scope).await.unwrap();

    // Verify the audit chain
    let anchor = audit_svc.verify_chain(&scope.project_id).await.unwrap();
    assert!(anchor.chain_integrity);
    assert!(
        anchor.event_count >= 3,
        "expected >=3 events, got {}",
        anchor.event_count
    );
}
