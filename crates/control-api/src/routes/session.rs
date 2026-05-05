use authority_domain::SessionId;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;

use crate::{
    error::ApiError,
    models::{IssueSessionRequest, RenewSessionRequest, RevokeSessionResponse, SessionResponse},
};

/// POST /v1/session
#[utoipa::path(
    post,
    path = "/v1/session",
    operation_id = "issue_session",
    summary = "Issue a new session for an actor within a tenant and project.",
    tag = "sessions",
    request_body = IssueSessionRequest,
    responses(
        (status = 201, description = "Session issued", body = SessionResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn issue_session(
    State(ctx): State<ServiceContext>,
    Json(req): Json<IssueSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let session = ctx
        .sessions
        .issue_session(
            &req.actor_id,
            &req.tenant_id,
            &req.project_id,
            req.ttl_seconds,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(SessionResponse { session })))
}

/// GET /v1/session/:id
#[utoipa::path(
    get,
    path = "/v1/session/{id}",
    operation_id = "get_session",
    summary = "Get session details by ID.",
    tag = "sessions",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session found", body = SessionResponse),
        (status = 404, description = "Session not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_session(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SessionId>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = ctx.sessions.validate_session(&id).await?;
    Ok(Json(SessionResponse { session }))
}

/// DELETE /v1/session/:id
#[utoipa::path(
    delete,
    path = "/v1/session/{id}",
    operation_id = "revoke_session",
    summary = "Revoke a session, preventing further use.",
    tag = "sessions",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session revoked", body = RevokeSessionResponse),
        (status = 404, description = "Session not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn revoke_session(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SessionId>,
) -> Result<Json<RevokeSessionResponse>, ApiError> {
    ctx.sessions.revoke_session(&id, "api-revoked").await?;
    Ok(Json(RevokeSessionResponse {
        status: "revoked".to_string(),
        session_id: id.as_str().to_string(),
    }))
}

/// POST /v1/session/:id/activate
#[utoipa::path(
    post,
    path = "/v1/session/{id}/activate",
    operation_id = "activate_session",
    summary = "Activate an issued session, transitioning it from issued to active.",
    tag = "sessions",
    params(("id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session activated", body = SessionResponse),
        (status = 404, description = "Session not found", body = crate::error::ProblemDetails),
        (status = 400, description = "Invalid session state transition", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn activate_session(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SessionId>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = ctx.sessions.activate_session(&id).await?;
    Ok(Json(SessionResponse { session }))
}

/// POST /v1/session/:id/renew
#[utoipa::path(
    post,
    path = "/v1/session/{id}/renew",
    operation_id = "renew_session",
    summary = "Extend a session's TTL.",
    tag = "sessions",
    params(("id" = String, Path, description = "Session ID")),
    request_body = RenewSessionRequest,
    responses(
        (status = 200, description = "Session renewed", body = SessionResponse),
        (status = 404, description = "Session not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn renew_session(
    State(ctx): State<ServiceContext>,
    Path(id): Path<SessionId>,
    Json(req): Json<RenewSessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = ctx.sessions.renew_session(&id, req.ttl_seconds).await?;
    Ok(Json(SessionResponse { session }))
}
