// ─────────────────────────────────────────────
// tree.rs — Decision Tree Definition Types
// ─────────────────────────────────────────────
// All types needed to represent a complete
// decision tree definition: questions, options,
// conditions, feature mappings, meta mappings,
// and plan stage handoff configuration.
// ─────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete decision tree definition.
/// Can be serialized from JSON or YAML config files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeDefinition {
    /// JSON schema URL for validation
    #[serde(rename = "$schema", default)]
    pub schema: String,

    /// Unique tree identifier (e.g. "tree-web-app")
    pub tree_id: String,

    /// Human-readable name
    pub name: String,

    /// Semantic version of this tree definition
    pub version: String,

    /// Application type IDs this tree applies to
    pub applies_to: Vec<String>,

    /// Description of what this tree covers
    #[serde(default)]
    pub description: String,

    /// Maximum depth for tree traversal
    pub max_depth: usize,

    /// Ordered list of questions in the decision tree
    pub questions: Vec<Question>,

    /// Feature mappings: answer values → modules/personas/skills
    #[serde(default)]
    pub feature_mappings: Vec<FeatureMapping>,

    /// Cross-cutting meta mappings: combinations → modules/personas/skills
    #[serde(default)]
    pub meta_mappings: HashMap<String, MetaMapping>,

    /// Plan stage handoff configuration
    #[serde(default)]
    pub plan_stage_handoff: PlanStageHandoff,
}

/// A single question in the decision tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Question {
    /// Unique question id within this tree
    pub question_id: String,

    /// The type of question determines input method and validation
    pub question_type: QuestionType,

    /// The question text shown to the user
    pub text: String,

    /// Available options (for single_choice, multiple_choice)
    #[serde(default)]
    pub options: Vec<QuestionOption>,

    /// Display order within tree
    pub order: u32,

    /// Whether this question must be answered
    #[serde(default = "default_true")]
    pub required: bool,

    /// AI assistance prompt — shown when user clicks "help"
    #[serde(default)]
    pub ai_assist_prompt: String,

    /// Whether AI fallback is allowed for this question
    #[serde(default)]
    pub ai_fallback_eligible: bool,

    /// Conditional visibility: only show if condition is met
    pub conditions: Option<QuestionCondition>,

    /// Whether the root question of the tree (default: false)
    pub is_root: Option<bool>,
}

fn default_true() -> bool {
    true
}

/// Question type variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "params")]
pub enum QuestionType {
    /// Yes/No
    Boolean,
    /// Pick exactly one from options
    SingleChoice,
    /// Pick N from options with min/max limits
    MultipleChoice {
        #[serde(default)]
        min_selections: u32,
        #[serde(default = "default_max_selections")]
        max_selections: u32,
    },
    /// Free text with optional max length and regex
    Text {
        #[serde(default)]
        max_length: u32,
        #[serde(default)]
        regex: String,
    },
    /// Numeric input with optional range
    Number {
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
    },
    /// Free text with AI suggestion capability
    AiAssistedText,
}

fn default_max_selections() -> u32 {
    5
}

/// A single option for choice-based questions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionOption {
    /// Machine value (e.g. "sso-saml")
    pub value: String,
    /// Human-readable label (e.g. "SSO (SAML/OIDC)")
    pub label: String,
    /// Optional detailed description
    #[serde(default)]
    pub description: String,
    /// Optional icon identifier
    #[serde(default)]
    pub icon: String,
    /// Whether this option is AI-suggested only (not in canonical list)
    #[serde(default)]
    pub ai_suggested: bool,
}

/// Conditional visibility rule for a question
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionCondition {
    /// The condition expression
    pub if_: ConditionExpression,

    /// Show an AI assistance banner when this condition triggers
    #[serde(default)]
    pub then_show_ai_banner: bool,

    /// Hide the question entirely when condition is not met
    #[serde(default)]
    pub else_hide: bool,
}

/// A condition expression for branching or filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionExpression {
    /// Simple comparison: operator takes [operand1, operand2]
    Simple {
        /// The condition operator name
        operator: String,
        /// Operands (question references or literal values)
        operands: Vec<serde_json::Value>,
    },
    /// Compound condition: and/or with nested conditions
    Compound {
        /// Logical operator: "and" or "or"
        operator: String,
        /// Nested conditions
        conditions: Vec<ConditionExpression>,
    },
}

/// Feature mapping: answer value → modules/personas/skills
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureMapping {
    /// The question being mapped
    pub question_id: String,
    /// Answer value that triggers this mapping
    pub value: serde_json::Value,
    /// Modules activated by this mapping
    #[serde(default)]
    pub modules: Vec<String>,
    /// Personas required for this mapping
    #[serde(default)]
    pub personas: Vec<String>,
    /// Skills required for this mapping
    #[serde(default)]
    pub skills: Vec<String>,
}

/// Cross-cutting meta mapping activated by question combinations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaMapping {
    /// Condition that must be satisfied
    pub condition: ConditionExpression,
    /// Modules activated
    #[serde(default)]
    pub modules: Vec<String>,
    /// Personas required
    #[serde(default)]
    pub personas: Vec<String>,
    /// Skills required
    #[serde(default)]
    pub skills: Vec<String>,
}

/// Configuration for handing off to the Plan pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStageHandoff {
    /// Whether to map requirements from answers
    #[serde(default)]
    pub map_requirements: bool,
    /// Whether to generate constraints from answers
    #[serde(default)]
    pub generate_constraints: bool,
    /// Fields to auto-populate from answers for the Plan name
    #[serde(default)]
    pub generate_name_from: Vec<String>,
    /// Default scope template identifier
    #[serde(default)]
    pub default_scope_template: String,
}

/// Runtime answer values collected during an intake session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnswerValue {
    Boolean(bool),
    SingleChoice(String),
    MultipleChoice(Vec<String>),
    Text(String),
    Number(f64),
}

/// Collected results from a completed tree walk
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TreeWalkResult {
    /// All answers mapped by question_id
    pub answers: HashMap<String, AnswerValue>,
    /// Activated modules from feature mappings
    pub modules: Vec<String>,
    /// Required personas from feature mappings
    pub personas: Vec<String>,
    /// Required skills from feature mappings
    pub skills: Vec<String>,
    /// Cross-cutting modules from meta mappings
    pub meta_modules: Vec<String>,
    /// Cross-cutting personas from meta mappings
    pub meta_personas: Vec<String>,
    /// Cross-cutting skills from meta mappings
    pub meta_skills: Vec<String>,
    /// Total number of questions asked during walk
    pub questions_asked: u32,
    /// Total number of applicable questions in tree
    pub questions_total: u32,
    /// Tree depth reached during walk
    pub depth_reached: usize,
    /// Ordered list of question IDs that were actually asked
    pub asked_question_ids: Vec<String>,
}

impl Default for PlanStageHandoff {
    fn default() -> Self {
        Self {
            map_requirements: true,
            generate_constraints: true,
            generate_name_from: Vec::new(),
            default_scope_template: String::new(),
        }
    }
}
