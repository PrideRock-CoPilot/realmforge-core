use clap::Subcommand;
use control_service::ServiceContext;
use serde_json::Value;
use std::collections::HashMap;

use crate::util::CliResult;

/// Subcommands for runtime instance management.
#[derive(Subcommand)]
pub enum RuntimeCommand {
    /// Deploy a bundle as a runtime instance
    Deploy {
        runtime_id: String,
        bundle_id: String,
        #[arg(long, default_value = "0")]
        active_sessions: i64,
        #[arg(long)]
        metadata: Option<String>,
    },
    /// Get runtime instance status
    Status {
        runtime_id: String,
    },
    /// List all runtime instances
    List {
        #[arg(long, default_value = "20")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Stop a runtime instance
    Stop {
        runtime_id: String,
    },
}

pub async fn handle_runtime(cmd: RuntimeCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        RuntimeCommand::Deploy {
            runtime_id,
            bundle_id,
            active_sessions,
            metadata,
        } => {
            let parsed_metadata: HashMap<String, Value> = match metadata {
                Some(ref m) => serde_json::from_str(m)?,
                None => HashMap::new(),
            };
            let instance = authority_domain::RuntimeInstance {
                runtime_id: authority_domain::RuntimeId::new(runtime_id)?,
                bundle_id: authority_domain::BundleId::new(bundle_id)?,
                status: "running".to_string(),
                started_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
                active_sessions,
                action_count: 0,
                error_count: 0,
                metadata: serde_json::to_value(parsed_metadata)?,
            };
            ctx.runtimes.deploy_bundle(&instance).await?;
            println!("{}", serde_json::to_string_pretty(&instance)?);
        }
        RuntimeCommand::Status { runtime_id } => {
            let instance = ctx
                .runtimes
                .get_runtime_status(&authority_domain::RuntimeId::new(runtime_id)?)
                .await?;
            println!("{}", serde_json::to_string_pretty(&instance)?);
        }
        RuntimeCommand::List { limit, offset } => {
            let instances = ctx.runtimes.list_runtimes(limit, offset).await?;
            println!("{}", serde_json::to_string_pretty(&instances)?);
        }
        RuntimeCommand::Stop { runtime_id } => {
            ctx.runtimes
                .stop_runtime(&authority_domain::RuntimeId::new(runtime_id.clone())?)
                .await?;
            let result: Value = serde_json::json!({
                "runtime_id": runtime_id,
                "status": "stopped",
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
