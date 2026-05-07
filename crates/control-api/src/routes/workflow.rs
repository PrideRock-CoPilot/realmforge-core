use authority_domain::{ProjectId, TenantId, WorkflowContext};
use axum::{extract::State, http::StatusCode, Json};
use control_service::ServiceContext;
use serde::{Deserialize, Serialize};
use workflow_engine::Orchestrator;

use crate::error::ApiError;

/// Request to run a full workflow lifecycle.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct RunWorkflowRequest {
    /// Unique workflow instance ID (e.g., "wf-001").
    pub workflow_id: String,
    /// Tenant this workflow belongs to.
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    /// Project this workflow belongs to.
    #[schema(value_type = String)]
    pub project_id: ProjectId,
    /// Whether to include extended stages (Design, Council).
    #[serde(default)]
    pub include_extended: bool,
}

/// Response from a workflow run.
#[derive(Serialize, utoipa::ToSchema)]
pub struct WorkflowRunResponse {
    /// The workflow ID.
    pub workflow_id: String,
    /// Whether the full workflow completed successfully.
    pub complete: bool,
    /// The final stage reached.
    pub final_stage: String,
    /// Number of stages completed.
    pub stages_completed: usize,
    /// Number of stage results recorded.
    pub results_count: usize,
}

/// POST /v1/workflow/run
///
/// Run a full workflow lifecycle through the stage orchestrator.
/// Starts from Idea and progresses through all stages until completion,
/// a gate failure, or a blocking decision.
#[utoipa::path(
    post,
    path = "/v1/workflow/run",
    operation_id = "run_workflow",
    summary = "Run a full workflow lifecycle through the stage orchestrator.",
    tag = "workflow",
    request_body = RunWorkflowRequest,
    responses(
        (status = 200, description = "Workflow completed or stopped at a blocked stage", body = WorkflowRunResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn run_workflow(
    State(_ctx): State<ServiceContext>,
    Json(req): Json<RunWorkflowRequest>,
) -> Result<(StatusCode, Json<WorkflowRunResponse>), ApiError> {
    let ctx = WorkflowContext::new(
        &req.workflow_id,
        req.tenant_id,
        req.project_id,
        req.include_extended,
    );

    let mut orchestrator = Orchestrator::new(ctx);

    // Run the full workflow. The orchestrator handles gate checks and
    // stops at any stage that fails or gets blocked.
    match orchestrator.run_full_workflow().await {
        Ok(final_ctx) => Ok((
            StatusCode::OK,
            Json(WorkflowRunResponse {
                workflow_id: final_ctx.workflow_id.clone(),
                complete: final_ctx.is_complete(),
                final_stage: final_ctx.current_stage.label().to_string(),
                stages_completed: final_ctx
                    .stage_statuses
                    .values()
                    .filter(|s| **s == authority_domain::StageStatus::Completed)
                    .count(),
                results_count: final_ctx.stage_results.len(),
            }),
        )),
        Err(e) => {
            // Gate failure — return partial results
            let partial = &orchestrator.context;
            Ok((
                StatusCode::OK,
                Json(WorkflowRunResponse {
                    workflow_id: partial.workflow_id.clone(),
                    complete: false,
                    final_stage: format!("{} (blocked: {})", partial.current_stage.label(), e),
                    stages_completed: partial
                        .stage_statuses
                        .values()
                        .filter(|s| **s == authority_domain::StageStatus::Completed)
                        .count(),
                    results_count: partial.stage_results.len(),
                }),
            ))
        }
    }
}
