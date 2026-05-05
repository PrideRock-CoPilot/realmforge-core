use authority_domain::{ProjectId, SnapshotId, TenantId};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{types::Json, PgPool};
use tracing::instrument;

use crate::StoreError;

/// Raw tuple returned by rollback preview queries.
pub type RollbackPreviewRow = (
    String,        // id
    String,        // tenant_id
    String,        // project_id
    String,        // from_snapshot_id
    String,        // to_snapshot_id
    String,        // status
    Value,         // preview_json
    DateTime<Utc>, // created_at
);

/// Insert a rollback preview record.
#[allow(clippy::too_many_arguments)]
#[instrument(skip(pool, preview_json), fields(preview_id = %id, tenant_id = %tenant_id, project_id = %project_id))]
pub async fn insert_rollback_preview(
    pool: &PgPool,
    id: &str,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    from_snapshot_id: &SnapshotId,
    to_snapshot_id: &SnapshotId,
    status: &str,
    preview_json: &Value,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO rollback_previews \
         (id, tenant_id, project_id, from_snapshot_id, to_snapshot_id, status, preview_json, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())",
    )
    .bind(id)
    .bind(tenant_id.as_str())
    .bind(project_id.as_str())
    .bind(from_snapshot_id.as_str())
    .bind(to_snapshot_id.as_str())
    .bind(status)
    .bind(Json(preview_json))
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a rollback preview by ID.
#[allow(clippy::type_complexity)]
#[instrument(skip(pool), fields(preview_id = %preview_id))]
pub async fn get_rollback_preview(
    pool: &PgPool,
    preview_id: &str,
) -> Result<
    Option<(
        String,
        String,
        String,
        String,
        String,
        String,
        Value,
        DateTime<Utc>,
    )>,
    StoreError,
> {
    #[derive(sqlx::FromRow)]
    struct RollbackRaw {
        id: String,
        tenant_id: String,
        project_id: String,
        from_snapshot_id: String,
        to_snapshot_id: String,
        status: String,
        preview_json: Json<Value>,
        created_at: DateTime<Utc>,
    }

    let raw = sqlx::query_as::<_, RollbackRaw>(
        "SELECT id, tenant_id, project_id, from_snapshot_id, to_snapshot_id, status, \
         preview_json, created_at FROM rollback_previews WHERE id = $1",
    )
    .bind(preview_id)
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(|r| {
        (
            r.id,
            r.tenant_id,
            r.project_id,
            r.from_snapshot_id,
            r.to_snapshot_id,
            r.status,
            r.preview_json.0,
            r.created_at,
        )
    }))
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a rollback preview record.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self))]
    pub async fn insert_rollback_preview(
        &self,
        id: &str,
        tenant_id: &authority_domain::TenantId,
        project_id: &authority_domain::ProjectId,
        from_snapshot_id: &authority_domain::SnapshotId,
        to_snapshot_id: &authority_domain::SnapshotId,
        status: &str,
        preview_json: &serde_json::Value,
    ) -> Result<(), StoreError> {
        insert_rollback_preview(
            &self.pool,
            id,
            tenant_id,
            project_id,
            from_snapshot_id,
            to_snapshot_id,
            status,
            preview_json,
        )
        .await
    }

    /// Get a rollback preview by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_rollback_preview(
        &self,
        preview_id: &str,
    ) -> Result<Option<crate::RollbackPreviewRow>, StoreError> {
        get_rollback_preview(&self.pool, preview_id).await
    }
}
