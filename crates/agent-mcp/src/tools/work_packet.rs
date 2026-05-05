use control_service::ServiceContext;
use serde_json::Value;

use crate::{
    error::McpError,
    types::{GenerateWorkPacketArgs, ValidateWorkPacketArgs},
};

/// core_generate_work_packet — Generate a scoped work packet for an agent.
pub async fn core_generate_work_packet(
    args: GenerateWorkPacketArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
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
    Ok(serde_json::to_value(packet)?)
}

/// core_validate_work_packet — Validate a work packet's scope boundaries.
pub async fn core_validate_work_packet(
    args: ValidateWorkPacketArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let result = ctx
        .work_packets
        .validate_packet_boundaries(&args.packet_id)
        .await?;
    Ok(serde_json::to_value(result)?)
}
