use chrono::{DateTime, Utc};
use rf_domain::{ActorId, AuditEventId, ProjectId, TenantId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("failed to serialize audit event for hashing")]
    Serialize(#[from] serde_json::Error),
}

/// Categorization of audit event types for querying and filtering.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EventCategory {
    Session,
    Command,
    Policy,
    Snapshot,
    Rollback,
    WorkPacket,
    System,
}

impl EventCategory {
    pub fn from_event_type(event_type: &str) -> Self {
        if event_type.starts_with("session.") {
            Self::Session
        } else if event_type.starts_with("command.") || event_type.starts_with("cmd.") {
            Self::Command
        } else if event_type.starts_with("policy.") {
            Self::Policy
        } else if event_type.starts_with("snapshot.") {
            Self::Snapshot
        } else if event_type.starts_with("rollback.") {
            Self::Rollback
        } else if event_type.starts_with("work_packet.") {
            Self::WorkPacket
        } else {
            Self::System
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
    pub previous_hash: Option<String>,
    pub event_hash: String,
}

#[derive(Serialize)]
struct AuditEventHashInput<'a> {
    id: &'a AuditEventId,
    tenant_id: &'a TenantId,
    project_id: &'a ProjectId,
    actor_id: &'a ActorId,
    event_type: &'a str,
    entity_type: &'a str,
    entity_id: &'a str,
    payload: &'a Value,
    occurred_at: DateTime<Utc>,
    previous_hash: &'a Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainAnchor {
    pub first_event_id: AuditEventId,
    pub last_event_id: AuditEventId,
    pub event_count: u64,
    pub chain_integrity: bool,
}

impl AuditEvent {
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        actor_id: ActorId,
        event_type: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        payload: Value,
        previous_hash: Option<String>,
    ) -> Result<Self, EventError> {
        let mut event = Self {
            id: AuditEventId::generate(),
            tenant_id,
            project_id,
            actor_id,
            event_type: event_type.into(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            payload,
            occurred_at: Utc::now(),
            previous_hash,
            event_hash: String::new(),
        };
        event.event_hash = event.compute_hash()?;
        Ok(event)
    }

    pub fn compute_hash(&self) -> Result<String, EventError> {
        let input = AuditEventHashInput {
            id: &self.id,
            tenant_id: &self.tenant_id,
            project_id: &self.project_id,
            actor_id: &self.actor_id,
            event_type: &self.event_type,
            entity_type: &self.entity_type,
            entity_id: &self.entity_id,
            payload: &self.payload,
            occurred_at: self.occurred_at,
            previous_hash: &self.previous_hash,
        };
        let bytes = serde_json::to_vec(&input)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    pub fn verify_hash(&self) -> Result<bool, EventError> {
        Ok(self.compute_hash()? == self.event_hash)
    }

    /// Category of this event.
    pub fn category(&self) -> EventCategory {
        EventCategory::from_event_type(&self.event_type)
    }
}

/// Verify a list of audit events in sequence — each event must chain to the next.
pub fn verify_event_chain(events: &[AuditEvent]) -> Result<bool, EventError> {
    if events.is_empty() {
        return Ok(true);
    }

    for (i, event) in events.iter().enumerate() {
        // Verify individual event hash
        if !event.verify_hash()? {
            return Ok(false);
        }
        // Verify chain linkage
        if let Some(prev_hash) = &event.previous_hash {
            if i == 0 {
                return Ok(false); // first event should have no previous hash
            }
            if events[i - 1].event_hash != *prev_hash {
                return Ok(false);
            }
        } else if i > 0 {
            return Ok(false); // non-first event must have previous hash
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_hash_detects_payload_change() {
        let mut event = AuditEvent::new(
            TenantId::new("tenant").unwrap(),
            ProjectId::new("project").unwrap(),
            ActorId::new("actor").unwrap(),
            "command.authorized",
            "command",
            "cmd-1",
            json!({"action": "core.create_snapshot"}),
            None,
        )
        .unwrap();

        assert!(event.verify_hash().unwrap());
        event.payload = json!({"action": "tampered"});
        assert!(!event.verify_hash().unwrap());
    }

    #[test]
    fn verify_chain_passes_for_linked_events() {
        let t = TenantId::new("t").unwrap();
        let p = ProjectId::new("p").unwrap();
        let a = ActorId::new("a").unwrap();

        let event1 = AuditEvent::new(
            t.clone(), p.clone(), a.clone(),
            "session.issued", "session", "sess-1",
            json!({"type": "session"}), None,
        )
        .unwrap();

        let event2 = AuditEvent::new(
            t.clone(), p.clone(), a.clone(),
            "command.proposed", "command", "cmd-1",
            json!({"action": "test"}),
            Some(event1.event_hash.clone()),
        )
        .unwrap();

        let event3 = AuditEvent::new(
            t, p, a,
            "command.authorized", "command", "cmd-1",
            json!({"action": "test"}),
            Some(event2.event_hash.clone()),
        )
        .unwrap();

        assert!(verify_event_chain(&[event1, event2, event3]).unwrap());
    }

    #[test]
    fn verify_chain_fails_when_tampered() {
        let t = TenantId::new("t").unwrap();
        let p = ProjectId::new("p").unwrap();
        let a = ActorId::new("a").unwrap();

        let mut event1 = AuditEvent::new(
            t.clone(), p.clone(), a.clone(),
            "cmd.proposed", "cmd", "cmd-1",
            json!({"action": "test"}), None,
        )
        .unwrap();

        let event2 = AuditEvent::new(
            t, p, a,
            "cmd.authorized", "cmd", "cmd-1",
            json!({"action": "test"}),
            Some(event1.event_hash.clone()),
        )
        .unwrap();

        // Tamper with event1
        event1.payload = json!({"action": "tampered"});
        assert!(!verify_event_chain(&[event1, event2]).unwrap());
    }

    #[test]
    fn event_category_is_correct() {
        assert_eq!(
            EventCategory::from_event_type("session.issued"),
            EventCategory::Session
        );
        assert_eq!(
            EventCategory::from_event_type("command.proposed"),
            EventCategory::Command
        );
        assert_eq!(
            EventCategory::from_event_type("policy.denied"),
            EventCategory::Policy
        );
        assert_eq!(
            EventCategory::from_event_type("snapshot.created"),
            EventCategory::Snapshot
        );
        assert_eq!(
            EventCategory::from_event_type("rollback.executed"),
            EventCategory::Rollback
        );
        assert_eq!(
            EventCategory::from_event_type("unknown.event"),
            EventCategory::System
        );
    }
}
