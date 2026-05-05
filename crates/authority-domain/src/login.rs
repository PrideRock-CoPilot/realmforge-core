use crate::ActorId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Typed scope — validated against allowlist patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope(String);

impl Scope {
    /// Create a new Scope, validating against allowed patterns.
    pub fn new(value: impl Into<String>) -> Result<Self, LoginDomainError> {
        let s: String = value.into();
        if s.trim().is_empty() {
            return Err(LoginDomainError::EmptyScope);
        }
        if s.len() > 1024 {
            return Err(LoginDomainError::ScopeTooLong(s.len()));
        }
        // Basic allowlist: alphanumeric, dots, colons, hyphens, underscores, slashes
        if !s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == ':' || c == '-' || c == '_' || c == '/')
        {
            return Err(LoginDomainError::InvalidScope(s));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Login credentials — consumed immediately after validation.
///
/// `credential` uses `Box<[u8]>` for memory safety (zeroed on drop when wrapped).
/// The caller should use `zeroize::Zeroizing` or similar at the transport layer.
#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub actor_id: ActorId,
    /// Opaque credential bytes. Must be zeroed after validation.
    /// Use `zeroize::Zeroizing<Vec<u8>>` at the call site.
    pub credential: Vec<u8>,
    pub scope: Scope,
}

/// Login attempt outcome from policy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginAttemptOutcome {
    Success {
        actor_id: ActorId,
        scope: Scope,
    },
    RateLimited {
        actor_id: ActorId,
        retry_after_secs: u64,
    },
    Blocked {
        actor_id: ActorId,
        blocked_until: chrono::DateTime<chrono::Utc>,
    },
    InvalidCredentials {
        /// None when the actor_id itself is unknown (prevents account enumeration).
        actor_id: Option<ActorId>,
    },
}

/// Record of a login attempt for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttemptRecord {
    pub actor_id: ActorId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub outcome: String, // "success", "invalid_credentials", "rate_limited", "blocked"
}

/// Record of a blocked actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginBlockRecord {
    pub actor_id: ActorId,
    pub blocked_at: chrono::DateTime<chrono::Utc>,
    pub blocked_until: chrono::DateTime<chrono::Utc>,
    pub reason: String,
}

/// Login policy configuration stored per-tenant or globally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPolicyConfig {
    /// Max failed attempts within the window before rate limiting kicks in.
    pub max_failed_attempts: u32,
    /// Time window (seconds) for the rate limit and block count.
    pub window_seconds: u64,
    /// Duration (seconds) to block after exceeding max_failed_attempts.
    pub block_duration_seconds: u64,
    /// Rate limit: max requests per window before being rate-limited.
    pub max_requests_per_window: u32,
    /// Whether credential validation is enabled.
    pub credential_validation_enabled: bool,
}

impl Default for LoginPolicyConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            window_seconds: 60,
            block_duration_seconds: 300, // 5 minutes
            max_requests_per_window: 10,
            credential_validation_enabled: true,
        }
    }
}

/// Domain errors for login operations.
#[derive(Debug, Error)]
pub enum LoginDomainError {
    #[error("scope cannot be empty")]
    EmptyScope,
    #[error("scope too long: {0} characters (max 1024)")]
    ScopeTooLong(usize),
    #[error("invalid scope pattern: {0}")]
    InvalidScope(String),
    #[error("credential cannot be empty")]
    EmptyCredential,
    #[error("credential too long: {0} bytes (max 2048)")]
    CredentialTooLong(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_valid_patterns() {
        assert!(Scope::new("read").is_ok());
        assert!(Scope::new("read:write").is_ok());
        assert!(Scope::new("namespace/resource.action").is_ok());
        assert!(Scope::new("a.b/c:d_e-f").is_ok());
    }

    #[test]
    fn scope_rejects_empty() {
        assert!(matches!(
            Scope::new("").unwrap_err(),
            LoginDomainError::EmptyScope
        ));
    }

    #[test]
    fn scope_rejects_invalid_chars() {
        assert!(matches!(
            Scope::new("bad scope!").unwrap_err(),
            LoginDomainError::InvalidScope(_)
        ));
    }

    #[test]
    fn scope_rejects_too_long() {
        let long = "a".repeat(1025);
        assert!(matches!(
            Scope::new(long).unwrap_err(),
            LoginDomainError::ScopeTooLong(_)
        ));
    }

    #[test]
    fn policy_config_defaults() {
        let config = LoginPolicyConfig::default();
        assert_eq!(config.max_failed_attempts, 5);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.block_duration_seconds, 300);
        assert_eq!(config.max_requests_per_window, 10);
        assert!(config.credential_validation_enabled);
    }
}
