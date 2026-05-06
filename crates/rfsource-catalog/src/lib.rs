//! # rfsource-catalog — Artifact Registry
//!
//! PostgreSQL-backed Artifact Registry for `.rfsource` metadata.
//!
//! This crate provides the metadata layer that bridges `.rfsource` files
//! and the governance system. It indexes what artifacts exist, their
//! versions, grants, policies, and metadata.
//!
//! ## Database Schema
//!
//! All tables use the `artifact_registry` schema with `ar_` prefix:
//!
//! - `ar_artifacts` — artifact registry entries
//! - `ar_versions` — version metadata
//! - `ar_grants` — grant bindings per artifact
//! - `ar_tags` — artifact tags/labels
//!
//! ## Crate Law
//!
//! - Metadata queries only — no direct frame I/O
//! - All artifact data reads go through `.rfsource` files directly

pub mod error;
pub mod models;
pub mod registry;

pub use error::CatalogError;
pub use models::*;
pub use registry::ArtifactRegistry;
