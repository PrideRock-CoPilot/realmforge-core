use crate::ActorId;
use serde::{Deserialize, Serialize};

/// Separation of duties configuration — defines which grant groups conflict.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeparationOfDuties {
    /// Pairs of grant groups that cannot be held by the same actor simultaneously.
    pub conflicting_groups: Vec<(String, String)>,
}

impl SeparationOfDuties {
    /// Validate that an actor holding `current_grant_groups` can also hold
    /// `requested_group` without violating any separation constraint.
    pub fn validate(
        &self,
        actor_id: &ActorId,
        requested_group: &str,
        current_grant_groups: &[String],
    ) -> Result<(), SeparationViolation> {
        for group in current_grant_groups {
            // Check if requested_group conflicts with any held group
            for (left, right) in &self.conflicting_groups {
                if (left == requested_group && right == group)
                    || (right == requested_group && left == group)
                {
                    return Err(SeparationViolation {
                        actor_id: actor_id.clone(),
                        held_group: group.clone(),
                        requested_group: requested_group.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Returned when a separation of duties conflict is detected.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeparationViolation {
    pub actor_id: ActorId,
    pub held_group: String,
    pub requested_group: String,
}

impl std::fmt::Display for SeparationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "actor {} holds group '{}' which conflicts with requested group '{}'",
            self.actor_id, self.held_group, self.requested_group
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sod() -> SeparationOfDuties {
        SeparationOfDuties {
            conflicting_groups: vec![
                ("implementer".to_string(), "approver".to_string()),
                ("developer".to_string(), "qa".to_string()),
            ],
        }
    }

    #[test]
    fn allows_non_conflicting_group() {
        let sod = make_sod();
        let actor = ActorId::new("alice").unwrap();
        let current = vec!["developer".to_string()];
        assert!(sod.validate(&actor, "observer", &current).is_ok());
    }

    #[test]
    fn denies_conflicting_group() {
        let sod = make_sod();
        let actor = ActorId::new("bob").unwrap();
        let current = vec!["implementer".to_string()];
        let err = sod.validate(&actor, "approver", &current).unwrap_err();
        assert_eq!(err.actor_id, actor);
        assert_eq!(err.held_group, "implementer");
        assert_eq!(err.requested_group, "approver");
    }

    #[test]
    fn allows_same_group_no_conflict() {
        let sod = make_sod();
        let actor = ActorId::new("carol").unwrap();
        let current = vec!["approver".to_string()];
        // approver only conflicts if implementer is held (not vice versa again)
        // the pair is (implementer, approver) — both directions checked
        assert!(sod.validate(&actor, "approver", &current).is_ok());
    }
}
