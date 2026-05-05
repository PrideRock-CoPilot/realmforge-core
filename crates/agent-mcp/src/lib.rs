pub mod error;
pub mod tools;
pub mod types;

use control_service::ServiceContext;
use error::McpError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, instrument};
use tools::build_watch::{
    GetCostSummaryArgs, GetWatchDashboardArgs, RecordWatchEventArgs,
};
use tools::bundle::{CreateBundleArgs, DeployBundleArgs, VerifyBundleArgs};
use tools::live_watch::{
    ApproveProposalArgs, GetWatchSignalsArgs, ListProposalsArgs, ProposeRemediationArgs,
};
use tools::runtime::{
    DeployRuntimeArgs, ExecuteRuntimeActionArgs, GetRuntimeStatusArgs, StopRuntimeArgs,
};
use types::{
    ActivateSkillSessionArgs, ApplyCommandArgs, AuthorizeCommandActionArgs, CompareSnapshotsArgs,
    CopyCatalogModuleArgs, CreateSnapshotArgs, ExecuteRollbackArgs, GenerateWorkPacketArgs,
    GetActorScopeArgs, IssueSessionArgs, ListCatalogModulesArgs, LoginArgs, PreviewRollbackArgs,
    ProposeCommandArgs, QueryEventsArgs, RegisterSkillArgs, RenewSessionArgs, RevokeSessionArgs,
    ValidateSnapshotArgs, ValidateWorkPacketArgs, VerifyChainArgs, VerifyRollbackArgs,
};

