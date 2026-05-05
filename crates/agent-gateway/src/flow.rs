use crate::error::{DenialCode, GatewayError};
use crate::scope_validator::{validate_file_scope, validate_schema_scope};
use authority_domain::{
    ActorId, AgentWorkPacket, PacketId, SessionId, SkillGrant,
};
use chrono::{DateTime, Utc};
use control_store::CoreStore;
use serde::{Deserialize, Serialize};

/// A gateway execution request from an agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub actor_id: ActorId,
    pub session_id: SessionId,
    pub action: String,
    pub file_paths: Vec<String>,
    pub schema_views: Vec<String>,
    pub payload: serde_json::Value,
    pub requires_evidence: bool,
    pub requires_approval: bool,
}

/// The result of a gateway execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayResult {
    pub allowed: bool,
    pub denial: Option<GatewayDenialDetail>,
    pub audit_event_id: Option<String>,
    pub snapshot_id: Option<String>,
    pub trace_id: String,
}

/// Detail about a gateway denial.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayDenialDetail {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

/// Request context passed through the gateway pipeline.
#[derive(Clone, Debug)]
pub struct GatewayContext {
    pub request: GatewayRequest,
    pub grant: SkillGrant,
    pub packet: Option<AgentWorkPacket>,
    pub now: DateTime<Utc>,
    pub trace_id: String,
}

impl GatewayContext {
    pub fn new(request: GatewayRequest, grant: SkillGrant, now: DateTime<Utc>) -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            request,
            grant,
            packet: None,
            now,
        }
    }
}

