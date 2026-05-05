use crate::StoreError;
use authority_domain::{BundleId, BundleManifest, BundleStatus, RuntimeId, RuntimeInstance};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::PgPool;

/// Raw row for bundle_manifests table.
#[derive(sqlx::FromRow)]
struct BundleManifestRaw {
    id: String,
    version: String,
    app_id: String,
    artifact_hashes: serde_json::Value,
    governance_signature: String,
    release_approval_ref: Option<String>,
    built_at: DateTime<Utc>,
    status: String,
}

impl TryInto<BundleManifest> for BundleManifestRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<BundleManifest, Self::Error> {
        let artifact_hashes: Vec<(String, String)> = serde_json::from_value(self.artifact_hashes)?;
        let status = parse_bundle_status(&self.status)?;
        let bundle_id = BundleId::new(&self.id)?;

        Ok(BundleManifest {
            bundle_id,
            version: self.version,
            app_id: self.app_id,
            artifact_hashes,
            governance_signature: self.governance_signature,
            release_approval_ref: self
                .release_approval_ref
                .map(authority_domain::ApprovalId::new)
                .transpose()?,
            built_at: self.built_at,
            status,
        })
    }
}

/// Raw row for runtime_instances table.
#[derive(sqlx::FromRow)]
struct RuntimeInstanceRaw {
    id: String,
    bundle_id: String,
    status: String,
    started_at: DateTime<Utc>,
    last_heartbeat: DateTime<Utc>,
    active_sessions: i64,
    action_count: i64,
    error_count: i64,
    metadata: serde_json::Value,
}

impl TryInto<RuntimeInstance> for RuntimeInstanceRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<RuntimeInstance, Self::Error> {
        Ok(RuntimeInstance {
            runtime_id: RuntimeId::new(&self.id)?,
            bundle_id: BundleId::new(&self.bundle_id)?,
            status: self.status,
            started_at: self.started_at,
            last_heartbeat: self.last_heartbeat,
            active_sessions: self.active_sessions,
            action_count: self.action_count,
            error_count: self.error_count,
            metadata: self.metadata,
        })
    }
}

fn parse_bundle_status(s: &str) -> Result<BundleStatus, StoreError> {
    match s {
        "building" => Ok(BundleStatus::Building),
        "signed" => Ok(BundleStatus::Signed),
        "verified" => Ok(BundleStatus::Verified),
        "deployed" => Ok(BundleStatus::Deployed),
        "running" => Ok(BundleStatus::Running),
        _ => Ok(BundleStatus::Failed(
            s.trim_start_matches("failed: ").to_string(),
        )),
    }
}

fn bundle_status_to_str(status: &BundleStatus) -> &str {
    match status {
        BundleStatus::Building => "building",
        BundleStatus::Signed => "signed",
        BundleStatus::Verified => "verified",
        BundleStatus::Deployed => "deployed",
        BundleStatus::Running => "running",
        BundleStatus::Failed(_) => "failed",
    }
}

/// Insert a new bundle manifest.
pub async fn insert_bundle_manifest(
    pool: &PgPool,
    manifest: &BundleManifest,
) -> Result<(), StoreError> {
    let artifact_hashes = serde_json::to_value(&manifest.artifact_hashes)?;
    sqlx::query(
        "INSERT INTO bundle_manifests (id, version, app_id, artifact_hashes, governance_signature, release_approval_ref, built_at, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(manifest.bundle_id.as_str())
    .bind(&manifest.version)
    .bind(&manifest.app_id)
    .bind(&artifact_hashes)
    .bind(&manifest.governance_signature)
    .bind(manifest.release_approval_ref.as_ref().map(|r| r.to_string()))
    .bind(manifest.built_at)
    .bind(bundle_status_to_str(&manifest.status))
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a bundle manifest by ID.
pub async fn get_bundle(
    pool: &PgPool,
    bundle_id: &BundleId,
) -> Result<Option<BundleManifest>, StoreError> {
    let row: Option<BundleManifestRaw> = sqlx::query_as(
        "SELECT id, version, app_id, artifact_hashes, governance_signature, release_approval_ref, built_at, status \
         FROM bundle_manifests WHERE id = $1",
    )
    .bind(bundle_id.as_str())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(r.try_into()?)),
        None => Ok(None),
    }
}

