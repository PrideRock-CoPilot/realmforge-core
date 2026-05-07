//! Artifact Registry — in-memory and PostgreSQL-backed operations.
//!
//! This module provides the `ArtifactRegistry` interface for managing
//! artifact metadata. The initial implementation is in-memory;
//! PostgreSQL persistence will be added when the schema is finalized.

use std::collections::HashMap;

use tracing::instrument;

use crate::error::{CatalogError, Result};
use crate::models::{ArtifactEntry, ArtifactTag, ArtifactVersionEntry, GrantBinding};

/// The Artifact Registry manages artifact metadata.
///
/// In this initial implementation, the registry is in-memory.
/// When the schema is finalized, a PostgreSQL-backed implementation
/// will be added in `control-store`.
#[derive(Clone, Debug)]
pub struct ArtifactRegistry {
    artifacts: HashMap<String, ArtifactEntry>,
    versions: HashMap<String, ArtifactVersionEntry>,
    grants: Vec<GrantBinding>,
    tags: Vec<ArtifactTag>,
}

impl ArtifactRegistry {
    /// Create a new empty Artifact Registry.
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            versions: HashMap::new(),
            grants: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Register a new artifact in the registry.
    #[instrument(skip(self))]
    pub fn register_artifact(&mut self, entry: ArtifactEntry) -> Result<()> {
        if self.artifacts.contains_key(&entry.artifact_id) {
            return Err(CatalogError::Validation(format!(
                "Artifact already registered: {}",
                entry.artifact_id
            )));
        }
        self.artifacts.insert(entry.artifact_id.clone(), entry);
        Ok(())
    }

    /// Get an artifact by ID.
    pub fn get_artifact(&self, artifact_id: &str) -> Option<&ArtifactEntry> {
        self.artifacts.get(artifact_id)
    }

    /// List all artifacts, optionally filtered by non-deleted only.
    pub fn list_artifacts(&self, include_deleted: bool) -> Vec<&ArtifactEntry> {
        self.artifacts
            .values()
            .filter(|a| include_deleted || !a.deleted)
            .collect()
    }

    /// Update an artifact's metadata.
    #[instrument(skip(self))]
    pub fn update_artifact(&mut self, entry: ArtifactEntry) -> Result<()> {
        if !self.artifacts.contains_key(&entry.artifact_id) {
            return Err(CatalogError::ArtifactNotFound(entry.artifact_id));
        }
        self.artifacts.insert(entry.artifact_id.clone(), entry);
        Ok(())
    }

    /// Mark an artifact as deleted (soft delete).
    #[instrument(skip(self))]
    pub fn delete_artifact(&mut self, artifact_id: &str) -> Result<()> {
        let entry = self
            .artifacts
            .get_mut(artifact_id)
            .ok_or_else(|| CatalogError::ArtifactNotFound(artifact_id.to_string()))?;
        entry.deleted = true;
        Ok(())
    }

    /// Register a new version for an artifact.
    #[instrument(skip(self))]
    pub fn register_version(&mut self, entry: ArtifactVersionEntry) -> Result<()> {
        if self.versions.contains_key(&entry.version_id) {
            return Err(CatalogError::Validation(format!(
                "Version already registered: {}",
                entry.version_id
            )));
        }
        self.versions.insert(entry.version_id.clone(), entry);
        Ok(())
    }

    /// Get a version by ID.
    pub fn get_version(&self, version_id: &str) -> Option<&ArtifactVersionEntry> {
        self.versions.get(version_id)
    }

    /// List all versions for a given artifact.
    pub fn list_versions(&self, artifact_id: &str) -> Vec<&ArtifactVersionEntry> {
        self.versions
            .values()
            .filter(|v| v.artifact_id == artifact_id)
            .collect()
    }

    /// Add a grant binding.
    #[instrument(skip(self))]
    pub fn add_grant(&mut self, grant: GrantBinding) {
        self.grants.push(grant);
    }

