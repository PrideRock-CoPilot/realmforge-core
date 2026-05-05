use crate::{ActorId, CommandId, CommandStatus, DomainError, ProjectId, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoundedCommand {
    pub id: CommandId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub payload: Value,
    pub status: CommandStatus,
    pub created_at: DateTime<Utc>,
}

impl BoundedCommand {
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        actor_id: ActorId,
        action: impl Into<String>,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            id: CommandId::generate(),
            tenant_id,
            project_id,
            actor_id,
            action: action.into(),
            target_type: target_type.into(),
            target_id: target_id.into(),
            payload,
            status: CommandStatus::Proposed,
            created_at: Utc::now(),
        }
    }

    /// Transition the command to `Authorized`.
    pub fn authorize(&mut self) -> Result<(), DomainError> {
        self.transition_to(CommandStatus::Authorized)
    }

    /// Transition the command to `Denied`.
    pub fn deny(&mut self) -> Result<(), DomainError> {
        self.transition_to(CommandStatus::Denied)
    }

    /// Transition the command to `Applied`.
    pub fn apply(&mut self) -> Result<(), DomainError> {
        self.transition_to(CommandStatus::Applied)
    }

    /// Transition the command to `Failed`.
    pub fn fail(&mut self) -> Result<(), DomainError> {
        self.transition_to(CommandStatus::Failed)
    }

    /// Valid state transitions for the command lifecycle.
    fn transition_to(&mut self, new_status: CommandStatus) -> Result<(), DomainError> {
        let from = &self.status;
        let allowed = matches!(
            (from, &new_status),
            (CommandStatus::Proposed, CommandStatus::Authorized)
                | (CommandStatus::Proposed, CommandStatus::Denied)
                | (CommandStatus::Authorized, CommandStatus::Applied)
                | (CommandStatus::Authorized, CommandStatus::Failed)
                | (CommandStatus::Authorized, CommandStatus::Denied)
        );
        if !allowed {
            return Err(DomainError::InvalidCommandTransition {
                command_id: self.id.clone(),
                from: from.clone(),
                to: new_status,
            });
        }
        self.status = new_status;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_cmd() -> BoundedCommand {
        BoundedCommand::new(
            TenantId::new("tenant").unwrap(),
            ProjectId::new("project").unwrap(),
            ActorId::new("actor").unwrap(),
            "core.create_snapshot",
            "snapshot",
            "obj-1",
            json!({"key": "value"}),
        )
    }

    #[test]
    fn starts_as_proposed() {
        let cmd = make_cmd();
        assert_eq!(cmd.status, CommandStatus::Proposed);
    }

    #[test]
    fn propose_to_authorize_to_apply() {
        let mut cmd = make_cmd();
        cmd.authorize().unwrap();
        assert_eq!(cmd.status, CommandStatus::Authorized);
        cmd.apply().unwrap();
        assert_eq!(cmd.status, CommandStatus::Applied);
    }

    #[test]
    fn propose_to_denied() {
        let mut cmd = make_cmd();
        cmd.deny().unwrap();
        assert_eq!(cmd.status, CommandStatus::Denied);
    }

    #[test]
    fn cannot_apply_without_authorize() {
        let mut cmd = make_cmd();
        assert!(matches!(
            cmd.apply(),
            Err(DomainError::InvalidCommandTransition { .. })
        ));
    }

    #[test]
    fn cannot_authorize_twice() {
        let mut cmd = make_cmd();
        cmd.authorize().unwrap();
        assert!(matches!(
            cmd.authorize(),
            Err(DomainError::InvalidCommandTransition { .. })
        ));
    }

    #[test]
    fn authorize_then_fail() {
        let mut cmd = make_cmd();
        cmd.authorize().unwrap();
        cmd.fail().unwrap();
        assert_eq!(cmd.status, CommandStatus::Failed);
    }

    #[test]
    fn authorize_then_deny() {
        let mut cmd = make_cmd();
        cmd.authorize().unwrap();
        cmd.deny().unwrap();
        assert_eq!(cmd.status, CommandStatus::Denied);
    }
}
