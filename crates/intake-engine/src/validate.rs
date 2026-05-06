// ─────────────────────────────────────────────
// validate.rs — Decision Tree Validation
// ─────────────────────────────────────────────
// Validates a TreeDefinition against the schema
// rules: required fields, question references,
// condition references, depth limits, and
// structural integrity.
// ─────────────────────────────────────────────

use crate::error::IntakeEngineError;
use crate::tree::{ConditionExpression, QuestionType, TreeDefinition};

/// Validate a complete tree definition.
///
/// Returns `Ok(())` if valid, or an error describing
/// the first validation failure.
pub fn validate_tree(tree: &TreeDefinition) -> Result<(), IntakeEngineError> {
    // 1. Must have at least one question
    if tree.questions.is_empty() {
        return Err(IntakeEngineError::EmptyTree);
    }

    // 2. Must have a root question
    let root_count = tree
        .questions
        .iter()
        .filter(|q| q.is_root.unwrap_or(false))
        .count();
    if root_count == 0 {
        // If no is_root marker, first question is considered root
        // Not an error — just a convention
    }

    // 3. Max depth must be positive
    if tree.max_depth == 0 {
        return Err(IntakeEngineError::ValidationError(
            "max_depth must be greater than 0".to_string(),
        ));
    }

    // 4. Tree ID must not be empty
    if tree.tree_id.is_empty() {
        return Err(IntakeEngineError::ValidationError(
            "tree_id is required".to_string(),
        ));
    }

    // 5. Version must not be empty
    if tree.version.is_empty() {
        return Err(IntakeEngineError::ValidationError(
            "version is required".to_string(),
        ));
    }

    // 6. Must apply to at least one app type
    if tree.applies_to.is_empty() {
        return Err(IntakeEngineError::ValidationError(
            "applies_to must have at least one entry".to_string(),
        ));
    }

    // 7. Validate each question
    for question in &tree.questions {
        validate_question(question, tree)?;
    }

    // 8. Validate feature mapping references
    for mapping in &tree.feature_mappings {
        let exists = tree
            .questions
            .iter()
            .any(|q| q.question_id == mapping.question_id);
        if !exists {
            return Err(IntakeEngineError::MappingConditionUnknown(
                mapping.question_id.clone(),
            ));
        }
    }

    // 9. Check depth doesn't exceed max questions
    if tree.questions.len() > tree.max_depth * 2 {
        return Err(IntakeEngineError::ValidationError(format!(
            "Number of questions ({}) exceeds reasonable limit for max_depth ({})",
            tree.questions.len(),
            tree.max_depth
        )));
    }

    Ok(())
}

/// Validate a single question.
fn validate_question(
    question: &super::tree::Question,
    tree: &TreeDefinition,
) -> Result<(), IntakeEngineError> {
    // Question ID must not be empty
    if question.question_id.is_empty() {
        return Err(IntakeEngineError::ValidationError(
            "question_id is required".to_string(),
        ));
    }

    // Text must not be empty
    if question.text.is_empty() {
        return Err(IntakeEngineError::ValidationError(format!(
            "Question '{}' has empty text",
            question.question_id
        )));
    }

    // Duplicate question IDs
    let count = tree
        .questions
        .iter()
        .filter(|q| q.question_id == question.question_id)
        .count();
    if count > 1 {
        return Err(IntakeEngineError::ValidationError(format!(
            "Duplicate question_id '{}'",
            question.question_id
        )));
    }

    // Choice questions must have options
    if matches!(
        &question.question_type,
        QuestionType::SingleChoice | QuestionType::MultipleChoice { .. }
    ) && question.options.is_empty()
    {
        return Err(IntakeEngineError::ValidationError(format!(
            "Question '{}' is type {:?} but has no options",
            question.question_id, question.question_type
        )));
    }

    // Choice options must have unique values
    if matches!(
        &question.question_type,
        QuestionType::SingleChoice | QuestionType::MultipleChoice { .. }
    ) {
        let mut seen = std::collections::HashSet::new();
        for option in &question.options {
            if !seen.insert(&option.value) {
                return Err(IntakeEngineError::ValidationError(format!(
                    "Question '{}' has duplicate option value '{}'",
                    question.question_id, option.value
                )));
            }
            if option.value.is_empty() {
                return Err(IntakeEngineError::ValidationError(format!(
                    "Question '{}' has option with empty value",
                    question.question_id
                )));
            }
        }
    }

    // Validate condition references if present
    if let Some(condition) = &question.conditions {
        validate_condition_refs(&condition.if_, tree, &question.question_id)?;
    }

    Ok(())
}

