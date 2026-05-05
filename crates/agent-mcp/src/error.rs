use thiserror::Error;

/// MCP-level error type.
#[derive(Debug, Error)]
pub enum McpError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),

    #[error("invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("service error: {0}")]
    Service(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("validation error: {0}")]
    Validation(String),
}

impl McpError {
    /// Return a user-facing error message that includes corrective action.
    pub fn user_message(&self) -> String {
        match self {
            Self::UnknownTool(name) => {
                format!("Unknown tool '{}'. Available tools: core_authorize_command, core_issue_session, core_renew_session, core_revoke_session, core_propose_command, core_authorize_command_action, core_apply_command, core_query_events, core_verify_chain, core_create_snapshot, core_validate_snapshot, core_compare_snapshots, core_preview_rollback, core_execute_rollback, core_verify_rollback, core_get_actor_scope, core_register_skill, core_activate_skill_session, core_generate_work_packet, core_validate_work_packet", name)
            }
            Self::InvalidArgs(msg) => {
                format!("Invalid arguments: {}. Check the tool's input_schema for required fields and types.", msg)
            }
            Self::Service(msg) => {
                format!(
                    "Service error: {}. If this persists, check the service layer or database.",
                    msg
                )
            }
            Self::NotFound(entity) => {
                format!("{} not found. Verify the ID is correct.", entity)
            }
            Self::PolicyDenied(msg) => {
                format!("Policy denied: {}. Ensure the actor has the correct skill, session, and approval state.", msg)
            }
            Self::Validation(msg) => {
                format!("Validation failed: {}. Correct the input and retry.", msg)
            }
        }
    }
}

impl From<control_service::ServiceError> for McpError {
    fn from(err: control_service::ServiceError) -> Self {
        match err {
            control_service::ServiceError::SessionNotFound => {
                McpError::NotFound("session".to_string())
            }
            control_service::ServiceError::CommandNotFound => {
                McpError::NotFound("command".to_string())
            }
            control_service::ServiceError::SnapshotNotFound => {
                McpError::NotFound("snapshot".to_string())
            }
            control_service::ServiceError::ActorNotFound => McpError::NotFound("actor".to_string()),
            control_service::ServiceError::SkillNotFound => McpError::NotFound("skill".to_string()),
            control_service::ServiceError::RollbackPreviewNotFound => {
                McpError::NotFound("rollback preview".to_string())
            }
            control_service::ServiceError::Validation(msg) => McpError::Validation(msg),
            control_service::ServiceError::PolicyDenied(d) => {
                let msg = d
                    .denial
                    .map(|x| format!("{:?}: {}", x.code, x.message))
                    .unwrap_or_default();
                McpError::PolicyDenied(msg)
            }
            _ => McpError::Service(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::InvalidArgs(err.to_string())
    }
}
