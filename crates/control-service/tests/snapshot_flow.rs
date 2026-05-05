// Integration tests for snapshot flow: Create → Validate → Compare → Store → Retrieve.
//
// Validates the full SnapshotService workflow against a live MongoDB database.
// Tests gracefully skip when no database is available.

mod common;

use control_service::SnapshotService;
use snapshot_ledger::SnapshotObjectRef;

fn make_object_ref(path: &str, hash: &str, size: u64) -> SnapshotObjectRef {
    SnapshotObjectRef {
        logical_path: path.to_string(),
        object_uri: format!("sha256/{}/{}", &hash[..2], hash),
        sha256: hash.to_string(),
        size_bytes: size,
    }
}

#[tokio::test]
async fn test_snapshot_create_and_validate() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = SnapshotService::new(store);
    let object_refs = vec![make_object_ref("test/file.txt", "test_hash_abc", 100)];

    let manifest = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "snapshot-flow test",
            None,
            object_refs,
            vec![],
            None,
        )
        .await
        .unwrap();

    assert_eq!(manifest.reason, "snapshot-flow test");
    assert_eq!(manifest.object_refs.len(), 1);

    // Validate the snapshot
    let report = svc.validate_snapshot(&manifest.id).await.unwrap();
    assert!(report.hash_valid);
    assert_eq!(report.snapshot_id, manifest.id);
}

#[tokio::test]
async fn test_snapshot_not_found() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = SnapshotService::new(store);
    let fake_id = authority_domain::SnapshotId::generate();
    let result = svc.validate_snapshot(&fake_id).await;
    assert!(matches!(
        result.unwrap_err(),
        control_service::error::ServiceError::SnapshotNotFound
    ));
}

#[tokio::test]
async fn test_snapshot_create_with_parent() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = SnapshotService::new(store);

    // Create first snapshot
    let obj1 = vec![make_object_ref("file1.txt", "hash1", 10)];
    let snap1 = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "parent snapshot",
            None,
            obj1,
            vec![],
            None,
        )
        .await
        .unwrap();

    // Create second snapshot with parent reference
    let obj2 = vec![
        make_object_ref("file1.txt", "hash1", 10),
        make_object_ref("file2.txt", "hash2", 20),
    ];
    let snap2 = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "child snapshot",
            Some(snap1.id.clone()),
            obj2,
            vec![],
            Some(snap1.manifest_hash.clone()),
        )
        .await
        .unwrap();

    assert_eq!(snap2.parent_snapshot_id, Some(snap1.id));
    assert_eq!(snap2.previous_manifest_hash, Some(snap1.manifest_hash));
}

#[tokio::test]
async fn test_snapshot_list_and_retrieve() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = SnapshotService::new(store);

    // Create a couple of snapshots
    let snap1 = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "list test 1",
            None,
            vec![make_object_ref("a.txt", "hash_a", 10)],
            vec![],
            None,
        )
        .await
        .unwrap();

    svc.create_snapshot(
        &common::test_tenant(),
        &common::test_project(),
        "list test 2",
        None,
        vec![make_object_ref("b.txt", "hash_b", 20)],
        vec![],
        None,
    )
    .await
    .unwrap();

    // List snapshots
    let manifests = svc
        .list_snapshots(&common::test_project(), 10, 0)
        .await
        .unwrap();
    assert!(
        manifests.len() >= 2,
        "expected >=2 snapshots, got {}",
        manifests.len()
    );

    // Verify snap1 is in the list
    let ids: Vec<&authority_domain::SnapshotId> = manifests.iter().map(|m| &m.id).collect();
    assert!(ids.contains(&&snap1.id), "expected snap1.id in list");
}

#[tokio::test]
async fn test_snapshot_compare() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = SnapshotService::new(store);

    let obj1 = vec![make_object_ref("common.txt", "hash_c", 15)];
    let snap1 = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "compare from",
            None,
            obj1,
            vec![],
            None,
        )
        .await
        .unwrap();

    let obj2 = vec![
        make_object_ref("common.txt", "hash_c", 15),
        make_object_ref("new.txt", "hash_new", 30),
    ];
    let snap2 = svc
        .create_snapshot(
            &common::test_tenant(),
            &common::test_project(),
            "compare to",
            None,
            obj2,
            vec![],
            None,
        )
        .await
        .unwrap();

    let delta = svc.compare_snapshots(&snap1.id, &snap2.id).await.unwrap();

    // snap2 has "new.txt" which snap1 doesn't
    assert!(
        delta.added.contains(&"new.txt".to_string()),
        "expected new.txt in added: {:?}",
        delta.added
    );
}
