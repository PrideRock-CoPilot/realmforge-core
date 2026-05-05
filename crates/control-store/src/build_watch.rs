use crate::StoreError;
use authority_domain::{CostRecord, ViolationId, ViolationRecord, WatchEvent, WatchEventId};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Insert a watch event.
pub async fn insert_watch_event(pool: &PgPool, event: &WatchEvent) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO watch_events (id, scope, event_type, severity, detail, evidence_ref, timestamp) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(event.id.as_str())
    .bind(&event.scope)
    .bind(format!("{}", event.event_type))
    .bind(format!("{}", event.severity))
    .bind(&event.detail)
    .bind(&event.evidence_ref)
    .bind(event.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query watch events with optional filters.
#[allow(clippy::too_many_arguments)]
pub async fn query_watch_events(
    pool: &PgPool,
    scope: Option<&str>,
    event_type: Option<&str>,
    severity: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<WatchEvent>, StoreError> {
    let rows: Vec<WatchEventRaw> = sqlx::query_as(
        "SELECT id, scope, event_type, severity, detail, evidence_ref, timestamp \
         FROM watch_events \
         WHERE ($1::text IS NULL OR scope = $1) \
           AND ($2::text IS NULL OR event_type = $2) \
           AND ($3::text IS NULL OR severity = $3) \
         ORDER BY timestamp DESC LIMIT $4 OFFSET $5",
    )
    .bind(scope)
    .bind(event_type)
    .bind(severity)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Get the total count of watch events (for dashboard).
pub async fn count_watch_events(pool: &PgPool) -> Result<i64, StoreError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM watch_events")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Insert a violation record.
pub async fn insert_violation(
    pool: &PgPool,
    violation: &ViolationRecord,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO violations (id, rule, severity, evidence_ref, detail, recorded_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(violation.id.as_str())
    .bind(&violation.rule)
    .bind(format!("{}", violation.severity))
    .bind(&violation.evidence_ref)
    .bind(&violation.detail)
    .bind(violation.recorded_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query violations with severity filter.
pub async fn query_violations(
    pool: &PgPool,
    severity: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ViolationRecord>, StoreError> {
    let rows: Vec<ViolationRaw> = if let Some(sev) = severity {
        sqlx::query_as(
            "SELECT id, rule, severity, evidence_ref, detail, recorded_at \
             FROM violations WHERE severity = $1 ORDER BY recorded_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(sev)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, rule, severity, evidence_ref, detail, recorded_at \
             FROM violations ORDER BY recorded_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };
    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Insert a cost record.
pub async fn insert_cost_record(pool: &PgPool, record: &CostRecord) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO cost_records (id, scope, token_cost, build_time_ms, storage_bytes, rework_count, recorded_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(record.id.as_str())
    .bind(&record.scope)
    .bind(record.token_cost as i64)
    .bind(record.build_time_ms as i64)
    .bind(record.storage_bytes as i64)
    .bind(record.rework_count as i32)
    .bind(record.recorded_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get aggregated cost summary by scope.
pub async fn get_cost_summary(pool: &PgPool) -> Result<Vec<CostSummaryRow>, StoreError> {
    let rows: Vec<CostSummaryRow> = sqlx::query_as(
        "SELECT scope, \
                SUM(token_cost)::BIGINT AS total_token_cost, \
                SUM(build_time_ms)::BIGINT AS total_build_time_ms, \
                SUM(storage_bytes)::BIGINT AS total_storage_bytes, \
                SUM(rework_count)::BIGINT AS total_rework_count, \
                COUNT(*) AS record_count \
         FROM cost_records \
         GROUP BY scope ORDER BY total_token_cost DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── Raw row types ──

#[derive(sqlx::FromRow)]
struct WatchEventRaw {
    id: String,
    scope: String,
    event_type: String,
    severity: String,
    detail: String,
    evidence_ref: Option<String>,
    timestamp: DateTime<Utc>,
}

impl TryInto<WatchEvent> for WatchEventRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<WatchEvent, Self::Error> {
        Ok(WatchEvent {
            id: WatchEventId::new(self.id).map_err(|e| StoreError::invalid_data(e.to_string()))?,
            scope: self.scope,
            event_type: parse_watch_event_type(&self.event_type)?,
            severity: parse_watch_severity(&self.severity)?,
            detail: self.detail,
            evidence_ref: self.evidence_ref,
            timestamp: self.timestamp,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ViolationRaw {
    id: String,
    rule: String,
    severity: String,
    evidence_ref: Option<String>,
    detail: String,
    recorded_at: DateTime<Utc>,
}

impl TryInto<ViolationRecord> for ViolationRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<ViolationRecord, Self::Error> {
        Ok(ViolationRecord {
            id: ViolationId::new(self.id).map_err(|e| StoreError::invalid_data(e.to_string()))?,
            rule: self.rule,
            severity: parse_watch_severity(&self.severity)?,
            evidence_ref: self.evidence_ref,
            detail: self.detail,
            recorded_at: self.recorded_at,
        })
    }
}

#[derive(Clone, Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct CostSummaryRow {
    pub scope: String,
    pub total_token_cost: Option<i64>,
    pub total_build_time_ms: Option<i64>,
    pub total_storage_bytes: Option<i64>,
    pub total_rework_count: Option<i64>,
    pub record_count: Option<i64>,
}

fn parse_watch_event_type(s: &str) -> Result<authority_domain::WatchEventType, StoreError> {
    match s {
        "file_mutation" => Ok(authority_domain::WatchEventType::FileMutation),
        "packet_submission" => Ok(authority_domain::WatchEventType::PacketSubmission),
        "policy_violation" => Ok(authority_domain::WatchEventType::PolicyViolation),
        "cost_anomaly" => Ok(authority_domain::WatchEventType::CostAnomaly),
        "build_failure" => Ok(authority_domain::WatchEventType::BuildFailure),
        "test_failure" => Ok(authority_domain::WatchEventType::TestFailure),
        "evidence_gap" => Ok(authority_domain::WatchEventType::EvidenceGap),
        _ => Err(StoreError::invalid_data(format!(
            "unknown watch event type: {s}"
        ))),
    }
}

fn parse_watch_severity(s: &str) -> Result<authority_domain::WatchSeverity, StoreError> {
    match s {
        "info" => Ok(authority_domain::WatchSeverity::Info),
        "warning" => Ok(authority_domain::WatchSeverity::Warning),
        "violation" => Ok(authority_domain::WatchSeverity::Violation),
        "critical" => Ok(authority_domain::WatchSeverity::Critical),
        _ => Err(StoreError::invalid_data(format!(
            "unknown watch severity: {s}"
        ))),
    }
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a watch event.
    #[tracing::instrument(skip(self))]
    pub async fn insert_watch_event(&self, event: &WatchEvent) -> Result<(), StoreError> {
        insert_watch_event(&self.pool, event).await
    }

    /// Query watch events with optional filters.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self))]
    pub async fn query_watch_events(
        &self,
        scope: Option<&str>,
        event_type: Option<&str>,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WatchEvent>, StoreError> {
        query_watch_events(&self.pool, scope, event_type, severity, limit, offset).await
    }

    /// Get the total count of watch events (for dashboard).
    #[tracing::instrument(skip(self))]
    pub async fn count_watch_events(&self) -> Result<i64, StoreError> {
        count_watch_events(&self.pool).await
    }

    /// Insert a violation record.
    #[tracing::instrument(skip(self))]
    pub async fn insert_violation(&self, violation: &ViolationRecord) -> Result<(), StoreError> {
        insert_violation(&self.pool, violation).await
    }

    /// Insert a cost record.
    #[tracing::instrument(skip(self))]
    pub async fn insert_cost_record(&self, record: &CostRecord) -> Result<(), StoreError> {
        insert_cost_record(&self.pool, record).await
    }

    /// Get aggregated cost summary by scope.
    #[tracing::instrument(skip(self))]
    pub async fn get_cost_summary(
        &self,
    ) -> Result<Vec<crate::build_watch::CostSummaryRow>, StoreError> {
        get_cost_summary(&self.pool).await
    }
}
