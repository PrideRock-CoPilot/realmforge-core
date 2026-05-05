//! RealmForge agent-mcp crate.
//!
//! MCP protocol surface: tool definitions, dispatch, and type marshalling.
//!
//! ## Module structure
//!
//! - [`definitions`] — [`ToolDefinition`] struct and [`core_tool_definitions()`] function
//! - [`handler`] — [`handle_tool()`] route-to-implementation dispatch
//! - [`types`] — Argument types for all tools
//! - [`tools`] — Tool implementations (one module per domain)
//! - [`error`] — MCP error types

pub mod definitions;
pub mod error;
pub mod handler;
pub mod tools;
pub mod types;

pub use definitions::{core_tool_definitions, ToolDefinition};
pub use handler::handle_tool;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::McpError;
    use control_service::ServiceContext;
    use serde_json::json;

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
        let names: std::collections::HashSet<String> = defs.into_iter().map(|t| t.name).collect();

        assert!(names.contains("core_create_bundle"));
        assert!(names.contains("core_verify_bundle"));
        assert!(names.contains("core_deploy_bundle"));
    }

    #[test]
    fn runtime_tools_are_registered() {
        let defs = core_tool_definitions();
        let names: std::collections::HashSet<String> = defs.into_iter().map(|t| t.name).collect();

        assert!(names.contains("core_get_runtime_status"));
        assert!(names.contains("core_deploy_runtime"));
        assert!(names.contains("core_execute_runtime_action"));
        assert!(names.contains("core_stop_runtime"));
    }
}
