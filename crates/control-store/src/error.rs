use authority_domain::IdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database query failed: {0}")]
    Query(#[from] sqlx::Error),
    #[error("migration failed: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("serialization failed")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid id: {0}")]
    Id(#[from] IdError),
    #[error("invalid data: {0}")]
    InvalidData(String),
}

impl StoreError {
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }
}
