use authority_domain::WorkflowContext;
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::Value;
use workflow_engine::Orchestrator;

use crate::error::McpError;

/// Arguments for the core_run_workflow tool.
#[derive(Deserialize)]
pub struct RunWorkflowArgs {
    pub workflow_id: String,
    pub tenant_id: String,
    pub project_id: String,
    #[serde(default)]
    pub include_extended: bool,
}

/// core_run_workflow — Run a full workflow lifecycle through the stage orchestrator.
pub async fn core_run_workflow(
    args: RunWorkflowArgs,
    _ctx: &ServiceContext,
) -> Result<Value, McpError> {
    use authority_domain::{ProjectId, TenantId};

    let tid = TenantId::new(&args.tenant_id)
        .map_err(|e| McpError::InvalidArgs(format!("Invalid tenant_id: {}", e)))?;
    let pid = ProjectId::new(&args.project_id)
        .map_err(|e| McpError::InvalidArgs(format!("Invalid project_id: {}", e)))?;

    let ctx = WorkflowContext::new(&args.workflow_id, tid, pid, args.include_extended);
    let mut orchestrator = Orchestrator::new(ctx);

    match orchestrator.run_full_workflow().await {
        Ok(final_ctx) => {
            let response = serde_json::json!({
                "workflow_id": final_ctx.workflow_id,
                "complete": final_ctx.is_complete(),
                "final_stage": final_ctx.current_stage.label(),
                "stages_completed": final_ctx.stage_statuses.values().filter(|s| **s == authority_domain::StageStatus::Completed).count(),
                "results_count": final_ctx.stage_results.len(),
                "status": "ok",
            });
            Ok(response)
        }
        Err(e) => {
            let partial = &orchestrator.context;
            let response = serde_json::json!({
                "workflow_id": partial.workflow_id,
                "complete": false,
                "final_stage": format!("{} (blocked: {})", partial.current_stage.label(), e),
                "stages_completed": partial.stage_statuses.values().filter(|s| **s == authority_domain::StageStatus::Completed).count(),
                "results_count": partial.stage_results.len(),
                "status": "blocked",
                "error": e.to_string(),
            });
            Ok(response)
        }
    }
}
