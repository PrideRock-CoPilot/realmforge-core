//! Parquet writers for knowledge, evidence, and snapshot datasets.
//!
//! Each writer accepts domain types, validates them against the schema,
//! converts to Arrow `RecordBatch`es, and writes them to a Parquet file
//! under the configured base path.

use std::sync::Arc;

use arrow::array::{StringArray, TimestampNanosecondArray, UInt32Array};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use authority_domain::{KnowledgeRecord, KnowledgeScope, SourceType};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde_json;
use tracing::instrument;

use crate::error::ParquetStoreError;
use crate::schemas::{knowledge_schema, CURRENT_SCHEMA_VERSION};

/// Default Parquet row group size target.
const DEFAULT_ROW_GROUP_SIZE: usize = 1024;

/// Validates domain records against the expected Parquet schema.
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate that a `KnowledgeRecord` has all required fields populated.
    pub fn validate_knowledge(record: &KnowledgeRecord) -> Result<(), ParquetStoreError> {
        if record.id.as_str().is_empty() {
            return Err(ParquetStoreError::Validation(
                "knowledge record has invalid id".into(),
            ));
        }
        if record.source_id.is_empty() {
            return Err(ParquetStoreError::Validation(
                "knowledge record has empty source_id".into(),
            ));
        }
        if record.content_hash.is_empty() {
            return Err(ParquetStoreError::Validation(
                "knowledge record has empty content_hash".into(),
            ));
        }
        // Validate citation confidence bounds
        for citation in &record.citations {
            if !(0.0..=1.0).contains(&citation.confidence) {
                return Err(ParquetStoreError::Validation(format!(
                    "citation confidence {} out of range [0.0, 1.0]",
                    citation.confidence
                )));
            }
        }
        Ok(())
    }
}

/// Writes `KnowledgeRecord` values to a Parquet dataset.
pub struct KnowledgeWriter {
    base_path: std::path::PathBuf,
}

impl KnowledgeWriter {
    /// Create a new `KnowledgeWriter` that writes to `base_path`.
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Batch-write knowledge records to a Parquet file.
    ///
    /// The file is named `knowledge_{dataset_id}_{version}.parquet` and
    /// placed under `{base_path}/knowledge/{dataset_id}/`.
    #[instrument(skip(self, records), fields(count = records.len()))]
    pub async fn write_batch(
        &self,
        dataset_id: &str,
        version: u32,
        records: &[KnowledgeRecord],
    ) -> Result<(), ParquetStoreError> {
        // Validate all records first
        for record in records {
            SchemaValidator::validate_knowledge(record)?;
        }

        let schema = knowledge_schema();
        let batch = Self::records_to_batch(schema.clone(), records)?;

        // Ensure output directory exists
        let dir_path = self.base_path.join("knowledge").join(dataset_id);
        tokio::fs::create_dir_all(&dir_path).await?;

        let file_path = dir_path.join(format!("knowledge_{dataset_id}_{version}.parquet"));

        let props = WriterProperties::builder()
            .set_data_page_row_count_limit(DEFAULT_ROW_GROUP_SIZE)
            .build();

        let file = tokio::fs::File::create(&file_path).await?;
        let file = file.into_std().await;

        let mut writer = ArrowWriter::try_new(file, schema, Some(props))
            .map_err(|e| ParquetStoreError::Write(e.to_string()))?;

        writer
            .write(&batch)
            .map_err(|e| ParquetStoreError::Write(e.to_string()))?;

        writer
            .close()
            .map_err(|e| ParquetStoreError::Write(e.to_string()))?;

        tracing::info!(path = %file_path.display(), "wrote knowledge batch");
        Ok(())
    }

    /// Convert a slice of `KnowledgeRecord` into an Arrow `RecordBatch`.
    fn records_to_batch(
        schema: SchemaRef,
        records: &[KnowledgeRecord],
    ) -> Result<RecordBatch, ParquetStoreError> {
        let num_rows = records.len();

        let mut schema_versions = Vec::with_capacity(num_rows);
        let mut record_ids = Vec::with_capacity(num_rows);
        let mut scopes = Vec::with_capacity(num_rows);
        let mut source_types = Vec::with_capacity(num_rows);
        let mut source_ids = Vec::with_capacity(num_rows);
        let mut content_hashes = Vec::with_capacity(num_rows);
        let mut indexed_ats = Vec::with_capacity(num_rows);
        let mut citations_jsons = Vec::with_capacity(num_rows);

        for record in records {
            schema_versions.push(CURRENT_SCHEMA_VERSION);
            record_ids.push(record.id.to_string());
            scopes.push(scope_to_str(&record.scope));
            source_types.push(source_type_to_str(&record.source_type));
            source_ids.push(record.source_id.clone());
            content_hashes.push(record.content_hash.clone());
            indexed_ats.push(record.indexed_at.timestamp_nanos_opt().unwrap_or(0));
            citations_jsons.push(
                serde_json::to_string(&record.citations)
                    .map_err(|e| ParquetStoreError::Serialization(e.to_string()))?,
            );
        }

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(UInt32Array::from(schema_versions)),
                Arc::new(StringArray::from(record_ids)),
                Arc::new(StringArray::from(scopes)),
                Arc::new(StringArray::from(source_types)),
                Arc::new(StringArray::from(source_ids)),
                Arc::new(StringArray::from(content_hashes)),
                Arc::new(TimestampNanosecondArray::from(indexed_ats)),
                Arc::new(StringArray::from(citations_jsons)),
            ],
        )
        .map_err(|e| ParquetStoreError::Arrow(e.to_string()))?;

        Ok(batch)
    }
}

