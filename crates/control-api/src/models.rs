use authority_domain::{ActorId, ActorScope, CommandId, PacketId, ProjectId, SnapshotId, TenantId};
use chrono::{DateTime, Utc};
use control_service::{
    rollback_service::{RollbackPreview, RollbackResult, RollbackVerification},
    session_service::SessionData,
    snapshot_service::{SnapshotDelta, SnapshotValidationReport},
    work_packet_service::PacketValidationResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::IntoParams;

// ── Session endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IssueSessionRequest {
    #[schema(value_type = String, example = "actor_01")]
    pub tenant_id: TenantId,
    #[schema(value_type = String, example = "proj_01")]
    pub project_id: ProjectId,
    #[schema(value_type = String, example = "alice")]
    pub actor_id: ActorId,
    #[schema(example = 3600)]
    pub ttl_seconds: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionResponse {
    #[schema(value_type = Object)]
    pub session: SessionData,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RenewSessionRequest {
    #[schema(example = 3600)]
    pub ttl_seconds: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RevokeSessionResponse {
    pub status: String,
    pub session_id: String,
}

// ── Command endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProposeCommandRequest {
    #[schema(value_type = Object)]
    #[serde(default)]
    pub scope: ActorScope,
    #[schema(example = "command.board.approve")]
    pub action: String,
    #[schema(example = "board")]
    pub target_type: String,
    #[schema(example = "board_01")]
    pub target_id: String,
    #[schema(value_type = Object)]
    pub payload: Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DenyCommandRequest {
    #[schema(value_type = Object)]
    pub scope: ActorScope,
    pub reason: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommandResponse {
    #[schema(value_type = String)]
    pub command_id: CommandId,
    pub status: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
}

// ── Audit endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct QueryEventsParams {
    pub project_id: String,
    pub event_type: Option<String>,
    #[param(value_type = Option<String>)]
    #[schema(value_type = Option<String>)]
    pub actor_id: Option<ActorId>,
    pub entity_type: Option<String>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ProjectParams {
    pub project_id: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EventsListResponse {
    #[schema(value_type = Vec<Object>)]
    pub events: Vec<Value>,
    pub total: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ChainVerifyResponse {
    pub chain_integrity: bool,
    pub event_count: u64,
}

// ── Snapshot endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSnapshotRequest {
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    #[schema(value_type = String)]
    pub project_id: ProjectId,
    pub reason: String,
    #[schema(value_type = Option<String>)]
    pub parent_snapshot_id: Option<SnapshotId>,
    pub previous_manifest_hash: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotResponse {
    #[schema(value_type = String)]
    pub snapshot_id: SnapshotId,
    pub status: String,
    pub reason: String,
    pub object_count: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotValidateResponse {
    #[schema(value_type = Object)]
    pub report: SnapshotValidationReport,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotCompareResponse {
    #[schema(value_type = Object)]
    pub delta: SnapshotDelta,
}

// ── Rollback endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RollbackPreviewRequest {
    #[schema(value_type = String)]
    pub from_snapshot_id: SnapshotId,
    #[schema(value_type = String)]
    pub to_snapshot_id: SnapshotId,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RollbackPreviewResponse {
    #[schema(value_type = Object)]
    pub preview: RollbackPreview,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RollbackExecuteRequest {
    #[schema(value_type = String)]
    pub from_snapshot_id: SnapshotId,
    #[schema(value_type = String)]
    pub to_snapshot_id: SnapshotId,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    #[schema(value_type = String)]
    pub project_id: ProjectId,
    #[schema(value_type = String)]
    pub actor_id: ActorId,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RollbackExecuteResponse {
    #[schema(value_type = Object)]
    pub result: RollbackResult,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RollbackVerifyResponse {
    #[schema(value_type = Object)]
    pub verification: RollbackVerification,
}

// ── Actor endpoints ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActorScopeResponse {
    #[schema(value_type = Object)]
    pub scope: ActorScope,
}

// ── Work Packet endpoints ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkPacketGenerateResponse {
    #[schema(value_type = String)]
    pub packet_id: PacketId,
    pub status: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkPacketValidateResponse {
    #[schema(value_type = Object)]
    pub result: PacketValidationResult,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateWorkPacketRequest {
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    #[schema(value_type = String)]
    pub project_id: ProjectId,
    #[schema(value_type = String)]
    pub actor_id: ActorId,
    pub work_path_node_id: String,
    pub objective: String,
    pub allowed_file_paths: Vec<String>,
}

// ── Scope / policy endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AuthorizeRequest {
    #[schema(value_type = Object)]
    pub scope: ActorScope,
    pub action: String,
    pub mutating: bool,
    pub approval_required: bool,
    pub max_context_age_seconds: Option<i64>,
}

/// Typed request body for command authorization and application endpoints.
/// Mirrors the ActorScope fields so utoipa can generate a schema without
/// requiring ActorScope to implement ToSchema.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CommandScopeRequest {
    #[schema(value_type = String, example = "ten_01")]
    pub tenant_id: TenantId,
    #[schema(value_type = String, example = "proj_01")]
    pub project_id: ProjectId,
    #[schema(value_type = String, example = "alice")]
    pub actor_id: ActorId,
    pub allowed_actions: Vec<String>,
    pub approval_state: String,
    pub execution_mode: String,
    #[schema(value_type = Option<String>)]
    pub session_id: Option<authority_domain::SessionId>,
}

// ── Login endpoints ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub session_token: String,
    #[schema(value_type = String)]
    pub actor_id: authority_domain::ActorId,
    pub scope: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub audit_event_id: String,
}

// ── Health endpoints ──

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub production_deploy_enabled: bool,
}

// ── Boards endpoints ──

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateBoardPlanRequest {
    #[schema(example = "Login Vertical — Phase 9")]
    pub title: String,
    pub work_path_refs: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BoardPlanResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub work_path_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BoardPlanListResponse {
    pub plans: Vec<BoardPlanResponse>,
    pub total: u64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ListPlansQuery {
    /// Filter by board state. One of: draft, in_review, approved, in_progress, completed, blocked, archived.
    pub status: Option<String>,
    #[schema(example = 20)]
    pub limit: Option<i64>,
    #[schema(example = 0)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BoardApprovalRequest {
    pub approver: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BoardReleaseRequest {
    pub plan_id: String,
    pub bundle_ref: String,
    pub approval_ref: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BoardReleaseResponse {
    pub command_id: String,
    pub status: String,
    pub plan_id: String,
}
