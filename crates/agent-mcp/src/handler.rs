//! MCP tool dispatch — routes tool names to their implementation.
//!
//! This module owns the [`handle_tool`] function that deserializes arguments
//! and dispatches to the appropriate tool implementation in the `tools` module.

use control_service::ServiceContext;
use serde_json::Value;
use tracing::{info, instrument};

use crate::error::McpError;
use crate::tools;
use crate::types::{
    ActivateSkillSessionArgs, ApplyCommandArgs, AuthorizeCommandActionArgs, CompareSnapshotsArgs,
    CopyCatalogModuleArgs, CreateSnapshotArgs, ExecuteRollbackArgs, GenerateWorkPacketArgs,
    GetActorScopeArgs, IssueSessionArgs, ListCatalogModulesArgs, LoginArgs, PreviewRollbackArgs,
    ProposeCommandArgs, QueryEventsArgs, RegisterSkillArgs, RenewSessionArgs, RevokeSessionArgs,
    ValidateSnapshotArgs, ValidateWorkPacketArgs, VerifyChainArgs, VerifyRollbackArgs,
};

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
            let args: tools::build_watch::RecordWatchEventArgs = serde_json::from_value(args)?;
            tools::build_watch::core_record_watch_event(args, ctx).await
        }
        "core_get_watch_dashboard" => {
            let args: tools::build_watch::GetWatchDashboardArgs = serde_json::from_value(args)?;
            tools::build_watch::core_get_watch_dashboard(args, ctx).await
        }
        "core_get_cost_summary" => {
            let args: tools::build_watch::GetCostSummaryArgs = serde_json::from_value(args)?;
            tools::build_watch::core_get_cost_summary(args, ctx).await
        }

        // ── Bundle ──
        "core_create_bundle" => {
            let args: tools::bundle::CreateBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_create_bundle(args, ctx).await
        }
        "core_verify_bundle" => {
            let args: tools::bundle::VerifyBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_verify_bundle(args, ctx).await
        }
        "core_deploy_bundle" => {
            let args: tools::bundle::DeployBundleArgs = serde_json::from_value(args)?;
            tools::bundle::core_deploy_bundle(args, ctx).await
        }

        // ── Runtime ──
        "core_get_runtime_status" => {
            let args: tools::runtime::GetRuntimeStatusArgs = serde_json::from_value(args)?;
            tools::runtime::core_get_runtime_status(args, ctx).await
        }
        "core_deploy_runtime" => {
            let args: tools::runtime::DeployRuntimeArgs = serde_json::from_value(args)?;
            tools::runtime::core_deploy_runtime(args, ctx).await
        }
        "core_execute_runtime_action" => {
            let args: tools::runtime::ExecuteRuntimeActionArgs = serde_json::from_value(args)?;
            tools::runtime::core_execute_runtime_action(args, ctx).await
        }
        "core_stop_runtime" => {
            let args: tools::runtime::StopRuntimeArgs = serde_json::from_value(args)?;
            tools::runtime::core_stop_runtime(args, ctx).await
        }

        // ── Login ──
        "core_login" => {
            let args: LoginArgs = serde_json::from_value(args)?;
            tools::login::core_login(args, ctx).await
        }

        // ── Live Watch ──
        "core_get_watch_signals" => {
            let args: tools::live_watch::GetWatchSignalsArgs = serde_json::from_value(args)?;
            tools::live_watch::core_get_watch_signals(args, ctx).await
        }
        "core_propose_remediation" => {
            let args: tools::live_watch::ProposeRemediationArgs = serde_json::from_value(args)?;
            tools::live_watch::core_propose_remediation(args, ctx).await
        }
        "core_list_proposals" => {
            let args: tools::live_watch::ListProposalsArgs = serde_json::from_value(args)?;
            tools::live_watch::core_list_proposals(args, ctx).await
        }
        "core_approve_proposal" => {
            let args: tools::live_watch::ApproveProposalArgs = serde_json::from_value(args)?;
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
