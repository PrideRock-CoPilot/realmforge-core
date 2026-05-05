use authority_domain::ProjectId;
use clap::Subcommand;
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Audit trail query and verification commands.
#[derive(Subcommand)]
pub enum AuditCommand {
    Events,
    VerifyChain,
    Export { format: Option<String> },
}

pub async fn handle_audit(cmd: AuditCommand, ctx: &ServiceContext) -> CliResult {
    let project_id = ProjectId::new("default").map_err(|e| format!("Invalid project id: {}", e))?;

    match cmd {
        AuditCommand::Events => {
            let (events, total) = ctx
                .audit
                .query_events(&project_id, None, None, None, None, None, 50, 0)
                .await?;
            output_json(&json!({"events": events, "total": total}));
        }
        AuditCommand::VerifyChain => {
            let anchor = ctx.audit.verify_chain(&project_id).await?;
            output_json(&json!({
                "chain_integrity": anchor.chain_integrity,
                "event_count": anchor.event_count,
                "first_event_id": anchor.first_event_id.as_str(),
                "last_event_id": anchor.last_event_id.as_str(),
            }));
        }
        AuditCommand::Export { format: _ } => {
            output_json(&json!({"status": "export not yet implemented"}));
        }
    }
    Ok(())
}
