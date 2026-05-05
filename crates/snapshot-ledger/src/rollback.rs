use crate::SnapshotManifest;
use authority_domain::{SnapshotId, SnapshotStatus};
use serde::{Deserialize, Serialize};

/// A preview of what a rollback would do.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollbackPreview {
    pub from_snapshot_id: SnapshotId,
    pub to_snapshot_id: SnapshotId,
    /// Objects that will be restored (exist in `to` but not in `from`).
    pub objects_to_restore: Vec<String>,
    /// Objects that will be lost (exist in `from` but not in `to`).
    pub objects_to_lose: Vec<String>,
    /// Objects whose content changed between snapshots.
    pub objects_changed: Vec<String>,
    /// Any issues that would block the rollback.
    pub blockers: Vec<String>,
    /// Status of the preview.
    pub status: RollbackPreviewStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackPreviewStatus {
    Ready,
    Blocked,
}

/// Analyze impact of rolling back from one snapshot to another.
pub fn analyze_rollback(from: &SnapshotManifest, to: &SnapshotManifest) -> RollbackPreview {
    let from_paths: std::collections::HashSet<&str> = from
        .object_refs
        .iter()
        .map(|r| r.logical_path.as_str())
        .collect();
    let to_paths: std::collections::HashSet<&str> = to
        .object_refs
        .iter()
        .map(|r| r.logical_path.as_str())
        .collect();

    let objects_to_restore: Vec<String> = to_paths
        .difference(&from_paths)
        .map(|s| s.to_string())
        .collect();
    let objects_to_lose: Vec<String> = from_paths
        .difference(&to_paths)
        .map(|s| s.to_string())
        .collect();

    let objects_changed: Vec<String> = to_paths
        .intersection(&from_paths)
        .filter(|path| {
            let from_ref = from.object_refs.iter().find(|r| r.logical_path == **path);
            let to_ref = to.object_refs.iter().find(|r| r.logical_path == **path);
            from_ref.map(|r| &r.sha256) != to_ref.map(|r| &r.sha256)
        })
        .map(|s| s.to_string())
        .collect();

    let mut blockers = Vec::new();

    if from.status != SnapshotStatus::KnownGood {
        blockers.push(format!(
            "Source snapshot {} status is {:?}, must be KnownGood",
            from.id, from.status
        ));
    }
    if to.status != SnapshotStatus::KnownGood {
        blockers.push(format!(
            "Target snapshot {} status is {:?}, must be KnownGood",
            to.id, to.status
        ));
    }

    let status = if blockers.is_empty() {
        RollbackPreviewStatus::Ready
    } else {
        RollbackPreviewStatus::Blocked
    };

    RollbackPreview {
        from_snapshot_id: from.id.clone(),
        to_snapshot_id: to.id.clone(),
        objects_to_restore,
        objects_to_lose,
        objects_changed,
        blockers,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SnapshotManifest, SnapshotObjectRef};
    use authority_domain::{ProjectId, SnapshotStatus, TenantId};

    fn make_manifest(status: SnapshotStatus, paths: &[&str]) -> SnapshotManifest {
        let refs: Vec<SnapshotObjectRef> = paths
            .iter()
            .map(|p| SnapshotObjectRef {
                logical_path: p.to_string(),
                object_uri: format!("sha256/ab/cd/{p}"),
                sha256: p.to_string(),
                size_bytes: 4,
            })
            .collect();

        let mut manifest = SnapshotManifest::create(
            TenantId::new("t").unwrap(),
            ProjectId::new("p").unwrap(),
            "test",
            None,
            refs,
            vec![],
            None,
        )
        .unwrap();

        // Override the default Draft status and recompute hash
        manifest.status = status;
        manifest.manifest_hash = manifest.compute_hash().unwrap();
        manifest
    }

    #[test]
    fn analyze_rollback_detects_added_and_removed() {
        let from = make_manifest(SnapshotStatus::KnownGood, &["a.txt", "b.txt"]);
        let to = make_manifest(SnapshotStatus::KnownGood, &["b.txt", "c.txt"]);

        let preview = analyze_rollback(&from, &to);

        assert!(preview.objects_to_restore.contains(&"c.txt".to_string()));
        assert!(preview.objects_to_lose.contains(&"a.txt".to_string()));
        assert_eq!(preview.status, RollbackPreviewStatus::Ready);
    }

    #[test]
    fn detect_blocker_when_snapshot_not_known_good() {
        let from = make_manifest(SnapshotStatus::Draft, &["a.txt"]);
        let to = make_manifest(SnapshotStatus::KnownGood, &["b.txt"]);

        let preview = analyze_rollback(&from, &to);

        assert_eq!(preview.status, RollbackPreviewStatus::Blocked);
        assert!(!preview.blockers.is_empty());
    }
}
