use chrono::Utc;
use rf_domain::ActorScope;
use rf_policy::{authorize_action, PolicyDecision};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("invalid tool arguments")]
    InvalidArgs(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorizeArgs {
    pub scope: ActorScope,
    pub action: String,
    #[serde(default)]
    pub mutating: bool,
    #[serde(default)]
    pub approval_required: bool,
    pub max_context_age_seconds: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthorizeResult {
    pub decision: PolicyDecision,
}

pub fn core_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        tool(
            "core_authorize_command",
            "Authorize a bounded command against actor, project, skill, approval, and session scope.",
            json!({
                "type": "object",
                "required": ["scope", "action"],
                "properties": {
                    "scope": {"type": "object"},
                    "action": {"type": "string"},
                    "mutating": {"type": "boolean"},
                    "approval_required": {"type": "boolean"}
                }
            }),
        ),
        tool(
            "core_append_audit_event",
            "Append a governed audit event after policy authorization.",
            json!({
                "type": "object",
                "required": ["event"],
                "properties": {"event": {"type": "object"}}
            }),
        ),
        tool(
            "core_create_snapshot",
            "Create a content-addressed snapshot manifest for approved project state.",
            json!({
                "type": "object",
                "required": ["tenant_id", "project_id", "reason"],
                "properties": {
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "reason": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_validate_snapshot",
            "Validate snapshot manifest hashes and object references.",
            json!({
                "type": "object",
                "required": ["snapshot_id"],
                "properties": {"snapshot_id": {"type": "string"}}
            }),
        ),
        tool(
            "core_preview_rollback",
            "Preview a rollback between two snapshots without applying it.",
            json!({
                "type": "object",
                "required": ["from_snapshot_id", "to_snapshot_id"],
                "properties": {
                    "from_snapshot_id": {"type": "string"},
                    "to_snapshot_id": {"type": "string"}
                }
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: description.to_string(),
        input_schema,
    }
}

pub fn handle_tool(name: &str, args: Value) -> Result<Value, McpError> {
    match name {
        "core_authorize_command" => {
            let args: AuthorizeArgs = serde_json::from_value(args)?;
            let result = AuthorizeResult {
                decision: authorize_action(
                    &args.scope,
                    &args.action,
                    args.mutating,
                    args.approval_required,
                    Utc::now(),
                    args.max_context_age_seconds.unwrap_or(300),
                ),
            };
            Ok(serde_json::to_value(result)?)
        }
        other => Err(McpError::UnknownTool(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_tool_surface_stays_small() {
        assert!(core_tool_definitions().len() <= 15);
    }
}
