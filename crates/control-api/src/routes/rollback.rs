use authority_domain::SnapshotId;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;

use crate::{
    error::ApiError,
    models::{
        RollbackExecuteRequest, RollbackExecuteResponse, RollbackPreviewRequest,
        RollbackPreviewResponse, RollbackVerifyResponse,
    },
};

/// POST /v1/rollback/preview
#[utoipa::path(
    post,
    path = "/v1/rollback/preview",
    operation_id = "preview_rollback",
    summary = "Preview what a rollback between two snapshots would restore. Does not execute.",
    tag = "rollback",
    request_body = RollbackPreviewRequest,
    responses(
        (status = 200, description = "Rollback preview with impact report", body = RollbackPreviewResponse),
        (status = 404, description = "Snapshot not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn preview_rollback(
    State(ctx): State<ServiceContext>,
    Json(req): Json<RollbackPreviewRequest>,
) -> Result<Json<RollbackPreviewResponse>, ApiError> {
    let preview = ctx
        .rollback
        .preview_rollback(&req.from_snapshot_id, &req.to_snapshot_id)
        .await?;
    Ok(Json(RollbackPreviewResponse { preview }))
}

/// POST /v1/rollback/execute
#[utoipa::path(
    post,
    path = "/v1/rollback/execute",
    operation_id = "execute_rollback",
    summary = "Execute a rollback, restoring state from an earlier snapshot.",
    tag = "rollback",
    request_body = RollbackExecuteRequest,
    responses(
        (status = 201, description = "Rollback executed, new snapshot created", body = RollbackExecuteResponse),
        (status = 404, description = "Snapshot not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn execute_rollback(
    State(ctx): State<ServiceContext>,
    Json(req): Json<RollbackExecuteRequest>,
) -> Result<(StatusCode, Json<RollbackExecuteResponse>), ApiError> {
    let result = ctx
        .rollback
        .execute_rollback(
            &req.from_snapshot_id,
            &req.to_snapshot_id,
            &req.tenant_id,
            &req.project_id,
            &req.actor_id,
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(RollbackExecuteResponse { result }),
    ))
}

/// GET /v1/rollback/:id/verify
#[utoipa::path(
    get,
    path = "/v1/rollback/{id}/verify",
    operation_id = "verify_rollback",
    summary = "Verify a rollback snapshot's consistency and hash integrity.",
    tag = "rollback",
    params(("id" = String, Path, description = "Snapshot ID of the rollback result")),
    responses(
        (status = 200, description = "Rollback verification report", body = RollbackVerifyResponse),
        (status = 404, description = "Snapshot not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn verify_rollback(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SnapshotId>,
) -> Result<Json<RollbackVerifyResponse>, ApiError> {
    let verification = ctx.rollback.verify_rollback(&id).await?;
    Ok(Json(RollbackVerifyResponse { verification }))
}
