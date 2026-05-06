//! # Workflow Engine
//!
//! The workflow lifecycle agent system for RealmForge.
//!
//! Implements the focused workflow lifecycle from `.clinerules/07-focused-workflow-lifecycle.md`
//! as runnable Rust stage agents that can be sequenced by the `Orchestrator`.
//!
//! ## Lifecycle
//!
//! ```text
//! Idea → Planning → [Design] → Architecture → [Council] →
//! Development → Peer Review → Code Review → Testing → Documentation
//! ```
//!
//! ## Architecture
//!
//! - **Domain types** live in `authority-domain::workflow` (pure types, no IO)
//! - **Stage agents** live in `workflow-engine::stages` (each implements `StageAgent`)
//! - **Orchestrator** lives in `workflow-engine::orchestrator` (sequences stages)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use workflow_engine::Orchestrator;
//! use authority_domain::{WorkflowContext, TenantId, ProjectId};
//!
//! let tid = TenantId::new("t1").unwrap();
//! let pid = ProjectId::new("p1").unwrap();
//! let ctx = WorkflowContext::new("wf-1", tid, pid, false);
//! let mut orchestrator = Orchestrator::new(ctx);
//! let result = orchestrator.run_full_workflow().await;
//! ```

pub mod error;
pub mod orchestrator;
pub mod stages;
pub mod traits;

pub use error::{Result, WorkflowError};
pub use orchestrator::Orchestrator;
pub use stages::*;
pub use traits::StageAgent;
