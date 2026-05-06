//! # rfsource-store
//!
//! Source state management — commit, branch, proposal, comment, time-warp operations.
//!
//! This crate orchestrates operations on `.rfsource` files: creating projects,
//! committing artifacts, managing branches and proposals, and time-warp operations.
//! It uses `rfsource-format` for frame I/O and `rfsource-governance` for validation.
//!
//! ## Crate Law
//!
//! - Owns the source state machine (commits, branches, proposals, comments)
//! - Uses `rfsource-format` for file I/O (not direct file access)
//! - Uses `rfsource-catalog` for Artifact Registry metadata
//! - No business logic beyond state transitions

pub mod error;
pub mod rf_source;
pub mod tree;

pub use error::StoreError;
pub use rf_source::RFSource;
pub use tree::*;
