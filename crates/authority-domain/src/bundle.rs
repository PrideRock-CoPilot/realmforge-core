use crate::{ApprovalId, BundleId, RuntimeId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a bundle in its lifecycle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleStatus {
    Building,
    Signed,
    Verified,
    Deployed,
    Running,
    Failed(String),
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleStatus::Building => write!(f, "building"),
            BundleStatus::Signed => write!(f, "signed"),
            BundleStatus::Verified => write!(f, "verified"),
            BundleStatus::Deployed => write!(f, "deployed"),
            BundleStatus::Running => write!(f, "running"),
            BundleStatus::Failed(msg) => write!(f, "failed: {}", msg),
        }
    }
}

/// A signed bundle manifest in the authority domain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: BundleId,
    pub version: String,
    pub app_id: String,
    pub artifact_hashes: Vec<(String, String)>,
    pub governance_signature: String,
    pub release_approval_ref: Option<ApprovalId>,
    pub built_at: DateTime<Utc>,
    pub status: BundleStatus,
}

/// A runtime instance record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeInstance {
    pub runtime_id: RuntimeId,
    pub bundle_id: BundleId,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub active_sessions: i64,
    pub action_count: i64,
    pub error_count: i64,
    pub metadata: serde_json::Value,
}
