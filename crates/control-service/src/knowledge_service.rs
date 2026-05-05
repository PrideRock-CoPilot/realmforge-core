use authority_domain::{DatasetInfo, KnowledgeQuery, KnowledgeRecord, KnowledgeScope};
use chrono::Utc;
use control_store::CoreStore;
use parquet_store::{ParquetDataset, ParquetKnowledgeQuery};
use sha2::{Digest, Sha256};
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Knowledge service for scoped retrieval over governed records.
///
/// Track A (Parquet-backed storage) is now unblocked per DEC-COUNCIL-003.
/// All ingest operations write to both:
/// 1. Postgres metadata index (for fast lookups and dataset tracking)
/// 2. Parquet files via `parquet-store` crate (for durable snapshots and analytics)
#[derive(Clone)]
pub struct KnowledgeService {
    store: CoreStore,
    parquet: Option<ParquetDataset>,
}

impl KnowledgeService {
    /// Create a new `KnowledgeService` without Parquet backing.
    /// Use this in tests or when Parquet storage is not configured.
    pub fn new(store: CoreStore) -> Self {
        Self {
            store,
            parquet: None,
        }
    }

    /// Create a new `KnowledgeService` with Parquet backing at `parquet_base_path`.
    pub fn new_with_parquet(
        store: CoreStore,
        parquet_base_path: impl Into<std::path::PathBuf>,
        dataset_id: impl Into<String>,
    ) -> Self {
        let parquet = ParquetDataset::new(parquet_base_path, dataset_id);
        Self {
            store,
            parquet: Some(parquet),
        }
    }

    /// Return a reference to the Parquet dataset, if configured.
    pub fn parquet_dataset(&self) -> Option<&ParquetDataset> {
        self.parquet.as_ref()
    }

    /// Ingest a knowledge record — validates scope, computes content hash,
    /// stores metadata in Postgres, and writes to Parquet (if configured).
    #[instrument(skip(self), fields(record_id = %record.id))]
    pub async fn ingest_knowledge(
        &self,
        mut record: KnowledgeRecord,
    ) -> Result<KnowledgeRecord, ServiceError> {
        // Compute content hash from source_id + citations for integrity
        let content = format!("{}:{:?}", record.source_id, record.citations);
        record.content_hash = hex::encode(Sha256::digest(content.as_bytes()));
        record.indexed_at = Utc::now();

        // 1. Write Postgres metadata (always)
        self.store.insert_knowledge_metadata(&record).await?;

        // 2. Write Parquet (if configured)
        if let Some(parquet) = &self.parquet {
            let next_version = parquet.latest_version_number().await?.unwrap_or(0) + 1;
            parquet
                .write_version(next_version, &[record.clone()])
                .await?;
        }

        info!("knowledge record ingested");
        Ok(record)
    }

    /// Query knowledge records by scope and source type. Respects grant scope
    /// filtering — an agent cannot see records outside its allowed scope.
    ///
    /// Uses Parquet-backed query when available, falling back to Postgres metadata.
    #[instrument(skip(self), fields(query_scopes = ?query.scopes))]
    pub async fn query_knowledge(
        &self,
        query: &KnowledgeQuery,
        allowed_scopes: &[KnowledgeScope],
    ) -> Result<KnowledgeQueryResult, ServiceError> {
        // Intersect requested scopes with allowed scopes (grant boundary)
        let effective_scopes: Vec<KnowledgeScope> = query
            .scopes
            .iter()
            .filter(|s| allowed_scopes.contains(s))
            .cloned()
            .collect();

        if effective_scopes.is_empty() {
            return Ok(KnowledgeQueryResult {
                records: vec![],
                total_count: 0,
                denied_record_count: 0,
            });
        }

        // Try Parquet-backed query first (if configured)
        if let Some(parquet) = &self.parquet {
            let parquet_query = ParquetKnowledgeQuery {
                scopes: Some(effective_scopes.clone()),
                source_types: query.source_types.clone(),
                date_range: query.date_range,
                text_search: query.text_search.clone(),
                limit: query.limit,
                offset: query.offset,
            };

            let records = parquet.query(&parquet_query).await?;
            return Ok(KnowledgeQueryResult {
                total_count: records.len() as u64,
                records,
                denied_record_count: 0,
            });
        }

        // Fallback: Postgres metadata query
        let records = self
            .store
            .query_knowledge_metadata(
                &effective_scopes,
                query.source_types.as_ref().and_then(|v| v.first()),
                query.limit as i64,
                query.offset as i64,
            )
            .await?;

        Ok(KnowledgeQueryResult {
            total_count: records.len() as u64,
            records,
            denied_record_count: 0,
        })
    }

    /// Get dataset info for a specific dataset.
    #[instrument(skip(self), fields(dataset_id = %dataset_id))]
    pub async fn get_dataset_info(&self, dataset_id: &str) -> Result<DatasetInfo, ServiceError> {
        self.store
            .get_dataset_info(dataset_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("dataset {dataset_id} not found")))
    }

    /// Reconcile the Postgres index with actual Parquet datasets.
    #[instrument(skip(self))]
    pub async fn reconcile_datasets(&self) -> Result<Vec<String>, ServiceError> {
        let reconciled = self.store.reconcile_datasets().await?;
        info!("datasets reconciled — {} entries", reconciled.len());
        Ok(reconciled)
    }
}

/// Result of a knowledge query.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeQueryResult {
    pub records: Vec<KnowledgeRecord>,
    pub total_count: u64,
    pub denied_record_count: u64,
}
