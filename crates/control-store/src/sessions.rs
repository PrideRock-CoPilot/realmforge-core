use authority_domain::{ActorId, ProjectId, SessionId, TenantId};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;

use crate::StoreError;

/// Row representation of a session in the database.
#[derive(Clone, Debug)]
pub struct SessionRow {
    pub id: SessionId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub state: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct SessionRaw {
    id: String,
    tenant_id: String,
    project_id: String,
    actor_id: String,
    state: String,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl TryFrom<SessionRaw> for SessionRow {
    type Error = StoreError;

    fn try_from(raw: SessionRaw) -> Result<Self, Self::Error> {
        Ok(SessionRow {
            id: SessionId::new(raw.id).map_err(|e| StoreError::InvalidData(e.to_string()))?,
            tenant_id: TenantId::new(raw.tenant_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            project_id: ProjectId::new(raw.project_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            actor_id: ActorId::new(raw.actor_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            state: raw.state,
            issued_at: raw.issued_at,
            expires_at: raw.expires_at,
        })
    }
}

/// Insert a new session record.
#[instrument(skip(pool), fields(session_id = %row.id, tenant_id = %row.tenant_id, actor_id = %row.actor_id))]
pub async fn insert_session(pool: &PgPool, row: &SessionRow) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO sessions (id, tenant_id, project_id, actor_id, state, issued_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(row.id.as_str())
    .bind(row.tenant_id.as_str())
    .bind(row.project_id.as_str())
    .bind(row.actor_id.as_str())
    .bind(&row.state)
    .bind(row.issued_at)
    .bind(row.expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a session by ID.
#[instrument(skip(pool), fields(session_id = %session_id))]
pub async fn get_session(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<SessionRow>, StoreError> {
    let raw = sqlx::query_as::<_, SessionRaw>(
        "SELECT id, tenant_id, project_id, actor_id, state, issued_at, expires_at \
         FROM sessions WHERE id = $1",
    )
    .bind(session_id.as_str())
    .fetch_optional(pool)
    .await?;

    raw.map(SessionRow::try_from).transpose()
}

/// List sessions for a tenant/project with pagination.
#[instrument(skip(pool), fields(tenant_id = %tenant_id, project_id = %project_id))]
pub async fn list_sessions(
    pool: &PgPool,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    limit: i64,
    offset: i64,
) -> Result<Vec<SessionRow>, StoreError> {
    let rows = sqlx::query_as::<_, SessionRaw>(
        "SELECT id, tenant_id, project_id, actor_id, state, issued_at, expires_at \
         FROM sessions \
         WHERE tenant_id = $1 AND project_id = $2 \
         ORDER BY issued_at DESC \
         LIMIT $3 OFFSET $4",
    )
    .bind(tenant_id.as_str())
    .bind(project_id.as_str())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(SessionRow::try_from).collect()
}

/// Update a session's state.
#[instrument(skip(pool), fields(session_id = %session_id, new_state = %new_state))]
pub async fn update_session_state(
    pool: &PgPool,
    session_id: &SessionId,
    new_state: &str,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE sessions SET state = $1 WHERE id = $2")
        .bind(new_state)
        .bind(session_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

/// Update a session's expiry timestamp.
#[instrument(skip(pool), fields(session_id = %session_id))]
pub async fn update_session_expiry(
    pool: &PgPool,
    session_id: &SessionId,
    expires_at: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE sessions SET expires_at = $1 WHERE id = $2")
        .bind(expires_at)
        .bind(session_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a new session record.
    #[tracing::instrument(skip(self))]
    pub async fn insert_session(&self, row: &SessionRow) -> Result<(), StoreError> {
        insert_session(&self.pool, row).await
    }

    /// Get a session by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<SessionRow>, StoreError> {
        get_session(&self.pool, session_id).await
    }

    /// Update a session's state.
    #[tracing::instrument(skip(self))]
    pub async fn update_session_state(
        &self,
        session_id: &SessionId,
        new_state: &str,
    ) -> Result<(), StoreError> {
        update_session_state(&self.pool, session_id, new_state).await
    }

    /// Update a session's expiry timestamp.
    #[tracing::instrument(skip(self))]
    pub async fn update_session_expiry(
        &self,
        session_id: &SessionId,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), StoreError> {
        update_session_expiry(&self.pool, session_id, expires_at).await
    }

    /// List sessions for a tenant/project with pagination.
    #[tracing::instrument(skip(self))]
    pub async fn list_sessions(
        &self,
        tenant_id: &authority_domain::TenantId,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SessionRow>, StoreError> {
        list_sessions(&self.pool, tenant_id, project_id, limit, offset).await
    }
}
