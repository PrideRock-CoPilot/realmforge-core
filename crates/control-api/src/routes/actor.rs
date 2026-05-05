use authority_domain::{ActorId, SessionId};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use control_service::ServiceContext;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{error::ApiError, models::ActorScopeResponse};

#[derive(Deserialize, IntoParams)]
pub struct ActorScopeParams {
    #[param(example = "ses_abc123")]
    pub session_id: String,
}

/// GET /v1/actors/:id/scope
#[utoipa::path(
    get,
    path = "/v1/actors/{id}/scope",
    operation_id = "get_actor_scope",
    summary = "Get the full ActorScope for an actor and session.",
    tag = "actors",
    params(
        ("id" = String, Path, description = "Actor ID"),
        ActorScopeParams,
    ),
    responses(
        (status = 200, description = "Actor scope returned", body = ActorScopeResponse),
        (status = 404, description = "Actor or session not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_actor_scope(
    State(ctx): State<ServiceContext>,
    Path(id): Path<ActorId>,
    Query(params): Query<ActorScopeParams>,
) -> Result<Json<ActorScopeResponse>, ApiError> {
    let session_id = SessionId::new(&params.session_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid session_id: {e}")))?;
    let scope = ctx.actors.get_scope(&id, &session_id).await?;
    Ok(Json(ActorScopeResponse { scope }))
}
