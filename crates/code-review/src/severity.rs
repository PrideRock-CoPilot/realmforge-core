use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity levels for code review findings.
///
/// Ordered from least to most severe for derived `Ord`.
/// Rust's derived `Ord` follows declaration order: later variants are "greater".
/// Therefore `Note < ShouldFix < RequiredChange < Blocker`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational note only. No action required.
    #[serde(rename = "NOTE")]
    Note,
    /// Should be fixed, but not a blocker — may be deferred.
    #[serde(rename = "SHOULD_FIX")]
    ShouldFix,
    /// Should be changed before merge. Code review requires action.
    #[serde(rename = "REQUIRED_CHANGE")]
    RequiredChange,
    /// Blocks merge/deployment. Must be resolved before the review can pass.
    #[serde(rename = "BLOCKER")]
    Blocker,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocker => write!(f, "BLOCKER"),
            Self::RequiredChange => write!(f, "REQUIRED_CHANGE"),
            Self::ShouldFix => write!(f, "SHOULD_FIX"),
            Self::Note => write!(f, "NOTE"),
        }
    }
}

impl Severity {
    /// Returns `true` if this severity blocks a review from passing.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Blocker | Self::RequiredChange)
    }

    /// Returns a human-readable label for the severity tier.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Blocker => "Blocker",
            Self::RequiredChange => "Required Change",
            Self::ShouldFix => "Should Fix",
            Self::Note => "Note",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Blocker > Severity::RequiredChange);
        assert!(Severity::RequiredChange > Severity::ShouldFix);
        assert!(Severity::ShouldFix > Severity::Note);
    }

    #[test]
    fn test_is_blocking() {
        assert!(Severity::Blocker.is_blocking());
        assert!(Severity::RequiredChange.is_blocking());
        assert!(!Severity::ShouldFix.is_blocking());
        assert!(!Severity::Note.is_blocking());
    }

    #[test]
    fn test_deserialize_from_yaml() {
        let yaml = "BLOCKER";
        let s: Severity = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(s, Severity::Blocker);

        let yaml = "REQUIRED_CHANGE";
        let s: Severity = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(s, Severity::RequiredChange);
    }
}
