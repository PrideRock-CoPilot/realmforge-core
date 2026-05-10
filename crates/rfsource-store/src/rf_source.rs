//! RFSource store — source ledger operations with multi-file support (DDR-003).
//!
//! Provides the main `RFSource` struct that wraps read/write operations
//! on `.rfsource` files (single-file mode) or multi-file repositories
//! (multi-file mode for repositories > 1.5 GB).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use rfsource_core::{
    short_id, BranchRecord, CommentRecord, CommitArtifactRequest, CommitBundle, CommitOutcome,
    CommitRecord, Manifest, ProjectStats, ProposalRecord, RFSourceError, SourceArtifact,
    SourceChunk, SymbolRecord, MAIN_BRANCH_ID, MAIN_BRANCH_NAME,
};
use rfsource_format::{
    append_frame, append_frames, detect_repository_mode, initialize, read_frames,
    single_file_path, RepositoryMode,
};
use rfsource_governance::run_checks;

use crate::error::{Result, StoreError};
use crate::multi_file::MultiFileRepo;
use crate::tree::resolve_branch;

/// The main RFSource store — wraps a `.rfsource` file or multi-file repository.
///
/// ## Multi-File Support (DDR-003)
///
/// When the repository directory contains `.rfsource.manifest`, the store operates
/// in multi-file mode with 1 GB segments. Otherwise, it uses the legacy single-file
/// `.rfsource` format.
#[derive(Debug)]
pub struct RFSource {
    pub(crate) path: PathBuf,
    mode: RepositoryMode,
    multi_file: RefCell<Option<MultiFileRepo>>,
}

