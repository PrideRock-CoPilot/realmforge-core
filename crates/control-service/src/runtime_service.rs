use crate::ServiceError;
use authority_domain::{RuntimeId, RuntimeInstance};
use control_store::CoreStore;
use tracing::instrument;

/// Service for live runtime lifecycle operations.
#[derive(Clone)]
pub struct RuntimeService {
    store: CoreStore,
}

impl RuntimeService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Deploy a bundle by creating a runtime instance.
    #[instrument(skip(self))]
    pub async fn deploy_bundle(
        &self,
        instance: &RuntimeInstance,
    ) -> Result<(), ServiceError> {
        self.store.insert_runtime_instance(instance).await?;
        Ok(())
    }

    /// Stop a runtime instance (update status to "stopped").
    #[instrument(skip(self))]
    pub async fn stop_runtime(
        &self,
        runtime_id: &RuntimeId,
    ) -> Result<(), ServiceError> {
        self.store.update_runtime_status(runtime_id, "stopped").await?;
        Ok(())
    }

    /// Get runtime health.
    #[instrument(skip(self))]
    pub async fn get_runtime_status(
        &self,
        runtime_id: &RuntimeId,
    ) -> Result<RuntimeInstance, ServiceError> {
        self.store
            .get_runtime_health(runtime_id)
            .await?
            .ok_or_else(|| ServiceError::Internal(format!("runtime not found: {}", runtime_id)))
    }

    /// List all runtime instances.
    #[instrument(skip(self))]
    pub async fn list_runtimes(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RuntimeInstance>, ServiceError> {
        Ok(self.store.list_runtime_instances(limit, offset).await?)
    }
}
