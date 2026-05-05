use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    NoAccess,
    ReadOnly,
    Proposal,
    ReadWrite,
    Approved,
}

impl ExecutionMode {
    pub fn may_mutate(&self) -> bool {
        matches!(self, Self::ReadWrite | Self::Approved)
    }
}

impl TryFrom<&str> for ExecutionMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "no_access" => Ok(Self::NoAccess),
            "read_only" => Ok(Self::ReadOnly),
            "proposal" => Ok(Self::Proposal),
            "read_write" => Ok(Self::ReadWrite),
            "approved" => Ok(Self::Approved),
            other => Err(format!("unknown execution mode: {other}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    NotRequired,
    Required,
    Approved,
    Rejected,
    Expired,
}

impl ApprovalState {
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

impl TryFrom<&str> for ApprovalState {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_required" | "notRequired" => Ok(Self::NotRequired),
            "required" => Ok(Self::Required),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "expired" => Ok(Self::Expired),
            other => Err(format!("unknown approval state: {other}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillSessionState {
    Issued,
    Active,
    Expired,
    Revoked,
    Stopped,
}

impl SkillSessionState {
    pub fn can_reactivate(&self) -> bool {
        matches!(self, Self::Issued | Self::Active | Self::Expired)
    }
}

impl TryFrom<&str> for SkillSessionState {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "issued" => Ok(Self::Issued),
            "active" => Ok(Self::Active),
            "expired" => Ok(Self::Expired),
            "revoked" => Ok(Self::Revoked),
            "stopped" => Ok(Self::Stopped),
            other => Err(format!("unknown skill session state: {other}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Proposed,
    Authorized,
    Denied,
    Applied,
    Failed,
}

impl CommandStatus {
    /// Return the DB string representation.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Authorized => "authorized",
            Self::Denied => "denied",
            Self::Applied => "applied",
            Self::Failed => "failed",
        }
    }

    /// Check whether a transition from `self` to `target` is valid.
    ///
    /// Valid transitions:
    /// - Proposed → Authorized, Denied
    /// - Authorized → Applied, Failed, Denied
    /// - Applied, Denied, Failed → terminal (no valid transitions)
    pub fn can_transition_to(&self, target: &CommandStatus) -> bool {
        matches!(
            (self, target),
            (CommandStatus::Proposed, CommandStatus::Authorized)
                | (CommandStatus::Proposed, CommandStatus::Denied)
                | (CommandStatus::Authorized, CommandStatus::Applied)
                | (CommandStatus::Authorized, CommandStatus::Failed)
                | (CommandStatus::Authorized, CommandStatus::Denied)
        )
    }

    /// Returns `true` if this status is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Applied | Self::Denied | Self::Failed)
    }
}

impl std::fmt::Display for CommandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_db_str())
    }
}

impl TryFrom<&str> for CommandStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "authorized" => Ok(Self::Authorized),
            "denied" => Ok(Self::Denied),
            "applied" => Ok(Self::Applied),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown command status: {other}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Draft,
    KnownGood,
    Invalid,
    RollbackPreviewed,
}

impl SnapshotStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::KnownGood => "known_good",
            Self::Invalid => "invalid",
            Self::RollbackPreviewed => "rollback_previewed",
        }
    }
}

impl TryFrom<&str> for SnapshotStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "draft" => Ok(Self::Draft),
            "known_good" => Ok(Self::KnownGood),
            "invalid" => Ok(Self::Invalid),
            "rollback_previewed" => Ok(Self::RollbackPreviewed),
            other => Err(format!("unknown snapshot status: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_mode_try_from_str() {
        assert_eq!(
            ExecutionMode::try_from("approved").unwrap(),
            ExecutionMode::Approved
        );
        assert!(ExecutionMode::try_from("unknown").is_err());
    }

    #[test]
    fn approval_state_try_from_str() {
        assert_eq!(
            ApprovalState::try_from("approved").unwrap(),
            ApprovalState::Approved
        );
        assert!(ApprovalState::try_from("bogus").is_err());
    }

    #[test]
    fn command_status_try_from_str() {
        assert_eq!(
            CommandStatus::try_from("applied").unwrap(),
            CommandStatus::Applied
        );
        assert!(CommandStatus::try_from("nope").is_err());
    }

    #[test]
    fn snapshot_status_try_from_str() {
        assert_eq!(
            SnapshotStatus::try_from("known_good").unwrap(),
            SnapshotStatus::KnownGood
        );
        assert!(SnapshotStatus::try_from("nope").is_err());
    }

    #[test]
    fn command_status_db_str_roundtrip() {
        for status in &[
            CommandStatus::Proposed,
            CommandStatus::Authorized,
            CommandStatus::Denied,
            CommandStatus::Applied,
            CommandStatus::Failed,
        ] {
            let db = status.as_db_str();
            let back = CommandStatus::try_from(db).unwrap();
            assert_eq!(status, &back);
        }
    }
}
