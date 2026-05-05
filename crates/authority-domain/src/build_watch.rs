use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for a watch event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WatchSeverity {
    Info,
    Warning,
    Violation,
    Critical,
}

impl fmt::Display for WatchSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatchSeverity::Info => write!(f, "info"),
            WatchSeverity::Warning => write!(f, "warning"),
            WatchSeverity::Violation => write!(f, "violation"),
            WatchSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Types of watch events monitored during construction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WatchEventType {
    FileMutation,
    PacketSubmission,
    PolicyViolation,
    CostAnomaly,
    BuildFailure,
    TestFailure,
    EvidenceGap,
}

impl fmt::Display for WatchEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatchEventType::FileMutation => write!(f, "file_mutation"),
            WatchEventType::PacketSubmission => write!(f, "packet_submission"),
            WatchEventType::PolicyViolation => write!(f, "policy_violation"),
            WatchEventType::CostAnomaly => write!(f, "cost_anomaly"),
            WatchEventType::BuildFailure => write!(f, "build_failure"),
            WatchEventType::TestFailure => write!(f, "test_failure"),
            WatchEventType::EvidenceGap => write!(f, "evidence_gap"),
        }
    }
}

/// A watch event recording construction-time behavior.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchEvent {
    pub id: WatchEventId,
    pub scope: String,
    pub event_type: WatchEventType,
    pub severity: WatchSeverity,
    pub detail: String,
    pub evidence_ref: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A violation record for a rule violation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub id: ViolationId,
    pub rule: String,
    pub severity: WatchSeverity,
    pub evidence_ref: Option<String>,
    pub detail: String,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

/// A cost record tracking resource usage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostRecord {
    pub id: CostRecordId,
    pub scope: String,
    pub token_cost: u64,
    pub build_time_ms: u64,
    pub storage_bytes: u64,
    pub rework_count: u32,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

use crate::{CostRecordId, ViolationId, WatchEventId};
