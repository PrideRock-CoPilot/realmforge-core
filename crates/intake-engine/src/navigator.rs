//! Decision tree navigation logic.
//!
//! This module handles traversing the decision tree to determine which question
//! should be shown next based on the current state and user responses.

use crate::error::{IntakeError, Result};
use crate::evaluator::evaluate_condition;
use crate::types::{IntakeSession, Question, QuestionId, Response, Template};
use std::collections::{HashMap, HashSet};

/// Get the next question that should be shown to the user
///
/// # Arguments
/// * `template` - The intake template containing all questions
/// * `session` - The current intake session with responses
///
/// # Returns
/// * `Ok(Some(question))` if there is a next question to show
/// * `Ok(None)` if the intake is complete
/// * `Err` if navigation logic fails (e.g., circular dependency)
pub fn get_next_question<'a>(
    template: &'a Template,
    session: &IntakeSession,
) -> Result<Option<&'a Question>> {
    // If session is not active, no next question
    if !session.is_active() {
        return Ok(None);
    }

    // Get current question or start from root
    let current_id = match &session.current_question {
        Some(id) => id,
        None => &template.root_question,
    };

    // If we have a response for the current question, find the branch
    if let Some(response) = session.responses.get(current_id) {
        let question = template
            .questions
            .get(current_id)
            .ok_or_else(|| {
                IntakeError::QuestionNotFound(current_id.0.clone())
            })?;

        // Get the branch key from the answer
        let branch_key = response.answer.as_branch_key();
        
        // Look up the branch
        if let Some(branch) = question.branches.get(&branch_key) {
            // If branch has a next question, navigate there
            if let Some(next_id) = &branch.next {
                return find_next_showable_question(
                    template,
                    next_id,
                    &session.responses,
                    &mut HashSet::new(),
                );
            } else {
                // No next question in this branch - intake complete
                return Ok(None);
            }
        } else {
            // No branch for this answer - this shouldn't happen with valid templates
            return Err(IntakeError::ConditionError(
                format!("No branch found for answer: {}", branch_key)
            ));
        }
    } else {
        // No response yet for current question - show it
        return find_next_showable_question(
            template,
            current_id,
            &session.responses,
            &mut HashSet::new(),
        );
    }
}

/// Find the next question that should be shown, skipping questions whose conditions aren't met
///
/// This handles conditional logic by recursively checking conditions and following the tree
/// until we find a question whose condition is satisfied.
///
/// # Arguments
/// * `template` - The intake template
/// * `question_id` - The question ID to check
/// * `responses` - Current responses
/// * `visited` - Set of visited question IDs (for cycle detection)
fn find_next_showable_question<'a>(
    template: &'a Template,
    question_id: &QuestionId,
    responses: &HashMap<QuestionId, Response>,
    visited: &mut HashSet<QuestionId>,
) -> Result<Option<&'a Question>> {
    // Check for circular dependencies
    if !visited.insert(question_id.clone()) {
        return Err(IntakeError::CircularDependency(
            question_id.0.clone()
        ));
    }

    // Get the question
    let question = template
        .questions
        .get(question_id)
        .ok_or_else(|| {
            IntakeError::QuestionNotFound(question_id.0.clone())
        })?;

    // Check if this question's condition is satisfied
    if evaluate_condition(&question.conditional, responses)? {
        // Condition is met - show this question
        return Ok(Some(question));
    }

    // Condition not met - need to skip this question and find what comes after
    // To do this, we need to follow all possible branches and find the next question
    
    // If there's no response for this question yet, we can't determine the branch
    // In this case, the question is skipped and we return None
    if !responses.contains_key(question_id) {
        return Ok(None);
    }

    // Get the response and follow the branch
    let response = responses.get(question_id).unwrap();
    let branch_key = response.answer.as_branch_key();
    
    if let Some(branch) = question.branches.get(&branch_key) {
        if let Some(next_id) = &branch.next {
            // Recursively check the next question
            return find_next_showable_question(template, next_id, responses, visited);
        }
    }

    // No next question found
    Ok(None)
}

