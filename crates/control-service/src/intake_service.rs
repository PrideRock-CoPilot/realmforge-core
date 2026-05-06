use chrono::Utc;
use control_store::{
    intake::{
        ApplicationTypeRow, IntakeSessionRow, IntakeTemplateRow,
    },
    CoreStore,
};
use intake_engine::{
    Answer, IntakeSession, Question, Response, SessionStatus, Template, TemplateId,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::error::ServiceError;

// AI provider imports
use ai_provider::{
    AiProvider, FormGenerationRequest, FormGenerationResponse, HelpRequest, HelpResponse,
};

// ============================================================================
// DATA TRANSFER OBJECTS
// ============================================================================

/// Application type information for intake UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationTypeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub complexity: String,
    pub timeline_weeks: String,
}

impl From<ApplicationTypeRow> for ApplicationTypeInfo {
    fn from(row: ApplicationTypeRow) -> Self {
        ApplicationTypeInfo {
            id: row.id,
            name: row.name,
            description: row.description,
            complexity: row.complexity,
            timeline_weeks: row.timeline_weeks,
        }
    }
}

/// Session state with current question and progress.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Uuid,
    pub application_type_id: String,
    pub status: String,
    pub current_question: Option<QuestionView>,
    pub responses_count: usize,
    pub progress_percent: u8,
    pub derived_modules: Vec<String>,
    pub derived_personas: Vec<String>,
}

/// Question view for frontend display.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestionView {
    pub id: String,
    pub text: String,
    pub question_type: String,
    pub options: Option<Vec<String>>,
    pub help_text: Option<String>,
}

/// Answer submission from user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnswerSubmission {
    pub question_id: String,
    pub answer: JsonValue,
}

// ============================================================================
// SERVICE
// ============================================================================

/// Intake system orchestration service.
///
/// Coordinates between intake-engine (pure logic) and control-store (persistence)
/// to manage structured intake sessions.
#[derive(Clone)]
pub struct IntakeService {
    store: CoreStore,
    ai_provider: Option<Arc<dyn AiProvider>>,
}

impl IntakeService {
    pub fn new(store: CoreStore) -> Self {
        Self {
            store,
            ai_provider: None,
        }
    }

    /// Create service with AI provider support.
    pub fn with_ai_provider(store: CoreStore, ai_provider: Arc<dyn AiProvider>) -> Self {
        Self {
            store,
            ai_provider: Some(ai_provider),
        }
    }

    // ========================================================================
    // APPLICATION TYPES
    // ========================================================================

    /// List all active application types.
    #[instrument(skip(self))]
    pub async fn list_application_types(&self) -> Result<Vec<ApplicationTypeInfo>, ServiceError> {
        let rows = control_store::intake::list_application_types(self.store.pool()).await?;
        Ok(rows.into_iter().map(ApplicationTypeInfo::from).collect())
    }

    /// Get an application type by ID.
    #[instrument(skip(self), fields(app_type_id = %app_type_id))]
    pub async fn get_application_type(
        &self,
        app_type_id: &str,
    ) -> Result<Option<ApplicationTypeInfo>, ServiceError> {
        let row = control_store::intake::get_application_type(self.store.pool(), app_type_id)
            .await?;
        Ok(row.map(ApplicationTypeInfo::from))
    }

    // ========================================================================
    // SESSION LIFECYCLE
    // ========================================================================

