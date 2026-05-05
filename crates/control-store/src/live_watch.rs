use crate::StoreError;
use authority_domain::{
    ProposalId, ProposalStatus, RemediationProposal, WatchProfile, WatchSignal, WatchSignalId,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Insert a watch signal.
pub async fn insert_watch_signal(pool: &PgPool, signal: &WatchSignal) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO watch_signals (id, app_id, signal_type, value, threshold, severity, timestamp) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(signal.id.as_str())
    .bind(&signal.app_id)
    .bind(format!("{}", signal.signal_type))
    .bind(signal.value)
    .bind(signal.threshold)
    .bind(format!("{}", signal.severity))
    .bind(signal.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query watch signals with optional filters.
#[allow(clippy::too_many_arguments)]
pub async fn query_watch_signals(
    pool: &PgPool,
    app_id: &str,
    signal_type: Option<&str>,
    severity: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<WatchSignal>, StoreError> {
    let rows = sqlx::query_as::<_, WatchSignalRow>(
        "SELECT id, app_id, signal_type, value, threshold, severity, timestamp \
         FROM watch_signals WHERE app_id = $1 \
         AND ($2::text IS NULL OR signal_type = $2) \
         AND ($3::text IS NULL OR severity = $3) \
         ORDER BY timestamp DESC LIMIT $4 OFFSET $5",
    )
    .bind(app_id)
    .bind(signal_type)
    .bind(severity)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Insert a remediation proposal.
pub async fn insert_remediation_proposal(
    pool: &PgPool,
    proposal: &RemediationProposal,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO remediation_proposals \
         (id, app_id, triggering_signal_ids, proposed_actions, impact_analysis, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(proposal.id.as_str())
    .bind(&proposal.app_id)
    .bind(
        serde_json::to_value(&proposal.triggering_signal_ids)
            .map_err(StoreError::Serialization)?,
    )
    .bind(
        serde_json::to_value(&proposal.proposed_actions)
            .map_err(StoreError::Serialization)?,
    )
    .bind(&proposal.impact_analysis)
    .bind(format!("{}", proposal.status))
    .bind(proposal.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// List remediation proposals for an app, optionally filtered by status.
pub async fn list_remediation_proposals(
    pool: &PgPool,
    app_id: &str,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<RemediationProposal>, StoreError> {
    let rows = sqlx::query_as::<_, RemediationProposalRow>(
        "SELECT id, app_id, triggering_signal_ids, proposed_actions, impact_analysis, status, created_at \
         FROM remediation_proposals WHERE app_id = $1 \
         AND ($2::text IS NULL OR status = $2) \
         ORDER BY created_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(app_id)
    .bind(status_filter)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Update the status of a remediation proposal.
pub async fn update_proposal_status(
    pool: &PgPool,
    proposal_id: &ProposalId,
    status: &ProposalStatus,
) -> Result<(), StoreError> {
    let n = sqlx::query(
        "UPDATE remediation_proposals SET status = $1 WHERE id = $2",
    )
    .bind(format!("{}", status))
    .bind(proposal_id.as_str())
    .execute(pool)
    .await?
    .rows_affected();

    if n == 0 {
        return Err(StoreError::invalid_data(format!(
            "proposal {} not found",
            proposal_id
        )));
    }
    Ok(())
}

/// Get a watch profile for an app.
pub async fn get_watch_profile(
    pool: &PgPool,
    app_id: &str,
) -> Result<Option<WatchProfile>, StoreError> {
    let row = sqlx::query_as::<_, WatchProfileRow>(
        "SELECT app_id, signal_thresholds, max_proposals_per_day, poll_interval_secs \
         FROM watch_profiles WHERE app_id = $1",
    )
    .bind(app_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(r.try_into()?)),
        None => Ok(None),
    }
}

/// Upsert a watch profile (insert or update).
pub async fn upsert_watch_profile(pool: &PgPool, profile: &WatchProfile) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO watch_profiles (app_id, signal_thresholds, max_proposals_per_day, poll_interval_secs) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (app_id) DO UPDATE SET \
         signal_thresholds = EXCLUDED.signal_thresholds, \
         max_proposals_per_day = EXCLUDED.max_proposals_per_day, \
         poll_interval_secs = EXCLUDED.poll_interval_secs",
    )
    .bind(&profile.app_id)
    .bind(
        serde_json::to_value(&profile.signal_thresholds)
            .map_err(StoreError::Serialization)?,
    )
    .bind(profile.max_proposals_per_day as i64)
    .bind(profile.poll_interval_secs as i64)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Row types for sqlx query_as ──

#[derive(sqlx::FromRow)]
struct WatchSignalRow {
    id: String,
    app_id: String,
    signal_type: String,
    value: f64,
    threshold: f64,
    severity: String,
    timestamp: DateTime<Utc>,
}

impl TryInto<WatchSignal> for WatchSignalRow {
    type Error = StoreError;

    fn try_into(self) -> Result<WatchSignal, Self::Error> {
        let signal_type = parse_signal_type(&self.signal_type)?;
        let severity = parse_signal_severity(&self.severity)?;
        Ok(WatchSignal {
            id: WatchSignalId::new(self.id).map_err(StoreError::Id)?,
            app_id: self.app_id,
            signal_type,
            value: self.value,
            threshold: self.threshold,
            severity,
            timestamp: self.timestamp,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RemediationProposalRow {
    id: String,
    app_id: String,
    triggering_signal_ids: sqlx::types::Json<Vec<String>>,
    proposed_actions: sqlx::types::Json<Vec<authority_domain::ProposedAction>>,
    impact_analysis: String,
    status: String,
    created_at: DateTime<Utc>,
}

impl TryInto<RemediationProposal> for RemediationProposalRow {
    type Error = StoreError;

    fn try_into(self) -> Result<RemediationProposal, Self::Error> {
        let status = parse_proposal_status(&self.status)?;
        let signal_ids = self
            .triggering_signal_ids
            .0
            .into_iter()
            .map(|s| WatchSignalId::new(s).map_err(StoreError::Id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RemediationProposal {
            id: ProposalId::new(self.id).map_err(StoreError::Id)?,
            app_id: self.app_id,
            triggering_signal_ids: signal_ids,
            proposed_actions: self.proposed_actions.0,
            impact_analysis: self.impact_analysis,
            status,
            created_at: self.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct WatchProfileRow {
    app_id: String,
    signal_thresholds: sqlx::types::Json<Vec<authority_domain::SignalThreshold>>,
    max_proposals_per_day: i64,
    poll_interval_secs: i64,
}

impl TryInto<WatchProfile> for WatchProfileRow {
    type Error = StoreError;

    fn try_into(self) -> Result<WatchProfile, Self::Error> {
        Ok(WatchProfile {
            app_id: self.app_id,
            signal_thresholds: self.signal_thresholds.0,
            max_proposals_per_day: self.max_proposals_per_day as u32,
            poll_interval_secs: self.poll_interval_secs as u64,
        })
    }
}

// ── Parsing helpers ──

fn parse_signal_type(s: &str) -> Result<authority_domain::SignalType, StoreError> {
    match s {
        "latency" => Ok(authority_domain::SignalType::Latency),
        "error_rate" => Ok(authority_domain::SignalType::ErrorRate),
        "action_count" => Ok(authority_domain::SignalType::ActionCount),
        "version_skew" => Ok(authority_domain::SignalType::VersionSkew),
        "artifact_age" => Ok(authority_domain::SignalType::ArtifactAge),
        "cost_rate" => Ok(authority_domain::SignalType::CostRate),
        "token_usage" => Ok(authority_domain::SignalType::TokenUsage),
        "missing_heartbeat" => Ok(authority_domain::SignalType::MissingHeartbeat),
        other => Err(StoreError::invalid_data(format!(
            "unknown signal_type: {other}"
        ))),
    }
}

fn parse_signal_severity(s: &str) -> Result<authority_domain::SignalSeverity, StoreError> {
    match s {
        "info" => Ok(authority_domain::SignalSeverity::Info),
        "warning" => Ok(authority_domain::SignalSeverity::Warning),
        "critical" => Ok(authority_domain::SignalSeverity::Critical),
        other => Err(StoreError::invalid_data(format!(
            "unknown severity: {other}"
        ))),
    }
}

fn parse_proposal_status(s: &str) -> Result<ProposalStatus, StoreError> {
    match s {
        "proposed" => Ok(ProposalStatus::Proposed),
        "approved" => Ok(ProposalStatus::Approved),
        "rejected" => Ok(ProposalStatus::Rejected),
        "executed" => Ok(ProposalStatus::Executed),
        other => Err(StoreError::invalid_data(format!(
            "unknown proposal status: {other}"
        ))),
    }
}
