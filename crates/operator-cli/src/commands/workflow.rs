use authority_domain::{ProjectId, TenantId, WorkflowContext};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use workflow_engine::Orchestrator;

use crate::util::CliResult;

/// Subcommands for workflow lifecycle management.
#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Run a full workflow lifecycle through the stage orchestrator
    Run(RunArgs),
}

/// Arguments for the workflow run command.
#[derive(Args)]
pub struct RunArgs {
    /// Unique workflow instance ID
    workflow_id: String,
    /// Tenant this workflow belongs to
    tenant_id: String,
    /// Project this workflow belongs to
    project_id: String,
    /// Include extended stages (Design, Council)
    #[arg(long, default_value_t = false)]
    include_extended: bool,
}

pub async fn handle_workflow(cmd: WorkflowCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        WorkflowCommand::Run(args) => handle_run(args, ctx).await,
    }
}

async fn handle_run(args: RunArgs, _ctx: &ServiceContext) -> CliResult {
    let tid = TenantId::new(&args.tenant_id).map_err(|e| format!("Invalid tenant_id: {}", e))?;
    let pid = ProjectId::new(&args.project_id).map_err(|e| format!("Invalid project_id: {}", e))?;

    let ctx = WorkflowContext::new(&args.workflow_id, tid, pid, args.include_extended);
    let mut orchestrator = Orchestrator::new(ctx);

    match orchestrator.run_full_workflow().await {
        Ok(final_ctx) => {
            let json = serde_json::json!({
                "workflow_id": final_ctx.workflow_id,
                "complete": final_ctx.is_complete(),
                "final_stage": final_ctx.current_stage.label(),
                "stages_completed": final_ctx.stage_statuses.values().filter(|s| **s == authority_domain::StageStatus::Completed).count(),
                "results_count": final_ctx.stage_results.len(),
                "status": "ok",
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        Err(e) => {
            let partial = &orchestrator.context;
            let json = serde_json::json!({
                "workflow_id": partial.workflow_id,
                "complete": false,
                "final_stage": format!("{} (blocked: {})", partial.current_stage.label(), e),
                "stages_completed": partial.stage_statuses.values().filter(|s| **s == authority_domain::StageStatus::Completed).count(),
                "results_count": partial.stage_results.len(),
                "status": "blocked",
                "error": e.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}
