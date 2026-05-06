// ─────────────────────────────────────────────
// condition.rs — Condition Operators
// ─────────────────────────────────────────────
// Evaluates condition expressions against a set
// of collected answers. Supports simple and
// compound (and/or) operators.
// ─────────────────────────────────────────────

use crate::error::IntakeEngineError;
use crate::tree::{AnswerValue, ConditionExpression};
use std::collections::HashMap;

/// Evaluates a condition expression against the current set of answers.
///
/// Returns `true` if the condition is satisfied, `false` if not.
/// Returns `Err` if the condition references unknown questions or
/// hits an evaluation error.
pub fn evaluate_condition(
    condition: &ConditionExpression,
    answers: &HashMap<String, AnswerValue>,
) -> Result<bool, IntakeEngineError> {
    match condition {
        ConditionExpression::Simple { operator, operands } => {
            evaluate_simple(operator.as_str(), operands, answers)
        }
        ConditionExpression::Compound {
            operator,
            conditions,
        } => {
            let op = operator.as_str();
            match op {
                "and" => {
                    for c in conditions {
                        if !evaluate_condition(c, answers)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                "or" => {
                    for c in conditions {
                        if evaluate_condition(c, answers)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                other => Err(IntakeEngineError::ConditionError(format!(
                    "Unknown compound operator '{}' — expected 'and' or 'or'",
                    other
                ))),
            }
        }
    }
}

fn evaluate_simple(
    operator: &str,
    operands: &[serde_json::Value],
    answers: &HashMap<String, AnswerValue>,
) -> Result<bool, IntakeEngineError> {
    match operator {
        "exists" => {
            // exists is a unary operator: needs exactly 1 operand
            if operands.is_empty() {
                return Err(IntakeEngineError::ConditionError(
                    "Operator 'exists' requires at least one operand".to_string(),
                ));
            }
            match &operands[0] {
                serde_json::Value::String(s) if s.starts_with("question:") => {
                    let qid = s.trim_start_matches("question:");
                    Ok(answers.contains_key(qid))
                }
                _ => Err(IntakeEngineError::ConditionError(
                    "'exists' requires a question: reference as first operand".to_string(),
                )),
            }
        }
        "not_exists" => {
            // not_exists is the inverse of exists
            if operands.is_empty() {
                return Err(IntakeEngineError::ConditionError(
                    "Operator 'not_exists' requires at least one operand".to_string(),
                ));
            }
            match &operands[0] {
                serde_json::Value::String(s) if s.starts_with("question:") => {
                    let qid = s.trim_start_matches("question:");
                    Ok(!answers.contains_key(qid))
                }
                _ => Err(IntakeEngineError::ConditionError(
                    "'not_exists' requires a question: reference as first operand".to_string(),
                )),
            }
        }
        _ => {
            // All other operators require at least 2 operands
            if operands.len() < 2 {
                return Err(IntakeEngineError::ConditionError(format!(
                    "Operator '{}' requires at least 2 operands, got {}",
                    operator,
                    operands.len()
                )));
            }

            let left = resolve_value(&operands[0], answers)?;
            let right = resolve_value(&operands[1], answers)?;

            match operator {
                "equals" => Ok(left == right),
                "not_equals" => Ok(left != right),
                "in" => {
                    // right should be an array; check if left is in right
                    match &operands[1] {
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                let resolved = resolve_value(item, answers)?;
                                if left == resolved {
                                    return Ok(true);
                                }
                            }
                            Ok(false)
                        }
                        other => Err(IntakeEngineError::ConditionError(format!(
                            "'in' operator requires an array as second operand, got {}",
                            other
                        ))),
                    }
                }
                "gt" => {
                    let (l, r) = to_numbers(&left, &right, "gt")?;
                    Ok(l > r)
                }
                "gte" => {
                    let (l, r) = to_numbers(&left, &right, "gte")?;
                    Ok(l >= r)
                }
                "lt" => {
                    let (l, r) = to_numbers(&left, &right, "lt")?;
                    Ok(l < r)
                }
                "lte" => {
                    let (l, r) = to_numbers(&left, &right, "lte")?;
                    Ok(l <= r)
                }
                other => Err(IntakeEngineError::ConditionError(format!(
                    "Unknown operator '{}'",
                    other
                ))),
            }
        }
    }
}

/// Resolve a value from either a literal or a question reference.
fn resolve_value(
    value: &serde_json::Value,
    answers: &HashMap<String, AnswerValue>,
) -> Result<serde_json::Value, IntakeEngineError> {
    match value {
        serde_json::Value::String(s) if s.starts_with("question:") => {
            let qid = s.trim_start_matches("question:");
            let answer = answers.get(qid).ok_or_else(|| {
                IntakeEngineError::ConditionError(format!(
                    "Question '{}' referenced in condition but not yet answered",
                    qid
                ))
            })?;
            Ok(answer_to_json(answer))
        }
        other => Ok(other.clone()),
    }
}

/// Convert an AnswerValue to a JSON Value for comparison
fn answer_to_json(answer: &AnswerValue) -> serde_json::Value {
    match answer {
        AnswerValue::Boolean(b) => serde_json::Value::Bool(*b),
        AnswerValue::SingleChoice(s) => serde_json::Value::String(s.clone()),
        AnswerValue::MultipleChoice(v) => serde_json::Value::Array(
            v.iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
        AnswerValue::Text(s) => serde_json::Value::String(s.clone()),
        AnswerValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from_f64(0.0).unwrap()),
        ),
    }
}

/// Extract numeric values for comparison operators.
fn to_numbers(
    left: &serde_json::Value,
    right: &serde_json::Value,
    operator: &str,
) -> Result<(f64, f64), IntakeEngineError> {
    let l = left.as_f64().ok_or_else(|| {
        IntakeEngineError::ConditionError(format!(
            "Operator '{}' requires numeric left operand, got '{}'",
            operator, left
        ))
    })?;
    let r = right.as_f64().ok_or_else(|| {
        IntakeEngineError::ConditionError(format!(
            "Operator '{}' requires numeric right operand, got '{}'",
            operator, right
        ))
    })?;
    Ok((l, r))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_answer(question_id: &str, value: serde_json::Value) -> (String, AnswerValue) {
        match value {
            serde_json::Value::Bool(b) => (question_id.to_string(), AnswerValue::Boolean(b)),
            serde_json::Value::String(s) => (question_id.to_string(), AnswerValue::SingleChoice(s)),
            serde_json::Value::Number(n) => (
                question_id.to_string(),
                AnswerValue::Number(n.as_f64().unwrap()),
            ),
            serde_json::Value::Array(arr) => {
                let v: Vec<String> = arr
                    .iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect();
                (question_id.to_string(), AnswerValue::MultipleChoice(v))
            }
            _ => panic!("Unsupported test value type"),
        }
    }

    fn cond_eq(question: &str, expected: serde_json::Value) -> ConditionExpression {
        ConditionExpression::Simple {
            operator: "equals".to_string(),
            operands: vec![
                serde_json::Value::String(format!("question:{}", question)),
                expected,
            ],
        }
    }

    #[test]
    fn condition_equals_true() {
        let answers = HashMap::from([make_answer("auth_required", serde_json::Value::Bool(true))]);
        let cond = cond_eq("auth_required", serde_json::Value::Bool(true));
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_equals_false() {
        let answers = HashMap::from([make_answer("auth_required", serde_json::Value::Bool(true))]);
        let cond = cond_eq("auth_required", serde_json::Value::Bool(false));
        assert!(!evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_in() {
        let answers = HashMap::from([make_answer(
            "hosting",
            serde_json::Value::String("cloud".to_string()),
        )]);
        let cond = ConditionExpression::Simple {
            operator: "in".to_string(),
            operands: vec![
                serde_json::Value::String("question:hosting".to_string()),
                serde_json::Value::Array(vec![
                    serde_json::Value::String("cloud".to_string()),
                    serde_json::Value::String("edge".to_string()),
                ]),
            ],
        };
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_and() {
        let answers = HashMap::from([
            make_answer("auth_required", serde_json::Value::Bool(true)),
            make_answer("hosting", serde_json::Value::String("cloud".to_string())),
        ]);
        let cond = ConditionExpression::Compound {
            operator: "and".to_string(),
            conditions: vec![
                cond_eq("auth_required", serde_json::Value::Bool(true)),
                cond_eq("hosting", serde_json::Value::String("cloud".to_string())),
            ],
        };
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_or() {
        let answers = HashMap::from([make_answer(
            "hosting",
            serde_json::Value::String("on-prem".to_string()),
        )]);
        let cond = ConditionExpression::Compound {
            operator: "or".to_string(),
            conditions: vec![
                cond_eq("hosting", serde_json::Value::String("cloud".to_string())),
                cond_eq("hosting", serde_json::Value::String("edge".to_string())),
                cond_eq("hosting", serde_json::Value::String("on-prem".to_string())),
            ],
        };
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_gt() {
        let answers = HashMap::from([make_answer("users", serde_json::Value::Number(1000.into()))]);
        let cond = ConditionExpression::Simple {
            operator: "gt".to_string(),
            operands: vec![
                serde_json::Value::String("question:users".to_string()),
                serde_json::Value::Number(500.into()),
            ],
        };
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_exists() {
        let answers = HashMap::from([make_answer(
            "auth_type",
            serde_json::Value::String("sso".to_string()),
        )]);
        let cond = ConditionExpression::Simple {
            operator: "exists".to_string(),
            operands: vec![serde_json::Value::String("question:auth_type".to_string())],
        };
        assert!(evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn condition_not_exists() {
        let answers: HashMap<String, AnswerValue> = HashMap::new();
        let cond = ConditionExpression::Simple {
            operator: "exists".to_string(),
            operands: vec![serde_json::Value::String("question:auth_type".to_string())],
        };
        assert!(!evaluate_condition(&cond, &answers).unwrap());
    }

    #[test]
    fn unknown_operator_returns_error() {
        let answers = HashMap::new();
        let cond = ConditionExpression::Simple {
            operator: "unknown_op".to_string(),
            operands: vec![
                serde_json::Value::String("a".to_string()),
                serde_json::Value::String("b".to_string()),
            ],
        };
        assert!(evaluate_condition(&cond, &answers).is_err());
    }
}
