// ─────────────────────────────────────────────
// mapper.rs — Feature Mapper
// ─────────────────────────────────────────────
// Maps decision tree answers to module/persona/skill
// requirements using feature mappings and meta
// mappings defined in the tree.
// ─────────────────────────────────────────────

use crate::error::IntakeEngineError;
use crate::tree::{AnswerValue, FeatureMapping, MetaMapping, TreeDefinition};
use std::collections::HashMap;

/// Result of applying all mappings for a set of answers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MappingResult {
    /// Modules activated by direct feature mappings
    pub modules: Vec<String>,
    /// Personas required by direct feature mappings
    pub personas: Vec<String>,
    /// Skills required by direct feature mappings
    pub skills: Vec<String>,
    /// Modules activated by cross-cutting meta mappings
    pub meta_modules: Vec<String>,
    /// Personas required by cross-cutting meta mappings
    pub meta_personas: Vec<String>,
    /// Skills required by cross-cutting meta mappings
    pub meta_skills: Vec<String>,
}

/// Apply all feature mappings from a tree against collected answers.
pub fn apply_all_mappings(
    tree: &TreeDefinition,
    answers: &HashMap<String, AnswerValue>,
) -> Result<MappingResult, IntakeEngineError> {
    let mut result = MappingResult {
        modules: Vec::new(),
        personas: Vec::new(),
        skills: Vec::new(),
        meta_modules: Vec::new(),
        meta_personas: Vec::new(),
        meta_skills: Vec::new(),
    };

    // Apply feature mappings
    for mapping in &tree.feature_mappings {
        let mapped = apply_single_mapping(mapping, answers)?;
        result.modules.extend(mapped.modules);
        result.personas.extend(mapped.personas);
        result.skills.extend(mapped.skills);
    }

    // Apply meta mappings
    for meta in tree.meta_mappings.values() {
        let mapped = apply_meta_mapping(meta, answers)?;
        result.meta_modules.extend(mapped.modules);
        result.meta_personas.extend(mapped.personas);
        result.meta_skills.extend(mapped.skills);
    }

    // Deduplicate
    result.modules = deduplicate(result.modules);
    result.personas = deduplicate(result.personas);
    result.skills = deduplicate(result.skills);
    result.meta_modules = deduplicate(result.meta_modules);
    result.meta_personas = deduplicate(result.meta_personas);
    result.meta_skills = deduplicate(result.meta_skills);

    Ok(result)
}

/// Apply a single feature mapping.
fn apply_single_mapping(
    mapping: &FeatureMapping,
    answers: &HashMap<String, AnswerValue>,
) -> Result<MappingResult, IntakeEngineError> {
    let answer = answers
        .get(&mapping.question_id)
        .ok_or_else(|| IntakeEngineError::MappingConditionUnknown(mapping.question_id.clone()))?;

    let answer_json = answer_to_json(answer);
    if answer_json == mapping.value {
        Ok(MappingResult {
            modules: mapping.modules.clone(),
            personas: mapping.personas.clone(),
            skills: mapping.skills.clone(),
            meta_modules: Vec::new(),
            meta_personas: Vec::new(),
            meta_skills: Vec::new(),
        })
    } else {
        Ok(MappingResult::default())
    }
}

/// Apply a meta mapping (cross-cutting).
fn apply_meta_mapping(
    meta: &MetaMapping,
    answers: &HashMap<String, AnswerValue>,
) -> Result<MappingResult, IntakeEngineError> {
    let satisfied = crate::condition::evaluate_condition(&meta.condition, answers)
        .map_err(|e| IntakeEngineError::MetaMappingError(e.to_string()))?;

    if satisfied {
        Ok(MappingResult {
            modules: meta.modules.clone(),
            personas: meta.personas.clone(),
            skills: meta.skills.clone(),
            meta_modules: Vec::new(),
            meta_personas: Vec::new(),
            meta_skills: Vec::new(),
        })
    } else {
        Ok(MappingResult::default())
    }
}

/// Convert AnswerValue to serde_json::Value for comparison.
fn answer_to_json(answer: &AnswerValue) -> serde_json::Value {
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

/// Deduplicate and preserve order.
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
    use crate::tree::FeatureMapping;

    #[test]
    fn single_mapping_matches() {
        let mut answers = HashMap::new();
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(true));

        let mapping = FeatureMapping {
            question_id: "auth_required".to_string(),
            value: serde_json::Value::Bool(true),
            modules: vec!["auth-module".to_string()],
            personas: vec!["backend-auth".to_string()],
            skills: vec!["backend".to_string()],
        };

        let result = apply_single_mapping(&mapping, &answers).unwrap();
        assert_eq!(result.modules, vec!["auth-module"]);
        assert_eq!(result.personas, vec!["backend-auth"]);
        assert_eq!(result.skills, vec!["backend"]);
    }

    #[test]
    fn single_mapping_no_match() {
        let mut answers = HashMap::new();
        answers.insert("auth_required".to_string(), AnswerValue::Boolean(false));

        let mapping = FeatureMapping {
            question_id: "auth_required".to_string(),
            value: serde_json::Value::Bool(true),
            modules: vec!["auth-module".to_string()],
            personas: vec![],
            skills: vec![],
        };

        let result = apply_single_mapping(&mapping, &answers).unwrap();
        assert!(result.modules.is_empty());
    }

    #[test]
    fn mapping_unknown_question_returns_error() {
        let answers = HashMap::new();
        let mapping = FeatureMapping {
            question_id: "nonexistent".to_string(),
            value: serde_json::Value::Bool(true),
            modules: vec![],
            personas: vec![],
            skills: vec![],
        };

        let result = apply_single_mapping(&mapping, &answers);
        assert!(result.is_err());
    }

    #[test]
    fn all_mappings_deduplicate() {
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), AnswerValue::Boolean(true));
        answers.insert(
            "q2".to_string(),
            AnswerValue::SingleChoice("yes".to_string()),
        );

        let tree = TreeDefinition {
            schema: String::new(),
            tree_id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            applies_to: vec!["app".to_string()],
            description: String::new(),
            max_depth: 5,
            questions: vec![],
            feature_mappings: vec![
                FeatureMapping {
                    question_id: "q1".to_string(),
                    value: serde_json::Value::Bool(true),
                    modules: vec!["module-a".to_string(), "module-b".to_string()],
                    personas: vec!["persona-x".to_string()],
                    skills: vec!["skill-z".to_string()],
                },
                FeatureMapping {
                    question_id: "q2".to_string(),
                    value: serde_json::Value::String("yes".to_string()),
                    modules: vec!["module-a".to_string()], // duplicate
                    personas: vec!["persona-y".to_string()],
                    skills: vec!["skill-z".to_string()], // duplicate
                },
            ],
            meta_mappings: HashMap::new(),
            plan_stage_handoff: Default::default(),
        };

        let result = apply_all_mappings(&tree, &answers).unwrap();
        assert_eq!(result.modules, vec!["module-a", "module-b"]);
        assert!(result.personas.contains(&"persona-x".to_string()));
        assert!(result.personas.contains(&"persona-y".to_string()));
        assert_eq!(result.skills, vec!["skill-z"]);
    }
}
