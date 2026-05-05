use control_service::ServiceContext;
use serde_json::Value;

use crate::{
    error::McpError,
    types::{ActivateSkillSessionArgs, RegisterSkillArgs},
};

/// core_register_skill — Register a new skill within a tenant/project.
pub async fn core_register_skill(
    args: RegisterSkillArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let registration = ctx
        .skills
        .register_skill(
            &args.skill_id,
            &args.tenant_id,
            args.project_id.as_ref(),
            &args.name,
            args.allowed_actions,
        )
        .await?;
    Ok(serde_json::to_value(registration)?)
}

/// core_activate_skill_session — Activate a skill session, binding skill to session.
pub async fn core_activate_skill_session(
    args: ActivateSkillSessionArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let session = ctx
        .skills
        .activate_skill_session(
            &args.skill_id,
            &args.tenant_id,
            &args.project_id,
            args.ttl_seconds,
        )
        .await?;
    Ok(serde_json::to_value(session)?)
}
