use authority_domain::{BoardPlanId, PlanStatus};
use clap::Subcommand;
use control_service::ServiceContext;

use crate::util::CliResult;

/// Subcommands for boards (planning, approval, releases).
#[derive(Subcommand)]
pub enum BoardsCommand {
    /// Create a new board plan
    CreatePlan {
        title: String,
        #[arg(long)]
        work_path_refs: Option<Vec<String>>,
    },
    /// List board plans
    ListPlans {
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value = "20")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Submit a plan for approval
    SubmitPlan {
        plan_id: String,
    },
    /// Approve a plan
    ApprovePlan {
        plan_id: String,
        approver: String,
        #[arg(long)]
        comment: Option<String>,
    },
    /// Reject a plan
    RejectPlan {
        plan_id: String,
        approver: String,
        #[arg(long)]
        comment: Option<String>,
    },
    /// Submit a release command
    SubmitRelease {
        plan_id: String,
        bundle_ref: String,
        approval_ref: String,
    },
}

pub async fn handle_boards(cmd: BoardsCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        BoardsCommand::CreatePlan { title, work_path_refs } => {
            let plan = ctx
                .boards
                .create_plan(title, work_path_refs.unwrap_or_default())
                .await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        BoardsCommand::ListPlans { status, limit, offset } => {
            let status_filter = status.as_ref().and_then(|s| match s.as_str() {
                "draft" => Some(PlanStatus::Draft),
                "in_review" => Some(PlanStatus::InReview),
                "approved" => Some(PlanStatus::Approved),
                "in_progress" => Some(PlanStatus::InProgress),
                "completed" => Some(PlanStatus::Completed),
                "blocked" => Some(PlanStatus::Blocked),
                "archived" => Some(PlanStatus::Archived),
                _ => None,
            });
            let plans = ctx.boards.list_plans(status_filter, limit, offset).await?;
            println!("{}", serde_json::to_string_pretty(&plans)?);
        }
        BoardsCommand::SubmitPlan { plan_id } => {
            let id = BoardPlanId::new(&plan_id)?;
            let plan = ctx.boards.submit_for_approval(&id).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        BoardsCommand::ApprovePlan { plan_id, approver, comment } => {
            let id = BoardPlanId::new(&plan_id)?;
            let plan = ctx.boards.approve_plan(&id, approver, comment).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        BoardsCommand::RejectPlan { plan_id, approver, comment } => {
            let id = BoardPlanId::new(&plan_id)?;
            let plan = ctx
                .boards
                .reject_plan(&id, approver, comment.unwrap_or_default())
                .await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        BoardsCommand::SubmitRelease { plan_id, bundle_ref, approval_ref } => {
            let id = BoardPlanId::new(&plan_id)?;
            let cmd = ctx
                .boards
                .submit_release_command(&id, bundle_ref, approval_ref)
                .await?;
            println!("{}", serde_json::to_string_pretty(&cmd)?);
        }
    }
    Ok(())
}
