use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::StoreError;

// ============================================================================
// APPLICATION TYPES
// ============================================================================

/// Row representation of an application type.
#[derive(Clone, Debug)]
pub struct ApplicationTypeRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub complexity: String,
    pub timeline_weeks: String,
    pub status: String,
    pub is_ai_generated: bool,
    pub created_by_actor_id: Option<String>,
    pub approved_by_actor_id: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub version: i32,
}

#[derive(sqlx::FromRow)]
struct ApplicationTypeRaw {
    id: String,
    name: String,
    description: String,
    complexity: String,
    timeline_weeks: String,
    status: String,
    is_ai_generated: bool,
    created_by_actor_id: Option<String>,
    approved_by_actor_id: Option<String>,
    approved_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    version: i32,
}

impl From<ApplicationTypeRaw> for ApplicationTypeRow {
    fn from(raw: ApplicationTypeRaw) -> Self {
        ApplicationTypeRow {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            complexity: raw.complexity,
            timeline_weeks: raw.timeline_weeks,
            status: raw.status,
            is_ai_generated: raw.is_ai_generated,
            created_by_actor_id: raw.created_by_actor_id,
            approved_by_actor_id: raw.approved_by_actor_id,
            approved_at: raw.approved_at,
            created_at: raw.created_at,
            version: raw.version,
        }
    }
}

/// List active application types (canonical forms).
#[instrument(skip(pool))]
pub async fn list_application_types(
    pool: &PgPool,
) -> Result<Vec<ApplicationTypeRow>, StoreError> {
    let rows = sqlx::query_as::<_, ApplicationTypeRaw>(
        "SELECT id, name, description, complexity, timeline_weeks, status, \
         is_ai_generated, created_by_actor_id, approved_by_actor_id, \
         approved_at, created_at, version \
         FROM application_types \
         WHERE status = 'active' \
         ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(ApplicationTypeRow::from).collect())
}

/// Get an application type by ID.
#[instrument(skip(pool), fields(app_type_id = %app_type_id))]
pub async fn get_application_type(
    pool: &PgPool,
    app_type_id: &str,
) -> Result<Option<ApplicationTypeRow>, StoreError> {
    let raw = sqlx::query_as::<_, ApplicationTypeRaw>(
        "SELECT id, name, description, complexity, timeline_weeks, status, \
         is_ai_generated, created_by_actor_id, approved_by_actor_id, \
         approved_at, created_at, version \
         FROM application_types \
         WHERE id = $1",
    )
    .bind(app_type_id)
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(ApplicationTypeRow::from))
}

// ============================================================================
// TEMPLATES
// ============================================================================

