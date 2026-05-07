use thiserror::Error;

/// Typed error variants for Design Council validation and operations.
///
/// Every error carries context about what failed and, where applicable,
/// what the expected value was. This enables callers to provide actionable
/// feedback rather than generic "validation failed" messages.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DesignCouncilError {
    /// Fewer research examples provided than the minimum required.
    #[error("insufficient examples: provided {count}, minimum {minimum}")]
    InsufficientExamples {
        /// Number of examples provided.
        count: usize,
        /// Minimum number of examples required.
        minimum: usize,
    },

    /// Fewer failure examples provided than the minimum required.
    #[error("insufficient failure examples: provided {failure_count}, minimum {minimum}")]
    InsufficientFailures {
        /// Number of failure-verdict examples provided.
        failure_count: usize,
        /// Minimum number of failure examples required.
        minimum: usize,
    },

    /// The problem statement was empty.
    #[error("problem statement is empty")]
    EmptyProblemStatement,

    /// The problem statement exceeds the maximum allowed length.
    #[error("problem statement too long: {length} characters, maximum {max}")]
    ProblemStatementTooLong {
        /// Actual length of the statement.
        length: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Pattern extraction was provided but contained no patterns.
    #[error("no patterns provided in pattern extraction")]
    NoPatternsProvided,

    /// A recommendation was provided without supporting evidence references.
    #[error("recommendation '{recommendation}' has no supporting evidence")]
    RecommendationWithoutEvidence {
        /// The name of the recommendation missing evidence.
        recommendation: String,
    },

    /// Risk surface was provided but contained no assumptions.
    #[error("risk surface contains no assumptions")]
    RiskSurfaceWithoutAssumptions,

    /// An invalid state transition was attempted.
    #[error("invalid status transition: from {from:?} to {to:?}")]
    InvalidStatusTransition {
        /// The current status.
        from: crate::state::DesignBriefStatus,
        /// The attempted new status.
        to: crate::state::DesignBriefStatus,
    },

    /// Two research examples have the same system name (duplicate).
    #[error("duplicate example system name: '{system_name}'")]
    DuplicateExampleName {
        /// The duplicated system name.
        system_name: String,
    },

    /// The recommendations list was provided but empty.
    #[error("recommendations list is empty")]
    EmptyRecommendations,
}
