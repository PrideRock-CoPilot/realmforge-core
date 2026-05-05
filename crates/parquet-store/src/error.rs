//! Error types for the `parquet-store` crate.

use thiserror::Error;

/// Errors that can occur during Parquet store operations.
#[derive(Error, Debug)]
pub enum ParquetStoreError {
    /// Validation failure for a record or schema.
    #[error("validation error: {0}")]
    Validation(String),

    /// I/O error from the filesystem.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parquet write error.
    #[error("write error: {0}")]
    Write(String),

    /// Parquet read error.
    #[error("read error: {0}")]
    Read(String),

    /// Arrow/DataFusion conversion error.
    #[error("Arrow error: {0}")]
    Arrow(String),

    /// Serialization error (e.g. JSON).
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Dataset not found.
    #[error("dataset not found: {0}")]
    NotFound(String),

    /// Schema mismatch between reader and writer.
    #[error("schema version mismatch: expected {expected} but found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    /// DataFusion query error.
    #[error("query error: {0}")]
    Query(String),
}
