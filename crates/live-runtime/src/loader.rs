use crate::LiveRuntime;
use runtime_bundle::{verify_bundle, BundlePackage, RuntimeStatus};
use std::collections::HashMap;

/// Errors that can occur during bundle loading/unloading.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("bundle not found: {0}")]
    BundleNotFound(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("bundle already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Load a verified bundle into the runtime.
/// 1. Verify the bundle (signature + artifact integrity)
/// 2. Prepare the execution environment
/// 3. Return a LiveRuntime instance
pub async fn load_bundle(
    package: BundlePackage,
    public_key_hex: &str,
    artifacts: Vec<(String, Vec<u8>)>,
    governance_context: HashMap<String, String>,
) -> Result<LiveRuntime, LoadError> {
    // Verify the bundle before loading
    let report = verify_bundle(&package, public_key_hex, &artifacts);

    if !report.signature_valid {
        return Err(LoadError::VerificationFailed(
            "bundle signature is invalid".to_string(),
        ));
    }

    if !report.all_artifact_hashes_match {
        return Err(LoadError::VerificationFailed(
            "bundle artifact hashes do not match manifest".to_string(),
        ));
    }

    // Extract artifact paths for tracking
    let loaded_artifacts: Vec<String> = artifacts.into_iter().map(|(path, _)| path).collect();

    let runtime_id = uuid::Uuid::new_v4().to_string();
    let manifest = package.manifest;

    let mut runtime = LiveRuntime::new(
        runtime_id,
        manifest.bundle_id.clone(),
        manifest,
        governance_context,
    );
    runtime.loaded_artifacts = loaded_artifacts;
    runtime.status = RuntimeStatus::Running;

    Ok(runtime)
}

/// Unload a bundle from the runtime, performing graceful shutdown.
pub async fn unload_bundle(runtime: &mut LiveRuntime) -> Result<(), LoadError> {
    runtime.status = RuntimeStatus::Stopped;
    runtime.loaded_artifacts.clear();
    tracing::info!(
        runtime_id = %runtime.runtime_id,
        bundle_id = %runtime.bundle_id,
        "runtime unloaded"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_bundle::{generate_key_pair, BundleBuilder};

    fn make_test_environment() -> (BundlePackage, String, Vec<(String, Vec<u8>)>) {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("1.0.0", "test-app", &kp.private_key_hex);
        builder.add_artifact("main.wasm", vec![1, 2, 3, 4]);
        builder.add_artifact("config.json", br#"{"key": "value"}"#.to_vec());
        let package = builder.build().unwrap();
        let artifacts = vec![
            ("main.wasm".to_string(), vec![1, 2, 3, 4]),
            ("config.json".to_string(), br#"{"key": "value"}"#.to_vec()),
        ];
        (package, kp.public_key_hex, artifacts)
    }

    #[tokio::test]
    async fn loads_verified_bundle() {
        let (package, pub_key, artifacts) = make_test_environment();
        let ctx = HashMap::new();
        let runtime = load_bundle(package, &pub_key, artifacts, ctx)
            .await
            .expect("load should succeed");

        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert_eq!(runtime.loaded_artifacts.len(), 2);
        assert!(runtime.started_at <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn rejects_invalid_signature() {
        let (package, _valid_key, artifacts) = make_test_environment();
        let wrong_key = generate_key_pair().public_key_hex;
        let ctx = HashMap::new();
        let result = load_bundle(package, &wrong_key, artifacts, ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LoadError::VerificationFailed(_)
        ));
    }

    #[tokio::test]
    async fn unload_cleans_up() {
        let (package, pub_key, artifacts) = make_test_environment();
        let ctx = HashMap::new();
        let mut runtime = load_bundle(package, &pub_key, artifacts, ctx)
            .await
            .unwrap();

        assert_eq!(runtime.status, RuntimeStatus::Running);
        assert_eq!(runtime.loaded_artifacts.len(), 2);

        unload_bundle(&mut runtime).await.unwrap();
        assert_eq!(runtime.status, RuntimeStatus::Stopped);
        assert!(runtime.loaded_artifacts.is_empty());
    }
}
