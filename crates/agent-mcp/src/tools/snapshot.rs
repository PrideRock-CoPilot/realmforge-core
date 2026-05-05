use control_service::ServiceContext;
use serde_json::Value;

use crate::{
    error::McpError,
    types::{CompareSnapshotsArgs, CreateSnapshotArgs, ValidateSnapshotArgs},
};

/// core_create_snapshot — Create a new snapshot.
pub async fn core_create_snapshot(
    args: CreateSnapshotArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let manifest = ctx
        .snapshots
        .create_snapshot(
            &args.tenant_id,
            &args.project_id,
            &args.reason,
            args.parent_snapshot_id,
            vec![],
            vec![],
            None,
        )
        .await?;
    Ok(serde_json::to_value(manifest)?)
}

/// core_validate_snapshot — Validate a snapshot's hash and references.
pub async fn core_validate_snapshot(
    args: ValidateSnapshotArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let report = ctx.snapshots.validate_snapshot(&args.snapshot_id).await?;
    Ok(serde_json::to_value(report)?)
}

/// core_compare_snapshots — Compare two snapshots and return the delta.
pub async fn core_compare_snapshots(
    args: CompareSnapshotsArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let delta = ctx
        .snapshots
        .compare_snapshots(&args.from_id, &args.to_id)
        .await?;
    Ok(serde_json::to_value(delta)?)
}
