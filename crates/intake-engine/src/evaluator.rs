//! Conditional logic evaluation for decision trees.
//!
//! This module provides the core logic for evaluating conditional expressions
//! against user responses to determine which questions should be shown.

use crate::error::{IntakeError, Result};
use crate::types::{Condition, QuestionId, Response};
use std::collections::HashMap;

/// Evaluate a condition against the current response set
///
/// # Arguments
/// * `condition` - The condition to evaluate
/// * `responses` - Map of question IDs to responses
///
/// # Returns
/// * `Ok(true)` if the condition is satisfied
/// * `Ok(false)` if the condition is not satisfied
/// * `Err` if the condition cannot be evaluated (e.g., references non-existent field)
pub fn evaluate_condition(
    condition: &Condition,
    responses: &HashMap<QuestionId, Response>,
) -> Result<bool> {
    match condition {
        Condition::Always => Ok(true),
        
        Condition::Equals { field, value } => {
            evaluate_equals(field, value, responses, false)
        }
        
        Condition::NotEquals { field, value } => {
            evaluate_equals(field, value, responses, true)
        }
        
        Condition::And { conditions } => {
            for cond in conditions {
                if !evaluate_condition(cond, responses)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        
        Condition::Or { conditions } => {
            if conditions.is_empty() {
                return Ok(false);
            }
            for cond in conditions {
                if evaluate_condition(cond, responses)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        
        Condition::Not { condition } => {
            Ok(!evaluate_condition(condition, responses)?)
        }
    }
}

/// Evaluate an equality condition
fn evaluate_equals(
    field: &str,
    expected: &serde_json::Value,
    responses: &HashMap<QuestionId, Response>,
    negate: bool,
) -> Result<bool> {
    // Parse field reference (format: "response.question_id" or just "question_id")
    let question_id = if let Some(stripped) = field.strip_prefix("response.") {
        stripped
    } else {
        field
    };

    // Get the response for this question
    let response = responses.get(&QuestionId(question_id.to_string()));
    
    // If no response yet, condition is false (unless negated)
    let Some(response) = response else {
        return Ok(negate);
    };

    // Compare the answer value with expected
    let actual = response.answer.as_json_value();
    let matches = actual == *expected;
    
    Ok(if negate { !matches } else { matches })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Answer, Response, QuestionId};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_response(question_id: &str, answer: Answer) -> (QuestionId, Response) {
        (
            QuestionId(question_id.to_string()),
            Response {
                question_id: QuestionId(question_id.to_string()),
                answer,
                answered_at: Utc::now(),
            },
        )
    }

    #[test]
    fn test_always_condition() {
        let responses = HashMap::new();
        assert!(evaluate_condition(&Condition::Always, &responses).unwrap());
    }

    #[test]
    fn test_equals_condition_match() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::Equals {
            field: "response.Q1".to_string(),
            value: serde_json::json!(true),
        };

        assert!(evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_equals_condition_no_match() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(false),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::Equals {
            field: "Q1".to_string(),
            value: serde_json::json!(true),
        };

        assert!(!evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_not_equals_condition() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::Choice("option_a".to_string()),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::NotEquals {
            field: "Q1".to_string(),
            value: serde_json::json!("option_b"),
        };

        assert!(evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_and_condition_all_true() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );
        responses.insert(
            QuestionId("Q2".to_string()),
            Response {
                question_id: QuestionId("Q2".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::And {
            conditions: vec![
                Condition::Equals {
                    field: "Q1".to_string(),
                    value: serde_json::json!(true),
                },
                Condition::Equals {
                    field: "Q2".to_string(),
                    value: serde_json::json!(true),
                },
            ],
        };

        assert!(evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_and_condition_one_false() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );
        responses.insert(
            QuestionId("Q2".to_string()),
            Response {
                question_id: QuestionId("Q2".to_string()),
                answer: Answer::YesNo(false),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::And {
            conditions: vec![
                Condition::Equals {
                    field: "Q1".to_string(),
                    value: serde_json::json!(true),
                },
                Condition::Equals {
                    field: "Q2".to_string(),
                    value: serde_json::json!(true),
                },
            ],
        };

        assert!(!evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_or_condition_one_true() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(true),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::Or {
            conditions: vec![
                Condition::Equals {
                    field: "Q1".to_string(),
                    value: serde_json::json!(true),
                },
                Condition::Equals {
                    field: "Q2".to_string(),
                    value: serde_json::json!(true),
                },
            ],
        };

        assert!(evaluate_condition(&condition, &responses).unwrap());
    }

    #[test]
    fn test_not_condition() {
        let mut responses = HashMap::new();
        responses.insert(
            QuestionId("Q1".to_string()),
            Response {
                question_id: QuestionId("Q1".to_string()),
                answer: Answer::YesNo(false),
                answered_at: Utc::now(),
            },
        );

        let condition = Condition::Not {
            condition: Box::new(Condition::Equals {
                field: "Q1".to_string(),
                value: serde_json::json!(true),
            }),
        };

        assert!(evaluate_condition(&condition, &responses).unwrap());
    }
}
