use crate::signing::{compute_hash, verify_signature, SigningError};
use crate::{BundleManifest, BundlePackage, BundleStatus};

/// Errors that can occur during bundle verification.
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("signature verification failed: {0}")]
    Signature(#[from] SigningError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("artifact hash mismatch for {path}: expected {expected}, got {actual}")]
    ArtifactHashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("missing artifact: {0}")]
    MissingArtifact(String),
    #[error("deployment not ready: {0}")]
    DeploymentNotReady(String),
    #[error("invalid bundle status: {0}")]
    InvalidStatus(String),
}

/// The result of a bundle verification.
#[derive(Clone, Debug)]
pub struct VerificationReport {
    pub bundle_id: String,
    pub signature_valid: bool,
    pub all_artifact_hashes_match: bool,
    pub artifact_count: usize,
    pub verified_artifacts: usize,
    pub failed_artifacts: Vec<String>,
    pub deployment_ready: bool,
    pub errors: Vec<String>,
}

/// Verify a bundle package against its manifest.
/// Checks:
/// 1. The governance signature is valid against the serialized manifest
/// 2. All artifact content hashes match the manifest
/// 3. The bundle status is appropriate
pub fn verify_bundle(
    package: &BundlePackage,
    public_key_hex: &str,
    artifacts: &[(String, Vec<u8>)],
) -> VerificationReport {
    let manifest = &package.manifest;
    let bundle_id = manifest.bundle_id.to_string();
    let mut errors: Vec<String> = Vec::new();
    let mut failed_artifacts: Vec<String> = Vec::new();

    // 1. Verify governance signature
    let signature_valid = match verify_manifest_signature(manifest, public_key_hex) {
        Ok(true) => true,
        Ok(false) => {
            errors.push("governance signature does not match public key".to_string());
            false
        }
        Err(e) => {
            errors.push(format!("signature verification error: {}", e));
            false
        }
    };

    // 2. Verify artifact content hashes
    let mut all_hashes_match = true;
    let mut verified_count = 0;
    let manifest_hashes: std::collections::HashMap<&str, &str> = manifest
        .artifact_hashes
        .iter()
        .map(|(path, hash)| (path.as_str(), hash.as_str()))
        .collect();

    for (path, content) in artifacts {
        let actual_hash = compute_hash(content);
        match manifest_hashes.get(path.as_str()) {
            Some(expected_hash) if **expected_hash == actual_hash => {
                verified_count += 1;
            }
            Some(expected_hash) => {
                all_hashes_match = false;
                failed_artifacts.push(path.clone());
                errors.push(format!(
                    "artifact hash mismatch for {}: expected {}, got {}",
                    path, expected_hash, actual_hash
                ));
            }
            None => {
                errors.push(format!("artifact {} not found in manifest", path));
                failed_artifacts.push(path.clone());
            }
        }
    }

    // Check for missing artifacts (in manifest but not provided)
    for (path, _) in &manifest.artifact_hashes {
        let found = artifacts.iter().any(|(p, _)| p == path);
        if !found {
            all_hashes_match = false;
            failed_artifacts.push(path.clone());
            errors.push(format!("missing artifact: {}", path));
        }
    }

    VerificationReport {
        bundle_id,
        signature_valid,
        all_artifact_hashes_match: all_hashes_match,
        artifact_count: manifest.artifact_hashes.len(),
        verified_artifacts: verified_count,
        failed_artifacts,
        deployment_ready: false, // Evaluated by verify_deployment_readiness
        errors,
    }
}

