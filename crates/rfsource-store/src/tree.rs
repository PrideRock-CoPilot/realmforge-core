//! Version tree construction, traversal, and comparison helpers.
//!
//! Every read path that needs artifact state at a given commit or branch HEAD
//! goes through the functions here. `compare_trees` drives diff and governance checks.
//!
//! ## Two modes
//!
//! - **Bundle-based** (new): reconstructs full content from `CommitBundle` frames,
//!   used by source-control operations (branches, proposals, time warp, compare).
//! - **Artifact-based** (legacy): works from `CommitRecord` + `SourceArtifact` maps,
//!   used by catalog queries that need tree structure without content.

use std::collections::{BTreeMap, BTreeSet};

use rfsource_core::{
    BranchRecord, ChangedFile, CommitBundle, CommitRecord, CompareReport, LineHunk, SourceArtifact,
    SourceChunk, SymbolRecord,
};

/// A view of an artifact at a specific point in the version tree.
#[derive(Clone, Debug)]
pub struct VersionView {
    pub artifact: SourceArtifact,
    pub version_id: String,
    pub content_hash: String,
    /// Reconstructed full content from chunks (empty for artifact-based trees).
    pub content: String,
    /// Chunks that make up the content (empty for artifact-based trees).
    pub chunks: Vec<SourceChunk>,
    /// Symbols defined in this version (empty for artifact-based trees).
    pub symbols: Vec<SymbolRecord>,
}

/// The version tree — a mapping from artifact_id to its state at a given point.
#[derive(Clone, Debug)]
pub struct VersionTree {
    pub artifacts: BTreeMap<String, VersionView>,
}

// ---------------------------------------------------------------------------
// Legacy artifact-based helpers (kept for backward compatibility)
// ---------------------------------------------------------------------------

/// Derive the heads (latest commit) for each branch.
pub fn derive_branch_heads(branches: &BTreeMap<String, BranchRecord>) -> BTreeMap<String, String> {
    let mut heads = BTreeMap::new();
    for (branch_id, branch) in branches {
        if let Some(ref head) = branch.head_commit_id {
            heads.insert(branch_id.clone(), head.clone());
        }
    }
    heads
}

/// Find a branch by name in the branches map.
pub fn resolve_branch<'a>(
    branches: &'a BTreeMap<String, BranchRecord>,
    name: &str,
) -> Option<&'a BranchRecord> {
    let raw = name.strip_prefix("branch:").unwrap_or(name);
    branches
        .get(raw)
        .or_else(|| branches.values().find(|b| b.name == raw))
}

// ---------------------------------------------------------------------------
// Bundle-based tree construction (full content reconstruction)
// ---------------------------------------------------------------------------

/// Build the version tree for a specific branch by walking its bundles.
pub fn tree_for_branch(
    branches: &BTreeMap<String, BranchRecord>,
    bundles: &[CommitBundle],
    branch_name: &str,
) -> Result<VersionTree, String> {
    let branch = resolve_branch(branches, branch_name)
        .ok_or_else(|| format!("branch '{branch_name}' not found"))?;
    let _branch_id = &branch.branch_id;
    match &branch.head_commit_id {
        Some(commit_id) => tree_at_commit(bundles, commit_id),
        None => Ok(VersionTree {
            artifacts: BTreeMap::new(),
        }),
    }
}

/// Build the version tree for a specific branch by ID.
pub fn tree_for_branch_id(
    branches: &BTreeMap<String, BranchRecord>,
    bundles: &[CommitBundle],
    branch_id: &str,
) -> Result<VersionTree, String> {
    let branch = branches
        .get(branch_id)
        .ok_or_else(|| format!("branch '{branch_id}' not found"))?;
    match &branch.head_commit_id {
        Some(commit_id) => tree_at_commit(bundles, commit_id),
        None => Ok(VersionTree {
            artifacts: BTreeMap::new(),
        }),
    }
}

/// Build the version tree from parts (convenience wrapper).
pub fn tree_for_branch_id_from_parts(
    branches: &BTreeMap<String, BranchRecord>,
    bundles: &[CommitBundle],
    branch_id: &str,
) -> Result<VersionTree, String> {
    tree_for_branch_id(branches, bundles, branch_id)
}