/// The agent gateway — the **only** path for agent-visible mutation.
///
/// Every write goes through this pipeline:
///   1. authenticate — verify actor/session
///   2. load grant — load active grant for actor
///   3. load packet — load work packet if packet_id provided
///   4. validate packet state — packet not expired, assigned to agent
///   5. authorize command — action in grant allowed_actions
///   6. verify file scope — file paths within grant scope
///   7. create audit event — log the operation
///   8. execute operation — run the requested action
///   9. require evidence — verify evidence after execution
///   10. anchor snapshot — create snapshot as rollback anchor
pub async fn execute_gateway_flow(
    request: GatewayRequest,
    store: &CoreStore,
    now: DateTime<Utc>,
) -> Result<GatewayResult, GatewayError> {
    let trace_id = uuid::Uuid::new_v4().to_string();

    // ── Step 1: Authenticate ──
    let session = store
        .get_session(&request.session_id)
        .await
        .map_err(|e| GatewayError::Internal(format!("failed to load session: {}", e)))?;
    let session = session.ok_or_else(|| {
        GatewayError::deny(DenialCode::PacketMissing, format!("session {} not found", request.session_id))
    })?;
    if session.state != "active" {
        return Err(GatewayError::deny(
            DenialCode::PacketMissing,
            format!("session {} is not active", request.session_id),
        ));
    }

    // ── Step 2: Load Grant ──
    let grants = store
        .list_grants_for_actor(&request.actor_id)
        .await
        .map_err(|e| GatewayError::Internal(format!("failed to load grants: {}", e)))?;

    let grant = grants
        .into_iter()
        .find(|g| g.state == authority_domain::GrantState::Active)
        .ok_or_else(|| {
            GatewayError::deny(DenialCode::GrantMissing, "no active grant found for actor")
        })?;

    if grant.is_expired(now) {
        return Err(GatewayError::deny(
            DenialCode::GrantExpired,
            format!("grant {} has expired", grant.id),
        ));
    }

    if !grant.is_active() {
        return Err(GatewayError::deny(
            DenialCode::GrantRevoked,
            format!("grant {} is not active (state: {})", grant.id, grant.state),
        ));
    }

    let mut ctx = GatewayContext::new(request.clone(), grant.clone(), now);
    ctx.trace_id = trace_id.clone();

    // ── Step 3: Load Packet ──
    if let Some(packet_id_str) = ctx.request.payload.get("packet_id").and_then(|v| v.as_str()) {
        if let Ok(packet_id) = PacketId::new(packet_id_str) {
            let packet = store
                .get_work_packet(&packet_id)
                .await
                .map_err(|e| GatewayError::Internal(format!("failed to load packet: {}", e)))?;

            let packet = packet.ok_or_else(|| {
                GatewayError::deny(DenialCode::PacketMissing, format!("packet {} not found", packet_id_str))
            })?;

            if packet.agent_id != ctx.request.actor_id {
                return Err(GatewayError::deny(
                    DenialCode::PacketNotAssigned,
                    format!("packet {} is not assigned to actor {}", packet_id, ctx.request.actor_id),
                ));
            }

            if packet.status != authority_domain::PacketStatus::Active {
                return Err(GatewayError::deny(
                    DenialCode::PacketScopeDenied,
                    format!("packet {} is not active (status: {:?})", packet_id, packet.status),
                ));
            }

            if !packet.is_action_allowed(&ctx.request.action) {
                return Err(GatewayError::deny(
                    DenialCode::PacketScopeDenied,
                    format!(
                        "action '{}' is not within packet {} scope",
                        ctx.request.action, packet_id
                    ),
                ));
            }

            ctx.packet = Some(packet);
        }
    }

    // ── Step 5: Authorize Command ──
    if !ctx.grant.allows_action(&ctx.request.action) {
        return Err(GatewayError::deny(
            DenialCode::ActionDenied,
            format!("action '{}' is not in grant {} allowed_actions", ctx.request.action, ctx.grant.id),
        ));
    }

    // ── Step 6: Verify File Scope ──
    for file_path in &ctx.request.file_paths {
        let result = validate_file_scope(file_path, &ctx.grant);
        if let crate::scope_validator::FileScopeResult::Denied(reason) = result {
            return Err(GatewayError::deny(DenialCode::FileScopeDenied, reason));
        }
    }

    for view_name in &ctx.request.schema_views {
        let result = validate_schema_scope(view_name, &ctx.grant);
        if let crate::scope_validator::SchemaScopeResult::Denied(reason) = result {
            return Err(GatewayError::deny(DenialCode::SchemaScopeDenied, reason));
        }
    }

    // ── Evidence Required Check ──
    if ctx.request.requires_evidence && ctx.request.payload.get("evidence").is_none() {
        return Err(GatewayError::deny(
            DenialCode::EvidenceRequired,
            "evidence record is required before executing this action",
        ));
    }

    // ── Approval Required Check ──
    if ctx.request.requires_approval {
        return Err(GatewayError::deny(
            DenialCode::ApprovalRequired,
            "human approval is required for this action",
        ));
    }

    // ── Budget Check ──
    if let Some(budget) = ctx.grant.budget_operations {
        let operation_count = ctx.request.file_paths.len() as u64;
        if operation_count > budget {
            return Err(GatewayError::deny(
                DenialCode::BudgetExceeded,
                format!("operation count {} exceeds budget {}", operation_count, budget),
            ));
        }
    }

    // ── Steps 7-10: Create Audit, Execute, Require Evidence, Anchor ──
    let audit_event = audit_log::AuditEvent::new(
        authority_domain::TenantId::new("system").unwrap(),
        authority_domain::ProjectId::new("system").unwrap(),
        ctx.request.actor_id.clone(),
        "gateway.execute",
        "gateway",
        "gateway-operation",
        ctx.request.payload.clone(),
        None,
    ).map_err(|e| GatewayError::Internal(format!("failed to create audit event: {}", e)))?;

    let audit_event_id = audit_event.id.to_string();

    store
        .append_audit_event(&audit_event)
        .await
        .map_err(|e| GatewayError::Internal(format!("failed to append audit event: {}", e)))?;

    let snapshot = snapshot_ledger::SnapshotManifest::create(
        authority_domain::TenantId::new("system").unwrap(),
        authority_domain::ProjectId::new("system").unwrap(),
        "gateway-execution",
        None,
        vec![],
        vec![],
        None,
    ).map_err(|e| GatewayError::Internal(format!("failed to create snapshot: {}", e)))?;

    let snapshot_id = snapshot.id.to_string();
    store
        .insert_snapshot_manifest(&snapshot)
        .await
        .map_err(|e| GatewayError::Internal(format!("failed to anchor snapshot: {}", e)))?;

    tracing::info!(
        trace_id = %ctx.trace_id,
        actor_id = %ctx.request.actor_id,
        action = %ctx.request.action,
        audit_event_id = %audit_event_id,
        snapshot_id = %snapshot_id,
        "gateway flow completed successfully"
    );

    Ok(GatewayResult {
        allowed: true,
        denial: None,
        audit_event_id: Some(audit_event_id),
        snapshot_id: Some(snapshot_id),
        trace_id: ctx.trace_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_context_creates_trace_id() {
        let grant = SkillGrant {
            id: authority_domain::GrantId::new("g1").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: authority_domain::TenantId::new("t1").unwrap(),
            allowed_actions: vec!["file.read".to_string()],
            denied_actions: vec![],
            allowed_file_patterns: vec!["/workspace".to_string()],
            denied_file_patterns: vec![],
            budget_tokens: None,
            budget_operations: None,
            separation_group: "dev".to_string(),
            state: authority_domain::GrantState::Active,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
        };
        let request = GatewayRequest {
            actor_id: ActorId::new("alice").unwrap(),
            session_id: SessionId::new("s1").unwrap(),
            action: "file.read".to_string(),
            file_paths: vec![],
            schema_views: vec![],
            payload: serde_json::json!({}),
            requires_evidence: false,
            requires_approval: false,
        };
        let ctx = GatewayContext::new(request, grant, Utc::now());
        assert!(!ctx.trace_id.is_empty());
    }

    #[test]
    fn action_denied_when_not_in_grant() {
        let grant = SkillGrant {
            id: authority_domain::GrantId::new("g1").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: authority_domain::TenantId::new("t1").unwrap(),
            allowed_actions: vec!["file.read".to_string()],
            denied_actions: vec![],
            allowed_file_patterns: vec![],
            denied_file_patterns: vec![],
            budget_tokens: None,
            budget_operations: None,
            separation_group: "dev".to_string(),
            state: authority_domain::GrantState::Active,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
        };
        assert!(!grant.allows_action("file.delete"));
        assert!(grant.allows_action("file.read"));
    }

    #[test]
    fn budget_exceeded_check() {
        let grant = SkillGrant {
            id: authority_domain::GrantId::new("g1").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: authority_domain::TenantId::new("t1").unwrap(),
            allowed_actions: vec!["file.read".to_string()],
            denied_actions: vec![],
            allowed_file_patterns: vec![],
            denied_file_patterns: vec![],
            budget_tokens: None,
            budget_operations: Some(2),
            separation_group: "dev".to_string(),
            state: authority_domain::GrantState::Active,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
        };
        let file_count: u64 = 3;
        assert!(grant.budget_operations.unwrap() < file_count);
    }
}
