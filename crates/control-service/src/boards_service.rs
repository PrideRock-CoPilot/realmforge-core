use authority_domain::{
    ApprovalDecision, BoardApproval, BoardApprovalId, BoardPlan, BoardPlanId, PlanStatus,
    ReleaseCommand, ReleaseId, ReleaseStatus,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Boards service — human planning, approval, status, and release command surface.
#[derive(Clone)]
pub struct BoardsService {
    store: CoreStore,
}

impl BoardsService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Create a new board plan in Draft status.
    #[instrument(skip(self))]
    pub async fn create_plan(
        &self,
        title: String,
        work_path_refs: Vec<String>,
    ) -> Result<BoardPlan, ServiceError> {
        let now = Utc::now();
        let plan = BoardPlan {
            id: BoardPlanId::generate(),
            title,
            work_path_refs,
            status: PlanStatus::Draft,
            created_at: now,
            updated_at: now,
        };
        self.store.insert_plan(&plan).await?;
        info!(plan_id = %plan.id, "board plan created");
        Ok(plan)
    }

    /// Submit a plan for review (Draft → InReview).
    #[instrument(skip(self))]
    pub async fn submit_for_approval(
        &self,
        plan_id: &BoardPlanId,
    ) -> Result<BoardPlan, ServiceError> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("plan {plan_id} not found")))?;

        if plan.status != PlanStatus::Draft {
            return Err(ServiceError::Validation(format!(
                "plan {plan_id} is in status {} — only Draft plans can be submitted",
                plan.status
            )));
        }

        self.store.update_plan_status(plan_id, "in_review").await?;
        plan.status = PlanStatus::InReview;
        plan.updated_at = Utc::now();

        info!(plan_id = %plan_id, "plan submitted for approval");
        Ok(plan)
    }

    /// Approve a plan (InReview → Approved).
    #[instrument(skip(self))]
    pub async fn approve_plan(
        &self,
        plan_id: &BoardPlanId,
        approver: String,
        comment: Option<String>,
    ) -> Result<BoardPlan, ServiceError> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("plan {plan_id} not found")))?;

        if plan.status != PlanStatus::InReview {
            return Err(ServiceError::Validation(format!(
                "plan {plan_id} is in status {} — only InReview plans can be approved",
                plan.status
            )));
        }

        let approval = BoardApproval {
            id: BoardApprovalId::generate(),
            plan_id: plan_id.clone(),
            approver,
            decision: ApprovalDecision::Approved,
            comment,
            timestamp: Utc::now(),
        };
        self.store.submit_approval(&approval).await?;
        self.store.update_plan_status(plan_id, "approved").await?;
        plan.status = PlanStatus::Approved;
        plan.updated_at = Utc::now();

        info!(plan_id = %plan_id, "plan approved");
        Ok(plan)
    }

    /// Reject a plan (InReview → Draft).
    #[instrument(skip(self))]
    pub async fn reject_plan(
        &self,
        plan_id: &BoardPlanId,
        approver: String,
        comment: String,
    ) -> Result<BoardPlan, ServiceError> {
        let mut plan = self
            .store
            .get_plan(plan_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("plan {plan_id} not found")))?;

        if plan.status != PlanStatus::InReview {
            return Err(ServiceError::Validation(format!(
                "plan {plan_id} is in status {} — only InReview plans can be rejected",
                plan.status
            )));
        }

        let approval = BoardApproval {
            id: BoardApprovalId::generate(),
            plan_id: plan_id.clone(),
            approver,
            decision: ApprovalDecision::Rejected,
            comment: Some(comment),
            timestamp: Utc::now(),
        };
        self.store.submit_approval(&approval).await?;
        self.store.update_plan_status(plan_id, "draft").await?;
        plan.status = PlanStatus::Draft;
        plan.updated_at = Utc::now();

        info!(plan_id = %plan_id, "plan rejected — returned to draft");
        Ok(plan)
    }

    /// List plans with optional status filter.
    #[instrument(skip(self))]
    pub async fn list_plans(
        &self,
        status_filter: Option<PlanStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BoardPlan>, ServiceError> {
        let status_str = status_filter.as_ref().map(|s| format!("{s}"));
        let plans = self
            .store
            .list_plans(status_str.as_deref(), limit, offset)
            .await?;
        Ok(plans)
    }

    /// Submit a release command for an approved plan.
    #[instrument(skip(self))]
    pub async fn submit_release_command(
        &self,
        plan_id: &BoardPlanId,
        bundle_ref: String,
        approval_ref: String,
    ) -> Result<ReleaseCommand, ServiceError> {
        let plan = self
            .store
            .get_plan(plan_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("plan {plan_id} not found")))?;

        if plan.status != PlanStatus::Approved {
            return Err(ServiceError::Validation(format!(
                "plan {plan_id} is in status {} — only Approved plans can be released",
                plan.status
            )));
        }

        let cmd = ReleaseCommand {
            id: ReleaseId::generate(),
            plan_id: plan_id.clone(),
            bundle_ref,
            approval_ref,
            status: ReleaseStatus::Pending,
            created_at: Utc::now(),
        };
        self.store.insert_release_command(&cmd).await?;
        info!(release_id = %cmd.id, "release command submitted");
        Ok(cmd)
    }
}
