use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use control_service::ServiceContext;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info_span;
use uuid::Uuid;

pub(crate) static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Middleware that adds request tracing and request ID.
pub async fn request_tracing_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let request_number = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);

    request.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
    });

    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %request.method(),
        path = %request.uri().path(),
        number = request_number,
    );

    let _guard = span.enter();
    let mut response = next.run(request).await;
    drop(_guard);

    // Set X-Request-Id header on the response for trace correlation
    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }

    response
}

/// Context set by the tracing middleware for downstream use.
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
}

/// Bearer token extraction middleware.
///
/// Extracts `Authorization: Bearer <token>` from the request header, validates
/// the token against the session store, and inserts `SessionData` into request
/// extensions. If the token is present but invalid, returns 401. If the header
/// is absent, the request continues unauthenticated (individual routes enforce
/// auth by requiring `SessionData` from extensions).
///
/// SECURITY: the Authorization header is intentionally NOT included in any
/// tracing span to prevent token leakage in log aggregation pipelines.
pub async fn bearer_auth_middleware(
    axum::extract::State(ctx): axum::extract::State<ServiceContext>,
    mut request: Request,
    next: Next,
) -> Response {
    // Authorization header is never logged — TraceLayer spans are scoped to
    // method/path/status only. Do not add headers to any span in this crate.
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_owned());

    if let Some(raw_token) = token {
        match authority_domain::SessionId::new(&raw_token) {
            Ok(session_id) => match ctx.sessions.validate_session(&session_id).await {
                Ok(session_data) => {
                    request.extensions_mut().insert(session_data);
                }
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(crate::error::ProblemDetails {
                            code: "INVALID_SESSION".to_string(),
                            title: "Unauthorized".to_string(),
                            status: 401,
                            detail: "Bearer token is invalid or expired.".to_string(),
                            instance: None,
                            trace_id: None,
                        }),
                    )
                        .into_response();
                }
            },
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(crate::error::ProblemDetails {
                        code: "INVALID_SESSION".to_string(),
                        title: "Unauthorized".to_string(),
                        status: 401,
                        detail: "Malformed Bearer token.".to_string(),
                        instance: None,
                        trace_id: None,
                    }),
                )
                    .into_response();
            }
        }
    }

    next.run(request).await
}
