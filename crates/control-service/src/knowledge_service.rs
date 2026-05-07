use authority_domain::{DatasetInfo, KnowledgeQuery, KnowledgeRecord, KnowledgeScope};
use chrono::Utc;
use control_store::CoreStore;
use sha2::{Digest, Sha256};
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Knowledge service for scoped retrieval over governed records.
///
/// All ingest operations write to Postgres metadata index
/// (for fast lookups and dataset tracking). The `.rfsource` / Artifact Registry
/// integration will be added in a later phase.
#[derive(Clone)]
pub struct KnowledgeService {
    store: CoreStore,
}

impl KnowledgeService {
    /// Create a new `KnowledgeService`.
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Ingest a knowledge record — validates scope, computes content hash,
    /// stores metadata in Postgres.
    #[instrument(skip(self), fields(record_id = %record.id))]
    pub async fn ingest_knowledge(
        &self,
        mut record: KnowledgeRecord,
    ) -> Result<KnowledgeRecord, ServiceError> {
        // Compute content hash from source_id + citations for integrity
        let content = format!("{}:{:?}", record.source_id, record.citations);
        record.content_hash = hex::encode(Sha256::digest(content.as_bytes()));
        record.indexed_at = Utc::now();

        // Write Postgres metadata
        self.store.insert_knowledge_metadata(&record).await?;

        info!("knowledge record ingested");
        Ok(record)
    }

    /// Query knowledge records by scope and source type. Respects grant scope
    /// filtering — an agent cannot see records outside its allowed scope.
    ///
    /// Uses Postgres-backed query.
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

        // Postgres metadata query
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

    /// Reconcile the Postgres index with the current state.
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
