use control_service::ServiceContext;
use serde_json::Value;

use crate::{
    error::McpError,
    types::{ExecuteRollbackArgs, PreviewRollbackArgs, VerifyRollbackArgs},
};

/// core_preview_rollback — Preview a rollback between two snapshots.
pub async fn core_preview_rollback(
    args: PreviewRollbackArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let preview = ctx
        .rollback
        .preview_rollback(&args.from_snapshot_id, &args.to_snapshot_id)
        .await?;
    Ok(serde_json::to_value(preview)?)
}

/// core_execute_rollback — Execute a rollback between two snapshots.
pub async fn core_execute_rollback(
    args: ExecuteRollbackArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let result = ctx
        .rollback
        .execute_rollback(
            &args.from_snapshot_id,
            &args.to_snapshot_id,
            &args.tenant_id,
            &args.project_id,
            &args.actor_id,
        )
        .await?;
    Ok(serde_json::to_value(result)?)
}

/// core_verify_rollback — Verify a rollback snapshot's consistency.
pub async fn core_verify_rollback(
    args: VerifyRollbackArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let verification = ctx.rollback.verify_rollback(&args.snapshot_id).await?;
    Ok(serde_json::to_value(verification)?)
}
