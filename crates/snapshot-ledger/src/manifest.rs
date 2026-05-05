use authority_domain::{ProjectId, SnapshotId, SnapshotStatus, TenantId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to serialize snapshot manifest")]
    Serialize(#[from] serde_json::Error),
    #[error("manifest validation failed: {0}")]
    Validation(String),
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

/// A computed delta between two snapshot manifests.
/// Used to determine what changed between snapshots.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotDelta {
    /// Object paths added in `new` that didn't exist in `old`.
    pub added: Vec<String>,
    /// Object paths removed in `new` that existed in `old`.
    pub removed: Vec<String>,
    /// Object paths that exist in both but with different sha256 hashes.
    pub changed: Vec<String>,
    /// Table exports present in `new` but not in `old`.
    pub added_exports: Vec<String>,
    /// Table exports present in `old` but not in `new`.
    pub removed_exports: Vec<String>,
}

impl SnapshotDelta {
    /// Compute the full delta between an old and new manifest.
    pub fn compute(old: &SnapshotManifest, new: &SnapshotManifest) -> Self {
        let old_paths: std::collections::HashMap<&str, &str> = old
            .object_refs
            .iter()
            .map(|r| (r.logical_path.as_str(), r.sha256.as_str()))
            .collect();
        let new_paths: std::collections::HashMap<&str, &str> = new
            .object_refs
            .iter()
            .map(|r| (r.logical_path.as_str(), r.sha256.as_str()))
            .collect();

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        for (path, new_hash) in &new_paths {
            if !old_paths.contains_key(path) {
                added.push((*path).to_string());
            } else if old_paths.get(path) != Some(new_hash) {
                changed.push((*path).to_string());
            }
        }
        for path in old_paths.keys() {
            if !new_paths.contains_key(path) {
                removed.push((*path).to_string());
            }
        }

        let old_exports: std::collections::HashSet<&str> = old
            .table_exports
            .iter()
            .map(|e| e.table_name.as_str())
            .collect();
        let new_exports: std::collections::HashSet<&str> = new
            .table_exports
            .iter()
            .map(|e| e.table_name.as_str())
            .collect();

        let added_exports: Vec<String> = new_exports
            .difference(&old_exports)
            .map(|s| s.to_string())
            .collect();
        let removed_exports: Vec<String> = old_exports
            .difference(&new_exports)
            .map(|s| s.to_string())
            .collect();

        Self {
            added,
            removed,
            changed,
            added_exports,
            removed_exports,
        }
    }

    /// Returns `true` if nothing changed between the two manifests.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && self.added_exports.is_empty()
            && self.removed_exports.is_empty()
    }

    /// Total number of object-level changes (added + removed + changed).
    pub fn object_change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
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

    /// Validate the manifest: verify hash integrity and that all object refs
    /// have non-empty sha256 hashes.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if !self.verify_hash()? {
            return Err(ManifestError::Validation(
                "manifest hash mismatch".to_string(),
            ));
        }
        for obj in &self.object_refs {
            if obj.sha256.is_empty() {
                return Err(ManifestError::Validation(format!(
                    "object ref {} has empty sha256",
                    obj.logical_path
                )));
            }
        }
        Ok(())
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

    #[test]
    fn snapshot_delta_detects_added_removed_changed() {
        let old = SnapshotManifest::create(
            TenantId::new("t").unwrap(),
            ProjectId::new("p").unwrap(),
            "old",
            None,
            vec![
                SnapshotObjectRef {
                    logical_path: "a.txt".to_string(),
                    object_uri: "uri/a".to_string(),
                    sha256: "aaa".to_string(),
                    size_bytes: 3,
                },
                SnapshotObjectRef {
                    logical_path: "b.txt".to_string(),
                    object_uri: "uri/b".to_string(),
                    sha256: "bbb".to_string(),
                    size_bytes: 3,
                },
            ],
            vec![],
            None,
        )
        .unwrap();

        let new = SnapshotManifest::create(
            TenantId::new("t").unwrap(),
            ProjectId::new("p").unwrap(),
            "new",
            None,
            vec![
                SnapshotObjectRef {
                    logical_path: "b.txt".to_string(),
                    object_uri: "uri/b".to_string(),
                    sha256: "bbb".to_string(),
                    size_bytes: 3,
                },
                SnapshotObjectRef {
                    logical_path: "c.txt".to_string(),
                    object_uri: "uri/c".to_string(),
                    sha256: "ccc".to_string(),
                    size_bytes: 3,
                },
            ],
            vec![],
            None,
        )
        .unwrap();

        let delta = SnapshotDelta::compute(&old, &new);
        assert!(delta.added.contains(&"c.txt".to_string()));
        assert!(delta.removed.contains(&"a.txt".to_string()));
        assert!(delta.changed.is_empty());
        assert!(delta.added_exports.is_empty());
        assert!(delta.removed_exports.is_empty());
        assert!(!delta.is_empty());
        assert_eq!(delta.object_change_count(), 2);
    }

    #[test]
    fn empty_delta_when_no_changes() {
        let m = SnapshotManifest::create(
            TenantId::new("t").unwrap(),
            ProjectId::new("p").unwrap(),
            "baseline",
            None,
            vec![],
            vec![],
            None,
        )
        .unwrap();
        let delta = SnapshotDelta::compute(&m, &m);
        assert!(delta.is_empty());
    }

    #[test]
    fn validate_rejects_corrupted_manifest() {
        let mut manifest = SnapshotManifest::create(
            TenantId::new("t").unwrap(),
            ProjectId::new("p").unwrap(),
            "test",
            None,
            vec![],
            vec![],
            None,
        )
        .unwrap();
        manifest.manifest_hash = "tampered".to_string();
        assert!(manifest.validate().is_err());
    }
}