/// List bundles for a given app, ordered by built_at DESC.
pub async fn list_bundles(
    pool: &PgPool,
    app_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<BundleManifest>, StoreError> {
    let rows: Vec<BundleManifestRaw> = sqlx::query_as(
        "SELECT id, version, app_id, artifact_hashes, governance_signature, release_approval_ref, built_at, status \
         FROM bundle_manifests WHERE app_id = $1 \
         ORDER BY built_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(app_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|r| r.try_into()).collect()
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a new bundle manifest.
    #[tracing::instrument(skip(self))]
    pub async fn insert_bundle_manifest(
        &self,
        manifest: &BundleManifest,
    ) -> Result<(), StoreError> {
        insert_bundle_manifest(&self.pool, manifest).await
    }

    /// Get a bundle manifest by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_bundle(
        &self,
        bundle_id: &BundleId,
    ) -> Result<Option<BundleManifest>, StoreError> {
        get_bundle(&self.pool, bundle_id).await
    }

    /// List bundles for a given app, ordered by built_at DESC.
    #[tracing::instrument(skip(self))]
    pub async fn list_bundles(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BundleManifest>, StoreError> {
        list_bundles(&self.pool, app_id, limit, offset).await
    }

    /// Update a bundle's status.
    #[tracing::instrument(skip(self))]
    pub async fn update_bundle_status(
        &self,
        bundle_id: &BundleId,
        status: &BundleStatus,
    ) -> Result<(), StoreError> {
        update_bundle_status(&self.pool, bundle_id, status).await
    }

    /// Insert a new runtime instance.
    #[tracing::instrument(skip(self))]
    pub async fn insert_runtime_instance(
        &self,
        instance: &RuntimeInstance,
    ) -> Result<(), StoreError> {
        insert_runtime_instance(&self.pool, instance).await
    }

    /// Update a runtime instance's status.
    #[tracing::instrument(skip(self))]
    pub async fn update_runtime_status(
        &self,
        runtime_id: &RuntimeId,
        status: &str,
    ) -> Result<(), StoreError> {
        update_runtime_status(&self.pool, runtime_id, status).await
    }

    /// Get runtime health data for an instance.
    #[tracing::instrument(skip(self))]
    pub async fn get_runtime_health(
        &self,
        runtime_id: &RuntimeId,
    ) -> Result<Option<RuntimeInstance>, StoreError> {
        get_runtime_health(&self.pool, runtime_id).await
    }

    /// List all runtime instances, ordered by started_at DESC.
    #[tracing::instrument(skip(self))]
    pub async fn list_runtime_instances(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RuntimeInstance>, StoreError> {
        list_runtime_instances(&self.pool, limit, offset).await
    }
}

/// Update a bundle's status.
pub async fn update_bundle_status(
    pool: &PgPool,
    bundle_id: &BundleId,
    status: &BundleStatus,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE bundle_manifests SET status = $1 WHERE id = $2")
        .bind(bundle_status_to_str(status))
        .bind(bundle_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

/// Insert a new runtime instance.
pub async fn insert_runtime_instance(
    pool: &PgPool,
    instance: &RuntimeInstance,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO runtime_instances (id, bundle_id, status, started_at, last_heartbeat, active_sessions, action_count, error_count, metadata) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(instance.runtime_id.as_str())
    .bind(instance.bundle_id.as_str())
    .bind(&instance.status)
    .bind(instance.started_at)
    .bind(instance.last_heartbeat)
    .bind(instance.active_sessions)
    .bind(instance.action_count)
    .bind(instance.error_count)
    .bind(&instance.metadata)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update a runtime instance's status.
pub async fn update_runtime_status(
    pool: &PgPool,
    runtime_id: &RuntimeId,
    status: &str,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE runtime_instances SET status = $1, last_heartbeat = NOW() WHERE id = $2")
        .bind(status)
        .bind(runtime_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

/// Get runtime health data for an instance.
pub async fn get_runtime_health(
    pool: &PgPool,
    runtime_id: &RuntimeId,
) -> Result<Option<RuntimeInstance>, StoreError> {
    let row: Option<RuntimeInstanceRaw> = sqlx::query_as(
        "SELECT id, bundle_id, status, started_at, last_heartbeat, active_sessions, action_count, error_count, metadata \
         FROM runtime_instances WHERE id = $1",
    )
    .bind(runtime_id.as_str())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(r.try_into()?)),
        None => Ok(None),
    }
}

/// List all runtime instances, ordered by started_at DESC.
pub async fn list_runtime_instances(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<RuntimeInstance>, StoreError> {
    let rows: Vec<RuntimeInstanceRaw> = sqlx::query_as(
        "SELECT id, bundle_id, status, started_at, last_heartbeat, active_sessions, action_count, error_count, metadata \
         FROM runtime_instances ORDER BY started_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(|r| r.try_into()).collect()
}
