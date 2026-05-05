use crate::LiveRuntime;
use audit_log::AuditEvent;
use authority_domain::{ActorScope, ApprovalState};
use policy_engine::{authorize_action, PolicyDecision};
use runtime_bundle::RuntimeStatus;
use serde_json::{json, Value};
use snapshot_ledger::SnapshotManifest;
use tracing::instrument;

const MAX_CONTEXT_AGE_SECONDS: i64 = 300;
const EXECUTE_PACKET_ACTION: &str = "runtime.execute_packet";

/// Errors that can occur during runtime execution.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("runtime not running (status: {0})")]
    NotRunning(String),
    #[error("action not allowed by governance: {0}")]
    GovernanceDenied(String),
    #[error("action failed: {0}")]
    ActionFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Governance evidence produced for a successful runtime execution.
#[derive(Clone, Debug)]
pub struct RuntimeExecutionRecord {
    pub action: String,
    pub policy_decision: PolicyDecision,
    pub audit_event: AuditEvent,
    pub snapshot_anchor: SnapshotManifest,
}

/// The runtime executor handles governed action execution.
/// Every execution goes through: state check -> policy check -> audit event
/// -> execution -> snapshot anchor -> metric recording.
pub struct RuntimeExecutor {
    runtime: LiveRuntime,
    execution_records: Vec<RuntimeExecutionRecord>,
}

impl RuntimeExecutor {
    pub fn new(runtime: LiveRuntime) -> Self {
        Self {
            runtime,
            execution_records: Vec::new(),
        }
    }

    /// Execute a governed action within the runtime context.
    /// Every execution goes through:
    ///   1. Runtime status check
    ///   2. Policy authorization for the actor scope
    ///   3. Audit event creation
    ///   4. Action execution within the bundle's scope
    ///   5. Snapshot anchor creation
    ///   6. Metric recording
    #[instrument(skip(self, action_payload))]
    pub async fn execute_action(
        &mut self,
        action: &str,
        action_payload: &Value,
        scope: &ActorScope,
    ) -> Result<Value, ExecutionError> {
        if self.runtime.status != RuntimeStatus::Running {
            return Err(ExecutionError::NotRunning(self.runtime.status.to_string()));
        }

        let policy_decision = self.authorize_scope(scope, action)?;
        let audit_event = self.create_audit_event(scope, action, action_payload)?;

        let mut result = self.execute_internal(action, action_payload).await?;
        let snapshot_anchor = self.create_snapshot_anchor(scope, action)?;
        Self::attach_governance_ids(&mut result, &audit_event, &snapshot_anchor);

        self.execution_records.push(RuntimeExecutionRecord {
            action: action.to_string(),
            policy_decision,
            audit_event,
            snapshot_anchor,
        });

        self.runtime.action_count += 1;
        self.runtime.last_action_at = Some(chrono::Utc::now());

        Ok(result)
    }

    /// Execute a work packet in the runtime context.
    #[instrument(skip(self, packet_payload))]
    pub async fn execute_packet(
        &mut self,
        packet_id: &str,
        packet_payload: &Value,
        scope: &ActorScope,
    ) -> Result<Value, ExecutionError> {
        if self.runtime.status != RuntimeStatus::Running {
            return Err(ExecutionError::NotRunning(self.runtime.status.to_string()));
        }

        let policy_decision = self.authorize_scope(scope, EXECUTE_PACKET_ACTION)?;
        let audit_payload = json!({
            "packet_id": packet_id,
            "payload": packet_payload,
        });
        let audit_event = self.create_audit_event(scope, EXECUTE_PACKET_ACTION, &audit_payload)?;

        let mut result = json!({
            "packet_id": packet_id,
            "status": "executed",
            "timestamp": chrono::Utc::now(),
        });
        let snapshot_anchor = self.create_snapshot_anchor(scope, EXECUTE_PACKET_ACTION)?;
        Self::attach_governance_ids(&mut result, &audit_event, &snapshot_anchor);

        self.execution_records.push(RuntimeExecutionRecord {
            action: EXECUTE_PACKET_ACTION.to_string(),
            policy_decision,
            audit_event,
            snapshot_anchor,
        });

        self.runtime.action_count += 1;
        self.runtime.last_action_at = Some(chrono::Utc::now());

        Ok(result)
    }

