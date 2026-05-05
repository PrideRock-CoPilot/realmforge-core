use crate::{
    login::{LoginAttemptOutcome, LoginPolicyConfig},
    ActorId,
};
use chrono::{Duration, Utc};

/// Evaluate login credentials against the policy and context.
///
/// Pure function — no IO. Policy evaluation happens before any service orchestration.
///
/// Fatima's SR-3.1: Rate limit check happens BEFORE credential validation.
/// This function is called in two phases:
///   1. Rate limit check (cheap, no credential comparison)
///   2. If not rate-limited, credential validation (comparison performed by caller)
///
/// # Arguments
/// * `actor_id` - The actor attempting login
/// * `recent_attempts` - Recent login attempts within the policy window
/// * `block_record` - Current block record if actor is blocked
/// * `config` - The login policy configuration
///
/// Returns the outcome decision — the caller must act on it.
pub fn evaluate_login_attempt(
    actor_id: &ActorId,
    recent_attempts: &[crate::login::LoginAttemptRecord],
    block_record: Option<&crate::login::LoginBlockRecord>,
    config: &LoginPolicyConfig,
) -> LoginAttemptOutcome {
    let now = Utc::now();

    // Check 1: Is the actor currently blocked?
    if let Some(block) = block_record {
        if now < block.blocked_until {
            return LoginAttemptOutcome::Blocked {
                actor_id: actor_id.clone(),
                blocked_until: block.blocked_until,
            };
        }
    }

    // Check 2: Rate limit — count all attempts in the window.
    let window_start = now - Duration::seconds(config.window_seconds as i64);
    let attempts_in_window: Vec<_> = recent_attempts
        .iter()
        .filter(|a| a.timestamp >= window_start)
        .collect();

    if attempts_in_window.len() >= config.max_requests_per_window as usize {
        // Calculate retry-after as remaining time in the window
        let oldest_in_window = attempts_in_window
            .iter()
            .map(|a| a.timestamp)
            .min()
            .unwrap_or(now);
        let window_end = oldest_in_window + Duration::seconds(config.window_seconds as i64);
        let retry_after = (window_end - now).num_seconds().max(0) as u64;

        return LoginAttemptOutcome::RateLimited {
            actor_id: actor_id.clone(),
            retry_after_secs: retry_after.max(1),
        };
    }

    // Check 3: Failed attempt threshold for blocking.
    let failed_in_window = attempts_in_window
        .iter()
        .filter(|a| a.outcome == "invalid_credentials" || a.outcome == "failed")
        .count();

    if failed_in_window >= config.max_failed_attempts as usize {
        // Auto-block: too many failures
        return LoginAttemptOutcome::RateLimited {
            actor_id: actor_id.clone(),
            retry_after_secs: config.block_duration_seconds,
        };
    }

    // No policy rejection — caller proceeds with credential validation
    // The caller returns InvalidCredentials or Success based on the comparison result.
    LoginAttemptOutcome::InvalidCredentials {
        actor_id: Some(actor_id.clone()),
    }
}

/// Determine whether the credential comparison result should produce
/// a Success or InvalidCredentials outcome.
pub fn credential_validation_result(
    actor_id: &ActorId,
    is_valid: bool,
    scope: crate::login::Scope,
) -> LoginAttemptOutcome {
    if is_valid {
        LoginAttemptOutcome::Success {
            actor_id: actor_id.clone(),
            scope,
        }
    } else {
        LoginAttemptOutcome::InvalidCredentials {
            actor_id: Some(actor_id.clone()),
        }
    }
}

/// Build a block record from exceeding the threshold.
pub fn build_block_record(
    actor_id: &ActorId,
    config: &LoginPolicyConfig,
    reason: &str,
) -> crate::login::LoginBlockRecord {
    let now = Utc::now();
    crate::login::LoginBlockRecord {
        actor_id: actor_id.clone(),
        blocked_at: now,
        blocked_until: now + Duration::seconds(config.block_duration_seconds as i64),
        reason: reason.to_string(),
    }
}

/// Default stored credential for an actor.
/// In a real system, these would be hashed with a KDF (argon2/bcrypt).
/// For Phase 9, we store a simple hash for proof-of-concept.
#[derive(Debug, Clone)]
pub struct StoredCredential {
    pub actor_id: ActorId,
    /// Hex-encoded SHA-256 hash of the credential.
    pub credential_hash: String,
}

/// Constant-time comparison of a raw credential against a stored hash.
///
/// Uses `subtle::ConstantTimeEq` if available. Falls back to a simple
/// comparison with a note that production should use `subtle`.
///
/// Fatima's SR-2.1 and SR-2.2: Constant-time validation with secure comparison.
#[cfg(not(feature = "subtle"))]
pub fn secure_compare(credential: &[u8], stored_hash: &str) -> bool {
    use sha2::{Digest, Sha256};
    let computed_hash = hex::encode(Sha256::digest(credential));
    // Simple comparison — NOTE: in production, use subtle::ConstantTimeEq
    computed_hash == stored_hash
}

