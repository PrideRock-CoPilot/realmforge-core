use authority_domain::{BundleManifest, BundleStatus};
use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::instrument;

use crate::error::McpError;

/// Args for core_create_bundle
#[derive(Clone, Debug, Deserialize)]
pub struct CreateBundleArgs {
    pub bundle_id: String,
    pub version: String,
    pub app_id: String,
    pub artifact_hashes: Vec<(String, String)>,
    pub governance_signature: String,
    #[serde(default)]
    pub release_approval_ref: Option<String>,
}

/// Args for core_verify_bundle
#[derive(Clone, Debug, Deserialize)]
pub struct VerifyBundleArgs {
    pub bundle_id: String,
    pub status: String,
}

/// Args for core_deploy_bundle
#[derive(Clone, Debug, Deserialize)]
pub struct DeployBundleArgs {
    pub bundle_id: String,
}

/// Create a new bundle manifest.
#[instrument(skip(ctx), fields(bundle_id = %args.bundle_id))]
pub async fn core_create_bundle(
    args: CreateBundleArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let manifest = BundleManifest {
        bundle_id: authority_domain::BundleId::new(args.bundle_id)
            .map_err(|e| McpError::InvalidArgs(format!("invalid bundle_id: {e}")))?,
        version: args.version,
        app_id: args.app_id,
        artifact_hashes: args.artifact_hashes,
        governance_signature: args.governance_signature,
        release_approval_ref: args
            .release_approval_ref
            .map(authority_domain::ApprovalId::new)
            .transpose()
            .map_err(|e| McpError::InvalidArgs(format!("invalid approval_ref: {e}")))?,
        built_at: chrono::Utc::now(),
        status: BundleStatus::Building,
    };

    ctx.bundles.record_bundle(&manifest).await?;
    Ok(json!(manifest))
}

/// Verify a bundle (update its verification status).
#[instrument(skip(ctx), fields(bundle_id = %args.bundle_id))]
pub async fn core_verify_bundle(
    args: VerifyBundleArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let bundle_id = authority_domain::BundleId::new(args.bundle_id)
        .map_err(|e| McpError::InvalidArgs(format!("invalid bundle_id: {e}")))?;
    let status = match args.status.as_str() {
        "verified" => BundleStatus::Verified,
        "signed" => BundleStatus::Signed,
        _ => BundleStatus::Failed(args.status),
    };

    ctx.bundles
        .update_bundle_status(&bundle_id, &status)
        .await?;
    Ok(json!({
        "bundle_id": bundle_id.to_string(),
        "status": format!("{:?}", status),
    }))
}

/// Deploy a bundle (mark as deployed).
#[instrument(skip(ctx), fields(bundle_id = %args.bundle_id))]
pub async fn core_deploy_bundle(
    args: DeployBundleArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let bundle_id = authority_domain::BundleId::new(args.bundle_id)
        .map_err(|e| McpError::InvalidArgs(format!("invalid bundle_id: {e}")))?;
    let manifest = ctx.bundles.get_bundle(&bundle_id).await?;
    ctx.bundles
        .update_bundle_status(&bundle_id, &BundleStatus::Deployed)
        .await?;
    Ok(json!({
        "bundle_id": bundle_id.to_string(),
        "status": "Deployed",
        "version": manifest.version,
        "app_id": manifest.app_id,
    }))
}
