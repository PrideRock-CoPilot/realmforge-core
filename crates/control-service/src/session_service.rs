use authority_domain::{ActorId, ActorScope, ProjectId, RoleId, SessionId, TenantId};
use chrono::{DateTime, Duration, Utc};
use control_store::{sessions::SessionRow, CoreStore};
use serde::{Deserialize, Serialize};

use tracing::{info, instrument};

use crate::error::ServiceError;
use crate::AuditService;

/// Session data as stored/retrieved from the store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub id: SessionId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub state: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<SessionRow> for SessionData {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            project_id: row.project_id,
            actor_id: row.actor_id,
            state: row.state,
            issued_at: row.issued_at,
            expires_at: row.expires_at,
        }
    }
}

/// Session lifecycle service.
///
/// Orchestrates: issue → activate → renew → revoke → expire lifecycle,
/// with audit events emitted on every state transition.
#[derive(Clone)]
pub struct SessionService {
    store: CoreStore,
    audit_service: AuditService,
}

impl SessionService {
    pub fn new(store: CoreStore, audit_service: AuditService) -> Self {
        Self {
            store,
            audit_service,
        }
    }

    /// Issue a new session for an actor within a tenant and project.
    #[instrument(skip(self), fields(actor_id = %actor_id, tenant_id = %tenant_id, project_id = %project_id, ttl = ttl_seconds))]
    pub async fn issue_session(
        &self,
        actor_id: &ActorId,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        ttl_seconds: i64,
    ) -> Result<SessionData, ServiceError> {
        let now = Utc::now();
        let session_id = SessionId::generate();
        let expires_at = now + Duration::seconds(ttl_seconds);

        let row = SessionRow {
            id: session_id,
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            actor_id: actor_id.clone(),
            state: "issued".to_string(),
            issued_at: now,
            expires_at,
        };

        self.store.insert_session(&row).await?;

        // Emit audit event
        self.audit_service
            .append_chained_event(
                tenant_id,
                project_id,
                actor_id,
                "session.issued",
                "session",
                row.id.as_str(),
                serde_json::json!({
                    "session_id": row.id.as_str(),
                    "ttl_seconds": ttl_seconds,
                    "expires_at": expires_at.to_rfc3339(),
                }),
            )
            .await?;

        let session: SessionData = row.into();
        info!(session_id = %session.id, "session issued");
        Ok(session)
    }

    /// Activate an issued session.
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn activate_session(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionData, ServiceError> {
        let row = self
            .store
            .get_session(session_id)
            .await?
            .ok_or(ServiceError::SessionNotFound)?;

        if row.state != "issued" {
            return Err(ServiceError::Validation(format!(
                "cannot activate session in state: {}",
                row.state
            )));
        }

        self.store
            .update_session_state(session_id, "active")
            .await?;

        // Emit audit event
        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&row.project_id)
            .await?;
        self.audit_service
            .append_event(
                &row.tenant_id,
                &row.project_id,
                &row.actor_id,
                "session.activated",
                "session",
                row.id.as_str(),
                serde_json::json!({
                    "session_id": row.id.as_str(),
                    "previous_state": "issued",
                }),
                previous_hash,
            )
            .await?;

        let mut updated = row;
        updated.state = "active".to_string();
        let result = updated.into();
        info!("session activated");
        Ok(result)
    }

    /// Renew a session by extending its expiry time.
    #[instrument(skip(self), fields(session_id = %session_id, new_ttl = new_ttl_seconds))]
    pub async fn renew_session(
        &self,
        session_id: &SessionId,
        new_ttl_seconds: i64,
    ) -> Result<SessionData, ServiceError> {
        let row = self
            .store
            .get_session(session_id)
            .await?
            .ok_or(ServiceError::SessionNotFound)?;

        if row.state != "active" && row.state != "issued" {
            return Err(ServiceError::Validation(format!(
                "cannot renew session in state: {}",
                row.state
            )));
        }

        let mut updated = row;
        updated.expires_at = Utc::now() + Duration::seconds(new_ttl_seconds);
        self.store
            .update_session_expiry(session_id, updated.expires_at)
            .await?;

        // Emit audit event
        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&updated.project_id)
            .await?;
        self.audit_service
            .append_event(
                &updated.tenant_id,
                &updated.project_id,
                &updated.actor_id,
                "session.renewed",
                "session",
                updated.id.as_str(),
                serde_json::json!({
                    "session_id": updated.id.as_str(),
                    "new_ttl_seconds": new_ttl_seconds,
                    "new_expires_at": updated.expires_at.to_rfc3339(),
                }),
                previous_hash,
            )
            .await?;

        let result = updated.into();
        info!("session renewed");
        Ok(result)
    }

    /// Revoke a session, marking it as revoked.
    #[instrument(skip(self), fields(session_id = %session_id, reason = %reason))]
    pub async fn revoke_session(
        &self,
        session_id: &SessionId,
        reason: &str,
    ) -> Result<SessionData, ServiceError> {
        let row = self
            .store
            .get_session(session_id)
            .await?
            .ok_or(ServiceError::SessionNotFound)?;

        if row.state == "revoked" || row.state == "stopped" {
            return Err(ServiceError::Validation(format!(
                "session already in terminal state: {}",
                row.state
            )));
        }

        self.store
            .update_session_state(session_id, "revoked")
            .await?;

        // Emit audit event
        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&row.project_id)
            .await?;
        self.audit_service
            .append_event(
                &row.tenant_id,
                &row.project_id,
                &row.actor_id,
                "session.revoked",
                "session",
                row.id.as_str(),
                serde_json::json!({
                    "session_id": row.id.as_str(),
                    "reason": reason,
                }),
                previous_hash,
            )
            .await?;

        let mut updated = row;
        updated.state = "revoked".to_string();
        let result = updated.into();
        info!("session revoked");
        Ok(result)
    }

    /// Validate a session — check it exists, is active or issued, and not expired.
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn validate_session(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionData, ServiceError> {
        let row = self
            .store
            .get_session(session_id)
            .await?
            .ok_or(ServiceError::SessionNotFound)?;

        if row.state == "revoked" || row.state == "stopped" {
            return Err(ServiceError::Validation(format!(
                "session is in terminal state: {}",
                row.state
            )));
        }
        if Utc::now() > row.expires_at {
            return Err(ServiceError::Validation("session has expired".to_string()));
        }
        let result = row.into();
        info!("session validated");
        Ok(result)
    }

    /// Build an ActorScope from a session using `ActorScope::from_session`.
    pub fn build_scope(
        &self,
        session: &SessionData,
        roles: Vec<RoleId>,
        allowed_actions: Vec<String>,
    ) -> ActorScope {
        ActorScope::from_session(
            session.tenant_id.clone(),
            session.project_id.clone(),
            session.actor_id.clone(),
            session.id.clone(),
            &session.state,
            session.expires_at,
            roles,
            allowed_actions,
        )
    }

    /// Get session data by ID from the store.
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session_data(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<SessionData>, ServiceError> {
        let row = self.store.get_session(session_id).await?;
        Ok(row.map(|r| r.into()))
    }
}
