// Integration tests for rollback flow: Preview → Execute → Verify → Validate.
//
// Validates the full RollbackService workflow against a live MongoDB database.
// Tests gracefully skip when no database is available.

mod common;

use control_service::{RollbackService, SnapshotService};
use snapshot_ledger::SnapshotObjectRef;

fn make_obj(path: &str, hash: &str, size: u64) -> SnapshotObjectRef {
    SnapshotObjectRef {
        logical_path: path.to_string(),
        object_uri: format!("sha256/{}/{}", &hash[..2], hash),
        sha256: hash.to_string(),
        size_bytes: size,
    }
}

#[tokio::test]
async fn test_rollback_preview() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let snap_svc = SnapshotService::new(store.clone());
    let rollback_svc = RollbackService::new(store.clone(), common::audit_service(&store));

    // Create two snapshots
    let obj1 = vec![make_obj("file1.txt", "aaa", 10)];
    let snap1 = snap_svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "rollback from",
            None,
            obj1,
            vec![],
            None,
        )
        .await
        .unwrap();

    let obj2 = vec![
        make_obj("file1.txt", "aaa", 10),
        make_obj("file2.txt", "bbb", 20),
    ];
    let snap2 = snap_svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "rollback to",
            Some(snap1.id.clone()),
            obj2,
            vec![],
            Some(snap1.manifest_hash.clone()),
        )
        .await
        .unwrap();

    // Preview rollback from snap2 to snap1
    let preview = rollback_svc
        .preview_rollback(&snap2.id, &snap1.id)
        .await
        .unwrap();

    assert_eq!(preview.from_snapshot_id, snap2.id);
    assert_eq!(preview.to_snapshot_id, snap1.id);

    // Should have blockers since status is default (not KnownGood)
    assert!(
        !preview.blockers.is_empty(),
        "should have blockers for non-KnownGood snapshots"
    );
}

#[tokio::test]
async fn test_rollback_execute_and_verify() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let snap_svc = SnapshotService::new(store.clone());
    let rollback_svc = RollbackService::new(store.clone(), common::audit_service(&store));

    // Create a target snapshot to rollback to
    let obj = vec![make_obj("restore.txt", "restore_hash", 50)];
    let target = snap_svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "rollback target",
            None,
            obj,
            vec![],
            None,
        )
        .await
        .unwrap();

    // Create a "current" snapshot (from which we rollback)
    let from_snap = snap_svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "rollback current",
            Some(target.id.clone()),
            vec![make_obj("extra.txt", "extra", 25)],
            vec![],
            Some(target.manifest_hash.clone()),
        )
        .await
        .unwrap();

    // Execute rollback from from_snap to target
    let result = rollback_svc
        .execute_rollback(
            &from_snap.id,
            &target.id,
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
        )
        .await
        .unwrap();

    assert!(result.objects_restored > 0);
    assert_eq!(result.verification_status, "pending");

    // Verify the new snapshot created by the rollback
    let verification = rollback_svc
        .verify_rollback(&result.new_snapshot_id)
        .await
        .unwrap();
    assert!(verification.all_hashes_match);
    assert!(verification.consistency_ok);
}

#[tokio::test]
async fn test_rollback_not_found() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let rollback_svc = RollbackService::new(store.clone(), common::audit_service(&store));
    let fake_id = authority_domain::SnapshotId::generate();

    // Preview with non-existent snapshots should fail
    let result = rollback_svc.preview_rollback(&fake_id, &fake_id).await;
    assert!(result.is_err());

    // Verify with non-existent snapshot should fail
    let verify_result = rollback_svc.verify_rollback(&fake_id).await;
    assert!(verify_result.is_err());
}

#[tokio::test]
async fn test_rollback_execute_creates_new_snapshot() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let snap_svc = SnapshotService::new(store.clone());
    let rollback_svc = RollbackService::new(store.clone(), common::audit_service(&store));

    // Create target snapshot
    let target = snap_svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "rollback target for new",
            None,
            vec![make_obj("keep.txt", "keep_hash", 10)],
            vec![],
            None,
        )
        .await
        .unwrap();

    // Execute rollback
    let result = rollback_svc
        .execute_rollback(
            &authority_domain::SnapshotId::generate(), // from = non-existent (will skip existing checks)
            &target.id,
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
        )
        .await
        .unwrap();

    // Should have created a new snapshot
    let new_manifest = snap_svc
        .validate_snapshot(&result.new_snapshot_id)
        .await
        .unwrap();
    assert!(new_manifest.hash_valid);
    assert_eq!(new_manifest.snapshot_id, result.new_snapshot_id);
}
