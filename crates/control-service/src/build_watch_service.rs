use authority_domain::{
    CostRecord, CostRecordId, ViolationId, ViolationRecord, WatchEvent, WatchEventId,
    WatchEventType, WatchSeverity,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Build Watch service — construction-time monitoring, violations, evidence, and cost tracking.
#[derive(Clone)]
pub struct BuildWatchService {
    store: CoreStore,
}

impl BuildWatchService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Record a build-time watch event.
    #[instrument(skip(self), fields(scope = %scope))]
    pub async fn record_event(
        &self,
        scope: String,
        event_type: WatchEventType,
        severity: WatchSeverity,
        detail: String,
        evidence_ref: Option<String>,
    ) -> Result<WatchEvent, ServiceError> {
        let event = WatchEvent {
            id: WatchEventId::generate(),
            scope,
            event_type,
            severity,
            detail,
            evidence_ref,
            timestamp: Utc::now(),
        };
        self.store.insert_watch_event(&event).await?;
        info!(event_id = %event.id, "watch event recorded");
        Ok(event)
    }

    /// Auto-detect a violation from an event and record it.
    #[instrument(skip(self))]
    pub async fn detect_violation(
        &self,
        rule: String,
        severity: WatchSeverity,
        detail: String,
        evidence_ref: Option<String>,
    ) -> Result<ViolationRecord, ServiceError> {
        let violation = ViolationRecord {
            id: ViolationId::generate(),
            rule,
            severity,
            evidence_ref,
            detail,
            recorded_at: Utc::now(),
        };
        self.store.insert_violation(&violation).await?;
        info!(violation_id = %violation.id, rule = %violation.rule, "violation recorded");
        Ok(violation)
    }

    /// Query watch events with optional filters.
    #[instrument(skip(self))]
    pub async fn query_watch_events(
        &self,
        scope: Option<String>,
        event_type: Option<String>,
        severity: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WatchEvent>, ServiceError> {
        Ok(self
            .store
            .query_watch_events(
                scope.as_deref(),
                event_type.as_deref(),
                severity.as_deref(),
                limit,
                offset,
            )
            .await?)
    }

    /// Summarize cost across all scopes.
    #[instrument(skip(self))]
    pub async fn get_cost_summary(
        &self,
        scope: String,
        token_cost: u64,
        build_time_ms: u64,
        storage_bytes: u64,
        rework_count: u32,
    ) -> Result<CostRecord, ServiceError> {
        let record = CostRecord {
            id: CostRecordId::generate(),
            scope,
            token_cost,
            build_time_ms,
            storage_bytes,
            rework_count,
            recorded_at: Utc::now(),
        };
        self.store.insert_cost_record(&record).await?;
        info!(record_id = %record.id, "cost record inserted");
        Ok(record)
    }

    /// Get the aggregated cost summary.
    #[instrument(skip(self))]
    pub async fn aggregate_cost_summary(
        &self,
    ) -> Result<Vec<control_store::build_watch::CostSummaryRow>, ServiceError> {
        Ok(self.store.get_cost_summary().await?)
    }

    /// Get the full watch dashboard — event count + violation count + cost summary.
    #[instrument(skip(self))]
    pub async fn get_watch_dashboard(&self) -> Result<WatchDashboard, ServiceError> {
        let event_count = self.store.count_watch_events().await?;
        let cost_summary = self.store.get_cost_summary().await?;

        Ok(WatchDashboard {
            total_events: event_count,
            cost_by_scope: cost_summary,
        })
    }
}

/// Dashboard summary of watch state.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WatchDashboard {
    pub total_events: i64,
    pub cost_by_scope: Vec<control_store::build_watch::CostSummaryRow>,
}
