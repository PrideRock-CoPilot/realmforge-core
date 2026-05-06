pub mod audit;
pub mod boards;
pub mod intake;
pub mod plan_store;

pub mod build_watch;
pub mod bundles;
pub mod catalog;
pub mod commands;
pub mod error;
pub mod grants;
pub mod knowledge;
pub mod live_watch;
pub mod login;
pub mod rollbacks;
pub mod sessions;
pub mod snapshots;
pub mod work_packets;
pub mod work_path;

pub use error::StoreError;
pub use rollbacks::RollbackPreviewRow;

use authority_domain::AuditEventId;
use sqlx::PgPool;

/// Result of get_chain_bounds: (first_event, last_event, count)
pub type ChainBounds = (
    Option<(AuditEventId, String)>,
    Option<(AuditEventId, String)>,
    u64,
);

#[derive(Clone)]
pub struct CoreStore {
    pool: PgPool,
}

impl CoreStore {
    pub async fn connect(uri: &str) -> Result<Self, StoreError> {
        let pool = PgPool::connect(uri).await?;
        Ok(Self { pool })
    }

    pub fn connect_lazy(uri: &str) -> Result<Self, StoreError> {
        let pool = PgPool::connect_lazy(uri)?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run all pending SQL migrations (idempotent).
    pub async fn migrate(&self) -> Result<(), StoreError> {
        sqlx::migrate!("../../db/migrations")
            .run(&self.pool)
            .await?;
        tracing::info!("database migrations applied");
        Ok(())
    }
}

impl std::fmt::Debug for CoreStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreStore")
            .field("pool", &"PgPool")
            .finish()
    }
}
