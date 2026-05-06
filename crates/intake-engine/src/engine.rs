// ─────────────────────────────────────────────
// engine.rs — Decision Tree Walking Engine
// ─────────────────────────────────────────────
// Walks a decision tree by evaluating conditions,
// presenting questions to the caller one at a time,
// and collecting answers.
//
// The engine is stateless — it takes a tree and
// answers, and returns the next question or the
// final result. Callers manage session state.
// ─────────────────────────────────────────────

use crate::condition::evaluate_condition;
use crate::error::IntakeEngineError;
use crate::tree::{
    AnswerValue, ConditionExpression, Question, QuestionCondition, TreeDefinition, TreeWalkResult,
};
use std::collections::HashMap;

type MappingTriple = (Vec<String>, Vec<String>, Vec<String>);

/// Determine the next question to ask in the tree walk.
///
/// Returns:
/// - `Ok(Some(&Question))` — the next question to ask
/// - `Ok(None)` — the walk is complete, all questions answered
/// - `Err(...)` — the tree has an error
pub fn next_question<'a>(
    tree: &'a TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<Option<&'a Question>, IntakeEngineError> {
    // Find questions that are visible but not yet answered
    for question in &tree.questions {
        if answers.contains_key(&question.question_id) {
            continue; // Already answered
        }

        if is_question_visible(question, tree, answers)? {
            return Ok(Some(question));
        }
    }

    Ok(None) // All visible questions answered
}

