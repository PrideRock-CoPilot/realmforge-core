//! Error types for the rfsource domain model.

use std::path::PathBuf;

/// Top-level error type for all rfsource operations.
#[derive(Debug, thiserror::Error)]
pub enum RFSourceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0} already exists: {1}")]
    AlreadyExists(String, PathBuf),

    #[error("{0} not found: {1}")]
    NotFound(String, PathBuf),

    #[error("Invalid container state: {0}")]
    InvalidContainer(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("SHA-256 hash error: {0}")]
    Hash(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, RFSourceError>;
