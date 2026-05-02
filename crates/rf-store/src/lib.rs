use rf_domain::{SnapshotId, SnapshotStatus};
use rf_events::AuditEvent;
use rf_snapshot::SnapshotManifest;
use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("serialization failed")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct CoreStore {
    pool: PgPool,
}

impl CoreStore {
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn append_audit_event(&self, event: &AuditEvent) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            INSERT INTO core_audit_events (
              id, tenant_id, project_id, actor_id, event_type, entity_type,
              entity_id, payload, occurred_at, previous_hash, event_hash
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(event.id.as_str())
        .bind(event.tenant_id.as_str())
        .bind(event.project_id.as_str())
        .bind(event.actor_id.as_str())
        .bind(&event.event_type)
        .bind(&event.entity_type)
        .bind(&event.entity_id)
        .bind(serde_json::to_value(&event.payload)?)
        .bind(event.occurred_at)
        .bind(&event.previous_hash)
        .bind(&event.event_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_snapshot_manifest(
        &self,
        manifest: &SnapshotManifest,
    ) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO snapshot_manifests (
              id, tenant_id, project_id, status, reason, parent_snapshot_id,
              manifest_json, previous_manifest_hash, manifest_hash
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(manifest.id.as_str())
        .bind(manifest.tenant_id.as_str())
        .bind(manifest.project_id.as_str())
        .bind(snapshot_status_value(&manifest.status))
        .bind(&manifest.reason)
        .bind(manifest.parent_snapshot_id.as_ref().map(|id| id.as_str()))
        .bind(serde_json::to_value(manifest)?)
        .bind(&manifest.previous_manifest_hash)
        .bind(&manifest.manifest_hash)
        .execute(&mut *tx)
        .await?;

        for object_ref in &manifest.object_refs {
            sqlx::query(
                r#"
                INSERT INTO snapshot_object_refs (
                  snapshot_id, logical_path, object_uri, sha256, size_bytes
                )
                VALUES ($1,$2,$3,$4,$5)
                "#,
            )
            .bind(manifest.id.as_str())
            .bind(&object_ref.logical_path)
            .bind(&object_ref.object_uri)
            .bind(&object_ref.sha256)
            .bind(object_ref.size_bytes as i64)
            .execute(&mut *tx)
            .await?;
        }

        for export in &manifest.table_exports {
            sqlx::query(
                r#"
                INSERT INTO snapshot_table_exports (
                  snapshot_id, table_name, export_uri, sha256, row_count
                )
                VALUES ($1,$2,$3,$4,$5)
                "#,
            )
            .bind(manifest.id.as_str())
            .bind(&export.table_name)
            .bind(&export.export_uri)
            .bind(&export.sha256)
            .bind(export.row_count as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_snapshot_manifest(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<Option<SnapshotManifest>, StoreError> {
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT manifest_json FROM snapshot_manifests WHERE id = $1")
                .bind(snapshot_id.as_str())
                .fetch_optional(&self.pool)
                .await?;
        row.map(|(value,)| serde_json::from_value(value))
            .transpose()
            .map_err(StoreError::from)
    }
}

fn snapshot_status_value(status: &SnapshotStatus) -> &'static str {
    match status {
        SnapshotStatus::Draft => "draft",
        SnapshotStatus::KnownGood => "known_good",
        SnapshotStatus::Invalid => "invalid",
        SnapshotStatus::RollbackPreviewed => "rollback_previewed",
    }
}