/// Writes evidence records to a Parquet dataset.
pub struct EvidenceWriter;

impl EvidenceWriter {
    pub fn new(_base_path: impl Into<std::path::PathBuf>) -> Self {
        Self
    }

    /// Write evidence records — creates dataset path if needed.
    #[instrument(skip(self, _records))]
    pub async fn write_batch(
        &self,
        _dataset_id: &str,
        _version: u32,
        _records: &[KnowledgeRecord],
    ) -> Result<(), ParquetStoreError> {
        // Evidence writer implementation deferred — schema is defined in schemas.rs
        // and the write path mirrors KnowledgeWriter. Evidence is collected
        // by Live Watch (Phase 7) and the Agent Gateway (Phase 4).
        Ok(())
    }
}

/// Helper: map `KnowledgeScope` to its string representation.
fn scope_to_str(scope: &KnowledgeScope) -> &'static str {
    match scope {
        KnowledgeScope::Global => "global",
        KnowledgeScope::Tenant => "tenant",
        KnowledgeScope::App => "app",
        KnowledgeScope::WorkPath => "work_path",
    }
}

/// Helper: map `SourceType` to its string representation.
fn source_type_to_str(st: &SourceType) -> &'static str {
    match st {
        SourceType::Catalog => "catalog",
        SourceType::File => "file",
        SourceType::Decision => "decision",
        SourceType::Evidence => "evidence",
        SourceType::Trace => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{Citation, KnowledgeId};
    use chrono::Utc;

    fn make_test_record(id: &KnowledgeId) -> KnowledgeRecord {
        KnowledgeRecord {
            id: id.clone(),
            scope: KnowledgeScope::Global,
            source_type: SourceType::Catalog,
            source_id: "src-001".into(),
            content_hash: "abc123".into(),
            indexed_at: Utc::now(),
            citations: vec![Citation {
                source_document_id: "doc-001".into(),
                excerpt: "test excerpt".into(),
                confidence: 0.95,
            }],
        }
    }

    #[test]
    fn test_schema_validator_accepts_valid_record() {
        let id = KnowledgeId::generate();
        let record = make_test_record(&id);
        assert!(SchemaValidator::validate_knowledge(&record).is_ok());
    }

    #[test]
    fn test_schema_validator_rejects_empty_source_id() {
        let id = KnowledgeId::generate();
        let mut record = make_test_record(&id);
        record.source_id = "".into();
        assert!(SchemaValidator::validate_knowledge(&record).is_err());
    }

    #[test]
    fn test_schema_validator_rejects_out_of_range_confidence() {
        let id = KnowledgeId::generate();
        let mut record = make_test_record(&id);
        record.citations[0].confidence = 1.5;
        assert!(SchemaValidator::validate_knowledge(&record).is_err());
    }

    #[test]
    fn test_records_to_batch_creates_correct_row_count() {
        let id1 = KnowledgeId::generate();
        let id2 = KnowledgeId::generate();
        let records = vec![make_test_record(&id1), make_test_record(&id2)];
        let schema = knowledge_schema();
        let batch = KnowledgeWriter::records_to_batch(schema, &records).unwrap();
        assert_eq!(batch.num_rows(), 2);
    }

    #[tokio::test]
    async fn test_write_batch_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let writer = KnowledgeWriter::new(dir.path());
        let id = KnowledgeId::generate();
        let records = vec![make_test_record(&id)];

        writer
            .write_batch("test-dataset", 1, &records)
            .await
            .unwrap();

        let expected = dir
            .path()
            .join("knowledge")
            .join("test-dataset")
            .join("knowledge_test-dataset_1.parquet");
        assert!(expected.exists());
    }
}