/// Derive features and personas from all responses
///
/// This walks through all responses and collects the features and personas
/// from the branches that were taken.
///
/// # Arguments
/// * `template` - The intake template
/// * `responses` - All responses provided
///
/// # Returns
/// * `Ok((features, personas))` - Lists of derived features and personas
pub fn derive_modules_and_personas(
    template: &Template,
    responses: &HashMap<QuestionId, Response>,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut features = Vec::new();
    let mut personas = Vec::new();

    for (question_id, response) in responses {
        // Get the question
        let question = template
            .questions
            .get(question_id)
            .ok_or_else(|| {
                IntakeError::QuestionNotFound(question_id.0.clone())
            })?;

        // Get the branch that was taken
        let branch_key = response.answer.as_branch_key();
        
        if let Some(branch) = question.branches.get(&branch_key) {
            // Add features from this branch
            features.extend(branch.features.clone());
            
            // Add personas from this branch
            personas.extend(branch.personas.clone());
        }
    }

    // Deduplicate while preserving order
    features.sort();
    features.dedup();
    personas.sort();
    personas.dedup();

    Ok((features, personas))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_template() -> Template {
        let mut questions = HashMap::new();

        // Q1: Always shown
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
                            features: vec!["feature_a".to_string()],
                            personas: vec![],
                        },
                    );
                    branches.insert(
                        "no".to_string(),
                        Branch {
                            next: Some(QuestionId("Q3".to_string())),
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

        // Q2: Shown only if Q1 == yes
        questions.insert(
            QuestionId("Q2".to_string()),
            Question {
                id: QuestionId("Q2".to_string()),
                text: "Question 2".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Equals {
                    field: "Q1".to_string(),
                    value: serde_json::json!(true),
                },
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: None,
                            features: vec!["feature_b".to_string()],
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

        // Q3: Shown only if Q1 == no
        questions.insert(
            QuestionId("Q3".to_string()),
            Question {
                id: QuestionId("Q3".to_string()),
                text: "Question 3".to_string(),
                question_type: QuestionType::YesNo,
                conditional: Condition::Equals {
                    field: "Q1".to_string(),
                    value: serde_json::json!(false),
                },
                branches: {
                    let mut branches = HashMap::new();
                    branches.insert(
                        "yes".to_string(),
                        Branch {
                            next: None,
                            features: vec![],
                            personas: vec!["persona_a".to_string()],
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
    fn test_get_next_question_at_start() {
        let template = make_template();
        let session = IntakeSession::new(
            template.id,
            template.root_question.clone(),
        );

        let next = get_next_question(&template, &session).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id.0, "Q1");
    }

    #[test]
    fn test_get_next_question_after_yes() {
        let template = make_template();
        let mut session = IntakeSession::new(
            template.id,
            template.root_question.clone(),
        );

        // Answer Q1 with yes
        session.responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        let next = get_next_question(&template, &session).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id.0, "Q2");
    }

    #[test]
    fn test_get_next_question_after_no() {
        let template = make_template();
        let mut session = IntakeSession::new(
            template.id,
            template.root_question.clone(),
        );

        // Answer Q1 with no
        session.responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(false),
                answered_at: Utc::now(),
            },
        );

        let next = get_next_question(&template, &session).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id.0, "Q3");
    }

    #[test]
    fn test_derive_modules_and_personas() {
        let template = make_template();
        let mut responses = HashMap::new();

        // Answer Q1 with yes -> should get feature_a
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        // Answer Q2 with yes -> should get feature_b
        responses.insert(
            QuestionId("Q2".to_string()),
            Response {
                question_id: QuestionId("Q2".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        let (features, personas) = derive_modules_and_personas(&template, &responses).unwrap();
        
        assert_eq!(features, vec!["feature_a", "feature_b"]);
        assert_eq!(personas, Vec::<String>::new());
    }
}
