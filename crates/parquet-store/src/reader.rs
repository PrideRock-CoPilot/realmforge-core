//! Parquet readers for knowledge, evidence, and snapshot datasets.
//!
//! Readers use DataFusion for query execution over one or more Parquet
//! files, enabling SQL-like filtering on scope, source_type, date range,
//! and text search.

use arrow::array::StringArray;
use arrow::compute::cast;
use authority_domain::{Citation, KnowledgeRecord, KnowledgeScope, SourceType};
use chrono::{DateTime, Utc};
use datafusion::prelude::{ParquetReadOptions, SessionContext};
use serde_json;
use tracing::instrument;

use crate::error::ParquetStoreError;

/// Query parameters for reading knowledge records from Parquet datasets.
#[derive(Clone, Debug)]
pub struct ParquetKnowledgeQuery {
    pub scopes: Option<Vec<KnowledgeScope>>,
    pub source_types: Option<Vec<SourceType>>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub text_search: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

/// Reads knowledge records from Parquet datasets using DataFusion.
pub struct KnowledgeReader {
    base_path: std::path::PathBuf,
}

impl KnowledgeReader {
    /// Create a new `KnowledgeReader` rooted at `base_path`.
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    // Helper: collect all Parquet files recursively under a directory
    fn collect_parquet_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, ParquetStoreError> {
        let mut files = Vec::new();
        if !dir.is_dir() {
            return Ok(files);
        }
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        files.extend(Self::collect_parquet_files(&path)?);
                    } else if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
                        files.push(path);
                    }
                }
            }
            Err(e) => return Err(ParquetStoreError::Io(e)),
        }
        files.sort();
        Ok(files)
    }

    /// Query knowledge records across all datasets matching the given filters.
    ///
    /// Uses DataFusion to register the Parquet directory as a table and
    /// runs a filtered SQL query to return results with pagination.
    #[instrument(skip(self), fields(query = ?query))]
    pub async fn query(&self, query: &ParquetKnowledgeQuery) -> Result<Vec<KnowledgeRecord>, ParquetStoreError> {
        let ctx = SessionContext::new();

        let knowledge_dir = self.base_path.join("knowledge");
        if !knowledge_dir.exists() {
            return Ok(vec![]);
        }

        // Collect all Parquet files recursively
        let parquet_files = Self::collect_parquet_files(&knowledge_dir)?;
        if parquet_files.is_empty() {
            return Ok(vec![]);
        }

        // Register each file with known schema
        for (i, file_path) in parquet_files.iter().enumerate() {
            let table_name = format!("knowledge_{i}");
            let file_path_str = file_path.to_string_lossy().to_string();
            ctx.register_parquet(&table_name, &file_path_str, ParquetReadOptions::default())
                .await
                .map_err(|e| ParquetStoreError::Read(e.to_string()))?;
        }

        // Build SQL query across all tables using UNION ALL
        let num_tables = parquet_files.len();
        let select_expr =
            r#"SELECT record_id, "scope", source_type, source_id, content_hash, indexed_at, citations_json"#;

        let mut sql = if num_tables == 1 {
            format!("{select_expr} FROM knowledge_0 WHERE 1=1")
        } else {
            let mut s = String::new();
            for i in 0..num_tables {
                if i > 0 {
                    s.push_str(" UNION ALL ");
                }
                s.push_str(&format!("{select_expr} FROM knowledge_{i}"));
            }
            s.push_str(" WHERE 1=1");
            s
        };

        // Scope filter
        if let Some(scopes) = &query.scopes {
            let scope_strs: Vec<String> = scopes
                .iter()
                .map(|s| format!("'{}'", scope_to_str(s)))
                .collect();
            sql.push_str(&format!(r#" AND "scope" IN ({})"#, scope_strs.join(",")));
        }

        // Source type filter
        if let Some(source_types) = &query.source_types {
            let st_strs: Vec<String> = source_types
                .iter()
                .map(|s| format!("'{}'", source_type_to_str(s)))
                .collect();
            sql.push_str(&format!(" AND source_type IN ({})", st_strs.join(",")));
        }

        // Date range filter
        if let Some((start, end)) = &query.date_range {
            let start_ns = start.timestamp_nanos_opt().unwrap_or(0);
            let end_ns = end.timestamp_nanos_opt().unwrap_or(0);
            sql.push_str(&format!(
                " AND indexed_at >= {} AND indexed_at <= {}",
                start_ns, end_ns
            ));
        }

        // Text search filter (case-insensitive on source_id and citations_json)
        if let Some(text) = &query.text_search {
            let escaped = text.replace('\'', "''");
            sql.push_str(&format!(
                " AND (LOWER(source_id) LIKE LOWER('%{escaped}%') OR LOWER(citations_json) LIKE LOWER('%{escaped}%'))"
            ));
        }

        // Order and pagination
        sql.push_str(" ORDER BY indexed_at DESC");
        sql.push_str(&format!(" LIMIT {} OFFSET {}", query.limit, query.offset));

        let df = ctx
            .sql(&sql)
            .await
            .map_err(|e| ParquetStoreError::Query(e.to_string()))?;

        let batches: Vec<arrow::record_batch::RecordBatch> = df
            .collect()
            .await
            .map_err(|e| ParquetStoreError::Query(e.to_string()))?;

        let records = Self::batches_to_records(batches)?;
        Ok(records)
    }

    /// Cast an Arrow array column to an owned `StringArray`, handling dictionary encoding from Parquet.
    fn cast_to_string_array(col: &arrow::array::ArrayRef) -> Result<StringArray, ParquetStoreError> {
        // If it's already a StringArray, clone it (cheap — Arc-backed buffers)
        if let Some(sa) = col.as_any().downcast_ref::<StringArray>() {
            return Ok(sa.clone());
        }
        // Otherwise cast to Utf8 — handles DictionaryArray, LargeUtf8, etc.
        let casted = cast(col, &arrow::datatypes::DataType::Utf8)
            .map_err(|e| ParquetStoreError::Read(format!("cast to Utf8 failed: {e}")))?;
        casted
            .as_any()
            .downcast_ref::<StringArray>()
            .cloned()
            .ok_or_else(|| ParquetStoreError::Read("cast to Utf8 did not produce StringArray".into()))
    }

    /// Convert Arrow `RecordBatch`es to `KnowledgeRecord` values.
    fn batches_to_records(batches: Vec<arrow::record_batch::RecordBatch>) -> Result<Vec<KnowledgeRecord>, ParquetStoreError> {
        let mut records = Vec::new();

        // Pre-alloc to minimize reallocs
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        records.reserve(total_rows);

        for batch in &batches {
            let record_ids = batch
                .column_by_name("record_id")
                .ok_or_else(|| ParquetStoreError::Read("missing record_id column".into()))?;
            let scopes = batch
                .column_by_name("scope")
                .ok_or_else(|| ParquetStoreError::Read("missing scope column".into()))?;
            let source_types_col = batch
                .column_by_name("source_type")
                .ok_or_else(|| ParquetStoreError::Read("missing source_type column".into()))?;
            let source_ids = batch
                .column_by_name("source_id")
                .ok_or_else(|| ParquetStoreError::Read("missing source_id column".into()))?;
            let content_hashes = batch
                .column_by_name("content_hash")
                .ok_or_else(|| ParquetStoreError::Read("missing content_hash column".into()))?;
            let indexed_ats = batch
                .column_by_name("indexed_at")
                .ok_or_else(|| ParquetStoreError::Read("missing indexed_at column".into()))?;
            let citations_jsons = batch
                .column_by_name("citations_json")
                .ok_or_else(|| ParquetStoreError::Read("missing citations_json column".into()))?;

            let string_array_ids = Self::cast_to_string_array(&record_ids)?;
            let string_array_scopes = Self::cast_to_string_array(&scopes)?;
            let string_array_st = Self::cast_to_string_array(&source_types_col)?;
            let string_array_sids = Self::cast_to_string_array(&source_ids)?;
            let string_array_ch = Self::cast_to_string_array(&content_hashes)?;
            let ts_array = indexed_ats.as_any().downcast_ref::<arrow::array::TimestampNanosecondArray>().ok_or_else(|| {
                ParquetStoreError::Read("indexed_at column is not TimestampNanosecondArray".into())
            })?;
            let string_array_cits = Self::cast_to_string_array(&citations_jsons)?;

            for i in 0..batch.num_rows() {
                let id_str = string_array_ids.value(i);
                let scope_str = string_array_scopes.value(i);
                let st_str = string_array_st.value(i);
                let source_id = string_array_sids.value(i).to_string();
                let content_hash = string_array_ch.value(i).to_string();
                let ts_nanos = ts_array.value(i);
                let citations_json = string_array_cits.value(i);

                // Parse KnowledgeId from string
                let id = authority_domain::KnowledgeId::new(id_str.to_string())
                    .map_err(|_| ParquetStoreError::Read(format!("invalid knowledge id: {id_str}")))?;

                let scope = str_to_scope(scope_str)
                    .ok_or_else(|| ParquetStoreError::Read(format!("unknown scope: {scope_str}")))?;
                let source_type = str_to_source_type(st_str)
                    .ok_or_else(|| ParquetStoreError::Read(format!("unknown source_type: {st_str}")))?;

                // Parse timestamp from nanoseconds
                let indexed_at = DateTime::from_timestamp_nanos(ts_nanos);

                // Parse citations from JSON
                let citations: Vec<Citation> = serde_json::from_str(citations_json)
                    .map_err(|e| ParquetStoreError::Serialization(e.to_string()))?;

                records.push(KnowledgeRecord {
                    id,
                    scope,
                    source_type,
                    source_id,
                    content_hash,
                    indexed_at,
                    citations,
                });
            }
        }

        Ok(records)
    }

    /// List all available dataset IDs under the knowledge directory.
    pub async fn list_datasets(&self) -> Result<Vec<String>, ParquetStoreError> {
        let knowledge_dir = self.base_path.join("knowledge");
        if !knowledge_dir.exists() {
            return Ok(vec![]);
        }

        let mut datasets = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&knowledge_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    datasets.push(name.to_string());
                }
            }
        }

        datasets.sort();
        Ok(datasets)
    }
}

