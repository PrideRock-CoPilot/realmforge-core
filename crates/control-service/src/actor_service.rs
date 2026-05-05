use authority_domain::{ActorId, ActorScope, ApprovalState, ExecutionMode, RoleId, SessionId};
use control_store::CoreStore;

use tracing::{info, instrument};

use crate::error::ServiceError;

/// Actor scope management service.
#[derive(Clone)]
pub struct ActorService {
    #[allow(dead_code)]
    store: CoreStore,
}

impl ActorService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Build an ActorScope for a given actor and session.
    ///
    /// This method assembles the full authorization context:
    /// roles, allowed actions, execution mode, approval state.
    #[instrument(skip(self), fields(actor_id = %actor_id, session_id = %session_id))]
    pub async fn get_scope(
        &self,
        actor_id: &ActorId,
        session_id: &SessionId,
    ) -> Result<ActorScope, ServiceError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or(ServiceError::SessionNotFound)?;

        if &session.actor_id != actor_id {
            return Err(ServiceError::ActorNotFound);
        }
        if session.state == "revoked" || session.state == "stopped" {
            return Err(ServiceError::Validation(format!(
                "cannot build scope for session in terminal state: {}",
                session.state
            )));
        }
        if chrono::Utc::now() > session.expires_at {
            return Err(ServiceError::Validation("session has expired".to_string()));
        }

        Ok(ActorScope {
            tenant_id: session.tenant_id,
            project_id: session.project_id,
            actor_id: session.actor_id,
            roles: vec![RoleId::new("actor").map_err(|e| ServiceError::Validation(e.to_string()))?],
            session_id: session.id,
            skill_session_id: None,
            requested_skill_id: None,
            active_skill_id: None,
            allowed_actions: Vec::new(),
            approval_id: None,
            approval_state: ApprovalState::NotRequired,
            execution_mode: ExecutionMode::ReadOnly,
            context_updated_at: chrono::Utc::now(),
            expires_at: session.expires_at,
        })
    }

    /// Update a scope's context timestamp (refresh staleness).
    #[instrument(skip(self))]
    pub async fn update_scope(&self, mut scope: ActorScope) -> ActorScope {
        scope.context_updated_at = chrono::Utc::now();
        info!("actor scope updated");
        scope
    }

    /// Build a scope for a specific execution mode.
    pub fn scope_with_mode(&self, scope: &ActorScope, mode: ExecutionMode) -> ActorScope {
        let mut s = scope.clone();
        s.execution_mode = mode;
        s
    }

    /// Build a scope with specific roles.
    pub fn scope_with_roles(
        &self,
        scope: &ActorScope,
        roles: Vec<RoleId>,
        actions: Vec<String>,
    ) -> ActorScope {
        let mut s = scope.clone();
        s.roles = roles;
        s.allowed_actions = actions;
        s
    }
}
