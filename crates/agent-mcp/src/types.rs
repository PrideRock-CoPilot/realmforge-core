use authority_domain::{
    ActorId, ActorScope, CatalogId, CommandId, PacketId, ProjectId, SessionId, SkillId, SnapshotId,
    TenantId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Session types ──

#[derive(Debug, Deserialize)]
pub struct IssueSessionArgs {
    pub actor_id: ActorId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub ttl_seconds: i64,
}

#[derive(Debug, Deserialize)]
pub struct RenewSessionArgs {
    pub session_id: SessionId,
    pub ttl_seconds: i64,
}

#[derive(Debug, Deserialize)]
pub struct RevokeSessionArgs {
    pub session_id: SessionId,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct GetSessionArgs {
    pub session_id: SessionId,
}

// ── Command types ──

#[derive(Debug, Deserialize)]
pub struct ProposeCommandArgs {
    pub scope: ActorScope,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeCommandActionArgs {
    pub command_id: CommandId,
    pub scope: ActorScope,
}

#[derive(Debug, Deserialize)]
pub struct ApplyCommandArgs {
    pub command_id: CommandId,
    pub scope: ActorScope,
}

// ── Audit types ──

#[derive(Debug, Deserialize)]
pub struct QueryEventsArgs {
    pub project_id: ProjectId,
    pub event_type: Option<String>,
    pub actor_id: Option<ActorId>,
    pub entity_type: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyChainArgs {
    pub project_id: ProjectId,
}

// ── Snapshot types ──

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub reason: String,
    pub parent_snapshot_id: Option<SnapshotId>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateSnapshotArgs {
    pub snapshot_id: SnapshotId,
}

#[derive(Debug, Deserialize)]
pub struct CompareSnapshotsArgs {
    pub from_id: SnapshotId,
    pub to_id: SnapshotId,
}

// ── Rollback types ──

#[derive(Debug, Deserialize)]
pub struct PreviewRollbackArgs {
    pub from_snapshot_id: SnapshotId,
    pub to_snapshot_id: SnapshotId,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteRollbackArgs {
    pub from_snapshot_id: SnapshotId,
    pub to_snapshot_id: SnapshotId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRollbackArgs {
    pub snapshot_id: SnapshotId,
}

// ── Actor types ──

#[derive(Debug, Deserialize)]
pub struct GetActorScopeArgs {
    pub actor_id: ActorId,
    pub session_id: SessionId,
}

// ── Skill types ──

#[derive(Debug, Deserialize)]
pub struct RegisterSkillArgs {
    pub skill_id: SkillId,
    pub tenant_id: TenantId,
    pub name: String,
    pub project_id: Option<ProjectId>,
    pub allowed_actions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivateSkillSessionArgs {
    pub skill_id: SkillId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub ttl_seconds: i64,
}

// ── Work Packet types ──

#[derive(Debug, Deserialize)]
pub struct GenerateWorkPacketArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub work_path_id: authority_domain::WorkPathId,
    pub node_id: authority_domain::WorkPathNodeId,
    pub objective: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidateWorkPacketArgs {
    pub packet_id: PacketId,
}

// ── Catalog types ──

#[derive(Debug, Deserialize)]
pub struct ListCatalogModulesArgs {
    pub scope: String,
    pub scope_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CopyCatalogModuleArgs {
    pub source_id: CatalogId,
    pub new_id: CatalogId,
    pub new_name: String,
    pub new_parent_id: Option<CatalogId>,
    pub target_scope: String,
    pub target_scope_id: Option<String>,
    pub copied_by: authority_domain::ActorId,
}

// ── Shared types ──

#[derive(Debug, Serialize)]
pub struct McpToolResult {
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
}

// ── Login types ──

#[derive(Debug, Deserialize)]
pub struct LoginArgs {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    /// Raw credential string to authenticate with.
    pub credential: String,
    /// Scope string matching allowlist patterns.
    pub scope: String,
}

impl McpToolResult {
    pub fn ok(data: Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }

    pub fn err(msg: String) -> Self {
        Self {
            success: false,
            data: Value::Null,
            error: Some(msg),
        }
    }
}
