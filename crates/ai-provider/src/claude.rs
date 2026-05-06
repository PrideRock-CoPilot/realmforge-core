use crate::{
    AiError, AiMessage, AiProvider, FormGenerationRequest, FormGenerationResponse, HelpRequest,
    HelpResponse, MessageRole, Result, TokenUsage,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Mutex;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";
const CLAUDE_API_VERSION: &str = "2023-06-01";

// Pricing (as of 2024): Claude 3.5 Sonnet
const INPUT_COST_PER_1M: f64 = 3.00;
const OUTPUT_COST_PER_1M: f64 = 15.00;

/// Claude 3.5 Sonnet AI provider implementation.
pub struct ClaudeProvider {
    api_key: String,
    client: Client,
    token_usage: Mutex<TokenUsage>,
}

impl ClaudeProvider {
    /// Create a new Claude provider with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key (from environment or config)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ai_provider::claude::ClaudeProvider;
    ///
    /// let provider = ClaudeProvider::new("sk-ant-...".to_string());
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            token_usage: Mutex::new(TokenUsage::new()),
        }
    }

    /// Create from environment variable ANTHROPIC_API_KEY.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AiError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }

    /// Make a request to the Claude API.
    async fn call_api(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: usize,
    ) -> Result<ClaudeResponse> {
        let request = ClaudeRequest {
            model: CLAUDE_MODEL.to_string(),
            max_tokens,
            system: system_prompt.to_string(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", CLAUDE_API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(AiError::ApiError(format!(
                "Claude API error {}: {}",
                status, body
            )));
        }

        let claude_response: ClaudeResponse = response.json().await?;

        // Update token usage
        let cost = calculate_cost(
            claude_response.usage.input_tokens,
            claude_response.usage.output_tokens,
        );
        self.token_usage
            .lock()
            .map_err(|e| AiError::InternalError(format!("Lock error: {}", e)))?
            .add_usage(
                claude_response.usage.input_tokens,
                claude_response.usage.output_tokens,
                cost,
            );

        Ok(claude_response)
    }

    /// Extract text content from Claude response.
    fn extract_content(response: &ClaudeResponse) -> Result<String> {
        response
            .content
            .first()
            .ok_or_else(|| AiError::InvalidResponse("Empty response content".to_string()))
            .map(|c| c.text.clone())
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    async fn assist_user(&self, request: &HelpRequest) -> Result<HelpResponse> {
        let system_prompt = r#"You are an expert technical consultant helping users complete an intake form for their software project.

Your goal: Ask ONE clarifying question that helps the user move forward.

Rules:
1. Be concise and specific
2. Focus on the immediate blocker
3. Provide context for why this matters
4. Offer 2-3 possible answers if helpful
5. Keep the tone friendly and supportive

Format your response as JSON:
{
  "clarifying_question": "Your question here",
  "explanation": "Why this question helps",
  "guidance": "Optional follow-up guidance"
}"#;

        let user_prompt = format!(
            r#"Session context: {}

User is stuck on: {}

Previous answers: {}

Generate a clarifying question."#,
            request.session_context,
            request.user_message,
            serde_json::to_string_pretty(&request.previous_answers)?
        );

        let response = self.call_api(system_prompt, &user_prompt, 1024).await?;
        let content = Self::extract_content(&response)?;

        // Parse JSON response
        let help_response: HelpResponse = serde_json::from_str(&content).map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse help response: {}", e))
        })?;

        Ok(help_response)
    }

    async fn generate_form(
        &self,
        request: &FormGenerationRequest,
    ) -> Result<FormGenerationResponse> {
        let system_prompt = r#"You are an expert at designing structured intake forms for software projects.

Your goal: Generate a complete decision tree for an intake form based on the user's description.

Requirements:
1. Create 10-15 well-structured questions
2. Use conditional logic (show_if) to create branching paths
3. Map answers to modules and personas
4. Include validation rules
5. Provide clear question text and help text

Decision tree format:
{
  "questions": [
    {
      "id": "q1",
      "text": "Question text",
      "type": "single_choice",
      "options": ["option1", "option2"],
      "help_text": "Explanation",
      "show_if": null,
      "required": true
    }
  ]
}

Feature mappings format:
{
  "modules": {
    "module_name": {"question_ids": ["q1"], "conditions": [{"question_id": "q1", "answer": "value"}]}
  },
  "personas": {
    "persona_name": {"question_ids": ["q2"], "conditions": [{"question_id": "q2", "answer": "value"}]}
  }
}

Respond with JSON:
{
  "application_type": {
    "id": "kebab-case-id",
    "name": "Display Name",
    "description": "...",
    "complexity": "simple|moderate|complex",
    "timeline_weeks": "2-4"
  },
  "decision_tree": {...},
  "feature_mappings": {...},
  "confidence": 0.85,
  "warnings": []
}"#;

        let user_prompt = format!(
            r#"Application description: {}

Hints: {}

Target question count: {}

Generate the intake form."#,
            request.app_description,
            request.hints.join(", "),
            request.target_question_count.unwrap_or(12)
        );

        let response = self.call_api(system_prompt, &user_prompt, 4096).await?;
        let content = Self::extract_content(&response)?;

        // Parse JSON response
        let form_response: FormGenerationResponse =
            serde_json::from_str(&content).map_err(|e| {
                AiError::InvalidResponse(format!("Failed to parse form generation response: {}", e))
            })?;

        Ok(form_response)
    }

    fn provider_name(&self) -> &str {
        CLAUDE_MODEL
    }

    fn get_token_usage(&self) -> TokenUsage {
        self.token_usage
            .lock()
            .map(|usage| usage.clone())
            .unwrap_or_default()
    }

    fn reset_token_usage(&mut self) {
        if let Ok(mut usage) = self.token_usage.lock() {
            *usage = TokenUsage::new();
        }
    }
}

// ============================================================================
// CLAUDE API TYPES
// ============================================================================

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: usize,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u64,
    output_tokens: u64,
}

// ============================================================================
// HELPERS
// ============================================================================

fn calculate_cost(input_tokens: u64, output_tokens: u64) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * INPUT_COST_PER_1M;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * OUTPUT_COST_PER_1M;
    input_cost + output_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        // 1000 input tokens, 500 output tokens
        let cost = calculate_cost(1000, 500);
        let expected = (1000.0 / 1_000_000.0) * 3.0 + (500.0 / 1_000_000.0) * 15.0;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_provider_name() {
        let provider = ClaudeProvider::new("test-key".to_string());
        assert_eq!(provider.provider_name(), CLAUDE_MODEL);
    }

    #[test]
    fn test_token_usage() {
        let mut provider = ClaudeProvider::new("test-key".to_string());
        
        // Initially zero
        let usage = provider.get_token_usage();
        assert_eq!(usage.input_tokens, 0);
        
        // Reset works
        provider.reset_token_usage();
        let usage = provider.get_token_usage();
        assert_eq!(usage.input_tokens, 0);
    }
}
