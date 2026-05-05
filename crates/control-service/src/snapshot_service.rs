use authority_domain::{ProjectId, SnapshotId, TenantId};
use control_store::CoreStore;
pub use snapshot_ledger::SnapshotDelta;
use snapshot_ledger::{SnapshotManifest, SnapshotObjectRef, SnapshotTableExport};

use tracing::{info, instrument};

use crate::error::ServiceError;

/// Snapshot orchestration service.
#[derive(Clone)]
pub struct SnapshotService {
    store: CoreStore,
}

impl SnapshotService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Create a new snapshot.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, object_refs, table_exports), fields(tenant_id = %tenant_id, project_id = %project_id, reason = %reason))]
    pub async fn create_snapshot(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        reason: &str,
        parent_snapshot_id: Option<SnapshotId>,
        object_refs: Vec<SnapshotObjectRef>,
        table_exports: Vec<SnapshotTableExport>,
        previous_manifest_hash: Option<String>,
    ) -> Result<SnapshotManifest, ServiceError> {
        let manifest = SnapshotManifest::create(
            tenant_id.clone(),
            project_id.clone(),
            reason,
            parent_snapshot_id,
            object_refs,
            table_exports,
            previous_manifest_hash,
        )?;

        self.store.insert_snapshot_manifest(&manifest).await?;

        info!(snapshot_id = %manifest.id, manifest_hash = %manifest.manifest_hash, "snapshot created");
        Ok(manifest)
    }

    /// Validate a snapshot by verifying its hash and object references.
    #[instrument(skip(self), fields(snapshot_id = %snapshot_id))]
    pub async fn validate_snapshot(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<SnapshotValidationReport, ServiceError> {
        let manifest = self
            .store
            .get_snapshot_manifest(snapshot_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;

        let hash_valid = manifest.verify_hash()?;
        let all_object_refs_exist = manifest.object_refs.iter().all(|object_ref| {
            !object_ref.logical_path.trim().is_empty()
                && !object_ref.object_uri.trim().is_empty()
                && !object_ref.sha256.trim().is_empty()
        });
        let table_exports_valid = manifest.table_exports.iter().all(|table_export| {
            !table_export.table_name.trim().is_empty()
                && !table_export.export_uri.trim().is_empty()
                && !table_export.sha256.trim().is_empty()
        });

        Ok(SnapshotValidationReport {
            snapshot_id: snapshot_id.clone(),
            hash_valid,
            all_object_refs_exist,
            table_exports_valid,
            overall_valid: hash_valid && all_object_refs_exist && table_exports_valid,
        })
    }

    /// List snapshots for a project, ordered by creation time descending.
    #[instrument(skip(self), fields(project_id = %project_id, limit = limit, offset = offset))]
    pub async fn list_snapshots(
        &self,
        project_id: &ProjectId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<SnapshotManifest>, ServiceError> {
        let manifests = self
            .store
            .list_snapshot_manifests(project_id, limit as i64, offset as i64)
            .await?;
        Ok(manifests)
    }

    /// Compare two snapshots and return the delta.
    #[instrument(skip(self), fields(from_id = %from_id, to_id = %to_id))]
    pub async fn compare_snapshots(
        &self,
        from_id: &SnapshotId,
        to_id: &SnapshotId,
    ) -> Result<SnapshotDelta, ServiceError> {
        let from = self
            .store
            .get_snapshot_manifest(from_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;
        let to = self
            .store
            .get_snapshot_manifest(to_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;

        let delta = SnapshotDelta::compute(&from, &to);
        info!(
            added = delta.added.len(),
            removed = delta.removed.len(),
            changed = delta.changed.len(),
            added_exports = delta.added_exports.len(),
            removed_exports = delta.removed_exports.len(),
            "snapshots compared"
        );
        Ok(delta)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SnapshotValidationReport {
    pub snapshot_id: SnapshotId,
    pub hash_valid: bool,
    pub all_object_refs_exist: bool,
    pub table_exports_valid: bool,
    pub overall_valid: bool,
}
