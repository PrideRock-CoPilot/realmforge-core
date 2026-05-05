//! # Parquet Store
//!
//! Parquet-backed storage for knowledge, evidence, and snapshot datasets.
//!
//! ## Architecture
//!
//! This crate provides the Parquet I/O layer for RealmForge's Knowledge module.
//! It uses Apache Arrow + DataFusion (per `DEC-COUNCIL-003`) for reading and
//! writing Parquet files, with DataFusion providing SQL query capability.
//!
//! ## Dataset Layout
//!
//! ```text
//! {base_path}/
//!   knowledge/
//!     {dataset_id}/
//!       knowledge_{dataset_id}_{version}.parquet
//!   evidence/
//!     {dataset_id}/
//!       evidence_{dataset_id}_{version}.parquet
//!   catalog_snapshots/
//!     {snapshot_id}/
//!       catalog_{snapshot_id}_{version}.parquet
//!   work_paths/
//!     {work_path_id}/
//!       work_path_{work_path_id}_{version}.parquet
//! ```
//!
//! ## Versioning
//!
//! Each dataset supports versioning — a write creates a new version file
//! rather than overwriting. Readers can target a specific version or read
//! across all versions. The Postgres index (in `control-store`) tracks which
//! version is the "current" one.

pub mod error;
pub mod reader;
pub mod schemas;
pub mod writer;

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use tracing::instrument;

pub use error::ParquetStoreError;
pub use reader::{DatasetReader, KnowledgeReader, ParquetKnowledgeQuery};
pub use writer::{EvidenceWriter, KnowledgeWriter, SchemaValidator};

/// Configuration for a `ParquetDataset`.
#[derive(Clone, Debug)]
pub struct ParquetDatasetConfig {
    /// Root directory for all Parquet files.
    pub base_path: PathBuf,

    /// Dataset identifier (e.g. UUID or human-readable name).
    pub dataset_id: String,
}

/// Represents a single version of a Parquet dataset.
#[derive(Clone, Debug)]
pub struct DatasetVersion {
    /// The numeric version (monotonically increasing per dataset).
    pub version: u32,

    /// The schema version used when writing this dataset.
    pub schema_version: u32,

    /// When this version was created.
    pub created_at: DateTime<Utc>,

    /// The file path for this version's Parquet file.
    pub file_path: PathBuf,

    /// Number of records written in this version.
    pub record_count: u64,
}

/// A Parquet dataset backed by the filesystem.
///
/// `ParquetDataset` provides the top-level interface for opening, reading,
/// and writing versioned datasets. It wraps the lower-level `KnowledgeWriter`,
/// `KnowledgeReader`, etc. for a unified API.
#[derive(Clone)]
pub struct ParquetDataset {
    base_path: PathBuf,
    dataset_id: String,
}

