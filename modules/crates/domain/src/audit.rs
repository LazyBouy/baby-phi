//! Audit event framework — base schema, class tiers, hash-chain seed.
//!
//! Every write handler in the server emits an `AuditEvent`. Events are
//! persisted to the primary `audit_events` store and also appended to a
//! shadow NDJSON log for cheap recoverability (decision D4 in the M1 plan).
//!
//! The `prev_event_hash` field chains events together within their
//! organization scope so tampering with a past event breaks every subsequent
//! event's chain — the M7b off-site-stream work verifies this end-to-end;
//! M1 only seeds the mechanism.
//!
//! Source of truth:
//! - `docs/specs/v0/requirements/cross-cutting/nfr-observability.md` — event
//!   schema + class-tier retention rules.
//! - `docs/specs/v0/concepts/permissions/02-auth-request.md` — `audit_class`
//!   field on Auth Request.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};
use crate::repository::RepositoryResult;

/// Retention class for an audit event.
///
/// Per `nfr-observability.md`:
/// - `Silent` — kept 30 days, no delivery.
/// - `Logged` — kept 365 days, logged to structured sink.
/// - `Alerted` — kept 365+ days, delivered to the org's alert channel
///   within 60 s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditClass {
    Silent,
    Logged,
    Alerted,
}

/// The canonical audit-event payload. Every write handler produces one of
/// these; the `AuditEmitter` writes them to storage and the shadow log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: AuditEventId,
    /// Dotted event name — e.g. `platform_admin.claimed`, `grant.issued`,
    /// `auth_request.transitioned`.
    pub event_type: String,
    pub actor_agent_id: Option<AgentId>,
    /// The entity the event is about (the object of the verb in `event_type`).
    pub target_entity_id: Option<NodeId>,
    pub timestamp: DateTime<Utc>,
    /// Structured diff — a free-form JSON value capturing before/after state.
    /// The shape is event-type-specific; schemas are documented alongside each
    /// emitter.
    pub diff: serde_json::Value,
    pub audit_class: AuditClass,
    /// The Auth Request that authorised this event, when one applies.
    pub provenance_auth_request_id: Option<AuthRequestId>,
    /// Scope for the hash-chain: events chain within an organization.
    pub org_scope: Option<OrgId>,
    /// SHA-256 of the previous event within the same `org_scope`. `None` only
    /// for the org's very first event.
    pub prev_event_hash: Option<[u8; 32]>,
}

impl AuditEvent {
    /// Deterministic bytes-to-hash for an event, excluding the `prev_event_hash`
    /// itself (that's the field the next event will copy). Any change to any
    /// captured field will change the hash.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct Canonical<'a> {
            event_id: &'a AuditEventId,
            event_type: &'a str,
            actor_agent_id: &'a Option<AgentId>,
            target_entity_id: &'a Option<NodeId>,
            timestamp: &'a DateTime<Utc>,
            diff: &'a serde_json::Value,
            audit_class: AuditClass,
            provenance_auth_request_id: &'a Option<AuthRequestId>,
            org_scope: &'a Option<OrgId>,
        }

        serde_json::to_vec(&Canonical {
            event_id: &self.event_id,
            event_type: &self.event_type,
            actor_agent_id: &self.actor_agent_id,
            target_entity_id: &self.target_entity_id,
            timestamp: &self.timestamp,
            diff: &self.diff,
            audit_class: self.audit_class,
            provenance_auth_request_id: &self.provenance_auth_request_id,
            org_scope: &self.org_scope,
        })
        .expect("canonical serialization is infallible for the event shape")
    }
}

/// SHA-256 of an event's canonical bytes — the digest the next event copies
/// into its `prev_event_hash` field.
pub fn hash_event(event: &AuditEvent) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&event.canonical_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

/// Write-side interface for audit events. Implementations write to both the
/// primary repository and the shadow NDJSON log, and also compute
/// `prev_event_hash` by looking up the last event's hash within the same
/// `org_scope`. Implementations land in P2 (store-side) and P6 (server
/// wiring).
#[async_trait]
pub trait AuditEmitter: Send + Sync + 'static {
    /// Emit one audit event. The emitter is responsible for:
    /// - Looking up the last event's hash within `org_scope` and populating
    ///   `prev_event_hash` before persisting.
    /// - Writing to the primary store.
    /// - Appending to the shadow NDJSON log.
    /// - For `Alerted` events, scheduling delivery to the org's alert channel.
    async fn emit(&self, event: AuditEvent) -> RepositoryResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(event_id: AuditEventId, prev: Option<[u8; 32]>) -> AuditEvent {
        AuditEvent {
            event_id,
            event_type: "platform_admin.claimed".to_string(),
            actor_agent_id: None,
            target_entity_id: None,
            timestamp: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            diff: serde_json::json!({"before": null, "after": {"id": "agent-1"}}),
            audit_class: AuditClass::Alerted,
            provenance_auth_request_id: None,
            org_scope: None,
            prev_event_hash: prev,
        }
    }

    #[test]
    fn canonical_bytes_excludes_prev_event_hash() {
        let id = AuditEventId::new();
        let a = sample_event(id, None);
        let b = sample_event(id, Some([7u8; 32]));
        assert_eq!(
            a.canonical_bytes(),
            b.canonical_bytes(),
            "prev_event_hash must not affect canonical bytes"
        );
    }

    #[test]
    fn hash_is_deterministic_for_same_content() {
        let id = AuditEventId::new();
        let a = sample_event(id, None);
        let b = sample_event(id, None);
        assert_eq!(hash_event(&a), hash_event(&b));
    }

    #[test]
    fn hash_changes_when_content_changes() {
        let id = AuditEventId::new();
        let a = sample_event(id, None);
        let mut b = a.clone();
        b.event_type = "platform_admin.rejected".to_string();
        assert_ne!(hash_event(&a), hash_event(&b));
    }

    #[test]
    fn audit_class_serde_roundtrip() {
        for c in [AuditClass::Silent, AuditClass::Logged, AuditClass::Alerted] {
            let j = serde_json::to_string(&c).unwrap();
            let back: AuditClass = serde_json::from_str(&j).unwrap();
            assert_eq!(back, c);
        }
    }
}
