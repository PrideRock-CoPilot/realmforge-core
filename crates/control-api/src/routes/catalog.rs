use authority_domain::{ActorId, CatalogId, CatalogScope};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use control_service::ServiceContext;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::error::ApiError;

/// GET /v1/catalog
#[utoipa::path(
    get,
    path = "/v1/catalog",
    operation_id = "list_catalog",
    summary = "List catalog entries at a given scope (global, tenant, or app).",
    tag = "catalog",
    params(ListCatalogParams),
    responses(
        (status = 200, description = "Catalog entries", body = Vec<CatalogEntryResponse>),
        (status = 400, description = "Invalid scope", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn list_catalog(
    State(ctx): State<ServiceContext>,
    Query(params): Query<ListCatalogParams>,
) -> Result<Json<Vec<CatalogEntryResponse>>, ApiError> {
    let scope = parse_scope(&params.scope, &params.scope_id)?;
    let entries = ctx.catalog.list_modules(&scope).await?;
    Ok(Json(entries.into_iter().map(|e| e.into()).collect()))
}

/// POST /v1/catalog/copy
#[utoipa::path(
    post,
    path = "/v1/catalog/copy",
    operation_id = "copy_catalog",
    summary = "Copy a catalog module to a new scope, recording provenance.",
    tag = "catalog",
    request_body = CopyCatalogRequest,
    responses(
        (status = 201, description = "Catalog entry copied", body = CatalogEntryResponse),
        (status = 400, description = "Invalid request", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn copy_catalog(
    State(ctx): State<ServiceContext>,
    Json(req): Json<CopyCatalogRequest>,
) -> Result<(StatusCode, Json<CatalogEntryResponse>), ApiError> {
    let target_scope = parse_scope(&req.target_scope, &req.target_scope_id)?;
    let entry = ctx
        .catalog
        .copy_module(
            &req.source_id,
            &req.new_id,
            &req.new_name,
            req.new_parent_id.as_ref(),
            &target_scope,
            &req.copied_by,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(entry.into())))
}

/// GET /v1/catalog/:id/provenance
#[utoipa::path(
    get,
    path = "/v1/catalog/{id}/provenance",
    operation_id = "get_catalog_provenance",
    summary = "Get the copy provenance chain for a catalog entry.",
    tag = "catalog",
    params(("id" = String, Path, description = "Catalog entry ID")),
    responses(
        (status = 200, description = "Provenance record", body = ProvenanceResponse),
        (status = 404, description = "Entry not found", body = crate::error::ProblemDetails),
        (status = 500, description = "Internal error", body = crate::error::ProblemDetails),
    )
)]
pub async fn get_catalog_provenance(
    State(ctx): State<ServiceContext>,
    Path(id): Path<CatalogId>,
) -> Result<Json<ProvenanceResponse>, ApiError> {
    let provenance = ctx.catalog.get_module_provenance(&id).await?;
    Ok(Json(ProvenanceResponse {
        source_scope: provenance.source_scope.to_string(),
        source_id: provenance.source_id.to_string(),
        copied_at: provenance.copied_at,
        copied_by: provenance.copied_by.to_string(),
    }))
}

// ── Request/Response types ──

#[derive(Debug, Deserialize, utoipa::ToSchema, IntoParams)]
pub struct ListCatalogParams {
    /// Scope level: "global", "tenant", or "app".
    #[param(example = "tenant")]
    pub scope: String,
    #[param(example = "ten_01")]
    pub scope_id: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CopyCatalogRequest {
    #[schema(value_type = String, example = "cat_01")]
    pub source_id: CatalogId,
    #[schema(value_type = String, example = "cat_02")]
    pub new_id: CatalogId,
    #[schema(example = "my-module-copy")]
    pub new_name: String,
    #[schema(value_type = Option<String>)]
    pub new_parent_id: Option<CatalogId>,
    #[schema(example = "tenant")]
    pub target_scope: String,
    #[schema(example = "ten_02")]
    pub target_scope_id: Option<String>,
    #[schema(value_type = String, example = "alice")]
    pub copied_by: ActorId,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CatalogEntryResponse {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub module_type: String,
    pub has_provenance: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProvenanceResponse {
    pub source_scope: String,
    pub source_id: String,
    pub copied_at: chrono::DateTime<chrono::Utc>,
    pub copied_by: String,
}

impl From<authority_domain::CatalogEntry> for CatalogEntryResponse {
    fn from(e: authority_domain::CatalogEntry) -> Self {
        Self {
            id: e.id.to_string(),
            name: e.name,
            scope: e.scope.to_string(),
            module_type: e.module_type.to_string(),
            has_provenance: e.provenance.is_some(),
        }
    }
}

/// Parse a scope string + optional scope_id into a CatalogScope.
fn parse_scope(scope_str: &str, scope_id: &Option<String>) -> Result<CatalogScope, ApiError> {
    match scope_str {
        "global" => Ok(CatalogScope::Global),
        "tenant" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| ApiError::BadRequest("scope_id required for tenant scope".into()))?;
            Ok(CatalogScope::Tenant(id))
        }
        "app" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| ApiError::BadRequest("scope_id required for app scope".into()))?;
            Ok(CatalogScope::App(id))
        }
        other => Err(ApiError::BadRequest(format!("invalid scope: {other}"))),
    }
}
