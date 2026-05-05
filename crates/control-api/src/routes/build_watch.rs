use axum::{extract::State, Json};
use authority_domain::{WatchEventType, WatchSeverity};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct RecordEventRequest {
    pub scope: String,
    pub event_type: String,
    pub severity: String,
    pub detail: String,
    pub evidence_ref: Option<String>,
}

/// POST /v1/watch/events — record a watch event
pub async fn record_event(
    State(ctx): State<ServiceContext>,
    Json(body): Json<RecordEventRequest>,
) -> Json<Value> {
    let event_type = match parse_event_type(&body.event_type) {
        Some(t) => t,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("invalid event_type: {}", body.event_type)
            }));
        }
    };
    let severity = match parse_severity(&body.severity) {
        Some(s) => s,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("invalid severity: {}", body.severity)
            }));
        }
    };

    match ctx
        .build_watch
        .record_event(body.scope, event_type, severity, body.detail, body.evidence_ref)
        .await
    {
        Ok(event) => Json(serde_json::json!({
            "success": true,
            "data": event
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

#[derive(Deserialize)]
pub struct QueryEventsQuery {
    pub scope: Option<String>,
    pub event_type: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /v1/watch/events — query watch events
pub async fn query_events(
    State(ctx): State<ServiceContext>,
    axum::extract::Query(query): axum::extract::Query<QueryEventsQuery>,
) -> Json<Value> {
    match ctx
        .build_watch
        .query_watch_events(
            query.scope,
            query.event_type,
            query.severity,
            query.limit.unwrap_or(20),
            query.offset.unwrap_or(0),
        )
        .await
    {
        Ok(events) => Json(serde_json::json!({
            "success": true,
            "data": events
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// GET /v1/watch/dashboard — get watch dashboard
pub async fn get_dashboard(
    State(ctx): State<ServiceContext>,
) -> Json<Value> {
    match ctx.build_watch.get_watch_dashboard().await {
        Ok(dashboard) => Json(serde_json::json!({
            "success": true,
            "data": dashboard
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// GET /v1/watch/cost-summary — get cost summary
pub async fn get_cost_summary(
    State(ctx): State<ServiceContext>,
) -> Json<Value> {
    match ctx.build_watch.aggregate_cost_summary().await {
        Ok(summary) => Json(serde_json::json!({
            "success": true,
            "data": summary
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

fn parse_event_type(s: &str) -> Option<WatchEventType> {
    match s {
        "file_mutation" => Some(WatchEventType::FileMutation),
        "packet_submission" => Some(WatchEventType::PacketSubmission),
        "policy_violation" => Some(WatchEventType::PolicyViolation),
        "cost_anomaly" => Some(WatchEventType::CostAnomaly),
        "build_failure" => Some(WatchEventType::BuildFailure),
        "test_failure" => Some(WatchEventType::TestFailure),
        "evidence_gap" => Some(WatchEventType::EvidenceGap),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<WatchSeverity> {
    match s {
        "info" => Some(WatchSeverity::Info),
        "warning" => Some(WatchSeverity::Warning),
        "violation" => Some(WatchSeverity::Violation),
        "critical" => Some(WatchSeverity::Critical),
        _ => None,
    }
}
