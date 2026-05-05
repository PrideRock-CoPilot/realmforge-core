use crate::LiveRuntime;
use runtime_bundle::RuntimeStatus;
use tracing::instrument;

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

/// The runtime executor handles governed action execution.
/// Every execution goes through: state check → execution → metric recording.
/// Full policy engine and audit log integration will be added in a later phase
/// when the concrete types are available in the workspace.
pub struct RuntimeExecutor {
    runtime: LiveRuntime,
}

impl RuntimeExecutor {
    pub fn new(runtime: LiveRuntime) -> Self {
        Self { runtime }
    }

    /// Execute a governed action within the runtime context.
    /// Every execution goes through:
    ///   1. Runtime status check
    ///   2. Action execution within the bundle's scope
    ///   3. Metric recording
    #[instrument(skip(self, action_payload))]
    pub async fn execute_action(
        &mut self,
        action: &str,
        action_payload: &serde_json::Value,
        _actor_id: &str,
    ) -> Result<serde_json::Value, ExecutionError> {
        if self.runtime.status != RuntimeStatus::Running {
            return Err(ExecutionError::NotRunning(self.runtime.status.to_string()));
        }

        // 1. Execute the action (simulated — actual execution depends on the bundle type)
        let result = self.execute_internal(action, action_payload).await?;

        // Update runtime metrics
        self.runtime.action_count += 1;
        self.runtime.last_action_at = Some(chrono::Utc::now());

        Ok(result)
    }

    /// Execute a work packet in the runtime context.
    #[instrument(skip(self, _packet_payload))]
    pub async fn execute_packet(
        &mut self,
        packet_id: &str,
        _packet_payload: &serde_json::Value,
    ) -> Result<serde_json::Value, ExecutionError> {
        if self.runtime.status != RuntimeStatus::Running {
            return Err(ExecutionError::NotRunning(self.runtime.status.to_string()));
        }

        // Simulate packet execution
        let result = serde_json::json!({
            "packet_id": packet_id,
            "status": "executed",
            "timestamp": chrono::Utc::now(),
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

    /// Internal execution simulation.
    /// In production, this would dispatch to the bundle's WASM or native handler.
    async fn execute_internal(
        &self,
        action: &str,
        _payload: &serde_json::Value,
    ) -> Result<serde_json::Value, ExecutionError> {
        // Simulated execution — return a success response
        Ok(serde_json::json!({
            "action": action,
            "status": "completed",
            "bundle_id": self.runtime.bundle_id.to_string(),
            "timestamp": chrono::Utc::now(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[tokio::test]
    async fn executes_action_when_running() {
        let runtime = make_test_runtime().await;
        let mut executor = RuntimeExecutor::new(runtime);

        let result = executor
            .execute_action(
                "test_action",
                &serde_json::json!({"key": "value"}),
                "test-actor",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(executor.runtime().action_count, 1);
        assert!(executor.runtime().last_action_at.is_some());
    }

    #[tokio::test]
    async fn rejects_action_when_stopped() {
        let mut runtime = make_test_runtime().await;
        runtime.status = RuntimeStatus::Stopped;
        let mut executor = RuntimeExecutor::new(runtime);

        let result = executor
            .execute_action("test_action", &serde_json::json!({}), "test-actor")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::NotRunning(_)));
    }

    #[tokio::test]
    async fn executes_packet() {
        let runtime = make_test_runtime().await;
        let mut executor = RuntimeExecutor::new(runtime);

        let result = executor
            .execute_packet("packet-1", &serde_json::json!({"task": "build"}))
            .await;

        assert!(result.is_ok());
        assert_eq!(executor.runtime().action_count, 1);
    }
}
