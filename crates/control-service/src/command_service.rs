use authority_domain::{ActorScope, BoundedCommand, CommandId};
use chrono::Utc;
use control_store::{commands::CommandRow, CoreStore};
use policy_engine::{authorize_command_action, PolicyDecision};
use serde_json::Value;

use tracing::{info, instrument};

use crate::error::ServiceError;
use crate::AuditService;

/// Command lifecycle service.
///
/// Orchestrates: propose → authorize → apply → deny lifecycle,
/// with policy checks and audit event emission at every step.
#[derive(Clone)]
pub struct CommandService {
    store: CoreStore,
    audit_service: AuditService,
}

impl CommandService {
    pub fn new(store: CoreStore, audit_service: AuditService) -> Self {
        Self {
            store,
            audit_service,
        }
    }

    /// Propose a new bounded command.
    #[instrument(skip(self, scope, payload), fields(action = %action, target_type = %target_type, target_id = %target_id))]
    pub async fn propose_command(
        &self,
        scope: &ActorScope,
        action: &str,
        target_type: &str,
        target_id: &str,
        payload: Value,
    ) -> Result<BoundedCommand, ServiceError> {
        let command = BoundedCommand::new(
            scope.tenant_id.clone(),
            scope.project_id.clone(),
            scope.actor_id.clone(),
            action,
            target_type,
            target_id,
            payload,
        );

        // Persist to store
        let row = command_row_from_bounded(&command);
        self.store.insert_command(&row).await?;

        // Emit audit event for proposal
        self.audit_service
            .append_event(
                &scope.tenant_id,
                &scope.project_id,
                &scope.actor_id,
                "command.proposed",
                "command",
                command.id.as_str(),
                serde_json::json!({
                    "action": command.action,
                    "target_type": command.target_type,
                    "target_id": command.target_id,
                    "command_id": command.id.as_str(),
                }),
                None,
            )
            .await?;

        info!(command_id = %command.id, "command proposed");
        Ok(command)
    }

    /// Authorize a proposed command.
    #[instrument(skip(self, scope), fields(command_id = %command_id))]
    pub async fn authorize_command(
        &self,
        command_id: &CommandId,
        scope: &ActorScope,
    ) -> Result<BoundedCommand, ServiceError> {
        let row = self
            .store
            .get_command(command_id)
            .await?
            .ok_or(ServiceError::CommandNotFound)?;

        let mut command = bounded_from_row(row)?;

        // Get the latest audit event hash for chaining
        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&scope.project_id)
            .await?;

        let decision = authorize_command_action(
            scope,
            &command.action,
            &command.status,
            true,
            false,
            Utc::now(),
            300,
        );

        if !decision.allowed {
            command.deny()?;
            self.store
                .update_command_status(command_id, "denied")
                .await?;

            self.audit_service
                .append_event(
                    &scope.tenant_id,
                    &scope.project_id,
                    &scope.actor_id,
                    "command.denied",
                    "command",
                    command.id.as_str(),
                    serde_json::json!({
                        "command_id": command.id.as_str(),
                        "denial_code": format!("{:?}", decision.denial.as_ref().map(|d| &d.code)),
                        "denial_message": decision.denial.as_ref().map(|d| &d.message),
                    }),
                    previous_hash,
                )
                .await?;
            return Err(ServiceError::PolicyDenied(decision));
        }

        command.authorize()?;
        self.store
            .update_command_status(command_id, "authorized")
            .await?;

        self.audit_service
            .append_event(
                &scope.tenant_id,
                &scope.project_id,
                &scope.actor_id,
                "command.authorized",
                "command",
                command.id.as_str(),
                serde_json::json!({
                    "command_id": command.id.as_str(),
                    "action": command.action,
                }),
                previous_hash,
            )
            .await?;

        info!(command_id = %command_id, status = ?command.status, "command authorized");
        Ok(command)
    }

    /// Apply an authorized command.
    #[instrument(skip(self, scope), fields(command_id = %command_id))]
    pub async fn apply_command(
        &self,
        command_id: &CommandId,
        scope: &ActorScope,
    ) -> Result<BoundedCommand, ServiceError> {
        let row = self
            .store
            .get_command(command_id)
            .await?
            .ok_or(ServiceError::CommandNotFound)?;

        let mut command = bounded_from_row(row)?;

        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&scope.project_id)
            .await?;

        command.apply()?;
        self.store
            .update_command_status(command_id, "applied")
            .await?;

        self.audit_service
            .append_event(
                &scope.tenant_id,
                &scope.project_id,
                &scope.actor_id,
                "command.applied",
                "command",
                command.id.as_str(),
                serde_json::json!({
                    "command_id": command.id.as_str(),
                    "action": command.action,
                    "target_type": command.target_type,
                    "target_id": command.target_id,
                }),
                previous_hash,
            )
            .await?;

        info!(command_id = %command_id, "command applied");
        Ok(command)
    }

    /// Deny a command manually (admin override).
    #[instrument(skip(self, scope, denial), fields(command_id = %command_id))]
    pub async fn deny_command(
        &self,
        command_id: &CommandId,
        scope: &ActorScope,
        denial: &PolicyDecision,
    ) -> Result<BoundedCommand, ServiceError> {
        let mut command = self
            .get_command(command_id)
            .await?
            .ok_or(ServiceError::CommandNotFound)?;

        let previous_hash = self
            .audit_service
            .get_latest_event_hash(&scope.project_id)
            .await?;

        command.deny()?;
        self.store
            .update_command_status(command_id, "denied")
            .await?;

        self.audit_service
            .append_event(
                &scope.tenant_id,
                &scope.project_id,
                &scope.actor_id,
                "command.denied",
                "command",
                command.id.as_str(),
                serde_json::json!({
                    "command_id": command.id.as_str(),
                    "denial_code": format!("{:?}", denial.denial.as_ref().map(|d| &d.code)),
                    "denial_message": denial.denial.as_ref().map(|d| &d.message),
                }),
                previous_hash,
            )
            .await?;

        info!(command_id = %command_id, "command denied");
        Ok(command)
    }

    /// Get a command by ID from the store.
    #[instrument(skip(self), fields(command_id = %command_id))]
    pub async fn get_command(
        &self,
        command_id: &CommandId,
    ) -> Result<Option<BoundedCommand>, ServiceError> {
        let row = self.store.get_command(command_id).await?;
        match row {
            Some(r) => Ok(Some(bounded_from_row(r)?)),
            None => Ok(None),
        }
    }
}

/// Convert a BoundedCommand to a CommandRow for DB persistence.
fn command_row_from_bounded(cmd: &BoundedCommand) -> CommandRow {
    CommandRow {
        id: cmd.id.clone(),
        tenant_id: cmd.tenant_id.clone(),
        project_id: cmd.project_id.clone(),
        actor_id: cmd.actor_id.clone(),
        action: cmd.action.clone(),
        target_type: cmd.target_type.clone(),
        target_id: cmd.target_id.clone(),
        payload: cmd.payload.clone(),
        status: cmd.status.clone(),
        created_at: cmd.created_at,
    }
}

/// Reconstruct a BoundedCommand from a CommandRow.
fn bounded_from_row(row: CommandRow) -> Result<BoundedCommand, ServiceError> {
    Ok(BoundedCommand {
        id: row.id,
        tenant_id: row.tenant_id,
        project_id: row.project_id,
        actor_id: row.actor_id,
        action: row.action,
        target_type: row.target_type,
        target_id: row.target_id,
        payload: row.payload,
        status: row.status,
        created_at: row.created_at,
    })
}