impl ParquetDataset {
    /// Create a new `ParquetDataset` for the given `dataset_id` under `base_path`.
    ///
    /// This does **not** create any files — it just sets up the path mapping.
    pub fn new(base_path: impl Into<PathBuf>, dataset_id: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
            dataset_id: dataset_id.into(),
        }
    }

    /// Return the path where this dataset's version files are stored.
    pub fn dataset_path(&self) -> PathBuf {
        self.base_path
            .join("knowledge")
            .join(&self.dataset_id)
    }

    /// Write a batch of knowledge records as a new version.
    #[instrument(skip(self, records), fields(dataset_id = %self.dataset_id, version = version))]
    pub async fn write_version(
        &self,
        version: u32,
        records: &[authority_domain::KnowledgeRecord],
    ) -> Result<DatasetVersion, ParquetStoreError> {
        let writer = KnowledgeWriter::new(&self.base_path);
        writer
            .write_batch(&self.dataset_id, version, records)
            .await?;

        let file_path = self
            .dataset_path()
            .join(format!("knowledge_{}_{version}.parquet", self.dataset_id));

        Ok(DatasetVersion {
            version,
            schema_version: schemas::CURRENT_SCHEMA_VERSION,
            created_at: Utc::now(),
            file_path,
            record_count: records.len() as u64,
        })
    }

    /// Query all versions of this dataset.
    #[instrument(skip(self), fields(dataset_id = %self.dataset_id))]
    pub async fn query(
        &self,
        query: &ParquetKnowledgeQuery,
    ) -> Result<Vec<authority_domain::KnowledgeRecord>, ParquetStoreError> {
        let reader = KnowledgeReader::new(&self.base_path);
        reader.query(query).await
    }

    /// List all versions available for this dataset.
    ///
    /// Reads the dataset directory and parses version numbers from file names.
    pub async fn list_versions(&self) -> Result<Vec<DatasetVersion>, ParquetStoreError> {
        let dir_path = self.dataset_path();
        if !dir_path.exists() {
            return Ok(vec![]);
        }

        let mut read_dir = tokio::fs::read_dir(&dir_path).await?;
        let mut versions = Vec::new();

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
                // Parse version from filename: knowledge_{dataset_id}_{version}.parquet
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(ver_str) = stem.rsplit('_').next() {
                        if let Ok(version) = ver_str.parse::<u32>() {
                            let metadata = tokio::fs::metadata(&path).await?;
                            // Estimate record count from file size (simplified)
                            let record_count = (metadata.len() / 1024).max(1) as u64;

                            versions.push(DatasetVersion {
                                version,
                                schema_version: schemas::CURRENT_SCHEMA_VERSION,
                                created_at: metadata
                                    .created()
                                    .ok()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .and_then(|d| {
                                        DateTime::from_timestamp(d.as_secs() as i64, 0)
                                    })
                                    .unwrap_or_else(|| {
                                        DateTime::from_timestamp_nanos(0)
                                    }),
                                file_path: path,
                                record_count,
                            });
                        }
                    }
                }
            }
        }

        versions.sort_by_key(|v| v.version);
        Ok(versions)
    }

    /// Get the latest (highest) version number for this dataset.
    pub async fn latest_version_number(&self) -> Result<Option<u32>, ParquetStoreError> {
        let versions = self.list_versions().await?;
        Ok(versions.last().map(|v| v.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{Citation, KnowledgeId, KnowledgeRecord, KnowledgeScope, SourceType};
    use chrono::Utc;

    fn make_record() -> KnowledgeRecord {
        KnowledgeRecord {
            id: KnowledgeId::generate(),
            scope: KnowledgeScope::Global,
            source_type: SourceType::Catalog,
            source_id: "src-001".into(),
            content_hash: "abc123".into(),
            indexed_at: Utc::now(),
            citations: vec![Citation {
                source_document_id: "doc-001".into(),
                excerpt: "test".into(),
                confidence: 0.95,
            }],
        }
    }

    #[tokio::test]
    async fn test_parquet_dataset_write_and_list_versions() {
        let dir = tempfile::tempdir().unwrap();
        let dataset = ParquetDataset::new(dir.path(), "test-ds");

        let record = make_record();
        let version = dataset.write_version(1, &[record]).await.unwrap();

        assert_eq!(version.version, 1);
        assert!(version.file_path.exists());

        let versions = dataset.list_versions().await.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
    }

    #[tokio::test]
    async fn test_parquet_dataset_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let dataset = ParquetDataset::new(dir.path(), "test-ds-2");

        assert!(dataset.latest_version_number().await.unwrap().is_none());

        dataset.write_version(1, &[make_record()]).await.unwrap();
        dataset.write_version(2, &[make_record()]).await.unwrap();

        assert_eq!(dataset.latest_version_number().await.unwrap(), Some(2));
    }

    #[tokio::test]
    async fn test_parquet_dataset_query_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let dataset = ParquetDataset::new(dir.path(), "test-query");
        let record = make_record();

        dataset.write_version(1, &[record]).await.unwrap();

        let query = ParquetKnowledgeQuery {
            scopes: Some(vec![KnowledgeScope::Global]),
            source_types: None,
            date_range: None,
            text_search: None,
            limit: 100,
            offset: 0,
        };

        let results = dataset.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, "src-001");
    }
}
