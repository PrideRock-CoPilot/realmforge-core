use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::{
    error::McpError,
    types::{IssueSessionArgs, RenewSessionArgs, RevokeSessionArgs},
};

/// core_issue_session — Issue a new session for an actor.
pub async fn core_issue_session(
    args: IssueSessionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let session = ctx
        .sessions
        .issue_session(
            &args.actor_id,
            &args.tenant_id,
            &args.project_id,
            args.ttl_seconds,
        )
        .await?;
    Ok(serde_json::to_value(session)?)
}

/// core_renew_session — Renew an existing session with a new TTL.
pub async fn core_renew_session(
    args: RenewSessionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let session = ctx
        .sessions
        .renew_session(&args.session_id, args.ttl_seconds)
        .await?;
    Ok(serde_json::to_value(session)?)
}

/// core_revoke_session — Revoke a session.
pub async fn core_revoke_session(
    args: RevokeSessionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    ctx.sessions
        .revoke_session(&args.session_id, &args.reason)
        .await?;
    Ok(json!({"status": "revoked", "session_id": args.session_id.as_str()}))
}
