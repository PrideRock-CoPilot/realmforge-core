use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::atomic::Ordering;

/// Access the global request counter from middleware.
use crate::middleware::REQUEST_COUNTER;

#[derive(Serialize)]
pub struct MetricsResponse {
    pub service: &'static str,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub request_counter: u64,
}

/// GET /metrics — basic diagnostic metrics endpoint.
///
/// Exposes:
/// - `service`: the service name
/// - `uptime_seconds`: system uptime (epoch-based)
/// - `total_requests` / `request_counter`: total HTTP requests processed
pub async fn get_metrics() -> impl IntoResponse {
    let start = std::time::Instant::now();
    let _span = tracing::info_span!("metrics_endpoint").entered();

    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let counter = REQUEST_COUNTER.load(Ordering::Relaxed);

    let response = MetricsResponse {
        service: "control-api",
        uptime_seconds: uptime,
        total_requests: counter,
        request_counter: counter,
    };

    tracing::debug!(elapsed_us = %start.elapsed().as_micros(), "metrics endpoint");
    (StatusCode::OK, Json(response))
}
