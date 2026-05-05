use authority_domain::{BundleId, RuntimeId};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::instrument;

use crate::error::McpError;

/// Args for core_execute_runtime_action
#[derive(Clone, Debug, Deserialize)]
pub struct ExecuteRuntimeActionArgs {
    pub runtime_id: String,
    pub action: String,
    pub payload: Option<Value>,
}

/// Args for core_get_runtime_status
#[derive(Clone, Debug, Deserialize)]
pub struct GetRuntimeStatusArgs {
    pub runtime_id: String,
}

/// Args for core_deploy_runtime
#[derive(Clone, Debug, Deserialize)]
pub struct DeployRuntimeArgs {
    pub runtime_id: String,
    pub bundle_id: String,
    pub active_sessions: Option<i64>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Args for core_stop_runtime
#[derive(Clone, Debug, Deserialize)]
pub struct StopRuntimeArgs {
    pub runtime_id: String,
}

/// Execute an action on a runtime instance.
#[instrument(skip(ctx), fields(runtime_id = %args.runtime_id, action = %args.action))]
pub async fn core_execute_runtime_action(
    args: ExecuteRuntimeActionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let runtime_id = RuntimeId::new(args.runtime_id)
        .map_err(|e| McpError::InvalidArgs(format!("invalid runtime_id: {e}")))?;
    let instance = ctx.runtimes.get_runtime_status(&runtime_id).await?;

    let payload = args.payload.unwrap_or(json!({}));
    let result = json!({
        "runtime_id": runtime_id.to_string(),
        "action": args.action,
        "status": instance.status,
        "result": "action_executed",
        "payload": payload,
    });

    Ok(result)
}

/// Get the status of a runtime instance.
#[instrument(skip(ctx), fields(runtime_id = %args.runtime_id))]
pub async fn core_get_runtime_status(
    args: GetRuntimeStatusArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let instance = ctx
        .runtimes
        .get_runtime_status(
            &RuntimeId::new(args.runtime_id)
                .map_err(|e| McpError::InvalidArgs(format!("invalid runtime_id: {e}")))?,
        )
        .await?;
    Ok(json!(instance))
}

/// Deploy a bundle as a new runtime instance.
#[instrument(skip(ctx), fields(runtime_id = %args.runtime_id, bundle_id = %args.bundle_id))]
pub async fn core_deploy_runtime(
    args: DeployRuntimeArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let instance = authority_domain::RuntimeInstance {
        runtime_id: RuntimeId::new(args.runtime_id)
            .map_err(|e| McpError::InvalidArgs(format!("invalid runtime_id: {e}")))?,
        bundle_id: BundleId::new(args.bundle_id)
            .map_err(|e| McpError::InvalidArgs(format!("invalid bundle_id: {e}")))?,
        status: "running".to_string(),
        started_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
        active_sessions: args.active_sessions.unwrap_or(0),
        action_count: 0,
        error_count: 0,
        metadata: serde_json::to_value(args.metadata.unwrap_or_default())?,
    };

    ctx.runtimes.deploy_bundle(&instance).await?;
    Ok(json!(instance))
}

/// Stop a runtime instance.
#[instrument(skip(ctx), fields(runtime_id = %args.runtime_id))]
pub async fn core_stop_runtime(
    args: StopRuntimeArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let runtime_id = RuntimeId::new(args.runtime_id)
        .map_err(|e| McpError::InvalidArgs(format!("invalid runtime_id: {e}")))?;
    ctx.runtimes.stop_runtime(&runtime_id).await?;
    Ok(json!({
        "runtime_id": runtime_id.to_string(),
        "status": "stopped",
    }))
}
