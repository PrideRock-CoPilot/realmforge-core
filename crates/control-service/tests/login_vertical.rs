// Integration tests for the Login Vertical — authentication, rate limiting, auto-blocking.
//
// Validates the full LoginHandler workflow against a live PostgreSQL database:
//   1. Successful credential validation and session issuance
//   2. Invalid credential rejection (constant-time comparison)
//   3. Rate limiting when exceeding max requests per window
//   4. Auto-blocking after exceeding max failed attempts
//   5. Active block prevents login
//   6. Policy management (set/get)
//   7. Block listing
//
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::login::{LoginCredentials, LoginPolicyConfig, Scope};
use authority_domain::{ActorId, ProjectId, SnapshotId, TenantId};
use control_service::error::ServiceError;
use control_service::{AuditService, LoginHandler, SessionService};
use sha2::{Digest, Sha256};

// ── Helpers ──

fn test_tenant(name: &str) -> TenantId {
    TenantId::new(format!("login-test-{name}")).unwrap()
}

fn test_project(name: &str) -> ProjectId {
    ProjectId::new(format!("login-test-{name}")).unwrap()
}

fn test_actor(name: &str) -> ActorId {
    ActorId::new(format!("login-test-actor-{name}")).unwrap()
}

fn make_handler(store: &control_store::CoreStore) -> LoginHandler {
    let audit_svc = AuditService::new(store.clone());
    let session_svc = SessionService::new(store.clone(), audit_svc.clone());
    LoginHandler::new(store.clone(), session_svc, audit_svc)
}

/// Compute hex-encoded SHA-256 hash for a credential string.
fn hash_credential(credential: &str) -> String {
    hex::encode(Sha256::digest(credential.as_bytes()))
}

/// Seed a test tenant, project, and actor so FK constraints are satisfied.
async fn seed_test_environment(
    store: &control_store::CoreStore,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    actor_id: &ActorId,
) {
    let pool = store.pool();
    sqlx::query("INSERT INTO tenants (id, name, status) VALUES ($1, $2, 'active') ON CONFLICT (id) DO NOTHING")
        .bind(tenant_id.as_str())
        .bind(tenant_id.as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects (id, tenant_id, name, status) VALUES ($1, $2, $3, 'active') ON CONFLICT (id) DO NOTHING")
        .bind(project_id.as_str())
        .bind(tenant_id.as_str())
        .bind(project_id.as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO actors (id, tenant_id, display_name, actor_type, status) VALUES ($1, $2, $3, 'human', 'active') ON CONFLICT (id) DO NOTHING")
        .bind(actor_id.as_str())
        .bind(tenant_id.as_str())
        .bind(actor_id.as_str())
        .execute(pool)
        .await
        .unwrap();
}

/// Remove stale blocks and policies for a tenant, so repeated test runs are isolated.
async fn cleanup_test_data(store: &control_store::CoreStore, tenant_id: &TenantId) {
    let pool = store.pool();
    sqlx::query("DELETE FROM login_blocks WHERE tenant_id = $1")
        .bind(tenant_id.as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM login_policies WHERE tenant_id = $1")
        .bind(tenant_id.as_str())
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM login_attempts WHERE tenant_id = $1")
        .bind(tenant_id.as_str())
        .execute(pool)
        .await
        .unwrap();
}

/// Set up a credential for an actor so login can succeed.
async fn setup_credential(
    store: &control_store::CoreStore,
    tenant_id: &TenantId,
    actor_id: &ActorId,
    credential: &str,
) {
    let hash = hash_credential(credential);
    store
        .upsert_credential(tenant_id, actor_id, &hash)
        .await
        .unwrap();
}

/// Configure a strict login policy for rate-limit and blocking tests.
fn strict_policy() -> LoginPolicyConfig {
    LoginPolicyConfig {
        max_failed_attempts: 3,
        window_seconds: 60,
        block_duration_seconds: 300, // 5 minutes
        max_requests_per_window: 5,
        credential_validation_enabled: true,
    }
}

// ── Tests ──

#[tokio::test]
async fn test_login_success() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("success");
    let project = test_project("success");
    let actor = test_actor("success");
    let password = "correct-password-123";

    seed_test_environment(&store, &tenant, &project, &actor).await;
    setup_credential(&store, &tenant, &actor, password).await;
    let handler = make_handler(&store);

    let scope = Scope::new("read:write").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: password.as_bytes().to_vec(),
        scope: scope.clone(),
    };

    let result = handler
        .handle_login(tenant.clone(), project.clone(), credentials)
        .await
        .unwrap();

    // Verify the response
    assert!(
        !result.session_token.is_empty(),
        "session token should be populated"
    );
    assert_eq!(result.actor_id, actor);
    assert_eq!(result.scope, "read:write");
    assert!(
        result.expires_at > chrono::Utc::now(),
        "expires_at should be in the future"
    );
    assert!(
        !result.audit_event_id.is_empty(),
        "audit event should be recorded"
    );
    assert!(
        !result.snapshot_id.is_empty(),
        "snapshot anchor should be recorded"
    );

    let snapshot_id = SnapshotId::new(result.snapshot_id.clone()).unwrap();
    let snapshot = store
        .get_snapshot_manifest(&snapshot_id)
        .await
        .unwrap()
        .expect("snapshot anchor should be persisted");
    assert_eq!(snapshot.tenant_id, tenant);
    assert_eq!(snapshot.project_id, project);
    assert!(
        snapshot.reason.contains(actor.as_str()),
        "snapshot reason should identify the login actor"
    );
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("invalid");
    let project = test_project("invalid");
    let actor = test_actor("invalid");
    let correct_password = "correct-password";
    let wrong_password = "wrong-password";

    seed_test_environment(&store, &tenant, &project, &actor).await;
    // Store the correct credential
    setup_credential(&store, &tenant, &actor, correct_password).await;
    let handler = make_handler(&store);

    let scope = Scope::new("read").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: wrong_password.as_bytes().to_vec(),
        scope,
    };

    let err = handler
        .handle_login(tenant, project, credentials)
        .await
        .unwrap_err();

    assert!(
        matches!(err, ServiceError::InvalidCredentials),
        "expected InvalidCredentials, got: {err}"
    );
}

