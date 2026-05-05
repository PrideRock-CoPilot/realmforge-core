use agent_gateway::flow::GatewayRequest;
use axum::{extract::State, Json};
use control_service::ServiceContext;
use serde_json::Value;

/// POST /v1/gateway/execute
#[utoipa::path(
    post,
    path = "/v1/gateway/execute",
    operation_id = "execute_gateway",
    summary = "Execute a command through the agent gateway. Every agent write must go through this endpoint.",
    tag = "gateway",
    request_body(content = Object, description = "GatewayRequest: actor_id, session_id, action, file_paths, schema_views"),
    responses(
        (status = 200, description = "Command executed"),
        (status = 403, description = "Unauthorized by policy"),
        (status = 500, description = "Execution error"),
    )
)]
pub async fn execute_gateway(
    State(ctx): State<ServiceContext>,
    Json(request): Json<GatewayRequest>,
) -> Json<Value> {
    match ctx.gateway.execute_command(request).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "data": result
        })),
        Err(err) => {
            let status = err.status_code();
            Json(serde_json::json!({
                "success": false,
                "error": err.problem_detail(),
                "status": status
            }))
        }
    }
}
