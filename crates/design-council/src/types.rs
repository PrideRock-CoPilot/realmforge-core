use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::DesignBriefStatus;
use crate::DesignCouncilError;

// ---------------------------------------------------------------------------
// Typed IDs
// ---------------------------------------------------------------------------

/// A typed newtype identifying a unique Design Problem study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DesignProblemId(Uuid);

impl DesignProblemId {
    /// Create a new random `DesignProblemId`.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DesignProblemId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DesignProblemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Whether an example system's approach succeeded or failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExampleVerdict {
    /// The approach succeeded.
    Success,
    /// The approach failed.
    Failure,
    /// Mixed results — succeeded in some contexts, failed in others.
    Mixed,
}

/// The source type of a research example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchSourceType {
    /// An open-source project with documented architecture.
    OpenSourceProject,
    /// A published post-mortem or retrospective.
    PublishedPostMortem,
    /// A recognized industry pattern (e.g., from pattern catalogues).
    IndustryPattern,
    /// Peer-reviewed academic research.
    AcademicPaper,
    /// A past decision within RealmForge itself.
    PastRealmForgeDecision,
    /// An internal post-mortem from RealmForge or partner projects.
    InternalPostMortem,
}

/// Whether a design pattern is a success pattern, failure pattern, or context-dependent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// What successful approaches shared.
    Success,
    /// What failures shared.
    Failure,
    /// Worked only under specific conditions.
    Contextual,
}

/// The rank of a recommendation: primary or alternative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationRank {
    /// The primary recommendation (best approach).
    Primary,
    /// An alternative approach worth considering.
    Alternative,
}

/// The assessed risk level of a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk — well-understood approach with strong evidence.
    Low,
    /// Medium risk — reasonable evidence but non-trivial trade-offs.
    Medium,
    /// High risk — significant unknowns or costly failure modes.
    High,
}

/// The type of a blind spot identified during the risk surface phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlindSpotType {
    /// A system that faced this problem but was not studied.
    UnstudiedSystem,
    /// An implicit assumption in the recommendations.
    ImplicitAssumption,
    /// Evidence that would disprove the recommendation.
    DisprovingEvidence,
    /// The cost of being wrong about this recommendation.
    ReversalCost,
}

// ---------------------------------------------------------------------------
// Core value objects
// ---------------------------------------------------------------------------

/// The design problem being studied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignProblem {
    /// Unique identifier.
    pub id: DesignProblemId,
    /// Single-sentence problem statement.
    pub problem_statement: String,
    /// What subsystem or area this problem affects.
    pub system_boundary: String,
    /// Key constraints (performance, security, compatibility, etc.).
    pub constraints: Vec<String>,
    /// What "good" looks like — a concrete success signal.
    pub success_signal: String,
    /// Explicitly out of scope.
    pub non_goals: Vec<String>,
    /// When this problem was first framed.
    pub created_at: DateTime<Utc>,
}

/// A single research example — a system that faced this design problem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchExample {
    /// The name of the system that faced this problem.
    pub system_name: String,
    /// Context about the system (scale, domain, tech stack, etc.).
    pub context_description: String,
    /// What approach they took.
    pub approach_taken: String,
    /// Whether it worked or not.
    pub verdict: ExampleVerdict,
    /// Why it worked or didn't — the deciding factors.
    pub deciding_factors: Vec<String>,
    /// Relevant constraints at the time (scale, team, regulatory, etc.).
    pub relevant_constraints: Vec<String>,
    /// What we can learn from this example.
    pub key_takeaway: String,
    /// The type of source this came from.
    pub source_type: ResearchSourceType,
}

/// A design pattern extracted from the research examples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignPattern {
    /// The pattern statement.
    pub description: String,
    /// Whether this is a success pattern, failure pattern, or contextual.
    pub pattern_type: PatternType,
    /// References to the example system names that support this pattern.
    pub supporting_examples: Vec<String>,
    /// Optional notes about when this pattern applies or doesn't.
    pub context_notes: Option<String>,
}