/// Check whether a bundle is ready for deployment.
/// Verifies:
/// 1. Signature is valid
/// 2. All artifact hashes match
/// 3. Bundle status is appropriate (Verified or Signed)
pub fn verify_deployment_readiness(
    package: &BundlePackage,
    public_key_hex: &str,
    artifacts: &[(String, Vec<u8>)],
) -> Result<VerificationReport, VerificationError> {
    let report = verify_bundle(package, public_key_hex, artifacts);

    let mut deployment_ready = true;
    let mut deployment_errors: Vec<String> = Vec::new();

    if !report.signature_valid {
        deployment_ready = false;
        deployment_errors.push("signature is not valid".to_string());
    }

    if !report.all_artifact_hashes_match {
        deployment_ready = false;
        deployment_errors.push("artifact hashes do not match manifest".to_string());
    }

    if package.manifest.status != BundleStatus::Verified
        && package.manifest.status != BundleStatus::Signed
    {
        deployment_ready = false;
        deployment_errors.push(format!(
            "bundle status is {:?}, expected Verified or Signed",
            package.manifest.status
        ));
    }

    if deployment_errors.is_empty() {
        deployment_errors.push("ready for deployment".to_string());
    }

    Ok(VerificationReport {
        deployment_ready,
        errors: deployment_errors,
        ..report
    })
}

/// Verify the governance signature on a manifest.
///
/// Reconstructs the exact payload that was signed during `BundleBuilder::build()`:
/// - `governance_signature` is cleared (was empty string when signed)
/// - `status` is set to `Building` (was `Building` when signed, later updated to `Signed`)
/// - `bundle_id` is preserved as-is
/// - All other fields match the original signed manifest
fn verify_manifest_signature(
    manifest: &BundleManifest,
    public_key_hex: &str,
) -> Result<bool, VerificationError> {
    // Rebuild the manifest exactly as it was at signing time:
    // status was BundleStatus::Building and governance_signature was ""
    let verify_manifest = serde_json::to_string(&BundleManifest {
        governance_signature: String::new(),
        status: BundleStatus::Building,
        ..manifest.clone()
    })?;

    let result = verify_signature(
        &verify_manifest,
        &manifest.governance_signature,
        public_key_hex,
    )?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::BundleBuilder;
    use crate::signing::generate_key_pair;

    fn make_test_package() -> (BundlePackage, String, Vec<(String, Vec<u8>)>) {
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

    #[test]
    fn verify_valid_bundle_succeeds() {
        let (package, pub_key, artifacts) = make_test_package();
        let report = verify_bundle(&package, &pub_key, &artifacts);

        assert!(report.signature_valid);
        assert!(report.all_artifact_hashes_match);
        assert_eq!(report.artifact_count, 2);
        assert_eq!(report.verified_artifacts, 2);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn wrong_key_fails_verification_test() {
        let (package, _, artifacts) = make_test_package();
        let wrong_key = generate_key_pair().public_key_hex;
        let report = verify_bundle(&package, &wrong_key, &artifacts);

        assert!(!report.signature_valid);
        assert!(report.all_artifact_hashes_match); // Hashes still match
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn tampered_artifact_fails_verification() {
        let (package, pub_key, _) = make_test_package();
        let tampered_artifacts = vec![
            ("main.wasm".to_string(), vec![9, 9, 9, 9]), // Different content
            ("config.json".to_string(), br#"{"key": "value"}"#.to_vec()),
        ];
        let report = verify_bundle(&package, &pub_key, &tampered_artifacts);

        assert!(!report.all_artifact_hashes_match);
        assert_eq!(report.failed_artifacts.len(), 1);
    }

    #[test]
    fn missing_artifact_detected() {
        let (package, pub_key, _) = make_test_package();
        let partial_artifacts = vec![("main.wasm".to_string(), vec![1, 2, 3, 4])];
        let report = verify_bundle(&package, &pub_key, &partial_artifacts);

        assert!(!report.all_artifact_hashes_match);
        assert_eq!(report.failed_artifacts.len(), 1);
    }

    #[test]
    fn deployment_readiness_requires_verified_status() {
        let (package, pub_key, artifacts) = make_test_package();
        // Status is Signed — deployment readiness should check
        let result = verify_deployment_readiness(&package, &pub_key, &artifacts);
        assert!(result.is_ok());
        let report = result.unwrap();
        // Should be deployment ready since Signed is acceptable
        assert!(report.deployment_ready);
    }
}
