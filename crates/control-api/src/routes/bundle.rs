use authority_domain::{BundleId, BundleStatus};
use axum::{extract::Path, extract::State, Json};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::IntoParams;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateBundleRequest {
    #[schema(example = "bundle_01")]
    pub bundle_id: String,
    #[schema(example = "1.0.0")]
    pub version: String,
    #[schema(example = "app_01")]
    pub app_id: String,
    #[schema(value_type = Vec<[String; 2]>)]
    pub artifact_hashes: Vec<(String, String)>,
    #[schema(example = "sig_abc123")]
    pub governance_signature: String,
    pub release_approval_ref: Option<String>,
}

/// POST /v1/bundles
#[utoipa::path(
    post,
    path = "/v1/bundles",
    operation_id = "create_bundle",
    summary = "Create a new bundle manifest with artifact hashes and governance signature.",
    tag = "bundles",
    request_body = CreateBundleRequest,
    responses(
        (status = 200, description = "Bundle recorded"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn create_bundle(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CreateBundleRequest>,
) -> Json<Value> {
    let bundle_id = BundleId::new(body.bundle_id).unwrap_or_else(|_| BundleId::generate());
    let manifest = authority_domain::BundleManifest {
        bundle_id,
        version: body.version,
        app_id: body.app_id,
        artifact_hashes: body.artifact_hashes,
        governance_signature: body.governance_signature,
        release_approval_ref: body
            .release_approval_ref
            .map(authority_domain::ApprovalId::new)
            .transpose()
            .unwrap_or(None),
        built_at: chrono::Utc::now(),
        status: BundleStatus::Building,
    };

    match ctx.bundles.record_bundle(&manifest).await {
        Ok(()) => Json(json!({
            "success": true,
            "data": manifest
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

#[derive(Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ListBundlesQuery {
    #[param(example = "app_01")]
    pub app_id: Option<String>,
    #[param(example = 20)]
    pub limit: Option<i64>,
    #[param(example = 0)]
    pub offset: Option<i64>,
}

/// GET /v1/bundles
#[utoipa::path(
    get,
    path = "/v1/bundles",
    operation_id = "list_bundles",
    summary = "List bundle manifests, optionally filtered by app ID.",
    tag = "bundles",
    params(ListBundlesQuery),
    responses(
        (status = 200, description = "Bundle list"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn list_bundles(
    State(ctx): State<ServiceContext>,
    axum::extract::Query(query): axum::extract::Query<ListBundlesQuery>,
) -> Json<Value> {
    let app_id = query.app_id.as_deref().unwrap_or("");
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    match ctx.bundles.list_bundles(app_id, limit, offset).await {
        Ok(bundles) => Json(json!({
            "success": true,
            "data": bundles
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// GET /v1/bundles/:id
#[utoipa::path(
    get,
    path = "/v1/bundles/{id}",
    operation_id = "get_bundle",
    summary = "Get a bundle manifest by ID.",
    tag = "bundles",
    params(("id" = String, Path, description = "Bundle ID")),
    responses(
        (status = 200, description = "Bundle manifest"),
        (status = 404, description = "Bundle not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn get_bundle(State(ctx): State<ServiceContext>, Path(id): Path<String>) -> Json<Value> {
    let bundle_id = BundleId::new(id).unwrap_or_else(|_| BundleId::generate());

    match ctx.bundles.get_bundle(&bundle_id).await {
        Ok(manifest) => Json(json!({
            "success": true,
            "data": manifest
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct VerifyBundleRequest {
    /// One of: "verified", "signed", "failed".
    #[schema(example = "verified")]
    pub status: String,
}

/// POST /v1/bundles/:id/verify
#[utoipa::path(
    post,
    path = "/v1/bundles/{id}/verify",
    operation_id = "verify_bundle",
    summary = "Update bundle status to verified, signed, or failed.",
    tag = "bundles",
    params(("id" = String, Path, description = "Bundle ID")),
    request_body = VerifyBundleRequest,
    responses(
        (status = 200, description = "Bundle status updated"),
        (status = 404, description = "Bundle not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn verify_bundle(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
    Json(body): Json<VerifyBundleRequest>,
) -> Json<Value> {
    let bundle_id = BundleId::new(id).unwrap_or_else(|_| BundleId::generate());
    let status = match body.status.as_str() {
        "verified" => BundleStatus::Verified,
        "signed" => BundleStatus::Signed,
        "failed" => BundleStatus::Failed(body.status.clone()),
        other => BundleStatus::Failed(format!("unknown status: {other}")),
    };

    match ctx.bundles.update_bundle_status(&bundle_id, &status).await {
        Ok(()) => Json(json!({
            "success": true,
            "data": { "id": bundle_id.to_string(), "status": format!("{:?}", status) }
        })),
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

/// POST /v1/bundles/:id/deploy
#[utoipa::path(
    post,
    path = "/v1/bundles/{id}/deploy",
    operation_id = "deploy_bundle",
    summary = "Mark a verified bundle as deployed.",
    tag = "bundles",
    params(("id" = String, Path, description = "Bundle ID")),
    responses(
        (status = 200, description = "Bundle marked deployed"),
        (status = 404, description = "Bundle not found"),
        (status = 500, description = "Internal error"),
    )
)]
pub async fn deploy_bundle(
    State(ctx): State<ServiceContext>,
    Path(id): Path<String>,
) -> Json<Value> {
    let bundle_id = BundleId::new(id).unwrap_or_else(|_| BundleId::generate());

    match ctx.bundles.get_bundle(&bundle_id).await {
        Ok(manifest) => {
            let status = BundleStatus::Deployed;
            match ctx.bundles.update_bundle_status(&bundle_id, &status).await {
                Ok(()) => Json(json!({
                    "success": true,
                    "data": { "id": bundle_id.to_string(), "status": "Deployed", "version": manifest.version, "app_id": manifest.app_id }
                })),
                Err(err) => Json(json!({
                    "success": false,
                    "error": err.to_string()
                })),
            }
        }
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}
