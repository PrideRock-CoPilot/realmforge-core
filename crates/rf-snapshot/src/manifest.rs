use chrono::{DateTime, Utc};
use rf_domain::{ProjectId, SnapshotId, SnapshotStatus, TenantId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to serialize snapshot manifest")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotObjectRef {
    pub logical_path: String,
    pub object_uri: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotTableExport {
    pub table_name: String,
    pub export_uri: String,
    pub sha256: String,
    pub row_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub id: SnapshotId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub status: SnapshotStatus,
    pub reason: String,
    pub parent_snapshot_id: Option<SnapshotId>,
    pub created_at: DateTime<Utc>,
    pub object_refs: Vec<SnapshotObjectRef>,
    pub table_exports: Vec<SnapshotTableExport>,
    pub previous_manifest_hash: Option<String>,
    pub manifest_hash: String,
}

#[derive(Serialize)]
struct ManifestHashInput<'a> {
    id: &'a SnapshotId,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    status: &'a SnapshotStatus,
    reason: &'a str,
    parent_snapshot_id: &'a Option<SnapshotId>,
    created_at: DateTime<Utc>,
    object_refs: &'a [SnapshotObjectRef],
    table_exports: &'a [SnapshotTableExport],
    previous_manifest_hash: &'a Option<String>,
}

impl SnapshotManifest {
    pub fn create(
        tenant_id: TenantId,
        project_id: ProjectId,
        reason: impl Into<String>,
        parent_snapshot_id: Option<SnapshotId>,
        object_refs: Vec<SnapshotObjectRef>,
        table_exports: Vec<SnapshotTableExport>,
        previous_manifest_hash: Option<String>,
    ) -> Result<Self, ManifestError> {
        let mut manifest = Self {
            id: SnapshotId::generate(),
            tenant_id,
            project_id,
            status: SnapshotStatus::Draft,
            reason: reason.into(),
            parent_snapshot_id,
            created_at: Utc::now(),
            object_refs,
            table_exports,
            previous_manifest_hash,
            manifest_hash: String::new(),
        };
        manifest.manifest_hash = manifest.compute_hash()?;
        Ok(manifest)
    }

    pub fn compute_hash(&self) -> Result<String, ManifestError> {
        let input = ManifestHashInput {
            id: &self.id,
            tenant_id: &self.tenant_id,
            project_id: &self.project_id,
            status: &self.status,
            reason: &self.reason,
            parent_snapshot_id: &self.parent_snapshot_id,
            created_at: self.created_at,
            object_refs: &self.object_refs,
            table_exports: &self.table_exports,
            previous_manifest_hash: &self.previous_manifest_hash,
        };
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(&input)?)))
    }

    pub fn verify_hash(&self) -> Result<bool, ManifestError> {
        Ok(self.compute_hash()? == self.manifest_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_hash_detects_changed_objects() {
        let mut manifest = SnapshotManifest::create(
            TenantId::new("tenant").unwrap(),
            ProjectId::new("project").unwrap(),
            "baseline",
            None,
            vec![SnapshotObjectRef {
                logical_path: "file.txt".to_string(),
                object_uri: "sha256/aa/bb/hash".to_string(),
                sha256: "hash".to_string(),
                size_bytes: 4,
            }],
            vec![],
            None,
        )
        .unwrap();

        assert!(manifest.verify_hash().unwrap());
        manifest.object_refs[0].size_bytes = 5;
        assert!(!manifest.verify_hash().unwrap());
    }
}