    /// Get a reference to the underlying runtime.
    pub fn runtime(&self) -> &LiveRuntime {
        &self.runtime
    }

    /// Get a mutable reference to the underlying runtime.
    pub fn runtime_mut(&mut self) -> &mut LiveRuntime {
        &mut self.runtime
    }

    /// Governance records produced by successful runtime executions.
    pub fn execution_records(&self) -> &[RuntimeExecutionRecord] {
        &self.execution_records
    }

    /// Internal execution simulation.
    /// In production, this would dispatch to the bundle's WASM or native handler.
    async fn execute_internal(
        &self,
        action: &str,
        _payload: &Value,
    ) -> Result<Value, ExecutionError> {
        Ok(json!({
            "action": action,
            "status": "completed",
            "bundle_id": self.runtime.bundle_id.to_string(),
            "timestamp": chrono::Utc::now(),
        }))
    }

    fn authorize_scope(
        &self,
        scope: &ActorScope,
        action: &str,
    ) -> Result<PolicyDecision, ExecutionError> {
        let approval_required = !matches!(scope.approval_state, ApprovalState::NotRequired);
        let decision = authorize_action(
            scope,
            action,
            true,
            approval_required,
            chrono::Utc::now(),
            MAX_CONTEXT_AGE_SECONDS,
        );

        if decision.allowed {
            Ok(decision)
        } else {
            Err(ExecutionError::GovernanceDenied(decision.to_string()))
        }
    }

    fn create_audit_event(
        &self,
        scope: &ActorScope,
        action: &str,
        payload: &Value,
    ) -> Result<AuditEvent, ExecutionError> {
        let previous_hash = self
            .execution_records
            .last()
            .map(|record| record.audit_event.event_hash.clone());

        AuditEvent::new(
            scope.tenant_id.clone(),
            scope.project_id.clone(),
            scope.actor_id.clone(),
            "runtime.action.executed",
            "runtime",
            self.runtime.runtime_id.clone(),
            json!({
                "action": action,
                "bundle_id": self.runtime.bundle_id,
                "runtime_id": self.runtime.runtime_id,
                "payload": payload,
            }),
            previous_hash,
        )
        .map_err(|e| ExecutionError::Internal(format!("failed to create audit event: {e}")))
    }

    fn create_snapshot_anchor(
        &self,
        scope: &ActorScope,
        action: &str,
    ) -> Result<SnapshotManifest, ExecutionError> {
        let parent_snapshot_id = self
            .execution_records
            .last()
            .map(|record| record.snapshot_anchor.id.clone());
        let previous_manifest_hash = self
            .execution_records
            .last()
            .map(|record| record.snapshot_anchor.manifest_hash.clone());

        SnapshotManifest::create(
            scope.tenant_id.clone(),
            scope.project_id.clone(),
            format!("runtime execution anchor: {action}"),
            parent_snapshot_id,
            vec![],
            vec![],
            previous_manifest_hash,
        )
        .map_err(|e| ExecutionError::Internal(format!("failed to create snapshot anchor: {e}")))
    }

