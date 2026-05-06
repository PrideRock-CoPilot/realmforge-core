//! # rfsource-core
//!
//! Foundational domain model for the `.rfsource` single-file source ledger.
//!
//! This crate provides pure types, typed IDs, error types, and state machine
//! definitions with no IO dependencies. It is the lowest layer in the rfsource
//! crate family — all other rfsource crates depend on this one.
//!
//! ## Crate Law
//!
//! - Pure types only — no database, no file IO, no HTTP, no policy
//! - All types derive `Serialize` + `Deserialize` for frame storage
//! - All IDs are typed newtypes (no bare `String` or `Uuid`)
//! - All errors use `thiserror` with typed variants

pub mod error;
pub mod ids;
pub mod model;

pub use error::RFSourceError;
pub use ids::*;
pub use model::*;