#[cfg(feature = "subtle")]
pub fn secure_compare(credential: &[u8], stored_hash: &str) -> bool {
    use sha2::{Digest, Sha256};
    use subtle::ConstantTimeEq;
    let computed_hash = hex::encode(Sha256::digest(credential));
    computed_hash
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::{LoginAttemptRecord, LoginBlockRecord, LoginPolicyConfig, Scope};
    use chrono::Utc;

    fn test_actor() -> ActorId {
        ActorId::generate()
    }

    fn default_config() -> LoginPolicyConfig {
        LoginPolicyConfig {
            max_failed_attempts: 5,
            window_seconds: 60,
            block_duration_seconds: 300,
            max_requests_per_window: 10,
            credential_validation_enabled: true,
        }
    }

    #[test]
    fn block_active_actor() {
        let actor = test_actor();
        let config = default_config();
        let now = Utc::now();
        let block = LoginBlockRecord {
            actor_id: actor.clone(),
            blocked_at: now - Duration::seconds(60),
            blocked_until: now + Duration::seconds(240), // still blocked
            reason: "too many failures".to_string(),
        };

        let outcome = evaluate_login_attempt(&actor, &[], Some(&block), &config);
        assert!(matches!(outcome, LoginAttemptOutcome::Blocked { .. }));
    }

    #[test]
    fn expired_block_allows_attempt() {
        let actor = test_actor();
        let config = default_config();
        let now = Utc::now();
        let block = LoginBlockRecord {
            actor_id: actor.clone(),
            blocked_at: now - Duration::seconds(600),
            blocked_until: now - Duration::seconds(300), // expired
            reason: "too many failures".to_string(),
        };

        let outcome = evaluate_login_attempt(&actor, &[], Some(&block), &config);
        // Should pass through to the default (not rate-limited, no policy violation)
        // which returns InvalidCredentials (needs credential validation)
        assert!(matches!(
            outcome,
            LoginAttemptOutcome::InvalidCredentials { .. }
        ));
    }

    #[test]
    fn rate_limit_exceeded() {
        let actor = test_actor();
        let mut config = default_config();
        config.max_requests_per_window = 3;

        let now = Utc::now();
        let attempts: Vec<LoginAttemptRecord> = (0..3)
            .map(|i| LoginAttemptRecord {
                actor_id: actor.clone(),
                timestamp: now - Duration::seconds(i * 10),
                outcome: "invalid_credentials".to_string(),
            })
            .collect();

        let outcome = evaluate_login_attempt(&actor, &attempts, None, &config);
        assert!(matches!(outcome, LoginAttemptOutcome::RateLimited { .. }));
    }

    #[test]
    fn within_rate_limit_passes() {
        let actor = test_actor();
        let mut config = default_config();
        config.max_requests_per_window = 5;

        let now = Utc::now();
        let attempts: Vec<LoginAttemptRecord> = (0..2)
            .map(|i| LoginAttemptRecord {
                actor_id: actor.clone(),
                timestamp: now - Duration::seconds(i * 20),
                outcome: "success".to_string(),
            })
            .collect();

        let outcome = evaluate_login_attempt(&actor, &attempts, None, &config);
        assert!(matches!(
            outcome,
            LoginAttemptOutcome::InvalidCredentials { .. }
        ));
    }

    #[test]
    fn failed_threshold_triggers_block() {
        let actor = test_actor();
        let mut config = default_config();
        config.max_failed_attempts = 3;
        config.max_requests_per_window = 20;

        let now = Utc::now();
        let attempts: Vec<LoginAttemptRecord> = (0..5)
            .map(|i| LoginAttemptRecord {
                actor_id: actor.clone(),
                timestamp: now - Duration::seconds(i * 5),
                outcome: "invalid_credentials".to_string(),
            })
            .collect();

        let outcome = evaluate_login_attempt(&actor, &attempts, None, &config);
        // 5 failures > 3 max, but all within window — should rate limit with block duration
        assert!(matches!(outcome, LoginAttemptOutcome::RateLimited { .. }));
    }

    #[test]
    fn credential_result_maps_correctly() {
        let actor = test_actor();
        let scope = Scope::new("read").unwrap();

        let success = credential_validation_result(&actor, true, scope.clone());
        assert!(matches!(success, LoginAttemptOutcome::Success { .. }));

        let failure = credential_validation_result(&actor, false, scope);
        assert!(matches!(
            failure,
            LoginAttemptOutcome::InvalidCredentials { .. }
        ));
    }
}
