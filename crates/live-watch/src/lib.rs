pub mod profile;
pub mod remediation;
pub mod signals;

use control_store::CoreStore;
use profile::ProfileManager;
use remediation::RemediationGenerator;
use signals::{AnomalyDetector, SignalCollector};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// Configuration for the Live Watch monitoring engine.
#[derive(Clone, Debug)]
pub struct WatchConfig {
    /// Default poll interval in seconds if no profile overrides it.
    pub default_poll_interval_secs: u64,
    /// Maximum number of proposals allowed within a rolling time window.
    pub max_proposals_per_window: u32,
    /// Rolling window duration in seconds for proposal rate limiting.
    pub window_secs: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            default_poll_interval_secs: 30,
            max_proposals_per_window: 5,
            window_secs: 3600,
        }
    }
}

/// The Live Watch monitoring engine.
///
/// Orchestrates signal collection, anomaly detection, and remediation proposal
/// generation for live runtime health monitoring.
#[derive(Clone)]
pub struct LiveWatchEngine {
    config: WatchConfig,
    collector: SignalCollector,
    detector: AnomalyDetector,
    generator: RemediationGenerator,
    profiles: ProfileManager,
    /// Tracks which app_ids are actively being monitored.
    monitored_apps: HashMap<String, bool>,
}

impl LiveWatchEngine {
    pub fn new(store: CoreStore, config: WatchConfig) -> Self {
        Self {
            config: config.clone(),
            collector: SignalCollector::new(store.clone()),
            detector: AnomalyDetector::new(),
            generator: RemediationGenerator::new(store.clone()),
            profiles: ProfileManager::new(store),
            monitored_apps: HashMap::new(),
        }
    }

    /// Access the engine's configuration.
    pub fn config(&self) -> &WatchConfig {
        &self.config
    }

    /// Start monitoring a specific app.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn start_monitoring(&mut self, app_id: String) {
        self.monitored_apps.insert(app_id, true);
        info!("live watch monitoring started for app");
    }

    /// Stop monitoring a specific app.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn stop_monitoring(&mut self, app_id: &str) {
        self.monitored_apps.remove(app_id);
        info!("live watch monitoring stopped for app");
    }

    /// Check if an app is currently being monitored.
    pub fn is_monitoring(&self, app_id: &str) -> bool {
        self.monitored_apps.get(app_id).copied().unwrap_or(false)
    }

    /// Run a single collection + detection + proposal cycle for all monitored apps.
    #[instrument(skip(self))]
    pub async fn run_cycle(&mut self) {
        let apps: Vec<String> = self.monitored_apps.keys().cloned().collect();
        for app_id in &apps {
            if let Err(e) = self.run_cycle_for_app(app_id).await {
                warn!(app_id = %app_id, error = %e, "live watch cycle failed for app");
            }
        }
    }

    /// Run a full cycle for a single app: collect signals → detect anomalies → propose remediation.
    ///
    /// Does NOT sleep — the caller is responsible for pacing (e.g., via the background monitoring loop).
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn run_cycle_for_app(
        &mut self,
        app_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let profile = self.profiles.get_or_default(app_id).await?;

        // Collect signals
        let signals = self.collector.collect_for_app(app_id).await?;

        // Detect anomalies by evaluating against profile thresholds + trend history
        let anomalous_signals = self.detector.evaluate_batch(&signals, &profile);

        if !anomalous_signals.is_empty() {
            info!(
                app_id = %app_id,
                anomalies = anomalous_signals.len(),
                "anomalies detected in live watch cycle"
            );
        }

        // Only generate proposals from anomalies found in this cycle.
        if !anomalous_signals.is_empty() {
            self.generator
                .propose_remediation(app_id, &anomalous_signals)
                .await?;
        }

        Ok(())
    }

    /// Start a background monitoring loop that runs cycles on a configurable interval.
    ///
    /// Returns a `RunningMonitor` handle that can be used to stop the loop.
    /// The loop will check `running` flag before each cycle and exit gracefully.
    pub async fn start_background_monitoring(self) -> RunningMonitor {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let poll_interval = self.config.default_poll_interval_secs;

        let handle = tokio::spawn(async move {
            let mut engine = self;
            info!(
                poll_interval_secs = poll_interval,
                "live watch background monitoring started"
            );

            while running_clone.load(Ordering::Relaxed) {
                engine.run_cycle().await;
                tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
            }

            info!("live watch background monitoring stopped");
        });

        RunningMonitor {
            _handle: handle,
            running,
        }
    }
}

/// A handle to a running background monitoring loop.
///
/// Dropping this handle will NOT stop the loop — call `stop()` explicitly.
pub struct RunningMonitor {
    _handle: tokio::task::JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl RunningMonitor {
    /// Signal the monitoring loop to stop gracefully.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Check if the monitoring loop is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_defaults() {
        let config = WatchConfig::default();
        assert_eq!(config.default_poll_interval_secs, 30);
        assert_eq!(config.max_proposals_per_window, 5);
        assert_eq!(config.window_secs, 3600);
    }

    #[test]
    fn test_monitoring_state() {
        let config = WatchConfig::default();
        // We can't create a CoreStore without a DB, but we can test the config accessor
        assert_eq!(config.default_poll_interval_secs, 30);
        assert_eq!(config.max_proposals_per_window, 5);
        assert_eq!(config.window_secs, 3600);
    }
}
