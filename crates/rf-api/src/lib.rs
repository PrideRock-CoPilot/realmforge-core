use axum::{routing::{get, post}, Json, Router};
use chrono::Utc;
use rf_domain::ActorScope;
use rf_policy::{authorize_action, PolicyDecision};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub production_deploy_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub scope: ActorScope,
    pub action: String,
    pub mutating: bool,
    pub approval_required: bool,
    pub max_context_age_seconds: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthorizeResponse {
    pub decision: PolicyDecision,
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/policy/authorize", post(authorize))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "realmforge-core",
        production_deploy_enabled: false,
    })
}

async fn authorize(Json(request): Json<AuthorizeRequest>) -> Json<AuthorizeResponse> {
    Json(AuthorizeResponse {
        decision: authorize_action(
            &request.scope,
            &request.action,
            request.mutating,
            request.approval_required,
            Utc::now(),
            request.max_context_age_seconds.unwrap_or(300),
        ),
    })
}
