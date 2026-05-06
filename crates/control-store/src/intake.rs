// ─────────────────────────────────────────────
// intake.rs — Intake Store (control-store)
// ─────────────────────────────────────────────
// PostgreSQL persistence for the intake decision
// tree system: app types, tree definitions,
// sessions, and observations.
// ─────────────────────────────────────────────

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// A registered application type in the intake system.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IntakeAppTypeRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub complexity: String,
    pub icon: String,
    pub requires_auth: bool,
    pub requires_db: bool,
    pub requires_hosting: bool,
    pub default_tree_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A stored decision tree definition.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IntakeTreeRow {
    pub id: String,
    pub tree_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub applies_to: Vec<String>,
    pub max_depth: i32,
    pub definition: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An intake session (in-progress or completed).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IntakeSessionRow {
    pub id: String,
    pub app_type_id: String,
    pub tree_id: String,
    pub status: String,
    pub answers: serde_json::Value,
    pub current_question_id: String,
    pub walk_result: Option<serde_json::Value>,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// An observation (AI learning data point).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IntakeObservationRow {
    pub id: String,
    pub question_text: String,
    pub context: String,
    pub app_type_id: Option<String>,
    pub frequency: i32,
    pub promotion_state: String,
    pub suggested_tree_id: String,
    pub admin_notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Intake store operations backed by PostgreSQL.
pub struct IntakeStore {
    pool: PgPool,
}

impl IntakeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── App Types ──

    pub async fn list_app_types(&self) -> Result<Vec<IntakeAppTypeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeAppTypeRow>(
            "SELECT * FROM intake_app_types WHERE is_active = true ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_app_type(&self, id: &str) -> Result<Option<IntakeAppTypeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeAppTypeRow>("SELECT * FROM intake_app_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    // ── Trees ──

    pub async fn list_trees(&self) -> Result<Vec<IntakeTreeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeTreeRow>(
            "SELECT * FROM intake_trees WHERE is_active = true ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_tree(&self, id: &str) -> Result<Option<IntakeTreeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeTreeRow>("SELECT * FROM intake_trees WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_tree_by_tree_id(
        &self,
        tree_id: &str,
    ) -> Result<Option<IntakeTreeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeTreeRow>(
            "SELECT * FROM intake_trees WHERE tree_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tree_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_trees_for_app_type(
        &self,
        app_type_id: &str,
    ) -> Result<Vec<IntakeTreeRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeTreeRow>(
            "SELECT * FROM intake_trees WHERE $1 = ANY(applies_to) AND is_active = true",
        )
        .bind(app_type_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn upsert_tree(&self, row: &IntakeTreeRow) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO intake_trees (id, tree_id, name, version, description, applies_to, max_depth, definition, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                description = EXCLUDED.description,
                applies_to = EXCLUDED.applies_to,
                max_depth = EXCLUDED.max_depth,
                definition = EXCLUDED.definition,
                is_active = EXCLUDED.is_active,
                updated_at = now()
            "#,
        )
        .bind(&row.id)
        .bind(&row.tree_id)
        .bind(&row.name)
        .bind(&row.version)
        .bind(&row.description)
        .bind(&row.applies_to)
        .bind(row.max_depth)
        .bind(&row.definition)
        .bind(row.is_active)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Sessions ──

    pub async fn create_session(&self, row: &IntakeSessionRow) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO intake_sessions (id, app_type_id, tree_id, status, answers, current_question_id, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&row.id)
        .bind(&row.app_type_id)
        .bind(&row.tree_id)
        .bind(&row.status)
        .bind(&row.answers)
        .bind(&row.current_question_id)
        .bind(&row.owner)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<IntakeSessionRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeSessionRow>("SELECT * FROM intake_sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_session_answers(
        &self,
        id: &str,
        answers: &serde_json::Value,
        current_question_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE intake_sessions
            SET answers = $2, current_question_id = $3, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(answers)
        .bind(current_question_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_session(
        &self,
        id: &str,
        walk_result: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE intake_sessions
            SET status = 'completed', walk_result = $2, completed_at = now(), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(walk_result)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_sessions_by_owner(
        &self,
        owner: &str,
    ) -> Result<Vec<IntakeSessionRow>, sqlx::Error> {
        sqlx::query_as::<_, IntakeSessionRow>(
            "SELECT * FROM intake_sessions WHERE owner = $1 ORDER BY created_at DESC",
        )
        .bind(owner)
        .fetch_all(&self.pool)
        .await
    }

    // ── Observations ──

    pub async fn upsert_observation(
        &self,
        question_text: &str,
        context: &str,
        app_type_id: Option<&str>,
    ) -> Result<String, sqlx::Error> {
        // Check if observation already exists (by question text + app type)
        let existing = sqlx::query_as::<_, IntakeObservationRow>(
            "SELECT * FROM intake_observations WHERE question_text = $1 AND app_type_id IS NOT DISTINCT FROM $2",
        )
        .bind(question_text)
        .bind(app_type_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = existing {
            // Increment frequency
            sqlx::query(
                "UPDATE intake_observations SET frequency = frequency + 1, updated_at = now() WHERE id = $1",
            )
            .bind(&row.id)
            .execute(&self.pool)
            .await?;
            Ok(row.id)
        } else {
            // Create new observation
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO intake_observations (id, question_text, context, app_type_id, frequency)
                VALUES ($1, $2, $3, $4, 1)
                "#,
            )
            .bind(&id)
            .bind(question_text)
            .bind(context)
            .bind(app_type_id)
            .execute(&self.pool)
            .await?;
            Ok(id)
        }
    }

    pub async fn list_observations(
        &self,
        promotion_state: Option<&str>,
    ) -> Result<Vec<IntakeObservationRow>, sqlx::Error> {
        match promotion_state {
            Some(state) => {
                sqlx::query_as::<_, IntakeObservationRow>(
                    "SELECT * FROM intake_observations WHERE promotion_state = $1 ORDER BY frequency DESC",
                )
                .bind(state)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, IntakeObservationRow>(
                    "SELECT * FROM intake_observations ORDER BY frequency DESC",
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }
}
