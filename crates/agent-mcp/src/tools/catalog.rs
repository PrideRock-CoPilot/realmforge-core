use authority_domain::CatalogScope;
use control_service::ServiceContext;
use serde_json::{json, Value};

use crate::error::McpError;
use crate::types::{CopyCatalogModuleArgs, ListCatalogModulesArgs};

/// core_list_catalog_modules — List catalog modules at a given scope.
pub async fn core_list_catalog_modules(
    args: ListCatalogModulesArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let scope = parse_scope(&args.scope, &args.scope_id)?;
    let entries = ctx.catalog.list_modules(&scope).await?;
    let result: Vec<Value> = entries
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id.as_str(),
                "name": e.name,
                "scope": e.scope.to_string(),
                "module_type": e.module_type.to_string(),
                "has_provenance": e.provenance.is_some(),
            })
        })
        .collect();
    Ok(serde_json::to_value(result)?)
}

/// core_copy_catalog_module — Copy a catalog module with provenance.
pub async fn core_copy_catalog_module(
    args: CopyCatalogModuleArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let target_scope = parse_scope(&args.target_scope, &args.target_scope_id)?;
    let entry = ctx
        .catalog
        .copy_module(
            &args.source_id,
            &args.new_id,
            &args.new_name,
            args.new_parent_id.as_ref(),
            &target_scope,
            &args.copied_by,
        )
        .await?;
    Ok(json!({
        "id": entry.id.as_str(),
        "name": entry.name,
        "scope": entry.scope.to_string(),
        "provenance": entry.provenance.is_some(),
    }))
}

fn parse_scope(scope_str: &str, scope_id: &Option<String>) -> Result<CatalogScope, McpError> {
    match scope_str {
        "global" => Ok(CatalogScope::Global),
        "tenant" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| McpError::InvalidArgs("scope_id required for tenant scope".into()))?;
            Ok(CatalogScope::Tenant(id))
        }
        "app" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| McpError::InvalidArgs("scope_id required for app scope".into()))?;
            Ok(CatalogScope::App(id))
        }
        other => Err(McpError::InvalidArgs(format!("invalid scope: {other}"))),
    }
}
