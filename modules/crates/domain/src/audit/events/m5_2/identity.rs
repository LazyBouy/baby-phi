//! Audit-event builders for the Identity tier (CH-16 / ADR-0038).
//!
//! Two events:
//!
//! - `platform.identity.created` — Alerted class. Emitted by
//!   `apply_agent_creation` immediately after the Identity row commits
//!   for an LLM agent. Pairs 1:1 with the `platform.agent.created`
//!   event that precedes it in the same compound tx.
//! - `platform.identity.updated` — Routine class. CH-16 ships the
//!   helper; CH-21 calls it once the memory-extraction listener body
//!   lands. Future writers (skill-change, rating-received in M6+) plug
//!   into the same builder.
//!
//! Concept doc cross-ref: `concepts/agent.md` § "Identity (Emergent,
//! Event-Driven)" + § "Materialization" — Identity is incrementally
//! updated by triggering events; the audit chain mirrors that.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::events::IdentityUpdateTrigger;
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, OrgId};
use crate::model::nodes::Identity;

/// `platform.identity.created` — Alerted class.
///
/// Emitted post-commit when `apply_agent_creation` writes a new Identity
/// row for an LLM agent. The diff names the agent + the four content
/// fields' summary shape (length-only for strings; counts for nested
/// vec fields) so ops dashboards can join against `platform.agent.created`
/// without serialising the full Identity payload.
pub fn identity_created(
    actor: AgentId,
    identity: &Identity,
    org: OrgId,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "identity_id": identity.id.to_string(),
            "agent_id": identity.agent_id.to_string(),
            "self_description_len": identity.self_description.len(),
            "lived_sessions_completed": identity.lived.sessions_completed,
            "witnessed_memories_extracted": identity.witnessed.memories_extracted,
            "embedding_dim": identity.embedding.len(),
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.identity.created".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(identity.id),
        timestamp,
        diff,
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `platform.identity.updated` — Logged class (mid-tier retention).
///
/// Emitted by future reactive update writers (CH-21 memory-extraction
/// listener body; M6+ skill-change / rating-received events). The diff
/// summarises before/after for the four content fields plus the trigger
/// kind so log readers can attribute the update without diffing the
/// full Identity payload.
pub fn identity_updated(
    actor: AgentId,
    before: &Identity,
    after: &Identity,
    trigger: IdentityUpdateTrigger,
    org: OrgId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let trigger_label = match trigger {
        IdentityUpdateTrigger::SessionEnded => "session_ended",
        IdentityUpdateTrigger::MemoryExtracted => "memory_extracted",
        IdentityUpdateTrigger::SkillChanged => "skill_changed",
        IdentityUpdateTrigger::RatingReceived => "rating_received",
    };
    let diff = serde_json::json!({
        "trigger": trigger_label,
        "before": {
            "self_description_len": before.self_description.len(),
            "lived_sessions_completed": before.lived.sessions_completed,
            "witnessed_memories_extracted": before.witnessed.memories_extracted,
            "embedding_dim": before.embedding.len(),
        },
        "after": {
            "self_description_len": after.self_description.len(),
            "lived_sessions_completed": after.lived.sessions_completed,
            "witnessed_memories_extracted": after.witnessed.memories_extracted,
            "embedding_dim": after.embedding.len(),
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.identity.updated".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(after.id),
        timestamp,
        diff,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity() -> Identity {
        Identity::default_for_llm(AgentId::new(), Utc::now())
    }

    #[test]
    fn identity_created_event_is_alerted_class() {
        let iden = sample_identity();
        let evt = identity_created(AgentId::new(), &iden, OrgId::new(), None, Utc::now());
        assert_eq!(evt.event_type, "platform.identity.created");
        assert_eq!(evt.audit_class, AuditClass::Alerted);
        assert_eq!(evt.target_entity_id, Some(iden.id));
        // Diff "after" carries the structured agent_id + summary fields.
        let after = &evt.diff["after"];
        assert_eq!(
            after["agent_id"].as_str().unwrap(),
            iden.agent_id.to_string()
        );
        assert_eq!(after["self_description_len"].as_u64().unwrap(), 0);
        assert_eq!(after["embedding_dim"].as_u64().unwrap(), 0);
    }

    #[test]
    fn identity_updated_event_is_routine_class_with_trigger_label() {
        let before = sample_identity();
        let mut after = before.clone();
        after.self_description = "agent-authored".into();
        after.lived.sessions_completed = 3;
        let evt = identity_updated(
            AgentId::new(),
            &before,
            &after,
            IdentityUpdateTrigger::MemoryExtracted,
            OrgId::new(),
            Utc::now(),
        );
        assert_eq!(evt.event_type, "platform.identity.updated");
        assert_eq!(evt.audit_class, AuditClass::Logged);
        assert_eq!(evt.diff["trigger"].as_str().unwrap(), "memory_extracted");
        assert_eq!(
            evt.diff["before"]["self_description_len"].as_u64().unwrap(),
            0
        );
        assert_eq!(
            evt.diff["after"]["self_description_len"].as_u64().unwrap(),
            "agent-authored".len() as u64
        );
        assert_eq!(
            evt.diff["after"]["lived_sessions_completed"]
                .as_u64()
                .unwrap(),
            3
        );
    }

    #[test]
    fn identity_updated_serialises_all_four_trigger_variants() {
        let before = sample_identity();
        let after = sample_identity();
        for (variant, expected) in [
            (IdentityUpdateTrigger::SessionEnded, "session_ended"),
            (IdentityUpdateTrigger::MemoryExtracted, "memory_extracted"),
            (IdentityUpdateTrigger::SkillChanged, "skill_changed"),
            (IdentityUpdateTrigger::RatingReceived, "rating_received"),
        ] {
            let evt = identity_updated(
                AgentId::new(),
                &before,
                &after,
                variant,
                OrgId::new(),
                Utc::now(),
            );
            assert_eq!(evt.diff["trigger"].as_str().unwrap(), expected);
        }
    }
}
