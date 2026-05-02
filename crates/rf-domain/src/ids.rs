use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum IdError {
    #[error("{kind} must not be empty")]
    Empty { kind: &'static str },
}

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(IdError::Empty {
                        kind: stringify!($name),
                    });
                }
                Ok(Self(value))
            }

            pub fn generate() -> Self {
                Self(Uuid::new_v4().to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

define_id!(TenantId);
define_id!(ProjectId);
define_id!(ActorId);
define_id!(RoleId);
define_id!(SessionId);
define_id!(SkillId);
define_id!(SkillVersionId);
define_id!(SkillSessionId);
define_id!(CommandId);
define_id!(ApprovalId);
define_id!(SnapshotId);
define_id!(AuditEventId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn rejects_empty_ids() {
        assert_eq!(
            TenantId::new(" ").unwrap_err(),
            IdError::Empty { kind: "TenantId" }
        );
    }

    #[test]
    fn generates_unique_ids() {
        let mut ids = HashSet::new();
        for _ in 0..100 {
            ids.insert(TenantId::generate().to_string());
        }
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn display_matches_as_str() {
        let id = TenantId::new("test-tenant").unwrap();
        assert_eq!(id.to_string(), id.as_str());
    }

    #[test]
    fn from_str_and_new_agree() {
        let a = TenantId::new("my-id").unwrap();
        let b: TenantId = "my-id".parse().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn serde_roundtrip() {
        let original = TenantId::new("serde-test").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: TenantId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn ordering_is_consistent() {
        let a = TenantId::new("aaa").unwrap();
        let b = TenantId::new("bbb").unwrap();
        assert!(a < b);
    }
}