/// Walk the entire tree and collect all answers.
/// This is a convenience method — callers can also
/// call next_question() + submit_answer() in a loop.
///
/// The `walk_tree` function expects all answers to be pre-populated.
/// It computes which questions are visible under the given answers,
/// reports which ones would be asked, and maps answers to
/// modules/personas/skills via feature and meta mappings.
pub fn walk_tree(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<TreeWalkResult, IntakeEngineError> {
    let mut asked_question_ids: Vec<String> = Vec::new();

    // Walk through all questions and identify which are visible (would be asked)
    // and verify all visible questions have answers.
    for question in &tree.questions {
        if is_question_visible(question, tree, answers)? {
            asked_question_ids.push(question.question_id.clone());
            if !answers.contains_key(&question.question_id) {
                return Err(IntakeEngineError::UnknownQuestion(format!(
                    "Question '{}' ('{}') is visible but has no answer in input set. \
                     Use next_question() + submit_answer() for interactive walk.",
                    question.question_id, question.text
                )));
            }
        }
    }

    // Compute total visible questions
    let total_visible = count_visible_questions(tree, answers)?;
    let depth = compute_depth(tree, answers)?;

    // Apply feature mappings
    let (modules, personas, skills) = apply_feature_mappings(tree, answers)?;

    // Apply meta mappings
    let (meta_modules, meta_personas, meta_skills) = apply_meta_mappings(tree, answers)?;

    Ok(TreeWalkResult {
        answers: answers.clone(),
        modules,
        personas,
        skills,
        meta_modules,
        meta_personas,
        meta_skills,
        questions_asked: asked_question_ids.len() as u32,
        questions_total: total_visible as u32,
        depth_reached: depth,
        asked_question_ids,
    })
}

/// Check whether a question is visible given current answers.
fn is_question_visible(
    question: &Question,
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<bool, IntakeEngineError> {
    match &question.conditions {
        None => Ok(true), // No condition → always visible
        Some(condition) => evaluate_question_condition(condition, tree, answers),
    }
}

/// Evaluate a question's condition expression.
fn evaluate_question_condition(
    condition: &QuestionCondition,
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<bool, IntakeEngineError> {
    // Resolve condition references to ensure referenced questions exist
    validate_condition_question_refs(&condition.if_, tree)?;

    let result = evaluate_condition(&condition.if_, answers)?;

    if result {
        Ok(true)
    } else if condition.else_hide {
        Ok(false)
    } else {
        // Default: show the question even if condition is false
        Ok(true)
    }
}

/// Validate that all question references in a condition exist in the tree.
fn validate_condition_question_refs(
    expr: &ConditionExpression,
    tree: &TreeDefinition,
) -> Result<(), IntakeEngineError> {
    match expr {
        ConditionExpression::Simple { operands, .. } => {
            for operand in operands {
                if let serde_json::Value::String(s) = operand {
                    if let Some(qid) = s.strip_prefix("question:") {
                        if !tree.questions.iter().any(|q| q.question_id == qid) {
                            return Err(IntakeEngineError::ConditionQuestionNotFound(
                                qid.to_string(),
                            ));
                        }
                    }
                }
            }
            Ok(())
        }
        ConditionExpression::Compound { conditions, .. } => {
            for c in conditions {
                validate_condition_question_refs(c, tree)?;
            }
            Ok(())
        }
    }
}

/// Count total visible questions in the tree for the given answers.
fn count_visible_questions(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<usize, IntakeEngineError> {
    let mut count = 0;
    for question in &tree.questions {
        if is_question_visible(question, tree, answers)? {
            count += 1;
        }
    }
    Ok(count)
}

/// Compute the maximum depth reached by the answer set.
fn compute_depth(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<usize, IntakeEngineError> {
    // Simple depth = number of answered questions
    let answered = answers.len();
    if answered > tree.max_depth {
        return Err(IntakeEngineError::MaxDepthExceeded {
            max_depth: tree.max_depth,
            actual: answered,
        });
    }
    Ok(answered)
}

/// Apply feature mappings: answer values → modules/personas/skills
fn apply_feature_mappings(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<MappingTriple, IntakeEngineError> {
    let mut modules = Vec::new();
    let mut personas = Vec::new();
    let mut skills = Vec::new();

    for mapping in &tree.feature_mappings {
        let answer = answers.get(&mapping.question_id).ok_or_else(|| {
            IntakeEngineError::MappingConditionUnknown(mapping.question_id.clone())
        })?;

        let answer_json = answer_to_json_value(answer);
        if answer_json == mapping.value {
            modules.extend(mapping.modules.clone());
            personas.extend(mapping.personas.clone());
            skills.extend(mapping.skills.clone());
        }
    }

    Ok((
        deduplicate(modules),
        deduplicate(personas),
        deduplicate(skills),
    ))
}

/// Apply meta mappings: cross-cutting combinations → modules/personas/skills
fn apply_meta_mappings(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<MappingTriple, IntakeEngineError> {
    let mut modules = Vec::new();
    let mut personas = Vec::new();
    let mut skills = Vec::new();

    for meta in tree.meta_mappings.values() {
        let satisfied = evaluate_condition(&meta.condition, answers)
            .map_err(|e| IntakeEngineError::MetaMappingError(e.to_string()))?;

        if satisfied {
            modules.extend(meta.modules.clone());
            personas.extend(meta.personas.clone());
            skills.extend(meta.skills.clone());
        }
    }

    Ok((
        deduplicate(modules),
        deduplicate(personas),
        deduplicate(skills),
    ))
}

/// Convert an AnswerValue to serde_json::Value for comparison
fn answer_to_json_value(answer: &AnswerValue) -> serde_json::Value {
    match answer {
        AnswerValue::Boolean(b) => serde_json::Value::Bool(*b),
        AnswerValue::SingleChoice(s) | AnswerValue::Text(s) => serde_json::Value::String(s.clone()),
        AnswerValue::MultipleChoice(v) => serde_json::Value::Array(
            v.iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
        AnswerValue::Number(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
    }
}

/// Deduplicate and preserve order
fn deduplicate(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{FeatureMapping, Question, QuestionOption, QuestionType};

    fn make_test_tree() -> TreeDefinition {
        TreeDefinition {
            schema: String::new(),
            tree_id: "test-tree".to_string(),
            name: "Test Tree".to_string(),
            version: "1.0.0".to_string(),
            applies_to: vec!["test-app".to_string()],
            description: String::new(),
            max_depth: 10,
            questions: vec![
                Question {
                    question_id: "app_name".to_string(),
                    question_type: QuestionType::Text {
                        max_length: 100,
                        regex: String::new(),
                    },
                    text: "What is your app name?".to_string(),
                    options: vec![],
                    order: 1,
                    required: true,
                    ai_assist_prompt: String::new(),
                    ai_fallback_eligible: false,
                    conditions: None,
                    is_root: Some(true),
                },
                Question {
                    question_id: "auth_required".to_string(),
                    question_type: QuestionType::Boolean,
                    text: "Does this app need authentication?".to_string(),
                    options: vec![],
                    order: 2,
                    required: true,
                    ai_assist_prompt: String::new(),
                    ai_fallback_eligible: false,
                    conditions: None,
                    is_root: None,
                },
                Question {
                    question_id: "auth_type".to_string(),
                    question_type: QuestionType::SingleChoice,
                    text: "What authentication type?".to_string(),
                    options: vec![
                        QuestionOption {
                            value: "email-password".to_string(),
                            label: "Email + Password".to_string(),
                            description: String::new(),
                            icon: String::new(),
                            ai_suggested: false,
                        },
                        QuestionOption {
                            value: "sso".to_string(),
                            label: "SSO".to_string(),
                            description: String::new(),
                            icon: String::new(),
                            ai_suggested: false,
                        },
                    ],
                    order: 3,
                    required: true,
                    ai_assist_prompt: String::new(),
                    ai_fallback_eligible: false,
                    conditions: Some(QuestionCondition {
                        if_: ConditionExpression::Simple {
                            operator: "equals".to_string(),
                            operands: vec![
                                serde_json::Value::String("question:auth_required".to_string()),
                                serde_json::Value::Bool(true),
                            ],
                        },
                        then_show_ai_banner: false,
                        else_hide: true,
                    }),
                    is_root: None,
                },
                Question {
                    question_id: "db_type".to_string(),
                    question_type: QuestionType::SingleChoice,
                    text: "What database?".to_string(),
                    options: vec![
                        QuestionOption {
                            value: "pg".to_string(),
                            label: "PostgreSQL".to_string(),
                            description: String::new(),
                            icon: String::new(),
                            ai_suggested: false,
                        },
                        QuestionOption {
                            value: "none".to_string(),
                            label: "No database".to_string(),
                            description: String::new(),
                            icon: String::new(),
                            ai_suggested: false,
                        },
                    ],
                    order: 4,
                    required: true,
                    ai_assist_prompt: String::new(),
                    ai_fallback_eligible: false,
                    conditions: None,
                    is_root: None,
                },
            ],
            feature_mappings: vec![
                FeatureMapping {
                    question_id: "auth_required".to_string(),
                    value: serde_json::Value::Bool(true),
                    modules: vec!["auth-module".to_string()],
                    personas: vec!["backend-auth".to_string()],
                    skills: vec!["backend".to_string()],
                },
                FeatureMapping {
                    question_id: "auth_type".to_string(),
                    value: serde_json::Value::String("sso".to_string()),
                    modules: vec!["auth-module-sso".to_string()],
                    personas: vec!["backend-auth".to_string()],
                    skills: vec!["backend".to_string()],
                },
                FeatureMapping {
                    question_id: "db_type".to_string(),
                    value: serde_json::Value::String("pg".to_string()),
                    modules: vec!["db-module-pg".to_string()],
                    personas: vec!["backend-data".to_string()],
                    skills: vec!["backend".to_string()],
                },
            ],
            meta_mappings: HashMap::new(),
            plan_stage_handoff: Default::default(),
        }
    }

    #[test]
    fn next_question_returns_first_unanswered() {
        let tree = make_test_tree();
        let answers = HashMap::new();
        let next = next_question(&tree, &answers).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().question_id, "app_name");
    }

    #[test]
    fn next_question_returns_conditional() {
        let tree = make_test_tree();
        let mut answers = HashMap::new();
        answers.insert(
            "app_name".to_string(),
            AnswerValue::Text("MyApp".to_string()),
        );
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(true));
        answers.insert(
            "db_type".to_string(),
            AnswerValue::SingleChoice("pg".to_string()),
        );

        let next = next_question(&tree, &answers).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().question_id, "auth_type");
    }

    #[test]
    fn next_question_hides_conditional_when_false() {
        let tree = make_test_tree();
        let mut answers = HashMap::new();
        answers.insert(
            "app_name".to_string(),
            AnswerValue::Text("MyApp".to_string()),
        );
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(false));
        answers.insert(
            "db_type".to_string(),
            AnswerValue::SingleChoice("pg".to_string()),
        );

        // auth_type should not be visible because auth_required=false
        let next = next_question(&tree, &answers).unwrap();
        assert!(next.is_none()); // All visible questions answered
    }

    #[test]
    fn walk_tree_completes() {
        let tree = make_test_tree();
        let mut answers = HashMap::new();
        answers.insert(
            "app_name".to_string(),
            AnswerValue::Text("MyApp".to_string()),
        );
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(true));
        answers.insert(
            "auth_type".to_string(),
            AnswerValue::SingleChoice("sso".to_string()),
        );
        answers.insert(
            "db_type".to_string(),
            AnswerValue::SingleChoice("pg".to_string()),
        );

        let result = walk_tree(&tree, &answers).unwrap();
        assert_eq!(result.questions_asked, 4);
        assert_eq!(result.questions_total, 4);
    }

    #[test]
    fn walk_tree_feature_mappings() {
        let tree = make_test_tree();
        let mut answers = HashMap::new();
        answers.insert(
            "app_name".to_string(),
            AnswerValue::Text("MyApp".to_string()),
        );
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(true));
        answers.insert(
            "auth_type".to_string(),
            AnswerValue::SingleChoice("sso".to_string()),
        );
        answers.insert(
            "db_type".to_string(),
            AnswerValue::SingleChoice("pg".to_string()),
        );

        let result = walk_tree(&tree, &answers).unwrap();
        assert!(result.modules.contains(&"auth-module".to_string()));
        assert!(result.modules.contains(&"auth-module-sso".to_string()));
        assert!(result.modules.contains(&"db-module-pg".to_string()));
        assert!(result.personas.contains(&"backend-auth".to_string()));
        assert!(result.personas.contains(&"backend-data".to_string()));
        assert!(result.skills.contains(&"backend".to_string()));
    }

    #[test]
    fn walk_tree_incomplete_answers_returns_error() {
        let tree = make_test_tree();
        let mut answers = HashMap::new();
        answers.insert(
            "app_name".to_string(),
            AnswerValue::Text("MyApp".to_string()),
        );
        // auth_required answered but auth_type and db_type not yet
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(true));

        let result = walk_tree(&tree, &answers);
        assert!(result.is_err());
    }
}
