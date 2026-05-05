use authority_domain::login::{LoginAttemptRecord, LoginBlockRecord, LoginPolicyConfig};
use authority_domain::{ActorId, TenantId};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;

use crate::StoreError;

/// Insert a login attempt record.
#[instrument(skip(pool), fields(actor_id = %record.actor_id))]
pub async fn insert_login_attempt(
    pool: &PgPool,
    tenant_id: &TenantId,
    record: &LoginAttemptRecord,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO login_attempts (actor_id, tenant_id, timestamp, outcome) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(record.actor_id.as_str())
    .bind(tenant_id.as_str())
    .bind(record.timestamp)
    .bind(&record.outcome)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get login attempts for an actor within a time window.
#[instrument(skip(pool), fields(actor_id = %actor_id))]
pub async fn get_login_attempts(
    pool: &PgPool,
    actor_id: &ActorId,
    tenant_id: &TenantId,
    since: DateTime<Utc>,
) -> Result<Vec<LoginAttemptRecord>, StoreError> {
    let rows = sqlx::query_as::<_, LoginAttemptRaw>(
        "SELECT actor_id, timestamp, outcome FROM login_attempts \
         WHERE actor_id = $1 AND tenant_id = $2 AND timestamp >= $3 \
         ORDER BY timestamp DESC",
    )
    .bind(actor_id.as_str())
    .bind(tenant_id.as_str())
    .bind(since)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(LoginAttemptRecord {
                actor_id: ActorId::new(r.actor_id)
                    .map_err(|e| StoreError::InvalidData(e.to_string()))?,
                timestamp: r.timestamp,
                outcome: r.outcome,
            })
        })
        .collect()
}

/// Insert a block record for an actor.
#[instrument(skip(pool), fields(actor_id = %record.actor_id))]
pub async fn insert_login_block(
    pool: &PgPool,
    tenant_id: &TenantId,
    record: &LoginBlockRecord,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO login_blocks (actor_id, tenant_id, blocked_at, blocked_until, reason) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (actor_id, tenant_id) DO UPDATE \
         SET blocked_at = EXCLUDED.blocked_at, \
             blocked_until = EXCLUDED.blocked_until, \
             reason = EXCLUDED.reason",
    )
    .bind(record.actor_id.as_str())
    .bind(tenant_id.as_str())
    .bind(record.blocked_at)
    .bind(record.blocked_until)
    .bind(&record.reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get the current block record for an actor, if one exists and is still active.
#[instrument(skip(pool), fields(actor_id = %actor_id))]
pub async fn get_login_block(
    pool: &PgPool,
    actor_id: &ActorId,
    tenant_id: &TenantId,
) -> Result<Option<LoginBlockRecord>, StoreError> {
    let row = sqlx::query_as::<_, LoginBlockRaw>(
        "SELECT actor_id, tenant_id, blocked_at, blocked_until, reason FROM login_blocks \
         WHERE actor_id = $1 AND tenant_id = $2 AND blocked_until > NOW()",
    )
    .bind(actor_id.as_str())
    .bind(tenant_id.as_str())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(LoginBlockRecord {
            actor_id: ActorId::new(r.actor_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            blocked_at: r.blocked_at,
            blocked_until: r.blocked_until,
            reason: r.reason,
        })),
        None => Ok(None),
    }
}

/// List all active blocks (for operator-cli).
#[instrument(skip(pool))]
pub async fn list_active_blocks(
    pool: &PgPool,
    tenant_id: &TenantId,
) -> Result<Vec<LoginBlockRecord>, StoreError> {
    let rows = sqlx::query_as::<_, LoginBlockRaw>(
        "SELECT actor_id, tenant_id, blocked_at, blocked_until, reason FROM login_blocks \
         WHERE tenant_id = $1 AND blocked_until > NOW() \
         ORDER BY blocked_until ASC",
    )
    .bind(tenant_id.as_str())
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(LoginBlockRecord {
                actor_id: ActorId::new(r.actor_id)
                    .map_err(|e| StoreError::InvalidData(e.to_string()))?,
                blocked_at: r.blocked_at,
                blocked_until: r.blocked_until,
                reason: r.reason,
            })
        })
        .collect()
}

/// Insert or update login policy configuration.
#[instrument(skip(pool))]
pub async fn insert_login_policy(
    pool: &PgPool,
    tenant_id: &TenantId,
    config: &LoginPolicyConfig,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO login_policies (tenant_id, config_json) \
         VALUES ($1, $2) \
         ON CONFLICT (tenant_id) DO UPDATE \
         SET config_json = EXCLUDED.config_json",
    )
    .bind(tenant_id.as_str())
    .bind(serde_json::to_value(config)?)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get login policy configuration for a tenant.