/// A tool definition exposed to the MCP protocol.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Return the complete list of tool definitions.
pub fn core_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // ── Session tools ──
        tool(
            "core_issue_session",
            "Issue a new session for an actor within a tenant and project.",
            json!({
                "type": "object",
                "required": ["actor_id", "tenant_id", "project_id", "ttl_seconds"],
                "properties": {
                    "actor_id": {"type": "string"},
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "ttl_seconds": {"type": "integer", "description": "Session TTL in seconds"}
                }
            }),
        ),
        tool(
            "core_renew_session",
            "Renew an existing session with a new TTL.",
            json!({
                "type": "object",
                "required": ["session_id", "ttl_seconds"],
                "properties": {
                    "session_id": {"type": "string"},
                    "ttl_seconds": {"type": "integer"}
                }
            }),
        ),
        tool(
            "core_revoke_session",
            "Revoke a session, marking it as revoked.",
            json!({
                "type": "object",
                "required": ["session_id", "reason"],
                "properties": {
                    "session_id": {"type": "string"},
                    "reason": {"type": "string"}
                }
            }),
        ),
        // ── Command tools ──
        tool(
            "core_propose_command",
            "Propose a new bounded command.",
            json!({
                "type": "object",
                "required": ["scope", "action", "target_type", "target_id", "payload"],
                "properties": {
                    "scope": {"type": "object", "description": "ActorScope for authorization context"},
                    "action": {"type": "string"},
                    "target_type": {"type": "string"},
                    "target_id": {"type": "string"},
                    "payload": {"type": "object"}
                }
            }),
        ),
        tool(
            "core_authorize_command_action",
            "Authorize a proposed command against policy engine.",
            json!({
                "type": "object",
                "required": ["command_id", "scope"],
                "properties": {
                    "command_id": {"type": "string"},
                    "scope": {"type": "object"}
                }
            }),
        ),
        tool(
            "core_apply_command",
            "Apply an authorized command, executing its effects.",
            json!({
                "type": "object",
                "required": ["command_id", "scope"],
                "properties": {
                    "command_id": {"type": "string"},
                    "scope": {"type": "object"}
                }
            }),
        ),
        // ── Audit tools ──
        tool(
            "core_query_events",
            "Query audit events for a project with optional filters.",
            json!({
                "type": "object",
                "required": ["project_id"],
                "properties": {
                    "project_id": {"type": "string"},
                    "event_type": {"type": "string"},
                    "actor_id": {"type": "string"},
                    "entity_type": {"type": "string"},
                    "limit": {"type": "integer"},
                    "offset": {"type": "integer"}
                }
            }),
        ),
        tool(
            "core_verify_chain",
            "Verify the integrity of the full audit hash chain for a project.",
            json!({
                "type": "object",
                "required": ["project_id"],
                "properties": {"project_id": {"type": "string"}}
            }),
        ),
        // ── Snapshot tools ──
        tool(
            "core_create_snapshot",
            "Create a new content-addressed snapshot manifest.",
            json!({
                "type": "object",
                "required": ["tenant_id", "project_id", "reason"],
                "properties": {
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "reason": {"type": "string"},
                    "parent_snapshot_id": {"type": "string"}
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
            "core_compare_snapshots",
            "Compare two snapshots and return the delta (added, removed, changed objects).",
            json!({
                "type": "object",
                "required": ["from_id", "to_id"],
                "properties": {
                    "from_id": {"type": "string"},
                    "to_id": {"type": "string"}
                }
            }),
        ),
        // ── Rollback tools ──
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
        tool(
            "core_execute_rollback",
            "Execute a rollback from one snapshot to another, restoring objects.",
            json!({
                "type": "object",
                "required": ["from_snapshot_id", "to_snapshot_id", "tenant_id", "project_id", "actor_id"],
                "properties": {
                    "from_snapshot_id": {"type": "string"},
                    "to_snapshot_id": {"type": "string"},
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "actor_id": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_verify_rollback",
            "Verify a rollback snapshot's consistency and hash integrity.",
            json!({
                "type": "object",
                "required": ["snapshot_id"],
                "properties": {"snapshot_id": {"type": "string"}}
            }),
        ),
        // ── Actor tools ──
        tool(
            "core_get_actor_scope",
            "Get the full ActorScope for an actor and session.",
            json!({
                "type": "object",
                "required": ["actor_id", "session_id"],
                "properties": {
                    "actor_id": {"type": "string"},
                    "session_id": {"type": "string"}
                }
            }),
        ),
        // ── Skill tools ──
        tool(
            "core_register_skill",
            "Register a new skill within a tenant/project.",
            json!({
                "type": "object",
                "required": ["skill_id", "tenant_id", "name", "allowed_actions"],
                "properties": {
                    "skill_id": {"type": "string"},
                    "tenant_id": {"type": "string"},
                    "name": {"type": "string"},
                    "project_id": {"type": "string"},
                    "allowed_actions": {"type": "array", "items": {"type": "string"}}
                }
            }),
        ),
        tool(
            "core_activate_skill_session",
            "Activate a skill session, binding a skill to a session.",
            json!({
                "type": "object",
                "required": ["skill_id", "tenant_id", "project_id", "ttl_seconds"],
                "properties": {
                    "skill_id": {"type": "string"},
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "ttl_seconds": {"type": "integer"}
                }
            }),
        ),
        // ── Catalog tools ──
        tool(
            "core_list_catalog_modules",
            "List catalog modules at a given scope (global, tenant, app).",
            json!({
                "type": "object",
                "required": ["scope"],
                "properties": {
                    "scope": {"type": "string", "description": "Scope level: global, tenant, or app"},
                    "scope_id": {"type": "string", "description": "Required for tenant or app scope"}
                }
            }),
        ),
        tool(
            "core_copy_catalog_module",
            "Copy a catalog module from one scope to another with provenance tracking.",
            json!({
                "type": "object",
                "required": ["source_id", "new_id", "new_name", "target_scope", "copied_by"],
                "properties": {
                    "source_id": {"type": "string"},
                    "new_id": {"type": "string"},
                    "new_name": {"type": "string"},
                    "new_parent_id": {"type": "string"},
                    "target_scope": {"type": "string"},
                    "target_scope_id": {"type": "string"},
                    "copied_by": {"type": "string"}
                }
            }),
        ),
        // ── Work Packet tools ──
        tool(
            "core_generate_work_packet",
            "Generate a scoped work packet for an agent from a work path node.",
            json!({
                "type": "object",
                "required": ["tenant_id", "project_id", "actor_id", "work_path_id", "node_id", "objective"],
                "properties": {
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "actor_id": {"type": "string"},
                    "work_path_id": {"type": "string", "description": "Work path graph ID containing the target node"},
                    "node_id": {"type": "string", "description": "Target work path node ID to scope the packet to"},
                    "objective": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_validate_work_packet",
            "Validate a work packet's scope boundaries and cost budget.",
            json!({
                "type": "object",
                "required": ["packet_id"],
                "properties": {"packet_id": {"type": "string"}}
            }),
        ),
        // ── Knowledge tools ──
        tool(
            "core_query_knowledge",
            "Query knowledge records with scope filtering.",
            json!({
                "type": "object",
                "required": ["scopes"],
                "properties": {
                    "scopes": {"type": "array", "items": {"type": "string"}, "description": "Scopes: global, tenant, app, work_path"},
                    "source_types": {"type": "array", "items": {"type": "string"}, "description": "Source types: catalog, file, decision, evidence, trace"},
                    "limit": {"type": "integer"},
                    "offset": {"type": "integer"}
                }
            }),
        ),
        tool(
            "core_ingest_knowledge",
            "Ingest a knowledge record with scope and source type validation.",
            json!({
                "type": "object",
                "required": ["source_id", "scope", "source_type"],
                "properties": {
                    "id": {"type": "string", "description": "Optional record ID — auto-generated if omitted"},
                    "source_id": {"type": "string"},
                    "scope": {"type": "string", "description": "Scope: global, tenant, app, work_path"},
                    "source_type": {"type": "string", "description": "Source type: catalog, file, decision, evidence, trace"}
                }
            }),
        ),
        // ── Build Watch tools ──
        tool(
            "core_record_watch_event",
            "Record a build watch event for monitoring construction-time activity.",
            json!({
                "type": "object",
                "required": ["scope", "event_type", "severity", "detail"],
                "properties": {
                    "scope": {"type": "string"},
                    "event_type": {"type": "string", "description": "file_mutation, packet_submission, policy_violation, cost_anomaly, build_failure, test_failure, evidence_gap"},
                    "severity": {"type": "string", "description": "info, warning, violation, critical"},
                    "detail": {"type": "string"},
                    "evidence_ref": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_get_watch_dashboard",
            "Get the build watch dashboard with event counts and cost summary.",
            json!({
                "type": "object",
                "required": [],
                "properties": {}
            }),
        ),
        tool(
            "core_get_cost_summary",
            "Get aggregated build cost summary grouped by scope.",
            json!({
                "type": "object",
                "required": [],
                "properties": {}
            }),
        ),
        // ── Bundle tools ──
        tool(
            "core_create_bundle",
            "Create a new bundle manifest.",
            json!({
                "type": "object",
                "required": ["bundle_id", "version", "app_id", "artifact_hashes", "governance_signature"],
                "properties": {
                    "bundle_id": {"type": "string"},
                    "version": {"type": "string"},
                    "app_id": {"type": "string"},
                    "artifact_hashes": {"type": "array", "items": {"type": "array", "prefixItems": [{"type": "string"}, {"type": "string"}], "minItems": 2, "maxItems": 2}},
                    "governance_signature": {"type": "string"},
                    "release_approval_ref": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_verify_bundle",
            "Update a bundle's verification status.",
            json!({
                "type": "object",
                "required": ["bundle_id", "status"],
                "properties": {
                    "bundle_id": {"type": "string"},
                    "status": {"type": "string", "description": "verified, signed, or failed"}
                }
            }),
        ),
        tool(
            "core_deploy_bundle",
            "Deploy a bundle by marking it as deployed.",
            json!({
                "type": "object",
                "required": ["bundle_id"],
                "properties": {
                    "bundle_id": {"type": "string"}
                }
            }),
        ),
        // ── Runtime tools ──
        tool(
            "core_get_runtime_status",
            "Get the status of a runtime instance.",
            json!({
                "type": "object",
                "required": ["runtime_id"],
                "properties": {
                    "runtime_id": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_deploy_runtime",
            "Deploy a bundle as a new runtime instance.",
            json!({
                "type": "object",
                "required": ["runtime_id", "bundle_id"],
                "properties": {
                    "runtime_id": {"type": "string"},
                    "bundle_id": {"type": "string"},
                    "active_sessions": {"type": "integer"},
                    "metadata": {"type": "object"}
                }
            }),
        ),
        tool(
            "core_execute_runtime_action",
            "Execute an action on a runtime instance.",
            json!({
                "type": "object",
                "required": ["runtime_id", "action"],
                "properties": {
                    "runtime_id": {"type": "string"},
                    "action": {"type": "string"},
                    "payload": {"type": "object"}
                }
            }),
        ),
        tool(
            "core_stop_runtime",
            "Stop a running runtime instance.",
            json!({
                "type": "object",
                "required": ["runtime_id"],
                "properties": {
                    "runtime_id": {"type": "string"}
                }
            }),
        ),
        // ── Gateway tool ──
        tool(
            "core_execute_gateway",
            "Execute a command through the agent gateway — the single entry point for all agent mutations.",
            json!({
                "type": "object",
                "required": ["actor_id", "session_id", "action"],
                "properties": {
                    "actor_id": {"type": "string"},
                    "session_id": {"type": "string"},
                    "action": {"type": "string"},
                    "file_paths": {"type": "array", "items": {"type": "string"}},
                    "schema_views": {"type": "array", "items": {"type": "string"}},
                    "payload": {"type": "object"},
                    "requires_evidence": {"type": "boolean"},
                    "requires_approval": {"type": "boolean"}
                }
            }),
        ),
        // ── Login tool ──
        tool(
            "core_login",
            "Authenticate an actor and issue a session token. Supports rate limiting, auto-blocking, and credential validation.",
            json!({
                "type": "object",
                "required": ["tenant_id", "project_id", "actor_id", "credential", "scope"],
                "properties": {
                    "tenant_id": {"type": "string"},
                    "project_id": {"type": "string"},
                    "actor_id": {"type": "string"},
                    "credential": {"type": "string", "description": "Raw credential string to authenticate with"},
                    "scope": {"type": "string", "description": "Scope string matching allowlist patterns"}
                }
            }),
        ),
        // ── Live Watch tools ──
        tool(
            "core_get_watch_signals",
            "Get recent runtime health watch signals for an app.",
            json!({
                "type": "object",
                "required": ["app_id"],
                "properties": {
                    "app_id": {"type": "string"},
                    "signal_type": {"type": "string", "description": "Filter: latency, error_rate, action_count, version_skew, artifact_age, cost_rate, token_usage, missing_heartbeat"},
                    "severity": {"type": "string", "description": "Filter: info, warning, critical"},
                    "limit": {"type": "integer"},
                    "offset": {"type": "integer"}
                }
            }),
        ),
        tool(
            "core_propose_remediation",
            "Generate a remediation proposal for an app based on recent anomaly signals.",
            json!({
                "type": "object",
                "required": ["app_id"],
                "properties": {
                    "app_id": {"type": "string"}
                }
            }),
        ),
        tool(
            "core_list_proposals",
            "List remediation proposals for an app, optionally filtered by status.",
            json!({
                "type": "object",
                "required": ["app_id"],
                "properties": {
                    "app_id": {"type": "string"},
                    "status": {"type": "string", "description": "Filter: proposed, approved, rejected, executed"},
                    "limit": {"type": "integer"},
                    "offset": {"type": "integer"}
                }
            }),
        ),
        tool(
            "core_approve_proposal",
            "Approve a remediation proposal, transitioning it to Approved status.",
            json!({
                "type": "object",
                "required": ["proposal_id"],
                "properties": {
                    "proposal_id": {"type": "string"}
                }
            }),
        ),
    ]
}

/// Handle a tool invocation, dispatching to the correct tool implementation.
#[instrument(skip(args, ctx), fields(tool_name = %name))]
pub async fn handle_tool(name: &str, args: Value, ctx: &ServiceContext) -> Result<Value, McpError> {
    let result = match name {
        // ── Session ──
        "core_issue_session" => {
            let args: IssueSessionArgs = serde_json::from_value(args)?;
            tools::session::core_issue_session(args, ctx).await
        }
        "core_renew_session" => {
            let args: RenewSessionArgs = serde_json::from_value(args)?;
            tools::session::core_renew_session(args, ctx).await
        }
        "core_revoke_session" => {
            let args: RevokeSessionArgs = serde_json::from_value(args)?;
            tools::session::core_revoke_session(args, ctx).await
        }

        // ── Command ──
        "core_propose_command" => {
            let args: ProposeCommandArgs = serde_json::from_value(args)?;
            tools::command::core_propose_command(args, ctx).await
        }
        "core_authorize_command_action" => {
            let args: AuthorizeCommandActionArgs = serde_json::from_value(args)?;
            tools::command::core_authorize_command_action(args, ctx).await
        }
        "core_apply_command" => {
            let args: ApplyCommandArgs = serde_json::from_value(args)?;
            tools::command::core_apply_command(args, ctx).await
        }

        // ── Audit ──
        "core_query_events" => {
            let args: QueryEventsArgs = serde_json::from_value(args)?;
            tools::audit::core_query_events(args, ctx).await
        }
        "core_verify_chain" => {
            let args: VerifyChainArgs = serde_json::from_value(args)?;
            tools::audit::core_verify_chain(args, ctx).await
        }

        // ── Snapshot ──
        "core_create_snapshot" => {
            let args: CreateSnapshotArgs = serde_json::from_value(args)?;
            tools::snapshot::core_create_snapshot(args, ctx).await
        }
        "core_validate_snapshot" => {
            let args: ValidateSnapshotArgs = serde_json::from_value(args)?;
            tools::snapshot::core_validate_snapshot(args, ctx).await
        }
        "core_compare_snapshots" => {
            let args: CompareSnapshotsArgs = serde_json::from_value(args)?;
            tools::snapshot::core_compare_snapshots(args, ctx).await
        }

        // ── Rollback ──
        "core_preview_rollback" => {
            let args: PreviewRollbackArgs = serde_json::from_value(args)?;
            tools::rollback::core_preview_rollback(args, ctx).await
        }
        "core_execute_rollback" => {
            let args: ExecuteRollbackArgs = serde_json::from_value(args)?;
            tools::rollback::core_execute_rollback(args, ctx).await
        }
        "core_verify_rollback" => {
            let args: VerifyRollbackArgs = serde_json::from_value(args)?;
            tools::rollback::core_verify_rollback(args, ctx).await
        }

        // ── Actor ──
        "core_get_actor_scope" => {
            let args: GetActorScopeArgs = serde_json::from_value(args)?;
            tools::actor::core_get_actor_scope(args, ctx).await
        }

        // ── Skill ──
        "core_register_skill" => {
            let args: RegisterSkillArgs = serde_json::from_value(args)?;
            tools::skill::core_register_skill(args, ctx).await
        }
        "core_activate_skill_session" => {
            let args: ActivateSkillSessionArgs = serde_json::from_value(args)?;
            tools::skill::core_activate_skill_session(args, ctx).await
        }

        // ── Catalog ──
        "core_list_catalog_modules" => {
            let args: ListCatalogModulesArgs = serde_json::from_value(args)?;
            tools::catalog::core_list_catalog_modules(args, ctx).await
        }
        "core_copy_catalog_module" => {
            let args: CopyCatalogModuleArgs = serde_json::from_value(args)?;
            tools::catalog::core_copy_catalog_module(args, ctx).await
        }

        // ── Work Packet ──
        "core_generate_work_packet" => {
            let args: GenerateWorkPacketArgs = serde_json::from_value(args)?;
            tools::work_packet::core_generate_work_packet(args, ctx).await
        }
        "core_validate_work_packet" => {
            let args: ValidateWorkPacketArgs = serde_json::from_value(args)?;
            tools::work_packet::core_validate_work_packet(args, ctx).await
        }

        // ── Gateway ──
        "core_execute_gateway" => {
            let args: agent_gateway::flow::GatewayRequest = serde_json::from_value(args)?;
            tools::gateway::core_execute_gateway(args, ctx).await
        }

        // ── Knowledge ──
        "core_query_knowledge" => {
            let args: tools::knowledge::QueryKnowledgeArgs = serde_json::from_value(args)?;
            tools::knowledge::core_query_knowledge(args, ctx).await
        }
        "core_ingest_knowledge" => {
            let args: tools::knowledge::IngestKnowledgeArgs = serde_json::from_value(args)?;
            tools::knowledge::core_ingest_knowledge(args, ctx).await
        }

        // ── Build Watch ──
        "core_record_watch_event" => {
            let args: RecordWatchEventArgs = serde_json::from_value(args)?;
            tools::build_watch::core_record_watch_event(args, ctx).await
        }
        "core_get_watch_dashboard" => {
            let args: GetWatchDashboardArgs = serde_json::from_value(args)?;
            tools::build_watch::core_get_watch_dashboard(args, ctx).await
        }
        "core_get_cost_summary" => {
            let args: GetCostSummaryArgs = serde_json::from_value(args)?;
            tools::build_watch::core_get_cost_summary(args, ctx).await
        }

        // ── Bundle ──
        "core_create_bundle" => {
            let args: CreateBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_create_bundle(args, ctx).await
        }
        "core_verify_bundle" => {
            let args: VerifyBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_verify_bundle(args, ctx).await
        }
        "core_deploy_bundle" => {
            let args: DeployBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_deploy_bundle(args, ctx).await
        }

        // ── Runtime ──
        "core_get_runtime_status" => {
            let args: GetRuntimeStatusArgs = serde_json::from_value(args)?;
            tools::runtime::core_get_runtime_status(args, ctx).await
        }
        "core_deploy_runtime" => {
            let args: DeployRuntimeArgs = serde_json::from_value(args)?;
            tools::runtime::core_deploy_runtime(args, ctx).await
        }
        "core_execute_runtime_action" => {
            let args: ExecuteRuntimeActionArgs = serde_json::from_value(args)?;
            tools::runtime::core_execute_runtime_action(args, ctx).await
        }
        "core_stop_runtime" => {
            let args: StopRuntimeArgs = serde_json::from_value(args)?;
            tools::runtime::core_stop_runtime(args, ctx).await
        }

        // ── Login ──
        "core_login" => {
            let args: LoginArgs = serde_json::from_value(args)?;
            tools::login::core_login(args, ctx).await
        }

        // ── Live Watch ──
        "core_get_watch_signals" => {
            let args: GetWatchSignalsArgs = serde_json::from_value(args)?;
            tools::live_watch::core_get_watch_signals(args, ctx).await
        }
        "core_propose_remediation" => {
            let args: ProposeRemediationArgs = serde_json::from_value(args)?;
            tools::live_watch::core_propose_remediation(args, ctx).await
        }
        "core_list_proposals" => {
            let args: ListProposalsArgs = serde_json::from_value(args)?;
            tools::live_watch::core_list_proposals(args, ctx).await
        }
        "core_approve_proposal" => {
            let args: ApproveProposalArgs = serde_json::from_value(args)?;
            tools::live_watch::core_approve_proposal(args, ctx).await
        }

        other => Err(McpError::UnknownTool(other.to_string())),
    };
    match &result {
        Ok(_) => info!(tool_name = %name, "mcp tool succeeded"),
        Err(e) => info!(tool_name = %name, error = %e, "mcp tool failed"),
    }
    result
}

fn tool(name: &str, description: &str, input_schema: Value) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: description.to_string(),
        input_schema,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tools_are_defined() {
        let defs = core_tool_definitions();
        let tool_names: std::collections::HashSet<String> =
            defs.into_iter().map(|t| t.name).collect();

        // Core tools that must be present
        for name in &[
            "core_issue_session",
            "core_renew_session",
            "core_revoke_session",
            "core_propose_command",
            "core_authorize_command_action",
            "core_apply_command",
            "core_query_events",
            "core_verify_chain",
            "core_create_snapshot",
            "core_validate_snapshot",
            "core_compare_snapshots",
            "core_preview_rollback",
            "core_execute_rollback",
            "core_verify_rollback",
            "core_get_actor_scope",
            "core_register_skill",
            "core_activate_skill_session",
            "core_list_catalog_modules",
            "core_copy_catalog_module",
            "core_generate_work_packet",
            "core_validate_work_packet",
            "core_execute_gateway",
            "core_query_knowledge",
            "core_ingest_knowledge",
            "core_record_watch_event",
            "core_get_watch_dashboard",
            "core_get_cost_summary",
            "core_create_bundle",
            "core_verify_bundle",
            "core_deploy_bundle",
            "core_get_runtime_status",
            "core_deploy_runtime",
            "core_execute_runtime_action",
            "core_stop_runtime",
        ] {
            assert!(
                tool_names.contains(*name),
                "Missing required tool: {}",
                name
            );
        }
    }

    #[test]
    fn core_tool_surface_stays_under_40() {
        assert!(core_tool_definitions().len() <= 40);
    }

    #[test]
    fn test_unknown_tool_returns_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            // connect_lazy builds the pool without opening a real connection.
            // The unknown-tool path returns immediately without touching the database.
            let ctx = ServiceContext::connect_lazy("postgres://unused:unused@localhost/unused")
                .expect("connect_lazy only validates the URI — no real connection is made");
            handle_tool("nonexistent_tool", json!({}), &ctx).await
        });
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::UnknownTool(name) => assert_eq!(name, "nonexistent_tool"),
            _ => panic!("Expected UnknownTool error"),
        }
    }

}

#[cfg(test)]
mod tests_bundle_tools {
    use super::*;

    #[test]
    fn bundle_tools_are_registered() {
        let defs = core_tool_definitions();
        let names: std::collections::HashSet<String> =
            defs.into_iter().map(|t| t.name).collect();

        assert!(names.contains("core_create_bundle"));
        assert!(names.contains("core_verify_bundle"));
        assert!(names.contains("core_deploy_bundle"));
    }

    #[test]
    fn runtime_tools_are_registered() {
        let defs = core_tool_definitions();
        let names: std::collections::HashSet<String> =
            defs.into_iter().map(|t| t.name).collect();

        assert!(names.contains("core_get_runtime_status"));
        assert!(names.contains("core_deploy_runtime"));
        assert!(names.contains("core_execute_runtime_action"));
        assert!(names.contains("core_stop_runtime"));
    }
}
