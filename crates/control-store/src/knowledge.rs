use crate::StoreError;
use authority_domain::{DatasetInfo, KnowledgeId, KnowledgeRecord, KnowledgeScope, SourceType};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

/// Insert metadata for a knowledge record into Postgres.
pub async fn insert_knowledge_metadata(
    pool: &PgPool,
    record: &KnowledgeRecord,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO knowledge_datasets (id, scope, source_type, source_id, content_hash, indexed_at, citations) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(record.id.as_str())
    .bind(format!("{}", record.scope))
    .bind(format!("{}", record.source_type))
    .bind(&record.source_id)
    .bind(&record.content_hash)
    .bind(record.indexed_at)
    .bind(serde_json::to_value(&record.citations)?)
    .execute(pool)
    .await?;
    Ok(())
}

/// Query knowledge metadata by scope and source type.
pub async fn query_knowledge_metadata(
    pool: &PgPool,
    scopes: &[KnowledgeScope],
    source_type: Option<&SourceType>,
    limit: i64,
    offset: i64,
) -> Result<Vec<KnowledgeRecord>, StoreError> {
    let scope_strs: Vec<String> = scopes.iter().map(|s| format!("{}", s)).collect();
    let source_str = source_type.map(|s| format!("{}", s));

    let rows: Vec<KnowledgeRaw> = if let Some(ref st) = source_str {
        sqlx::query_as(
            "SELECT id, scope, source_type, source_id, content_hash, indexed_at, citations \
             FROM knowledge_datasets \
             WHERE scope = ANY($1) AND source_type = $2 \
             ORDER BY indexed_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(&scope_strs)
        .bind(st)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, scope, source_type, source_id, content_hash, indexed_at, citations \
             FROM knowledge_datasets \
             WHERE scope = ANY($1) \
             ORDER BY indexed_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&scope_strs)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    rows.into_iter().map(|r| r.try_into()).collect()
}

/// Get dataset info for a specific dataset.
pub async fn get_dataset_info(
    pool: &PgPool,
    dataset_id: &str,
) -> Result<Option<DatasetInfo>, StoreError> {
    let row: Option<DatasetInfoRaw> = sqlx::query_as(
        "SELECT dataset_id, source_type, schema_version, record_count, created_at, last_indexed_at \
         FROM knowledge_datasets WHERE id = $1",
    )
    .bind(dataset_id)
    .fetch_optional(pool)
    .await?;

    row.map(|r| r.try_into()).transpose()
}

/// Reconcile the Postgres index with the actual dataset files (stub — full impl needs Parquet crate).
pub async fn reconcile_datasets(pool: &PgPool) -> Result<Vec<String>, StoreError> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, source_type FROM knowledge_datasets ORDER BY indexed_at")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .into_iter()
        .map(|(id, st)| format!("{id} ({st})"))
        .collect())
}

// ── Raw row types ──

#[derive(sqlx::FromRow)]
struct KnowledgeRaw {
    id: String,
    scope: String,
    source_type: String,
    source_id: String,
    content_hash: String,
    indexed_at: DateTime<Utc>,
    citations: Value,
}

impl TryInto<KnowledgeRecord> for KnowledgeRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<KnowledgeRecord, Self::Error> {
        let scope = parse_scope(&self.scope)?;
        let source_type = parse_source_type(&self.source_type)?;
        Ok(KnowledgeRecord {
            id: KnowledgeId::new(self.id).map_err(|e| StoreError::invalid_data(e.to_string()))?,
            scope,
            source_type,
            source_id: self.source_id,
            content_hash: self.content_hash,
            indexed_at: self.indexed_at,
            citations: serde_json::from_value(self.citations)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct DatasetInfoRaw {
    dataset_id: String,
    source_type: String,
    schema_version: i32,
    record_count: i64,
    created_at: DateTime<Utc>,
    last_indexed_at: DateTime<Utc>,
}

impl TryInto<DatasetInfo> for DatasetInfoRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<DatasetInfo, Self::Error> {
        let source_type = parse_source_type(&self.source_type)?;
        Ok(DatasetInfo {
            dataset_id: self.dataset_id,
            source_type,
            schema_version: self.schema_version as u32,
            record_count: self.record_count as u64,
            created_at: self.created_at,
            last_indexed_at: self.last_indexed_at,
        })
    }
}

fn parse_scope(s: &str) -> Result<KnowledgeScope, StoreError> {
    match s {
        "global" => Ok(KnowledgeScope::Global),
        "tenant" => Ok(KnowledgeScope::Tenant),
        "app" => Ok(KnowledgeScope::App),
        "work_path" => Ok(KnowledgeScope::WorkPath),
        _ => Err(StoreError::invalid_data(format!("unknown scope: {s}"))),
    }
}

fn parse_source_type(s: &str) -> Result<SourceType, StoreError> {
    match s {
        "catalog" => Ok(SourceType::Catalog),
        "file" => Ok(SourceType::File),
        "decision" => Ok(SourceType::Decision),
        "evidence" => Ok(SourceType::Evidence),
        "trace" => Ok(SourceType::Trace),
        _ => Err(StoreError::invalid_data(format!(
            "unknown source type: {s}"
        ))),
    }
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert metadata for a knowledge record into Postgres.
    #[tracing::instrument(skip(self))]
    pub async fn insert_knowledge_metadata(
        &self,
        record: &KnowledgeRecord,
    ) -> Result<(), StoreError> {
        insert_knowledge_metadata(&self.pool, record).await
    }

    /// Query knowledge metadata by scope and source type.
    #[tracing::instrument(skip(self))]
    pub async fn query_knowledge_metadata(
        &self,
        scopes: &[KnowledgeScope],
        source_type: Option<&SourceType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<KnowledgeRecord>, StoreError> {
        query_knowledge_metadata(&self.pool, scopes, source_type, limit, offset).await
    }

    /// Get dataset info for a specific dataset.
    #[tracing::instrument(skip(self))]
    pub async fn get_dataset_info(
        &self,
        dataset_id: &str,
    ) -> Result<Option<DatasetInfo>, StoreError> {
        get_dataset_info(&self.pool, dataset_id).await
    }

    /// Reconcile the Postgres index with the actual dataset files.
    #[tracing::instrument(skip(self))]
    pub async fn reconcile_datasets(&self) -> Result<Vec<String>, StoreError> {
        reconcile_datasets(&self.pool).await
    }
}
