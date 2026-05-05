use crate::{
    ActorId, ApprovalId, ApprovalState, DomainError, ExecutionMode, ProjectId, RoleId, SessionId,
    SkillId, SkillSessionId, TenantId,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActorScope {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub roles: Vec<RoleId>,
    pub session_id: SessionId,
    pub skill_session_id: Option<SkillSessionId>,
    pub requested_skill_id: Option<SkillId>,
    pub active_skill_id: Option<SkillId>,
    pub allowed_actions: Vec<String>,
    pub approval_id: Option<ApprovalId>,
    pub approval_state: ApprovalState,
    pub execution_mode: ExecutionMode,
    pub context_updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Default for ActorScope {
    fn default() -> Self {
        // INVARIANT: All four IDs use the non-empty string "default" which
        // passes the empty-string guard in `IdError::Empty`.
        Self {
            tenant_id: TenantId::new("default").unwrap(),
            project_id: ProjectId::new("default").unwrap(),
            actor_id: ActorId::new("default").unwrap(),
            roles: vec![],
            session_id: SessionId::new("default").unwrap(),
            skill_session_id: None,
            requested_skill_id: None,
            active_skill_id: None,
            allowed_actions: vec![],
            approval_id: None,
            approval_state: ApprovalState::NotRequired,
            execution_mode: ExecutionMode::ReadOnly,
            context_updated_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
        }
    }
}

impl ActorScope {
    /// Check if an action is allowed by this scope.
    /// Supports `"*"` as a wildcard meaning "allow all actions".
    pub fn allows_action(&self, action: &str) -> bool {
        self.allowed_actions
            .iter()
            .any(|item| item == "*" || item == action)
    }

    /// Check if the scope has expired relative to `now`.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now > self.expires_at
    }

    /// Check if the scope context is stale (age > max_context_age_seconds).
    pub fn is_stale(&self, now: DateTime<Utc>, max_age_seconds: i64) -> bool {
        now.signed_duration_since(self.context_updated_at)
            .num_seconds()
            > max_age_seconds
    }

    /// Build a read-only scope from an existing scope.
    pub fn as_readonly(&self) -> Self {
        let mut scope = self.clone();
        scope.execution_mode = ExecutionMode::ReadOnly;
        scope
    }

    /// Validate scope invariants.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.roles.is_empty() {
            return Err(DomainError::ScopeValidation(
                "scope must have at least one role".to_string(),
            ));
        }
        if self.allowed_actions.is_empty() {
            return Err(DomainError::ScopeValidation(
                "scope must have at least one allowed action".to_string(),
            ));
        }
        Ok(())
    }

    /// Return time-to-live as a Duration, or zero if already expired.
    pub fn ttl(&self, now: DateTime<Utc>) -> Duration {
        if now >= self.expires_at {
            Duration::zero()
        } else {
            self.expires_at.signed_duration_since(now)
        }
    }

    /// Create an ActorScope from session parameters.
    /// Maps session state to the appropriate ExecutionMode:
    ///   "issued"  → ReadOnly
    ///   "active"  → ReadWrite
    ///   "revoked" → NoAccess
    ///   "stopped" → NoAccess
    ///   others    → NoAccess (expired, unknown)
    #[allow(clippy::too_many_arguments)]
    pub fn from_session(
        tenant_id: TenantId,
        project_id: ProjectId,
        actor_id: ActorId,
        session_id: SessionId,
        state: &str,
        expires_at: DateTime<Utc>,
        roles: Vec<RoleId>,
        allowed_actions: Vec<String>,
    ) -> Self {
        let execution_mode = Self::execution_mode_from_state(state);
        Self {
            tenant_id,
            project_id,
            actor_id,
            roles,
            session_id,
            skill_session_id: None,
            requested_skill_id: None,
            active_skill_id: None,
            allowed_actions,
            approval_id: None,
            approval_state: ApprovalState::NotRequired,
            execution_mode,
            context_updated_at: Utc::now(),
            expires_at,
        }
    }

    /// Map a session state string to the appropriate ExecutionMode.
    ///
    /// - `"issued"` → ReadOnly (session exists but not yet active)
    /// - `"active"` → ReadWrite (session is active and can mutate)
    /// - `"revoked"`, `"stopped"`, or any unrecognized state → NoAccess
    pub fn execution_mode_from_state(state: &str) -> ExecutionMode {
        match state {
            "issued" => ExecutionMode::ReadOnly,
            "active" => ExecutionMode::ReadWrite,
            _ => ExecutionMode::NoAccess,
        }
    }
}