/// Validate condition references to ensure all referenced questions exist.
fn validate_condition_refs(
    expr: &ConditionExpression,
    tree: &TreeDefinition,
    question_id: &str,
) -> Result<(), IntakeEngineError> {
    match expr {
        ConditionExpression::Simple { operands, .. } => {
            for operand in operands {
                if let serde_json::Value::String(s) = operand {
                    if let Some(qid) = s.strip_prefix("question:") {
                        if !tree.questions.iter().any(|q| q.question_id == qid) {
                            return Err(IntakeEngineError::ConditionQuestionNotFound(format!(
                                "Question '{}' references '{}' in condition, but no such question exists",
                                question_id, qid
                            )));
                        }
                    }
                }
            }
            Ok(())
        }
        ConditionExpression::Compound { conditions, .. } => {
            for c in conditions {
                validate_condition_refs(c, tree, question_id)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Question, QuestionOption, QuestionType};

    fn make_valid_tree() -> TreeDefinition {
        TreeDefinition {
            schema: String::new(),
            tree_id: "test-tree".to_string(),
            name: "Test Tree".to_string(),
            version: "1.0.0".to_string(),
            applies_to: vec!["test-app".to_string()],
            description: String::new(),
            max_depth: 5,
            questions: vec![Question {
                question_id: "q1".to_string(),
                question_type: QuestionType::Boolean,
                text: "Question 1?".to_string(),
                options: vec![],
                order: 1,
                required: true,
                ai_assist_prompt: String::new(),
                ai_fallback_eligible: false,
                conditions: None,
                is_root: Some(true),
            }],
            feature_mappings: vec![],
            meta_mappings: std::collections::HashMap::new(),
            plan_stage_handoff: Default::default(),
        }
    }

    #[test]
    fn valid_tree_passes() {
        let tree = make_valid_tree();
        assert!(validate_tree(&tree).is_ok());
    }

    #[test]
    fn empty_tree_fails() {
        let mut tree = make_valid_tree();
        tree.questions.clear();
        assert_eq!(
            validate_tree(&tree).unwrap_err(),
            IntakeEngineError::EmptyTree
        );
    }

    #[test]
    fn missing_tree_id_fails() {
        let mut tree = make_valid_tree();
        tree.tree_id.clear();
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn zero_max_depth_fails() {
        let mut tree = make_valid_tree();
        tree.max_depth = 0;
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn empty_applies_to_fails() {
        let mut tree = make_valid_tree();
        tree.applies_to.clear();
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn empty_question_text_fails() {
        let mut tree = make_valid_tree();
        tree.questions[0].text.clear();
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn choice_without_options_fails() {
        let mut tree = make_valid_tree();
        tree.questions[0].question_type = QuestionType::SingleChoice;
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn duplicate_option_values_fails() {
        let mut tree = make_valid_tree();
        tree.questions[0].question_type = QuestionType::SingleChoice;
        tree.questions[0].options = vec![
            QuestionOption {
                value: "a".to_string(),
                label: "A".to_string(),
                description: String::new(),
                icon: String::new(),
                ai_suggested: false,
            },
            QuestionOption {
                value: "a".to_string(),
                label: "Duplicate A".to_string(),
                description: String::new(),
                icon: String::new(),
                ai_suggested: false,
            },
        ];
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn condition_refers_to_nonexistent_question_fails() {
        let mut tree = make_valid_tree();
        tree.questions.push(Question {
            question_id: "q2".to_string(),
            question_type: QuestionType::Boolean,
            text: "Question 2?".to_string(),
            options: vec![],
            order: 2,
            required: true,
            ai_assist_prompt: String::new(),
            ai_fallback_eligible: false,
            conditions: Some(crate::tree::QuestionCondition {
                if_: ConditionExpression::Simple {
                    operator: "equals".to_string(),
                    operands: vec![
                        serde_json::Value::String("question:nonexistent".to_string()),
                        serde_json::Value::Bool(true),
                    ],
                },
                then_show_ai_banner: false,
                else_hide: true,
            }),
            is_root: None,
        });
        assert!(validate_tree(&tree).is_err());
    }
}
