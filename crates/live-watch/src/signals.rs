use authority_domain::{
    SignalSeverity, SignalType, WatchProfile, WatchSignal, WatchSignalId,
};
use chrono::{Duration, Utc};
use control_store::CoreStore;
use std::collections::HashMap;
use tracing::instrument;

/// Collects health signals from runtime and bundle data for anomaly detection.
#[derive(Clone)]
pub struct SignalCollector {
    store: CoreStore,
}

impl SignalCollector {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Collect all relevant signals for an app by querying runtime health and bundle data.
    #[instrument(skip(self), fields(app_id = %app_id))]
    pub async fn collect_for_app(&self, app_id: &str) -> Result<Vec<WatchSignal>, String> {
        let mut signals = Vec::new();

        // 1. Check runtime health: query latest runtime instances for error rates, latency, etc.
        let runtimes = self
            .store
            .list_runtime_instances(50, 0)
            .await
            .map_err(|e| format!("failed to list runtimes: {e}"))?;

        for rt in &runtimes {
            // Check for missing heartbeat (no heartbeat in last 60 seconds)
            let since = Utc::now() - Duration::seconds(60);
            if rt.last_heartbeat < since {
                let signal = WatchSignal {
                    id: WatchSignalId::generate(),
                    app_id: app_id.to_string(),
                    signal_type: SignalType::MissingHeartbeat,
                    value: 0.0,
                    threshold: 1.0,
                    severity: SignalSeverity::Critical,
                    timestamp: Utc::now(),
                };
                self.store
                    .insert_watch_signal(&signal)
                    .await
                    .map_err(|e| format!("store error: {e}"))?;
                signals.push(signal);
            }

            // Error rate signal
            if rt.error_count > 0 {
                let error_rate = rt.error_count as f64 / (rt.action_count.max(1) as f64);
                let signal = WatchSignal {
                    id: WatchSignalId::generate(),
                    app_id: app_id.to_string(),
                    signal_type: SignalType::ErrorRate,
                    value: error_rate,
                    threshold: 0.1, // 10% error rate threshold
                    severity: if error_rate > 0.5 {
                        SignalSeverity::Critical
                    } else {
                        SignalSeverity::Warning
                    },
                    timestamp: Utc::now(),
                };
                self.store
                    .insert_watch_signal(&signal)
                    .await
                    .map_err(|e| format!("store error: {e}"))?;
                signals.push(signal);
            }
        }

        // 2. Check bundle version skew
        let bundles = self
            .store
            .list_bundles(app_id, 10, 0)
            .await
            .map_err(|e| format!("failed to list bundles: {e}"))?;

        if bundles.len() > 1 {
            let versions: Vec<&str> = bundles.iter().map(|b| b.version.as_str()).collect();
            if let Some(first) = versions.first() {
                let has_skew = versions.iter().any(|v| *v != *first);
                if has_skew {
                    let signal = WatchSignal {
                        id: WatchSignalId::generate(),
                        app_id: app_id.to_string(),
                        signal_type: SignalType::VersionSkew,
                        value: versions.len() as f64,
                        threshold: 1.0,
                        severity: SignalSeverity::Warning,
                        timestamp: Utc::now(),
                    };
                    self.store
                        .insert_watch_signal(&signal)
                        .await
                        .map_err(|e| format!("store error: {e}"))?;
                    signals.push(signal);
                }
            }
        }

        Ok(signals)
    }
}

/// Tracks signal value history for trend deviation detection.
#[derive(Clone, Debug)]
struct SignalHistory {
    /// Rolling window of recent values (newest last).
    values: Vec<f64>,
    /// Maximum number of values to retain.
    max_len: usize,
}

impl SignalHistory {
    fn new(max_len: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_len),
            max_len,
        }
    }

    fn push(&mut self, value: f64) {
        self.values.push(value);
        if self.values.len() > self.max_len {
            self.values.remove(0);
        }
    }

    /// Compute the simple moving average of the stored values.
    fn moving_average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Detects anomalies by evaluating signals against profile thresholds
/// and detecting trend deviations from historical signal values.
#[derive(Clone)]
pub struct AnomalyDetector {
    /// In-memory signal history per (app_id, signal_type) key.
    /// Key format: "{app_id}:{signal_type}"
    history: HashMap<String, SignalHistory>,
    /// Maximum history window size per signal.
    history_max_len: usize,
    /// Trend deviation factor: if latest value deviates more than this
    /// multiple of the moving average, flag as anomalous.
    trend_deviation_factor: f64,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {
            history: HashMap::new(),
            history_max_len: 10,
            trend_deviation_factor: 2.0,
        }
    }
}

