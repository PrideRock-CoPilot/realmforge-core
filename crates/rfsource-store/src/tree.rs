//! Version tree operations for `.rfsource`.
//!
//! The version tree is derived from the append-only commit chain.
//! It provides branch-specific views and comparison between states.

use std::collections::BTreeMap;

use rfsource_core::{
    BranchRecord, ChangedFile, CommitRecord, CompareReport, LineHunk, SourceArtifact,
};

/// A view of an artifact at a specific point in the version tree.
#[derive(Clone, Debug)]
pub struct VersionView {
    pub artifact: SourceArtifact,
    pub version_id: String,
    pub content_hash: String,
}

/// The version tree — a mapping from artifact_id to its state at a given point.
#[derive(Clone, Debug)]
pub struct VersionTree {
    pub artifacts: BTreeMap<String, VersionView>,
}

/// Derive the heads (latest commit) for each branch.
pub fn derive_branch_heads(
    branches: &BTreeMap<String, BranchRecord>,
) -> BTreeMap<String, String> {
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
    branches.values().find(|b| b.name == name)
}

/// Build the version tree for the main branch.
pub fn tree_for_branch(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    branch_id: &str,
) -> VersionTree {
    // Walk commits in reverse, collect the latest version per artifact
    let mut tree = BTreeMap::new();
    for commit in commits.iter().rev() {
        if commit.branch_id != branch_id {
            continue;
        }
        for art_id in &commit.artifacts_changed {
            if let Some(artifact) = artifacts.get(art_id) {
                tree.entry(art_id.clone()).or_insert(VersionView {
                    artifact: artifact.clone(),
                    version_id: artifact.current_version_id.clone(),
                    content_hash: String::new(),
                });
            }
        }
    }
    VersionTree { artifacts: tree }
}

/// Build the version tree for a specific branch by ID.
pub fn tree_for_branch_id(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    branch_id: &str,
) -> VersionTree {
    tree_for_branch(commits, artifacts, branch_id)
}

/// Build the version tree from parts (convenience wrapper).
pub fn tree_for_branch_id_from_parts(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    branch_id: &str,
) -> VersionTree {
    tree_for_branch(commits, artifacts, branch_id)
}

/// Build the version tree at a specific commit.
pub fn tree_at_commit(
    commits: &[CommitRecord],
    artifacts: &BTreeMap<String, SourceArtifact>,
    target_commit_id: &str,
) -> VersionTree {
    let mut tree = BTreeMap::new();
    for commit in commits.iter().rev() {
        if tree.len() == artifacts.len() {
            break; // All artifacts accounted for
        }
        if commit.commit_id == target_commit_id || {
            // Walk the chain up to target
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
                    })
                });
            }
        }
    }
    VersionTree {
        artifacts: tree.into_values().flatten().map(|v| (v.artifact.artifact_id.clone(), v)).collect(),
    }
}

/// Apply a commit bundle to the version tree (mutates the tree in place).
pub fn apply_bundle_to_tree(
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
        },
    );
}

/// Compare two version trees and produce a diff report.
pub fn compare_trees(from: &VersionTree, to: &VersionTree) -> CompareReport {
    let mut changed_files = Vec::new();
    let mut finding_count = 0;
    let mut blocking_finding_count = 0;

    // Find added/changed artifacts
    for (art_id, to_view) in &to.artifacts {
        match from.artifacts.get(art_id) {
            None => {
                // Added
                changed_files.push(ChangedFile {
                    artifact_id: art_id.clone(),
                    logical_path: to_view.artifact.logical_path.clone(),
                    change_kind: "added".to_string(),
                    old_version_id: None,
                    new_version_id: Some(to_view.version_id.clone()),
                    old_hash: None,
                    new_hash: Some(to_view.content_hash.clone()),
                    hunks: vec![],
                });
            }
            Some(from_view) if from_view.content_hash != to_view.content_hash => {
                // Changed
                changed_files.push(ChangedFile {
                    artifact_id: art_id.clone(),
                    logical_path: to_view.artifact.logical_path.clone(),
                    change_kind: "modified".to_string(),
                    old_version_id: Some(from_view.version_id.clone()),
                    new_version_id: Some(to_view.version_id.clone()),
                    old_hash: Some(from_view.content_hash.clone()),
                    new_hash: Some(to_view.content_hash.clone()),
                    hunks: vec![],
                });
            }
            _ => {} // Unchanged
        }
    }

    // Find removed artifacts
    for (art_id, from_view) in &from.artifacts {
        if !to.artifacts.contains_key(art_id) {
            changed_files.push(ChangedFile {
                artifact_id: art_id.clone(),
                logical_path: from_view.artifact.logical_path.clone(),
                change_kind: "removed".to_string(),
                old_version_id: Some(from_view.version_id.clone()),
                new_version_id: None,
                old_hash: Some(from_view.content_hash.clone()),
                new_hash: None,
                hunks: vec![],
            });
        }
    }

    changed_files.sort_by(|a, b| a.logical_path.cmp(&b.logical_path));

    CompareReport {
        from_ref: String::new(),
        to_ref: String::new(),
        changed_files,
        finding_count,
        blocking_finding_count,
    }
}

/// Compute diff hunks between two file contents (full-file diff).
pub fn diff_hunks(old_content: &str, new_content: &str) -> Vec<LineHunk> {
    if old_content == new_content {
        return vec![];
    }

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    if old_content.is_empty() {
        return vec![LineHunk {
            old_start: 0,
            old_end: 0,
            new_start: 1,
            new_end: new_lines.len() as u32,
            old_lines: vec![],
            new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
        }];
    }

    if new_content.is_empty() {
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
        };
        tree.artifacts.insert("art_1".to_string(), view.clone());

        let report = compare_trees(&tree, &tree);
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
            },
        );

        let report = compare_trees(&from, &to);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].change_kind, "added");
    }

    #[test]
    fn test_diff_hunks_same_content() {
        let hunks = diff_hunks("hello\nworld", "hello\nworld");
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_diff_hunks_different_content() {
        let hunks = diff_hunks("old\ncontent", "new\ncontent");
        assert!(!hunks.is_empty());
    }
}
