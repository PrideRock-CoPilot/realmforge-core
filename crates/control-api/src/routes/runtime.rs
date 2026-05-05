use authority_domain::{BundleId, RuntimeId};
use axum::{extract::Path, extract::State, Json};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use utoipa::IntoParams;

/// GET /v1/runtime/:id
#[utoipa::path(
    get,
    path = "/v1/runtime/{id}",
    operation_id = "get_runtime_status",
    summary = "Get the current status of a runtime instance.",
    tag = "runtime",
    params(("id" = String, Path, description = "Runtime ID")),
    responses(
        (status = 200, description = "Runtime instance status"),
        (status = 404, description = "Runtime not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_runtime_status(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Json<Value> {
    let runtime_id = RuntimeId::new(id).unwrap_or_else(|_| RuntimeId::generate());

    match ctx.runtimes.get_runtime_status(&runtime_id).await {
        Ok(instance) => Json(json!({
            "success": true,
            "data": instance
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

#[derive(Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ListRuntimesQuery {
    #[param(example = 20)]
    pub limit: Option<i64>,
    #[param(example = 0)]
    pub offset: Option<i64>,
}

/// GET /v1/runtime
#[utoipa::path(
    get,
    path = "/v1/runtime",
    operation_id = "list_runtimes",
    summary = "List all active runtime instances.",
    tag = "runtime",
    params(ListRuntimesQuery),
    responses(
        (status = 200, description = "Runtime instance list"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_runtimes(
    State(ctx): State<ServiceContext>,
    axum::extract::Query(query): axum::extract::Query<ListRuntimesQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    match ctx.runtimes.list_runtimes(limit, offset).await {
        Ok(instances) => Json(json!({
            "success": true,
            "data": instances
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeployBundleRequest {
    #[schema(example = "runtime_01")]
    pub runtime_id: String,
    #[schema(example = "bundle_01")]
    pub bundle_id: String,
    pub active_sessions: Option<i64>,
    #[schema(value_type = Object)]
    pub metadata: Option<HashMap<String, Value>>,
}

/// POST /v1/runtime/deploy
#[utoipa::path(
    post,
    path = "/v1/runtime/deploy",
    operation_id = "deploy_runtime",
    summary = "Deploy a bundle as a new runtime instance.",
    tag = "runtime",
    request_body = DeployBundleRequest,
    responses(
        (status = 200, description = "Runtime deployed"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn deploy_runtime(
    State(ctx): State<ServiceContext>,
    Json(body): Json<DeployBundleRequest>,
) -> Json<Value> {
    let instance = authority_domain::RuntimeInstance {
        runtime_id: RuntimeId::new(body.runtime_id).unwrap_or_else(|_| RuntimeId::generate()),
        bundle_id: BundleId::new(body.bundle_id).unwrap_or_else(|_| BundleId::generate()),
        status: "running".to_string(),
        started_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
        active_sessions: body.active_sessions.unwrap_or(0),
        action_count: 0,
        error_count: 0,
        metadata: serde_json::to_value(body.metadata.unwrap_or_default()).unwrap_or_default(),
    };

    match ctx.runtimes.deploy_bundle(&instance).await {
        Ok(()) => Json(json!({
            "success": true,
            "data": instance
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// POST /v1/runtime/:id/stop
#[utoipa::path(
    post,
    path = "/v1/runtime/{id}/stop",
    operation_id = "stop_runtime",
    summary = "Stop a running runtime instance.",
    tag = "runtime",
    params(("id" = String, Path, description = "Runtime ID")),
    responses(
        (status = 200, description = "Runtime stopped"),
        (status = 404, description = "Runtime not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn stop_runtime(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Json<Value> {
    let runtime_id = RuntimeId::new(id).unwrap_or_else(|_| RuntimeId::generate());

    match ctx.runtimes.stop_runtime(&runtime_id).await {
        Ok(()) => Json(json!({
            "success": true,
            "data": { "runtime_id": runtime_id.to_string(), "status": "stopped" }
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}
