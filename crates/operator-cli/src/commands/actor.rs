use authority_domain::{ActorId, SessionId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;

use crate::util::{output_json, CliResult};

/// Actor scope commands.
#[derive(Subcommand)]
pub enum ActorCommand {
    Scope(ScopeArgs),
}

#[derive(Args)]
pub struct ScopeArgs {
    pub actor_id: ActorId,
    pub session_id: SessionId,
}

pub async fn handle_actor(cmd: ActorCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        ActorCommand::Scope(args) => {
            let scope = ctx
                .actors
                .get_scope(&args.actor_id, &args.session_id)
                .await?;
            output_json(&scope);
        }
    }
    Ok(())
}