/// Build the version tree at a specific commit by walking the parent chain.
pub fn tree_at_commit(bundles: &[CommitBundle], commit_id: &str) -> Result<VersionTree, String> {
    let bundle_map: BTreeMap<&str, &CommitBundle> = bundles
        .iter()
        .map(|b| (b.commit.commit_id.as_str(), b))
        .collect();
    let mut seen = BTreeSet::new();
    let mut chain = Vec::new();
    let mut cursor = Some(commit_id);

    while let Some(current) = cursor {
        if !seen.insert(current) {
            return Err(format!("commit parent cycle at {current}"));
        }
        let bundle = bundle_map
            .get(current)
            .ok_or_else(|| format!("commit '{current}' not found in bundle stream"))?;
        chain.push(*bundle);
        cursor = bundle.commit.parent_commit_id.as_deref();
    }

    chain.reverse();
    let mut tree = BTreeMap::new();
    for bundle in chain {
        apply_bundle_to_tree(&mut tree, bundle);
    }
    Ok(VersionTree { artifacts: tree })
}

/// Apply a commit bundle to the version tree (mutates the tree in place).
/// Reconstructs full content from chunks.
fn apply_bundle_to_tree(tree: &mut BTreeMap<String, VersionView>, bundle: &CommitBundle) {
    if bundle.artifact.deleted {
        tree.remove(&bundle.artifact.artifact_id);
        return;
    }
    let mut content = bundle
        .chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    let view = VersionView {
        artifact: bundle.artifact.clone(),
        version_id: bundle.version.version_id.clone(),
        content_hash: bundle.version.content_hash.clone(),
        content,
        chunks: bundle.chunks.clone(),
        symbols: bundle.symbols.clone(),
    };
    tree.insert(bundle.artifact.artifact_id.clone(), view);
}

/// Resolve a reference string to a commit ID.
///
/// Supports:
/// - `commit:<id>` — literal commit
/// - `branch:<name_or_id>` — head of branch
/// - bare name — tried as commit, then branch
pub fn resolve_commit_ref(
    commits: &[CommitRecord],
    branches: &BTreeMap<String, BranchRecord>,
    reference: &str,
) -> Option<String> {
    let raw = reference.strip_prefix("commit:").unwrap_or(reference);
    if commits.iter().any(|c| c.commit_id == raw) {
        return Some(raw.to_string());
    }
    if let Some(branch_ref) = reference.strip_prefix("branch:") {
        return resolve_branch(branches, branch_ref).and_then(|b| b.head_commit_id.clone());
    }
    // Try bare name as branch head
    resolve_branch(branches, raw).and_then(|b| b.head_commit_id.clone())
}

/// Build the version tree for any reference (commit, branch, or branch head).
pub fn tree_for_ref(
    bundles: &[CommitBundle],
    commits: &[CommitRecord],
    branches: &BTreeMap<String, BranchRecord>,
    reference: &str,
) -> Result<VersionTree, String> {
    if let Some(commit_id) = resolve_commit_ref(commits, branches, reference) {
        return tree_at_commit(bundles, &commit_id);
    }
    let branch = resolve_branch(branches, reference)
        .ok_or_else(|| format!("reference '{reference}' could not be resolved"))?;
    tree_for_branch_id(branches, bundles, &branch.branch_id)
}

// ---------------------------------------------------------------------------
// Legacy artifact-based tree construction (no content, used by catalog)
// ---------------------------------------------------------------------------

/// Build the version tree for a specific branch from artifact snapshots.
pub fn tree_for_branch_legacy(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    branch_id: &str,
) -> VersionTree {
    let mut tree = BTreeMap::new();
    for commit in commits.iter().rev() {
        if commit.branch_id != branch_id {
            continue;
        }
        for art_id in &commit.artifacts_changed {
            tree.entry(art_id.clone()).or_insert_with(|| {
                artifacts.get(art_id).map(|a| VersionView {
                    artifact: a.clone(),
                    version_id: a.current_version_id.clone(),
                    content_hash: String::new(),
                    content: String::new(),
                    chunks: vec![],
                    symbols: vec![],
                })
            });
        }
    }
    VersionTree {
        artifacts: tree
            .into_values()
            .flatten()
            .map(|v| (v.artifact.artifact_id.clone(), v))
            .collect(),
    }
}

