use audit_log::AuditEvent;
use authority_domain::{ActorId, AuditEventId, ProjectId};
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, QueryBuilder};
use tracing::instrument;

use crate::StoreError;

#[derive(sqlx::FromRow)]
struct AuditEventRaw {
    id: String,
    tenant_id: String,
    project_id: String,
    actor_id: String,
    event_type: String,
    entity_type: String,
    entity_id: String,
    payload: Json<serde_json::Value>,
    occurred_at: DateTime<Utc>,
    previous_hash: Option<String>,
    event_hash: String,
}

impl TryFrom<AuditEventRaw> for AuditEvent {
    type Error = StoreError;

    fn try_from(raw: AuditEventRaw) -> Result<Self, Self::Error> {
        Ok(AuditEvent {
            id: AuditEventId::new(raw.id).map_err(|e| StoreError::InvalidData(e.to_string()))?,
            tenant_id: authority_domain::TenantId::new(raw.tenant_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            project_id: ProjectId::new(raw.project_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            actor_id: ActorId::new(raw.actor_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            event_type: raw.event_type,
            entity_type: raw.entity_type,
            entity_id: raw.entity_id,
            payload: raw.payload.0,
            occurred_at: raw.occurred_at,
            previous_hash: raw.previous_hash,
            event_hash: raw.event_hash,
        })
    }
}

/// Append an audit event.
#[instrument(skip(pool, event), fields(event_id = %event.id, project_id = %event.project_id))]
pub async fn append_audit_event(pool: &PgPool, event: &AuditEvent) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO core_audit_events \
         (id, tenant_id, project_id, actor_id, event_type, entity_type, entity_id, \
          payload, occurred_at, previous_hash, event_hash) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(event.id.as_str())
    .bind(event.tenant_id.as_str())
    .bind(event.project_id.as_str())
    .bind(event.actor_id.as_str())
    .bind(&event.event_type)
    .bind(&event.entity_type)
    .bind(&event.entity_id)
    .bind(Json(&event.payload))
    .bind(event.occurred_at)
    .bind(&event.previous_hash)
    .bind(&event.event_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query audit events with optional filters and pagination.
/// Returns (events, total_count).
#[allow(clippy::too_many_arguments)]
#[instrument(skip(pool), fields(project_id = %project_id))]
pub async fn query_audit_events(
    pool: &PgPool,
    project_id: &ProjectId,
    event_type: Option<&str>,
    actor_id: Option<&ActorId>,
    entity_type: Option<&str>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AuditEvent>, u64), StoreError> {
    let mut count_qb: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new("SELECT COUNT(*) FROM core_audit_events WHERE project_id = ");
    count_qb.push_bind(project_id.as_str());

    let mut data_qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "SELECT id, tenant_id, project_id, actor_id, event_type, entity_type, entity_id, \
         payload, occurred_at, previous_hash, event_hash \
         FROM core_audit_events WHERE project_id = ",
    );
    data_qb.push_bind(project_id.as_str());

    if let Some(et) = event_type {
        if !et.is_empty() {
            count_qb.push(" AND event_type = ").push_bind(et);
            data_qb.push(" AND event_type = ").push_bind(et);
        }
    }
    if let Some(aid) = actor_id {
        count_qb.push(" AND actor_id = ").push_bind(aid.as_str());
        data_qb.push(" AND actor_id = ").push_bind(aid.as_str());
    }
    if let Some(ent) = entity_type {
        if !ent.is_empty() {
            count_qb.push(" AND entity_type = ").push_bind(ent);
            data_qb.push(" AND entity_type = ").push_bind(ent);
        }
    }
    if let Some(ft) = from_time {
        count_qb.push(" AND occurred_at >= ").push_bind(ft);
        data_qb.push(" AND occurred_at >= ").push_bind(ft);
    }
    if let Some(tt) = to_time {
        count_qb.push(" AND occurred_at <= ").push_bind(tt);
        data_qb.push(" AND occurred_at <= ").push_bind(tt);
    }

    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    data_qb
        .push(" ORDER BY occurred_at DESC LIMIT ")
        .push_bind(limit);
    data_qb.push(" OFFSET ").push_bind(offset);

    let rows = data_qb
        .build_query_as::<AuditEventRaw>()
        .fetch_all(pool)
        .await?;

    let events = rows
        .into_iter()
        .map(AuditEvent::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((events, total as u64))
}

/// Get the latest event hash for a project (for chain linking).
#[instrument(skip(pool), fields(project_id = %project_id))]
pub async fn get_latest_event_hash(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Option<String>, StoreError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT event_hash FROM core_audit_events \
         WHERE project_id = $1 ORDER BY occurred_at DESC LIMIT 1",
    )
    .bind(project_id.as_str())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(h,)| h))
}

/// Get all events for a project ordered by occurrence (for chain verification).
#[instrument(skip(pool), fields(project_id = %project_id))]
pub async fn get_all_events(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Vec<AuditEvent>, StoreError> {
    let rows = sqlx::query_as::<_, AuditEventRaw>(
        "SELECT id, tenant_id, project_id, actor_id, event_type, entity_type, entity_id, \
         payload, occurred_at, previous_hash, event_hash \
         FROM core_audit_events WHERE project_id = $1 ORDER BY occurred_at ASC",
    )
    .bind(project_id.as_str())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(AuditEvent::try_from).collect()
}

/// Get the first and last event info for chain anchor computation.
#[allow(clippy::type_complexity)]
#[instrument(skip(pool), fields(project_id = %project_id))]
pub async fn get_chain_bounds(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<
    (
        Option<(AuditEventId, String)>,
        Option<(AuditEventId, String)>,
        u64,
    ),
    StoreError,
> {
    let first: Option<(String, String)> = sqlx::query_as(
        "SELECT id, event_hash FROM core_audit_events \
         WHERE project_id = $1 ORDER BY occurred_at ASC LIMIT 1",
    )
    .bind(project_id.as_str())
    .fetch_optional(pool)
    .await?;

    let last: Option<(String, String)> = sqlx::query_as(
        "SELECT id, event_hash FROM core_audit_events \
         WHERE project_id = $1 ORDER BY occurred_at DESC LIMIT 1",
    )
    .bind(project_id.as_str())
    .fetch_optional(pool)
    .await?;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM core_audit_events WHERE project_id = $1")
            .bind(project_id.as_str())
            .fetch_one(pool)
            .await?;

    let first_opt = first.and_then(|(id, hash)| AuditEventId::new(id).ok().map(|eid| (eid, hash)));
    let last_opt = last.and_then(|(id, hash)| AuditEventId::new(id).ok().map(|eid| (eid, hash)));

    Ok((first_opt, last_opt, count as u64))
}
