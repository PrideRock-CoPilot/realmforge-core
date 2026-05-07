//! RFSource store — single-file source ledger operations.
//!
//! Provides the main `RFSource` struct that wraps read/write operations
//! on a `.rfsource` file, including commit, branch, proposal, and
//! time-warp operations.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use rfsource_core::{
    short_id, BranchRecord, CommentRecord, CommitArtifactRequest, CommitBundle, CommitOutcome,
    CommitRecord, Manifest, ProjectStats, ProposalRecord, RFSourceError, SourceArtifact,
    SourceChunk, SymbolRecord, MAIN_BRANCH_ID, MAIN_BRANCH_NAME,
};
use rfsource_format::{append_frame, append_frames, initialize, read_frames};
use rfsource_governance::run_checks;

use crate::error::{Result, StoreError};
use crate::tree::resolve_branch;

/// The main RFSource store — wraps a `.rfsource` file.
#[derive(Clone, Debug)]
pub struct RFSource {
    pub(crate) path: PathBuf,
}

/// In-memory representation of a `.rfsource` file's state.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SourceState {
    pub(crate) manifest: Manifest,
    pub(crate) branches: BTreeMap<String, BranchRecord>,
    pub(crate) commits: Vec<CommitRecord>,
    pub(crate) bundles: Vec<CommitBundle>,
    pub(crate) proposals: BTreeMap<String, ProposalRecord>,
    pub(crate) comments: BTreeMap<String, CommentRecord>,
    pub(crate) artifacts: BTreeMap<String, SourceArtifact>,
    pub(crate) chunks: Vec<SourceChunk>,
    pub(crate) symbols: Vec<SymbolRecord>,
}

/// Internal result of building a commit.
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct BuiltCommit {
    pub(crate) bundle: CommitBundle,
    pub(crate) outcome: CommitOutcome,
}

