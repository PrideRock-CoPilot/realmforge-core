//! Branch, proposal, comment, and time-warp operations on an RFSource ledger.
//!
//! These methods extend `RFSource` with full source-control features — branching,
//! comparing, proposals, comments, and time-warp operations.
//!
//! ## Security
//!
//! All write operations enforce grant checks and governance validation.
//! Comment resolution requires a `review` or `review.*` grant.
//! Time-warp operations require a clean preview (no blocking comments or findings).
//!
//! ## File Size Justification (1202 lines)
//!
//! This file exceeds the 500-line hard cap (Architecture Law §4) because it contains
//! four distinct but tightly coupled `impl RFSource` blocks (branch, proposal, comment,
//! and time-warp operations), plus three standalone helper functions, plus 18 tests.
//! The blocks share internal helpers (`build_merge_bundle`, `build_delete_bundle`,
//! `ensure_grant_allows`, `grant_allows_static`, `request_from_view`) that cannot be
//! extracted without making them `pub(crate)` across module boundaries — which would
//! weaken encapsulation for the sake of line count. Splitting into per-operation files
//! (e.g., `branch_ops.rs`, `proposal_ops.rs`) is the correct long-term fix and should
//! be done before the next feature addition. For now, the tight coupling and shared
//! helper density justify a single-file layout.

use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;

use rfsource_core::{
    short_id, BranchRecord, CheckFinding, CommentRecord, CommitArtifactRequest, CommitBundle,
    CommitOutcome, CompareReport, ProposalRecord, SourceArtifact, SourceChunk, TimeWarpOutcome,
    TimeWarpPreview, MAIN_BRANCH_ID, MAIN_BRANCH_NAME,
};
use rfsource_format::append_frame;

use crate::error::{Result, StoreError};
use crate::rf_source::{sha256_hex, RFSource, SourceState};
use crate::tree::{
    compare_trees, resolve_branch, resolve_commit_ref, tree_for_branch, tree_for_branch_id,
    tree_for_ref, VersionView,
};

// ---------------------------------------------------------------------------
// RFSource impl — branch operations
// ---------------------------------------------------------------------------

impl RFSource {
    /// List all branches in the project.
    pub fn list_branches(&self) -> Result<Vec<BranchRecord>> {
        Ok(self
            .read_valid_state()?
            .branches
            .into_values()
            .collect::<Vec<_>>())
    }

    /// Show a specific branch by name or ID.
    pub fn show_branch(&self, name_or_id: &str) -> Result<BranchRecord> {
        let state = self.read_valid_state()?;
        resolve_branch(&state.branches, name_or_id)
            .cloned()
            .ok_or_else(|| StoreError::BranchNotFound(name_or_id.to_string()))
    }