/// Build the version tree at a specific commit from artifact snapshots.
pub fn tree_at_commit_legacy(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    target_commit_id: &str,
) -> VersionTree {
    let mut tree = BTreeMap::new();
    for commit in commits.iter().rev() {
        if tree.len() == artifacts.len() {
            break;
        }
        if commit.commit_id == target_commit_id || {
            let mut found = false;
            let mut current_id = Some(commit.commit_id.as_str());
            while let Some(cid) = current_id {
                if cid == target_commit_id {
                    found = true;
                    break;
                }
                current_id = commits
                    .iter()
                    .find(|c| c.commit_id == cid)
                    .and_then(|c| c.parent_commit_id.as_deref());
            }
            found
        } {
            for art_id in &commit.artifacts_changed {
                tree.entry(art_id.clone()).or_insert_with(|| {
                    artifacts.get(art_id).map(|a| VersionView {
                        artifact: a.clone(),
                        version_id: a.current_version_id.clone(),
                        content_hash: String::new(),
                        content: String::new(),
                        chunks: vec![],
                        symbols: vec![],
                    })
                });
            }
        }
    }
    VersionTree {
        artifacts: tree
            .into_values()
            .flatten()
            .map(|v| (v.artifact.artifact_id.clone(), v))
            .collect(),
    }
}

