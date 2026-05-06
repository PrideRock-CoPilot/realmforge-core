//! Template validation logic.
//!
//! This module validates intake templates to ensure they are structurally sound,
//! have no circular dependencies, and follow required constraints.

use crate::error::{IntakeError, Result};
use crate::evaluator::evaluate_condition;
use crate::types::{Condition, Question, QuestionId, Template};
use std::collections::{HashMap, HashSet};

/// Validation result with optional warnings
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether the template is valid
    pub is_valid: bool,
    /// Critical errors that prevent template usage
    pub errors: Vec<String>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a valid result with no errors
    pub fn valid() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid result with an error
    pub fn invalid(error: String) -> Self {
        ValidationResult {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add an error to the result
    pub fn with_error(mut self, error: String) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }

    /// Merge another result into this one
    pub fn merge(mut self, other: ValidationResult) -> Self {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }
}

/// Validate an intake template
///
/// This performs comprehensive validation including:
/// - Root question exists
/// - All referenced questions exist
/// - No circular dependencies
/// - All branches have valid next question references
/// - Conditional logic references valid questions
/// - Questions have at least one branch
///
/// # Arguments
/// * `template` - The template to validate
///
/// # Returns
/// * `Ok(ValidationResult)` with validation details
/// * `Err` only if validation logic itself fails (should be rare)
pub fn validate_template(template: &Template) -> Result<ValidationResult> {
    let mut result = ValidationResult::valid();

    // Check 1: Root question exists
    if !template.questions.contains_key(&template.root_question) {
        result = result.with_error(format!(
            "Root question '{}' does not exist in template",
            template.root_question.0
        ));
        // Can't continue validation without root question
        return Ok(result);
    }

    // Check 2: All questions are reachable from root
    let reachable = find_reachable_questions(template);
    let unreachable: Vec<_> = template
        .questions
        .keys()
        .filter(|id| !reachable.contains(id))
        .collect();
    
    if !unreachable.is_empty() {
        for question_id in unreachable {
            result = result.with_warning(format!(
                "Question '{}' is unreachable from root question",
                question_id.0
            ));
        }
    }

    // Check 3: Validate each question
    for (question_id, question) in &template.questions {
        result = result.merge(validate_question(question, &template.questions)?);
    }

    // Check 4: Check for circular dependencies
    match check_circular_dependencies(template) {
        Ok(()) => {},
        Err(e) => {
            result = result.with_error(format!("Circular dependency detected: {}", e));
        }
    }

    Ok(result)
}

/// Validate a single question
fn validate_question(
    question: &Question,
    all_questions: &HashMap<QuestionId, Question>,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::valid();

    // Check: Question must have at least one branch
    if question.branches.is_empty() {
        result = result.with_error(format!(
            "Question '{}' has no branches",
            question.id.0
        ));
    }

    // Check: All branch next references must be valid
    for (answer, branch) in &question.branches {
        if let Some(next_id) = &branch.next {
            if !all_questions.contains_key(next_id) {
                result = result.with_error(format!(
                    "Question '{}' branch '{}' references non-existent question '{}'",
                    question.id.0, answer, next_id.0
                ));
            }
        }
    }

    // Check: Conditional logic references valid questions
    result = result.merge(validate_condition(&question.conditional, all_questions)?);

    Ok(result)
}

/// Validate a condition recursively
fn validate_condition(
    condition: &Condition,
    all_questions: &HashMap<QuestionId, Question>,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::valid();

    match condition {
        Condition::Always => {},
        
        Condition::Equals { field, .. } | Condition::NotEquals { field, .. } => {
            // Extract question ID from field reference
            let question_id = if let Some(stripped) = field.strip_prefix("response.") {
                stripped
            } else {
                field.as_str()
            };

            if !all_questions.contains_key(&QuestionId(question_id.to_string())) {
                result = result.with_warning(format!(
                    "Condition references non-existent question: {}",
                    question_id
                ));
            }
        }
        
        Condition::And { conditions } | Condition::Or { conditions } => {
            for cond in conditions {
                result = result.merge(validate_condition(cond, all_questions)?);
            }
        }
        
        Condition::Not { condition } => {
            result = result.merge(validate_condition(condition, all_questions)?);
        }
    }

    Ok(result)
}

/// Find all questions reachable from the root question
fn find_reachable_questions(template: &Template) -> HashSet<QuestionId> {
    let mut reachable = HashSet::new();
    let mut to_visit = vec![template.root_question.clone()];

    while let Some(question_id) = to_visit.pop() {
        if !reachable.insert(question_id.clone()) {
            // Already visited
            continue;
        }

        if let Some(question) = template.questions.get(&question_id) {
            // Add all branch targets to visit list
            for branch in question.branches.values() {
                if let Some(next_id) = &branch.next {
                    if !reachable.contains(next_id) {
                        to_visit.push(next_id.clone());
                    }
                }
            }
        }
    }

    reachable
}

