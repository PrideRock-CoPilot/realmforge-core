use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ============================================================================
// USER ASSISTANCE
// ============================================================================

/// Request for AI assistance when user is stuck.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpRequest {
    /// Full context of the intake session so far
    pub session_context: String,

    /// The specific question or confusion the user has
    pub user_message: String,

    /// Previous answers in the session (for context)
    pub previous_answers: Vec<(String, JsonValue)>,
}

/// Response with a clarifying question to help the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpResponse {
    /// The clarifying question to ask the user
    pub clarifying_question: String,

    /// Context or explanation for why this question helps
    pub explanation: String,

    /// Suggested follow-up guidance
    pub guidance: Option<String>,
}

// ============================================================================
// FORM GENERATION
// ============================================================================

/// Request to generate a draft intake form for a new application type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormGenerationRequest {
    /// Natural language description of the application type
    pub app_description: String,

    /// Optional hints about complexity or specific requirements
    pub hints: Vec<String>,

    /// Target number of questions (default: 10-15)
    pub target_question_count: Option<usize>,
}

/// Response with a generated intake template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormGenerationResponse {
    /// Generated application type metadata
    pub application_type: ApplicationTypeMetadata,

    /// Complete decision tree (JSONB-compatible)
    pub decision_tree: JsonValue,

    /// Feature mappings (modules and personas)
    pub feature_mappings: JsonValue,

    /// AI's confidence in this generation (0.0-1.0)
    pub confidence: f32,

    /// Warnings or notes about the generated form
    pub warnings: Vec<String>,
}

/// Metadata for a generated application type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTypeMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub complexity: String,
    pub timeline_weeks: String,
}

// ============================================================================
// AI MESSAGE TYPES (Generic)
// ============================================================================

/// Generic AI message for conversation-style APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Role of a message in conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Generic AI request wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub messages: Vec<AiMessage>,
    pub max_tokens: usize,
    pub temperature: f32,
}

impl AiRequest {
    pub fn new(system_prompt: &str, user_prompt: &str) -> Self {
        Self {
            messages: vec![
                AiMessage {
                    role: MessageRole::System,
                    content: system_prompt.to_string(),
                },
                AiMessage {
                    role: MessageRole::User,
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

/// Generic AI response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}
