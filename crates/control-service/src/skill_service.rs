use authority_domain::{
    ProjectId, SkillId, SkillRegistration, SkillSession, SkillSessionState, SkillVersionId,
    TenantId,
};
use chrono::{Duration, Utc};
use control_store::CoreStore;

use tracing::{info, instrument};

use crate::error::ServiceError;

/// Skill management service.
#[derive(Clone)]
pub struct SkillService {
    #[allow(dead_code)]
    store: CoreStore,
}

impl SkillService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Register a new skill within a tenant/project.
    #[instrument(skip(self), fields(skill_id = %skill_id, tenant_id = %tenant_id, name = %name))]
    pub async fn register_skill(
        &self,
        skill_id: &SkillId,
        tenant_id: &TenantId,
        project_id: Option<&ProjectId>,
        name: &str,
        allowed_actions: Vec<String>,
    ) -> Result<SkillRegistration, ServiceError> {
        let registration = SkillRegistration {
            id: skill_id.clone(),
            tenant_id: tenant_id.clone(),
            project_id: project_id.cloned(),
            version_id: SkillVersionId::new("v1")
                .map_err(|e| ServiceError::Validation(e.to_string()))?,
            name: name.to_string(),
            allowed_actions,
            integrity_state: authority_domain::SkillIntegrityState::Pending,
            approved: false,
        };

        info!("skill registered");
        // Store would persist here
        Ok(registration)
    }

    /// Activate a skill session by binding a skill to a session.
    #[instrument(skip(self), fields(skill_id = %skill_id, ttl = ttl_seconds))]
    pub async fn activate_skill_session(
        &self,
        skill_id: &SkillId,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        ttl_seconds: i64,
    ) -> Result<SkillSession, ServiceError> {
        let session = SkillSession {
            id: authority_domain::SkillSessionId::generate(),
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            skill_id: skill_id.clone(),
            state: SkillSessionState::Issued,
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(ttl_seconds),
        };
        info!(skill_session_id = %session.id, "skill session activated");
        Ok(session)
    }

    /// Validate that a skill registration is in a usable state.
    #[instrument(skip(self))]
    pub fn validate_skill_integrity(&self, skill: &SkillRegistration) -> Result<(), ServiceError> {
        skill.is_usable().map_err(ServiceError::Domain)
    }

    /// Approve a skill registration.
    pub fn approve_skill(&self, skill: &mut SkillRegistration) {
        skill.approved = true;
    }

    /// Set integrity state for a skill.
    pub fn set_skill_integrity(
        &self,
        skill: &mut SkillRegistration,
        state: authority_domain::SkillIntegrityState,
    ) {
        skill.set_integrity(state);
    }
}