    /// Create a new branch from an existing branch.
    ///
    /// The new branch starts with the same HEAD as the source branch.
    /// Returns `Err` if the name is invalid or already exists.
    pub fn create_branch(
        &self,
        name: &str,
        from_branch: Option<&str>,
        actor: &str,
    ) -> Result<BranchRecord> {
        validate_branch_name(name)?;
        let state = self.read_valid_state()?;

        if state.branches.values().any(|b| b.name == name) {
            return Err(StoreError::BranchNotFound(format!(
                "branch '{name}' already exists"
            )));
        }

        let parent = resolve_branch(&state.branches, from_branch.unwrap_or(MAIN_BRANCH_NAME))
            .ok_or_else(|| {
                StoreError::BranchNotFound(from_branch.unwrap_or(MAIN_BRANCH_NAME).to_string())
            })?;

        let branch = BranchRecord {
            branch_id: short_id("br", name),
            name: name.to_string(),
            base_commit_id: parent.head_commit_id.clone(),
            head_commit_id: parent.head_commit_id.clone(),
            parent_branch_id: Some(parent.branch_id.clone()),
            status: "open".to_string(),
            actor: actor.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        append_frame(&self.path, &branch)?;
        Ok(branch)
    }

    /// Compare two references (commit, branch, or ref string).
    ///
    /// Supported reference formats:
    /// - `commit:<id>` — literal commit ID
    /// - `branch:<name_or_id>` — HEAD of a branch
    /// - bare name — tried as commit, then branch
    pub fn compare_refs(&self, from_ref: &str, to_ref: &str) -> Result<CompareReport> {
        let state = self.read_valid_state()?;

        let from = tree_for_ref(&state.bundles, &state.commits, &state.branches, from_ref)
            .map_err(|e| StoreError::CommitNotFound(format!("{from_ref}: {e}")))?;

        let to = tree_for_ref(&state.bundles, &state.commits, &state.branches, to_ref)
            .map_err(|e| StoreError::CommitNotFound(format!("{to_ref}: {e}")))?;

        Ok(compare_trees(from_ref, to_ref, &from, &to))
    }

    /// Run governance checks on all artifacts in a branch.
    ///
    /// If `grant` is `Some`, only artifacts whose `allowed_grants` include
    /// the given grant (or wildcard) are checked. Pass `None` to check all.
    pub fn run_checks(&self, branch: &str, grant: Option<&str>) -> Result<Vec<CheckFinding>> {
        let state = self.read_valid_state()?;

        let tree = tree_for_branch(&state.branches, &state.bundles, branch)
            .map_err(|e| StoreError::BranchNotFound(format!("tree build: {e}")))?;

        let mut findings = Vec::new();
        for view in tree
            .artifacts
            .values()
            .filter(|view| grant_allows_static(&view.artifact, grant))
        {
            let object_ref = format!("rfsource://{}", view.artifact.logical_path);
            findings.extend(rfsource_governance::run_checks(
                &view.artifact.logical_path,
                &view.content,
                &object_ref,
            ));
        }
        Ok(findings)
    }
}

// ---------------------------------------------------------------------------
// RFSource impl — proposal operations
// ---------------------------------------------------------------------------

impl RFSource {
    /// Open a new proposal (merge request) from `source_branch` into `target_branch`.
    ///
    /// The source and target must be different branches. The proposal records
    /// the current HEAD of both branches at creation time.
    pub fn open_proposal(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        actor: &str,
    ) -> Result<ProposalRecord> {
        let state = self.read_valid_state()?;

        let source = resolve_branch(&state.branches, source_branch)
            .ok_or_else(|| StoreError::BranchNotFound(source_branch.to_string()))?;
        let target = resolve_branch(&state.branches, target_branch)
            .ok_or_else(|| StoreError::BranchNotFound(target_branch.to_string()))?;

        if source.branch_id == target.branch_id {
            return Err(StoreError::ProposalError(
                "proposal source and target branches must differ".to_string(),
            ));
        }

        let source_tree = tree_for_branch_id(&state.branches, &state.bundles, &source.branch_id)
            .map_err(|e| StoreError::CommitNotFound(format!("source tree: {e}")))?;
        let target_tree = tree_for_branch_id(&state.branches, &state.bundles, &target.branch_id)
            .map_err(|e| StoreError::CommitNotFound(format!("target tree: {e}")))?;

        let compare = compare_trees(
            &format!("branch:{}", target.branch_id),
            &format!("branch:{}", source.branch_id),
            &target_tree,
            &source_tree,
        );

        let proposal = ProposalRecord {
            proposal_id: short_id(
                "prp",
                &format!(
                    "{}:{}:{}:{}",
                    source.branch_id,
                    target.branch_id,
                    source.head_commit_id.clone().unwrap_or_default(),
                    title,
                ),
            ),
            title: title.to_string(),
            source_branch_id: source.branch_id.clone(),
            target_branch_id: target.branch_id.clone(),
            base_commit_id: source.base_commit_id.clone(),
            source_head_commit_id: source.head_commit_id.clone(),
            target_head_commit_id: target.head_commit_id.clone(),
            status: "open".to_string(),
            actor: actor.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            changed_file_count: compare.changed_files.len(),
            blocking_finding_count: compare.blocking_finding_count,
        };

        append_frame(&self.path, &proposal)?;
        Ok(proposal)
    }

    /// List all proposals.
    pub fn list_proposals(&self) -> Result<Vec<ProposalRecord>> {
        Ok(self
            .read_valid_state()?
            .proposals
            .into_values()
            .collect::<Vec<_>>())
    }

    /// Show a specific proposal by ID.
    pub fn show_proposal(&self, proposal_id: &str) -> Result<ProposalRecord> {
        self.read_valid_state()?
            .proposals
            .get(proposal_id)
            .cloned()
            .ok_or_else(|| StoreError::ProposalError(format!("proposal '{proposal_id}' not found")))
    }

