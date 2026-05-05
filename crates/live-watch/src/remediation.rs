use authority_domain::{
    ProposalId, ProposalStatus, ProposedAction, RemediationProposal, WatchSignal, WatchSignalId,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

/// Generates remediation proposals from detected anomaly signals.
///
/// ⚠️ Does NOT auto-execute — all proposals require human or Board approval.
#[derive(Clone)]
pub struct RemediationGenerator {
    store: CoreStore,
}

impl RemediationGenerator {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Generate a remediation proposal from a set of anomaly signals.
    ///
    /// Returns the generated proposal without executing any actions.
    #[instrument(skip(self, signals), fields(app_id = %app_id, signal_count = signals.len()))]
    pub async fn propose_remediation(
        &self,
        app_id: &str,
        signals: &[WatchSignal],
    ) -> Result<RemediationProposal, String> {
        let signal_ids: Vec<WatchSignalId> = signals.iter().map(|s| s.id.clone()).collect();

        // Build proposed actions based on signal types
        let mut proposed_actions = Vec::new();
        let mut impact_lines = Vec::new();

        for signal in signals {
            match signal.signal_type {
                authority_domain::SignalType::ErrorRate => {
                    proposed_actions.push(ProposedAction {
                        action_type: "restart_runtime".to_string(),
                        description: format!(
                            "Restart runtime due to high error rate: {:.2} (threshold: {:.2})",
                            signal.value, signal.threshold
                        ),
                        params: serde_json::json!({}),
                    });
                    impact_lines.push(
                        "Restarting runtime will cause brief service interruption".to_string(),
                    );
                }
                authority_domain::SignalType::MissingHeartbeat => {
                    proposed_actions.push(ProposedAction {
                        action_type: "restart_runtime".to_string(),
                        description: "Runtime heartbeat missing — runtime may be unresponsive"
                            .to_string(),
                        params: serde_json::json!({}),
                    });
                    impact_lines.push(
                        "Restarting runtime will restore service but active sessions will drain"
                            .to_string(),
                    );
                }
                authority_domain::SignalType::VersionSkew => {
                    proposed_actions.push(ProposedAction {
                        action_type: "schedule_bundle_update".to_string(),
                        description: "Multiple bundle versions detected — schedule roll-forward to latest".to_string(),
                        params: serde_json::json!({}),
                    });
                    impact_lines
                        .push("Bundle update is non-disruptive if rolling deployment is used"
                            .to_string());
                }
                authority_domain::SignalType::Latency => {
                    proposed_actions.push(ProposedAction {
                        action_type: "scale_runtime".to_string(),
                        description: format!(
                            "High latency detected: {:.2} ms (threshold: {:.2} ms)",
                            signal.value, signal.threshold
                        ),
                        params: serde_json::json!({"scale_factor": 2}),
                    });
                    impact_lines.push("Scaling runtime will increase resource usage".to_string());
                }
                authority_domain::SignalType::CostRate | authority_domain::SignalType::TokenUsage => {
                    proposed_actions.push(ProposedAction {
                        action_type: "optimize_cost".to_string(),
                        description: format!(
                            "Cost anomaly: value={:.2}, threshold={:.2}",
                            signal.value, signal.threshold
                        ),
                        params: serde_json::json!({}),
                    });
                    impact_lines.push("Cost optimization may reduce throughput".to_string());
                }
                _ => {
                    // ActionCount, ArtifactAge — generic remediation
                    proposed_actions.push(ProposedAction {
                        action_type: "investigate".to_string(),
                        description: format!(
                            "Investigate anomaly: {} value={:.2}",
                            signal.signal_type, signal.value
                        ),
                        params: serde_json::json!({}),
                    });
                    impact_lines.push("Investigation has no runtime impact".to_string());
                }
            }
        }

        // Deduplicate actions by type
        proposed_actions.dedup_by(|a, b| a.action_type == b.action_type);

        let impact_analysis = if impact_lines.is_empty() {
            "No direct impact expected from proposed actions".to_string()
        } else {
            impact_lines.join("; ")
        };

        let proposal = RemediationProposal {
            id: ProposalId::generate(),
            app_id: app_id.to_string(),
            triggering_signal_ids: signal_ids,
            proposed_actions,
            impact_analysis,
            status: ProposalStatus::Proposed,
            created_at: Utc::now(),
        };

        self.store
            .insert_remediation_proposal(&proposal)
            .await
            .map_err(|e| format!("store error: {e}"))?;

        info!(
            proposal_id = %proposal.id,
            app_id = %app_id,
            "remediation proposal generated (pending approval)"
        );

        Ok(proposal)
    }
}
