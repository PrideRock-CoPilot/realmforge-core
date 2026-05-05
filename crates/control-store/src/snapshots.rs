use authority_domain::{ProjectId, SnapshotId};
use snapshot_ledger::SnapshotManifest;
use sqlx::{types::Json, PgPool};
use tracing::instrument;

use crate::StoreError;

#[derive(sqlx::FromRow)]
struct ManifestRaw {
    manifest_json: Json<SnapshotManifest>,
}

/// Insert a snapshot manifest, storing the full document as JSONB.
#[instrument(skip(pool, manifest), fields(snapshot_id = %manifest.id, project_id = %manifest.project_id))]
pub async fn insert_snapshot_manifest(
    pool: &PgPool,
    manifest: &SnapshotManifest,
) -> Result<(), StoreError> {
    let parent_id = manifest.parent_snapshot_id.as_ref().map(|id| id.as_str());
    let prev_hash = manifest.previous_manifest_hash.as_deref();

    sqlx::query(
        "INSERT INTO snapshot_manifests \
         (id, tenant_id, project_id, status, reason, parent_snapshot_id, \
          manifest_json, previous_manifest_hash, manifest_hash, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(manifest.id.as_str())
    .bind(manifest.tenant_id.as_str())
    .bind(manifest.project_id.as_str())
    .bind(manifest.status.as_db_str())
    .bind(&manifest.reason)
    .bind(parent_id)
    .bind(Json(manifest))
    .bind(prev_hash)
    .bind(&manifest.manifest_hash)
    .bind(manifest.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a snapshot manifest by ID.
#[instrument(skip(pool), fields(snapshot_id = %snapshot_id))]
pub async fn get_snapshot_manifest(
    pool: &PgPool,
    snapshot_id: &SnapshotId,
) -> Result<Option<SnapshotManifest>, StoreError> {
    let row = sqlx::query_as::<_, ManifestRaw>(
        "SELECT manifest_json FROM snapshot_manifests WHERE id = $1",
    )
    .bind(snapshot_id.as_str())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.manifest_json.0))
}

/// List snapshot manifests for a project with pagination.
#[instrument(skip(pool), fields(project_id = %project_id, limit = limit, offset = offset))]
pub async fn list_snapshot_manifests(
    pool: &PgPool,
    project_id: &ProjectId,
    limit: i64,
    offset: i64,
) -> Result<Vec<SnapshotManifest>, StoreError> {
    let rows = sqlx::query_as::<_, ManifestRaw>(
        "SELECT manifest_json FROM snapshot_manifests \
         WHERE project_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(project_id.as_str())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.manifest_json.0).collect())
}
