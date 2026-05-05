use crate::{ProposalId, WatchSignalId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of runtime health signals that Live Watch monitors.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignalType {
    Latency,
    ErrorRate,
    ActionCount,
    VersionSkew,
    ArtifactAge,
    CostRate,
    TokenUsage,
    MissingHeartbeat,
}

impl fmt::Display for SignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalType::Latency => write!(f, "latency"),
            SignalType::ErrorRate => write!(f, "error_rate"),
            SignalType::ActionCount => write!(f, "action_count"),
            SignalType::VersionSkew => write!(f, "version_skew"),
            SignalType::ArtifactAge => write!(f, "artifact_age"),
            SignalType::CostRate => write!(f, "cost_rate"),
            SignalType::TokenUsage => write!(f, "token_usage"),
            SignalType::MissingHeartbeat => write!(f, "missing_heartbeat"),
        }
    }
}

/// Severity level for a watch signal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignalSeverity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for SignalSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalSeverity::Info => write!(f, "info"),
            SignalSeverity::Warning => write!(f, "warning"),
            SignalSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// A single runtime health signal collected by Live Watch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchSignal {
    pub id: WatchSignalId,
    pub app_id: String,
    pub signal_type: SignalType,
    pub value: f64,
    pub threshold: f64,
    pub severity: SignalSeverity,
    pub timestamp: DateTime<Utc>,
}

/// Status of a remediation proposal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Proposed,
    Approved,
    Rejected,
    Executed,
}

impl fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProposalStatus::Proposed => write!(f, "proposed"),
            ProposalStatus::Approved => write!(f, "approved"),
            ProposalStatus::Rejected => write!(f, "rejected"),
            ProposalStatus::Executed => write!(f, "executed"),
        }
    }
}

/// A single action proposed as part of a remediation proposal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposedAction {
    pub action_type: String,
    pub description: String,
    pub params: serde_json::Value,
}

/// A remediation proposal generated from detected anomalies.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemediationProposal {
    pub id: ProposalId,
    pub app_id: String,
    pub triggering_signal_ids: Vec<WatchSignalId>,
    pub proposed_actions: Vec<ProposedAction>,
    pub impact_analysis: String,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
}

/// A watch profile with per-app signal thresholds and notification config.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchProfile {
    pub app_id: String,
    pub signal_thresholds: Vec<SignalThreshold>,
    pub max_proposals_per_day: u32,
    pub poll_interval_secs: u64,
}

/// A threshold configuration for a specific signal type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalThreshold {
    pub signal_type: SignalType,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
}