#[tokio::test]
async fn test_login_rate_limited() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("ratelimit");
    let project = test_project("ratelimit");
    let actor = test_actor("ratelimit");

    seed_test_environment(&store, &tenant, &project, &actor).await;
    // Clear any stale blocks/policies from previous test runs
    cleanup_test_data(&store, &tenant).await;

    // Set a strict policy with NO auto-block — high max_failed_attempts so
    // the test hits the rate limit before reaching the block threshold.
    let mut policy = strict_policy();
    policy.max_failed_attempts = 100; // don't auto-block during rate limit test
    store.insert_login_policy(&tenant, &policy).await.unwrap();

    // Don't set up a credential — all attempts will be invalid, but the rate
    // limit check happens BEFORE credential validation (SR-3.1).
    let handler = make_handler(&store);

    // Generate enough requests to trigger rate limiting
    for i in 0..policy.max_requests_per_window {
        let scope = Scope::new("read").unwrap();
        let credentials = LoginCredentials {
            actor_id: actor.clone(),
            credential: format!("attempt-{i}").into_bytes(),
            scope,
        };

        let result = handler
            .handle_login(tenant.clone(), project.clone(), credentials)
            .await;

        match &result {
            Err(ServiceError::RateLimited(_)) => return, // ✅ rate-limited as expected
            Err(ServiceError::InvalidCredentials) => continue, // normal — keep going
            other => {
                panic!("unexpected result at attempt {i}: {other:?}");
            }
        }
    }

    // If we got here without being rate-limited, that's a problem
    // Make one more attempt that should definitely trigger rate limiting
    let scope = Scope::new("read").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: b"final-attempt".to_vec(),
        scope,
    };

    let err = handler
        .handle_login(tenant, project, credentials)
        .await
        .unwrap_err();

    assert!(
        matches!(err, ServiceError::RateLimited(_)),
        "expected RateLimited, got: {err}"
    );
}

#[tokio::test]
async fn test_login_auto_block() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("autoblock");
    let project = test_project("autoblock");
    let actor = test_actor("autoblock");

    seed_test_environment(&store, &tenant, &project, &actor).await;

    // Set a policy with low max_failed_attempts
    let mut policy = strict_policy();
    policy.max_failed_attempts = 2; // Only 2 failures → block
    store.insert_login_policy(&tenant, &policy).await.unwrap();

    let handler = make_handler(&store);

    // Make failed attempts until blocked
    for i in 0..=policy.max_failed_attempts {
        let scope = Scope::new("read").unwrap();
        let credentials = LoginCredentials {
            actor_id: actor.clone(),
            credential: format!("wrong-attempt-{i}").into_bytes(),
            scope,
        };

        let result = handler
            .handle_login(tenant.clone(), project.clone(), credentials)
            .await;

        match &result {
            Err(ServiceError::LoginBlocked(msg)) => {
                assert!(
                    msg.contains("blocked"),
                    "block message should indicate blocking: {msg}"
                );
                return; // ✅ auto-blocked as expected
            }
            Err(ServiceError::InvalidCredentials) => continue, // normal — keep going
            other => {
                panic!("unexpected result at attempt {i}: {other:?}");
            }
        }
    }

    panic!("actor was not auto-blocked after exceeding max failed attempts");
}

