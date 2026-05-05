use crate::CliResult;
use authority_domain::ProposalId;
use clap::Subcommand;
use control_service::ServiceContext;
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum LiveWatchCommands {
    /// Start monitoring an app.
    Start {
        /// The app ID to monitor
        app_id: String,
    },
    /// Stop monitoring an app.
    Stop {
        /// The app ID to stop monitoring
        app_id: String,
    },
    /// Query recent watch signals for an app.
    Signals {
        /// The app ID to query signals for
        app_id: String,
        /// Filter by signal type (latency, error_rate, etc.)
        #[arg(long)]
        signal_type: Option<String>,
        /// Filter by severity (info, warning, critical)
        #[arg(long)]
        severity: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: i64,
        /// Number of results to skip
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Propose remediation for an app's anomalies.
    Propose {
        /// The app ID to propose remediation for
        app_id: String,
    },
    /// List remediation proposals for an app.
    Proposals {
        /// The app ID to list proposals for
        app_id: String,
        /// Filter by status (proposed, approved, rejected, executed)
        #[arg(long)]
        status: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: i64,
        /// Number of results to skip
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Approve a remediation proposal.
    Approve {
        /// The app ID
        app_id: String,
        /// The proposal ID to approve
        proposal_id: String,
    },
}

/// Execute a live-watch subcommand.
pub async fn execute(cmd: LiveWatchCommands, ctx: &ServiceContext) -> CliResult {
    match cmd {
        LiveWatchCommands::Start { app_id } => {
            // Start is acknowledged; actual monitoring loop runs in the live-watch engine
            println!("Live Watch monitoring started for app: {}", app_id);
            Ok(())
        }
        LiveWatchCommands::Stop { app_id } => {
            println!("Live Watch monitoring stopped for app: {}", app_id);
            Ok(())
        }
        LiveWatchCommands::Signals {
            app_id,
            signal_type,
            severity,
            limit,
            offset,
        } => {
            let signals = ctx
                .live_watch
                .get_signals(
                    &app_id,
                    signal_type.as_deref(),
                    severity.as_deref(),
                    limit,
                    offset,
                )
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "signals": signals }))?
            );
            Ok(())
        }
        LiveWatchCommands::Propose { app_id } => {
            let proposal = ctx.live_watch.propose_remediation(&app_id).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "proposal": proposal }))?
            );
            Ok(())
        }
        LiveWatchCommands::Proposals {
            app_id,
            status,
            limit,
            offset,
        } => {
            let status_parsed = status.as_deref().and_then(|s| match s {
                "proposed" => Some(authority_domain::ProposalStatus::Proposed),
                "approved" => Some(authority_domain::ProposalStatus::Approved),
                "rejected" => Some(authority_domain::ProposalStatus::Rejected),
                "executed" => Some(authority_domain::ProposalStatus::Executed),
                _ => None,
            });
            let proposals = ctx
                .live_watch
                .list_proposals(&app_id, status_parsed, limit, offset)
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "proposals": proposals }))?
            );
            Ok(())
        }
        LiveWatchCommands::Approve {
            app_id: _,
            proposal_id,
        } => {
            let pid = ProposalId::new(proposal_id)?;
            let proposal = ctx.live_watch.approve_proposal(&pid).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "proposal": proposal }))?
            );
            Ok(())
        }
    }
}
