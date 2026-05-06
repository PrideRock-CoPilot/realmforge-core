pub mod audit;
pub mod boards;
pub mod build_watch;
pub mod bundles;
pub mod catalog;
pub mod commands;
pub mod error;
pub mod grants;
pub mod intake;
pub mod knowledge;
pub mod live_watch;
pub mod login;
pub mod rollbacks;
pub mod sessions;
pub mod snapshots;
pub mod work_packets;
pub mod work_path;

// CoreStore facade implementation
mod store_impl;

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

/// Unified store for control-plane persistence operations.
/// 
/// CoreStore wraps a PostgreSQL connection pool and provides a facade over
/// domain-specific modules (audit, sessions, commands, etc.). It delegates
/// all operations to the underlying modules while providing a unified interface.
#[derive(Clone)]
pub struct CoreStore {
    pool: PgPool,
}

impl CoreStore {
    /// Connect to the database using the provided URI.
    pub async fn connect(uri: &str) -> Result<Self, StoreError> {
        let pool = PgPool::connect(uri).await?;
        Ok(Self { pool })
    }

    /// Create a lazy connection pool that connects on first use.
    pub fn connect_lazy(uri: &str) -> Result<Self, StoreError> {
        let pool = PgPool::connect_lazy(uri)?;
        Ok(Self { pool })
    }

    /// Create a CoreStore from an existing connection pool.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Access the underlying connection pool.
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
