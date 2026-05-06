//! Error types for the rfsource format layer.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid magic header in {0}")]
    InvalidMagic(PathBuf),

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),

    #[error("Corrupt frame data: {0}")]
    CorruptFrame(String),

    #[error("Frame too large (max {max} bytes, got {actual})")]
    FrameTooLarge { max: u64, actual: u64 },
}

pub type Result<T> = std::result::Result<T, FormatError>;
