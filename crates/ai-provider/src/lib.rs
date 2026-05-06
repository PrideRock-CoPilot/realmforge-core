//! AI provider abstraction for RealmForge intake system.
//!
//! Provides a pluggable architecture for AI providers (Claude, GPT, etc.)
//! following ADR-001 Decision 2: Pluggable AI Provider Architecture.

pub mod claude;
pub mod error;
pub mod types;

pub use error::{AiError, Result};
pub use types::{
    AiMessage, AiRequest, AiResponse, FormGenerationRequest, FormGenerationResponse, HelpRequest,
    HelpResponse, MessageRole,
};

use async_trait::async_trait;

/// AI provider trait for intake system operations.
///
/// Implementations must provide:
/// - User assistance (clarifying questions)
/// - Form generation (AI-generated templates)
/// - Cost tracking and rate limiting
///
/// # Example
///
/// ```no_run
/// use ai_provider::{AiProvider, HelpRequest};
///
/// async fn assist_user(provider: &dyn AiProvider) {
///     let request = HelpRequest {
///         session_context: "User stuck on authentication question".to_string(),
///         user_message: "Not sure if I need SSO".to_string(),
///         previous_answers: vec![],
///     };
///     
///     let response = provider.assist_user(&request).await.unwrap();
///     println!("AI: {}", response.clarifying_question);
/// }
/// ```
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Assist a stuck user with a clarifying question.
    ///
    /// Given the session context and user's confusion, generates a targeted
    /// clarifying question to help them proceed.
    ///
    /// # Arguments
    ///
    /// * `request` - Context about the user's situation and confusion
    ///
    /// # Returns
    ///
    /// A clarifying question and explanation to help the user
    async fn assist_user(&self, request: &HelpRequest) -> Result<HelpResponse>;

    /// Generate a draft intake form from a natural language description.
    ///
    /// Creates a complete decision tree template for a new application type
    /// based on the user's description. The generated form is marked as draft
    /// and requires admin review before being promoted to active.
    ///
    /// # Arguments
    ///
    /// * `request` - Description of the application type and generation hints
    ///
    /// # Returns
    ///
    /// A complete intake template (decision tree + metadata) marked as draft
    async fn generate_form(&self, request: &FormGenerationRequest)
        -> Result<FormGenerationResponse>;

    /// Get the provider's name (e.g., "claude-3.5", "gpt-4").
    fn provider_name(&self) -> &str;

    /// Get the current token usage for cost tracking.
    fn get_token_usage(&self) -> TokenUsage;

    /// Reset token usage counters (called at billing period boundaries).
    fn reset_token_usage(&mut self);
}

/// Token usage tracking for cost monitoring.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost_usd: f64,
}

impl TokenUsage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_usage(&mut self, input: u64, output: u64, cost: f64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.total_cost_usd += cost;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_tracking() {
        let mut usage = TokenUsage::new();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total_cost_usd, 0.0);

        usage.add_usage(100, 50, 0.01);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_cost_usd, 0.01);

        usage.add_usage(200, 100, 0.02);
        assert_eq!(usage.input_tokens, 300);
        assert_eq!(usage.output_tokens, 150);
        assert_eq!(usage.total_cost_usd, 0.03);
    }
}
