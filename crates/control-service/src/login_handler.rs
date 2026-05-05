use authority_domain::login::{LoginAttemptOutcome, LoginCredentials, LoginPolicyConfig};
use authority_domain::login_policy;
use authority_domain::{ActorId, TenantId};
use chrono::{Duration, Utc};
use control_store::CoreStore;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};

use crate::error::ServiceError;
use crate::{AuditService, SessionService};

/// Response returned on successful login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_token: String,
    pub actor_id: ActorId,
    pub scope: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub audit_event_id: String,
}

/// Login handler — orchestrates the full login lifecycle:
///
/// 1. Load policy config (or default)
/// 2. Rate limit check (SR-3)
/// 3. Block check (SR-3)
/// 4. Credential validation (SR-2)
/// 5. Session issuance
/// 6. Audit trail (SR-5)
/// 7. Login attempt recording
///
/// Layer: service → policy → store
#[derive(Clone)]
pub struct LoginHandler {
    store: CoreStore,
    sessions: SessionService,
    audit_service: AuditService,
}

impl LoginHandler {
    pub fn new(store: CoreStore, sessions: SessionService, audit_service: AuditService) -> Self {
        Self {
            store,
            sessions,
            audit_service,
        }
    }

    /// Handle a login attempt.
    ///
    /// Returns `LoginResponse` on success, or a `ServiceError` describing the reason for denial.
    #[instrument(skip(self, credentials), fields(actor_id = %credentials.actor_id))]
    pub async fn handle_login(
        &self,
        tenant_id: TenantId,
        project_id: authority_domain::ProjectId,
        credentials: LoginCredentials,
    ) -> Result<LoginResponse, ServiceError> {
        let actor_id = credentials.actor_id.clone();
        let scope = credentials.scope.clone();

        // Step 1: Load policy config (or use default)
        let config = self
            .store
            .get_login_policy(&tenant_id)
            .await?
            .unwrap_or_default();

        // Step 2: Check for active block
        let block_record = self.store.get_login_block(&actor_id, &tenant_id).await?;

        // Step 3: Load recent attempts for rate limit evaluation
        let window_start = Utc::now() - Duration::seconds(config.window_seconds as i64);
        let recent_attempts = self
            .store
            .get_login_attempts(&actor_id, &tenant_id, window_start)
            .await?;

        // Step 4: Policy evaluation — rate limit check BEFORE credential validation (SR-3.1)
        let policy_outcome = login_policy::evaluate_login_attempt(
            &actor_id,
            &recent_attempts,
            block_record.as_ref(),
            &config,
        );

        match &policy_outcome {
            LoginAttemptOutcome::Blocked { .. } => {
                warn!(actor_id = %actor_id, "login blocked");
                self.record_attempt(&tenant_id, &actor_id, "blocked")
                    .await?;
                return Err(ServiceError::LoginBlocked(
                    "actor is blocked until further notice".to_string(),
                ));
            }
            LoginAttemptOutcome::RateLimited {
                retry_after_secs, ..
            } => {
                let retry = *retry_after_secs;
                warn!(actor_id = %actor_id, retry_after = %retry, "rate limited");
                self.record_attempt(&tenant_id, &actor_id, "rate_limited")
                    .await?;

                // Auto-block if the rate limit is due to excessive failures
                if retry >= config.block_duration_seconds {
                    let block = login_policy::build_block_record(
                        &actor_id,
                        &config,
                        "exceeded max failed attempts",
                    );
                    self.store.insert_login_block(&tenant_id, &block).await?;
                    return Err(ServiceError::LoginBlocked(format!(
                        "actor blocked for {} seconds due to excessive failures",
                        config.block_duration_seconds
                    )));
                }

                return Err(ServiceError::RateLimited(retry));
            }
            _ => { /* pass through to credential validation */ }
        }

        // Step 5: Credential validation (SR-2)
        let stored_hash = self.store.get_credential(&actor_id, &tenant_id).await?;

        let is_valid = match stored_hash {
            Some(hash) => login_policy::secure_compare(&credentials.credential, &hash),
            None => false, // Unknown actor — return same error to prevent enumeration (SR-6.3)
        };

        let credential_outcome =
            login_policy::credential_validation_result(&actor_id, is_valid, scope.clone());

        match credential_outcome {
            LoginAttemptOutcome::Success {
                actor_id: a,
                scope: s,
            } => {
                // Step 6: Issue session (SR-4)
                let session = self
                    .sessions
                    .issue_session(&a, &tenant_id, &project_id, 3600)
                    .await?; // 1 hour TTL

                // Step 7: Record audit event (SR-5)
                let audit_event = self
                    .audit_service
                    .append_chained_event(
                        &tenant_id,
                        &project_id,
                        &a,
                        "login.success",
                        "actor",
                        a.as_str(),
                        serde_json::json!({
                            "scope": s.as_str(),
                            "session_id": session.id.as_str(),
                        }),
                    )
                    .await?;

                // Step 8: Record successful attempt
                self.record_attempt(&tenant_id, &a, "success").await?;

                info!(actor_id = %a, "login successful");
                Ok(LoginResponse {
                    session_token: session.id.as_str().to_string(),
                    actor_id: a,
                    scope: s.to_string(),
                    expires_at: session.expires_at,
                    audit_event_id: audit_event.id.as_str().to_string(),
                })
            }
            _ => {
                // Step 9: Record failed attempt
                self.record_attempt(&tenant_id, &actor_id, "invalid_credentials")
                    .await?;

                // Step 10: Check if auto-block needed
                let failed_count = recent_attempts
                    .iter()
                    .filter(|a| a.outcome == "invalid_credentials" || a.outcome == "failed")
                    .count()
                    + 1; // include this attempt

                if failed_count >= config.max_failed_attempts as usize {
                    let block = login_policy::build_block_record(
                        &actor_id,
                        &config,
                        "exceeded max failed attempts",
                    );
                    self.store.insert_login_block(&tenant_id, &block).await?;
                    warn!(
                        actor_id = %actor_id,
                        failed_count = failed_count,
                        "auto-blocked after exceeding max failed attempts"
                    );
                }

                warn!(actor_id = %actor_id, "invalid credentials");
                Err(ServiceError::InvalidCredentials)
            }
        }
    }

    /// Get the current login policy for a tenant.
    #[instrument(skip(self))]
    pub async fn get_policy(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Option<LoginPolicyConfig>, ServiceError> {
        Ok(self.store.get_login_policy(tenant_id).await?)
    }

    /// Set the login policy for a tenant.
    #[instrument(skip(self, config))]
    pub async fn set_policy(
        &self,
        tenant_id: &TenantId,
        config: &LoginPolicyConfig,
    ) -> Result<(), ServiceError> {
        self.store.insert_login_policy(tenant_id, config).await?;
        info!(tenant_id = %tenant_id, "login policy updated");
        Ok(())
    }

    /// List all active blocks for a tenant.
    #[instrument(skip(self))]
    pub async fn list_blocks(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Vec<authority_domain::login::LoginBlockRecord>, ServiceError> {
        Ok(self.store.list_active_blocks(tenant_id).await?)
    }

    // ── Internal helpers ──

    /// Record a login attempt outcome in the store.
    async fn record_attempt(
        &self,
        tenant_id: &TenantId,
        actor_id: &ActorId,
        outcome: &str,
    ) -> Result<(), ServiceError> {
        let record = authority_domain::login::LoginAttemptRecord {
            actor_id: actor_id.clone(),
            timestamp: Utc::now(),
            outcome: outcome.to_string(),
        };
        self.store.insert_login_attempt(tenant_id, &record).await?;
        Ok(())
    }
}
