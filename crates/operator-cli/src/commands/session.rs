use authority_domain::{ActorId, ProjectId, SessionId, TenantId};
use clap::{Args, Subcommand};
use control_service::{session_service::SessionData, ServiceContext};
use serde_json::json;

use crate::util::{output_json, output_pretty, CliResult};

/// Session management commands.
#[derive(Subcommand)]
pub enum SessionCommand {
    Issue(IssueArgs),
    Renew(RenewArgs),
    Revoke(RevokeArgs),
    List,
    Get(GetArgs),
}

#[derive(Args)]
pub struct IssueArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    #[arg(long, default_value = "3600")]
    pub ttl: i64,
}

#[derive(Args)]
pub struct RenewArgs {
    pub session_id: SessionId,
    #[arg(long, default_value = "3600")]
    pub ttl: i64,
}

#[derive(Args)]
pub struct RevokeArgs {
    pub session_id: SessionId,
    #[arg(long, default_value = "cli-revoked")]
    pub reason: String,
}

#[derive(Args)]
pub struct GetArgs {
    pub session_id: SessionId,
}

pub async fn handle_session(cmd: SessionCommand, ctx: &ServiceContext, pretty: bool) -> CliResult {
    match cmd {
        SessionCommand::Issue(args) => {
            let session = ctx
                .sessions
                .issue_session(&args.actor_id, &args.tenant_id, &args.project_id, args.ttl)
                .await?;
            output(&session, pretty)
        }
        SessionCommand::Renew(args) => {
            let session = ctx
                .sessions
                .renew_session(&args.session_id, args.ttl)
                .await?;
            output(&session, pretty)
        }
        SessionCommand::Revoke(args) => {
            ctx.sessions
                .revoke_session(&args.session_id, &args.reason)
                .await?;
            let result = json!({"status": "revoked", "session_id": args.session_id.as_str()});
            output_json(&result);
            Ok(())
        }
        SessionCommand::List => {
            output_json(&json!({"sessions": []}));
            Ok(())
        }
        SessionCommand::Get(args) => {
            let session = ctx.sessions.validate_session(&args.session_id).await?;
            output(&session, pretty)
        }
    }
}

fn output(session: &SessionData, pretty: bool) -> CliResult {
    if pretty {
        output_pretty(session);
    } else {
        output_json(session);
    }
    Ok(())
}
