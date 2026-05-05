use control_service::ServiceContext;
use serde_json::Value;

use crate::{
    error::McpError,
    types::{ApplyCommandArgs, AuthorizeCommandActionArgs, ProposeCommandArgs},
};

/// core_propose_command — Propose a new bounded command.
pub async fn core_propose_command(
    args: ProposeCommandArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let command = ctx
        .commands
        .propose_command(
            &args.scope,
            &args.action,
            &args.target_type,
            &args.target_id,
            args.payload,
        )
        .await?;
    Ok(serde_json::to_value(command)?)
}

/// core_authorize_command_action — Authorize a proposed command against policy.
pub async fn core_authorize_command_action(
    args: AuthorizeCommandActionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let command = ctx
        .commands
        .authorize_command(&args.command_id, &args.scope)
        .await?;
    Ok(serde_json::to_value(command)?)
}

/// core_apply_command — Apply an authorized command.
pub async fn core_apply_command(
    args: ApplyCommandArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let command = ctx
        .commands
        .apply_command(&args.command_id, &args.scope)
        .await?;
    Ok(serde_json::to_value(command)?)
}
