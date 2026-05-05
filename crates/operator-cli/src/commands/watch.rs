use clap::Subcommand;
use control_service::ServiceContext;

use crate::util::CliResult;

/// Subcommands for build watch (events, violations, cost).
#[derive(Subcommand)]
pub enum WatchCommand {
    /// Record a build watch event
    RecordEvent {
        scope: String,
        event_type: String,
        severity: String,
        detail: String,
        #[arg(long)]
        evidence_ref: Option<String>,
    },
    /// Query watch events
    QueryEvents {
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        event_type: Option<String>,
        #[arg(long)]
        severity: Option<String>,
        #[arg(long, default_value = "20")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Get the watch dashboard summary
    Dashboard,
    /// Get cost summary by scope
    CostSummary,
    /// Record a cost entry
    RecordCost {
        scope: String,
        #[arg(default_value = "0")]
        token_cost: u64,
        #[arg(default_value = "0")]
        build_time_ms: u64,
        #[arg(default_value = "0")]
        storage_bytes: u64,
        #[arg(default_value = "0")]
        rework_count: u32,
    },
    /// Record a violation
    RecordViolation {
        rule: String,
        severity: String,
        detail: String,
        #[arg(long)]
        evidence_ref: Option<String>,
    },
}

pub async fn handle_watch(cmd: WatchCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        WatchCommand::RecordEvent {
            scope,
            event_type,
            severity,
            detail,
            evidence_ref,
        } => {
            let et = parse_cli_event_type(&event_type)?;
            let sev = parse_cli_severity(&severity)?;
            let event = ctx
                .build_watch
                .record_event(scope, et, sev, detail, evidence_ref)
                .await?;
            println!("{}", serde_json::to_string_pretty(&event)?);
        }
        WatchCommand::QueryEvents {
            scope,
            event_type,
            severity,
            limit,
            offset,
        } => {
            let events = ctx
                .build_watch
                .query_watch_events(scope, event_type, severity, limit, offset)
                .await?;
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        WatchCommand::Dashboard => {
            let dashboard = ctx.build_watch.get_watch_dashboard().await?;
            println!("{}", serde_json::to_string_pretty(&dashboard)?);
        }
        WatchCommand::CostSummary => {
            let summary = ctx.build_watch.aggregate_cost_summary().await?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        WatchCommand::RecordCost {
            scope,
            token_cost,
            build_time_ms,
            storage_bytes,
            rework_count,
        } => {
            let record = ctx
                .build_watch
                .get_cost_summary(
                    scope,
                    token_cost,
                    build_time_ms,
                    storage_bytes,
                    rework_count,
                )
                .await?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }
        WatchCommand::RecordViolation {
            rule,
            severity,
            detail,
            evidence_ref,
        } => {
            let sev = parse_cli_severity(&severity)?;
            let violation = ctx
                .build_watch
                .detect_violation(rule, sev, detail, evidence_ref)
                .await?;
            println!("{}", serde_json::to_string_pretty(&violation)?);
        }
    }
    Ok(())
}

fn parse_cli_event_type(s: &str) -> Result<authority_domain::WatchEventType, String> {
    match s {
        "file_mutation" => Ok(authority_domain::WatchEventType::FileMutation),
        "packet_submission" => Ok(authority_domain::WatchEventType::PacketSubmission),
        "policy_violation" => Ok(authority_domain::WatchEventType::PolicyViolation),
        "cost_anomaly" => Ok(authority_domain::WatchEventType::CostAnomaly),
        "build_failure" => Ok(authority_domain::WatchEventType::BuildFailure),
        "test_failure" => Ok(authority_domain::WatchEventType::TestFailure),
        "evidence_gap" => Ok(authority_domain::WatchEventType::EvidenceGap),
        _ => Err(format!("invalid event_type: {s}")),
    }
}

fn parse_cli_severity(s: &str) -> Result<authority_domain::WatchSeverity, String> {
    match s {
        "info" => Ok(authority_domain::WatchSeverity::Info),
        "warning" => Ok(authority_domain::WatchSeverity::Warning),
        "violation" => Ok(authority_domain::WatchSeverity::Violation),
        "critical" => Ok(authority_domain::WatchSeverity::Critical),
        _ => Err(format!("invalid severity: {s}")),
    }
}
