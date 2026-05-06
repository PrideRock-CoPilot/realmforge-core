//! Domain model types for the RFSource single-file source ledger.
//!
//! ## Layer Law
//!
//! This module contains pure types only — no IO, no database, no HTTP.
//! All types derive `Serialize` + `Deserialize` for frame storage in
//! `.rfsource` files and for PostgreSQL Artifact Registry persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Format name for the `.rfsource` file format.
pub const FORMAT_NAME: &str = "realmforge.rfsource";

/// Current schema version of the domain model.
pub const SCHEMA_VERSION: u32 = 1;

/// Default branch ID for the main branch.
pub const MAIN_BRANCH_ID: &str = "br_main";

/// Default branch name for the main branch.
pub const MAIN_BRANCH_NAME: &str = "main";

/// Default for the `branch_id` field in CommitRecord.
pub fn default_main_branch_id() -> String {
    MAIN_BRANCH_ID.to_string()
}

/// Default commit kind.
pub fn default_commit_kind() -> String {
    "change".to_string()
}

/// RFSource project manifest — stored at the start of every `.rfsource` file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub schema_version: u32,
    pub project_id: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub current_commit_id: Option<String>,
    pub active_segment_format: String,
    pub snapshot_format: String,
    pub invariants: Vec<String>,
}

impl Manifest {
    pub fn new(project_name: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            format: FORMAT_NAME.to_string(),
            schema_version: SCHEMA_VERSION,
            project_id: project_id.into(),
            project_name: project_name.into(),
            created_at: Utc::now(),
            current_commit_id: None,
            active_segment_format: "jsonl-mvp-arrow-ipc-target".to_string(),
            snapshot_format: "rfsource-native".to_string(),
            invariants: vec![
                "rfsource is canonical source state".to_string(),
                "filesystem files are materialized projections".to_string(),
                "all artifact writes create versioned commits".to_string(),
                "branch state is derived from append-only commits".to_string(),
                "time warp creates forward rollback commits".to_string(),
                "indexes are derived and rebuildable".to_string(),
                "agent reads must be policy/grant scoped".to_string(),
            ],
        }
    }
}

/// A branch in the source control model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchRecord {
    pub branch_id: String,
    pub name: String,
    pub base_commit_id: Option<String>,
    pub head_commit_id: Option<String>,
    pub parent_branch_id: Option<String>,
    pub status: String,
    pub actor: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A source artifact tracked in the ledger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceArtifact {
    pub artifact_id: String,
    pub logical_path: String,
    pub language: String,
    pub owner_capability: String,
    pub risk_level: String,
    pub current_version_id: String,
    pub policy_bindings: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub required_tests: Vec<String>,
    pub materialization_mode: String,
    #[serde(default)]
    pub deleted: bool,
}

/// A specific version of a source artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactVersion {
    pub version_id: String,
    pub artifact_id: String,
    pub parent_version_id: Option<String>,
    pub content_hash: String,
    pub commit_id: String,
    pub created_at: DateTime<Utc>,
    pub line_count: u32,
    pub chunk_count: u32,
}

/// A chunk of source text extracted from an artifact version.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceChunk {
    pub chunk_id: String,
    pub artifact_id: String,
    pub version_id: String,
    pub ordinal: u32,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
    pub text_hash: String,
    pub symbols_defined: Vec<String>,
    pub symbols_referenced: Vec<String>,
}

/// A symbol (function, struct, type, etc.) extracted from source.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub symbol_id: String,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub artifact_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub line_start: u32,
}

/// A dependency edge between artifacts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from_artifact_id: String,
    pub from_version_id: String,
    pub dependency: String,
    pub dependency_kind: String,
    pub evidence: String,
}

/// A commit record in the append-only commit chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitRecord {
    pub commit_id: String,
    pub sequence: u64,
    pub parent_commit_id: Option<String>,
    #[serde(default = "default_main_branch_id")]
    pub branch_id: String,
    #[serde(default = "default_commit_kind")]
    pub commit_kind: String,
    #[serde(default)]
    pub rollback_anchor_commit_id: Option<String>,
    pub message: String,
    pub actor: String,
    pub created_at: DateTime<Utc>,
    pub artifacts_changed: Vec<String>,
    #[serde(default)]
    pub standards_findings: Vec<String>,
    pub segment_refs: Vec<String>,
}

