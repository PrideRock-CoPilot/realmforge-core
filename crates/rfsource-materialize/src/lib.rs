//! # rfsource-materialize
//!
//! Materialize logical source artifacts from `.rfsource` to the filesystem.
//!
//! Reads the current state from a `.rfsource` file and writes the rendered
//! source files to the target directory. This is how `.rs`, `.ts`, `.py`,
//! `.md` and other source files are produced from the canonical `.rfsource`
//! storage.
//!
//! ## Crate Law
//!
//! - Filesystem projection only — no business logic
//! - Output files are "rendered projections", not truth
//! - Grant-scoped: only materialize artifacts the caller has access to

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use rfsource_core::SourceChunk;
use rfsource_store::RFSource;

/// Materialize all current artifacts to the given output directory.
///
/// Writes each artifact to the filesystem at its `logical_path` within `output_dir`.
/// Only artifacts matching the optional `grant` are materialized.
pub fn materialize(
    rfsource: &RFSource,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    let output_dir = output_dir.as_ref();

    let artifacts = rfsource
        .current_artifacts()
        .map_err(|e| MaterializeError::Store(e.to_string()))?;
    let all_chunks = rfsource
        .chunks()
        .map_err(|e| MaterializeError::Store(e.to_string()))?;

    let mut count = 0;

    // Group chunks by version_id
    let mut version_chunks: BTreeMap<String, Vec<SourceChunk>> = BTreeMap::new();
    for chunk in &all_chunks {
        version_chunks
            .entry(chunk.version_id.clone())
            .or_default()
            .push(chunk.clone());
    }

    for (art_id, artifact) in &artifacts {
        // Grant check
        if let Some(g) = grant {
            if !artifact.allowed_grants.is_empty()
                && !artifact.allowed_grants.contains(&g.to_string())
                && !artifact.allowed_grants.iter().any(|g| g == "*")
            {
                continue;
            }
        }

        // Reconstruct content from chunks for this artifact's current version
        let chunks = all_chunks
            .iter()
            .filter(|c| c.artifact_id == *art_id && c.version_id == artifact.current_version_id)
            .collect::<Vec<_>>();

        if chunks.is_empty() && !artifact.deleted {
            continue;
        }

        let file_path = output_dir.join(&artifact.logical_path);

        if artifact.deleted {
            // Skip deleted artifacts (or remove if they exist)
            let _ = fs::remove_file(&file_path);
            continue;
        }

        // Write the reconstructed file
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        let mut chunks_sorted = chunks.clone();
        chunks_sorted.sort_by_key(|c| c.ordinal);
        for chunk in chunks_sorted {
            content.push_str(&chunk.text);
            content.push('\n');
        }

        fs::write(&file_path, content.trim())?;
        count += 1;
    }

    Ok(count)
}

/// Materialize artifacts for a specific branch.
pub fn materialize_branch(
    rfsource: &RFSource,
    _branch: &str,
    output_dir: impl AsRef<Path>,
    grant: Option<&str>,
) -> Result<usize, MaterializeError> {
    // For now, materialize works the same regardless of branch
    // since the store's current_artifacts() reflects the main branch state.
    // Branch-specific materialization will be a future enhancement.
    materialize(rfsource, output_dir, grant)
}

#[derive(Debug, thiserror::Error)]
pub enum MaterializeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Store error: {0}")]
    Store(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rfsource_core::CommitArtifactRequest;
    use rfsource_store::RFSource;
    use tempfile::tempdir;

    #[test]
    fn test_materialize_single_file() {
        let dir = tempdir().unwrap();
        let rfsource_path = dir.path().join("test.rfsource");
        let store = RFSource::create(&rfsource_path, "test").unwrap();

        store
            .commit_artifact(
                CommitArtifactRequest {
                    logical_path: "src/main.rs".to_string(),
                    language: "Rust".to_string(),
                    content: "fn main() {}\n".to_string(),
                    owner_capability: "backend".to_string(),
                    risk_level: "low".to_string(),
                    policy_bindings: vec![],
                    allowed_grants: vec![],
                    required_tests: vec![],
                    actor: "test".to_string(),
                    message: "init".to_string(),
                },
                "*",
            )
            .unwrap();

        let out_dir = dir.path().join("out");
        let count = materialize(&store, &out_dir, None).unwrap();
        assert_eq!(count, 1);
        assert!(out_dir.join("src/main.rs").exists());
    }
}
