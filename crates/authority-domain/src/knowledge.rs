use crate::KnowledgeId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Scope level for a knowledge record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KnowledgeScope {
    Global,
    Tenant,
    App,
    WorkPath,
}

impl fmt::Display for KnowledgeScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnowledgeScope::Global => write!(f, "global"),
            KnowledgeScope::Tenant => write!(f, "tenant"),
            KnowledgeScope::App => write!(f, "app"),
            KnowledgeScope::WorkPath => write!(f, "work_path"),
        }
    }
}

/// Source type for a knowledge record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Catalog,
    File,
    Decision,
    Evidence,
    Trace,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Catalog => write!(f, "catalog"),
            SourceType::File => write!(f, "file"),
            SourceType::Decision => write!(f, "decision"),
            SourceType::Evidence => write!(f, "evidence"),
            SourceType::Trace => write!(f, "trace"),
        }
    }
}

/// A citation linking a knowledge answer to a governed source record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citation {
    pub source_document_id: String,
    pub excerpt: String,
    pub confidence: f32,
}

/// A knowledge record stored in a Parquet dataset (metadata lives in Postgres).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeRecord {
    pub id: KnowledgeId,
    pub scope: KnowledgeScope,
    pub source_type: SourceType,
    pub source_id: String,
    pub content_hash: String,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
    pub citations: Vec<Citation>,
}

/// Query parameters for knowledge retrieval.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub scopes: Vec<KnowledgeScope>,
    pub source_types: Option<Vec<SourceType>>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub text_search: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

/// Summary metadata for a Parquet dataset stored in Postgres.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub dataset_id: String,
    pub source_type: SourceType,
    pub schema_version: u32,
    pub record_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_indexed_at: chrono::DateTime<chrono::Utc>,
}
