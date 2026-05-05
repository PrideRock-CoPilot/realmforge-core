use serde::{Deserialize, Serialize};
use std::fmt;

/// Plan status for a board plan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlanStatus {
    Draft,
    InReview,
    Approved,
    InProgress,
    Completed,
    Blocked,
    Archived,
}

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanStatus::Draft => write!(f, "draft"),
            PlanStatus::InReview => write!(f, "in_review"),
            PlanStatus::Approved => write!(f, "approved"),
            PlanStatus::InProgress => write!(f, "in_progress"),
            PlanStatus::Completed => write!(f, "completed"),
            PlanStatus::Blocked => write!(f, "blocked"),
            PlanStatus::Archived => write!(f, "archived"),
        }
    }
}

/// Board approval decision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    NeedsChanges,
}

impl fmt::Display for ApprovalDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApprovalDecision::Approved => write!(f, "approved"),
            ApprovalDecision::Rejected => write!(f, "rejected"),
            ApprovalDecision::NeedsChanges => write!(f, "needs_changes"),
        }
    }
}

/// Release command status.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReleaseStatus {
    Pending,
    Approved,
    Deployed,
    RolledBack,
}

impl fmt::Display for ReleaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseStatus::Pending => write!(f, "pending"),
            ReleaseStatus::Approved => write!(f, "approved"),
            ReleaseStatus::Deployed => write!(f, "deployed"),
            ReleaseStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// A board plan representing human planning state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardPlan {
    pub id: BoardPlanId,
    pub title: String,
    pub work_path_refs: Vec<String>,
    pub status: PlanStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A board approval on a plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardApproval {
    pub id: BoardApprovalId,
    pub plan_id: BoardPlanId,
    pub approver: String,
    pub decision: ApprovalDecision,
    pub comment: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A release command for deploying a bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseCommand {
    pub id: ReleaseId,
    pub plan_id: BoardPlanId,
    pub bundle_ref: String,
    pub approval_ref: String,
    pub status: ReleaseStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

use crate::{BoardApprovalId, BoardPlanId, ReleaseId};
