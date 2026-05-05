// Shared test helpers for control-service integration tests.
//
// All tests gracefully skip when no database is available.

use authority_domain::{ActorId, ActorScope, ProjectId, TenantId};
use control_service::AuditService;
use control_store::CoreStore;

/// Connect to PostgreSQL and run migrations, or return None if unavailable.
pub async fn get_store() -> Option<CoreStore> {
    let uri = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/realmforge".to_string());

    let store = match CoreStore::connect(&uri).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "Skipping integration test - no database available (tried: {})",
                uri
            );
            return None;
        }
    };

    match store.migrate().await {
        Ok(_) => Some(store),
        Err(e) => {
            eprintln!(
                "Skipping integration test - migration failed (tried: {}): {e}",
                uri
            );
            None
        }
    }
}

/// Create an AuditService from a store.
#[allow(dead_code)]
pub fn audit_service(store: &CoreStore) -> AuditService {
    AuditService::new(store.clone())
}

#[allow(dead_code)]
pub fn test_tenant() -> TenantId {
    TenantId::new("test-tenant").unwrap()
}

#[allow(dead_code)]
pub fn test_project() -> ProjectId {
    ProjectId::new("test-project").unwrap()
}

#[allow(dead_code)]
pub fn test_actor() -> ActorId {
    ActorId::new("test-actor").unwrap()
}

#[allow(dead_code)]
pub fn test_scope() -> ActorScope {
    ActorScope::default()
}
