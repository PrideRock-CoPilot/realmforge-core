//! Error types for the rfsource format layer.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid magic header in {0}")]
    InvalidMagic(PathBuf),

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),

    #[error("Corrupt frame data: {0}")]
    CorruptFrame(String),

    #[error("Frame too large (max {max} bytes, got {actual})")]
    FrameTooLarge { max: u64, actual: u64 },

    #[error("File size limit exceeded: current size {current_bytes} bytes ({current_mb:.2} MB), limit {limit_bytes} bytes ({limit_mb:.2} MB), attempted write {attempted_bytes} bytes would exceed limit by {excess_bytes} bytes ({excess_mb:.2} MB)")]
    FileSizeLimitExceeded {
        current_bytes: u64,
        current_mb: f64,
        limit_bytes: u64,
        limit_mb: f64,
        attempted_bytes: u64,
        excess_bytes: u64,
        excess_mb: f64,
    },
}

pub type Result<T> = std::result::Result<T, FormatError>;
