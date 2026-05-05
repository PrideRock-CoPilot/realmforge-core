use authority_domain::login::{LoginCredentials, Scope};
use axum::{extract::State, http::StatusCode, Json};
use control_service::ServiceContext;
use serde::Deserialize;

use crate::error::ApiError;
use crate::models::LoginResponse;

/// POST /v1/login
#[utoipa::path(
    post,
    path = "/v1/login",
    operation_id = "login",
    summary = "Authenticate an actor and issue a session token.",
    tag = "login",
    request_body = LoginRequest,
    responses(
        (status = 201, description = "Login successful, session issued", body = LoginResponse),
        (status = 400, description = "Invalid credentials or scope", body = crate::error::ProblemDetails),
        (status = 401, description = "Authentication failed", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn login(
    State(ctx): State<ServiceContext>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    let scope = Scope::new(&req.scope).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let credentials = LoginCredentials {
        actor_id: req.actor_id,
        credential: req.credential.into_bytes(),
        scope,
    };

    let response = ctx
        .login
        .handle_login(req.tenant_id, req.project_id, credentials)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(LoginResponse {
            session_token: response.session_token,
            actor_id: response.actor_id,
            scope: response.scope,
            expires_at: response.expires_at,
            audit_event_id: response.audit_event_id,
            snapshot_id: response.snapshot_id,
        }),
    ))
}

/// POST /v1/login/policy
#[utoipa::path(
    post,
    path = "/v1/login/policy",
    operation_id = "set_login_policy",
    summary = "Set the login policy (rate limits, block duration) for a tenant.",
    tag = "login",
    request_body = SetPolicyRequest,
    responses(
        (status = 200, description = "Policy updated"),
        (status = 400, description = "Invalid policy", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn set_login_policy(
    State(ctx): State<ServiceContext>,
    Json(req): Json<SetPolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    ctx.login.set_policy(&req.tenant_id, &req.config).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "policy updated", "tenant_id": req.tenant_id.as_str()})),
    ))
}

/// GET /v1/login/policy/:tenant_id
#[utoipa::path(
    get,
    path = "/v1/login/policy/{tenant_id}",
    operation_id = "get_login_policy",
    summary = "Get the active login policy for a tenant.",
    tag = "login",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Login policy"),
        (status = 404, description = "Tenant not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_login_policy(
    State(ctx): State<ServiceContext>,
    axum::extract::Path(tenant_id): axum::extract::Path<authority_domain::TenantId>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let policy = ctx.login.get_policy(&tenant_id).await?;
    Ok(Json(serde_json::json!({
        "tenant_id": tenant_id.as_str(),
        "config": policy,
    })))
}

/// GET /v1/login/blocks/:tenant_id
#[utoipa::path(
    get,
    path = "/v1/login/blocks/{tenant_id}",
    operation_id = "list_login_blocks",
    summary = "List active login blocks (rate-limit lockouts) for a tenant.",
    tag = "login",
    params(("tenant_id" = String, Path, description = "Tenant ID")),
    responses(
        (status = 200, description = "Active login blocks"),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn list_login_blocks(
    State(ctx): State<ServiceContext>,
    axum::extract::Path(tenant_id): axum::extract::Path<authority_domain::TenantId>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let blocks = ctx.login.list_blocks(&tenant_id).await?;
    Ok(Json(serde_json::json!({
        "tenant_id": tenant_id.as_str(),
        "blocks": blocks,
    })))
}

// ── Request types ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    #[schema(value_type = String, example = "ten_01")]
    pub tenant_id: authority_domain::TenantId,
    #[schema(value_type = String, example = "proj_01")]
    pub project_id: authority_domain::ProjectId,
    #[schema(value_type = String, example = "alice")]
    pub actor_id: authority_domain::ActorId,
    #[schema(example = "s3cr3t")]
    pub credential: String,
    #[schema(example = "read")]
    pub scope: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetPolicyRequest {
    #[schema(value_type = String, example = "ten_01")]
    pub tenant_id: authority_domain::TenantId,
    #[schema(value_type = Object)]
    pub config: authority_domain::login::LoginPolicyConfig,
}
