use crate::LiveRuntime;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Runtime health data.
#[derive(Clone, Debug, Serialize)]
pub struct RuntimeHealth {
    pub runtime_id: String,
    pub bundle_id: String,
    pub status: String,
    pub uptime_seconds: i64,
    pub active_sessions: u64,
    pub last_action_at: Option<DateTime<Utc>>,
    pub healthy: bool,
}

/// Runtime performance metrics.
#[derive(Clone, Debug, Serialize)]
pub struct RuntimeMetrics {
    pub runtime_id: String,
    pub action_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub loaded_artifacts: Vec<String>,
}

/// Get the health status of a runtime.
pub fn runtime_health(runtime: &LiveRuntime) -> RuntimeHealth {
    let now = Utc::now();
    let uptime = now.signed_duration_since(runtime.started_at);
    let uptime_seconds = uptime.num_seconds();

    let healthy = match runtime.status {
        runtime_bundle::RuntimeStatus::Running => true,
        runtime_bundle::RuntimeStatus::Loading => true, // Still initializing
        _ => false,
    };

    RuntimeHealth {
        runtime_id: runtime.runtime_id.clone(),
        bundle_id: runtime.bundle_id.to_string(),
        status: runtime.status.to_string(),
        uptime_seconds,
        active_sessions: runtime.active_sessions,
        last_action_at: runtime.last_action_at,
        healthy,
    }
}

/// Get performance metrics for a runtime.
pub fn runtime_metrics(runtime: &LiveRuntime) -> RuntimeMetrics {
    let total = runtime.action_count + runtime.error_count;
    let success_rate = if total > 0 {
        runtime.action_count as f64 / total as f64 * 100.0
    } else {
        100.0
    };

    RuntimeMetrics {
        runtime_id: runtime.runtime_id.clone(),
        action_count: runtime.action_count,
        error_count: runtime.error_count,
        success_rate,
        loaded_artifacts: runtime.loaded_artifacts.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_bundle::{generate_key_pair, BundleBuilder, RuntimeStatus};
    use std::collections::HashMap;

    async fn make_test_runtime() -> LiveRuntime {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("1.0.0", "test-app", &kp.private_key_hex);
        builder.add_artifact("main.wasm", vec![1, 2, 3]);
        let package = builder.build().unwrap();
        let artifacts = vec![("main.wasm".to_string(), vec![1, 2, 3])];

        crate::loader::load_bundle(package, &kp.public_key_hex, artifacts, HashMap::new())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn reports_healthy_when_running() {
        let runtime = make_test_runtime().await;
        let health = runtime_health(&runtime);
        assert!(health.healthy);
        assert_eq!(health.status, "running");
        assert!(health.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn reports_unhealthy_when_stopped() {
        let mut runtime = make_test_runtime().await;
        runtime.status = RuntimeStatus::Stopped;
        let health = runtime_health(&runtime);
        assert!(!health.healthy);
        assert_eq!(health.status, "stopped");
    }

    #[tokio::test]
    async fn metrics_show_success_rate() {
        let mut runtime = make_test_runtime().await;
        runtime.action_count = 95;
        runtime.error_count = 5;

        let metrics = runtime_metrics(&runtime);
        assert_eq!(metrics.action_count, 95);
        assert_eq!(metrics.error_count, 5);
        assert!((metrics.success_rate - 95.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn metrics_default_100_percent() {
        let runtime = make_test_runtime().await;
        let metrics = runtime_metrics(&runtime);
        assert!((metrics.success_rate - 100.0).abs() < f64::EPSILON);
    }
}