/// The pattern extraction deliverable — synthesized patterns from examples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternExtraction {
    /// Patterns that successful approaches shared.
    pub success_patterns: Vec<DesignPattern>,
    /// Patterns that failed approaches shared.
    pub failure_patterns: Vec<DesignPattern>,
    /// Factors that were context-dependent (worked only in specific conditions).
    pub contextual_factors: Vec<DesignPattern>,
}

/// A design recommendation with evidence backing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignRecommendation {
    /// Short descriptive label for the approach.
    pub name: String,
    /// Detailed description of what this approach entails.
    pub description: String,
    /// Whether this is the primary or alternative recommendation.
    pub rank: RecommendationRank,
    /// References to supporting examples and patterns.
    pub supporting_evidence: Vec<String>,
    /// Overall risk level.
    pub risk_level: RiskLevel,
    /// Specific risks, referencing failure examples.
    pub risks: Vec<String>,
    /// Mitigations for the identified risks.
    pub mitigations: Vec<String>,
    /// If combining elements from multiple examples, notes on the combination.
    pub combination_notes: Option<String>,
}

/// A blind spot identified during the risk surface phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlindSpot {
    /// Description of what we didn't study or assumed.
    pub description: String,
    /// The type of blind spot.
    pub spot_type: BlindSpotType,
    /// What could go wrong because of this blind spot.
    pub potential_impact: String,
}

/// The risk surface deliverable — structured analysis of risks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskSurface {
    /// Systems that faced this problem but were not studied.
    pub unstudied_systems: Vec<String>,
    /// Implicit assumptions in the recommendations.
    pub assumptions: Vec<String>,
    /// Evidence that would disprove the recommendation.
    pub disproving_evidence: Vec<String>,
    /// Estimate of the cost of being wrong.
    pub reversal_cost_estimate: String,
    /// Specific blind spots identified.
    pub blind_spots: Vec<BlindSpot>,
}

// ---------------------------------------------------------------------------
// Aggregate root — DesignBrief
// ---------------------------------------------------------------------------

/// The aggregate root of a Design Council session.
///
/// A `DesignBrief` represents a complete design research session:
/// from problem framing through example gathering, pattern extraction,
/// recommendation synthesis, and risk surfacing.
///
/// # Invariants
///
/// - `research_examples` must have at least 6 entries when validated
/// - At least 2 research examples must have verdict `Failure` (unless documented gap)
/// - `design_problem.problem_statement` must be non-empty and ≤ 200 chars
/// - Status transitions must follow the valid state machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignBrief {
    /// Unique identifier for this design brief.
    pub id: DesignProblemId,
    /// Human-readable title for this brief.
    pub title: String,
    /// When the Design Council session took place.
    pub session_date: DateTime<Utc>,
    /// Participants — the domain experts convened.
    pub participants: Vec<String>,
    /// The design problem being studied.
    pub design_problem: DesignProblem,
    /// Research examples gathered.
    pub research_examples: Vec<ResearchExample>,
    /// Extracted patterns (optional until Phase 3 is complete).
    pub pattern_extraction: Option<PatternExtraction>,
    /// Design recommendations (optional until Phase 4 is complete).
    pub recommendations: Vec<DesignRecommendation>,
    /// Risk surface analysis (optional until Phase 5 is complete).
    pub risk_surface: Option<RiskSurface>,
    /// Reference to an ADR once a decision follows from this brief.
    pub adr_reference: Option<String>,
    /// Current lifecycle status.
    pub status: DesignBriefStatus,
}

impl DesignBrief {
    /// Attempt to transition the brief to a new status.
    ///
    /// Returns `Ok(())` on success, or an error if the transition is invalid.
    pub fn transition_to(&mut self, target: DesignBriefStatus) -> Result<(), DesignCouncilError> {
        let new_status = self.status.transition_to(target)?;
        self.status = new_status;
        Ok(())
    }