/// Apply a commit bundle to the legacy tree (inserts artifact snapshot).
pub fn apply_bundle_to_tree_legacy(
    tree: &mut VersionTree,
    artifact: SourceArtifact,
    version_id: String,
    content_hash: String,
) {
    tree.artifacts.insert(
        artifact.artifact_id.clone(),
        VersionView {
            artifact,
            version_id,
            content_hash,
            content: String::new(),
            chunks: vec![],
            symbols: vec![],
        },
    );
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// Compare two version trees and produce a diff report.
///
/// This is the **content-aware** version that fills hunks with actual line diffs.
pub fn compare_trees(
    from_ref: &str,
    to_ref: &str,
    from: &VersionTree,
    to: &VersionTree,
) -> CompareReport {
    let mut keys: BTreeSet<String> = from.artifacts.keys().cloned().collect();
    keys.extend(to.artifacts.keys().cloned());

    let mut changed_files = Vec::new();
    let finding_count = 0;
    let blocking_finding_count = 0;

    for key in keys {
        let old = from.artifacts.get(&key);
        let new = to.artifacts.get(&key);
        let old_ver = old.map(|v| v.version_id.clone());
        let new_ver = new.map(|v| v.version_id.clone());
        if old_ver == new_ver {
            continue;
        }

        let old_content = old.as_ref().map(|v| v.content.as_str());
        let new_content = new.as_ref().map(|v| v.content.as_str());

        changed_files.push(ChangedFile {
            artifact_id: key.clone(),
            logical_path: new
                .or(old)
                .map(|v| v.artifact.logical_path.clone())
                .unwrap_or_default(),
            change_kind: match (old, new) {
                (None, Some(_)) => "added",
                (Some(_), None) => "deleted",
                (Some(_), Some(_)) => "modified",
                (None, None) => "unchanged",
            }
            .to_string(),
            old_version_id: old.map(|v| v.version_id.clone()),
            new_version_id: new.map(|v| v.version_id.clone()),
            old_hash: old.map(|v| v.content_hash.clone()),
            new_hash: new.map(|v| v.content_hash.clone()),
            hunks: diff_hunks(old_content, new_content),
        });
    }

    changed_files.sort_by(|a, b| a.logical_path.cmp(&b.logical_path));

    CompareReport {
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        changed_files,
        finding_count,
        blocking_finding_count,
    }
}

/// Compare two version trees (simple version, no ref labels).
pub fn compare_trees_simple(from: &VersionTree, to: &VersionTree) -> CompareReport {
    compare_trees("", "", from, to)
}

// ---------------------------------------------------------------------------
// Diff helpers
// ---------------------------------------------------------------------------

/// Compute diff hunks between two file contents.
pub fn diff_hunks(old_content: Option<&str>, new_content: Option<&str>) -> Vec<LineHunk> {
    match (old_content, new_content) {
        (Some(old), Some(new)) if old == new => return vec![],
        _ => {}
    }

    let old_lines: Vec<&str> = old_content.unwrap_or("").lines().collect();
    let new_lines: Vec<&str> = new_content.unwrap_or("").lines().collect();

    if old_lines.is_empty() && new_lines.is_empty() {
        return vec![];
    }

    if old_lines.is_empty() {
        return vec![LineHunk {
            old_start: 0,
            old_end: 0,
            new_start: 1,
            new_end: new_lines.len() as u32,
            old_lines: vec![],
            new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
        }];
    }

    if new_lines.is_empty() {
        return vec![LineHunk {
            old_start: 1,
            old_end: old_lines.len() as u32,
            new_start: 0,
            new_end: 0,
            old_lines: old_lines.iter().map(|s| s.to_string()).collect(),
            new_lines: vec![],
        }];
    }

    // Simple full-file diff for now
    vec![LineHunk {
        old_start: 1,
        old_end: old_lines.len() as u32,
        new_start: 1,
        new_end: new_lines.len() as u32,
        old_lines: old_lines.iter().map(|s| s.to_string()).collect(),
        new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_artifact(id: &str, path: &str) -> SourceArtifact {
        SourceArtifact {
            artifact_id: id.to_string(),
            logical_path: path.to_string(),
            language: "Rust".to_string(),
            owner_capability: "backend".to_string(),
            risk_level: "low".to_string(),
            current_version_id: "ver_1".to_string(),
            policy_bindings: vec![],
            allowed_grants: vec![],
            required_tests: vec![],
            materialization_mode: "sync".to_string(),
            deleted: false,
        }
    }

    #[test]
    fn test_compare_identical_trees() {
        let mut tree = VersionTree {
            artifacts: BTreeMap::new(),
        };
        let art = make_artifact("art_1", "src/main.rs");
        let view = VersionView {
            artifact: art.clone(),
            version_id: "ver_1".to_string(),
            content_hash: "abc".to_string(),
            content: String::new(),
            chunks: vec![],
            symbols: vec![],
        };
        tree.artifacts.insert("art_1".to_string(), view.clone());

        let report = compare_trees_simple(&tree, &tree);
        assert!(report.changed_files.is_empty());
    }

    #[test]
    fn test_compare_added_artifact() {
        let from = VersionTree {
            artifacts: BTreeMap::new(),
        };
        let mut to = VersionTree {
            artifacts: BTreeMap::new(),
        };
        let art = make_artifact("art_1", "src/main.rs");
        to.artifacts.insert(
            "art_1".to_string(),
            VersionView {
                artifact: art,
                version_id: "ver_1".to_string(),
                content_hash: "abc".to_string(),
                content: String::new(),
                chunks: vec![],
                symbols: vec![],
            },
        );

        let report = compare_trees_simple(&from, &to);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].change_kind, "added");
    }

    #[test]
    fn test_diff_hunks_same_content() {
        let hunks = diff_hunks(Some("hello\nworld"), Some("hello\nworld"));
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_diff_hunks_different_content() {
        let hunks = diff_hunks(Some("old\ncontent"), Some("new\ncontent"));
        assert!(!hunks.is_empty());
    }

    #[test]
    fn test_resolve_branch_with_prefix() {
        let mut branches = BTreeMap::new();
        branches.insert(
            "br_1".to_string(),
            BranchRecord {
                branch_id: "br_1".to_string(),
                name: "main".to_string(),
                base_commit_id: None,
                head_commit_id: Some("cmt_1".to_string()),
                parent_branch_id: None,
                status: "active".to_string(),
                actor: "test".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        );
        let result = resolve_branch(&branches, "branch:br_1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "main");
    }
}