/// Check for circular dependencies in the template
fn check_circular_dependencies(template: &Template) -> Result<()> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    check_circular_dependencies_recursive(
        &template.root_question,
        &template.questions,
        &mut visited,
        &mut rec_stack,
    )
}

/// Recursive helper for cycle detection using DFS
fn check_circular_dependencies_recursive(
    question_id: &QuestionId,
    all_questions: &HashMap<QuestionId, Question>,
    visited: &mut HashSet<QuestionId>,
    rec_stack: &mut HashSet<QuestionId>,
) -> Result<()> {
    // Mark this node as being in the recursion stack
    rec_stack.insert(question_id.clone());

    if let Some(question) = all_questions.get(question_id) {
        // Visit all branch targets
        for branch in question.branches.values() {
            if let Some(next_id) = &branch.next {
                if rec_stack.contains(next_id) {
                    // Found a cycle
                    return Err(IntakeError::CircularDependency(
                        format!("{} -> {}", question_id.0, next_id.0)
                    ));
                }

                if !visited.contains(next_id) {
                    check_circular_dependencies_recursive(
                        next_id,
                        all_questions,
                        visited,
                        rec_stack,
                    )?;
                }
            }
        }
    }

    // Mark as visited and remove from recursion stack
    visited.insert(question_id.clone());
    rec_stack.remove(question_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_valid_template() -> Template {
        let mut questions = HashMap::new();

        questions.insert(
            QuestionId("Q1".to_string()),
            Question {
                id: QuestionId("Q1".to_string()),
                text: "Question 1".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Always,
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: Some(QuestionId("Q2".to_string())),
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches.insert(
                        "no".to_string(),
                        Branch {
                            next: None,
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches
                },
                required: true,
                help_text: None,
            },
        );

        questions.insert(
            QuestionId("Q2".to_string()),
            Question {
                id: QuestionId("Q2".to_string()),
                text: "Question 2".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Always,
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: None,
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches.insert(
                        "no".to_string(),
                        Branch {
                            next: None,
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches
                },
                required: true,
                help_text: None,
            },
        );

        Template {
            id: TemplateId(Uuid::new_v4()),
            name: "Test Template".to_string(),
            description: "Test".to_string(),
            version: "1.0".to_string(),
            root_question: QuestionId("Q1".to_string()),
            questions,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_ai_generated: false,
            admin_reviewed: true,
        }
    }

    #[test]
    fn test_valid_template() {
        let template = make_valid_template();
        let result = validate_template(&template).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_root_question() {
        let mut template = make_valid_template();
        template.root_question = QuestionId("Q_MISSING".to_string());
        
        let result = validate_template(&template).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Root question")));
    }

    #[test]
    fn test_invalid_branch_reference() {
        let mut template = make_valid_template();
        
        // Add a branch that references a non-existent question
        if let Some(question) = template.questions.get_mut(&QuestionId("Q1".to_string())) {
            question.branches.insert(
                "maybe".to_string(),
                Branch {
                    next: Some(QuestionId("Q_MISSING".to_string())),
                    features: vec![],
                    personas: vec![],
                },
            );
        }
        
        let result = validate_template(&template).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("non-existent question")));
    }

    #[test]
    fn test_circular_dependency() {
        let mut questions = HashMap::new();

        // Q1 -> Q2 -> Q1 (circular)
        questions.insert(
            QuestionId("Q1".to_string()),
            Question {
                id: QuestionId("Q1".to_string()),
                text: "Question 1".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Always,
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: Some(QuestionId("Q2".to_string())),
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches
                },
                required: true,
                help_text: None,
            },
        );

        questions.insert(
            QuestionId("Q2".to_string()),
            Question {
                id: QuestionId("Q2".to_string()),
                text: "Question 2".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Always,
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: Some(QuestionId("Q1".to_string())),
                            features: vec![],
                            personas: vec![],
                        },
                    );
                    branches
                },
                required: true,
                help_text: None,
            },
        );

        let template = Template {
            id: TemplateId(Uuid::new_v4()),
            name: "Circular Template".to_string(),
            description: "Test".to_string(),
            version: "1.0".to_string(),
            root_question: QuestionId("Q1".to_string()),
            questions,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_ai_generated: false,
            admin_reviewed: true,
        };

        let result = validate_template(&template).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Circular dependency")));
    }

    #[test]
    fn test_unreachable_question_warning() {
        let mut template = make_valid_template();
        
        // Add an unreachable question
        template.questions.insert(
            QuestionId("Q_UNREACHABLE".to_string()),
            Question {
                id: QuestionId("Q_UNREACHABLE".to_string()),
                text: "Unreachable Question".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Always,
                branches: HashMap::new(),
                required: true,
                help_text: None,
            },
        );
        
        let result = validate_template(&template).unwrap();
        // Should still be valid, just with a warning
        assert!(result.warnings.iter().any(|w| w.contains("unreachable")));
    }
}
