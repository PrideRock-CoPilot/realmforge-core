use crate::ServiceError;
use authority_domain::{BundleId, BundleManifest, BundleStatus};
use control_store::CoreStore;
use tracing::instrument;

/// Service for bundle lifecycle operations.
#[derive(Clone)]
pub struct BundleService {
    store: CoreStore,
}

impl BundleService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Create a new bundle manifest entry in the store.
    #[instrument(skip(self))]
    pub async fn record_bundle(
        &self,
        manifest: &BundleManifest,
    ) -> Result<(), ServiceError> {
        self.store.insert_bundle_manifest(manifest).await?;
        Ok(())
    }

    /// Get a bundle manifest by ID.
    #[instrument(skip(self))]
    pub async fn get_bundle(
        &self,
        bundle_id: &BundleId,
    ) -> Result<BundleManifest, ServiceError> {
        self.store
            .get_bundle(bundle_id)
            .await?
            .ok_or_else(|| ServiceError::Internal(format!("bundle not found: {}", bundle_id)))
    }

    /// List bundles for a given app.
    #[instrument(skip(self))]
    pub async fn list_bundles(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BundleManifest>, ServiceError> {
        Ok(self.store.list_bundles(app_id, limit, offset).await?)
    }

    /// Update a bundle's status.
    #[instrument(skip(self))]
    pub async fn update_bundle_status(
        &self,
        bundle_id: &BundleId,
        status: &BundleStatus,
    ) -> Result<(), ServiceError> {
        self.store.update_bundle_status(bundle_id, status).await?;
        Ok(())
    }
}
