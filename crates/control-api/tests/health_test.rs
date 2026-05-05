// Integration tests for API health endpoints.
// Tests gracefully skip when no database is available.

mod common;

#[tokio::test]
async fn test_health_returns_ok() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    let resp = srv.get("/health").await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "control-api");
}

#[tokio::test]
async fn test_health_ready_returns_ok() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    let resp = srv.get("/v1/health/ready").await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_health_live_returns_ok() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };
    let srv = common::TestServer::new(store).await;

    let resp = srv.get("/v1/health/live").await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}
