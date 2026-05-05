use crate::util::{output_json, CliResult};
use authority_domain::{ProjectId, SnapshotId, TenantId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

/// Snapshot management commands.
#[derive(Subcommand)]
pub enum SnapshotCommand {
    Create(CreateArgs),
    Validate(ValidateArgs),
    List(ListArgs),
    Compare(CompareArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    #[arg(long)]
    pub reason: String,
    #[arg(long)]
    pub parent: Option<SnapshotId>,
}

#[derive(Args)]
pub struct ValidateArgs {
    pub snapshot_id: SnapshotId,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, default_value = "50")]
    pub limit: u64,
    #[arg(long, default_value = "0")]
    pub offset: u64,
}

#[derive(Args)]
pub struct CompareArgs {
    pub from_id: SnapshotId,
    pub to_id: SnapshotId,
}

pub async fn handle_snapshot(cmd: SnapshotCommand, ctx: &ServiceContext) -> CliResult {
    let project_id = ProjectId::new("default").map_err(|e| format!("Invalid project id: {}", e))?;

    match cmd {
        SnapshotCommand::Create(args) => {
            let manifest = ctx
                .snapshots
                .create_snapshot(
                    &args.tenant_id,
                    &args.project_id,
                    &args.reason,
                    args.parent,
                    vec![],
                    vec![],
                    None,
                )
                .await?;
            output_json(&json!({
                "snapshot_id": manifest.id.as_str(),
                "status": format!("{:?}", manifest.status),
                "reason": manifest.reason,
                "object_count": manifest.object_refs.len(),
            }));
        }
        SnapshotCommand::Validate(args) => {
            let report = ctx.snapshots.validate_snapshot(&args.snapshot_id).await?;
            output_json(&report);
        }
        SnapshotCommand::List(args) => {
            let manifests = ctx
                .snapshots
                .list_snapshots(&project_id, args.limit, args.offset)
                .await?;
            output_json(&manifests);
        }
        SnapshotCommand::Compare(args) => {
            let delta = ctx
                .snapshots
                .compare_snapshots(&args.from_id, &args.to_id)
                .await?;
            output_json(&delta);
        }
    }
    Ok(())
}
