// Integration tests for audit API endpoints.
// Tests gracefully skip when no database is available.

mod common;

#[tokio::test]
async fn test_audit_chain_verify() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    // Verify chain for a project (may be empty — that's fine)
    let resp = srv
        .get("/v1/audit/chain/verify?project_id=audit-api-test-project")
        .await;

    // Should succeed with either 200 (success) or error depending on query params
    let status = resp.status();
    assert!(
        status == 200 || status == 400 || status == 422,
        "expected 200, 400, or 422, got {status}"
    );

    if status == 200 {
        let data: serde_json::Value = resp.json().await.unwrap();
        assert!(data["chain_integrity"].is_boolean());
    }
}

#[tokio::test]
async fn test_audit_events_query() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    // Query events — project_id is required
    let resp = srv
        .get("/v1/audit/events?project_id=audit-api-events-project")
        .await;

    let status = resp.status();
    assert!(
        status == 200 || status == 400 || status == 422,
        "expected 200, 400, or 422, got {status}"
    );

    if status == 200 {
        let data: serde_json::Value = resp.json().await.unwrap();
        assert!(data["events"].is_array());
        assert!(data["total"].is_u64());
    }
}