#[instrument(skip(pool))]
pub async fn get_login_policy(
    pool: &PgPool,
    tenant_id: &TenantId,
) -> Result<Option<LoginPolicyConfig>, StoreError> {
    let row = sqlx::query_as::<_, LoginPolicyRaw>(
        "SELECT tenant_id, config_json FROM login_policies WHERE tenant_id = $1",
    )
    .bind(tenant_id.as_str())
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let config: LoginPolicyConfig = serde_json::from_value(r.config_json)?;
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

/// Store a credential hash for an actor (for Phase 9 POC).
#[instrument(skip(pool), fields(actor_id = %actor_id))]
pub async fn upsert_credential(
    pool: &PgPool,
    tenant_id: &TenantId,
    actor_id: &ActorId,
    credential_hash: &str,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO stored_credentials (actor_id, tenant_id, credential_hash) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (actor_id, tenant_id) DO UPDATE \
         SET credential_hash = EXCLUDED.credential_hash",
    )
    .bind(actor_id.as_str())
    .bind(tenant_id.as_str())
    .bind(credential_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get stored credential hash for an actor.
#[instrument(skip(pool), fields(actor_id = %actor_id))]
pub async fn get_credential(
    pool: &PgPool,
    actor_id: &ActorId,
    tenant_id: &TenantId,
) -> Result<Option<String>, StoreError> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT credential_hash FROM stored_credentials \
         WHERE actor_id = $1 AND tenant_id = $2",
    )
    .bind(actor_id.as_str())
    .bind(tenant_id.as_str())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ── Raw SQL row types ──

#[derive(sqlx::FromRow)]
struct LoginAttemptRaw {
    actor_id: String,
    timestamp: DateTime<Utc>,
    outcome: String,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct LoginBlockRaw {
    actor_id: String,
    tenant_id: String,
    blocked_at: DateTime<Utc>,
    blocked_until: DateTime<Utc>,
    reason: String,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct LoginPolicyRaw {
    tenant_id: String,
    config_json: serde_json::Value,
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a login attempt record.
    #[tracing::instrument(skip(self))]
    pub async fn insert_login_attempt(
        &self,
        tenant_id: &authority_domain::TenantId,
        record: &authority_domain::login::LoginAttemptRecord,
    ) -> Result<(), StoreError> {
        insert_login_attempt(&self.pool, tenant_id, record).await
    }

    /// Get login attempts for an actor within a time window.
    #[tracing::instrument(skip(self))]
    pub async fn get_login_attempts(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<authority_domain::login::LoginAttemptRecord>, StoreError> {
        get_login_attempts(&self.pool, actor_id, tenant_id, since).await
    }

    /// Insert a block record for an actor.
    #[tracing::instrument(skip(self))]
    pub async fn insert_login_block(
        &self,
        tenant_id: &authority_domain::TenantId,
        record: &authority_domain::login::LoginBlockRecord,
    ) -> Result<(), StoreError> {
        insert_login_block(&self.pool, tenant_id, record).await
    }

    /// Get the current block record for an actor.
    #[tracing::instrument(skip(self))]
    pub async fn get_login_block(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<authority_domain::login::LoginBlockRecord>, StoreError> {
        get_login_block(&self.pool, actor_id, tenant_id).await
    }

    /// List all active blocks (for operator-cli).
    #[tracing::instrument(skip(self))]
    pub async fn list_active_blocks(
        &self,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Vec<authority_domain::login::LoginBlockRecord>, StoreError> {
        list_active_blocks(&self.pool, tenant_id).await
    }

    /// Insert or update login policy configuration.
    #[tracing::instrument(skip(self))]
    pub async fn insert_login_policy(
        &self,
        tenant_id: &authority_domain::TenantId,
        config: &authority_domain::login::LoginPolicyConfig,
    ) -> Result<(), StoreError> {
        insert_login_policy(&self.pool, tenant_id, config).await
    }

    /// Get login policy configuration for a tenant.
    #[tracing::instrument(skip(self))]
    pub async fn get_login_policy(
        &self,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<authority_domain::login::LoginPolicyConfig>, StoreError> {
        get_login_policy(&self.pool, tenant_id).await
    }

    /// Store a credential hash for an actor.
    #[tracing::instrument(skip(self))]
    pub async fn upsert_credential(
        &self,
        tenant_id: &authority_domain::TenantId,
        actor_id: &ActorId,
        credential_hash: &str,
    ) -> Result<(), StoreError> {
        upsert_credential(&self.pool, tenant_id, actor_id, credential_hash).await
    }

    /// Get stored credential hash for an actor.
    #[tracing::instrument(skip(self))]
    pub async fn get_credential(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<String>, StoreError> {
        get_credential(&self.pool, actor_id, tenant_id).await
    }
}
