use crate::StoreError;
use authority_domain::{BoardApprovalId, BoardPlan, BoardPlanId, ReleaseCommand};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Insert a new board plan.
pub async fn insert_plan(pool: &PgPool, plan: &BoardPlan) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO board_plans (id, title, work_path_refs, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(plan.id.as_str())
    .bind(&plan.title)
    .bind(serde_json::to_value(&plan.work_path_refs)?)
    .bind(format!("{}", plan.status))
    .bind(plan.created_at)
    .bind(plan.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// List board plans, optionally filtered by status.
pub async fn list_plans(
    pool: &PgPool,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BoardPlan>, StoreError> {
    let rows: Vec<BoardPlanRaw> = if let Some(status) = status_filter {
        sqlx::query_as(
            "SELECT id, title, work_path_refs, status, created_at, updated_at \
             FROM board_plans WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, title, work_path_refs, status, created_at, updated_at \
             FROM board_plans ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };
    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Update plan status.
pub async fn update_plan_status(
    pool: &PgPool,
    plan_id: &BoardPlanId,
    status: &str,
) -> Result<(), StoreError> {
    let updated = sqlx::query(
        "UPDATE board_plans SET status = $1, updated_at = now() WHERE id = $2",
    )
    .bind(status)
    .bind(plan_id.as_str())
    .execute(pool)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(StoreError::invalid_data(format!(
            "plan {plan_id} not found"
        )));
    }
    Ok(())
}

/// Submit an approval decision for a plan.
pub async fn submit_approval(
    pool: &PgPool,
    approval: &authority_domain::BoardApproval,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO board_approvals (id, plan_id, approver, decision, comment, timestamp) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(approval.id.as_str())
    .bind(approval.plan_id.as_str())
    .bind(&approval.approver)
    .bind(format!("{}", approval.decision))
    .bind(&approval.comment)
    .bind(approval.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

/// List approvals for a plan.
pub async fn list_approvals(
    pool: &PgPool,
    plan_id: &BoardPlanId,
) -> Result<Vec<authority_domain::BoardApproval>, StoreError> {
    let rows: Vec<BoardApprovalRaw> = sqlx::query_as(
        "SELECT id, plan_id, approver, decision, comment, timestamp \
         FROM board_approvals WHERE plan_id = $1 ORDER BY timestamp DESC",
    )
    .bind(plan_id.as_str())
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Insert a release command.
pub async fn insert_release_command(
    pool: &PgPool,
    cmd: &ReleaseCommand,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO release_commands (id, plan_id, bundle_ref, approval_ref, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(cmd.id.as_str())
    .bind(cmd.plan_id.as_str())
    .bind(&cmd.bundle_ref)
    .bind(&cmd.approval_ref)
    .bind(format!("{}", cmd.status))
    .bind(cmd.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Raw row types ──

#[derive(sqlx::FromRow)]
struct BoardPlanRaw {
    id: String,
    title: String,
    work_path_refs: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryInto<BoardPlan> for BoardPlanRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<BoardPlan, Self::Error> {
        Ok(BoardPlan {
            id: BoardPlanId::new(self.id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            title: self.title,
            work_path_refs: serde_json::from_value(self.work_path_refs)?,
            status: parse_plan_status(&self.status)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BoardApprovalRaw {
    id: String,
    plan_id: String,
    approver: String,
    decision: String,
    comment: Option<String>,
    timestamp: DateTime<Utc>,
}

impl TryInto<authority_domain::BoardApproval> for BoardApprovalRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<authority_domain::BoardApproval, Self::Error> {
        Ok(authority_domain::BoardApproval {
            id: BoardApprovalId::new(self.id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            plan_id: BoardPlanId::new(self.plan_id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            approver: self.approver,
            decision: parse_approval_decision(&self.decision)?,
            comment: self.comment,
            timestamp: self.timestamp,
        })
    }
}

fn parse_plan_status(s: &str) -> Result<authority_domain::PlanStatus, StoreError> {
    match s {
        "draft" => Ok(authority_domain::PlanStatus::Draft),
        "in_review" => Ok(authority_domain::PlanStatus::InReview),
        "approved" => Ok(authority_domain::PlanStatus::Approved),
        "in_progress" => Ok(authority_domain::PlanStatus::InProgress),
        "completed" => Ok(authority_domain::PlanStatus::Completed),
        "blocked" => Ok(authority_domain::PlanStatus::Blocked),
        "archived" => Ok(authority_domain::PlanStatus::Archived),
        _ => Err(StoreError::invalid_data(format!("unknown plan status: {s}"))),
    }
}

fn parse_approval_decision(s: &str) -> Result<authority_domain::ApprovalDecision, StoreError> {
    match s {
        "approved" => Ok(authority_domain::ApprovalDecision::Approved),
        "rejected" => Ok(authority_domain::ApprovalDecision::Rejected),
        "needs_changes" => Ok(authority_domain::ApprovalDecision::NeedsChanges),
        _ => Err(StoreError::invalid_data(format!(
            "unknown approval decision: {s}"
        ))),
    }
}
