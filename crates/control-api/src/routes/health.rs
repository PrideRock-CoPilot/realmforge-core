use crate::models::HealthResponse;
use axum::Json;

/// GET /health
#[utoipa::path(
    get,
    path = "/health",
    operation_id = "health",
    summary = "Basic liveness check.",
    tag = "health",
    responses(
        (status = 200, description = "Service is live", body = HealthResponse),
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "control-api",
        production_deploy_enabled: false,
    })
}

/// GET /v1/health/ready
#[utoipa::path(
    get,
    path = "/v1/health/ready",
    operation_id = "health_ready",
    summary = "Readiness probe — service is ready to accept traffic.",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", body = HealthResponse),
    )
)]
pub async fn ready() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "control-api",
        production_deploy_enabled: false,
    })
}

/// GET /v1/health/live
#[utoipa::path(
    get,
    path = "/v1/health/live",
    operation_id = "health_live",
    summary = "Liveness probe — service process is running.",
    tag = "health",
    responses(
        (status = 200, description = "Service is live", body = HealthResponse),
    )
)]
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "control-api",
        production_deploy_enabled: false,
    })
}
