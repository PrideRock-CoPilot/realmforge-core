use crate::{DomainError, ProjectId, SkillId, SkillSessionId, SkillSessionState, SkillVersionId, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillIntegrityState {
    Pending,
    Valid,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillRegistration {
    pub id: SkillId,
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub version_id: SkillVersionId,
    pub name: String,
    pub allowed_actions: Vec<String>,
    pub integrity_state: SkillIntegrityState,
    pub approved: bool,
}

impl SkillRegistration {
    /// Check that the skill is in a usable state.
    pub fn is_usable(&self) -> Result<(), DomainError> {
        if !self.approved {
            return Err(DomainError::SkillIntegrityViolation {
                skill_id: self.id.clone(),
                reason: "skill is not approved".to_string(),
            });
        }
        if self.integrity_state != SkillIntegrityState::Valid {
            return Err(DomainError::SkillIntegrityViolation {
                skill_id: self.id.clone(),
                reason: format!("skill integrity is {:?}", self.integrity_state),
            });
        }
        Ok(())
    }

    /// Mark the integrity state.
    pub fn set_integrity(&mut self, state: SkillIntegrityState) {
        self.integrity_state = state;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillSession {
    pub id: SkillSessionId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub skill_id: SkillId,
    pub state: SkillSessionState,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SkillSession {
    pub fn is_active(&self) -> bool {
        self.state == SkillSessionState::Active
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }

    /// Transition the session state.
    pub fn activate(&mut self) {
        self.state = SkillSessionState::Active;
    }

    pub fn expire(&mut self) {
        self.state = SkillSessionState::Expired;
    }

    pub fn revoke(&mut self) {
        self.state = SkillSessionState::Revoked;
    }

    pub fn stop(&mut self) {
        self.state = SkillSessionState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usable_skill_must_be_approved_and_valid() {
        let skill = SkillRegistration {
            id: SkillId::new("skill.test").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            project_id: None,
            version_id: SkillVersionId::new("v1").unwrap(),
            name: "Test".to_string(),
            allowed_actions: vec!["test.action".to_string()],
            integrity_state: SkillIntegrityState::Valid,
            approved: true,
        };
        assert!(skill.is_usable().is_ok());
    }

    #[test]
    fn unapproved_skill_is_not_usable() {
        let skill = SkillRegistration {
            id: SkillId::new("skill.test").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            project_id: None,
            version_id: SkillVersionId::new("v1").unwrap(),
            name: "Test".to_string(),
            allowed_actions: vec![],
            integrity_state: SkillIntegrityState::Valid,
            approved: false,
        };
        assert!(skill.is_usable().is_err());
    }

    #[test]
    fn skill_session_state_transitions() {
        let mut session = SkillSession {
            id: SkillSessionId::new("ss-1").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            project_id: ProjectId::new("proj").unwrap(),
            skill_id: SkillId::new("skill").unwrap(),
            state: SkillSessionState::Issued,
            issued_at: Utc::now(),
            expires_at: Utc::now(),
        };
        assert!(!session.is_active());
        session.activate();
        assert!(session.is_active());
        session.revoke();
        assert!(!session.is_active());
    }
}
