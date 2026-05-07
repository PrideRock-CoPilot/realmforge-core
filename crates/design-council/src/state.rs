use serde::{Deserialize, Serialize};

use crate::DesignCouncilError;

/// The lifecycle status of a Design Brief.
///
/// Valid transitions are enforced by `DesignBriefStatus::transition_to()`:
///
/// ```text
/// Draft ───→ Recommendation ───→ Accepted
///                │                    │
///                │                    ├──→ Superseded
///                │                    └──→ Archived
///                └──→ Archived
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesignBriefStatus {
    /// Being researched. Examples are being gathered and analyzed.
    Draft,

    /// Research phase complete. A Design Brief with recommendations has been produced.
    Recommendation,

    /// A decision has been made to adopt the recommendations.
    Accepted,

    /// Newer research has replaced this brief.
    Superseded,

    /// Retained for historical reference. No longer active.
    Archived,
}

impl DesignBriefStatus {
    /// Attempt to transition from this status to `target`.
    ///
    /// Returns `Ok(DesignBriefStatus)` with the new status on success,
    /// or `Err(DesignCouncilError::InvalidStatusTransition)` if the
    /// transition is not allowed by the state machine.
    ///
    /// # Valid transitions
    ///
    /// | From | To | Meaning |
    /// |------|----|---------|
    /// | Draft | Recommendation | Research complete, brief produced |
    /// | Recommendation | Accepted | Decision made to adopt |
    /// | Recommendation | Archived | Decision not to adopt; retained for reference |
    /// | Accepted | Superseded | New research replaced this |
    /// | Accepted | Archived | No longer relevant but retained |
    ///
    /// # Invalid transitions (return error)
    ///
    /// - Draft → Accepted (must go through Recommendation)
    /// - Recommendation → Draft (cannot reopen closed research)
    /// - Accepted → Recommendation (cannot revise after decision)
    /// - Any transition not listed above
    pub fn transition_to(self, target: DesignBriefStatus) -> Result<DesignBriefStatus, DesignCouncilError> {
        use DesignBriefStatus::*;

        match (self, target) {
            // Valid transitions
            (Draft, Recommendation) => Ok(Recommendation),
            (Recommendation, Accepted) => Ok(Accepted),
            (Recommendation, Archived) => Ok(Archived),
            (Accepted, Superseded) => Ok(Superseded),
            (Accepted, Archived) => Ok(Archived),

            // Invalid transitions
            _ => Err(DesignCouncilError::InvalidStatusTransition {
                from: self,
                to: target,
            }),
        }
    }

    /// Returns `true` if this status allows editing the brief's content.
    ///
    /// Only `Draft` briefs can be edited. Once a brief reaches
    /// `Recommendation` status, it is considered published.
    pub fn is_editable(&self) -> bool {
        matches!(self, DesignBriefStatus::Draft)
    }

    /// Returns `true` if this status represents an active, in-use brief.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            DesignBriefStatus::Draft | DesignBriefStatus::Recommendation | DesignBriefStatus::Accepted
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_draft_to_recommendation() {
        let result = DesignBriefStatus::Draft.transition_to(DesignBriefStatus::Recommendation);
        assert_eq!(result, Ok(DesignBriefStatus::Recommendation));
    }

    #[test]
    fn test_valid_recommendation_to_accepted() {
        let result = DesignBriefStatus::Recommendation.transition_to(DesignBriefStatus::Accepted);
        assert_eq!(result, Ok(DesignBriefStatus::Accepted));
    }

    #[test]
    fn test_valid_recommendation_to_archived() {
        let result = DesignBriefStatus::Recommendation.transition_to(DesignBriefStatus::Archived);
        assert_eq!(result, Ok(DesignBriefStatus::Archived));
    }

    #[test]
    fn test_valid_accepted_to_superseded() {
        let result = DesignBriefStatus::Accepted.transition_to(DesignBriefStatus::Superseded);
        assert_eq!(result, Ok(DesignBriefStatus::Superseded));
    }

    #[test]
    fn test_valid_accepted_to_archived() {
        let result = DesignBriefStatus::Accepted.transition_to(DesignBriefStatus::Archived);
        assert_eq!(result, Ok(DesignBriefStatus::Archived));
    }

    #[test]
    fn test_invalid_draft_to_accepted() {
        let result = DesignBriefStatus::Draft.transition_to(DesignBriefStatus::Accepted);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid status transition: from Draft to Accepted"
        );
    }

    #[test]
    fn test_invalid_recommendation_to_draft() {
        let result = DesignBriefStatus::Recommendation.transition_to(DesignBriefStatus::Draft);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_accepted_to_recommendation() {
        let result = DesignBriefStatus::Accepted.transition_to(DesignBriefStatus::Recommendation);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_archived_to_anything() {
        let result = DesignBriefStatus::Archived.transition_to(DesignBriefStatus::Draft);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_editable() {
        assert!(DesignBriefStatus::Draft.is_editable());
        assert!(!DesignBriefStatus::Recommendation.is_editable());
        assert!(!DesignBriefStatus::Accepted.is_editable());
        assert!(!DesignBriefStatus::Superseded.is_editable());
        assert!(!DesignBriefStatus::Archived.is_editable());
    }

    #[test]
    fn test_is_active() {
        assert!(DesignBriefStatus::Draft.is_active());
        assert!(DesignBriefStatus::Recommendation.is_active());
        assert!(DesignBriefStatus::Accepted.is_active());
        assert!(!DesignBriefStatus::Superseded.is_active());
        assert!(!DesignBriefStatus::Archived.is_active());
    }
}
