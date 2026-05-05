use authority_domain::{ActorId, PacketId, ProjectId, TenantId, WorkPathId, WorkPathNodeId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Work packet generation and validation commands.
#[derive(Subcommand)]
pub enum WorkPacketCommand {
    Generate(GenerateArgs),
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct GenerateArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    /// Work path graph ID containing the target node.
    #[arg(long)]
    pub work_path_id: WorkPathId,
    /// Target work path node ID to scope the packet to.
    #[arg(long)]
    pub node_id: WorkPathNodeId,
    #[arg(long)]
    pub objective: String,
}

#[derive(Args)]
pub struct ValidateArgs {
    pub packet_id: PacketId,
}

pub async fn handle_work_packet(cmd: WorkPacketCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        WorkPacketCommand::Generate(args) => {
            let packet = ctx
                .work_packets
                .generate_work_packet(
                    &args.tenant_id,
                    &args.project_id,
                    &args.actor_id,
                    &args.work_path_id,
                    &args.node_id,
                    &args.objective,
                    None,
                )
                .await?;
            output_json(&json!({
                "packet_id": packet.id.as_str(),
                "status": format!("{:?}", packet.status),
                "objective": packet.objective,
                "allowed_files": packet.allowed_file_paths.len(),
                "denied_files": packet.denied_file_paths.len(),
                "has_rollback_anchor": packet.rollback_anchor.is_some(),
            }));
        }
        WorkPacketCommand::Validate(args) => {
            let result = ctx
                .work_packets
                .validate_packet_boundaries(&args.packet_id)
                .await?;
            output_json(&result);
        }
    }
    Ok(())
}