impl AnomalyDetector {
    /// Create a new detector with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate a signal against its profile thresholds and historical trend.
    ///
    /// Returns `true` if the signal exceeds the warning or critical threshold,
    /// or if the signal shows anomalous trend deviation.
    ///
    /// Also records the signal value in the trend history for future deviation detection.
    #[instrument(skip(self, signal, profile))]
    pub fn evaluate(&mut self, signal: &WatchSignal, profile: &WatchProfile) -> bool {
        let key = format!("{}:{}", signal.app_id, signal.signal_type);

        // Find the matching threshold from the profile
        let matching_threshold = profile.signal_thresholds.iter().find(|t| {
            std::mem::discriminant(&t.signal_type)
                == std::mem::discriminant(&signal.signal_type)
        });

        // Threshold check — does the signal exceed profile thresholds?
        let exceeds_threshold = if let Some(t) = matching_threshold {
            signal.value > t.warning_threshold || signal.value > t.critical_threshold
        } else {
            // Fallback: compare against the signal's own threshold
            signal.value > signal.threshold
        };

        // Trend deviation check — does this signal value deviate significantly
        // from the recent historical average?
        let history = self
            .history
            .entry(key)
            .or_insert_with(|| SignalHistory::new(self.history_max_len));

        let is_trend_deviation = if history.len() >= 3 {
            history.moving_average().is_some_and(|avg| {
                if avg == 0.0 {
                    // Avoid division by zero; flag if value is non-zero when average was zero
                    signal.value > 0.1
                } else {
                    let deviation = (signal.value / avg) - 1.0;
                    deviation.abs() > self.trend_deviation_factor
                }
            })
        } else {
            false
        };

        // Record this value in history for future trend detection
        history.push(signal.value);

        exceeds_threshold || is_trend_deviation
    }

    /// Run batch evaluation for multiple signals, returning only the anomalous ones.
    pub fn evaluate_batch(
        &mut self,
        signals: &[WatchSignal],
        profile: &WatchProfile,
    ) -> Vec<WatchSignal> {
        signals
            .iter()
            .filter(|s| self.evaluate(s, profile))
            .cloned()
            .collect()
    }

    /// Reset the history for a given app and signal type.
    pub fn reset_history(&mut self, app_id: &str, signal_type: &SignalType) {
        let key = format!("{app_id}:{signal_type}");
        self.history.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{SignalThreshold, WatchProfile};

    fn make_signal(
        app_id: &str,
        signal_type: SignalType,
        value: f64,
        threshold: f64,
    ) -> WatchSignal {
        WatchSignal {
            id: WatchSignalId::generate(),
            app_id: app_id.to_string(),
            signal_type,
            value,
            threshold,
            severity: SignalSeverity::Warning,
            timestamp: Utc::now(),
        }
    }

    fn default_profile() -> WatchProfile {
        WatchProfile {
            app_id: "test-app".to_string(),
            signal_thresholds: vec![SignalThreshold {
                signal_type: SignalType::ErrorRate,
                warning_threshold: 0.1,
                critical_threshold: 0.5,
            }],
            max_proposals_per_day: 10,
            poll_interval_secs: 30,
        }
    }

    #[test]
    fn detects_threshold_breach() {
        let mut detector = AnomalyDetector::new();
        let profile = default_profile();
        let signal = make_signal("test-app", SignalType::ErrorRate, 0.3, 1.0);
        // 0.3 > warning threshold 0.1 → should be detected
        assert!(detector.evaluate(&signal, &profile));
    }

    #[test]
    fn passes_normal_signals() {
        let mut detector = AnomalyDetector::new();
        let profile = default_profile();
        let signal = make_signal("test-app", SignalType::ErrorRate, 0.05, 1.0);
        // 0.05 < warning threshold 0.1 → should pass through
        assert!(!detector.evaluate(&signal, &profile));
    }

    #[test]
    fn detects_trend_deviation() {
        let mut detector = AnomalyDetector::new();
        let profile = default_profile();
        let signal_type = SignalType::ErrorRate;

        // Push 3 low values to establish a baseline
        for value in &[0.05, 0.06, 0.04] {
            let signal = make_signal("test-app", signal_type.clone(), *value, 1.0);
            detector.evaluate(&signal, &profile);
        }

        // Now push a value that's 3x the running average (~0.05 * 2 > 0.1)
        let signal = make_signal("test-app", signal_type, 0.4, 1.0);
        assert!(
            detector.evaluate(&signal, &profile),
            "trend deviation should be detected"
        );
    }

    #[test]
    fn batch_evaluate_filters_anomalies() {
        let mut detector = AnomalyDetector::new();
        let profile = default_profile();

        let signals = vec![
            make_signal("test-app", SignalType::ErrorRate, 0.05, 1.0),
            make_signal("test-app", SignalType::ErrorRate, 0.3, 1.0),
            make_signal("test-app", SignalType::ErrorRate, 0.02, 1.0),
        ];

        let anomalies = detector.evaluate_batch(&signals, &profile);
        assert_eq!(anomalies.len(), 1, "only 0.3 should exceed warning threshold");
        assert!((anomalies[0].value - 0.3).abs() < f64::EPSILON);
    }
}
