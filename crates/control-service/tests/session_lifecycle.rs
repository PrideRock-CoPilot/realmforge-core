// Integration tests for session lifecycle: Issue → Activate → Renew → Revoke → Verify.
//
// Validates the full SessionService workflow against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use control_service::error::ServiceError;
use control_service::SessionService;

/// Seed tenant, project, and actor so FK constraints are satisfied.
async fn seed_test_env(store: &control_store::CoreStore) {
    let pool = store.pool();
    sqlx::query("INSERT INTO tenants (id, name, status) VALUES ($1, $2, 'active') ON CONFLICT (id) DO NOTHING")
        .bind(common::test_tenant().as_str())
        .bind(common::test_tenant().as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects (id, tenant_id, name, status) VALUES ($1, $2, $3, 'active') ON CONFLICT (id) DO NOTHING")
        .bind(common::test_project().as_str())
        .bind(common::test_tenant().as_str())
        .bind(common::test_project().as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO actors (id, tenant_id, display_name, actor_type, status) VALUES ($1, $2, $3, 'human', 'active') ON CONFLICT (id) DO NOTHING")
        .bind(common::test_actor().as_str())
        .bind(common::test_tenant().as_str())
        .bind(common::test_actor().as_str())
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_session_issue_and_validate() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);
    let session = svc
        .issue_session(
            &common::test_actor(),
            &common::test_tenant(),
            &common::test_project(),
            3600,
        )
        .await
        .unwrap();

    assert_eq!(session.state, "issued");
    assert_eq!(session.actor_id, common::test_actor());

    // Validate should succeed
    let validated = svc.validate_session(&session.id).await.unwrap();
    assert_eq!(validated.id, session.id);
}

#[tokio::test]
async fn test_session_full_lifecycle() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);

    // 1. Issue
    let session = svc
        .issue_session(
            &common::test_actor(),
            &common::test_tenant(),
            &common::test_project(),
            3600,
        )
        .await
        .unwrap();
    assert_eq!(session.state, "issued");

    // 2. Activate
    let activated = svc.activate_session(&session.id).await.unwrap();
    assert_eq!(activated.state, "active");

    // 3. Renew
    let renewed = svc.renew_session(&session.id, 7200).await.unwrap();
    assert_eq!(renewed.state, "active");

    // 4. Revoke
    let revoked = svc
        .revoke_session(&session.id, "test-lifecycle")
        .await
        .unwrap();
    assert_eq!(revoked.state, "revoked");

    // 5. Verify — should fail validation after revocation
    let result = svc.validate_session(&session.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_not_found() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);
    let fake_id = authority_domain::SessionId::generate();
    let result = svc.validate_session(&fake_id).await;
    assert!(matches!(result.unwrap_err(), ServiceError::SessionNotFound));
}

#[tokio::test]
async fn test_session_revoke_twice_fails() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);

    // Issue and revoke
    let session = svc
        .issue_session(
            &common::test_actor(),
            &common::test_tenant(),
            &common::test_project(),
            3600,
        )
        .await
        .unwrap();
    svc.revoke_session(&session.id, "first-revoke")
        .await
        .unwrap();

    // Second revoke should fail
    let result = svc.revoke_session(&session.id, "second-revoke").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::Validation(msg) => {
            assert!(msg.contains("revoked") || msg.contains("terminal state"));
        }
        _ => panic!("expected Validation error"),
    }
}

#[tokio::test]
async fn test_session_expired_fails_validation() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);

    // Issue with 0-second TTL (should expire immediately)
    let session = svc
        .issue_session(
            &common::test_actor(),
            &common::test_tenant(),
            &common::test_project(),
            0,
        )
        .await
        .unwrap();

    // Give a moment to persist, then validate should fail due to expiry
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let result = svc.validate_session(&session.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_session_data() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    seed_test_env(&store).await;

    let audit_svc = common::audit_service(&store);
    let svc = SessionService::new(store, audit_svc);
    let session = svc
        .issue_session(
            &common::test_actor(),
            &common::test_tenant(),
            &common::test_project(),
            3600,
        )
        .await
        .unwrap();

    let retrieved = svc.get_session_data(&session.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, session.id);

    // Non-existent session returns None
    let fake_id = authority_domain::SessionId::generate();
    let not_found = svc.get_session_data(&fake_id).await.unwrap();
    assert!(not_found.is_none());
}
