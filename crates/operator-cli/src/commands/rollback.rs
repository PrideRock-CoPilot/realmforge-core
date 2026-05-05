use crate::util::{output_json, CliResult};
use authority_domain::{ActorId, ProjectId, SnapshotId, TenantId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;

/// Rollback management commands.
#[derive(Subcommand)]
pub enum RollbackCommand {
    Preview(PreviewArgs),
    Execute(ExecuteArgs),
    Verify(VerifyArgs),
}

#[derive(Args)]
pub struct PreviewArgs {
    #[arg(long)]
    pub from: SnapshotId,
    #[arg(long)]
    pub to: SnapshotId,
}

#[derive(Args)]
pub struct ExecuteArgs {
    #[arg(long)]
    pub from: SnapshotId,
    #[arg(long)]
    pub to: SnapshotId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
}

#[derive(Args)]
pub struct VerifyArgs {
    pub snapshot_id: SnapshotId,
}

pub async fn handle_rollback(cmd: RollbackCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        RollbackCommand::Preview(args) => {
            let preview = ctx.rollback.preview_rollback(&args.from, &args.to).await?;
            output_json(&preview);
        }
        RollbackCommand::Execute(args) => {
            let result = ctx
                .rollback
                .execute_rollback(
                    &args.from,
                    &args.to,
                    &args.tenant_id,
                    &args.project_id,
                    &args.actor_id,
                )
                .await?;
            output_json(&result);
        }
        RollbackCommand::Verify(args) => {
            let verification = ctx.rollback.verify_rollback(&args.snapshot_id).await?;
            output_json(&verification);
        }
    }
    Ok(())
}