    /// List all grants for a given artifact.
    pub fn list_grants(&self, artifact_id: &str) -> Vec<&GrantBinding> {
        self.grants
            .iter()
            .filter(|g| g.artifact_id == artifact_id)
            .collect()
    }

    /// Check if a grant allows access to an artifact.
    pub fn grant_allows(&self, artifact_id: &str, grant: &str) -> bool {
        self.grants
            .iter()
            .any(|g| g.artifact_id == artifact_id && (g.grant == "*" || g.grant == grant))
    }

    /// Tag an artifact.
    pub fn tag_artifact(&mut self, key: &str, value: &str, artifact_id: &str) {
        self.tags.push(ArtifactTag {
            artifact_id: artifact_id.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            created_at: chrono::Utc::now(),
        });
    }

    /// List tags for an artifact.
    pub fn list_tags(&self, artifact_id: &str) -> Vec<&ArtifactTag> {
        self.tags
            .iter()
            .filter(|t| t.artifact_id == artifact_id)
            .collect()
    }
}

impl Default for ArtifactRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str) -> ArtifactEntry {
        ArtifactEntry {
            artifact_id: id.to_string(),
            logical_path: "src/main.rs".to_string(),
            language: "Rust".to_string(),
            current_version_id: "ver_1".to_string(),
            risk_level: "low".to_string(),
            allowed_grants: vec!["SGL-RFSOURCE-READ".to_string()],
            deleted: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ArtifactRegistry::new();
        let entry = sample_entry("art_1");
        reg.register_artifact(entry.clone()).unwrap();
        let retrieved = reg.get_artifact("art_1").unwrap();
        assert_eq!(retrieved.artifact_id, "art_1");
    }

    #[test]
    fn test_duplicate_register_fails() {
        let mut reg = ArtifactRegistry::new();
        reg.register_artifact(sample_entry("art_1")).unwrap();
        let result = reg.register_artifact(sample_entry("art_1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_soft_delete() {
        let mut reg = ArtifactRegistry::new();
        reg.register_artifact(sample_entry("art_1")).unwrap();
        assert_eq!(reg.list_artifacts(false).len(), 1);
        reg.delete_artifact("art_1").unwrap();
        assert_eq!(reg.list_artifacts(false).len(), 0);
        assert_eq!(reg.list_artifacts(true).len(), 1);
    }

    #[test]
    fn test_grant_check() {
        let mut reg = ArtifactRegistry::new();
        reg.register_artifact(sample_entry("art_1")).unwrap();
        reg.add_grant(GrantBinding {
            grant: "SGL-RFSOURCE-READ".to_string(),
            artifact_id: "art_1".to_string(),
            created_at: chrono::Utc::now(),
        });
        assert!(reg.grant_allows("art_1", "SGL-RFSOURCE-READ"));
        assert!(!reg.grant_allows("art_1", "SGL-RFSOURCE-WRITE"));
    }

    #[test]
    fn test_wildcard_grant() {
        let mut reg = ArtifactRegistry::new();
        reg.register_artifact(sample_entry("art_1")).unwrap();
        reg.add_grant(GrantBinding {
            grant: "*".to_string(),
            artifact_id: "art_1".to_string(),
            created_at: chrono::Utc::now(),
        });
        assert!(reg.grant_allows("art_1", "anything"));
    }

    #[test]
    fn test_register_version() {
        let mut reg = ArtifactRegistry::new();
        let entry = sample_entry("art_1");
        reg.register_artifact(entry).unwrap();
        reg.register_version(ArtifactVersionEntry {
            version_id: "ver_1".to_string(),
            artifact_id: "art_1".to_string(),
            commit_id: "cmt_1".to_string(),
            rfsource_path: None,
            content_hash: "abc123".to_string(),
            line_count: 100,
            created_at: chrono::Utc::now(),
        })
        .unwrap();
        assert_eq!(reg.list_versions("art_1").len(), 1);
    }
}
