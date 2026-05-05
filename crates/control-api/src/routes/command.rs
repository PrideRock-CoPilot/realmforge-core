use authority_domain::{ActorScope, CommandId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use control_service::{PolicyDecision, ServiceContext};

use crate::{
    error::ApiError,
    models::{CommandResponse, CommandScopeRequest, DenyCommandRequest, ProposeCommandRequest},
};

/// POST /v1/commands
#[utoipa::path(
    post,
    path = "/v1/commands",
    operation_id = "propose_command",
    summary = "Propose a new bounded command within the command lifecycle.",
    tag = "commands",
    request_body = ProposeCommandRequest,
    responses(
        (status = 201, description = "Command proposed", body = CommandResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 403, description = "Policy denied", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn propose_command(
    State(ctx): State<ServiceContext>,
    Json(req): Json<ProposeCommandRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), ApiError> {
    let command = ctx
        .commands
        .propose_command(
            &req.scope,
            &req.action,
            &req.target_type,
            &req.target_id,
            req.payload,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CommandResponse {
            command_id: command.id,
            status: format!("{:?}", command.status),
            action: command.action,
            target_type: command.target_type,
            target_id: command.target_id,
        }),
    ))
}

/// PUT /v1/commands/:id/authorize
#[utoipa::path(
    put,
    path = "/v1/commands/{id}/authorize",
    operation_id = "authorize_command",
    summary = "Authorize a proposed command against the policy engine.",
    tag = "commands",
    params(("id" = String, Path, description = "Command ID")),
    request_body = CommandScopeRequest,
    responses(
        (status = 200, description = "Command authorized", body = CommandResponse),
        (status = 403, description = "Policy denied", body = crate::error::ProblemDetails),
        (status = 404, description = "Command not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn authorize_command(
    State(ctx): State<ServiceContext>,
    Path(id): Path<CommandId>,
    Json(scope): Json<ActorScope>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = ctx.commands.authorize_command(&id, &scope).await?;

    Ok(Json(CommandResponse {
        command_id: command.id,
        status: format!("{:?}", command.status),
        action: command.action,
        target_type: command.target_type,
        target_id: command.target_id,
    }))
}

/// PUT /v1/commands/:id/apply
#[utoipa::path(
    put,
    path = "/v1/commands/{id}/apply",
    operation_id = "apply_command",
    summary = "Apply an authorized command, executing its effects.",
    tag = "commands",
    params(("id" = String, Path, description = "Command ID")),
    request_body = CommandScopeRequest,
    responses(
        (status = 200, description = "Command applied", body = CommandResponse),
        (status = 400, description = "Command not in authorized state", body = crate::error::ProblemDetails),
        (status = 404, description = "Command not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn apply_command(
    State(ctx): State<ServiceContext>,
    Path(id): Path<CommandId>,
    Json(scope): Json<ActorScope>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = ctx.commands.apply_command(&id, &scope).await?;

    Ok(Json(CommandResponse {
        command_id: command.id,
        status: format!("{:?}", command.status),
        action: command.action,
        target_type: command.target_type,
        target_id: command.target_id,
    }))
}

/// GET /v1/commands/:id
#[utoipa::path(
    get,
    path = "/v1/commands/{id}",
    operation_id = "get_command",
    summary = "Retrieve a command by ID.",
    tag = "commands",
    params(("id" = String, Path, description = "Command ID")),
    responses(
        (status = 200, description = "Command found", body = CommandResponse),
        (status = 404, description = "Command not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_command(
    State(ctx): State<ServiceContext>,
    Path(id): Path<CommandId>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = ctx
        .commands
        .get_command(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("command {}", id.as_str())))?;

    Ok(Json(CommandResponse {
        command_id: command.id,
        status: format!("{:?}", command.status),
        action: command.action,
        target_type: command.target_type,
        target_id: command.target_id,
    }))
}

/// POST /v1/commands/:id/deny
#[utoipa::path(
    post,
    path = "/v1/commands/{id}/deny",
    operation_id = "deny_command",
    summary = "Explicitly deny a command, recording the reason in the audit trail.",
    tag = "commands",
    params(("id" = String, Path, description = "Command ID")),
    request_body = DenyCommandRequest,
    responses(
        (status = 200, description = "Command denied", body = CommandResponse),
        (status = 404, description = "Command not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn deny_command(
    State(ctx): State<ServiceContext>,
    Path(id): Path<CommandId>,
    Json(req): Json<DenyCommandRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let denial = PolicyDecision {
        allowed: false,
        denial: None,
    };
    let command = ctx.commands.deny_command(&id, &req.scope, &denial).await?;

    Ok(Json(CommandResponse {
        command_id: command.id,
        status: format!("{:?}", command.status),
        action: command.action,
        target_type: command.target_type,
        target_id: command.target_id,
    }))
}
