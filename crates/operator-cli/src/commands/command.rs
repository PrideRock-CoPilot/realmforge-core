use authority_domain::{ActorScope, CommandId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::util::CliResult;

/// Command lifecycle management.
#[derive(Subcommand)]
pub enum CommandCommand {
    Propose(ProposeArgs),
    Authorize(AuthorizeArgs),
    Apply(ApplyArgs),
    Deny(DenyArgs),
}

#[derive(Args)]
pub struct ProposeArgs {
    #[arg(long)]
    pub action: String,
    #[arg(long)]
    pub target_type: String,
    #[arg(long)]
    pub target_id: String,
    #[arg(long)]
    pub payload: Option<String>,
}

#[derive(Args)]
pub struct AuthorizeArgs {
    pub command_id: CommandId,
}

#[derive(Args)]
pub struct ApplyArgs {
    pub command_id: CommandId,
}

#[derive(Args)]
pub struct DenyArgs {
    pub command_id: CommandId,
}

pub async fn handle_command(cmd: CommandCommand, ctx: &ServiceContext) -> CliResult {
    let scope = ActorScope::default();

    match cmd {
        CommandCommand::Propose(args) => {
            let payload: Value = args
                .payload
                .map(|p| serde_json::from_str(&p).unwrap_or(json!({"raw": p})))
                .unwrap_or(json!({}));
            let command = ctx
                .commands
                .propose_command(
                    &scope,
                    &args.action,
                    &args.target_type,
                    &args.target_id,
                    payload,
                )
                .await?;
            output_json(&command);
        }
        CommandCommand::Authorize(args) => {
            let command = ctx
                .commands
                .authorize_command(&args.command_id, &scope)
                .await?;
            output_json(&command);
        }
        CommandCommand::Apply(args) => {
            let command = ctx.commands.apply_command(&args.command_id, &scope).await?;
            output_json(&command);
        }
        CommandCommand::Deny(args) => {
            let denial = control_service::PolicyDecision {
                allowed: false,
                denial: None,
            };
            let command = ctx
                .commands
                .deny_command(&args.command_id, &scope, &denial)
                .await?;
            output_json(&command);
        }
    }
    Ok(())
}

fn output_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}
