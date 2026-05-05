use authority_domain::{WatchEventType, WatchSeverity};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::instrument;

use crate::error::McpError;

/// Args for core_record_watch_event
#[derive(Clone, Debug, Deserialize)]
pub struct RecordWatchEventArgs {
    pub scope: String,
    pub event_type: String,
    pub severity: String,
    pub detail: String,
    pub evidence_ref: Option<String>,
}

/// Args for core_get_watch_dashboard
#[derive(Clone, Debug, Deserialize)]
pub struct GetWatchDashboardArgs {}

/// Args for core_get_cost_summary
#[derive(Clone, Debug, Deserialize)]
pub struct GetCostSummaryArgs {}

/// Record a build watch event.
#[instrument(skip(ctx), fields(scope = %args.scope))]
pub async fn core_record_watch_event(
    args: RecordWatchEventArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let event_type = match args.event_type.as_str() {
        "file_mutation" => WatchEventType::FileMutation,
        "packet_submission" => WatchEventType::PacketSubmission,
        "policy_violation" => WatchEventType::PolicyViolation,
        "cost_anomaly" => WatchEventType::CostAnomaly,
        "build_failure" => WatchEventType::BuildFailure,
        "test_failure" => WatchEventType::TestFailure,
        "evidence_gap" => WatchEventType::EvidenceGap,
        other => {
            return Err(McpError::InvalidArgs(format!(
                "invalid event_type: {other} — use: file_mutation, packet_submission, policy_violation, cost_anomaly, build_failure, test_failure, evidence_gap"
            )));
        }
    };
    let severity = match args.severity.as_str() {
        "info" => WatchSeverity::Info,
        "warning" => WatchSeverity::Warning,
        "violation" => WatchSeverity::Violation,
        "critical" => WatchSeverity::Critical,
        other => {
            return Err(McpError::InvalidArgs(format!(
                "invalid severity: {other} — use: info, warning, violation, critical"
            )));
        }
    };

    let event = ctx
        .build_watch
        .record_event(args.scope, event_type, severity, args.detail, args.evidence_ref)
        .await?;
    Ok(json!({
        "id": event.id,
        "scope": event.scope,
        "event_type": event.event_type,
        "severity": event.severity,
        "detail": event.detail,
        "evidence_ref": event.evidence_ref,
        "timestamp": event.timestamp
    }))
}

/// Get the watch dashboard (event count + cost summary).
#[instrument(skip(ctx))]
pub async fn core_get_watch_dashboard(
    _args: GetWatchDashboardArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let dashboard = ctx.build_watch.get_watch_dashboard().await?;
    Ok(serde_json::to_value(dashboard)?)
}

/// Get aggregated cost summary.
#[instrument(skip(ctx))]
pub async fn core_get_cost_summary(
    _args: GetCostSummaryArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let summary = ctx.build_watch.aggregate_cost_summary().await?;
    Ok(serde_json::to_value(summary)?)
}