/// A text index entry for full-text search.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextIndexEntry {
    pub term: String,
    pub artifact_id: String,
    pub version_id: String,
    pub chunk_id: String,
}

/// A symbol index entry for symbol search.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolIndexEntry {
    pub symbol_name: String,
    pub artifact_id: String,
    pub version_id: String,
    pub chunk_id: String,
}

/// A search hit returned from text or symbol search.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub artifact_id: String,
    pub logical_path: String,
    pub version_id: String,
    pub chunk_id: String,
    pub line_start: u32,
    pub line_end: u32,
    pub score_reason: String,
    pub excerpt: String,
}

/// A governance check finding.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckFinding {
    pub finding_id: String,
    pub object_ref: String,
    pub logical_path: String,
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: Option<u32>,
    pub blocking: bool,
}

/// A hunk of changed lines between two versions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineHunk {
    pub old_start: u32,
    pub old_end: u32,
    pub new_start: u32,
    pub new_end: u32,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
}

/// A file that changed between two tree states.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangedFile {
    pub artifact_id: String,
    pub logical_path: String,
    pub change_kind: String,
    pub old_version_id: Option<String>,
    pub new_version_id: Option<String>,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub hunks: Vec<LineHunk>,
}

/// A comparison report between two tree states.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompareReport {
    pub from_ref: String,
    pub to_ref: String,
    pub changed_files: Vec<ChangedFile>,
    pub finding_count: usize,
    pub blocking_finding_count: usize,
}

/// A proposal (merge request / pull request) between branches.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalRecord {
    pub proposal_id: String,
    pub title: String,
    pub source_branch_id: String,
    pub target_branch_id: String,
    pub base_commit_id: Option<String>,
    pub source_head_commit_id: Option<String>,
    pub target_head_commit_id: Option<String>,
    pub status: String,
    pub actor: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub changed_file_count: usize,
    pub blocking_finding_count: usize,
}

/// A code review comment on a chunk, artifact, or general thread.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentRecord {
    pub comment_id: String,
    pub object_ref: String,
    pub thread_id: String,
    pub body: String,
    pub actor: String,
    pub severity: String,
    pub status: String,
    pub artifact_id: Option<String>,
    pub version_id: Option<String>,
    pub chunk_id: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A preview of a time-warp operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeWarpPreview {
    pub target_ref: String,
    pub branch_id: String,
    pub scope: String,
    pub logical_path: Option<String>,
    pub changed_files: Vec<ChangedFile>,
    pub blocking_comment_count: usize,
    pub blocking_finding_count: usize,
    pub clean: bool,
}

/// Outcome of a time-warp operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeWarpOutcome {
    pub preview: TimeWarpPreview,
    pub commits: Vec<CommitOutcome>,
}

/// Aggregated statistics for a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectStats {
    pub manifest: Manifest,
    pub branch_count: usize,
    pub commit_count: usize,
    pub artifact_count: usize,
    pub chunk_count: usize,
    pub symbol_count: usize,
    pub proposal_count: usize,
    pub comment_count: usize,
}

/// Request to commit an artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitArtifactRequest {
    pub logical_path: String,
    pub language: String,
    pub content: String,
    pub owner_capability: String,
    pub risk_level: String,
    pub policy_bindings: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub required_tests: Vec<String>,
    pub actor: String,
    pub message: String,
}

/// Outcome of committing an artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitOutcome {
    pub commit: CommitRecord,
    pub artifact: SourceArtifact,
    pub version: ArtifactVersion,
    pub chunk_count: usize,
    pub symbol_count: usize,
}

/// A bundle of data written in a single commit frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitBundle {
    pub commit: CommitRecord,
    pub artifact: SourceArtifact,
    pub version: ArtifactVersion,
    pub chunks: Vec<SourceChunk>,
    pub symbols: Vec<SymbolRecord>,
    pub dependencies: Vec<DependencyEdge>,
}
