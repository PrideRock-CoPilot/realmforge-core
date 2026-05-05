// Integration tests for command lifecycle: Propose → Authorize → Apply → Audit → Verify.
//
// Validates the full CommandService workflow with audit event emission
// and policy enforcement at every step.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{ActorScope, CommandStatus};
use control_service::{error::ServiceError, AuditService, CommandService};
use serde_json::json;

fn test_scope() -> ActorScope {
    ActorScope::default()
}

#[tokio::test]
async fn test_command_propose() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

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

#[tokio::test]
async fn test_command_chain_integrity() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let audit_svc = AuditService::new(store.clone());
    let cmd_svc = CommandService::new(store, audit_svc.clone());
    let scope = test_scope();

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
