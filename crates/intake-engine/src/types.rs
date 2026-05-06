//! Core types for the intake engine.
//!
//! This module defines all data structures used in the intake system, including
//! templates, questions, answers, sessions, and conditional logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Unique identifier for an intake template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(pub Uuid);

/// Unique identifier for an intake session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

/// Unique identifier for a question within a template
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(pub String);

impl From<&str> for QuestionId {
    fn from(s: &str) -> Self {
        QuestionId(s.to_string())
    }
}

/// Type of question
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionType {
    /// Yes/No question
    YesNo,
    /// Multiple choice (single selection)
    MultipleChoice { choices: Vec<String> },
    /// Free-form text input
    Text { max_length: Option<usize> },
    /// Numeric input
    Number { min: Option<i64>, max: Option<i64> },
}

/// Conditional expression for showing/hiding questions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Always show (default)
    Always,
    /// Equality check: response.field == value
    Equals {
        field: String,
        value: serde_json::Value,
    },
    /// Not equals check
    NotEquals {
        field: String,
        value: serde_json::Value,
    },
    /// AND combination of conditions
    And { conditions: Vec<Condition> },
    /// OR combination of conditions
    Or { conditions: Vec<Condition> },
    /// NOT (negation)
    Not { condition: Box<Condition> },
}

/// Branch configuration based on answer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Branch {
    /// Next question ID (None if end of path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<QuestionId>,
    
    /// Features/modules enabled by this branch
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    
    /// Personas enabled by this branch
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub personas: Vec<String>,
}

/// A question in the intake template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    /// Unique identifier for this question
    pub id: QuestionId,
    
    /// Question text shown to user
    pub text: String,
    
    /// Type of question (determines valid answers)
    #[serde(flatten)]
    pub question_type: QuestionType,
    
    /// Conditional logic for showing this question
    #[serde(default, skip_serializing_if = "Condition::is_always")]
    pub conditional: Condition,
    
    /// Branches based on answer (key is answer value)
    pub branches: HashMap<String, Branch>,
    
    /// Whether this question is required
    #[serde(default = "default_true")]
    pub required: bool,
    
    /// Help text or additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_text: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Condition {
    fn is_always(&self) -> bool {
        matches!(self, Condition::Always)
    }
}

/// An intake template with decision tree structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    /// Template identifier
    pub id: TemplateId,
    
    /// Application type name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Version of this template
    pub version: String,
    
    /// Root question ID (starting point)
    pub root_question: QuestionId,
    
    /// All questions in this template (keyed by ID)
    pub questions: HashMap<QuestionId, Question>,
    
    /// When this template was created
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Whether this template was AI-generated
    #[serde(default)]
    pub is_ai_generated: bool,
    
    /// Whether this template has been reviewed by an admin
    #[serde(default)]
    pub admin_reviewed: bool,
}

/// An answer provided by the user
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    /// Yes/No answer
    YesNo(bool),
    /// Multiple choice selection
    Choice(String),
    /// Text answer
    Text(String),
    /// Numeric answer
    Number(i64),
}

impl Answer {
    /// Get the string representation for branching logic
    pub fn as_branch_key(&self) -> String {
        match self {
            Answer::YesNo(true) => "yes".to_string(),
            Answer::YesNo(false) => "no".to_string(),
            Answer::Choice(s) => s.clone(),
            Answer::Text(s) => s.clone(),
            Answer::Number(n) => n.to_string(),
        }
    }

    /// Get the value as JSON for condition evaluation
    pub fn as_json_value(&self) -> serde_json::Value {
        match self {
            Answer::YesNo(b) => serde_json::json!(b),
            Answer::Choice(s) => serde_json::json!(s),
            Answer::Text(s) => serde_json::json!(s),
            Answer::Number(n) => serde_json::json!(n),
        }
    }
}

/// A response to a specific question
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Question that was answered
    pub question_id: QuestionId,
    
    /// Answer provided
    pub answer: Answer,
    
    /// When this answer was provided
    pub answered_at: DateTime<Utc>,
}

/// Current status of an intake session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is active (in progress)
    Active,
    /// Session is completed
    Completed,
    /// Session was abandoned
    Abandoned,
}

/// An intake session tracking user progress through a template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntakeSession {
    /// Session identifier
    pub id: SessionId,
    
    /// Template being used
    pub template_id: TemplateId,
    
    /// Current question ID (None if completed or not started)
    pub current_question: Option<QuestionId>,
    
    /// All responses provided so far (keyed by question ID)
    pub responses: HashMap<QuestionId, Response>,
    
    /// Derived features/modules based on answers
    #[serde(default)]
    pub derived_features: Vec<String>,
    
    /// Derived personas based on answers
    #[serde(default)]
    pub derived_personas: Vec<String>,
    
    /// Session status
    pub status: SessionStatus,
    
    /// When this session was created
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// When this session was completed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl IntakeSession {
    /// Create a new intake session
    pub fn new(template_id: TemplateId, root_question: QuestionId) -> Self {
        let now = Utc::now();
        IntakeSession {
            id: SessionId(Uuid::new_v4()),
            template_id,
            current_question: Some(root_question),
            responses: HashMap::new(),
            derived_features: Vec::new(),
            derived_personas: Vec::new(),
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Check if session is still active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    /// Mark session as completed
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.current_question = None;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}
