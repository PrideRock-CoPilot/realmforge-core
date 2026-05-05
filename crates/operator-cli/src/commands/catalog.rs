use authority_domain::{ActorId, CatalogId, CatalogScope};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Catalog management commands.
#[derive(Subcommand)]
pub enum CatalogCommand {
    List(ListArgs),
    Copy(CopyArgs),
    Provenance(ProvenanceArgs),
}

#[derive(Args)]
pub struct ListArgs {
    pub scope: String,
    #[arg(long)]
    pub scope_id: Option<String>,
}

#[derive(Args)]
pub struct CopyArgs {
    pub source_id: CatalogId,
    pub new_id: CatalogId,
    #[arg(long)]
    pub new_name: String,
    #[arg(long)]
    pub new_parent_id: Option<CatalogId>,
    #[arg(long)]
    pub target_scope: String,
    #[arg(long)]
    pub target_scope_id: Option<String>,
    #[arg(long)]
    pub copied_by: ActorId,
}

#[derive(Args)]
pub struct ProvenanceArgs {
    pub id: CatalogId,
}

pub async fn handle_catalog(cmd: CatalogCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        CatalogCommand::List(args) => {
            let scope = parse_scope(&args.scope, &args.scope_id)?;
            let entries = ctx.catalog.list_modules(&scope).await?;
            let result: Vec<serde_json::Value> = entries
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
            output_json(&result);
        }
        CatalogCommand::Copy(args) => {
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
            output_json(&json!({
                "id": entry.id.as_str(),
                "name": entry.name,
                "scope": entry.scope.to_string(),
                "provenance": entry.provenance.is_some(),
            }));
        }
        CatalogCommand::Provenance(args) => {
            let provenance = ctx.catalog.get_module_provenance(&args.id).await?;
            output_json(&json!({
                "source_scope": provenance.source_scope.to_string(),
                "source_id": provenance.source_id.as_str(),
                "copied_at": provenance.copied_at,
                "copied_by": provenance.copied_by.as_str(),
            }));
        }
    }
    Ok(())
}

fn parse_scope(scope_str: &str, scope_id: &Option<String>) -> Result<CatalogScope, String> {
    match scope_str {
        "global" => Ok(CatalogScope::Global),
        "tenant" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| "scope_id required for tenant scope".to_string())?;
            Ok(CatalogScope::Tenant(id))
        }
        "app" => {
            let id = scope_id
                .clone()
                .ok_or_else(|| "scope_id required for app scope".to_string())?;
            Ok(CatalogScope::App(id))
        }
        other => Err(format!("invalid scope: {other}")),
    }
}