impl RFSource {
    /// Create a new `.rfsource` project at the given path.
    pub fn create(path: impl Into<PathBuf>, project_name: &str) -> Result<Self> {
        let path = path.into();
        initialize(&path)?;

        let manifest = Manifest::new(project_name, short_id("prj", project_name));
        append_frames(&path, &[serde_json::to_value(&manifest)?])?;

        // Create main branch
        let main_branch = BranchRecord {
            branch_id: MAIN_BRANCH_ID.to_string(),
            name: MAIN_BRANCH_NAME.to_string(),
            base_commit_id: None,
            head_commit_id: None,
            parent_branch_id: None,
            status: "active".to_string(),
            actor: "system".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        append_frames(&path, &[serde_json::to_value(&main_branch)?])?;

        Ok(Self { path })
    }

    /// Open an existing `.rfsource` project.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if !path.exists() {
            return Err(RFSourceError::NotFound("RFSource file".to_string(), path.clone()).into());
        }
        Ok(Self { path })
    }

    /// Get the root path.
    pub fn root(&self) -> &Path {
        &self.path
    }

    /// Read the manifest from the `.rfsource` file.
    pub fn manifest(&self) -> Result<Manifest> {
        let frames: Vec<serde_json::Value> = read_frames(&self.path)?;
        frames
            .first()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| {
                RFSourceError::InvalidContainer("No manifest frame found".to_string()).into()
            })
    }

    /// Get project statistics.
    pub fn stats(&self) -> Result<ProjectStats> {
        let state = self.read_valid_state()?;
        Ok(ProjectStats {
            manifest: state.manifest,
            branch_count: state.branches.len(),
            commit_count: state.commits.len(),
            artifact_count: state.artifacts.len(),
            chunk_count: state.chunks.len(),
            symbol_count: state.symbols.len(),
            proposal_count: state.proposals.len(),
            comment_count: state.comments.len(),
        })
    }

    /// Commit an artifact to the main branch.
    ///
    /// Requires `actor_grant` — authorization will be enforced against
    /// the artifact's `allowed_grants`. Pass `Some("*")` for unrestricted
    /// system-level writes.
    pub fn commit_artifact(
        &self,
        req: CommitArtifactRequest,
        actor_grant: &str,
    ) -> Result<CommitOutcome> {
        self.commit_artifact_on_branch(MAIN_BRANCH_NAME, req, Some(actor_grant))
    }

    /// Commit multiple artifacts to the main branch.
    pub fn commit_artifacts(
        &self,
        requests: Vec<CommitArtifactRequest>,
        actor_grant: &str,
    ) -> Result<Vec<CommitOutcome>> {
        requests
            .into_iter()
            .map(|req| self.commit_artifact(req, actor_grant))
            .collect()
    }

    /// Commit an artifact on a specific branch with optional actor grant check.
    pub fn commit_artifact_on_branch(
        &self,
        branch: &str,
        req: CommitArtifactRequest,
        actor_grant: Option<&str>,
    ) -> Result<CommitOutcome> {
        let state = self.read_valid_state()?;

        // Resolve branch
        let branch_record = resolve_branch(&state.branches, branch)
            .ok_or_else(|| StoreError::BranchNotFound(branch.to_string()))?;

        // Grant check for write
        //
        // INVARIANT: Default-deny when allowed_grants is non-empty and actor_grant
        // is not a match. If allowed_grants is empty, the artifact has no grant
        // protections — only grant "*" passes through (F-002 mitigation).
        if let Some(grant) = actor_grant {
            let allowed_contains_grant = req.allowed_grants.contains(&grant.to_string());
            let allowed_contains_wildcard = req.allowed_grants.contains(&"*".to_string());
            if !allowed_contains_grant && !allowed_contains_wildcard {
                // Default-deny: if allowed_grants is non-empty and doesn't match, reject.
                // If allowed_grants is empty, only grant "*" passes (system-level).
                if grant != "*" || !req.allowed_grants.is_empty() {
                    return Err(StoreError::GrantDenied(format!(
                        "Actor grant '{}' not in allowed grants for '{}'",
                        grant, req.logical_path
                    )));
                }
            }
        }

        // Run governance checks
        let object_ref = format!("rfsource://{}", req.logical_path);
        let findings = run_checks(&req.logical_path, &req.content, &object_ref);
        let blocking_findings: Vec<_> = findings.iter().filter(|f| f.blocking).collect();
        if !blocking_findings.is_empty() {
            return Err(StoreError::Governance(format!(
                "Blocking governance findings for '{}': {:?}",
                req.logical_path,
                blocking_findings
                    .iter()
                    .map(|f| &f.rule_id)
                    .collect::<Vec<_>>()
            )));
        }

        // Build commit bundle
        // INVARIANT: req.logical_path is unique per commit within a branch
        let artifact_id = short_id(
            "art",
            &format!(
                "{}@{}",
                req.logical_path,
                branch_record.head_commit_id.as_deref().unwrap_or("root")
            ),
        );
        let commit_id = short_id(
            "cmt",
            &format!(
                "{}-{}",
                req.logical_path,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
        );
        let version_id = short_id("ver", &format!("{}-{}", artifact_id, commit_id));
        let content_hash = sha256_hex(&req.content);

        // Chunk the content
        let lines: Vec<&str> = req.content.lines().collect();
        let line_count = lines.len() as u32;
        let chunk_size = 40;
        let chunks: Vec<SourceChunk> = lines
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk_lines)| {
                let start = (i * chunk_size) as u32 + 1;
                let end = start + chunk_lines.len() as u32 - 1;
                let text = chunk_lines.join("\n");
                SourceChunk {
                    chunk_id: short_id("chk", &format!("{}-{}", version_id, i)),
                    artifact_id: artifact_id.clone(),
                    version_id: version_id.clone(),
                    ordinal: i as u32,
                    line_start: start,
                    line_end: end,
                    text_hash: sha256_hex(&text),
                    text,
                    symbols_defined: vec![],
                    symbols_referenced: vec![],
                }
            })
            .collect();

        let artifact = SourceArtifact {
            artifact_id: artifact_id.clone(),
            logical_path: req.logical_path.clone(),
            language: req.language,
            owner_capability: req.owner_capability,
            risk_level: req.risk_level,
            current_version_id: version_id.clone(),
            policy_bindings: req.policy_bindings,
            allowed_grants: req.allowed_grants,
            required_tests: req.required_tests,
            materialization_mode: "sync".to_string(),
            deleted: false,
        };

        let version = rfsource_core::ArtifactVersion {
            version_id: version_id.clone(),
            artifact_id: artifact_id.clone(),
            parent_version_id: state
                .artifacts
                .get(&artifact_id)
                .map(|a| a.current_version_id.clone()),
            content_hash: content_hash.clone(),
            commit_id: commit_id.clone(),
            created_at: Utc::now(),
            line_count,
            chunk_count: chunks.len() as u32,
        };

        let commit = CommitRecord {
            commit_id: commit_id.clone(),
            sequence: state.commits.len() as u64 + 1,
            parent_commit_id: branch_record.head_commit_id.clone(),
            branch_id: branch_record.branch_id.clone(),
            commit_kind: "change".to_string(),
            rollback_anchor_commit_id: None,
            message: req.message,
            actor: req.actor,
            created_at: Utc::now(),
            artifacts_changed: vec![artifact_id.clone()],
            standards_findings: vec![],
            segment_refs: vec![],
        };

        let bundle = CommitBundle {
            commit: commit.clone(),
            artifact: artifact.clone(),
            version: version.clone(),
            chunks: chunks.clone(),
            symbols: vec![],
            dependencies: vec![],
        };

        // Write the bundle frames
        append_frames(&self.path, &[serde_json::to_value(&bundle)?])?;

        // Update the branch record with the new head commit ID
        let mut updated_branch = branch_record.clone();
        updated_branch.head_commit_id = Some(commit.commit_id.clone());
        updated_branch.updated_at = Utc::now();
        append_frame(&self.path, &updated_branch)?;

        let outcome = CommitOutcome {
            commit,
            artifact,
            version,
            chunk_count: chunks.len(),
            symbol_count: 0,
        };

        Ok(outcome)
    }

    /// Commit multiple artifacts on a branch.
    pub fn commit_artifacts_on_branch(
        &self,
        branch: &str,
        requests: Vec<CommitArtifactRequest>,
        actor_grant: Option<&str>,
    ) -> Result<Vec<CommitOutcome>> {
        requests
            .into_iter()
            .map(|req| self.commit_artifact_on_branch(branch, req, actor_grant))
            .collect()
    }

    /// Get current artifacts map.
    pub fn current_artifacts(&self) -> Result<BTreeMap<String, SourceArtifact>> {
        let state = self.read_valid_state()?;
        Ok(state.artifacts)
    }

    /// Get all chunks.
    pub fn chunks(&self) -> Result<Vec<SourceChunk>> {
        let state = self.read_valid_state()?;
        Ok(state.chunks)
    }

    /// Get all symbols.
    pub fn symbols(&self) -> Result<Vec<SymbolRecord>> {
        let state = self.read_valid_state()?;
        Ok(state.symbols)
    }

    /// Check if a grant is allowed for all artifacts (global check).
    pub fn grant_allows(&self, grant: &str) -> Result<bool> {
        let state = self.read_valid_state()?;
        Ok(state.artifacts.values().all(|a| {
            a.allowed_grants.is_empty()
                || a.allowed_grants.contains(&grant.to_string())
                || a.allowed_grants.contains(&"*".to_string())
        }))
    }

    /// Read and validate the full state from the `.rfsource` file.
    pub(crate) fn read_valid_state(&self) -> Result<SourceState> {
        let frames: Vec<serde_json::Value> = read_frames(&self.path)?;
        let mut manifest: Option<Manifest> = None;
        let mut branches = BTreeMap::new();
        let mut commits = Vec::new();
        let mut bundles = Vec::new();
        let mut proposals = BTreeMap::new();
        let mut comments = BTreeMap::new();
        let mut artifacts = BTreeMap::new();
        let mut chunks = Vec::new();
        let mut symbols = Vec::new();

        for frame in frames {
            // Try to deserialize as each known type
            if let Ok(m) = serde_json::from_value::<Manifest>(frame.clone()) {
                manifest = Some(m);
                continue;
            }
            if let Ok(b) = serde_json::from_value::<BranchRecord>(frame.clone()) {
                branches.insert(b.branch_id.clone(), b);
                continue;
            }
            if let Ok(c) = serde_json::from_value::<CommitBundle>(frame.clone()) {
                artifacts.insert(c.artifact.artifact_id.clone(), c.artifact.clone());
                commits.push(c.commit.clone());
                chunks.extend(c.chunks.clone());
                symbols.extend(c.symbols.clone());
                bundles.push(c);
                continue;
            }
            if let Ok(p) = serde_json::from_value::<ProposalRecord>(frame.clone()) {
                proposals.insert(p.proposal_id.clone(), p);
                continue;
            }
            if let Ok(c) = serde_json::from_value::<CommentRecord>(frame.clone()) {
                comments.insert(c.comment_id.clone(), c);
                continue;
            }
        }

        Ok(SourceState {
            manifest: manifest
                .ok_or_else(|| RFSourceError::InvalidContainer("No manifest".to_string()))?,
            branches,
            commits,
            bundles,
            proposals,
            comments,
            artifacts,
            chunks,
            symbols,
        })
    }
}

