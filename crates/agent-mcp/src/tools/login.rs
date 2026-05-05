use authority_domain::login::{LoginCredentials, Scope};
use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::{error::McpError, types::LoginArgs};

/// core_login — Authenticate an actor and issue a session token.
pub async fn core_login(args: LoginArgs, ctx: &ServiceContext) -> Result<Value, McpError> {
    let scope =
        Scope::new(&args.scope).map_err(|e| McpError::InvalidArgs(format!("invalid scope: {}", e)))?;

    let credentials = LoginCredentials {
        actor_id: args.actor_id,
        credential: args.credential.into_bytes(),
        scope,
    };

    let response = ctx
        .login
        .handle_login(args.tenant_id, args.project_id, credentials)
        .await?;

    Ok(json!({
        "session_token": response.session_token,
        "actor_id": response.actor_id.as_str(),
        "scope": response.scope,
        "expires_at": response.expires_at.to_rfc3339(),
        "audit_event_id": response.audit_event_id,
    }))
}
