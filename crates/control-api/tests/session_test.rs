// Integration tests for session API endpoints.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{ActorId, ProjectId, TenantId};
use serde_json::json;

#[tokio::test]
async fn test_session_issue_and_get() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    // Issue a session
    let body = json!({
        "actor_id": ActorId::new("sess-api-test-actor").unwrap().as_str(),
        "tenant_id": TenantId::new("sess-api-test-tenant").unwrap().as_str(),
        "project_id": ProjectId::new("sess-api-test-project").unwrap().as_str(),
        "ttl_seconds": 3600,
    });

    let resp = srv.post("/v1/session", &body).await;
    assert_eq!(resp.status(), 201);

    let data: serde_json::Value = resp.json().await.unwrap();
    assert!(data["session"]["id"].is_string());
    let session_id = data["session"]["id"].as_str().unwrap().to_string();

    // Get the session
    let resp = srv.get(&format!("/v1/session/{}", session_id)).await;
    assert_eq!(resp.status(), 200, "get session should succeed");

    let data: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(data["session"]["id"], session_id);
}

#[tokio::test]
async fn test_session_revoke() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    // Issue a session
    let body = json!({
        "actor_id": ActorId::new("sess-revoke-actor").unwrap().as_str(),
        "tenant_id": TenantId::new("sess-revoke-tenant").unwrap().as_str(),
        "project_id": ProjectId::new("sess-revoke-project").unwrap().as_str(),
        "ttl_seconds": 3600,
    });

    let resp = srv.post("/v1/session", &body).await;
    assert_eq!(resp.status(), 201);

    let data: serde_json::Value = resp.json().await.unwrap();
    let session_id = data["session"]["id"].as_str().unwrap().to_string();

    // Revoke
    let resp = srv
        .client
        .delete(format!("{}/v1/session/{}", srv.base_url, session_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let data: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(data["status"], "revoked");
}