    /// Start a new intake session.
    ///
    /// Loads the active template for the application type and creates
    /// a new session with the root question ready.
    #[instrument(skip(self), fields(app_type_id = %app_type_id, user_actor_id = %user_actor_id))]
    pub async fn start_intake(
        &self,
        app_type_id: &str,
        user_actor_id: &str,
    ) -> Result<SessionState, ServiceError> {
        // Load active template
        let template_row =
            control_store::intake::get_active_template_for_app_type(self.store.pool(), app_type_id)
                .await?
                .ok_or_else(|| {
                    ServiceError::NotFound(format!("No active template for app type {}", app_type_id))
                })?;

        // Parse template (convert from store to engine format)
        let template = self.parse_template(&template_row)?;

        // Validate template
        let validation = intake_engine::validate_template(&template);
        if !validation.is_valid {
            warn!(
                template_id = %template_row.id,
                errors = ?validation.errors,
                "Template validation failed"
            );
            return Err(ServiceError::InvalidData(format!(
                "Template validation failed: {:?}",
                validation.errors
            )));
        }

        // Create session
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let session_row = IntakeSessionRow {
            id: session_id,
            template_id: Some(template_row.id),
            application_type_id: Some(app_type_id.to_string()),
            user_actor_id: user_actor_id.to_string(),
            session_type: "standard".to_string(),
            responses: json!({}),
            ai_questions: json!([]),
            derived_modules: None,
            derived_personas: None,
            status: "in_progress".to_string(),
            created_at: now,
            completed_at: None,
            project_id: None,
        };

        control_store::intake::create_session(self.store.pool(), &session_row).await?;

        info!(
            session_id = %session_id,
            app_type_id = %app_type_id,
            user_actor_id = %user_actor_id,
            "Intake session started"
        );

        // Get first question
        self.get_session_state(session_id).await
    }

