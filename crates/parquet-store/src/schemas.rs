//! Parquet schema definitions for knowledge, evidence, catalog snapshots,
//! and work paths.
//!
//! These schemas define the column layout of each Parquet dataset written
//! by `parquet-store`. Every dataset begins with a schema version field to
//! enable forward migration.

use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};

/// Schema version constant embedded in each schema.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Return the Parquet schema for a `KnowledgeRecord` dataset.
///
/// Columns:
/// - `schema_version`: `UInt32` — enables forward-compatible readers
/// - `record_id`: `Utf8` — `KnowledgeId` as string
/// - `scope`: `Utf8` — `"global"`, `"tenant"`, `"app"`, `"work_path"`
/// - `source_type`: `Utf8` — `"catalog"`, `"file"`, `"decision"`, `"evidence"`, `"trace"`
/// - `source_id`: `Utf8` — original source document identifier
/// - `content_hash`: `Utf8` — SHA-256 hex digest
/// - `indexed_at`: `Timestamp(Nanosecond, None)` — UTC index timestamp
/// - `citations_json`: `Utf8` — serialized `Vec<Citation>` for query-time deserialization
pub fn knowledge_schema() -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
        Field::new("schema_version", DataType::UInt32, false),
        Field::new("record_id", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("source_type", DataType::Utf8, false),
        Field::new("source_id", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new(
            "indexed_at",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("citations_json", DataType::Utf8, false),
    ]))
}

/// Return the Parquet schema for an `EvidenceRecord` dataset.
///
/// Columns:
/// - `schema_version`: `UInt32`
/// - `evidence_id`: `Utf8`
/// - `record_id`: `Utf8` — FK to `KnowledgeRecord.record_id`
/// - `source_type`: `Utf8`
/// - `source_id`: `Utf8`
/// - `collected_at`: `Timestamp(Nanosecond, None)`
/// - `payload_json`: `Utf8` — type-specific evidence payload
/// - `confidence`: `Float32` — 0.0–1.0
pub fn evidence_schema() -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
        Field::new("schema_version", DataType::UInt32, false),
        Field::new("evidence_id", DataType::Utf8, false),
        Field::new("record_id", DataType::Utf8, false),
        Field::new("source_type", DataType::Utf8, false),
        Field::new("source_id", DataType::Utf8, false),
        Field::new(
            "collected_at",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("payload_json", DataType::Utf8, false),
        Field::new("confidence", DataType::Float32, false),
    ]))
}

/// Return the Parquet schema for a `CatalogSnapshot` dataset.
///
/// Columns:
/// - `schema_version`: `UInt32`
/// - `snapshot_id`: `Utf8`
/// - `scope`: `Utf8`
/// - `module_type`: `Utf8`
/// - `path`: `Utf8` — fully qualified module path
/// - `snapshot_at`: `Timestamp(Nanosecond, None)`
/// - `metadata_json`: `Utf8` — serialized module metadata
pub fn catalog_snapshot_schema() -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
        Field::new("schema_version", DataType::UInt32, false),
        Field::new("snapshot_id", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("module_type", DataType::Utf8, false),
        Field::new("path", DataType::Utf8, false),
        Field::new(
            "snapshot_at",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("metadata_json", DataType::Utf8, false),
    ]))
}

/// Return the Parquet schema for a `WorkPathSnapshot` dataset.
///
/// Columns:
/// - `schema_version`: `UInt32`
/// - `snapshot_id`: `Utf8`
/// - `work_path_id`: `Utf8`
/// - `node_type`: `Utf8`
/// - `node_name`: `Utf8`
/// - `snapshot_at`: `Timestamp(Nanosecond, None)`
/// - `metadata_json`: `Utf8`
pub fn work_path_schema() -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
        Field::new("schema_version", DataType::UInt32, false),
        Field::new("snapshot_id", DataType::Utf8, false),
        Field::new("work_path_id", DataType::Utf8, false),
        Field::new("node_type", DataType::Utf8, false),
        Field::new("node_name", DataType::Utf8, false),
        Field::new(
            "snapshot_at",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("metadata_json", DataType::Utf8, false),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_schema_has_all_fields() {
        let schema = knowledge_schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.contains(&"record_id"));
        assert!(field_names.contains(&"scope"));
        assert!(field_names.contains(&"source_type"));
        assert!(field_names.contains(&"content_hash"));
        assert!(field_names.contains(&"citations_json"));
        assert_eq!(schema.fields().len(), 8);
    }

    #[test]
    fn test_evidence_schema_has_all_fields() {
        let schema = evidence_schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.contains(&"evidence_id"));
        assert!(field_names.contains(&"record_id"));
        assert!(field_names.contains(&"payload_json"));
        assert!(field_names.contains(&"confidence"));
        assert_eq!(schema.fields().len(), 8);
    }

    #[test]
    fn test_catalog_snapshot_schema_has_all_fields() {
        let schema = catalog_snapshot_schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.contains(&"snapshot_id"));
        assert!(field_names.contains(&"module_type"));
        assert!(field_names.contains(&"path"));
        assert_eq!(schema.fields().len(), 7);
    }

    #[test]
    fn test_work_path_schema_has_all_fields() {
        let schema = work_path_schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.contains(&"work_path_id"));
        assert!(field_names.contains(&"node_type"));
        assert!(field_names.contains(&"node_name"));
        assert_eq!(schema.fields().len(), 7);
    }
}
