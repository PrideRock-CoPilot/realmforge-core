use authority_domain::ProjectId;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use control_service::ServiceContext;
use std::convert::Infallible;
use std::time::Duration;

use crate::{
    error::ApiError,
    models::{ChainVerifyResponse, EventsListResponse, ProjectParams, QueryEventsParams},
};

/// GET /v1/audit/events
#[utoipa::path(
    get,
    path = "/v1/audit/events",
    operation_id = "query_audit_events",
    summary = "Query audit events for a project with optional filters and pagination.",
    tag = "audit",
    params(QueryEventsParams),
    responses(
        (status = 200, description = "Events returned", body = EventsListResponse),
        (status = 400, description = "Invalid query parameters", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn query_events(
    State(ctx): State<ServiceContext>,
    Query(params): Query<QueryEventsParams>,
) -> Result<Json<EventsListResponse>, ApiError> {
    let project_id =
        ProjectId::new(&params.project_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let (events, total) = ctx
        .audit
        .query_events(
            &project_id,
            params.event_type.as_deref(),
            params.actor_id.as_ref(),
            params.entity_type.as_deref(),
            params.from_time,
            params.to_time,
            params.limit.unwrap_or(50),
            params.offset.unwrap_or(0),
        )
        .await?;

    let event_values = events
        .into_iter()
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect();

    Ok(Json(EventsListResponse {
        events: event_values,
        total,
    }))
}

/// GET /v1/audit/stream
///
/// SSE endpoint — streams audit events for a project as they arrive.
/// Reconnect-safe: pass the `Last-Event-ID` header with the last received
/// event's offset to resume without gaps.
#[utoipa::path(
    get,
    path = "/v1/audit/stream",
    operation_id = "stream_audit_events",
    summary = "Stream audit events for a project as Server-Sent Events (SSE).",
    tag = "audit",
    params(
        ("project_id" = String, Query, description = "Project ID to stream events for"),
        ("Last-Event-ID" = Option<String>, Header, description = "Resume offset from reconnect"),
    ),
    responses(
        (status = 200, description = "SSE stream of audit events (text/event-stream)"),
        (status = 400, description = "Invalid project ID", body = crate::error::ProblemDetails),
    )
)]
pub async fn stream_audit_events(
    State(ctx): State<ServiceContext>,
    Query(params): Query<ProjectParams>,
    headers: HeaderMap,
) -> Result<Sse<tokio_stream::wrappers::ReceiverStream<Result<Event, Infallible>>>, ApiError> {
    let project_id =
        ProjectId::new(&params.project_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Resume from Last-Event-ID if the client is reconnecting
    let initial_offset: u64 = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

    // Background task: polls audit store every 2s and sends new events.
    // Exits automatically when the client disconnects (tx.send fails).
    tokio::spawn(async move {
        let mut offset = initial_offset;
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let result = ctx
                .audit
                .query_events(&project_id, None, None, None, None, None, 25, offset)
                .await;
            if let Ok((events, _)) = result {
                let count = events.len() as u64;
                for (i, event) in events.into_iter().enumerate() {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    let msg = Ok(Event::default()
                        .id((offset + i as u64).to_string())
                        .data(data));
                    if tx.send(msg).await.is_err() {
                        return; // client disconnected, task exits cleanly
                    }
                }
                offset += count;
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// GET /v1/audit/chain/verify
#[utoipa::path(
    get,
    path = "/v1/audit/chain/verify",
    operation_id = "verify_audit_chain",
    summary = "Verify the integrity of the full audit hash chain for a project.",
    tag = "audit",
    params(ProjectParams),
    responses(
        (status = 200, description = "Chain verification result", body = ChainVerifyResponse),
        (status = 400, description = "Invalid project ID", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn verify_chain(
    State(ctx): State<ServiceContext>,
    Query(params): Query<ProjectParams>,
) -> Result<Json<ChainVerifyResponse>, ApiError> {
    let project_id =
        ProjectId::new(&params.project_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let anchor = ctx.audit.verify_chain(&project_id).await?;

    Ok(Json(ChainVerifyResponse {
        chain_integrity: anchor.chain_integrity,
        event_count: anchor.event_count,
    }))
}
