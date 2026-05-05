use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DenialCode {
    ActorUnauthorized,
    SkillNotUsable,
    WrongSkillActive,
    ApprovalRequired,
    ProposalOnly,
    SessionStale,
    SessionExpired,
    /// The command has already reached the `Applied` terminal state.
    CommandAlreadyApplied,
    /// The command has already been denied and cannot be re-authorized.
    CommandAlreadyDenied,
    /// The command is already `Authorized` and cannot be authorized again.
    CommandAlreadyAuthorized,
    RateLimited,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectiveAction {
    RefreshSession,
    RebindSkill,
    RequestApproval,
    SwitchExecutionMode,
    VerifyCommandStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDenial {
    pub code: DenialCode,
    pub message: String,
    pub retryable: bool,
    pub corrective_action: Option<CorrectiveAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub denial: Option<PolicyDenial>,
}

impl PolicyDecision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            denial: None,
        }
    }

    pub fn deny(
        code: DenialCode,
        message: impl Into<String>,
        retryable: bool,
        corrective_action: Option<CorrectiveAction>,
    ) -> Self {
        Self {
            allowed: false,
            denial: Some(PolicyDenial {
                code,
                message: message.into(),
                retryable,
                corrective_action,
            }),
        }
    }
}

impl std::fmt::Display for PolicyDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.allowed {
            write!(f, "allowed")
        } else if let Some(d) = &self.denial {
            write!(f, "denied({:?}): {}", d.code, d.message)
        } else {
            write!(f, "denied")
        }
    }
}
