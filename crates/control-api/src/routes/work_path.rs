use authority_domain::{WorkPathGraph, WorkPathId, WorkPathNodeId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

/// POST /v1/work-paths
#[utoipa::path(
    post,
    path = "/v1/work-paths",
    operation_id = "create_work_path",
    summary = "Create a new work path graph.",
    tag = "work-paths",
    request_body = CreateWorkPathRequest,
    responses(
        (status = 201, description = "Work path created", body = WorkPathGraphResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn create_work_path(
    State(ctx): State<ServiceContext>,
    Json(req): Json<CreateWorkPathRequest>,
) -> Result<(StatusCode, Json<WorkPathGraphResponse>), ApiError> {
    let graph = ctx
        .work_paths
        .create_work_path(&req.id, &req.name, &req.description)
        .await?;
    Ok((StatusCode::CREATED, Json(graph.into())))
}

/// GET /v1/work-paths/:id
#[utoipa::path(
    get,
    path = "/v1/work-paths/{id}",
    operation_id = "get_work_path",
    summary = "Get a work path graph by ID.",
    tag = "work-paths",
    params(("id" = String, Path, description = "Work path ID")),
    responses(
        (status = 200, description = "Work path found", body = WorkPathGraphResponse),
        (status = 404, description = "Work path not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_work_path(
    State(ctx): State<ServiceContext>,
    Path(id): Path<WorkPathId>,
) -> Result<Json<WorkPathGraphResponse>, ApiError> {
    let graph = ctx.work_paths.get_work_path_graph(&id).await?;
    Ok(Json(graph.into()))
}

/// POST /v1/work-paths/:id/traverse
#[utoipa::path(
    post,
    path = "/v1/work-paths/{id}/traverse",
    operation_id = "traverse_work_path",
    summary = "Traverse a work path node and generate a scoped work packet.",
    tag = "work-paths",
    params(("id" = String, Path, description = "Work path ID")),
    request_body = TraverseWorkPathRequest,
    responses(
        (status = 201, description = "Traversal complete, packet generated", body = TraverseResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 404, description = "Work path not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn traverse_work_path(
    State(ctx): State<ServiceContext>,
    Path(id): Path<WorkPathId>,
    Json(req): Json<TraverseWorkPathRequest>,
) -> Result<(StatusCode, Json<TraverseResponse>), ApiError> {
    let packet = ctx
        .work_paths
        .traverse_to_packet(
            &id,
            &req.node_id,
            &req.agent_id,
            &req.tenant_id,
            &req.project_id,
            &req.objective,
            None,
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(TraverseResponse {
            packet_id: packet.id.to_string(),
            allowed_file_count: packet.allowed_file_paths.len(),
            required_contract_count: packet.required_contracts.len(),
            status: format!("{:?}", packet.status),
        }),
    ))
}

// ── Request/Response types ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkPathRequest {
    #[schema(value_type = String, example = "wp_01")]
    pub id: WorkPathId,
    #[schema(example = "Feature Delivery")]
    pub name: String,
    #[schema(example = "End-to-end path for feature delivery")]
    pub description: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TraverseWorkPathRequest {
    #[schema(value_type = String, example = "node_01")]
    pub node_id: WorkPathNodeId,
    #[schema(value_type = String, example = "agent_01")]
    pub agent_id: authority_domain::ActorId,
    #[schema(value_type = String, example = "ten_01")]
    pub tenant_id: authority_domain::TenantId,
    #[schema(value_type = String, example = "proj_01")]
    pub project_id: authority_domain::ProjectId,
    #[schema(example = "Implement the login vertical")]
    pub objective: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkPathGraphResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TraverseResponse {
    pub packet_id: String,
    pub allowed_file_count: usize,
    pub required_contract_count: usize,
    pub status: String,
}

impl From<WorkPathGraph> for WorkPathGraphResponse {
    fn from(g: WorkPathGraph) -> Self {
        Self {
            node_count: g.nodes.len(),
            id: g.id.to_string(),
            name: g.name,
            description: g.description,
        }
    }
}
