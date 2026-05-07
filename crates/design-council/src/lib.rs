//! # Design Council — Evidence-Driven Design Research Engine
//!
//! The `design-council` crate provides the pure type system for structured,
//! evidence-driven design research. It implements the Design Council protocol:
//! frame the problem → gather examples → extract patterns → synthesize
//! recommendations → surface risks → produce a Design Brief.
//!
//! ## Design Philosophy
//!
//! This crate owns the **data model** of design research. It has no I/O, no
//! database access, no HTTP. It is pure logic and type definitions.
//!
//! - `DesignBrief` is the aggregate root — a complete design research session
//! - `DesignBriefStatus` enforces valid state transitions
//! - `DesignCouncilError` provides typed error handling for all validation rules
//!
//! ## Layer Position
//!
//! ```text
//! design-council (this crate — pure domain types, no IO)
//!       ↓
//! control-service (orchestration, persistence)
//!       ↓
//! control-store (PostgreSQL persistence)
//! ```
//!
//! ## Future Extension
//!
//! This crate is the foundation. Future phases will add:
//! - `control-service/src/design_council_service.rs` — service orchestration
//! - `control-store` — design brief persistence (JSONB)
//! - `control-api` — `/v1/design-briefs` REST endpoints
//! - `agent-mcp` — MCP tools for AI agent invocation

pub mod error;
pub mod state;
pub mod types;

pub use error::DesignCouncilError;
pub use state::DesignBriefStatus;
pub use types::*;
