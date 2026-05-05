use crate::signing::{compute_hash, sign_payload, SigningError};
use crate::{BundleId, BundleManifest, BundlePackage, BundleStatus};
use chrono::Utc;
use serde_json;

/// Errors that can occur during bundle building.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("no artifacts provided")]
    NoArtifacts,
    #[error("empty version string")]
    EmptyVersion,
    #[error("empty app id")]
    EmptyAppId,
    #[error("signing failed: {0}")]
    Signing(#[from] SigningError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Builds signed runtime bundles from a set of artifacts.
pub struct BundleBuilder {
    version: String,
    app_id: String,
    artifacts: Vec<(String, Vec<u8>)>, // (artifact_path, content)
    private_key_hex: String,
}

impl BundleBuilder {
    /// Create a new BundleBuilder.
    pub fn new(
        version: impl Into<String>,
        app_id: impl Into<String>,
        private_key_hex: impl Into<String>,
    ) -> Self {
        Self {
            version: version.into(),
            app_id: app_id.into(),
            artifacts: Vec::new(),
            private_key_hex: private_key_hex.into(),
        }
    }

    /// Add an artifact (file path + content bytes) to the bundle.
    pub fn add_artifact(&mut self, path: impl Into<String>, content: Vec<u8>) -> &mut Self {
        self.artifacts.push((path.into(), content));
        self
    }

    /// Build and sign the bundle package.
    pub fn build(&self) -> Result<BundlePackage, BuildError> {
        if self.artifacts.is_empty() {
            return Err(BuildError::NoArtifacts);
        }
        if self.version.is_empty() {
            return Err(BuildError::EmptyVersion);
        }
        if self.app_id.is_empty() {
            return Err(BuildError::EmptyAppId);
        }

        // Compute content hashes for all artifacts
        let mut artifact_hashes: Vec<(String, String)> = Vec::new();
        for (path, content) in &self.artifacts {
            let hash = compute_hash(content);
            artifact_hashes.push((path.clone(), hash));
        }

        let bundle_id = BundleId::new(uuid_v4());

        let manifest = BundleManifest {
            bundle_id: bundle_id.clone(),
            version: self.version.clone(),
            app_id: self.app_id.clone(),
            artifact_hashes: artifact_hashes.clone(),
            governance_signature: String::new(), // Placeholder — will be filled after signing
            release_approval_ref: None,
            built_at: Utc::now(),
            status: BundleStatus::Building,
        };

        // Serialize manifest deterministically for signing payload
        let manifest_json = serde_json::to_string(&manifest)?;
        let signature = sign_payload(&manifest_json, &self.private_key_hex)?;

        // Build the final signed manifest
        let signed_manifest = BundleManifest {
            governance_signature: signature,
            status: BundleStatus::Signed,
            ..manifest
        };

        // Store archive reference — in production this would write to an object store
        let archive_ref = format!("bundles/{}/archive", bundle_id);

        Ok(BundlePackage::new(signed_manifest, archive_ref))
    }
}

/// Generate a simple UUID v4 for bundle IDs without pulling in the uuid crate.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (ts >> 32) as u32,
        (ts >> 16) as u16,
        (ts & 0xfff) as u16,
        (ts >> 48) as u16 & 0x3fff | 0x8000,
        ts as u64 & 0xffff_ffff_ffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::generate_key_pair;

    #[test]
    fn builds_signed_bundle() {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("1.0.0", "test-app", kp.private_key_hex);
        builder.add_artifact("main.wasm", vec![1, 2, 3, 4]);
        builder.add_artifact("config.json", br#"{"key": "value"}"#.to_vec());

        let package = builder.build().expect("build should succeed");
        assert_eq!(package.manifest.version, "1.0.0");
        assert_eq!(package.manifest.app_id, "test-app");
        assert!(!package.manifest.governance_signature.is_empty());
        assert_eq!(package.manifest.artifact_hashes.len(), 2);
        assert!(package.manifest.built_at <= chrono::Utc::now());
    }

    #[test]
    fn reject_empty_artifacts() {
        let kp = generate_key_pair();
        let builder = BundleBuilder::new("1.0.0", "test-app", kp.private_key_hex);
        let result = builder.build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BuildError::NoArtifacts));
    }

    #[test]
    fn reject_empty_version() {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("", "test-app", kp.private_key_hex);
        builder.add_artifact("main.wasm", vec![1, 2, 3]);
        let result = builder.build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BuildError::EmptyVersion));
    }

    #[test]
    fn reject_empty_app_id() {
        let kp = generate_key_pair();
        let mut builder = BundleBuilder::new("1.0.0", "", kp.private_key_hex);
        builder.add_artifact("main.wasm", vec![1, 2, 3]);
        let result = builder.build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BuildError::EmptyAppId));
    }
}
