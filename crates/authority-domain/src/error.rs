use crate::{CommandId, CommandStatus, SessionId, SkillId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid command transition: {command_id} from {from} to {to}")]
    InvalidCommandTransition {
        command_id: CommandId,
        from: CommandStatus,
        to: CommandStatus,
    },

    #[error("invalid session state transition: session {session_id} from {from} to {to}")]
    InvalidSessionTransition {
        session_id: SessionId,
        from: String,
        to: String,
    },

    #[error("skill {skill_id} integrity check failed: {reason}")]
    SkillIntegrityViolation { skill_id: SkillId, reason: String },

    #[error("scope validation failed: {0}")]
    ScopeValidation(String),

    #[error("id error: {0}")]
    Id(#[from] crate::IdError),

    #[error("command not found: {0}")]
    CommandNotFound(CommandId),

    #[error("session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("approval required but no approval id provided")]
    MissingApprovalId,

    #[error("invalid state: {0}")]
    InvalidState(String),
}

impl DomainError {
    /// Create an invalid state error.
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }
}
