//! # rfsource-service
//!
//! Service layer for `.rfsource` operations.
//!
//! Provides the high-level API that bridges `.rfsource` file operations
//! with the Artifact Registry and query/materialization services.
//! This is the crate that other RealmForge crates (control-service,
//! control-api, agent-mcp, operator-cli) depend on.
//!
//! ## Crate Law
//!
//! - Orchestration layer only — delegates to rfsource-store, rfsource-catalog
//! - No direct file IO — goes through rfsource-store
//! - No direct database IO — goes through rfsource-catalog / control-store

use tracing::instrument;

use rfsource_catalog::ArtifactRegistry;
use rfsource_core::{
    CommitArtifactRequest, CommitOutcome, Manifest, ProjectStats, SearchHit,
};
use rfsource_materialize;
use rfsource_query::{ArtifactQuery, SearchResults};
use rfsource_store::RFSource;

/// The RFSource service — provides the full API surface for rfsource operations.
///
/// Wraps RFSource (store), ArtifactRegistry (catalog), and query/materialize
/// into a single cohesive service interface.
pub struct RFSourceService {
    store: RFSource,
    registry: ArtifactRegistry,
}

impl RFSourceService {
    /// Open an existing `.rfsource` project and load its Artifact Registry.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, ServiceError> {
        let store = RFSource::open(path.into())?;
        let registry = ArtifactRegistry::new();
        Ok(Self { store, registry })
    }

    /// Get the project manifest.
    pub fn manifest(&self) -> Result<Manifest, ServiceError> {
        Ok(self.store.manifest()?)
    }

    /// Get project statistics.
    pub fn stats(&self) -> Result<ProjectStats, ServiceError> {
        Ok(self.store.stats()?)
    }

    /// Commit an artifact.
    #[instrument(skip(self, req))]
    pub fn commit_artifact(
        &mut self,
        req: CommitArtifactRequest,
    ) -> Result<CommitOutcome, ServiceError> {
        let outcome = self.store.commit_artifact(req.clone())?;

        // Register in Artifact Registry
        self.registry
            .register_artifact(rfsource_catalog::ArtifactEntry {
                artifact_id: outcome.artifact.artifact_id.clone(),
                logical_path: req.logical_path,
                language: outcome.artifact.language.clone(),
                current_version_id: outcome.version.version_id.clone(),
                risk_level: outcome.artifact.risk_level.clone(),
                allowed_grants: outcome.artifact.allowed_grants.clone(),
                deleted: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .map_err(|e| ServiceError::Catalog(e.to_string()))?;

        self.registry
            .register_version(rfsource_catalog::ArtifactVersionEntry {
                version_id: outcome.version.version_id.clone(),
                artifact_id: outcome.artifact.artifact_id.clone(),
                commit_id: outcome.commit.commit_id.clone(),
                rfsource_path: Some(self.store.root().to_string_lossy().to_string()),
                content_hash: outcome.version.content_hash.clone(),
                line_count: outcome.version.line_count,
                created_at: chrono::Utc::now(),
            })
            .map_err(|e| ServiceError::Catalog(e.to_string()))?;

        Ok(outcome)
    }

    /// Search artifacts.
    pub fn search(&self, query: &ArtifactQuery, actor_grant: &str) -> Result<SearchResults, ServiceError> {
        let chunks = self.store.chunks()?;
        let symbols = self.store.symbols()?;
        Ok(rfsource_query::search_artifacts(
            query,
            &chunks,
            &symbols,
            &self.registry,
            actor_grant,
        ))
    }

    /// Materialize artifacts to the filesystem.
    pub fn materialize(
        &self,
        output_dir: impl AsRef<std::path::Path>,
        grant: Option<&str>,
    ) -> Result<usize, ServiceError> {
        Ok(rfsource_materialize::materialize(&self.store, output_dir, grant)?)
    }

    /// Get the Artifact Registry (for inspection/grant management).
    pub fn registry(&self) -> &ArtifactRegistry {
        &self.registry
    }

    /// Get the Artifact Registry mutably.
    pub fn registry_mut(&mut self) -> &mut ArtifactRegistry {
        &mut self.registry
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Store error: {0}")]
    Store(#[from] rfsource_store::StoreError),

    #[error("Core error: {0}")]
    Core(#[from] rfsource_core::RFSourceError),

    #[error("Materialize error: {0}")]
    Materialize(#[from] rfsource_materialize::MaterializeError),

    #[error("Catalog error: {0}")]
    Catalog(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_service_open() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test").unwrap();
        drop(store);

        let service = RFSourceService::open(&path).unwrap();
        let manifest = service.manifest().unwrap();
        assert_eq!(manifest.project_name, "test");
    }

    #[test]
    fn test_service_commit_and_search() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rfsource");
        let store = RFSource::create(&path, "test").unwrap();
        drop(store);

        let mut service = RFSourceService::open(&path).unwrap();
        service
            .commit_artifact(CommitArtifactRequest {
                logical_path: "src/main.rs".to_string(),
                language: "Rust".to_string(),
                content: "fn hello() {}".to_string(),
                owner_capability: "backend".to_string(),
                risk_level: "low".to_string(),
                policy_bindings: vec![],
                allowed_grants: vec![],
                required_tests: vec![],
                actor: "test".to_string(),
                message: "init".to_string(),
            })
            .unwrap();

        let results = service
            .search(
                &ArtifactQuery {
                    text_search: Some("hello".to_string()),
                    symbol_search: None,
                    limit: 10,
                    offset: 0,
                },
                "*",
            )
            .unwrap();
        assert!(results.total_count > 0);
    }
}
