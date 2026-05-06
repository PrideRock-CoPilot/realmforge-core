use thiserror::Error;

/// AI provider errors.
#[derive(Debug, Error)]
pub enum AiError {
    /// Network or API communication error.
    #[error("API request failed: {0}")]
    ApiError(String),

    /// Invalid API key or authentication failure.
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// AI provider returned an invalid or unexpected response.
    #[error("Invalid response from AI provider: {0}")]
    InvalidResponse(String),

    /// Request validation error (malformed input).
    #[error("Invalid request: {0}")]
    ValidationError(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Configuration error (missing API keys, etc.).
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Unexpected internal error.
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for AI provider operations.
pub type Result<T> = std::result::Result<T, AiError>;

impl AiError {
    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AiError::ApiError(_) | AiError::RateLimitError(_) | AiError::HttpError(_)
        )
    }

    /// Get HTTP status code hint for this error (for API responses).
    pub fn status_code(&self) -> u16 {
        match self {
            AiError::AuthError(_) => 401,
            AiError::RateLimitError(_) => 429,
            AiError::ValidationError(_) => 400,
            _ => 500,
        }
    }
}
