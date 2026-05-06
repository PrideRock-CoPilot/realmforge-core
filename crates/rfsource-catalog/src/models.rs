//! Artifact Registry data models.
//!
//! These types represent the metadata stored in PostgreSQL.
//! The actual artifact content lives in `.rfsource` files.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single artifact entry in the Artifact Registry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactEntry {
    /// Unique artifact identifier (matches `.rfsource` artifact_id).
    pub artifact_id: String,

    /// Logical path (e.g. "src/main.rs").
    pub logical_path: String,

    /// Detected language (e.g. "Rust", "TypeScript").
    pub language: String,

    /// Current (latest) version ID.
    pub current_version_id: String,

    /// Risk classification.
    pub risk_level: String,

    /// Allowed grant tokens for access control.
    pub allowed_grants: Vec<String>,

    /// Whether this artifact has been deleted/marked for removal.
    pub deleted: bool,

    /// When this artifact was first registered.
    pub created_at: DateTime<Utc>,

    /// When this artifact was last updated.
    pub updated_at: DateTime<Utc>,
}

/// A version entry in the registry pointing to a `.rfsource` version frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactVersionEntry {
    /// Version ID.
    pub version_id: String,

    /// Parent artifact ID.
    pub artifact_id: String,

    /// Commit ID that created this version.
    pub commit_id: String,

    /// Optional reference to the `.rfsource` file containing this version.
    pub rfsource_path: Option<String>,

    /// Content hash (SHA-256) for integrity verification.
    pub content_hash: String,

    /// Line count of the artifact at this version.
    pub line_count: u32,

    /// When this version was created.
    pub created_at: DateTime<Utc>,
}

/// A grant binding — who/what can access a given artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantBinding {
    /// Grant token (e.g. "SGL-BACKEND-READ", "SGL-RFSOURCE-*").
    pub grant: String,

    /// Artifact ID this grant applies to.
    pub artifact_id: String,

    /// When this binding was created.
    pub created_at: DateTime<Utc>,
}

/// A tag/label on an artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactTag {
    pub artifact_id: String,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}
