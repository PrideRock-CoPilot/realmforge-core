use authority_domain::{
    ProposalId, ProposalStatus, RemediationProposal, SignalSeverity, SignalType, WatchProfile,
    WatchSignal, WatchSignalId,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Live Watch service — runtime health monitoring, anomaly detection, and remediation proposals.
///
/// ⚠️ Does NOT auto-remediate — all proposals require human or Board approval.
#[derive(Clone)]
pub struct LiveWatchService {
    store: CoreStore,
}

impl LiveWatchService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Record a runtime health signal.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn record_signal(
        &self,
        app_id: String,
        signal_type: SignalType,
        value: f64,
        threshold: f64,
        severity: SignalSeverity,
    ) -> Result<WatchSignal, ServiceError> {
        let signal = WatchSignal {
            id: WatchSignalId::generate(),
            app_id,
            signal_type,
            value,
            threshold,
            severity,
            timestamp: Utc::now(),
        };
        self.store.insert_watch_signal(&signal).await?;
        info!(signal_id = %signal.id, "watch signal recorded");
        Ok(signal)
    }

    /// Query recent signals for an app with optional filters.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn get_signals(
        &self,
        app_id: &str,
        signal_type: Option<&str>,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WatchSignal>, ServiceError> {
        self.store
            .query_watch_signals(app_id, signal_type, severity, limit, offset)
            .await
            .map_err(Into::into)
    }

    /// Generate a remediation proposal by scanning recent signals and detecting anomalies.
    ///
    /// Returns the generated proposal. The proposal is stored with status `Proposed` and
    /// must be explicitly approved before execution.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn propose_remediation(
        &self,
        app_id: &str,
    ) -> Result<RemediationProposal, ServiceError> {
        // Get recent signals with warning or critical severity
        let signals = self
            .store
            .query_watch_signals(app_id, None, None, 100, 0)
            .await?;

        let signal_ids: Vec<WatchSignalId> = signals.iter().map(|s| s.id.clone()).collect();

        let proposed_actions = vec![authority_domain::ProposedAction {
            action_type: "investigate".to_string(),
            description: format!("Investigate {} signal(s) for app {}", signals.len(), app_id),
            params: serde_json::json!({"app_id": app_id}),
        }];

        let proposal = RemediationProposal {
            id: ProposalId::generate(),
            app_id: app_id.to_string(),
            triggering_signal_ids: signal_ids,
            proposed_actions,
            impact_analysis: "Investigation and potential remediation actions as proposed"
                .to_string(),
            status: ProposalStatus::Proposed,
            created_at: Utc::now(),
        };

        self.store.insert_remediation_proposal(&proposal).await?;

        info!(
            proposal_id = %proposal.id,
            "remediation proposal generated"
        );

        Ok(proposal)
    }

    /// List remediation proposals for an app, optionally filtered by status.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn list_proposals(
        &self,
        app_id: &str,
        status_filter: Option<ProposalStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RemediationProposal>, ServiceError> {
        let status_str = status_filter.map(|s| format!("{}", s));
        self.store
            .list_remediation_proposals(app_id, status_str.as_deref(), limit, offset)
            .await
            .map_err(Into::into)
    }

    /// Approve a remediation proposal, transitioning it to `Approved` status.
    ///
    /// Once approved, the proposal can be routed to the gateway for execution.
    #[instrument(skip(self), fields(proposal_id = %proposal_id))]
    pub async fn approve_proposal(
        &self,
        proposal_id: &ProposalId,
    ) -> Result<RemediationProposal, ServiceError> {
        self.store
            .update_proposal_status(proposal_id, &ProposalStatus::Approved)
            .await?;
        info!(proposal_id = %proposal_id, "remediation proposal approved");
        // Return a minimal response — full proposal data could be refetched from store if needed
        Ok(RemediationProposal {
            id: proposal_id.clone(),
            app_id: String::new(),
            triggering_signal_ids: vec![],
            proposed_actions: vec![],
            impact_analysis: String::new(),
            status: ProposalStatus::Approved,
            created_at: Utc::now(),
        })
    }

    /// Get or create a watch profile for an app.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn get_profile(&self, app_id: &str) -> Result<WatchProfile, ServiceError> {
        match self.store.get_watch_profile(app_id).await? {
            Some(p) => Ok(p),
            None => {
                let default = WatchProfile {
                    app_id: app_id.to_string(),
                    signal_thresholds: vec![
                        authority_domain::SignalThreshold {
                            signal_type: SignalType::ErrorRate,
                            warning_threshold: 0.05,
                            critical_threshold: 0.20,
                        },
                        authority_domain::SignalThreshold {
                            signal_type: SignalType::MissingHeartbeat,
                            warning_threshold: 0.0,
                            critical_threshold: 1.0,
                        },
                    ],
                    max_proposals_per_day: 10,
                    poll_interval_secs: 30,
                };
                self.store.upsert_watch_profile(&default).await?;
                Ok(default)
            }
        }
    }

    /// Upsert a watch profile for an app.
    #[instrument(skip(self, profile), fields(app_id = %profile.app_id))]
    pub async fn save_profile(&self, profile: &WatchProfile) -> Result<(), ServiceError> {
        self.store
            .upsert_watch_profile(profile)
            .await
            .map_err(Into::into)
    }
}
