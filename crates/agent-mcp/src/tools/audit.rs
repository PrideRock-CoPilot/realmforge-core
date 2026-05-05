use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::{
    error::McpError,
    types::{QueryEventsArgs, VerifyChainArgs},
};

/// core_query_events — Query audit events for a project.
pub async fn core_query_events(
    args: QueryEventsArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let (events, total) = ctx
        .audit
        .query_events(
            &args.project_id,
            args.event_type.as_deref(),
            args.actor_id.as_ref(),
            args.entity_type.as_deref(),
            None,
            None,
            args.limit.unwrap_or(50),
            args.offset.unwrap_or(0),
        )
        .await?;

    let event_values: Vec<Value> = events
        .into_iter()
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect();

    Ok(json!({
        "events": event_values,
        "total": total,
    }))
}

/// core_verify_chain — Verify the integrity of the audit hash chain.
pub async fn core_verify_chain(
    args: VerifyChainArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let anchor = ctx.audit.verify_chain(&args.project_id).await?;
    Ok(serde_json::to_value(anchor)?)
}
