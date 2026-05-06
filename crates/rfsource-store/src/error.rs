//! Error types for the rfsource store layer.

use rfsource_core::RFSourceError;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Core error: {0}")]
    Core(#[from] RFSourceError),

    #[error("Format error: {0}")]
    Format(#[from] rfsource_format::FormatError),

    #[error("Governance check failed: {0}")]
    Governance(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("Grant denied: {0}")]
    GrantDenied(String),

    #[error("Proposal error: {0}")]
    ProposalError(String),

    #[error("Time-warp error: {0}")]
    TimeWarpError(String),

    #[error("Comment error: {0}")]
    CommentError(String),

    #[error("Catalog error: {0}")]
    Catalog(#[from] rfsource_catalog::CatalogError),
}

pub type Result<T> = std::result::Result<T, StoreError>;