    fn attach_governance_ids(
        result: &mut Value,
        audit_event: &AuditEvent,
        snapshot_anchor: &SnapshotManifest,
    ) {
        if let Some(object) = result.as_object_mut() {
            object.insert("audit_event_id".to_string(), json!(audit_event.id));
            object.insert("snapshot_id".to_string(), json!(snapshot_anchor.id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{
        ActorId, ApprovalState, ExecutionMode, ProjectId, RoleId, SessionId, SkillId,
        SkillSessionId, TenantId,
    };
    use chrono::{Duration, Utc};
    use runtime_bundle::{generate_key_pair, BundleBuilder};
    use std::collections::HashMap;

    async fn make_test_runtime() -> LiveRuntime {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("1.0.0", "test-app", &kp.private_key_hex);
        builder.add_artifact("action.wasm", vec![1, 2, 3]);
        let package = builder.build().unwrap();

        let artifacts = vec![("action.wasm".to_string(), vec![1, 2, 3])];

        crate::loader::load_bundle(package, &kp.public_key_hex, artifacts, HashMap::new())
            .await
            .unwrap()
    }

    fn approved_scope(actions: &[&str]) -> ActorScope {
        let now = Utc::now();
        ActorScope {
            tenant_id: TenantId::new("tenant").unwrap(),
            project_id: ProjectId::new("project").unwrap(),
            actor_id: ActorId::new("test-actor").unwrap(),
            roles: vec![RoleId::new("runtime-operator").unwrap()],
            session_id: SessionId::new("session").unwrap(),
            skill_session_id: Some(SkillSessionId::new("skill-session").unwrap()),
            requested_skill_id: Some(SkillId::new("runtime").unwrap()),
            active_skill_id: Some(SkillId::new("runtime").unwrap()),
            allowed_actions: actions.iter().map(|action| action.to_string()).collect(),
            approval_id: None,
            approval_state: ApprovalState::Approved,
            execution_mode: ExecutionMode::Approved,
            context_updated_at: now,
            expires_at: now + Duration::minutes(5),
        }
    }

    #[tokio::test]
    async fn executes_action_through_policy_audit_snapshot_chain() {
        let runtime = make_test_runtime().await;
        let mut executor = RuntimeExecutor::new(runtime);
        let scope = approved_scope(&["test_action"]);

        let result = executor
            .execute_action("test_action", &serde_json::json!({"key": "value"}), &scope)
            .await;

        let result = result.unwrap();
        assert_eq!(executor.runtime().action_count, 1);
        assert!(executor.runtime().last_action_at.is_some());

        let records = executor.execution_records();
        assert_eq!(records.len(), 1);
        assert!(records[0].policy_decision.allowed);
        assert_eq!(records[0].action, "test_action");
        assert_eq!(records[0].audit_event.event_type, "runtime.action.executed");
        assert_eq!(records[0].audit_event.actor_id, scope.actor_id);
        assert_eq!(
            records[0].snapshot_anchor.reason,
            "runtime execution anchor: test_action"
        );
        assert_eq!(
            result.get("audit_event_id").and_then(Value::as_str),
            Some(records[0].audit_event.id.as_str())
        );
        assert_eq!(
            result.get("snapshot_id").and_then(Value::as_str),
            Some(records[0].snapshot_anchor.id.as_str())
        );
    }

    #[tokio::test]
    async fn rejects_action_when_stopped() {
        let mut runtime = make_test_runtime().await;
        runtime.status = RuntimeStatus::Stopped;
        let mut executor = RuntimeExecutor::new(runtime);
        let scope = approved_scope(&["test_action"]);

        let result = executor
            .execute_action("test_action", &serde_json::json!({}), &scope)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::NotRunning(_)));
    }

    #[tokio::test]
    async fn denies_action_before_execution_when_policy_rejects() {
        let runtime = make_test_runtime().await;
        let mut executor = RuntimeExecutor::new(runtime);
        let scope = approved_scope(&["different_action"]);

        let result = executor
            .execute_action("test_action", &serde_json::json!({}), &scope)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecutionError::GovernanceDenied(_)
        ));
        assert_eq!(executor.runtime().action_count, 0);
        assert!(executor.execution_records().is_empty());
    }

    #[tokio::test]
    async fn executes_packet_through_policy_audit_snapshot_chain() {
        let runtime = make_test_runtime().await;
        let mut executor = RuntimeExecutor::new(runtime);
        let scope = approved_scope(&[EXECUTE_PACKET_ACTION]);

        let result = executor
            .execute_packet("packet-1", &serde_json::json!({"task": "build"}), &scope)
            .await;

        let result = result.unwrap();
        assert_eq!(executor.runtime().action_count, 1);
        assert_eq!(executor.execution_records().len(), 1);
        assert_eq!(
            executor.execution_records()[0].action,
            EXECUTE_PACKET_ACTION
        );
        assert_eq!(
            result.get("audit_event_id").and_then(Value::as_str),
            Some(executor.execution_records()[0].audit_event.id.as_str())
        );
    }
}
