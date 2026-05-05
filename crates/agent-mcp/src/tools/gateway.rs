use agent_gateway::flow::GatewayRequest;
use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::types::McpToolResult;

/// core_execute_command — execute a command through the agent gateway.
///
/// This is the single tool that routes all agent command execution
/// through the gateway pipeline (authenticate → authorize → execute → audit → anchor).
pub async fn core_execute_gateway(
    args: GatewayRequest,
    ctx: &ServiceContext,
) -> Result<Value, crate::error::McpError> {
    match ctx.gateway.execute_command(args).await {
        Ok(result) => Ok(json!(McpToolResult::ok(json!(result)))),
        Err(err) => {
            let detail = err.problem_detail();
            Ok(json!(McpToolResult::err(format!(
                "gateway denied: {} — {}",
                detail["title"], detail["detail"]
            ))))
        }
    }
}