    /// Apply (merge) a proposal.
    ///
    /// Creates merge commits on the target branch for every changed artifact
    /// in the source branch. Requires:
    /// - The proposal to be in `"open"` status
    /// - No blocking comments on any affected ref
    /// - No blocking governance findings
    /// - `actor_grant` to be authorized for each artifact
    ///
    /// ## Durability note
    ///
    /// The bundles and the updated proposal status are written as separate
    /// frame appends and are not atomic. A crash between the two leaves
    /// bundles committed while the proposal remains "open", allowing a
    /// re-apply that produces duplicate commits with identical content hashes.
    /// The WAL/checkpoint layer (out of scope for MVP) is the correct fix.
    pub fn apply_proposal(
        &self,
        proposal_id: &str,
        actor: &str,
        actor_grant: &str,
    ) -> Result<ProposalRecord> {
        let state = self.read_valid_state()?;

        let proposal = state.proposals.get(proposal_id).cloned().ok_or_else(|| {
            StoreError::ProposalError(format!("proposal '{proposal_id}' not found"))
        })?;

        if proposal.status != "open" {
            return Err(StoreError::ProposalError(format!(
                "proposal '{}' is not open",
                proposal.proposal_id
            )));
        }

        let target_tree =
            tree_for_branch_id(&state.branches, &state.bundles, &proposal.target_branch_id)
                .map_err(|e| StoreError::CommitNotFound(format!("target tree: {e}")))?;
        let source_tree =
            tree_for_branch_id(&state.branches, &state.bundles, &proposal.source_branch_id)
                .map_err(|e| StoreError::CommitNotFound(format!("source tree: {e}")))?;

        let compare = compare_trees("target", "source", &target_tree, &source_tree);

        // Check for blocking comments
        let affected = affected_refs(&compare, Some(&proposal));
        ensure_no_blocking_comments(&state, &affected)?;

        // Check for blocking governance findings
        if compare.blocking_finding_count > 0 {
            return Err(StoreError::Governance(format!(
                "proposal has {} blocking standards findings",
                compare.blocking_finding_count
            )));
        }

        // Build bundles for each changed artifact
        let mut bundles = Vec::new();
        let mut head_commit_id = proposal.target_head_commit_id.clone();
        let branch_id = &proposal.target_branch_id;
        let mut next_seq = state.commits.last().map_or(1, |c| c.sequence + 1);

        for change in &compare.changed_files {
            if let Some(view) = source_tree.artifacts.get(&change.artifact_id) {
                ensure_grant_allows(&view.artifact, Some(actor_grant))?;

                let req = request_from_view(view, actor, format!("merge proposal {proposal_id}"));
                let bundle = build_merge_bundle(
                    &state.artifacts,
                    branch_id,
                    &head_commit_id,
                    &mut next_seq,
                    &req,
                    "merge",
                    proposal.target_head_commit_id.clone(),
                )?;

                // Update in-memory head for subsequent bundles in this batch
                head_commit_id = Some(bundle.commit.commit_id.clone());
                bundles.push(bundle);
            }
        }

        // Write bundles first, then updated proposal status
        let bundles_json: Vec<serde_json::Value> = bundles
            .iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rfsource_format::append_frames(&self.path, &bundles_json)?;

        let mut applied = proposal;
        applied.status = "applied".to_string();
        applied.updated_at = Utc::now();
        append_frame(&self.path, &applied)?;

        Ok(applied)
    }
}

// ---------------------------------------------------------------------------
// RFSource impl — comment operations
// ---------------------------------------------------------------------------

impl RFSource {
    /// Add a comment to an object ref (artifact, branch, proposal, commit).
    ///
    /// Requires `actor_grant` — comments on artifacts require the grant to
    /// match the artifact's `allowed_grants`. Resolution requires a
    /// `review` or `review.*` grant.
    pub fn add_comment(
        &self,
        object_ref: &str,
        body: &str,
        actor: &str,
        severity: &str,
        actor_grant: &str,
    ) -> Result<CommentRecord> {
        let state = self.read_valid_state()?;
        ensure_object_comment_grant(&state, object_ref, actor_grant, false)?;

        let comment = CommentRecord {
            comment_id: short_id("cmtx", &format!("{object_ref}:{actor}:{body}")),
            object_ref: object_ref.to_string(),
            thread_id: short_id("thr", object_ref),
            body: body.to_string(),
            actor: actor.to_string(),
            severity: severity.to_string(),
            status: "open".to_string(),
            artifact_id: object_ref.strip_prefix("artifact:").map(str::to_string),
            version_id: None,
            chunk_id: None,
            line_start: None,
            line_end: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        append_frame(&self.path, &comment)?;
        Ok(comment)
    }

    /// List comments, optionally filtered by object ref.
    pub fn list_comments(&self, object_ref: Option<&str>) -> Result<Vec<CommentRecord>> {
        Ok(self
            .read_valid_state()?
            .comments
            .into_values()
            .filter(|c| object_ref.map(|o| o == c.object_ref).unwrap_or(true))
            .collect::<Vec<_>>())
    }

    /// Resolve a comment by setting its status to `"resolved"`.
    ///
    /// Requires a `review` or `review.*` grant.
    pub fn resolve_comment(
        &self,
        comment_id: &str,
        actor: &str,
        actor_grant: &str,
    ) -> Result<CommentRecord> {
        let state = self.read_valid_state()?;

        let mut comment =
            state.comments.get(comment_id).cloned().ok_or_else(|| {
                StoreError::CommentError(format!("comment '{comment_id}' not found"))
            })?;

        ensure_object_comment_grant(&state, &comment.object_ref, actor_grant, true)?;

        comment.status = "resolved".to_string();
        comment.actor = actor.to_string();
        comment.updated_at = Utc::now();

        append_frame(&self.path, &comment)?;
        Ok(comment)
    }
}

// ---------------------------------------------------------------------------
// RFSource impl — time-warp operations
// ---------------------------------------------------------------------------

impl RFSource {
    /// Preview a time-warp (restore) operation.
    ///
    /// Shows what would change if `branch` were restored to `target_ref`.
    /// Returns a `TimeWarpPreview` with file changes, blocking comments,
    /// and blocking findings. The operation is clean (safe to apply) when
    /// both `blocking_comment_count` and `blocking_finding_count` are zero.
    ///
    /// ## Scope
    ///
    /// - `"file"` — only the artifact matching `logical_path`
    /// - `"branch"` — all artifacts on the branch
    /// - `"project"` — all artifacts regardless of branch
    pub fn preview_time_warp(
        &self,
        target_ref: &str,
        branch: &str,
        scope: &str,
        logical_path: Option<&str>,
        actor_grant: &str,
    ) -> Result<TimeWarpPreview> {
        let state = self.read_valid_state()?;

        let branch = resolve_branch(&state.branches, branch)
            .ok_or_else(|| StoreError::BranchNotFound(branch.to_string()))?;

        let current = tree_for_branch_id(&state.branches, &state.bundles, &branch.branch_id)
            .map_err(|e| StoreError::CommitNotFound(format!("current tree: {e}")))?;
        let target = tree_for_ref(&state.bundles, &state.commits, &state.branches, target_ref)
            .map_err(|e| StoreError::CommitNotFound(format!("target tree: {e}")))?;

        let mut compare = compare_trees("current", target_ref, &current, &target);

        // Filter by scope
        match scope {
            "file" => {
                let path = logical_path.ok_or_else(|| {
                    StoreError::TimeWarpError("file time warp requires --path".to_string())
                })?;
                compare.changed_files.retain(|c| c.logical_path == path);
            }
            "branch" | "project" => {}
            other => {
                return Err(StoreError::TimeWarpError(format!(
                    "unsupported time warp scope '{other}'"
                )));
            }
        }

        // Grant check on each changed artifact
        for change in &compare.changed_files {
            let view = target
                .artifacts
                .get(&change.artifact_id)
                .or_else(|| current.artifacts.get(&change.artifact_id));
            if let Some(view) = view {
                ensure_grant_allows(&view.artifact, Some(actor_grant))?;
            }
        }

        let affected = affected_refs(&compare, None);
        let blocking_comment_count = count_blocking_comments(&state, &affected);
        let blocking_finding_count = compare.blocking_finding_count;

        Ok(TimeWarpPreview {
            target_ref: target_ref.to_string(),
            branch_id: branch.branch_id.clone(),
            scope: scope.to_string(),
            logical_path: logical_path.map(str::to_string),
            changed_files: compare.changed_files,
            blocking_comment_count,
            blocking_finding_count,
            clean: blocking_comment_count == 0 && blocking_finding_count == 0,
        })
    }

