use axum::{extract::Path, extract::State, Json};
use control_service::ServiceContext;
use serde_json::Value;

/// GET /v1/knowledge/datasets
#[utoipa::path(
    get,
    path = "/v1/knowledge/datasets",
    operation_id = "list_datasets",
    summary = "List all indexed knowledge datasets.",
    tag = "knowledge",
    responses(
        (status = 200, description = "Dataset list"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_datasets(State(ctx): State<ServiceContext>) -> Json<Value> {
    match ctx.knowledge.reconcile_datasets().await {
        Ok(datasets) => Json(serde_json::json!({
            "success": true,
            "data": datasets
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// GET /v1/knowledge/datasets/:id
#[utoipa::path(
    get,
    path = "/v1/knowledge/datasets/{id}",
    operation_id = "get_dataset",
    summary = "Get metadata for a specific knowledge dataset.",
    tag = "knowledge",
    params(("id" = String, Path, description = "Dataset ID")),
    responses(
        (status = 200, description = "Dataset info"),
        (status = 404, description = "Dataset not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_dataset(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Json<Value> {
    match ctx.knowledge.get_dataset_info(&id).await {
        Ok(info) => Json(serde_json::json!({
            "success": true,
            "data": info
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// POST /v1/knowledge/query
#[utoipa::path(
    post,
    path = "/v1/knowledge/query",
    operation_id = "query_knowledge",
    summary = "Query knowledge records by scope, type, and text filters.",
    tag = "knowledge",
    request_body(content = Object, description = "KnowledgeQuery fields: scope, record_type, tags, text_contains, limit, offset"),
    responses(
        (status = 200, description = "Query results"),
        (status = 400, description = "Invalid query"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn query_knowledge(
    State(ctx): State<ServiceContext>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let query: authority_domain::KnowledgeQuery = match serde_json::from_value(body.clone()) {
        Ok(q) => q,
        Err(e) => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("invalid query: {e}")
            }));
        }
    };

    let allowed_scopes = vec![
        authority_domain::KnowledgeScope::Global,
        authority_domain::KnowledgeScope::Tenant,
        authority_domain::KnowledgeScope::App,
        authority_domain::KnowledgeScope::WorkPath,
    ];

    match ctx.knowledge.query_knowledge(&query, &allowed_scopes).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "data": result
        })),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}
