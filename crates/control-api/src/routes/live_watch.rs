use axum::{
    extract::{Path, Query, State},
    Json,
};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::instrument;
use utoipa::IntoParams;

use crate::error::ApiError;

/// Query parameters for listing signals.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct SignalQuery {
    pub signal_type: Option<String>,
    pub severity: Option<String>,
    #[param(example = 50)]
    pub limit: Option<i64>,
    #[param(example = 0)]
    pub offset: Option<i64>,
}

/// Query parameters for listing proposals.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ProposalQuery {
    pub status: Option<String>,
    #[param(example = 50)]
    pub limit: Option<i64>,
    #[param(example = 0)]
    pub offset: Option<i64>,
}

/// Body for recording a signal.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct RecordSignalBody {
    #[schema(example = "latency")]
    pub signal_type: String,
    #[schema(example = 250.0)]
    pub value: f64,
    #[schema(example = 200.0)]
    pub threshold: f64,
    #[schema(example = "warning")]
    pub severity: String,
}

/// Body for updating a watch profile.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct ProfileBody {
    pub signal_thresholds: Option<Vec<ThresholdJson>>,
    pub max_proposals_per_day: Option<u32>,
    pub poll_interval_secs: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct ThresholdJson {
    pub signal_type: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
}

/// POST /v1/live-watch/:app_id/start
#[utoipa::path(
    post,
    path = "/v1/live-watch/{app_id}/start",
    operation_id = "start_monitoring",
    summary = "Enable Live Watch monitoring for an app.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    responses(
        (status = 200, description = "Monitoring started"),
        (status = 500, description = "Internal error"),
    )
)]
#[instrument(skip(_ctx))]
pub async fn start_monitoring(
    State(_ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "status": "started",
        "app_id": app_id,
        "message": "Live Watch monitoring enabled for app"
    })))
}

/// POST /v1/live-watch/:app_id/stop
#[utoipa::path(
    post,
    path = "/v1/live-watch/{app_id}/stop",
    operation_id = "stop_monitoring",
    summary = "Disable Live Watch monitoring for an app.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    responses(
        (status = 200, description = "Monitoring stopped"),
        (status = 500, description = "Internal error"),
    )
)]
#[instrument(skip(_ctx))]
pub async fn stop_monitoring(
    State(_ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "status": "stopped",
        "app_id": app_id,
        "message": "Live Watch monitoring disabled for app"
    })))
}

