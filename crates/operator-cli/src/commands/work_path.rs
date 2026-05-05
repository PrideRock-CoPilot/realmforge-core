use authority_domain::{WorkPathId, WorkPathNodeId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Work path graph management commands.
#[derive(Subcommand)]
pub enum WorkPathCommand {
    Create(CreateArgs),
    Get(GetArgs),
    Traverse(TraverseArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    pub id: WorkPathId,
    pub name: String,
    #[arg(default_value = "")]
    pub description: String,
}

#[derive(Args)]
pub struct GetArgs {
    pub id: WorkPathId,
}

#[derive(Args)]
pub struct TraverseArgs {
    pub work_path_id: WorkPathId,
    pub node_id: WorkPathNodeId,
    pub agent_id: authority_domain::ActorId,
    pub tenant_id: authority_domain::TenantId,
    pub project_id: authority_domain::ProjectId,
    #[arg(long)]
    pub objective: String,
}

pub async fn handle_work_path(cmd: WorkPathCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        WorkPathCommand::Create(args) => {
            let graph = ctx
                .work_paths
                .create_work_path(&args.id, &args.name, &args.description)
                .await?;
            output_json(&json!({
                "id": graph.id.as_str(),
                "name": graph.name,
                "description": graph.description,
                "node_count": graph.nodes.len(),
            }));
        }
        WorkPathCommand::Get(args) => {
            let graph = ctx.work_paths.get_work_path_graph(&args.id).await?;
            output_json(&json!({
                "id": graph.id.as_str(),
                "name": graph.name,
                "node_count": graph.nodes.len(),
            }));
        }
        WorkPathCommand::Traverse(args) => {
            let packet = ctx
                .work_paths
                .traverse_to_packet(
                    &args.work_path_id,
                    &args.node_id,
                    &args.agent_id,
                    &args.tenant_id,
                    &args.project_id,
                    &args.objective,
                    None,
                )
                .await?;
            output_json(&json!({
                "packet_id": packet.id.as_str(),
                "work_path_node_id": packet.work_path_node_id,
                "allowed_files": packet.allowed_file_paths,
                "required_contracts": packet.required_contracts,
                "required_tests": packet.required_tests,
                "status": format!("{:?}", packet.status),
            }));
        }
    }
    Ok(())
}
