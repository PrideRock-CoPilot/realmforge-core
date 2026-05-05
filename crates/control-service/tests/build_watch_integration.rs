// Integration tests for Build Watch: events, violations, cost summary, and dashboard.
//
// Validates the full BuildWatchService workflow against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{WatchEventType, WatchSeverity};
use control_service::BuildWatchService;
use sqlx::PgPool;

/// Clean up stale cost records and events for a given scope.
async fn clear_scope_data(pool: &PgPool, scope: &str) {
    sqlx::query("DELETE FROM cost_records WHERE scope = $1")
        .bind(scope)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM watch_events WHERE scope = $1")
        .bind(scope)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM violations WHERE evidence_ref LIKE $1")
        .bind(format!("%{scope}%"))
        .execute(pool)
        .await
        .ok();
}

/// Get the pool from a CoreStore.
fn get_pool(store: &control_store::CoreStore) -> PgPool {
    store.pool().clone()
}

#[tokio::test]
async fn test_build_watch_record_and_query_events() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BuildWatchService::new(store);

    let event = svc
        .record_event(
            "scope-1".to_string(),
            WatchEventType::FileMutation,
            WatchSeverity::Warning,
            "unauthorized file mutation detected".to_string(),
            Some("evidence-1".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(event.scope, "scope-1");
    assert_eq!(format!("{}", event.event_type), "file_mutation");
    assert_eq!(format!("{}", event.severity), "warning");

    // Query back the event
    let events = svc
        .query_watch_events(Some("scope-1".to_string()), None, None, 10, 0)
        .await
        .unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.id == event.id));
}

#[tokio::test]
async fn test_build_watch_record_violation() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BuildWatchService::new(store);

    let violation = svc
        .detect_violation(
            "no-direct-db-access".to_string(),
            WatchSeverity::Violation,
            "agent attempted direct database access".to_string(),
            Some("evidence-db-001".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(violation.rule, "no-direct-db-access");
    assert_eq!(format!("{}", violation.severity), "violation");
    assert_eq!(violation.evidence_ref, Some("evidence-db-001".to_string()));
}

#[tokio::test]
async fn test_build_watch_cost_summary() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let pool = get_pool(&store);
    let svc = BuildWatchService::new(store);

    // Clean up stale data from previous runs
    clear_scope_data(&pool, "scope-cost").await;

    // Record two cost entries in the same scope
    svc.get_cost_summary("scope-cost".to_string(), 100, 5000, 1024, 1)
        .await
        .unwrap();

    svc.get_cost_summary("scope-cost".to_string(), 200, 3000, 2048, 0)
        .await
        .unwrap();

    // Aggregate summary
    let summary = svc.aggregate_cost_summary().await.unwrap();
    assert!(!summary.is_empty());

    let scope_row = summary.iter().find(|r| r.scope == "scope-cost");
    assert!(scope_row.is_some());
    let row = scope_row.unwrap();
    assert_eq!(row.total_token_cost, Some(300)); // 100 + 200
    assert_eq!(row.total_build_time_ms, Some(8000)); // 5000 + 3000
    assert_eq!(row.total_storage_bytes, Some(3072)); // 1024 + 2048
    assert_eq!(row.total_rework_count, Some(1)); // 1 + 0
    assert_eq!(row.record_count, Some(2));
}

#[tokio::test]
async fn test_build_watch_dashboard() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let pool = get_pool(&store);
    let svc = BuildWatchService::new(store);

    // Clean up stale data from previous runs
    clear_scope_data(&pool, "dash-scope").await;

    // Record some events and a cost record
    svc.record_event(
        "dash-scope".to_string(),
        WatchEventType::BuildFailure,
        WatchSeverity::Critical,
        "build pipeline failed".to_string(),
        None,
    )
    .await
    .unwrap();

    svc.record_event(
        "dash-scope".to_string(),
        WatchEventType::TestFailure,
        WatchSeverity::Violation,
        "3 tests failed".to_string(),
        None,
    )
    .await
    .unwrap();

    svc.get_cost_summary("dash-scope".to_string(), 500, 10000, 4096, 2)
        .await
        .unwrap();

    // Dashboard combines event count + cost summary
    let dashboard = svc.get_watch_dashboard().await.unwrap();
    assert!(dashboard.total_events >= 2);
    assert!(!dashboard.cost_by_scope.is_empty());

    let scope_row = dashboard
        .cost_by_scope
        .iter()
        .find(|r| r.scope == "dash-scope");
    assert!(scope_row.is_some());
    assert_eq!(scope_row.unwrap().total_token_cost, Some(500));
}

#[tokio::test]
async fn test_build_watch_query_with_severity_filter() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BuildWatchService::new(store);

    // Record one info event and one violation
    svc.record_event(
        "filter-scope".to_string(),
        WatchEventType::PacketSubmission,
        WatchSeverity::Info,
        "packet submitted".to_string(),
        None,
    )
    .await
    .unwrap();

    svc.record_event(
        "filter-scope".to_string(),
        WatchEventType::PolicyViolation,
        WatchSeverity::Violation,
        "policy violated".to_string(),
        Some("ev-policy".to_string()),
    )
    .await
    .unwrap();

    // Query only violations
    let violations = svc
        .query_watch_events(None, None, Some("violation".to_string()), 10, 0)
        .await
        .unwrap();
    assert!(!violations.is_empty());
    assert!(violations
        .iter()
        .all(|e| format!("{}", e.severity) == "violation"));
}

#[tokio::test]
async fn test_build_watch_violation_detection() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BuildWatchService::new(store);

    // Record a violation that simulates Build Watch detecting
    // an unauthorized file mutation and recording it as a violation
    svc.record_event(
        "detect-scope".to_string(),
        WatchEventType::FileMutation,
        WatchSeverity::Warning,
        "unauthorized file: /etc/config.yaml".to_string(),
        None,
    )
    .await
    .unwrap();

    // Detect and record the violation
    let violation = svc
        .detect_violation(
            "watch-rule-file-mutation".to_string(),
            WatchSeverity::Violation,
            "unauthorized file mutation in scope detect-scope".to_string(),
            Some("ev-detect-scope".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(violation.rule, "watch-rule-file-mutation");
    assert_eq!(format!("{}", violation.severity), "violation");
}
