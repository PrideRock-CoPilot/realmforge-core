use authority_domain::{KnowledgeId, KnowledgeQuery, KnowledgeRecord, KnowledgeScope, SourceType};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::instrument;

use crate::error::McpError;

/// Args for core_query_knowledge
#[derive(Clone, Debug, Deserialize)]
pub struct QueryKnowledgeArgs {
    pub scopes: Vec<String>,
    pub source_types: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Args for core_ingest_knowledge
#[derive(Clone, Debug, Deserialize)]
pub struct IngestKnowledgeArgs {
    pub id: Option<String>,
    pub source_id: String,
    pub scope: String,
    pub source_type: String,
}

/// Query knowledge with scope filtering — returns ranked results with citations.
#[instrument(skip(ctx), fields(query_scopes = ?args.scopes))]
pub async fn core_query_knowledge(
    args: QueryKnowledgeArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let parsed_scopes: Vec<KnowledgeScope> = args
        .scopes
        .iter()
        .filter_map(|s| match s.as_str() {
            "global" => Some(KnowledgeScope::Global),
            "tenant" => Some(KnowledgeScope::Tenant),
            "app" => Some(KnowledgeScope::App),
            "work_path" => Some(KnowledgeScope::WorkPath),
            _ => None,
        })
        .collect();

    if parsed_scopes.is_empty() {
        return Err(McpError::InvalidArgs(
            "at least one valid scope required: global, tenant, app, work_path".to_string(),
        ));
    }

    let parsed_st = args.source_types.as_ref().map(|v| {
        v.iter()
            .filter_map(|s| match s.as_str() {
                "catalog" => Some(SourceType::Catalog),
                "file" => Some(SourceType::File),
                "decision" => Some(SourceType::Decision),
                "evidence" => Some(SourceType::Evidence),
                "trace" => Some(SourceType::Trace),
                _ => None,
            })
            .collect()
    });

    let query = KnowledgeQuery {
        scopes: parsed_scopes,
        source_types: parsed_st,
        date_range: None,
        text_search: None,
        limit: args.limit.unwrap_or(10),
        offset: args.offset.unwrap_or(0),
    };

    let allowed = vec![
        KnowledgeScope::Global,
        KnowledgeScope::Tenant,
        KnowledgeScope::App,
        KnowledgeScope::WorkPath,
    ];

    let result = ctx.knowledge.query_knowledge(&query, &allowed).await?;

    Ok(json!({
        "records": result.records,
        "total_count": result.total_count,
        "denied_record_count": result.denied_record_count
    }))
}

/// Batch ingest knowledge records with validation.
#[instrument(skip(ctx), fields(source_id = %args.source_id))]
pub async fn core_ingest_knowledge(
    args: IngestKnowledgeArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let scope = match args.scope.as_str() {
        "global" => KnowledgeScope::Global,
        "tenant" => KnowledgeScope::Tenant,
        "app" => KnowledgeScope::App,
        "work_path" => KnowledgeScope::WorkPath,
        _ => {
            return Err(McpError::InvalidArgs(format!(
                "invalid scope: {} — use: global, tenant, app, work_path",
                args.scope
            )))
        }
    };

    let source_type = match args.source_type.as_str() {
        "catalog" => SourceType::Catalog,
        "file" => SourceType::File,
        "decision" => SourceType::Decision,
        "evidence" => SourceType::Evidence,
        "trace" => SourceType::Trace,
        _ => {
            return Err(McpError::InvalidArgs(format!(
                "invalid source type: {} — use: catalog, file, decision, evidence, trace",
                args.source_type
            )))
        }
    };

    let record = KnowledgeRecord {
        id: match args.id {
            Some(v) => KnowledgeId::new(&v)
                .map_err(|e| McpError::InvalidArgs(format!("invalid id: {e}")))?,
            None => KnowledgeId::generate(),
        },
        scope,
        source_type,
        source_id: args.source_id,
        content_hash: String::new(),
        indexed_at: chrono::Utc::now(),
        citations: vec![],
    };

    let result = ctx.knowledge.ingest_knowledge(record).await?;

    Ok(json!({
        "id": result.id,
        "scope": result.scope,
        "source_type": result.source_type,
        "source_id": result.source_id,
        "content_hash": result.content_hash,
        "indexed_at": result.indexed_at
    }))
}