/// POST /v1/live-watch/:app_id/signals
#[utoipa::path(
    post,
    path = "/v1/live-watch/{app_id}/signals",
    operation_id = "record_signal",
    summary = "Record a runtime signal (latency, error_rate, cost_rate, etc.) for an app.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    request_body = RecordSignalBody,
    responses(
        (status = 200, description = "Signal recorded"),
        (status = 400, description = "Invalid signal type or severity"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn record_signal(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
    Json(body): Json<RecordSignalBody>,
) -> Result<Json<Value>, ApiError> {
    let signal_type = parse_signal_type(&body.signal_type).ok_or_else(|| {
        ApiError::BadRequest(format!("unknown signal_type: {}", body.signal_type))
    })?;
    let severity = parse_severity(&body.severity)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown severity: {}", body.severity)))?;

    let signal = ctx
        .live_watch
        .record_signal(app_id, signal_type, body.value, body.threshold, severity)
        .await?;
    Ok(Json(json!(signal)))
}

/// GET /v1/live-watch/:app_id/signals
#[utoipa::path(
    get,
    path = "/v1/live-watch/{app_id}/signals",
    operation_id = "get_signals",
    summary = "List recorded signals for an app, optionally filtered by type and severity.",
    tag = "live-watch",
    params(
        ("app_id" = String, Path, description = "Application ID"),
        SignalQuery,
    ),
    responses(
        (status = 200, description = "Signal list"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_signals(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
    Query(query): Query<SignalQuery>,
) -> Result<Json<Value>, ApiError> {
    let signals = ctx
        .live_watch
        .get_signals(
            &app_id,
            query.signal_type.as_deref(),
            query.severity.as_deref(),
            query.limit.unwrap_or(50),
            query.offset.unwrap_or(0),
        )
        .await?;
    Ok(Json(json!({"signals": signals, "count": signals.len()})))
}

/// POST /v1/live-watch/:app_id/propose
#[utoipa::path(
    post,
    path = "/v1/live-watch/{app_id}/propose",
    operation_id = "propose_remediation",
    summary = "Propose a remediation action based on current signal state.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    responses(
        (status = 200, description = "Proposal created"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn propose_remediation(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let proposal = ctx.live_watch.propose_remediation(&app_id).await?;
    Ok(Json(json!(proposal)))
}

/// GET /v1/live-watch/:app_id/proposals
#[utoipa::path(
    get,
    path = "/v1/live-watch/{app_id}/proposals",
    operation_id = "list_proposals",
    summary = "List remediation proposals for an app.",
    tag = "live-watch",
    params(
        ("app_id" = String, Path, description = "Application ID"),
        ProposalQuery,
    ),
    responses(
        (status = 200, description = "Proposal list"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_proposals(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
    Query(query): Query<ProposalQuery>,
) -> Result<Json<Value>, ApiError> {
    let status = query.status.as_deref().and_then(parse_proposal_status);
    let proposals = ctx
        .live_watch
        .list_proposals(
            &app_id,
            status,
            query.limit.unwrap_or(50),
            query.offset.unwrap_or(0),
        )
        .await?;
    Ok(Json(
        json!({"proposals": proposals, "count": proposals.len()}),
    ))
}

/// POST /v1/live-watch/:app_id/proposals/:id/approve
#[utoipa::path(
    post,
    path = "/v1/live-watch/{app_id}/proposals/{id}/approve",
    operation_id = "approve_proposal",
    summary = "Approve a remediation proposal, authorizing its execution.",
    tag = "live-watch",
    params(
        ("app_id" = String, Path, description = "Application ID"),
        ("id" = String, Path, description = "Proposal ID"),
    ),
    responses(
        (status = 200, description = "Proposal approved"),
        (status = 400, description = "Invalid proposal ID"),
        (status = 404, description = "Proposal not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn approve_proposal(
    State(ctx): State<ServiceContext>,
    Path((_app_id, proposal_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pid = ProposalId::new(proposal_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid proposal_id: {e}")))?;
    let proposal = ctx.live_watch.approve_proposal(&pid).await?;
    Ok(Json(json!(proposal)))
}

/// GET /v1/live-watch/profiles/:app_id
#[utoipa::path(
    get,
    path = "/v1/live-watch/profiles/{app_id}",
    operation_id = "get_profile",
    summary = "Get the Live Watch monitoring profile for an app.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    responses(
        (status = 200, description = "Watch profile"),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_profile(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let profile = ctx.live_watch.get_profile(&app_id).await?;
    Ok(Json(json!(profile)))
}

/// PUT /v1/live-watch/profiles/:app_id
#[utoipa::path(
    put,
    path = "/v1/live-watch/profiles/{app_id}",
    operation_id = "update_profile",
    summary = "Update the Live Watch monitoring profile for an app.",
    tag = "live-watch",
    params(("app_id" = String, Path, description = "Application ID")),
    request_body = ProfileBody,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn update_profile(
    State(ctx): State<ServiceContext>,
    Path(app_id): Path<String>,
    Json(body): Json<ProfileBody>,
) -> Result<Json<Value>, ApiError> {
    let mut profile = ctx.live_watch.get_profile(&app_id).await?;
    if let Some(thresholds) = body.signal_thresholds {
        profile.signal_thresholds = thresholds
            .into_iter()
            .filter_map(|t| {
                Some(authority_domain::SignalThreshold {
                    signal_type: parse_signal_type(&t.signal_type)?,
                    warning_threshold: t.warning_threshold,
                    critical_threshold: t.critical_threshold,
                })
            })
            .collect();
    }
    if let Some(max) = body.max_proposals_per_day {
        profile.max_proposals_per_day = max;
    }
    if let Some(interval) = body.poll_interval_secs {
        profile.poll_interval_secs = interval;
    }
    ctx.live_watch.save_profile(&profile).await?;
    Ok(Json(json!(profile)))
}

use authority_domain::ProposalId;

fn parse_signal_type(s: &str) -> Option<authority_domain::SignalType> {
    match s {
        "latency" => Some(authority_domain::SignalType::Latency),
        "error_rate" => Some(authority_domain::SignalType::ErrorRate),
        "action_count" => Some(authority_domain::SignalType::ActionCount),
        "version_skew" => Some(authority_domain::SignalType::VersionSkew),
        "artifact_age" => Some(authority_domain::SignalType::ArtifactAge),
        "cost_rate" => Some(authority_domain::SignalType::CostRate),
        "token_usage" => Some(authority_domain::SignalType::TokenUsage),
        "missing_heartbeat" => Some(authority_domain::SignalType::MissingHeartbeat),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<authority_domain::SignalSeverity> {
    match s {
        "info" => Some(authority_domain::SignalSeverity::Info),
        "warning" => Some(authority_domain::SignalSeverity::Warning),
        "critical" => Some(authority_domain::SignalSeverity::Critical),
        _ => None,
    }
}

fn parse_proposal_status(s: &str) -> Option<authority_domain::ProposalStatus> {
    match s {
        "proposed" => Some(authority_domain::ProposalStatus::Proposed),
        "approved" => Some(authority_domain::ProposalStatus::Approved),
        "rejected" => Some(authority_domain::ProposalStatus::Rejected),
        "executed" => Some(authority_domain::ProposalStatus::Executed),
        _ => None,
    }
}
