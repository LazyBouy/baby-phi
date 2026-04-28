//! Audit-event builders for the Memory tier (CH-21 / ADR-0041).
//!
//! One event:
//!
//! - `platform.memory.extracted` — Logged class. Emitted by the
//!   `MemoryExtractionListener` body (CH-21 first emitter) every time a
//!   non-aborted `SessionEnded` produces a Memory row at v0's heuristic
//!   extraction. The diff carries `{session_id, owning_agent, tags,
//!   scope_bucket}` so log readers can correlate against the
//!   `platform.session.ended` event that preceded it without
//!   serialising the full Memory payload (Memory at v0 has no body
//!   field — content is encoded in `tags`).
//!
//! Concept doc cross-ref: `concepts/system-agents.md` § "Memory
//! Extraction Agent — Behaviour 5" — extraction emits one
//! `MemoryExtracted` audit per memory. The v0 heuristic listener emits
//! one per session.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, NodeId, OrgId, SessionId};
use crate::model::nodes::Memory;

/// Binary scope bucket assigned to an extracted Memory. Mirrors the
/// `Identity.witnessed.extraction_scope_distribution` carrier shipped at
/// CH-16 — `{private, public}` only at v0. The 4-pool routing (private
/// / project / org / `#public`) from concept-`system-agents.md` is
/// deferred to **M6-DEFERRED-04** (LLM-driven supervisor body); see
/// ADR-0040 § Out-of-Scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionScope {
    Private,
    Public,
}

impl ExtractionScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtractionScope::Private => "private",
            ExtractionScope::Public => "public",
        }
    }
}

/// `platform.memory.extracted` — Logged class (mid-tier retention).
///
/// Emitted post-commit by the `MemoryExtractionListener` body once the
/// `Memory` row is durably persisted. Pairs 1:1 with the
/// `platform.session.ended` event that triggered the extraction (same
/// `org_scope`, same hash-chain). `actor` is the working agent at v0
/// (the session's `started_by`) — when the LLM-driven supervisor body
/// lands in M6, `actor` becomes the memory-extractor system agent
/// instead per the supervisor-as-actor pattern in
/// `concepts/system-agents.md`.
pub fn memory_extracted(
    actor: AgentId,
    memory: &Memory,
    session_id: SessionId,
    scope_bucket: ExtractionScope,
    org: OrgId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "memory_id": memory.id.to_string(),
        "session_id": session_id.to_string(),
        "owning_agent": memory.owning_agent.to_string(),
        "tags": memory.tags,
        "scope_bucket": scope_bucket.as_str(),
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.memory.extracted".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*memory.id.as_uuid())),
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
    use crate::model::ids::{AgentId, MemoryId, OrgId, SessionId};

    fn sample_memory() -> Memory {
        Memory {
            id: MemoryId::new(),
            owning_agent: AgentId::new(),
            tags: vec!["agent:abc".into(), "session:def".into()],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn memory_extracted_event_is_logged_class() {
        let mem = sample_memory();
        let evt = memory_extracted(
            AgentId::new(),
            &mem,
            SessionId::new(),
            ExtractionScope::Private,
            OrgId::new(),
            Utc::now(),
        );
        assert_eq!(evt.event_type, "platform.memory.extracted");
        assert_eq!(evt.audit_class, AuditClass::Logged);
        assert_eq!(
            evt.target_entity_id,
            Some(NodeId::from_uuid(*mem.id.as_uuid()))
        );
        assert_eq!(evt.diff["scope_bucket"].as_str().unwrap(), "private");
        assert_eq!(evt.diff["memory_id"].as_str().unwrap(), mem.id.to_string());
        assert_eq!(
            evt.diff["owning_agent"].as_str().unwrap(),
            mem.owning_agent.to_string()
        );
    }

    #[test]
    fn memory_extracted_carries_public_scope_label_when_set() {
        let mem = sample_memory();
        let evt = memory_extracted(
            AgentId::new(),
            &mem,
            SessionId::new(),
            ExtractionScope::Public,
            OrgId::new(),
            Utc::now(),
        );
        assert_eq!(evt.diff["scope_bucket"].as_str().unwrap(), "public");
    }

    #[test]
    fn memory_extracted_with_empty_tag_list_serialises_an_empty_array() {
        let mut mem = sample_memory();
        mem.tags.clear();
        let evt = memory_extracted(
            AgentId::new(),
            &mem,
            SessionId::new(),
            ExtractionScope::Private,
            OrgId::new(),
            Utc::now(),
        );
        let arr = evt.diff["tags"].as_array().expect("tags is an array");
        assert!(arr.is_empty());
    }
}
