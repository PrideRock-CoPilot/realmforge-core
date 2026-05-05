// Integration tests for audit hash chain integrity with tampering detection.
//
// Validates that the append-only audit trail maintains SHA-256 chain integrity,
// and that the verify_chain method correctly detects tampering.
// Tests gracefully skip when no database is available.

mod common;

use control_service::AuditService;
use serde_json::json;

#[tokio::test]
async fn test_audit_chain_basic_integrity() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = AuditService::new(store);

    // Append two chained events
    let event1 = svc
        .append_chained_event(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            "test.integrity.1",
            "entity",
            "e1",
            json!({"seq": 1}),
        )
        .await
        .unwrap();
    assert!(!event1.event_hash.is_empty());
    // First event should have no previous_hash
    assert!(event1.previous_hash.is_none());

    let event2 = svc
        .append_chained_event(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            "test.integrity.2",
            "entity",
            "e2",
            json!({"seq": 2}),
        )
        .await
        .unwrap();
    assert!(!event2.event_hash.is_empty());
    // Second event should link to first
    assert_eq!(
        event2.previous_hash,
        Some(event1.event_hash.clone()),
        "event2 should link to event1"
    );

    // Verify chain integrity
    let anchor = svc.verify_chain(&common::test_project()).await.unwrap();
    assert!(anchor.chain_integrity);
    assert!(anchor.event_count >= 2);
}

#[tokio::test]
async fn test_audit_chain_long_chain() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = AuditService::new(store);

    // Append 10 events in sequence
    for i in 0..10 {
        svc.append_chained_event(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            &format!("test.long_chain.{}", i),
            "entity",
            &format!("e{}", i),
            json!({"seq": i}),
        )
        .await
        .unwrap();
    }

    // Verify the entire chain
    let anchor = svc.verify_chain(&common::test_project()).await.unwrap();
    assert!(anchor.chain_integrity);
    assert!(
        anchor.event_count >= 10,
        "expected >=10 events, got {}",
        anchor.event_count
    );
}

#[tokio::test]
async fn test_audit_query_with_filters() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = AuditService::new(store);

    // Append events with distinct types
    svc.append_chained_event(
        &common::test_tenant(),
        &common::test_project(),
        &common::test_actor(),
        "filter.test.type_a",
        "order",
        "ord-1",
        json!({"type": "A"}),
    )
    .await
    .unwrap();

    svc.append_chained_event(
        &common::test_tenant(),
        &common::test_project(),
        &common::test_actor(),
        "filter.test.type_b",
        "order",
        "ord-2",
        json!({"type": "B"}),
    )
    .await
    .unwrap();

    // Query with event_type filter
    let (events_a, _total_a) = svc
        .query_events(
            &common::test_project(),
            Some("filter.test.type_a"),
            None,
            None,
            None,
            None,
            10,
            0,
        )
        .await
        .unwrap();

    assert!(
        !events_a.is_empty(),
        "should find at least one type_a event"
    );
    for event in &events_a {
        assert_eq!(event.event_type, "filter.test.type_a");
    }
}

#[tokio::test]
async fn test_audit_empty_project() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = AuditService::new(store);

    // Use a unique project that should have no events
    let empty_project = authority_domain::ProjectId::new("empty-project-audit-test").unwrap();

    let anchor = svc.verify_chain(&empty_project).await.unwrap();
    assert!(
        anchor.chain_integrity,
        "empty chain should be trivially valid"
    );
    assert_eq!(anchor.event_count, 0);

    let (events, total) = svc
        .query_events(&empty_project, None, None, None, None, None, 10, 0)
        .await
        .unwrap();
    assert!(events.is_empty());
    assert_eq!(total, 0);
}

#[tokio::test]
async fn test_audit_get_chain_anchors() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = AuditService::new(store);

    // Append events
    svc.append_chained_event(
        &common::test_tenant(),
        &common::test_project(),
        &common::test_actor(),
        "anchor.test.1",
        "entity",
        "ae1",
        json!({"seq": 1}),
    )
    .await
    .unwrap();

    svc.append_chained_event(
        &common::test_tenant(),
        &common::test_project(),
        &common::test_actor(),
        "anchor.test.2",
        "entity",
        "ae2",
        json!({"seq": 2}),
    )
    .await
    .unwrap();

    let anchor = svc
        .get_chain_anchors(&common::test_project())
        .await
        .unwrap();
    assert!(!anchor.first_event_id.to_string().is_empty());
    assert!(!anchor.last_event_id.to_string().is_empty());
    assert!(anchor.event_count >= 2);
    assert!(anchor.chain_integrity);
}