    /// Validate the Design Brief against all invariant rules.
    ///
    /// Returns `Ok(())` if all rules pass, or the first error found.
    ///
    /// # Validation rules
    ///
    /// 1. Problem statement must be non-empty and ≤ 200 characters
    /// 2. At least 6 research examples required
    /// 3. At least 2 research examples must be failures
    /// 4. No duplicate system names in examples
    /// 5. If `pattern_extraction` is present, must have at least one pattern
    /// 6. If `recommendations` is non-empty, each must have supporting evidence
    /// 7. If `risk_surface` is present, assumptions must be non-empty
    pub fn validate(&self) -> Result<(), DesignCouncilError> {
        // Rule 1: Problem statement
        let statement = self.design_problem.problem_statement.trim();
        if statement.is_empty() {
            return Err(DesignCouncilError::EmptyProblemStatement);
        }
        let max_statement_length: usize = 200;

        if statement.len() > max_statement_length {
            return Err(DesignCouncilError::ProblemStatementTooLong {
                length: statement.len(),
                max: max_statement_length,
            });

        }

        // Rule 2: Minimum examples
        let min_examples: usize = 6;
        if self.research_examples.len() < min_examples {
            return Err(DesignCouncilError::InsufficientExamples {
                count: self.research_examples.len(),
                minimum: min_examples,
            });

        }

        // Rule 3: Minimum failures
        let min_failures: usize = 2;
        let failure_count = self
            .research_examples
            .iter()
            .filter(|e| e.verdict == ExampleVerdict::Failure)
            .count();
        if failure_count < min_failures {
            return Err(DesignCouncilError::InsufficientFailures {
                failure_count,
                minimum: min_failures,
            });

        }

        // Rule 4: No duplicate system names
        let mut seen_names = std::collections::HashSet::new();
        for example in &self.research_examples {
            if !seen_names.insert(example.system_name.as_str()) {
                return Err(DesignCouncilError::DuplicateExampleName {
                    system_name: example.system_name.clone(),
                });
            }
        }

        // Rule 5: Pattern extraction must have at least one pattern
        if let Some(ref extraction) = self.pattern_extraction {
            let total = extraction.success_patterns.len()
                + extraction.failure_patterns.len()
                + extraction.contextual_factors.len();
            if total == 0 {
                return Err(DesignCouncilError::NoPatternsProvided);
            }
        }

        // Rule 6: Recommendations must have evidence
        if !self.recommendations.is_empty() {
            for rec in &self.recommendations {
                if rec.supporting_evidence.is_empty() {
                    return Err(DesignCouncilError::RecommendationWithoutEvidence {
                        recommendation: rec.name.clone(),
                    });
                }
            }
        }

        // Rule 7: Risk surface must have assumptions
        if let Some(ref surface) = self.risk_surface {
            if surface.assumptions.is_empty() {
                return Err(DesignCouncilError::RiskSurfaceWithoutAssumptions);
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn valid_problem() -> DesignProblem {
        DesignProblem {
            id: DesignProblemId::new(),
            problem_statement: "How should session tokens be managed for scalability?"
                .to_string(),
            system_boundary: "Authentication subsystem".to_string(),
            constraints: vec!["Must support 10k+ concurrent sessions".to_string()],
            success_signal: "Sessions scale linearly with no increase in auth latency".to_string(),
            non_goals: vec![
                "Multi-factor authentication".to_string(),
                "OAuth integration".to_string(),
            ]
            .into(),
            created_at: Utc::now(),
        }
    }

    fn example(name: &str, verdict: ExampleVerdict) -> ResearchExample {
        ResearchExample {
            system_name: name.to_string(),
            context_description: "A test system".to_string(),
            approach_taken: "Approach X".to_string(),
            verdict,
            deciding_factors: vec!["Factor 1".to_string()],
            relevant_constraints: vec!["Constraint 1".to_string()],
            key_takeaway: "Lesson learned".to_string(),
            source_type: ResearchSourceType::PublishedPostMortem,
        }
    }

    fn valid_examples() -> Vec<ResearchExample> {
        vec![
            example("System A", ExampleVerdict::Success),
            example("System B", ExampleVerdict::Success),
            example("System C", ExampleVerdict::Failure),
            example("System D", ExampleVerdict::Success),
            example("System E", ExampleVerdict::Failure),
            example("System F", ExampleVerdict::Mixed),
        ]
    }

    #[test]
    fn test_valid_brief_passes_validation() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Session Token Design".to_string(),
            session_date: Utc::now(),
            participants: vec!["backend".to_string(), "security-architect".to_string()],
            design_problem: valid_problem(),
            research_examples: valid_examples(),
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        assert!(brief.validate().is_ok());
    }

    #[test]
    fn test_empty_problem_statement_fails() {
        let mut problem = valid_problem();
        problem.problem_statement = "   ".to_string();

        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: problem,
            research_examples: valid_examples(),
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        assert_eq!(
            brief.validate().unwrap_err(),
            DesignCouncilError::EmptyProblemStatement
        );
    }

    #[test]
    fn test_too_long_problem_statement_fails() {
        let mut problem = valid_problem();
        problem.problem_statement = "A".repeat(201);

        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: problem,
            research_examples: valid_examples(),
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert!(matches!(err, DesignCouncilError::ProblemStatementTooLong { .. }));
    }

    #[test]
    fn test_insufficient_examples_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: vec![
                example("A", ExampleVerdict::Success),
                example("B", ExampleVerdict::Failure),
            ],
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert!(
            matches!(err, DesignCouncilError::InsufficientExamples { count: 2, .. }),
            "Expected InsufficientExamples with count=2, got {:?}",
            err
        );
    }

    #[test]
    fn test_insufficient_failures_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: vec![
                example("A", ExampleVerdict::Success),
                example("B", ExampleVerdict::Success),
                example("C", ExampleVerdict::Success),
                example("D", ExampleVerdict::Success),
                example("E", ExampleVerdict::Success),
                example("F", ExampleVerdict::Mixed),
            ],
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert!(
            matches!(err, DesignCouncilError::InsufficientFailures { failure_count: 0, .. })
        );
    }

