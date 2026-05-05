pub mod executor;
pub mod health;
pub mod loader;

pub use executor::{RuntimeExecutionRecord, RuntimeExecutor};
pub use health::{runtime_health, runtime_metrics, RuntimeHealth, RuntimeMetrics};
pub use loader::{load_bundle, unload_bundle, LoadError};

use chrono::{DateTime, Utc};
use runtime_bundle::{BundleId, BundleManifest, RuntimeStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A live runtime instance.
/// Holds the currently loaded bundle, governance context, and execution state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveRuntime {
    pub runtime_id: String,
    pub bundle_id: BundleId,
    pub manifest: BundleManifest,
    pub status: RuntimeStatus,
    pub started_at: DateTime<Utc>,
    pub last_action_at: Option<DateTime<Utc>>,
    pub active_sessions: u64,
    pub action_count: u64,
    pub error_count: u64,
    pub governance_context: HashMap<String, String>,
    pub loaded_artifacts: Vec<String>,
}

impl LiveRuntime {
    pub fn new(
        runtime_id: impl Into<String>,
        bundle_id: BundleId,
        manifest: BundleManifest,
        governance_context: HashMap<String, String>,
    ) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            bundle_id,
            manifest,
            status: RuntimeStatus::Loading,
            started_at: Utc::now(),
            last_action_at: None,
            active_sessions: 0,
            action_count: 0,
            error_count: 0,
            governance_context,
            loaded_artifacts: Vec::new(),
        }
    }
}
