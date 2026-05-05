use authority_domain::{ProjectId, SnapshotId};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    error::ApiError,
    models::{
        CreateSnapshotRequest, SnapshotCompareResponse, SnapshotResponse, SnapshotValidateResponse,
    },
};

#[derive(Deserialize, IntoParams)]
pub struct ListSnapshotsParams {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[derive(Deserialize, IntoParams)]
pub struct CompareSnapshotsParams {
    #[param(example = "snap_001")]
    from_id: String,
    #[param(example = "snap_002")]
    to_id: String,
}

/// POST /v1/snapshots
#[utoipa::path(
    post,
    path = "/v1/snapshots",
    operation_id = "create_snapshot",
    summary = "Create a new content-addressed snapshot manifest.",
    tag = "snapshots",
    request_body = CreateSnapshotRequest,
    responses(
        (status = 201, description = "Snapshot created", body = SnapshotResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn create_snapshot(
    State(ctx): State<ServiceContext>,
    Json(req): Json<CreateSnapshotRequest>,
) -> Result<(StatusCode, Json<SnapshotResponse>), ApiError> {
    let manifest = ctx
        .snapshots
        .create_snapshot(
            &req.tenant_id,
            &req.project_id,
            &req.reason,
            req.parent_snapshot_id,
            vec![],
            vec![],
            req.previous_manifest_hash,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(SnapshotResponse {
            snapshot_id: manifest.id,
            status: format!("{:?}", manifest.status),
            reason: manifest.reason,
            object_count: manifest.object_refs.len(),
        }),
    ))
}

/// GET /v1/snapshots
#[utoipa::path(
    get,
    path = "/v1/snapshots",
    operation_id = "list_snapshots",
    summary = "List snapshots for the default project with pagination.",
    tag = "snapshots",
    params(ListSnapshotsParams),
    responses(
        (status = 200, description = "Snapshots listed", body = Vec<SnapshotResponse>),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn list_snapshots(
    State(ctx): State<ServiceContext>,
    Query(params): Query<ListSnapshotsParams>,
) -> Result<Json<Vec<SnapshotResponse>>, ApiError> {
    let project_id = ProjectId::new("default").map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let manifests = ctx
        .snapshots
        .list_snapshots(
            &project_id,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await?;

    let responses = manifests
        .into_iter()
        .map(|m| SnapshotResponse {
            snapshot_id: m.id,
            status: format!("{:?}", m.status),
            reason: m.reason,
            object_count: m.object_refs.len(),
        })
        .collect();

    Ok(Json(responses))
}

/// GET /v1/snapshots/:id
#[utoipa::path(
    get,
    path = "/v1/snapshots/{id}",
    operation_id = "get_snapshot",
    summary = "Get a snapshot by ID, returning its validation report.",
    tag = "snapshots",
    params(("id" = String, Path, description = "Snapshot ID")),
    responses(
        (status = 200, description = "Snapshot found", body = SnapshotValidateResponse),
        (status = 404, description = "Snapshot not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_snapshot(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SnapshotId>,
) -> Result<Json<SnapshotValidateResponse>, ApiError> {
    let report = ctx.snapshots.validate_snapshot(&id).await?;
    Ok(Json(SnapshotValidateResponse { report }))
}

/// POST /v1/snapshots/:id/validate
#[utoipa::path(
    post,
    path = "/v1/snapshots/{id}/validate",
    operation_id = "validate_snapshot",
    summary = "Validate a snapshot's manifest hash integrity and object reference consistency.",
    tag = "snapshots",
    params(("id" = String, Path, description = "Snapshot ID")),
    responses(
        (status = 200, description = "Validation report", body = SnapshotValidateResponse),
        (status = 404, description = "Snapshot not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn validate_snapshot(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SnapshotId>,
) -> Result<Json<SnapshotValidateResponse>, ApiError> {
    let report = ctx.snapshots.validate_snapshot(&id).await?;
    Ok(Json(SnapshotValidateResponse { report }))
}

/// GET /v1/snapshots/compare
#[utoipa::path(
    get,
    path = "/v1/snapshots/compare",
    operation_id = "compare_snapshots",
    summary = "Compare two snapshots and return the delta (added, removed, changed objects).",
    tag = "snapshots",
    params(CompareSnapshotsParams),
    responses(
        (status = 200, description = "Snapshot delta", body = SnapshotCompareResponse),
        (status = 404, description = "One or both snapshots not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn compare_snapshots(
    State(ctx): State<ServiceContext>,
    Query(params): Query<CompareSnapshotsParams>,
) -> Result<Json<SnapshotCompareResponse>, ApiError> {
    let from_id = SnapshotId::new(&params.from_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid from_id: {e}")))?;
    let to_id = SnapshotId::new(&params.to_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid to_id: {e}")))?;
    let delta = ctx.snapshots.compare_snapshots(&from_id, &to_id).await?;
    Ok(Json(SnapshotCompareResponse { delta }))
}
