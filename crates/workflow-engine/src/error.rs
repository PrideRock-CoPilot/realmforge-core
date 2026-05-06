//! # Workflow Engine Error Types
//!
//! Typed errors for the workflow lifecycle engine.

use authority_domain::LifecycleStage;
use thiserror::Error;

/// Errors that can occur during workflow execution.
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// A stage agent failed during execution.
    #[error("stage {stage} execution failed: {reason}")]
    StageExecutionFailed {
        stage: LifecycleStage,
        reason: String,
    },

    /// The exit gate for a stage is not satisfied.
    #[error("exit gate not satisfied for {stage}: {summary}")]
    ExitGateUnsatisfied {
        stage: LifecycleStage,
        summary: String,
    },

    /// The workflow is blocked on an external decision.
    #[error("workflow blocked on decision {decision_id} at stage {stage}")]
    BlockedOnDecision {
        decision_id: String,
        stage: LifecycleStage,
    },

    /// Stages were attempted in the wrong order.
    #[error("stage order violation: attempted {attempted}, expected {expected}")]
    StageOrderViolation {
        attempted: LifecycleStage,
        expected: LifecycleStage,
    },

    /// A required artifact is missing for the current stage.
    #[error("missing artifact for stage {stage}: {expected_artifact}")]
    MissingArtifact {
        stage: LifecycleStage,
        expected_artifact: String,
    },

    /// A handoff between stages was rejected.
    #[error("handoff rejected from {from} to {to}: {reason}")]
    HandoffRejected {
        from: LifecycleStage,
        to: LifecycleStage,
        reason: String,
    },

    /// An internal workflow engine error.
    #[error("internal workflow error: {0}")]
    Internal(String),
}

impl WorkflowError {
    /// Create a stage execution failure error.
    pub fn stage_failure(stage: LifecycleStage, reason: impl Into<String>) -> Self {
        Self::StageExecutionFailed {
            stage,
            reason: reason.into(),
        }
    }

    /// Create an exit gate unsatisfied error.
    pub fn gate_unsatisfied(stage: LifecycleStage, summary: impl Into<String>) -> Self {
        Self::ExitGateUnsatisfied {
            stage,
            summary: summary.into(),
        }
    }

    /// Create a blocked on decision error.
    pub fn blocked(stage: LifecycleStage, decision_id: impl Into<String>) -> Self {
        Self::BlockedOnDecision {
            stage,
            decision_id: decision_id.into(),
        }
    }

    /// Create a stage order violation error.
    pub fn order_violation(attempted: LifecycleStage, expected: LifecycleStage) -> Self {
        Self::StageOrderViolation {
            attempted,
            expected,
        }
    }

    /// Create a handoff rejected error.
    pub fn handoff_rejected(
        from: LifecycleStage,
        to: LifecycleStage,
        reason: impl Into<String>,
    ) -> Self {
        Self::HandoffRejected {
            from,
            to,
            reason: reason.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, WorkflowError>;
