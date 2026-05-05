use authority_domain::{
    KnowledgeId, KnowledgeQuery, KnowledgeRecord, KnowledgeScope, SourceType,
};
use chrono::Utc;
use clap::Subcommand;
use control_service::ServiceContext;

use crate::util::CliResult;

/// Knowledge lifecycle management commands.
#[derive(Subcommand)]
pub enum KnowledgeCommand {
    /// Ingest a knowledge record
    Ingest {
        /// Record ID (optional — auto-generated if omitted)
        #[arg(long)]
        id: Option<String>,
        /// Source ID
        #[arg(long)]
        source_id: String,
        /// Scope (global, tenant, app, work_path)
        #[arg(long, default_value = "global")]
        scope: String,
        /// Source type (catalog, file, decision, evidence, trace)
        #[arg(long)]
        source_type: String,
    },
    /// Query knowledge records
    Query {
        /// Scopes (comma-separated)
        #[arg(long, default_value = "global")]
        scopes: String,
        /// Source type filter
        #[arg(long)]
        source_type: Option<String>,
        /// Limit
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// List all datasets
    Datasets,
    /// Reconcile dataset index
    Reconcile,
}

pub async fn handle_knowledge(
    cmd: KnowledgeCommand,
    ctx: &ServiceContext,
) -> CliResult {
    match cmd {
        KnowledgeCommand::Ingest {
            id,
            source_id,
            scope,
            source_type,
        } => ingest_knowledge(id, source_id, scope, source_type, ctx).await,
        KnowledgeCommand::Query {
            scopes,
            source_type,
            limit,
        } => query_knowledge(scopes, source_type, limit, ctx).await,
        KnowledgeCommand::Datasets => list_datasets(ctx).await,
        KnowledgeCommand::Reconcile => reconcile_datasets(ctx).await,
    }
}

async fn ingest_knowledge(
    id: Option<String>,
    source_id: String,
    scope: String,
    source_type: String,
    ctx: &ServiceContext,
) -> CliResult {
    let parsed_scope = match scope.as_str() {
        "global" => KnowledgeScope::Global,
        "tenant" => KnowledgeScope::Tenant,
        "app" => KnowledgeScope::App,
        "work_path" => KnowledgeScope::WorkPath,
        _ => return Err(format!("invalid scope: {scope} — use: global, tenant, app, work_path").into()),
    };

    let parsed_st = match source_type.as_str() {
        "catalog" => SourceType::Catalog,
        "file" => SourceType::File,
        "decision" => SourceType::Decision,
        "evidence" => SourceType::Evidence,
        "trace" => SourceType::Trace,
        _ => return Err(format!("invalid source type: {source_type} — use: catalog, file, decision, evidence, trace").into()),
    };

    let record = KnowledgeRecord {
        id: match id {
            Some(v) => KnowledgeId::new(v).map_err(|e| format!("invalid id: {e}"))?,
            None => KnowledgeId::generate(),
        },
        scope: parsed_scope,
        source_type: parsed_st,
        source_id,
        content_hash: String::new(),
        indexed_at: Utc::now(),
        citations: vec![],
    };

    let result = ctx.knowledge.ingest_knowledge(record).await?;
    println!("Knowledge record ingested:");
    println!("  ID: {}", result.id);
    println!("  Scope: {}", result.scope);
    println!("  Source: {} ({})", result.source_type, result.source_id);
    println!("  Hash: {}", result.content_hash);
    Ok(())
}

async fn query_knowledge(
    scopes: String,
    source_type: Option<String>,
    limit: u32,
    ctx: &ServiceContext,
) -> CliResult {
    let parsed_scopes: Vec<KnowledgeScope> = scopes
        .split(',')
        .filter_map(|s| match s.trim() {
            "global" => Some(KnowledgeScope::Global),
            "tenant" => Some(KnowledgeScope::Tenant),
            "app" => Some(KnowledgeScope::App),
            "work_path" => Some(KnowledgeScope::WorkPath),
            _ => None,
        })
        .collect();

    if parsed_scopes.is_empty() {
        return Err("at least one valid scope required".to_string().into());
    }

    let parsed_st = source_type.as_deref().and_then(|s| match s {
        "catalog" => Some(SourceType::Catalog),
        "file" => Some(SourceType::File),
        "decision" => Some(SourceType::Decision),
        "evidence" => Some(SourceType::Evidence),
        "trace" => Some(SourceType::Trace),
        _ => None,
    });

    let query = KnowledgeQuery {
        scopes: parsed_scopes,
        source_types: parsed_st.map(|st| vec![st]),
        date_range: None,
        text_search: None,
        limit,
        offset: 0,
    };

    let allowed = vec![
        KnowledgeScope::Global,
        KnowledgeScope::Tenant,
        KnowledgeScope::App,
        KnowledgeScope::WorkPath,
    ];

    let result = ctx.knowledge.query_knowledge(&query, &allowed).await?;

    if result.records.is_empty() {
        println!("No knowledge records found");
        return Ok(());
    }

    println!("Knowledge records ({} total, {} denied):", result.total_count, result.denied_record_count);
    for record in &result.records {
        println!("  {} — scope: {}, type: {}", record.id, record.scope, record.source_type);
        println!("    Source: {}", record.source_id);
        println!("    Citations: {}", record.citations.len());
    }
    Ok(())
}

async fn list_datasets(ctx: &ServiceContext) -> CliResult {
    let datasets = ctx.knowledge.reconcile_datasets().await?;
    if datasets.is_empty() {
        println!("No datasets found");
        return Ok(());
    }
    println!("Knowledge datasets:");
    for ds in &datasets {
        println!("  {ds}");
    }
    Ok(())
}

async fn reconcile_datasets(ctx: &ServiceContext) -> CliResult {
    let reconciled = ctx.knowledge.reconcile_datasets().await?;
    println!("Reconciled {} datasets", reconciled.len());
    for ds in &reconciled {
        println!("  {ds}");
    }
    Ok(())
}
