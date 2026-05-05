use authority_domain::{BoardPlanId, PlanStatus};
use axum::{extract::Path, extract::Query, extract::State, http::StatusCode, Json};
use control_service::ServiceContext;

use crate::{
    error::ApiError,
    models::{
        BoardApprovalRequest, BoardPlanListResponse, BoardPlanResponse, BoardReleaseRequest,
        BoardReleaseResponse, CreateBoardPlanRequest, ListPlansQuery,
    },
};

fn plan_to_response(plan: impl serde::Serialize) -> Result<BoardPlanResponse, ApiError> {
    // Service layer returns a serializable plan type; round-trip through Value to extract fields.
    let v = serde_json::to_value(&plan)
        .map_err(|e| ApiError::Internal(format!("plan serialization: {e}")))?;
    serde_json::from_value(v).map_err(|e| ApiError::Internal(format!("plan deserialization: {e}")))
}

/// POST /v1/boards/plans
#[utoipa::path(
    post,
    path = "/v1/boards/plans",
    operation_id = "create_board_plan",
    summary = "Create a new board plan for a work path.",
    tag = "boards",
    request_body = CreateBoardPlanRequest,
    responses(
        (status = 201, description = "Plan created", body = BoardPlanResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn create_plan(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CreateBoardPlanRequest>,
) -> Result<(StatusCode, Json<BoardPlanResponse>), ApiError> {
    let refs = body.work_path_refs.unwrap_or_default();
    let plan = ctx
        .boards
        .create_plan(body.title, refs)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let response = plan_to_response(plan)?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /v1/boards/plans
#[utoipa::path(
    get,
    path = "/v1/boards/plans",
    operation_id = "list_board_plans",
    summary = "List board plans with optional status filter and pagination.",
    tag = "boards",
    params(ListPlansQuery),
    responses(
        (status = 200, description = "Plans listed", body = BoardPlanListResponse),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn list_plans(
    State(ctx): State<ServiceContext>,
    Query(query): Query<ListPlansQuery>,
) -> Result<Json<BoardPlanListResponse>, ApiError> {
    let status_filter = query.status.as_deref().and_then(|s| match s {
        "draft" => Some(PlanStatus::Draft),
        "in_review" => Some(PlanStatus::InReview),
        "approved" => Some(PlanStatus::Approved),
        "in_progress" => Some(PlanStatus::InProgress),
        "completed" => Some(PlanStatus::Completed),
        "blocked" => Some(PlanStatus::Blocked),
        "archived" => Some(PlanStatus::Archived),
        _ => None,
    });

    let plans = ctx
        .boards
        .list_plans(
            status_filter,
            query.limit.unwrap_or(20),
            query.offset.unwrap_or(0),
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items: Vec<BoardPlanResponse> = plans
        .into_iter()
        .map(plan_to_response)
        .collect::<Result<_, _>>()?;

    let total = items.len() as u64;
    Ok(Json(BoardPlanListResponse {
        plans: items,
        total,
    }))
}

/// POST /v1/boards/plans/:id/submit
#[utoipa::path(
    post,
    path = "/v1/boards/plans/{id}/submit",
    operation_id = "submit_board_plan",
    summary = "Submit a draft board plan for approval.",
    tag = "boards",
    params(("id" = String, Path, description = "Board plan ID")),
    responses(
        (status = 200, description = "Plan submitted for review", body = BoardPlanResponse),
        (status = 400, description = "Invalid plan ID or state transition", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn submit_plan(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Result<Json<BoardPlanResponse>, ApiError> {
    let plan_id =
        BoardPlanId::new(&id).map_err(|e| ApiError::BadRequest(format!("invalid plan id: {e}")))?;
    let plan = ctx
        .boards
        .submit_for_approval(&plan_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(plan_to_response(plan)?))
}

/// POST /v1/boards/plans/:id/approve
#[utoipa::path(
    post,
    path = "/v1/boards/plans/{id}/approve",
    operation_id = "approve_board_plan",
    summary = "Approve a board plan submitted for review.",
    tag = "boards",
    params(("id" = String, Path, description = "Board plan ID")),
    request_body = BoardApprovalRequest,
    responses(
        (status = 200, description = "Plan approved", body = BoardPlanResponse),
        (status = 400, description = "Invalid request or state", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn approve_plan(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<BoardApprovalRequest>,
) -> Result<Json<BoardPlanResponse>, ApiError> {
    let plan_id =
        BoardPlanId::new(&id).map_err(|e| ApiError::BadRequest(format!("invalid plan id: {e}")))?;
    let plan = ctx
        .boards
        .approve_plan(&plan_id, body.approver, body.comment)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(plan_to_response(plan)?))
}

/// POST /v1/boards/plans/:id/reject
#[utoipa::path(
    post,
    path = "/v1/boards/plans/{id}/reject",
    operation_id = "reject_board_plan",
    summary = "Reject a board plan and return it to draft.",
    tag = "boards",
    params(("id" = String, Path, description = "Board plan ID")),
    request_body = BoardApprovalRequest,
    responses(
        (status = 200, description = "Plan rejected", body = BoardPlanResponse),
        (status = 400, description = "Invalid request or state", body = crate::error::ProblemDetails),
        (status = 404, description = "Plan not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn reject_plan(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<BoardApprovalRequest>,
) -> Result<Json<BoardPlanResponse>, ApiError> {
    let plan_id =
        BoardPlanId::new(&id).map_err(|e| ApiError::BadRequest(format!("invalid plan id: {e}")))?;
    let plan = ctx
        .boards
        .reject_plan(&plan_id, body.approver, body.comment.unwrap_or_default())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(plan_to_response(plan)?))
}

/// POST /v1/boards/releases
#[utoipa::path(
    post,
    path = "/v1/boards/releases",
    operation_id = "submit_board_release",
    summary = "Submit a release command from an approved board plan.",
    tag = "boards",
    request_body = BoardReleaseRequest,
    responses(
        (status = 201, description = "Release command submitted", body = BoardReleaseResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn submit_release(
    State(ctx): State<ServiceContext>,
    Json(body): Json<BoardReleaseRequest>,
) -> Result<(StatusCode, Json<BoardReleaseResponse>), ApiError> {
    let plan_id = BoardPlanId::new(&body.plan_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid plan id: {e}")))?;
    let cmd = ctx
        .boards
        .submit_release_command(&plan_id, body.bundle_ref, body.approval_ref)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let v = serde_json::to_value(&cmd)
        .map_err(|e| ApiError::Internal(format!("cmd serialization: {e}")))?;
    let command_id = v["id"].as_str().unwrap_or("").to_string();
    let status = v["status"].as_str().unwrap_or("proposed").to_string();

    Ok((
        StatusCode::CREATED,
        Json(BoardReleaseResponse {
            command_id,
            status,
            plan_id: body.plan_id,
        }),
    ))
}
