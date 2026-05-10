//! # rfsource-store
//!
//! Source state management — commit, branch, proposal, comment, time-warp operations.
//!
//! This crate orchestrates operations on `.rfsource` files: creating projects,
//! committing artifacts, managing branches and proposals, and time-warp operations.
//! It uses `rfsource-format` for frame I/O and `rfsource-governance` for validation.
//!
//! ## Multi-File Support (DDR-003)
//!
//! Repositories larger than 1.5 GB use fixed 1 GB segments for scalability.
//! See `/docs/design/DDR-003-MULTI-FILE-DESIGN.md` for design details.
//!
//! ## Crate Law
//!
//! - Owns the source state machine (commits, branches, proposals, comments)
//! - Uses `rfsource-format` for frame I/O (not direct file access)
//! - Uses `rfsource-catalog` for Artifact Registry metadata
//! - No business logic beyond state transitions

pub mod error;
pub mod multi_file;
pub mod rf_source;
pub mod source_control;
pub mod tree;
pub mod compaction;

pub use error::StoreError;
pub use multi_file::MultiFileRepo;
pub use rf_source::RFSource;
pub use tree::*;
pub use compaction::{CompactionStrategy, CompactionPlan, Compactor, CompactionResult};
