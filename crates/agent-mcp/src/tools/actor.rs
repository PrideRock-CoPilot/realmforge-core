use control_service::ServiceContext;
use serde_json::Value;

use crate::{error::McpError, types::GetActorScopeArgs};

/// core_get_actor_scope — Get the full ActorScope for an actor and session.
pub async fn core_get_actor_scope(
    args: GetActorScopeArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let scope = ctx
        .actors
        .get_scope(&args.actor_id, &args.session_id)
        .await?;
    Ok(serde_json::to_value(scope)?)
}
