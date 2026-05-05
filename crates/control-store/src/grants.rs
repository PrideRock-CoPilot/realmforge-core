use authority_domain::{ActorId, GrantId, GrantState, SkillGrant};
use sqlx::PgPool;
use tracing::instrument;

use crate::StoreError;

/// Insert a new skill grant.
#[instrument(skip(pool), fields(grant_id = %grant.id, actor_id = %grant.actor_id))]
pub async fn insert_grant(pool: &PgPool, grant: &SkillGrant) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO skill_grants (id, actor_id, tenant_id, allowed_actions, denied_actions, \
         allowed_file_patterns, denied_file_patterns, budget_tokens, budget_operations, \
         separation_group, state, expires_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(grant.id.as_str())
    .bind(grant.actor_id.as_str())
    .bind(grant.tenant_id.as_str())
    .bind(&grant.allowed_actions)
    .bind(&grant.denied_actions)
    .bind(&grant.allowed_file_patterns)
    .bind(&grant.denied_file_patterns)
    .bind(grant.budget_tokens.map(|v| v as i64))
    .bind(grant.budget_operations.map(|v| v as i64))
    .bind(&grant.separation_group)
    .bind(grant.state.to_string())
    .bind(grant.expires_at)
    .bind(grant.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a grant by ID.
#[instrument(skip(pool), fields(grant_id = %grant_id))]
pub async fn get_grant(
    pool: &PgPool,
    grant_id: &GrantId,
) -> Result<Option<SkillGrant>, StoreError> {
    let row = sqlx::query_as::<_, GrantRaw>(
        "SELECT id, actor_id, tenant_id, allowed_actions, denied_actions, \
         allowed_file_patterns, denied_file_patterns, budget_tokens, budget_operations, \
         separation_group, state, expires_at, created_at \
         FROM skill_grants WHERE id = $1",
    )
    .bind(grant_id.as_str())
    .fetch_optional(pool)
    .await?;

    row.map(GrantRaw::try_into).transpose()
}

/// List all grants for an actor.
#[instrument(skip(pool), fields(actor_id = %actor_id))]
pub async fn list_grants_for_actor(
    pool: &PgPool,
    actor_id: &ActorId,
) -> Result<Vec<SkillGrant>, StoreError> {
    let rows = sqlx::query_as::<_, GrantRaw>(
        "SELECT id, actor_id, tenant_id, allowed_actions, denied_actions, \
         allowed_file_patterns, denied_file_patterns, budget_tokens, budget_operations, \
         separation_group, state, expires_at, created_at \
         FROM skill_grants WHERE actor_id = $1 \
         ORDER BY created_at DESC",
    )
    .bind(actor_id.as_str())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(GrantRaw::try_into).collect()
}

/// Revoke a grant (set state to Revoked).
#[instrument(skip(pool), fields(grant_id = %grant_id))]
pub async fn revoke_grant(pool: &PgPool, grant_id: &GrantId) -> Result<(), StoreError> {
    sqlx::query("UPDATE skill_grants SET state = 'revoked' WHERE id = $1")
        .bind(grant_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

/// Update grant state.
#[instrument(skip(pool), fields(grant_id = %grant_id, state = %state))]
pub async fn update_grant_state(
    pool: &PgPool,
    grant_id: &GrantId,
    state: &GrantState,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE skill_grants SET state = $1 WHERE id = $2")
        .bind(state.to_string())
        .bind(grant_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

/// Raw database row for a skill grant.
#[derive(sqlx::FromRow)]
struct GrantRaw {
    id: String,
    actor_id: String,
    tenant_id: String,
    allowed_actions: serde_json::Value,
    denied_actions: serde_json::Value,
    allowed_file_patterns: serde_json::Value,
    denied_file_patterns: serde_json::Value,
    budget_tokens: Option<i64>,
    budget_operations: Option<i64>,
    separation_group: String,
    state: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryInto<SkillGrant> for GrantRaw {
    type Error = StoreError;

    fn try_into(self) -> Result<SkillGrant, Self::Error> {
        let allowed_actions: Vec<String> = serde_json::from_value(self.allowed_actions)?;
        let denied_actions: Vec<String> = serde_json::from_value(self.denied_actions)?;
        let allowed_file_patterns: Vec<String> =
            serde_json::from_value(self.allowed_file_patterns)?;
        let denied_file_patterns: Vec<String> =
            serde_json::from_value(self.denied_file_patterns)?;

        let state = match self.state.as_str() {
            "active" => GrantState::Active,
            "expired" => GrantState::Expired,
            "revoked" => GrantState::Revoked,
            "suspended" => GrantState::Suspended,
            other => {
                return Err(StoreError::invalid_data(format!(
                    "unknown grant state: {}",
                    other
                )));
            }
        };

        Ok(SkillGrant {
            id: GrantId::new(self.id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            actor_id: ActorId::new(self.actor_id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            tenant_id: authority_domain::TenantId::new(self.tenant_id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            allowed_actions,
            denied_actions,
            allowed_file_patterns,
            denied_file_patterns,
            budget_tokens: self.budget_tokens.map(|v| v as u64),
            budget_operations: self.budget_operations.map(|v| v as u64),
            separation_group: self.separation_group,
            state,
            expires_at: self.expires_at,
            created_at: self.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ActorId, GrantId, GrantState, TenantId};
    use chrono::{Duration, Utc};

    fn test_grant() -> SkillGrant {
        SkillGrant {
            id: GrantId::new("grant-1").unwrap(),
            actor_id: ActorId::new("alice").unwrap(),
            tenant_id: TenantId::new("tenant").unwrap(),
            allowed_actions: vec!["file.read".to_string()],
            denied_actions: vec![],
            allowed_file_patterns: vec!["/workspace".to_string()],
            denied_file_patterns: vec![],
            budget_tokens: Some(1000),
            budget_operations: Some(50),
            separation_group: "developer".to_string(),
            state: GrantState::Active,
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn grant_serde_roundtrip() {
        let grant = test_grant();
        let json = serde_json::to_string(&grant).unwrap();
        let restored: SkillGrant = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, grant.id);
        assert_eq!(restored.state, grant.state);
    }

    #[test]
    fn grant_state_parsing_from_string() {
        assert_eq!("active".to_string(), GrantState::Active.to_string());
        assert_eq!("revoked".to_string(), GrantState::Revoked.to_string());
    }
}
