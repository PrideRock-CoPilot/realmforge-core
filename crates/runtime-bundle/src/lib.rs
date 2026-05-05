pub mod builder;
pub mod signing;
pub mod verifier;

pub use builder::BundleBuilder;
pub use signing::{
    compute_hash, generate_key_pair, sign_payload, verify_signature, KeyPair, SigningError,
};
pub use verifier::{verify_bundle, verify_deployment_readiness, VerificationReport};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A unique identifier for a runtime bundle.
/// For now this is a string-based ID; in production it would use the authority-domain BundleId.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BundleId(pub String);

impl BundleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BundleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Status of a bundle's lifecycle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleStatus {
    Building,
    Signed,
    Verified,
    Deployed,
    Running,
    Failed(String),
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleStatus::Building => write!(f, "building"),
            BundleStatus::Signed => write!(f, "signed"),
            BundleStatus::Verified => write!(f, "verified"),
            BundleStatus::Deployed => write!(f, "deployed"),
            BundleStatus::Running => write!(f, "running"),
            BundleStatus::Failed(msg) => write!(f, "failed: {}", msg),
        }
    }
}

/// The signed manifest for a runtime bundle.
/// Contains everything needed to verify and deploy a bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: BundleId,
    pub version: String,
    pub app_id: String,
    pub artifact_hashes: Vec<(String, String)>, // (artifact_path, sha256)
    pub governance_signature: String,
    pub release_approval_ref: Option<String>,
    pub built_at: DateTime<Utc>,
    pub status: BundleStatus,
}

/// A complete bundle package: manifest + artifact archive reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundlePackage {
    pub manifest: BundleManifest,
    pub archive_ref: String,
}

impl BundlePackage {
    pub fn new(manifest: BundleManifest, archive_ref: String) -> Self {
        Self {
            manifest,
            archive_ref,
        }
    }
}

/// Runtime status enum for the live runtime engine.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Loading,
    Running,
    Paused,
    Stopped,
    Failed(String),
}

impl std::fmt::Display for RuntimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeStatus::Loading => write!(f, "loading"),
            RuntimeStatus::Running => write!(f, "running"),
            RuntimeStatus::Paused => write!(f, "paused"),
            RuntimeStatus::Stopped => write!(f, "stopped"),
            RuntimeStatus::Failed(msg) => write!(f, "failed: {}", msg),
        }
    }
}