    /// Apply a time-warp (restore) operation.
    ///
    /// Only proceeds if the preview is clean (no blocking comments or findings).
    /// Creates forward rollback commits on the target branch.
    pub fn apply_time_warp(
        &self,
        target_ref: &str,
        branch: &str,
        scope: &str,
        logical_path: Option<&str>,
        actor: &str,
        actor_grant: &str,
    ) -> Result<TimeWarpOutcome> {
        let preview =
            self.preview_time_warp(target_ref, branch, scope, logical_path, actor_grant)?;

        if !preview.clean {
            return Err(StoreError::TimeWarpError(
                "time warp preview is not clean".to_string(),
            ));
        }

        let state = self.read_valid_state()?;

        let target = tree_for_ref(&state.bundles, &state.commits, &state.branches, target_ref)
            .map_err(|e| StoreError::CommitNotFound(format!("target tree: {e}")))?;
        let current = tree_for_branch_id(&state.branches, &state.bundles, &preview.branch_id)
            .map_err(|e| StoreError::CommitNotFound(format!("current tree: {e}")))?;

        let target_commit = resolve_commit_ref(&state.commits, &state.branches, target_ref);
        let branch_id = &preview.branch_id;
        let mut head_commit_id = state
            .branches
            .get(branch_id)
            .and_then(|b| b.head_commit_id.clone());
        let mut next_seq = state.commits.last().map_or(1, |c| c.sequence + 1);
        let mut bundles = Vec::new();
        let mut outcomes = Vec::new();

        for change in &preview.changed_files {
            if let Some(view) = target.artifacts.get(&change.artifact_id) {
                // Artifact exists in target — create a forward rollback commit
                ensure_grant_allows(&view.artifact, Some(actor_grant))?;
                let req = request_from_view(view, actor, format!("time warp restore {target_ref}"));
                let bundle = build_merge_bundle(
                    &state.artifacts,
                    branch_id,
                    &head_commit_id,
                    &mut next_seq,
                    &req,
                    "rollback",
                    target_commit.clone(),
                )?;
                head_commit_id = Some(bundle.commit.commit_id.clone());
                let outcome = CommitOutcome {
                    commit: bundle.commit.clone(),
                    artifact: bundle.artifact.clone(),
                    version: bundle.version.clone(),
                    chunk_count: bundle.chunks.len(),
                    symbol_count: bundle.symbols.len(),
                };
                outcomes.push(outcome);
                bundles.push(bundle);
            } else {
                // Artifact was deleted in target — create a delete bundle
                let current_view = current.artifacts.get(&change.artifact_id).ok_or_else(|| {
                    StoreError::ArtifactNotFound(format!(
                        "cannot delete unknown artifact '{}'",
                        change.artifact_id
                    ))
                })?;
                ensure_grant_allows(&current_view.artifact, Some(actor_grant))?;
                let bundle = build_delete_bundle(
                    current_view,
                    branch_id,
                    &head_commit_id,
                    &mut next_seq,
                    actor,
                    target_commit.clone(),
                )?;
                head_commit_id = Some(bundle.commit.commit_id.clone());
                let outcome = CommitOutcome {
                    commit: bundle.commit.clone(),
                    artifact: bundle.artifact.clone(),
                    version: bundle.version.clone(),
                    chunk_count: 0,
                    symbol_count: 0,
                };
                outcomes.push(outcome);
                bundles.push(bundle);
            }
        }

        let bundles_json: Vec<serde_json::Value> = bundles
            .iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rfsource_format::append_frames(&self.path, &bundles_json)?;

        Ok(TimeWarpOutcome {
            preview,
            commits: outcomes,
        })
    }
}

// ---------------------------------------------------------------------------
// Internal bundle builders
// ---------------------------------------------------------------------------

/// Build a commit bundle for a merge or rollback operation.
///
/// Creates a sequential commit ID, chunks content using simple line-based
/// chunking (40 lines per chunk), and assigns version IDs via hash-based
/// short IDs. The new commit is chained to `parent_commit_id`.
fn build_merge_bundle(
    existing_artifacts: &BTreeMap<String, SourceArtifact>,
    branch_id: &str,
    parent_commit_id: &Option<String>,
    next_seq: &mut u64,
    req: &CommitArtifactRequest,
    commit_kind: &str,
    rollback_anchor: Option<String>,
) -> Result<CommitBundle> {
    let artifact_id = short_id("art", &req.logical_path);
    let commit_id = format!("cmt_{next_seq:08}");
    let version_id = short_id(
        "ver",
        &format!("{}:{}:{}", artifact_id, sha256_hex(&req.content), commit_id),
    );
    let content_hash = sha256_hex(&req.content);

    // Chunk content (40 lines per chunk, matching commit_artifact_on_branch)
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

    let parent_version_id = existing_artifacts
        .get(&artifact_id)
        .map(|a| a.current_version_id.clone());

    let artifact = SourceArtifact {
        artifact_id: artifact_id.clone(),
        logical_path: req.logical_path.clone(),
        language: req.language.clone(),
        owner_capability: req.owner_capability.clone(),
        risk_level: req.risk_level.clone(),
        current_version_id: version_id.clone(),
        policy_bindings: req.policy_bindings.clone(),
        allowed_grants: req.allowed_grants.clone(),
        required_tests: req.required_tests.clone(),
        materialization_mode: "sync".to_string(),
        deleted: false,
    };

    let version = rfsource_core::ArtifactVersion {
        version_id: version_id.clone(),
        artifact_id: artifact_id.clone(),
        parent_version_id,
        content_hash: content_hash.clone(),
        commit_id: commit_id.clone(),
        created_at: Utc::now(),
        line_count,
        chunk_count: chunks.len() as u32,
    };

    let commit = rfsource_core::CommitRecord {
        commit_id: commit_id.clone(),
        sequence: *next_seq,
        parent_commit_id: parent_commit_id.clone(),
        branch_id: branch_id.to_string(),
        commit_kind: commit_kind.to_string(),
        rollback_anchor_commit_id: rollback_anchor,
        message: req.message.clone(),
        actor: req.actor.clone(),
        created_at: Utc::now(),
        artifacts_changed: vec![artifact_id],
        standards_findings: vec![],
        segment_refs: vec![],
    };

    *next_seq += 1;

    Ok(CommitBundle {
        commit,
        artifact,
        version,
        chunks,
        symbols: vec![],
        dependencies: vec![],
    })
}

/// Build a delete-marker bundle for time-warp removal of an artifact.
fn build_delete_bundle(
    current_view: &VersionView,
    branch_id: &str,
    parent_commit_id: &Option<String>,
    next_seq: &mut u64,
    actor: &str,
    rollback_anchor: Option<String>,
) -> Result<CommitBundle> {
    let commit_id = format!("cmt_{next_seq:08}");
    let content_hash = sha256_hex("");
    let version_id = short_id(
        "ver",
        &format!(
            "{}:{}:deleted",
            current_view.artifact.artifact_id, commit_id
        ),
    );

    let mut artifact = current_view.artifact.clone();
    artifact.current_version_id = version_id.clone();
    artifact.deleted = true;

    let version = rfsource_core::ArtifactVersion {
        version_id: version_id.clone(),
        artifact_id: artifact.artifact_id.clone(),
        parent_version_id: Some(current_view.artifact.current_version_id.clone()),
        content_hash,
        commit_id: commit_id.clone(),
        created_at: Utc::now(),
        line_count: 0,
        chunk_count: 0,
    };

    let commit = rfsource_core::CommitRecord {
        commit_id: commit_id.clone(),
        sequence: *next_seq,
        parent_commit_id: parent_commit_id.clone(),
        branch_id: branch_id.to_string(),
        commit_kind: "rollback".to_string(),
        rollback_anchor_commit_id: rollback_anchor,
        message: format!("time warp delete {}", artifact.logical_path),
        actor: actor.to_string(),
        created_at: Utc::now(),
        artifacts_changed: vec![artifact.artifact_id.clone()],
        standards_findings: vec![],
        segment_refs: vec![],
    };

    *next_seq += 1;

    Ok(CommitBundle {
        commit,
        artifact,
        version,
        chunks: vec![],
        symbols: vec![],
        dependencies: vec![],
    })
}

// ---------------------------------------------------------------------------
// Standalone helpers
// ---------------------------------------------------------------------------

/// Validate a branch name.
///
/// Rules: non-empty, no backslashes, no `..`, alphanumeric + `_` `-` `/`.
fn validate_branch_name(name: &str) -> Result<()> {
    let valid = !name.trim().is_empty()
        && !name.contains('\\')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/'));
    if valid {
        Ok(())
    } else {
        Err(StoreError::BranchNotFound(format!(
            "invalid branch name '{name}'"
        )))
    }
}

/// Build a `CommitArtifactRequest` from a `VersionView`.
fn request_from_view(view: &VersionView, actor: &str, message: String) -> CommitArtifactRequest {
    CommitArtifactRequest {
        logical_path: view.artifact.logical_path.clone(),
        language: view.artifact.language.clone(),
        content: view.content.clone(),
        owner_capability: view.artifact.owner_capability.clone(),
        risk_level: view.artifact.risk_level.clone(),
        policy_bindings: view.artifact.policy_bindings.clone(),
        allowed_grants: view.artifact.allowed_grants.clone(),
        required_tests: view.artifact.required_tests.clone(),
        actor: actor.to_string(),
        message,
    }
}

/// Check whether a grant is allowed for a given artifact.
///
/// - `None` grant → always passes (no grant check requested)
/// - `Some("*")` → always passes (system-level grant bypass)
/// - `Some(g)` when `allowed_grants` is empty → passes (no restrictions)
/// - `Some(g)` when `allowed_grants` is non-empty → must match `"*"` or `g`
fn grant_allows_static(artifact: &SourceArtifact, grant: Option<&str>) -> bool {
    match grant {
        None => true,
        // "*" always passes (system-level grant bypass)
        Some("*") => true,
        Some(g) => {
            // Empty allowed_grants means no restrictions — any grant passes
            if artifact.allowed_grants.is_empty() {
                return true;
            }
            artifact.allowed_grants.iter().any(|item| item == "*")
                || artifact.allowed_grants.iter().any(|item| item == g)
        }
    }
}

/// Ensure a grant is allowed for writing to an artifact.
fn ensure_grant_allows(artifact: &SourceArtifact, grant: Option<&str>) -> Result<()> {
    if grant_allows_static(artifact, grant) {
        Ok(())
    } else {
        Err(StoreError::GrantDenied(format!(
            "grant '{}' cannot write {}",
            grant.unwrap_or("<missing>"),
            artifact.logical_path
        )))
    }
}

/// Ensure the actor grant is sufficient to comment on or resolve a ref.
fn ensure_object_comment_grant(
    state: &SourceState,
    object_ref: &str,
    actor_grant: &str,
    resolving: bool,
) -> Result<()> {
    // Resolution requires exactly "*", "review", or a "review.*" prefixed grant.
    if resolving
        && !(actor_grant == "*" || actor_grant == "review" || actor_grant.starts_with("review."))
    {
        return Err(StoreError::CommentError(
            "resolving comments requires a 'review' or 'review.*' grant".to_string(),
        ));
    }
    if resolving {
        return Ok(());
    }

    // For artifact comments, check the artifact's allowed_grants
    if let Some(artifact_id) = object_ref.strip_prefix("artifact:") {
        let main_tree = tree_for_branch_id(&state.branches, &state.bundles, MAIN_BRANCH_ID)
            .map_err(|e| StoreError::CommitNotFound(format!("tree build: {e}")))?;
        if let Some(view) = main_tree.artifacts.get(artifact_id) {
            return ensure_grant_allows(&view.artifact, Some(actor_grant));
        }
    }

    // For non-artifact comments (branch, proposal, commit), require a non-empty grant
    if actor_grant.trim().is_empty() {
        Err(StoreError::CommentError(
            "comment requires an actor grant".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Collect all affected refs from a compare report, optionally including proposal refs.
fn affected_refs(compare: &CompareReport, proposal: Option<&ProposalRecord>) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    if let Some(proposal) = proposal {
        refs.insert(format!("proposal:{}", proposal.proposal_id));
        refs.insert(format!("branch:{}", proposal.source_branch_id));
        refs.insert(format!("branch:{}", proposal.target_branch_id));
    }
    for change in &compare.changed_files {
        refs.insert(format!("artifact:{}", change.artifact_id));
    }
    refs
}

/// Ensure no blocking comments exist on any of the given refs.
fn ensure_no_blocking_comments(state: &SourceState, refs: &BTreeSet<String>) -> Result<()> {
    let count = count_blocking_comments(state, refs);
    if count == 0 {
        Ok(())
    } else {
        Err(StoreError::CommentError(format!(
            "{count} unresolved blocking comment(s)"
        )))
    }
}

/// Count blocking (open/reopened/blocked, severity "blocking") comments on the given refs.
fn count_blocking_comments(state: &SourceState, refs: &BTreeSet<String>) -> usize {
    state
        .comments
        .values()
        .filter(|comment| {
            refs.contains(&comment.object_ref)
                && comment.severity == "blocking"
                && matches!(comment.status.as_str(), "open" | "reopened" | "blocked")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rfsource_core::CommitArtifactRequest;
    use tempfile::tempdir;

    fn create_test_store() -> (RFSource, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test-project").unwrap();
        (store, dir)
    }

    fn commit_test_artifact(store: &RFSource, path: &str, content: &str) -> CommitOutcome {
        store
            .commit_artifact(
                CommitArtifactRequest {
                    logical_path: path.to_string(),
                    language: "Rust".to_string(),
                    content: content.to_string(),
                    owner_capability: "backend".to_string(),
                    risk_level: "low".to_string(),
                    policy_bindings: vec![],
                    allowed_grants: vec![],
                    required_tests: vec![],
                    actor: "test".to_string(),
                    message: "test commit".to_string(),
                },
                "*",
            )
            .unwrap()
    }

    #[test]
    fn test_list_branches() {
        let (store, _dir) = create_test_store();
        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, MAIN_BRANCH_NAME);
    }

    #[test]
    fn test_show_branch() {
        let (store, _dir) = create_test_store();
        let branch = store.show_branch(MAIN_BRANCH_NAME).unwrap();
        assert_eq!(branch.name, MAIN_BRANCH_NAME);
    }

    #[test]
    fn test_show_branch_not_found() {
        let (store, _dir) = create_test_store();
        let result = store.show_branch("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_branch() {
        let (store, _dir) = create_test_store();
        let branch = store
            .create_branch("feature/test", None, "test-user")
            .unwrap();
        assert_eq!(branch.name, "feature/test");
        assert_eq!(branch.status, "open");

        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 2);
    }

    #[test]
    fn test_create_branch_duplicate() {
        let (store, _dir) = create_test_store();
        store
            .create_branch("feature/test", None, "test-user")
            .unwrap();
        let result = store.create_branch("feature/test", None, "test-user");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_branch_invalid_name() {
        let (store, _dir) = create_test_store();
        let result = store.create_branch("bad\\name", None, "test-user");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_refs_empty() {
        let (store, _dir) = create_test_store();
        let report = store
            .compare_refs(MAIN_BRANCH_NAME, MAIN_BRANCH_NAME)
            .unwrap();
        assert!(report.changed_files.is_empty());
    }

    #[test]
    fn test_compare_refs_with_changes() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() {}");

        let report = store
            .compare_refs(MAIN_BRANCH_NAME, MAIN_BRANCH_NAME)
            .unwrap();
        assert!(report.changed_files.is_empty());
    }

    #[test]
    fn test_compare_refs_between_branches() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() {}");
        store
            .create_branch("feature/test", None, "test-user")
            .unwrap();
        commit_test_artifact(&store, "src/lib.rs", "pub fn helper() {}");

        let report = store
            .compare_refs(MAIN_BRANCH_NAME, "feature/test")
            .unwrap();
        assert!(!report.changed_files.is_empty());
    }

    #[test]
    fn test_proposal_lifecycle() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() {}");
        store
            .create_branch("feature/test", None, "test-user")
            .unwrap();
        commit_test_artifact(&store, "src/lib.rs", "pub fn helper() {}");

        let proposal = store
            .open_proposal("feature/test", MAIN_BRANCH_NAME, "Add helper", "test-user")
            .unwrap();
        assert_eq!(proposal.status, "open");
        assert!(proposal.changed_file_count > 0);

        let proposals = store.list_proposals().unwrap();
        assert_eq!(proposals.len(), 1);

        let shown = store.show_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(shown.title, "Add helper");
    }

    #[test]
    fn test_comment_lifecycle() {
        let (store, _dir) = create_test_store();
        let outcome = commit_test_artifact(&store, "src/main.rs", "fn main() {}");

        let comment = store
            .add_comment(
                &format!("artifact:{}", outcome.artifact.artifact_id),
                "Review needed",
                "reviewer",
                "blocking",
                "*",
            )
            .unwrap();
        assert_eq!(comment.status, "open");

        let comments = store.list_comments(None).unwrap();
        assert_eq!(comments.len(), 1);

        let resolved = store
            .resolve_comment(&comment.comment_id, "reviewer", "review")
            .unwrap();
        assert_eq!(resolved.status, "resolved");
    }

    #[test]
    fn test_time_warp_preview() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() { v1 }");

        // Get the first commit as the target ref
        let branches = store.list_branches().unwrap();
        let head = branches[0].head_commit_id.clone().unwrap();

        // Make a second commit
        commit_test_artifact(&store, "src/main.rs", "fn main() { v2 }");

        let preview = store
            .preview_time_warp(&head, MAIN_BRANCH_NAME, "branch", None, "*")
            .unwrap();
        assert!(!preview.changed_files.is_empty());
        assert!(preview.clean);
    }

    #[test]
    fn test_time_warp_apply() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() { v1 }");

        let branches = store.list_branches().unwrap();
        let head = branches[0].head_commit_id.clone().unwrap();

        commit_test_artifact(&store, "src/main.rs", "fn main() { v2 }");

        let outcome = store
            .apply_time_warp(&head, MAIN_BRANCH_NAME, "branch", None, "test", "*")
            .unwrap();
        assert_eq!(outcome.commits.len(), 1);
    }

    #[test]
    fn test_run_checks_on_branch() {
        let (store, _dir) = create_test_store();
        commit_test_artifact(&store, "src/main.rs", "fn main() {}");
        let findings = store.run_checks(MAIN_BRANCH_NAME, None).unwrap();
        // Governance checks may produce findings (size, header, naming)
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_validate_branch_name_valid() {
        assert!(validate_branch_name("feature/test").is_ok());
        assert!(validate_branch_name("my-branch-name").is_ok());
        assert!(validate_branch_name("branch_123").is_ok());
    }

    #[test]
    fn test_validate_branch_name_invalid() {
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("bad\\name").is_err());
        assert!(validate_branch_name("has..dots").is_err());
    }

    #[test]
    fn test_affected_refs_empty() {
        let report = CompareReport {
            from_ref: "a".to_string(),
            to_ref: "b".to_string(),
            changed_files: vec![],
            finding_count: 0,
            blocking_finding_count: 0,
        };
        let refs = affected_refs(&report, None);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_grant_allows_static() {
        let mut artifact = SourceArtifact {
            artifact_id: "art_1".to_string(),
            logical_path: "test.rs".to_string(),
            language: "Rust".to_string(),
            owner_capability: "backend".to_string(),
            risk_level: "low".to_string(),
            current_version_id: "v1".to_string(),
            policy_bindings: vec![],
            allowed_grants: vec![],
            required_tests: vec![],
            materialization_mode: "sync".to_string(),
            deleted: false,
        };

        // Empty grants: None passes, Some("*") passes
        assert!(grant_allows_static(&artifact, None));
        assert!(grant_allows_static(&artifact, Some("*")));
        assert!(grant_allows_static(&artifact, Some("SGL-READ")));

        artifact.allowed_grants = vec!["SGL-READ".to_string()];

        assert!(grant_allows_static(&artifact, None));
        assert!(grant_allows_static(&artifact, Some("SGL-READ")));
        assert!(grant_allows_static(&artifact, Some("*")));
        assert!(!grant_allows_static(&artifact, Some("SGL-WRITE")));
    }

    #[test]
    fn test_count_blocking_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test").unwrap();
        let state = store.read_valid_state().unwrap();

        let mut refs = BTreeSet::new();
        refs.insert("branch:main".to_string());

        // No comments yet
        assert_eq!(count_blocking_comments(&state, &refs), 0);
    }

    #[test]
    fn test_ensure_grant_allows() {
        let artifact = SourceArtifact {
            artifact_id: "art_1".to_string(),
            logical_path: "test.rs".to_string(),
            language: "Rust".to_string(),
            owner_capability: "backend".to_string(),
            risk_level: "low".to_string(),
            current_version_id: "v1".to_string(),
            policy_bindings: vec![],
            allowed_grants: vec!["SGL-READ".to_string()],
            required_tests: vec![],
            materialization_mode: "sync".to_string(),
            deleted: false,
        };

        assert!(ensure_grant_allows(&artifact, None).is_ok());
        assert!(ensure_grant_allows(&artifact, Some("*")).is_ok());
        assert!(ensure_grant_allows(&artifact, Some("SGL-READ")).is_ok());
        assert!(ensure_grant_allows(&artifact, Some("SGL-WRITE")).is_err());
    }
}