/// Compute the SHA-256 hex digest of a string.
pub(crate) fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_open() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let _store = RFSource::create(&path, "test-project").unwrap();
        assert!(path.exists());

        let opened = RFSource::open(&path).unwrap();
        let manifest = opened.manifest().unwrap();
        assert_eq!(manifest.project_name, "test-project");
    }

    #[test]
    fn test_commit_artifact() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test-project").unwrap();

        let outcome = store
            .commit_artifact(
                CommitArtifactRequest {
                    logical_path: "src/main.rs".to_string(),
                    language: "Rust".to_string(),
                    content: "fn main() {}".to_string(),
                    owner_capability: "backend".to_string(),
                    risk_level: "low".to_string(),
                    policy_bindings: vec![],
                    allowed_grants: vec![],
                    required_tests: vec![],
                    actor: "test".to_string(),
                    message: "Initial commit".to_string(),
                },
                "*",
            )
            .unwrap();

        assert_eq!(outcome.artifact.logical_path, "src/main.rs");
        assert_eq!(outcome.chunk_count, 1);
        assert_eq!(outcome.version.line_count, 1);
    }

    #[test]
    fn test_stats() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test-project").unwrap();

        store
            .commit_artifact(
                CommitArtifactRequest {
                    logical_path: "src/main.rs".to_string(),
                    language: "Rust".to_string(),
                    content: "fn main() {}".to_string(),
                    owner_capability: "backend".to_string(),
                    risk_level: "low".to_string(),
                    policy_bindings: vec![],
                    allowed_grants: vec![],
                    required_tests: vec![],
                    actor: "test".to_string(),
                    message: "Initial".to_string(),
                },
                "*",
            )
            .unwrap();

        let stats = store.stats().unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.commit_count, 1);
    }
}
