use audit_log::EventError;
use authority_domain::DomainError;
use control_store::StoreError;
use policy_engine::PolicyDecision;
use snapshot_ledger::{ManifestError, ObjectStoreError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    // ── Domain violations ──
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),

    // ── Store/persistence errors ──
    #[error("store error: {0}")]
    Store(#[from] StoreError),

    // ── Event errors ──
    #[error("event error: {0}")]
    Event(#[from] EventError),

    // ── Policy denials ──
    #[error("policy denied: {0}")]
    PolicyDenied(PolicyDecision),

    // ── Snapshot errors ──
    #[error("manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("object store error: {0}")]
    ObjectStore(#[from] ObjectStoreError),

    // ── Not found errors ──
    #[error("session not found")]
    SessionNotFound,

    #[error("command not found")]
    CommandNotFound,

    #[error("snapshot not found")]
    SnapshotNotFound,

    #[error("actor not found")]
    ActorNotFound,

    #[error("skill not found")]
    SkillNotFound,

    #[error("rollback preview not found")]
    RollbackPreviewNotFound,

    #[error("work packet not found")]
    WorkPacketNotFound,

    #[error("work path not found")]
    WorkPathNotFound,

    // ── Login errors ──
    #[error("login blocked: {0}")]
    LoginBlocked(String),

    #[error("rate limited: retry after {0} seconds")]
    RateLimited(u64),

    #[error("invalid credentials")]
    InvalidCredentials,

    // ── Validation errors ──
    #[error("validation error: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // ── Configuration errors ──
    #[error("configuration error: {0}")]
    Configuration(String),

    // ── Internal errors ──
    #[error("internal error: {0}")]
    Internal(String),
}

impl ServiceError {
    /// Returns a human-readable error code for API responses.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Domain(_) => "DOMAIN_ERROR",
            Self::Store(_) => "STORE_ERROR",
            Self::Event(_) => "EVENT_ERROR",
            Self::PolicyDenied(_) => "POLICY_DENIED",
            Self::Manifest(_) => "MANIFEST_ERROR",
            Self::ObjectStore(_) => "OBJECT_STORE_ERROR",
            Self::LoginBlocked(_) => "LOGIN_BLOCKED",
            Self::RateLimited(_) => "RATE_LIMITED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::CommandNotFound => "COMMAND_NOT_FOUND",
            Self::SnapshotNotFound => "SNAPSHOT_NOT_FOUND",
            Self::ActorNotFound => "ACTOR_NOT_FOUND",
            Self::SkillNotFound => "SKILL_NOT_FOUND",
            Self::RollbackPreviewNotFound => "ROLLBACK_PREVIEW_NOT_FOUND",
            Self::WorkPacketNotFound => "WORK_PACKET_NOT_FOUND",
            Self::WorkPathNotFound => "WORK_PATH_NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Returns a severity level for observability.
    pub fn severity(&self) -> &'static str {
        match self {
            Self::PolicyDenied(_) | Self::RateLimited(_) => "info",
            Self::LoginBlocked(_) | Self::InvalidCredentials => "warn",
            Self::Validation(_) => "warn",
            Self::Domain(_) | Self::Store(_) | Self::Event(_) => "error",
            Self::Internal(_) => "critical",
            _ => "warn",
        }
    }
}

impl From<PolicyDecision> for ServiceError {
    fn from(decision: PolicyDecision) -> Self {
        ServiceError::PolicyDenied(decision)
    }
}
