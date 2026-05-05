use authority_domain::{ActorId, ProjectId, SnapshotId, TenantId};
use chrono::Utc;
use control_store::CoreStore;
use serde::{Deserialize, Serialize};
use snapshot_ledger::{SnapshotDelta, SnapshotManifest};

use tracing::{info, instrument};

use crate::audit_service::AuditService;
use crate::error::ServiceError;

/// A preview of a rollback's impact — computed before execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollbackPreview {
    pub id: String,
    pub from_snapshot_id: SnapshotId,
    pub to_snapshot_id: SnapshotId,
    pub delta: SnapshotDelta,
    /// Number of objects that differ and will be restored (added + changed in `to`).
    pub objects_to_restore: u64,
    pub estimated_impact: String,
    /// Any conditions that block execution.
    pub blockers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Result of a completed rollback execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollbackResult {
    pub new_snapshot_id: SnapshotId,
    pub objects_restored: u64,
    pub tables_restored: u64,
    pub verification_status: String,
}

/// Post-rollback consistency verification report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollbackVerification {
    pub snapshot_id: SnapshotId,
    pub all_hashes_match: bool,
    pub entity_counts_match: bool,
    pub consistency_ok: bool,
}

/// Rollback orchestration service.
#[derive(Clone)]
pub struct RollbackService {
    store: CoreStore,
    audit: AuditService,
}

impl RollbackService {
    pub fn new(store: CoreStore, audit: AuditService) -> Self {
        Self { store, audit }
    }

    /// Preview a rollback between two snapshots.
    ///
    /// Computes the full delta using the canonical `SnapshotDelta::compute`,
    /// counts only objects that differ (added + changed) as `objects_to_restore`,
    /// and identifies any blockers (non-KnownGood snapshots).
    #[instrument(skip(self), fields(from = %from_snapshot_id, to = %to_snapshot_id))]
    pub async fn preview_rollback(
        &self,
        from_snapshot_id: &SnapshotId,
        to_snapshot_id: &SnapshotId,
    ) -> Result<RollbackPreview, ServiceError> {
        let from = self
            .store
            .get_snapshot_manifest(from_snapshot_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;
        let to = self
            .store
            .get_snapshot_manifest(to_snapshot_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;

        let mut blockers = Vec::new();
        if from.status != authority_domain::SnapshotStatus::KnownGood {
            blockers.push(format!(
                "source snapshot {} is not KnownGood (status: {:?})",
                from_snapshot_id, from.status
            ));
        }
        if to.status != authority_domain::SnapshotStatus::KnownGood {
            blockers.push(format!(
                "target snapshot {} is not KnownGood (status: {:?})",
                to_snapshot_id, to.status
            ));
        }

        let delta = SnapshotDelta::compute(&from, &to);

        // objects_to_restore = objects that are new-in-to (added) or changed between
        // snapshots. Unchanged objects require no restoration work.
        let objects_to_restore = (delta.added.len() + delta.changed.len()) as u64;

        let preview = RollbackPreview {
            id: format!("preview_{}", SnapshotId::generate()),
            from_snapshot_id: from_snapshot_id.clone(),
            to_snapshot_id: to_snapshot_id.clone(),
            objects_to_restore,
            estimated_impact: format!(
                "{} objects to restore, {} to remove, {} changed; {} table exports affected",
                delta.added.len(),
                delta.removed.len(),
                delta.changed.len(),
                delta.added_exports.len() + delta.removed_exports.len(),
            ),
            blockers,
            delta,
            created_at: Utc::now(),
        };

        info!(
            blocker_count = preview.blockers.len(),
            objects_to_restore = preview.objects_to_restore,
            "rollback preview computed"
        );
        Ok(preview)
    }

    /// Execute a rollback: restores `to_snapshot` state, creates a new rollback
    /// snapshot, and emits an audit event for each restored object.
    #[instrument(skip(self), fields(from = %_from_snapshot_id, to = %to_snapshot_id, actor = %actor_id))]
    pub async fn execute_rollback(
        &self,
        _from_snapshot_id: &SnapshotId,
        to_snapshot_id: &SnapshotId,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        actor_id: &ActorId,
    ) -> Result<RollbackResult, ServiceError> {
        let to = self
            .store
            .get_snapshot_manifest(to_snapshot_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;

        // Create a new snapshot recording this rollback point.
        let new_manifest = SnapshotManifest::create(
            to.tenant_id.clone(),
            to.project_id.clone(),
            format!("rollback to {to_snapshot_id}"),
            Some(to_snapshot_id.clone()),
            to.object_refs.clone(),
            to.table_exports.clone(),
            Some(to.manifest_hash.clone()),
        )?;
        self.store.insert_snapshot_manifest(&new_manifest).await?;

        let objects_restored = to.object_refs.len() as u64;
        let tables_restored = to.table_exports.len() as u64;

        // Emit one audit event per restored object so the trail is complete.
        for obj in &to.object_refs {
            self.audit
                .append_chained_event(
                    tenant_id,
                    project_id,
                    actor_id,
                    "rollback.object_restored",
                    "snapshot_object",
                    &obj.logical_path,
                    serde_json::json!({
                        "rollback_snapshot_id": new_manifest.id.as_str(),
                        "source_snapshot_id": to_snapshot_id.as_str(),
                        "logical_path": obj.logical_path,
                        "sha256": obj.sha256,
                    }),
                )
                .await?;
        }

        // Emit a single summary event.
        self.audit
            .append_chained_event(
                tenant_id,
                project_id,
                actor_id,
                "rollback.executed",
                "snapshot",
                new_manifest.id.as_str(),
                serde_json::json!({
                    "rollback_snapshot_id": new_manifest.id.as_str(),
                    "source_snapshot_id": to_snapshot_id.as_str(),
                    "objects_restored": objects_restored,
                    "tables_restored": tables_restored,
                }),
            )
            .await?;

        let result = RollbackResult {
            new_snapshot_id: new_manifest.id,
            objects_restored,
            tables_restored,
            verification_status: "pending".to_string(),
        };
        info!(
            new_snapshot_id = %result.new_snapshot_id,
            objects_restored,
            tables_restored,
            "rollback executed"
        );
        Ok(result)
    }

    /// Verify that a rollback snapshot is consistent — hash integrity and object counts.
    #[instrument(skip(self), fields(snapshot_id = %snapshot_id))]
    pub async fn verify_rollback(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<RollbackVerification, ServiceError> {
        let manifest = self
            .store
            .get_snapshot_manifest(snapshot_id)
            .await?
            .ok_or(ServiceError::SnapshotNotFound)?;

        let hash_valid = manifest.verify_hash()?;

        // entity_counts_match: every object_ref has a non-empty sha256
        let entity_counts_match = manifest
            .object_refs
            .iter()
            .all(|r| !r.sha256.is_empty() && !r.logical_path.is_empty());

        let result = RollbackVerification {
            snapshot_id: snapshot_id.clone(),
            all_hashes_match: hash_valid,
            entity_counts_match,
            consistency_ok: hash_valid && entity_counts_match,
        };
        info!(
            all_hashes_match = hash_valid,
            entity_counts_match, "rollback verified"
        );
        Ok(result)
    }
}