// Manual Clone implementation since RefCell<Option<MultiFileRepo>> needs special handling
impl Clone for RFSource {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            mode: self.mode,
            multi_file: RefCell::new(None), // Don't clone the multi_file repo
        }
    }
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
    ///
    /// Creates a single-file repository. It will automatically transition to
    /// multi-file mode when size exceeds 1 GB during writes.
    pub fn create(path: impl Into<PathBuf>, project_name: &str) -> Result<Self> {
        let path = path.into();
        let file_path = single_file_path(&path);
        initialize(&file_path)?;

        let manifest = Manifest::new(project_name, short_id("prj", project_name));
        append_frames(&file_path, &[serde_json::to_value(&manifest)?])?;

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
        append_frames(&file_path, &[serde_json::to_value(&main_branch)?])?;

        Ok(Self {
            path,
            mode: RepositoryMode::SingleFile,
            multi_file: RefCell::new(None),
        })
    }

    /// Open an existing `.rfsource` project.
    ///
    /// Automatically detects single-file vs. multi-file mode.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mode = detect_repository_mode(&path);

        // Verify the repository exists
        let file_to_check = match mode {
            RepositoryMode::SingleFile => single_file_path(&path),
            RepositoryMode::MultiFile => path.join(".rfsource.manifest"),
        };

        if !file_to_check.exists() {
            return Err(
                RFSourceError::NotFound("RFSource file".to_string(), file_to_check).into(),
            );
        }

        Ok(Self {
            path,
            mode,
            multi_file: RefCell::new(None),
        })
    }

    /// Get the root path.
    pub fn root(&self) -> &Path {
        &self.path
    }

    /// Get the repository mode (single-file or multi-file).
    pub fn mode(&self) -> RepositoryMode {
        self.mode
    }

    /// Get or initialize the multi-file repo (lazy initialization).
    fn get_multi_file(&self) -> Result<std::cell::Ref<MultiFileRepo>> {
        // Initialize if needed
        if self.multi_file.borrow().is_none() {
            let repo = MultiFileRepo::open_or_init(&self.path)?;
            *self.multi_file.borrow_mut() = Some(repo);
        }

        Ok(std::cell::Ref::map(self.multi_file.borrow(), |opt| {
            opt.as_ref().unwrap()
        }))
    }

    /// Get mutable multi-file repo reference.
    fn get_multi_file_mut(&self) -> Result<std::cell::RefMut<MultiFileRepo>> {
        // Initialize if needed
        if self.multi_file.borrow().is_none() {
            let repo = MultiFileRepo::open_or_init(&self.path)?;
            *self.multi_file.borrow_mut() = Some(repo);
        }

        Ok(std::cell::RefMut::map(self.multi_file.borrow_mut(), |opt| {
            opt.as_mut().unwrap()
        }))
    }

    /// Append a frame to the repository (mode-aware).
    fn append_frame_internal<T: serde::Serialize>(&self, value: &T) -> Result<()> {
        match self.mode {
            RepositoryMode::SingleFile => {
                let file_path = single_file_path(&self.path);
                append_frame(&file_path, value)?;
                Ok(())
            }
            RepositoryMode::MultiFile => {
                let mut repo = self.get_multi_file_mut()?;
                repo.append_frame(value)?;
                Ok(())
            }
        }
    }

    /// Append multiple frames to the repository (mode-aware).
    fn append_frames_internal<T: serde::Serialize>(&self, values: &[T]) -> Result<()> {
        match self.mode {
            RepositoryMode::SingleFile => {
                let file_path = single_file_path(&self.path);
                append_frames(&file_path, values)?;
                Ok(())
            }
            RepositoryMode::MultiFile => {
                let mut repo = self.get_multi_file_mut()?;
                repo.append_frames(values)?;
                Ok(())
            }
        }
    }

    /// Read all frames from the repository (mode-aware).
    fn read_frames_internal(&self) -> Result<Vec<serde_json::Value>> {
        match self.mode {
            RepositoryMode::SingleFile => {
                let file_path = single_file_path(&self.path);
                Ok(read_frames(&file_path)?)
            }
            RepositoryMode::MultiFile => {
                let repo = self.get_multi_file()?;
                Ok(repo.read_all_frames()?)
            }
        }
    }

    /// Read the manifest from the repository.
    pub fn manifest(&self) -> Result<Manifest> {
        let frames = self.read_frames_internal()?;
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

    /// Get the current size of the repository in bytes.
    ///
    /// For single-file mode, returns the `.rfsource` file size.
    /// For multi-file mode, returns the total size across all segments.
    pub fn file_size(&self) -> Result<u64> {
        match self.mode {
            RepositoryMode::SingleFile => {
                let file_path = single_file_path(&self.path);
                Ok(rfsource_format::get_file_size(&file_path)?)
            }
            RepositoryMode::MultiFile => {
                let repo = self.get_multi_file()?;
                Ok(repo.manifest().total_size_bytes)
            }
        }
    }

    /// Get the maximum file size limit in bytes (1.5 GB).
    ///
    /// Note: This applies only to single-file mode. Multi-file mode has no limit.
    pub fn file_size_limit(&self) -> u64 {
        rfsource_format::MAX_FILE_SIZE_BYTES
    }

    /// Get the file size warning threshold in bytes (1 GB).
    pub fn file_size_warning_threshold(&self) -> u64 {
        rfsource_format::FILE_SIZE_WARNING_BYTES
    }

    /// Check if the file size has exceeded the warning threshold.
    pub fn file_size_warning(&self) -> Result<bool> {
        Ok(self.file_size()? > self.file_size_warning_threshold())
    }

    /// Commit an artifact to the main branch.
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
        if let Some(grant) = actor_grant {
            let allowed_contains_grant = req.allowed_grants.contains(&grant.to_string());
            let allowed_contains_wildcard = req.allowed_grants.contains(&"*".to_string());
            if !allowed_contains_grant && !allowed_contains_wildcard {
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

        // Write the bundle frames (mode-aware)
        self.append_frames_internal(&[serde_json::to_value(&bundle)?])?;

        // Update the branch record with the new head commit ID
        let mut updated_branch = branch_record.clone();
        updated_branch.head_commit_id = Some(commit.commit_id.clone());
        updated_branch.updated_at = Utc::now();
        self.append_frame_internal(&updated_branch)?;

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

    /// Read and parse all frames into a complete source state.
    pub(crate) fn read_valid_state(&self) -> Result<SourceState> {
        let frames = self.read_frames_internal()?;
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

        let store = RFSource::create(&path, "test_project").unwrap();
        assert_eq!(store.mode(), RepositoryMode::SingleFile);

        let opened = RFSource::open(&dir.path()).unwrap();
        assert_eq!(opened.mode(), RepositoryMode::SingleFile);

        let manifest = opened.manifest().unwrap();
        assert_eq!(manifest.project_name, "test_project");
    }

    #[test]
    fn test_single_file_mode() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");

        let store = RFSource::create(&path, "test").unwrap();
        assert_eq!(store.mode(), RepositoryMode::SingleFile);
        assert!(store.file_size().unwrap() > 0);
    }
}
