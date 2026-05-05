use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use control_service::ServiceError;
use serde::Serialize;
use thiserror::Error;

/// API-level error type that maps service errors to HTTP responses.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("rate limited: retry after {0} seconds")]
    TooManyRequests(u64),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::SessionNotFound
            | ServiceError::CommandNotFound
            | ServiceError::SnapshotNotFound
            | ServiceError::ActorNotFound
            | ServiceError::SkillNotFound
            | ServiceError::RollbackPreviewNotFound
            | ServiceError::WorkPacketNotFound
            | ServiceError::WorkPathNotFound => ApiError::NotFound(err.to_string()),

            ServiceError::LoginBlocked(msg) => ApiError::PolicyDenied(msg),
            ServiceError::InvalidCredentials => ApiError::BadRequest(err.to_string()),
            ServiceError::RateLimited(retry_after) => {
                // Map to 429 Too Many Requests with Retry-After
                ApiError::TooManyRequests(retry_after)
            }

            ServiceError::Validation(msg) => ApiError::BadRequest(msg),
            ServiceError::PolicyDenied(decision) => {
                let msg = decision
                    .denial
                    .map(|d| format!("{:?}: {}", d.code, d.message))
                    .unwrap_or_else(|| "policy denied".to_string());
                ApiError::PolicyDenied(msg)
            }
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

/// RFC 7807 Problem Details response, extended with a machine-readable error code
/// and trace ID per the RealmForge API error standard.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    /// Machine-readable SCREAMING_SNAKE_CASE error code. Stable across versions.
    pub code: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Correlation ID matching the X-Request-Id response header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match &self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg.clone()),
            ApiError::PolicyDenied(msg) => (StatusCode::FORBIDDEN, "Policy Denied", msg.clone()),
            ApiError::TooManyRequests(retry_after) => {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("Retry-After", &retry_after.to_string())],
                    Json(ProblemDetails {
                        code: "TOO_MANY_REQUESTS".to_string(),
                        title: "Too Many Requests".to_string(),
                        status: StatusCode::TOO_MANY_REQUESTS.as_u16(),
                        detail: format!("retry after {} seconds", retry_after),
                        instance: None,
                        trace_id: None,
                    }),
                )
                    .into_response()
            }
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    "An unexpected error occurred".to_string(),
                )
            }
        };

        let body = ProblemDetails {
            code: title.to_uppercase().replace(' ', "_"),
            title: title.to_string(),
            status: status.as_u16(),
            detail,
            instance: None,
            trace_id: None,
        };

        (status, Json(body)).into_response()
    }
}
