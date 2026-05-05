use audit_log::{verify_event_chain, AuditEvent, ChainAnchor};
use authority_domain::{ActorId, AuditEventId, ProjectId, TenantId};
use chrono::{DateTime, Utc};
use control_store::CoreStore;
use serde_json::Value;

use tracing::{info, instrument};

use crate::error::ServiceError;

/// Audit trail service.
///
/// Manages append-only audit events with hash-chain integrity,
/// query, and chain verification.
#[derive(Clone)]
pub struct AuditService {
    store: CoreStore,
}

impl AuditService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Append a new audit event, computing its hash and linking to
    /// the previous event in the chain.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, payload), fields(event_type = %event_type, entity_type = %entity_type, entity_id = %entity_id, project_id = %project_id))]
    pub async fn append_event(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        actor_id: &ActorId,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        payload: Value,
        previous_hash: Option<String>,
    ) -> Result<AuditEvent, ServiceError> {
        let event = AuditEvent::new(
            tenant_id.clone(),
            project_id.clone(),
            actor_id.clone(),
            event_type,
            entity_type,
            entity_id,
            payload,
            previous_hash,
        )?;

        self.store.append_audit_event(&event).await?;

        info!(event_id = %event.id, event_hash = %event.event_hash, "audit event appended");
        Ok(event)
    }

    /// Append an event, automatically finding the latest event for chain linking.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, payload), fields(event_type = %event_type, entity_type = %entity_type, entity_id = %entity_id))]
    pub async fn append_chained_event(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        actor_id: &ActorId,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        payload: Value,
    ) -> Result<AuditEvent, ServiceError> {
        let previous_hash = self.get_latest_event_hash(project_id).await?;
        self.append_event(
            tenant_id,
            project_id,
            actor_id,
            event_type,
            entity_type,
            entity_id,
            payload,
            previous_hash,
        )
        .await
    }

    /// Get the hash of the latest event for a project (for chain linking).
    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn get_latest_event_hash(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<String>, ServiceError> {
        Ok(self.store.get_latest_event_hash(project_id).await?)
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(project_id = %project_id))]
    /// Query events by project with optional filters.
    pub async fn query_events(
        &self,
        project_id: &ProjectId,
        event_type: Option<&str>,
        actor_id: Option<&ActorId>,
        entity_type: Option<&str>,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AuditEvent>, u64), ServiceError> {
        let (events, total) = self
            .store
            .query_audit_events(
                project_id,
                event_type,
                actor_id,
                entity_type,
                from_time,
                to_time,
                limit as i64,
                offset as i64,
            )
            .await?;
        Ok((events, total))
    }

    /// Verify the integrity of the entire hash chain for a project.
    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn verify_chain(&self, project_id: &ProjectId) -> Result<ChainAnchor, ServiceError> {
        let events = self.store.get_all_audit_events(project_id).await?;
        let chain_integrity = verify_event_chain(&events)?;

        let (first_opt, last_opt, count) = self.store.get_chain_bounds(project_id).await?;

        Ok(ChainAnchor {
            first_event_id: first_opt
                .map(|(id, _)| id)
                .unwrap_or_else(AuditEventId::generate),
            last_event_id: last_opt
                .map(|(id, _)| id)
                .unwrap_or_else(AuditEventId::generate),
            event_count: count,
            chain_integrity,
        })
    }

    /// Get chain anchors (first, last, count, integrity).
    #[instrument(skip(self), fields(project_id = %project_id))]
    pub async fn get_chain_anchors(
        &self,
        project_id: &ProjectId,
    ) -> Result<ChainAnchor, ServiceError> {
        let (first_opt, last_opt, count) = self.store.get_chain_bounds(project_id).await?;

        // Verify chain integrity from all events
        let events = self.store.get_all_audit_events(project_id).await?;
        let chain_integrity = verify_event_chain(&events)?;

        Ok(ChainAnchor {
            first_event_id: first_opt
                .map(|(id, _)| id)
                .unwrap_or_else(AuditEventId::generate),
            last_event_id: last_opt
                .map(|(id, _)| id)
                .unwrap_or_else(AuditEventId::generate),
            event_count: count,
            chain_integrity,
        })
    }
}
