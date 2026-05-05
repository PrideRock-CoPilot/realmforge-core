use authority_domain::{SignalThreshold, WatchProfile};
use control_store::CoreStore;
use tracing::instrument;

/// Manages watch profiles — per-app threshold and notification configuration.
#[derive(Clone)]
pub struct ProfileManager {
    store: CoreStore,
}

impl ProfileManager {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Get the profile for an app, or return a default profile.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn get_or_default(&self, app_id: &str) -> Result<WatchProfile, String> {
        match self.store.get_watch_profile(app_id).await {
            Ok(Some(profile)) => Ok(profile),
            Ok(None) => Ok(default_profile_for(app_id)),
            Err(e) => Err(format!("failed to get watch profile: {e}")),
        }
    }

    /// Upsert a watch profile for an app.
    #[instrument(skip(self, profile), fields(app_id = %profile.app_id))]
    pub async fn save(&self, profile: &WatchProfile) -> Result<(), String> {
        self.store
            .upsert_watch_profile(profile)
            .await
            .map_err(|e| format!("failed to save watch profile: {e}"))
    }
}

/// Create a default watch profile with sensible thresholds.
fn default_profile_for(app_id: &str) -> WatchProfile {
    WatchProfile {
        app_id: app_id.to_string(),
        signal_thresholds: vec![
            SignalThreshold {
                signal_type: authority_domain::SignalType::Latency,
                warning_threshold: 200.0,   // ms
                critical_threshold: 1000.0, // ms
            },
            SignalThreshold {
                signal_type: authority_domain::SignalType::ErrorRate,
                warning_threshold: 0.05,  // 5%
                critical_threshold: 0.20, // 20%
            },
            SignalThreshold {
                signal_type: authority_domain::SignalType::MissingHeartbeat,
                warning_threshold: 0.0,
                critical_threshold: 1.0, // binary — any missed heartbeat is critical
            },
            SignalThreshold {
                signal_type: authority_domain::SignalType::CostRate,
                warning_threshold: 100.0, // cost units per hour
                critical_threshold: 500.0,
            },
        ],
        max_proposals_per_day: 10,
        poll_interval_secs: 30,
    }
}
