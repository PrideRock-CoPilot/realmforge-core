use control_service::ServiceContext;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::McpError;

// ============================================================================
// ARGUMENT TYPES
// ============================================================================

/// Arguments for starting an intake session.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntakeStartArgs {
    pub actor_id: String,
    pub application_type_id: String,
}

/// Arguments for answering an intake question.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntakeAnswerArgs {
    pub session_id: Uuid,
    pub question_id: String,
    pub answer: Value,
}

/// Arguments for completing an intake session.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntakeCompleteArgs {
    pub session_id: Uuid,
    pub project_id: Option<String>,
}

/// Arguments for AI assistance during intake.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntakeAiAssistArgs {
    pub session_id: Uuid,
    pub stuck_question_id: String,
    pub user_description: String,
}

/// Arguments for AI form generation.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntakeGenerateFormArgs {
    pub actor_id: String,
    pub app_description: String,
    pub hints: Option<Vec<String>>,
}

// ============================================================================
// MCP TOOLS
// ============================================================================

/// intake_start — Start a new structured intake session.
///
/// Loads the active template for the specified application type and creates
/// a new session with the root question ready.
///
/// Returns: { session_id, template_id, first_question }
pub async fn intake_start(
    args: IntakeStartArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let state = ctx
        .intake
        .start_intake(&args.application_type_id, &args.actor_id)
        .await?;

    Ok(json!({
        "session_id": state.session_id,
        "application_type_id": state.application_type_id,
        "first_question": state.current_question,
        "progress_percent": state.progress_percent,
    }))
}

/// intake_answer — Submit an answer and get the next question.
///
/// Records the user's answer, derives modules/personas if applicable,
/// and returns the next question in the decision tree.
///
/// Returns: { next_question, derived_modules, completion_percentage }
pub async fn intake_answer(
    args: IntakeAnswerArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let answer_submission = control_service::intake_service::AnswerSubmission {
        question_id: args.question_id,
        answer: args.answer,
    };

    let state = ctx
        .intake
        .answer_question(args.session_id, answer_submission)
        .await?;

    Ok(json!({
        "session_id": state.session_id,
        "next_question": state.current_question,
        "derived_modules": state.derived_modules,
        "derived_personas": state.derived_personas,
        "progress_percent": state.progress_percent,
        "responses_count": state.responses_count,
    }))
}

/// intake_complete — Complete an intake session and finalize configuration.
///
/// Marks the session as complete and links it to a project if provided.
///
/// Returns: { session_id, status }
pub async fn intake_complete(
    args: IntakeCompleteArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    ctx.intake
        .complete_intake(args.session_id, args.project_id.as_deref())
        .await?;

    Ok(json!({
        "session_id": args.session_id,
        "status": "completed",
        "project_id": args.project_id,
    }))
}

/// intake_ai_assist — Get AI assistance when stuck on a question.
///
/// Generates a clarifying question based on session context and user confusion.
/// Logs the AI interaction for frequency tracking.
///
/// Returns: { ai_question, context }
pub async fn intake_ai_assist(
    args: IntakeAiAssistArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let help_response = ctx
        .intake
        .ai_assist(
            args.session_id,
            &args.stuck_question_id,
            &args.user_description,
        )
        .await?;

    Ok(json!({
        "clarifying_question": help_response.clarifying_question,
        "explanation": help_response.explanation,
        "guidance": help_response.guidance,
    }))
}

/// intake_generate_form — Generate a draft intake form from description.
///
/// Uses AI to create a complete decision tree template for a new application type.
/// The generated form is marked as draft and requires admin review.
///
/// Returns: { application_type, template, session_id }
pub async fn intake_generate_form(
    args: IntakeGenerateFormArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let hints = args.hints.unwrap_or_default();

    let form_response = ctx
        .intake
        .generate_form(&args.actor_id, &args.app_description, hints)
        .await?;

    Ok(json!({
        "application_type": form_response.application_type,
        "confidence": form_response.confidence,
        "warnings": form_response.warnings,
        "question_count": form_response.decision_tree.get("questions")
            .and_then(|q| q.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0),
    }))
}