    /// Submit an answer and get the next question.
    #[instrument(skip(self, answer), fields(session_id = %session_id, question_id = %answer.question_id))]
    pub async fn answer_question(
        &self,
        session_id: Uuid,
        answer: AnswerSubmission,
    ) -> Result<SessionState, ServiceError> {
        // Load session
        let session_row = control_store::intake::get_session(self.store.pool(), &session_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Session {} not found", session_id)))?;

        if session_row.status != "in_progress" {
            return Err(ServiceError::InvalidData(format!(
                "Session {} is not in progress (status: {})",
                session_id, session_row.status
            )));
        }

        // Load template
        let template_id = session_row
            .template_id
            .ok_or_else(|| ServiceError::NotFound("Session has no template".to_string()))?;
        let template_row = control_store::intake::get_template(self.store.pool(), &template_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Template {} not found", template_id)))?;

        let template = self.parse_template(&template_row)?;

        // Parse existing responses
        let mut responses = self.parse_responses(&session_row.responses)?;

        // Add new response
        let new_response = Response {
            question_id: answer.question_id.clone(),
            answer: self.parse_answer(&answer.answer)?,
            timestamp: Utc::now(),
        };
        responses.push(new_response);

        // Derive modules and personas
        let (modules, personas) =
            intake_engine::derive_modules_and_personas(&template, &responses)?;

        // Update session in database
        let responses_json = self.responses_to_json(&responses)?;
        control_store::intake::update_session(
            self.store.pool(),
            &session_id,
            &responses_json,
            &session_row.ai_questions,
            Some(&json!(modules)),
            Some(&json!(personas)),
        )
        .await?;

        info!(
            session_id = %session_id,
            question_id = %answer.question_id,
            response_count = responses.len(),
            "Answer recorded"
        );

        // Get updated state
        self.get_session_state(session_id).await
    }

    /// Get the current state of a session.
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session_state(&self, session_id: Uuid) -> Result<SessionState, ServiceError> {
        // Load session
        let session_row = control_store::intake::get_session(self.store.pool(), &session_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Session {} not found", session_id)))?;

        // Load template
        let template_id = session_row
            .template_id
            .ok_or_else(|| ServiceError::NotFound("Session has no template".to_string()))?;
        let template_row = control_store::intake::get_template(self.store.pool(), &template_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Template {} not found", template_id)))?;

        let template = self.parse_template(&template_row)?;
        let responses = self.parse_responses(&session_row.responses)?;

        // Build engine session
        let engine_session = IntakeSession {
            id: session_row.id.into(),
            template_id: TemplateId::from(template_row.id),
            responses,
            status: match session_row.status.as_str() {
                "in_progress" => SessionStatus::Active,
                "completed" => SessionStatus::Completed,
                _ => SessionStatus::Abandoned,
            },
            created_at: session_row.created_at,
            updated_at: session_row.created_at,
        };

        // Get next question
        let next_question = intake_engine::get_next_question(&template, &engine_session)?;

        // Calculate progress
        let total_questions = template.questions.len();
        let answered_questions = engine_session.responses.len();
        let progress_percent =
            ((answered_questions as f32 / total_questions as f32) * 100.0).min(100.0) as u8;

        // Extract modules and personas
        let derived_modules: Vec<String> = session_row
            .derived_modules
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let derived_personas: Vec<String> = session_row
            .derived_personas
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(SessionState {
            session_id,
            application_type_id: session_row
                .application_type_id
                .unwrap_or_else(|| "unknown".to_string()),
            status: session_row.status,
            current_question: next_question.map(Self::question_to_view),
            responses_count: answered_questions,
            progress_percent,
            derived_modules,
            derived_personas,
        })
    }

    /// Complete an intake session and finalize the project configuration.
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn complete_intake(
        &self,
        session_id: Uuid,
        project_id: Option<&str>,
    ) -> Result<(), ServiceError> {
        control_store::intake::complete_session(self.store.pool(), &session_id, project_id)
            .await?;
        info!(session_id = %session_id, "Intake session completed");
        Ok(())
    }

    // ========================================================================
    // AI ASSISTANCE (Phase 0b)
    // ========================================================================

    /// AI-assisted help when user is stuck on a question.
    ///
    /// Generates a clarifying question based on session context and logs it
    /// to the ai_intake_questions table for frequency tracking.
    #[instrument(skip(self, user_message), fields(session_id = %session_id, stuck_question_id = %stuck_question_id))]
    pub async fn ai_assist(
        &self,
        session_id: Uuid,
        stuck_question_id: &str,
        user_message: &str,
    ) -> Result<HelpResponse, ServiceError> {
        let ai_provider = self
            .ai_provider
            .as_ref()
            .ok_or_else(|| ServiceError::InvalidData("AI provider not configured".to_string()))?;

        // Load session context
        let session_row = control_store::intake::get_session(self.store.pool(), &session_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Session {} not found", session_id)))?;

        let responses = self.parse_responses(&session_row.responses)?;
        let previous_answers: Vec<(String, JsonValue)> = responses
            .iter()
            .map(|r| (r.question_id.clone(), r.answer.as_json_value()))
            .collect();

        // Build AI request
        let help_request = HelpRequest {
            session_context: format!(
                "Application type: {}, {} answers provided",
                session_row
                    .application_type_id
                    .as_deref()
                    .unwrap_or("unknown"),
                responses.len()
            ),
            user_message: user_message.to_string(),
            previous_answers,
        };

        // Call AI provider
        let help_response = ai_provider.assist_user(&help_request).await.map_err(|e| {
            ServiceError::InternalError(format!("AI assistance failed: {}", e))
        })?;

        // Log AI question to database
        control_store::intake::log_ai_question(
            self.store.pool(),
            &session_id,
            &help_response.clarifying_question,
            &json!({
                "stuck_question_id": stuck_question_id,
                "user_message": user_message,
                "explanation": help_response.explanation,
                "guidance": help_response.guidance,
            }),
            session_row.application_type_id.as_deref(),
        )
        .await?;

        info!(
            session_id = %session_id,
            stuck_question_id = %stuck_question_id,
            "AI assistance provided"
        );

        Ok(help_response)
    }

    /// AI-generated intake form from natural language description.
    ///
    /// Creates a draft template that requires admin review before activation.
    /// The generated form is validated against the intake-engine rules.
    #[instrument(skip(self, app_description, hints), fields(user_actor_id = %user_actor_id))]
    pub async fn generate_form(
        &self,
        user_actor_id: &str,
        app_description: &str,
        hints: Vec<String>,
    ) -> Result<FormGenerationResponse, ServiceError> {
        let ai_provider = self
            .ai_provider
            .as_ref()
            .ok_or_else(|| ServiceError::InvalidData("AI provider not configured".to_string()))?;

        // Build generation request
        let form_request = FormGenerationRequest {
            app_description: app_description.to_string(),
            hints,
            target_question_count: Some(12),
        };

        // Call AI provider
        let form_response = ai_provider
            .generate_form(&form_request)
            .await
            .map_err(|e| ServiceError::InternalError(format!("Form generation failed: {}", e)))?;

        // Validate generated template
        let template: Template = serde_json::from_value(form_response.decision_tree.clone())
            .map_err(|e| {
                ServiceError::InvalidData(format!("Generated template is invalid JSON: {}", e))
            })?;

        let validation = intake_engine::validate_template(&template);
        if !validation.is_valid {
            warn!(
                errors = ?validation.errors,
                "AI-generated template failed validation"
            );
            return Err(ServiceError::InvalidData(format!(
                "AI-generated template validation failed: {:?}",
                validation.errors
            )));
        }

        // Create draft template (status: draft, requires admin review)
        let template_id = Uuid::new_v4();
        let template_row = IntakeTemplateRow {
            id: template_id,
            application_type_id: form_response.application_type.id.clone(),
            version: 1,
            decision_tree: form_response.decision_tree.clone(),
            feature_mappings: form_response.feature_mappings.clone(),
            status: "draft".to_string(),
            is_ai_generated: true,
            created_by: Some(user_actor_id.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        control_store::intake::insert_template(self.store.pool(), &template_row).await?;

        info!(
            template_id = %template_id,
            app_type_id = %form_response.application_type.id,
            user_actor_id = %user_actor_id,
            "AI-generated form created (draft)"
        );

        Ok(form_response)
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Parse template from database row to engine format.
    fn parse_template(&self, row: &IntakeTemplateRow) -> Result<Template, ServiceError> {
        serde_json::from_value(row.decision_tree.clone()).map_err(|e| {
            ServiceError::InvalidData(format!("Failed to parse template decision tree: {}", e))
        })
    }

    /// Parse responses from JSONB to engine format.
    fn parse_responses(&self, json: &JsonValue) -> Result<Vec<Response>, ServiceError> {
        // Responses stored as {question_id: answer_json, ...}
        let map = json
            .as_object()
            .ok_or_else(|| ServiceError::InvalidData("Responses must be a JSON object".to_string()))?;

        let mut responses = Vec::new();
        for (question_id, answer_json) in map {
            responses.push(Response {
                question_id: question_id.clone(),
                answer: self.parse_answer(answer_json)?,
                timestamp: Utc::now(), // Original timestamp not preserved in simplified format
            });
        }
        Ok(responses)
    }

    /// Parse answer from JSON to engine format.
    fn parse_answer(&self, json: &JsonValue) -> Result<Answer, ServiceError> {
        if let Some(b) = json.as_bool() {
            return Ok(Answer::YesNo(b));
        }
        if let Some(s) = json.as_str() {
            return Ok(Answer::Choice(s.to_string()));
        }
        if let Some(n) = json.as_i64() {
            return Ok(Answer::Number(n));
        }
        Err(ServiceError::InvalidData(format!(
            "Invalid answer format: {:?}",
            json
        )))
    }

    /// Convert responses to JSON for storage.
    fn responses_to_json(&self, responses: &[Response]) -> Result<JsonValue, ServiceError> {
        let mut map = serde_json::Map::new();
        for response in responses {
            map.insert(
                response.question_id.clone(),
                response.answer.as_json_value(),
            );
        }
        Ok(JsonValue::Object(map))
    }

    /// Convert engine Question to view format.
    fn question_to_view(question: &Question) -> QuestionView {
        QuestionView {
            id: question.id.clone(),
            text: question.text.clone(),
            question_type: match question.question_type {
                intake_engine::QuestionType::YesNo => "yes_no".to_string(),
                intake_engine::QuestionType::MultipleChoice => "multiple_choice".to_string(),
                intake_engine::QuestionType::Text => "text".to_string(),
                intake_engine::QuestionType::Number => "number".to_string(),
            },
            options: question.options.clone(),
            help_text: question.help_text.clone(),
        }
    }
}