    #[test]
    fn test_duplicate_example_names_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: vec![
                example("SameSystem", ExampleVerdict::Success),
                example("B", ExampleVerdict::Success),
                example("C", ExampleVerdict::Failure),
                example("D", ExampleVerdict::Success),
                example("E", ExampleVerdict::Failure),
                example("SameSystem", ExampleVerdict::Mixed),
            ],
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert!(
            matches!(err, DesignCouncilError::DuplicateExampleName { ref system_name } if system_name == "SameSystem")
        );
    }

    #[test]
    fn test_empty_pattern_extraction_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: valid_examples(),
            pattern_extraction: Some(PatternExtraction {
                success_patterns: vec![],
                failure_patterns: vec![],
                contextual_factors: vec![],
            }),
            recommendations: vec![],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert_eq!(err, DesignCouncilError::NoPatternsProvided);
    }

    #[test]
    fn test_recommendation_without_evidence_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: valid_examples(),
            pattern_extraction: None,
            recommendations: vec![DesignRecommendation {
                name: "Short-Lived JWTs".to_string(),
                description: "Use short-lived JWTs with refresh tokens.".to_string(),
                rank: RecommendationRank::Primary,
                supporting_evidence: vec![],
                risk_level: RiskLevel::Low,
                risks: vec![],
                mitigations: vec![],
                combination_notes: None,
            }],
            risk_surface: None,
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert!(
            matches!(err, DesignCouncilError::RecommendationWithoutEvidence { ref recommendation } if recommendation == "Short-Lived JWTs")
        );
    }

    #[test]
    fn test_risk_surface_without_assumptions_fails() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Test".to_string(),
            session_date: Utc::now(),
            participants: vec![],
            design_problem: valid_problem(),
            research_examples: valid_examples(),
            pattern_extraction: None,
            recommendations: vec![],
            risk_surface: Some(RiskSurface {
                unstudied_systems: vec![],
                assumptions: vec![],
                disproving_evidence: vec![],
                reversal_cost_estimate: "High".to_string(),
                blind_spots: vec![],
            }),
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        let err = brief.validate().unwrap_err();
        assert_eq!(err, DesignCouncilError::RiskSurfaceWithoutAssumptions);
    }

    #[test]
    fn test_fully_loaded_brief_validates() {
        let brief = DesignBrief {
            id: DesignProblemId::new(),
            title: "Session Token Architecture".to_string(),
            session_date: Utc::now(),
            participants: vec![
                "security-architect".to_string(),
                "backend".to_string(),
                "domain-architect".to_string(),
            ],
            design_problem: valid_problem(),
            research_examples: valid_examples(),
            pattern_extraction: Some(PatternExtraction {
                success_patterns: vec![DesignPattern {
                    description: "Short-lived tokens with refresh mechanisms".to_string(),
                    pattern_type: PatternType::Success,
                    supporting_examples: vec!["System A".to_string(), "System B".to_string()],
                    context_notes: None,
                }],
                failure_patterns: vec![DesignPattern {
                    description: "Long-lived tokens without revocation".to_string(),
                    pattern_type: PatternType::Failure,
                    supporting_examples: vec!["System C".to_string()],
                    context_notes: None,
                }],
                contextual_factors: vec![],
            }),
            recommendations: vec![DesignRecommendation {
                name: "Short-Lived JWTs with DB-backed Refresh".to_string(),
                description: "Issue 5-minute access tokens with 24-hour refresh tokens.".to_string(),
                rank: RecommendationRank::Primary,
                supporting_evidence: vec![
                    "Success pattern: short-lived tokens".to_string(),
                    "System A approach".to_string(),
                    "System B approach".to_string(),
                ],
                risk_level: RiskLevel::Low,
                risks: vec!["Refresh token theft".to_string()],
                mitigations: vec!["Rotate refresh tokens on each use".to_string()],
                combination_notes: Some(
                    "Combines System A's token rotation with System B's DB-backed validation."
                        .to_string(),
                ),
            }],
            risk_surface: Some(RiskSurface {
                unstudied_systems: vec!["Google's internal session system".to_string()],
                assumptions: vec!["Token validation latency <5ms at the DB layer".to_string()],
                disproving_evidence: vec![
                    "If refresh requires DB lookup at scale, latency could degrade".to_string(),
                ],
                reversal_cost_estimate: "Medium — 2 weeks to migrate token formats".to_string(),
                blind_spots: vec![BlindSpot {
                    description: "We assume all clients support JWT libraries".to_string(),
                    spot_type: BlindSpotType::ImplicitAssumption,
                    potential_impact: "IoT or constrained clients may lack JWT support.".to_string(),
                }],
            }),
            adr_reference: None,
            status: DesignBriefStatus::Draft,
        };

        assert!(brief.validate().is_ok());
        assert!(brief.status.is_editable());
    }

    #[test]
    fn test_design_problem_id_display() {
        let id = DesignProblemId::new();
        let display = format!("{}", id);
        let uuid_len = "550e8400-e29b-41d4-a716-446655440000".len();
        assert_eq!(display.len(), uuid_len);
    }

    #[test]
    fn test_design_problem_id_default() {
        let id1 = DesignProblemId::default();
        let id2 = DesignProblemId::default();
        assert_ne!(id1, id2); // Random generation — should differ
    }
}
