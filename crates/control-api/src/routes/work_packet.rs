use authority_domain::{work_packet::CostBudget, ActorId, PacketId, ProjectId, TenantId, WorkPathId, WorkPathNodeId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;
use serde::Deserialize;

use crate::{
    error::ApiError,
    models::{WorkPacketGenerateResponse, WorkPacketValidateResponse},
};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct GenerateWorkPacketRequest {
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    #[schema(value_type = String)]
    pub project_id: ProjectId,
    #[schema(value_type = String)]
    pub actor_id: ActorId,
    #[schema(value_type = String)]
    pub work_path_id: WorkPathId,
    #[schema(value_type = String)]
    pub node_id: WorkPathNodeId,
    pub objective: String,
    #[schema(value_type = Option<Object>)]
    pub cost_budget: Option<CostBudget>,
}

/// POST /v1/work-packets/generate
#[utoipa::path(
    post,
    path = "/v1/work-packets/generate",
    operation_id = "generate_work_packet",
    summary = "Generate a scoped work packet for an AI agent from a work path node.",
    tag = "work-packets",
    request_body = GenerateWorkPacketRequest,
    responses(
        (status = 201, description = "Work packet generated", body = WorkPacketGenerateResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn generate_work_packet(
    State(ctx): State<ServiceContext>,
    Json(req): Json<GenerateWorkPacketRequest>,
) -> Result<(StatusCode, Json<WorkPacketGenerateResponse>), ApiError> {
    let packet = ctx
        .work_packets
        .generate_work_packet(
            &req.tenant_id,
            &req.project_id,
            &req.actor_id,
            &req.work_path_id,
            &req.node_id,
            &req.objective,
            req.cost_budget,
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(WorkPacketGenerateResponse {
            packet_id: packet.id,
            status: format!("{:?}", packet.status),
        }),
    ))
}

/// POST /v1/work-packets/:id/validate
#[utoipa::path(
    post,
    path = "/v1/work-packets/{id}/validate",
    operation_id = "validate_work_packet",
    summary = "Validate a work packet's scope boundaries and cost budget.",
    tag = "work-packets",
    params(("id" = String, Path, description = "Work packet ID")),
    responses(
        (status = 200, description = "Validation result", body = WorkPacketValidateResponse),
        (status = 404, description = "Packet not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn validate_work_packet(
    State(ctx): State<ServiceContext>,
    Path(id): Path<PacketId>,
) -> Result<Json<WorkPacketValidateResponse>, ApiError> {
    let result = ctx.work_packets.validate_packet_boundaries(&id).await?;
    Ok(Json(WorkPacketValidateResponse { result }))
}