/// Row representation of an intake template.
#[derive(Clone, Debug)]
pub struct IntakeTemplateRow {
    pub id: Uuid,
    pub application_type_id: String,
    pub version: i32,
    pub decision_tree: JsonValue,
    pub feature_mappings: JsonValue,
    pub status: String,
    pub is_ai_generated: bool,
    pub created_by_actor_id: Option<String>,
    pub approved_by_actor_id: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct IntakeTemplateRaw {
    id: Uuid,
    application_type_id: String,
    version: i32,
    decision_tree: JsonValue,
    feature_mappings: JsonValue,
    status: String,
    is_ai_generated: bool,
    created_by_actor_id: Option<String>,
    approved_by_actor_id: Option<String>,
    approved_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<IntakeTemplateRaw> for IntakeTemplateRow {
    fn from(raw: IntakeTemplateRaw) -> Self {
        IntakeTemplateRow {
            id: raw.id,
            application_type_id: raw.application_type_id,
            version: raw.version,
            decision_tree: raw.decision_tree,
            feature_mappings: raw.feature_mappings,
            status: raw.status,
            is_ai_generated: raw.is_ai_generated,
            created_by_actor_id: raw.created_by_actor_id,
            approved_by_actor_id: raw.approved_by_actor_id,
            approved_at: raw.approved_at,
            created_at: raw.created_at,
        }
    }
}

/// Get a template by ID.
#[instrument(skip(pool), fields(template_id = %template_id))]
pub async fn get_template(
    pool: &PgPool,
    template_id: &Uuid,
) -> Result<Option<IntakeTemplateRow>, StoreError> {
    let raw = sqlx::query_as::<_, IntakeTemplateRaw>(
        "SELECT id, application_type_id, version, decision_tree, feature_mappings, \
         status, is_ai_generated, created_by_actor_id, approved_by_actor_id, \
         approved_at, created_at \
         FROM intake_templates \
         WHERE id = $1",
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(IntakeTemplateRow::from))
}

/// Get the active template for an application type (highest version with status='active').
#[instrument(skip(pool), fields(app_type_id = %app_type_id))]
pub async fn get_active_template_for_app_type(
    pool: &PgPool,
    app_type_id: &str,
) -> Result<Option<IntakeTemplateRow>, StoreError> {
    let raw = sqlx::query_as::<_, IntakeTemplateRaw>(
        "SELECT id, application_type_id, version, decision_tree, feature_mappings, \
         status, is_ai_generated, created_by_actor_id, approved_by_actor_id, \
         approved_at, created_at \
         FROM intake_templates \
         WHERE application_type_id = $1 AND status = 'active' \
         ORDER BY version DESC \
         LIMIT 1",
    )
    .bind(app_type_id)
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(IntakeTemplateRow::from))
}

/// Insert a new template.
#[instrument(skip(pool, row), fields(template_id = %row.id, app_type_id = %row.application_type_id))]
pub async fn insert_template(
    pool: &PgPool,
    row: &IntakeTemplateRow,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO intake_templates \
         (id, application_type_id, version, decision_tree, feature_mappings, \
          status, is_ai_generated, created_by_actor_id, approved_by_actor_id, \
          approved_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(row.id)
    .bind(&row.application_type_id)
    .bind(row.version)
    .bind(&row.decision_tree)
    .bind(&row.feature_mappings)
    .bind(&row.status)
    .bind(row.is_ai_generated)
    .bind(row.created_by_actor_id.as_deref())
    .bind(row.approved_by_actor_id.as_deref())
    .bind(row.approved_at)
    .bind(row.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all templates for an application type (all versions).
#[instrument(skip(pool), fields(app_type_id = %app_type_id))]
pub async fn list_templates_for_app_type(
    pool: &PgPool,
    app_type_id: &str,
) -> Result<Vec<IntakeTemplateRow>, StoreError> {
    let rows = sqlx::query_as::<_, IntakeTemplateRaw>(
        "SELECT id, application_type_id, version, decision_tree, feature_mappings, \
         status, is_ai_generated, created_by_actor_id, approved_by_actor_id, \
         approved_at, created_at \
         FROM intake_templates \
         WHERE application_type_id = $1 \
         ORDER BY version DESC",
    )
    .bind(app_type_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(IntakeTemplateRow::from).collect())
}

// ============================================================================
// SESSIONS
// ============================================================================

/// Row representation of an intake session.
#[derive(Clone, Debug)]
pub struct IntakeSessionRow {
    pub id: Uuid,
    pub template_id: Option<Uuid>,
    pub application_type_id: Option<String>,
    pub user_actor_id: String,
    pub session_type: String,
    pub responses: JsonValue,
    pub ai_questions: JsonValue,
    pub derived_modules: Option<JsonValue>,
    pub derived_personas: Option<JsonValue>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub project_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct IntakeSessionRaw {
    id: Uuid,
    template_id: Option<Uuid>,
    application_type_id: Option<String>,
    user_actor_id: String,
    session_type: String,
    responses: JsonValue,
    ai_questions: JsonValue,
    derived_modules: Option<JsonValue>,
    derived_personas: Option<JsonValue>,
    status: String,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    project_id: Option<String>,
}

impl From<IntakeSessionRaw> for IntakeSessionRow {
    fn from(raw: IntakeSessionRaw) -> Self {
        IntakeSessionRow {
            id: raw.id,
            template_id: raw.template_id,
            application_type_id: raw.application_type_id,
            user_actor_id: raw.user_actor_id,
            session_type: raw.session_type,
            responses: raw.responses,
            ai_questions: raw.ai_questions,
            derived_modules: raw.derived_modules,
            derived_personas: raw.derived_personas,
            status: raw.status,
            created_at: raw.created_at,
            completed_at: raw.completed_at,
            project_id: raw.project_id,
        }
    }
}

/// Create a new intake session.
#[instrument(skip(pool, row), fields(session_id = %row.id, user_actor_id = %row.user_actor_id))]
pub async fn create_session(pool: &PgPool, row: &IntakeSessionRow) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO intake_sessions \
         (id, template_id, application_type_id, user_actor_id, session_type, \
          responses, ai_questions, derived_modules, derived_personas, status, \
          created_at, completed_at, project_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(row.id)
    .bind(row.template_id)
    .bind(row.application_type_id.as_deref())
    .bind(&row.user_actor_id)
    .bind(&row.session_type)
    .bind(&row.responses)
    .bind(&row.ai_questions)
    .bind(row.derived_modules.as_ref())
    .bind(row.derived_personas.as_ref())
    .bind(&row.status)
    .bind(row.created_at)
    .bind(row.completed_at)
    .bind(row.project_id.as_deref())
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a session by ID.
#[instrument(skip(pool), fields(session_id = %session_id))]
pub async fn get_session(
    pool: &PgPool,
    session_id: &Uuid,
) -> Result<Option<IntakeSessionRow>, StoreError> {
    let raw = sqlx::query_as::<_, IntakeSessionRaw>(
        "SELECT id, template_id, application_type_id, user_actor_id, session_type, \
         responses, ai_questions, derived_modules, derived_personas, status, \
         created_at, completed_at, project_id \
         FROM intake_sessions \
         WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(IntakeSessionRow::from))
}

/// Update session responses and derived data.
#[instrument(skip(pool, responses, ai_questions), fields(session_id = %session_id))]
pub async fn update_session(
    pool: &PgPool,
    session_id: &Uuid,
    responses: &JsonValue,
    ai_questions: &JsonValue,
    derived_modules: Option<&JsonValue>,
    derived_personas: Option<&JsonValue>,
) -> Result<(), StoreError> {
    sqlx::query(
        "UPDATE intake_sessions \
         SET responses = $2, ai_questions = $3, derived_modules = $4, derived_personas = $5 \
         WHERE id = $1",
    )
    .bind(session_id)
    .bind(responses)
    .bind(ai_questions)
    .bind(derived_modules)
    .bind(derived_personas)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark a session as completed.
#[instrument(skip(pool), fields(session_id = %session_id))]
pub async fn complete_session(
    pool: &PgPool,
    session_id: &Uuid,
    project_id: Option<&str>,
) -> Result<(), StoreError> {
    sqlx::query(
        "UPDATE intake_sessions \
         SET status = 'completed', completed_at = now(), project_id = $2 \
         WHERE id = $1",
    )
    .bind(session_id)
    .bind(project_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// List active sessions for a user.
#[instrument(skip(pool), fields(user_actor_id = %user_actor_id))]
pub async fn list_user_sessions(
    pool: &PgPool,
    user_actor_id: &str,
    limit: i64,
) -> Result<Vec<IntakeSessionRow>, StoreError> {
    let rows = sqlx::query_as::<_, IntakeSessionRaw>(
        "SELECT id, template_id, application_type_id, user_actor_id, session_type, \
         responses, ai_questions, derived_modules, derived_personas, status, \
         created_at, completed_at, project_id \
         FROM intake_sessions \
         WHERE user_actor_id = $1 \
         ORDER BY created_at DESC \
         LIMIT $2",
    )
    .bind(user_actor_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(IntakeSessionRow::from).collect())
}

// ============================================================================
// AI QUESTIONS (Learning System)
// ============================================================================

/// Row representation of an AI-asked question.
#[derive(Clone, Debug)]
pub struct AiIntakeQuestionRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub question_text: String,
    pub question_context: JsonValue,
    pub answer_text: Option<String>,
    pub application_type_id: Option<String>,
    pub asked_at: DateTime<Utc>,
    pub promoted_to_template: bool,
    pub promotion_date: Option<DateTime<Utc>>,
}

/// Log an AI-asked question.
#[instrument(skip(pool, question_context), fields(session_id = %session_id))]
pub async fn log_ai_question(
    pool: &PgPool,
    session_id: &Uuid,
    question_text: &str,
    question_context: &JsonValue,
    application_type_id: Option<&str>,
) -> Result<Uuid, StoreError> {
    let question_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO ai_intake_questions \
         (id, session_id, question_text, question_context, application_type_id, asked_at) \
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(question_id)
    .bind(session_id)
    .bind(question_text)
    .bind(question_context)
    .bind(application_type_id)
    .execute(pool)
    .await?;
    Ok(question_id)
}

/// Update the answer to an AI question.
#[instrument(skip(pool), fields(question_id = %question_id))]
pub async fn update_ai_question_answer(
    pool: &PgPool,
    question_id: &Uuid,
    answer_text: &str,
) -> Result<(), StoreError> {
    sqlx::query(
        "UPDATE ai_intake_questions SET answer_text = $2 WHERE id = $1",
    )
    .bind(question_id)
    .bind(answer_text)
    .execute(pool)
    .await?;
    Ok(())
}