#[tokio::test]
async fn test_login_active_block() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("activeblock");
    let project = test_project("activeblock");
    let actor = test_actor("activeblock");

    // Manually insert a block record
    let block = authority_domain::login::LoginBlockRecord {
        actor_id: actor.clone(),
        blocked_at: chrono::Utc::now(),
        blocked_until: chrono::Utc::now() + chrono::Duration::seconds(3600),
        reason: "manual block for testing".to_string(),
    };
    store.insert_login_block(&tenant, &block).await.unwrap();

    let handler = make_handler(&store);

    // Even with correct credential, login should be blocked
    let scope = Scope::new("read").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: b"any-credential".to_vec(),
        scope,
    };

    let err = handler
        .handle_login(tenant, project, credentials)
        .await
        .unwrap_err();

    assert!(
        matches!(err, ServiceError::LoginBlocked(_)),
        "expected LoginBlocked, got: {err}"
    );
}

#[tokio::test]
async fn test_login_policy_set_and_get() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("policy");
    let handler = make_handler(&store);

    // Clear any stale policy from previous runs
    cleanup_test_data(&store, &tenant).await;

    // Initially should be None
    let initial = handler.get_policy(&tenant).await.unwrap();
    assert!(initial.is_none(), "no policy should exist initially");

    // Set a custom policy
    let custom_policy = LoginPolicyConfig {
        max_failed_attempts: 10,
        window_seconds: 120,
        block_duration_seconds: 600,
        max_requests_per_window: 20,
        credential_validation_enabled: false,
    };
    handler.set_policy(&tenant, &custom_policy).await.unwrap();

    // Retrieve and verify
    let retrieved = handler.get_policy(&tenant).await.unwrap();
    assert!(retrieved.is_some(), "policy should exist after set");
    let config = retrieved.unwrap();
    assert_eq!(config.max_failed_attempts, 10);
    assert_eq!(config.window_seconds, 120);
    assert_eq!(config.block_duration_seconds, 600);
    assert_eq!(config.max_requests_per_window, 20);
    assert!(!config.credential_validation_enabled);
}

#[tokio::test]
async fn test_login_list_blocks() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("listblocks");

    // Insert two block records
    for i in 0..2 {
        let actor = test_actor(&format!("blocked-{i}"));
        let block = authority_domain::login::LoginBlockRecord {
            actor_id: actor,
            blocked_at: chrono::Utc::now(),
            blocked_until: chrono::Utc::now() + chrono::Duration::seconds(3600),
            reason: format!("test block {i}"),
        };
        store.insert_login_block(&tenant, &block).await.unwrap();
    }

    let handler = make_handler(&store);
    let blocks = handler.list_blocks(&tenant).await.unwrap();

    assert_eq!(blocks.len(), 2, "should list 2 active blocks");
    for block in &blocks {
        assert!(
            block.blocked_until > chrono::Utc::now(),
            "blocks should not be expired"
        );
    }
}

#[tokio::test]
async fn test_login_unknown_actor_returns_same_error() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let tenant = test_tenant("unknownactor");
    let project = test_project("unknownactor");
    let actor = test_actor("does-not-exist");

    // No credential stored for this actor
    let handler = make_handler(&store);

    let scope = Scope::new("read").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: b"some-password".to_vec(),
        scope,
    };

    let err = handler
        .handle_login(tenant, project, credentials)
        .await
        .unwrap_err();

    // Should return InvalidCredentials — NOT a different error like "actor not found"
    // This prevents account enumeration (SR-6.3)
    assert!(
        matches!(err, ServiceError::InvalidCredentials),
        "expected InvalidCredentials for unknown actor, got: {err}"
    );
}

#[tokio::test]
async fn test_login_default_policy_is_applied() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    // Use a tenant with NO policy set — default should apply
    let tenant = test_tenant("defaultpolicy");
    let project = test_project("defaultpolicy");
    let actor = test_actor("defaultpolicy");

    seed_test_environment(&store, &tenant, &project, &actor).await;

    // Set up a correct credential
    setup_credential(&store, &tenant, &actor, "my-password").await;
    let handler = make_handler(&store);

    // Should succeed — default policy allows it
    let scope = Scope::new("read").unwrap();
    let credentials = LoginCredentials {
        actor_id: actor.clone(),
        credential: b"my-password".to_vec(),
        scope,
    };

    let result = handler
        .handle_login(tenant, project, credentials)
        .await
        .unwrap();

    assert_eq!(result.actor_id, actor);
    assert!(!result.session_token.is_empty());
}