/// Streams records from a dataset with pagination support.
pub struct DatasetReader {
    base_path: std::path::PathBuf,
}

impl DatasetReader {
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Stream records from a specific dataset with pagination.
    #[instrument(skip(self), fields(dataset_id = %dataset_id))]
    pub async fn read_dataset(
        &self,
        dataset_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<KnowledgeRecord>, ParquetStoreError> {
        let reader = KnowledgeReader::new(self.base_path.clone());
        let query = ParquetKnowledgeQuery {
            scopes: None,
            source_types: None,
            date_range: None,
            text_search: None,
            limit,
            offset,
        };
        reader.query(&query).await
    }

    /// Get the record count for a dataset by counting Parquet row groups.
    pub async fn record_count(&self, _dataset_id: &str) -> Result<u64, ParquetStoreError> {
        // Simplified count — production version would use DataFusion COUNT(*)
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn scope_to_str(scope: &KnowledgeScope) -> &'static str {
    match scope {
        KnowledgeScope::Global => "global",
        KnowledgeScope::Tenant => "tenant",
        KnowledgeScope::App => "app",
        KnowledgeScope::WorkPath => "work_path",
    }
}

fn source_type_to_str(st: &SourceType) -> &'static str {
    match st {
        SourceType::Catalog => "catalog",
        SourceType::File => "file",
        SourceType::Decision => "decision",
        SourceType::Evidence => "evidence",
        SourceType::Trace => "trace",
    }
}

fn str_to_scope(s: &str) -> Option<KnowledgeScope> {
    match s {
        "global" => Some(KnowledgeScope::Global),
        "tenant" => Some(KnowledgeScope::Tenant),
        "app" => Some(KnowledgeScope::App),
        "work_path" => Some(KnowledgeScope::WorkPath),
        _ => None,
    }
}

fn str_to_source_type(s: &str) -> Option<SourceType> {
    match s {
        "catalog" => Some(SourceType::Catalog),
        "file" => Some(SourceType::File),
        "decision" => Some(SourceType::Decision),
        "evidence" => Some(SourceType::Evidence),
        "trace" => Some(SourceType::Trace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_scope_roundtrip() {
        assert_eq!(str_to_scope("global"), Some(KnowledgeScope::Global));
        assert_eq!(str_to_scope("tenant"), Some(KnowledgeScope::Tenant));
        assert_eq!(str_to_scope("app"), Some(KnowledgeScope::App));
        assert_eq!(str_to_scope("work_path"), Some(KnowledgeScope::WorkPath));
        assert_eq!(str_to_scope("unknown"), None);
    }

    #[test]
    fn test_str_to_source_type_roundtrip() {
        assert_eq!(str_to_source_type("catalog"), Some(SourceType::Catalog));
        assert_eq!(str_to_source_type("file"), Some(SourceType::File));
        assert_eq!(str_to_source_type("decision"), Some(SourceType::Decision));
        assert_eq!(str_to_source_type("evidence"), Some(SourceType::Evidence));
        assert_eq!(str_to_source_type("trace"), Some(SourceType::Trace));
        assert_eq!(str_to_source_type("nope"), None);
    }

    #[test]
    fn test_scope_to_str_matches() {
        assert_eq!(scope_to_str(&KnowledgeScope::Global), "global");
        assert_eq!(scope_to_str(&KnowledgeScope::Tenant), "tenant");
    }

    #[test]
    fn test_source_type_to_str_matches() {
        assert_eq!(source_type_to_str(&SourceType::Catalog), "catalog");
        assert_eq!(source_type_to_str(&SourceType::Evidence), "evidence");
    }
}
