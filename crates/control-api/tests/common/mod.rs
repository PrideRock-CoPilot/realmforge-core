// Shared test helpers for control-api integration tests.
// Uses reqwest against a spawned test server.
// Tests gracefully skip when no database is available.

use authority_domain::{ActorId, ProjectId, TenantId};
use control_service::ServiceContext;
use control_store::CoreStore;
use sqlx::PgPool;
use std::net::TcpListener;
use std::time::Duration;

/// Connect to PostgreSQL and run migrations, or return None if unavailable.
pub async fn get_store() -> Option<CoreStore> {
    let uri = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/realmforge".to_string());

    let store = match CoreStore::connect(&uri).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "Skipping API integration test - no database available (tried: {})",
                uri
            );
            return None;
        }
    };

    match store.migrate().await {
        Ok(_) => Some(store),
        Err(e) => {
            eprintln!(
                "Skipping API integration test - migration failed (tried: {}): {e}",
                uri
            );
            None
        }
    }
}

#[allow(dead_code)]
pub async fn seed_identity(
    pool: &PgPool,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    actor_id: &ActorId,
) {
    sqlx::query(
        "INSERT INTO tenants (id, name, status, created_at) \
         VALUES ($1, $2, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant_id.as_str())
    .bind(tenant_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed tenant");

    sqlx::query(
        "INSERT INTO projects (id, tenant_id, name, status, created_at) \
         VALUES ($1, $2, $3, 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(project_id.as_str())
    .bind(tenant_id.as_str())
    .bind(project_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed project");

    sqlx::query(
        "INSERT INTO actors (id, tenant_id, display_name, actor_type, status, created_at) \
         VALUES ($1, $2, $3, 'human', 'active', now()) ON CONFLICT (id) DO NOTHING",
    )
    .bind(actor_id.as_str())
    .bind(tenant_id.as_str())
    .bind(actor_id.as_str())
    .execute(pool)
    .await
    .expect("failed to seed actor");
}

/// Spawn an API server on a random port and return a reqwest client and the base URL.
pub struct TestServer {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl TestServer {
    #[allow(dead_code)]
    pub async fn new(store: CoreStore) -> Self {
        let ctx = ServiceContext::new(store);
        let app = control_api::router(ctx);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client, base_url }
    }

    #[allow(dead_code)]
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .unwrap()
    }

    #[allow(dead_code)]
    pub async fn post(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()
            .await
            .unwrap()
    }
}
