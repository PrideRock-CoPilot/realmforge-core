//! # StageAgent Trait
//!
//! The common interface that every lifecycle stage agent implements.
//! Each stage in the focused workflow lifecycle is an agent that:
//! - Owns its stage-specific validation logic
//! - Enforces its exit gate before allowing handoff
//! - Produces structured artifacts downstream stages can consume

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, GateResult, LifecycleStage, StageArtifact, StageResult, WorkflowContext,
};

/// The common interface for all lifecycle stage agents.
///
/// # Lifecycle
///
/// 1. `stage()` — identify which stage this agent represents
/// 2. `execute()` — run the stage's validation logic
/// 3. `check_exit_gate()` — verify the stage's exit criteria
/// 4. `produce_artifact()` — produce the required artifact
#[async_trait]
pub trait StageAgent: Send + Sync + std::fmt::Debug {
    /// The lifecycle stage this agent represents.
    fn stage(&self) -> LifecycleStage;

    /// Execute the stage's validation logic.
    ///
    /// Reads from and writes to the workflow context. Returns a `StageResult`
    /// indicating pass, fail, or blocked status with detailed findings.
    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult;

    /// Check whether the stage's exit gate is satisfied.
    ///
    /// Called after `execute()` to determine if the handoff to the next stage
    /// can proceed. Returns a `GateResult` with individual check results.
    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult;

    /// The artifact descriptor for what this stage produces.
    fn required_artifact(&self) -> ArtifactDescriptor;

    /// Produce the stage's required artifact and add it to the context.
    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact;

    /// Get a human-readable status summary for this stage.
    fn status_summary(&self, ctx: &WorkflowContext) -> String;
}
