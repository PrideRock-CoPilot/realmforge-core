//! Error types for the Artifact Registry.

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, CatalogError>;
