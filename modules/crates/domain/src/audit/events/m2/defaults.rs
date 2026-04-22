//! Audit-event builder for page 05 (platform defaults).
//!
//! One event type ships with M2/P7:
//! - `platform.defaults.updated` (Alerted) — the singleton row was
//!   mutated. Diff carries:
//!     - `before.version` / `after.version` — monotonic revision bump;
//!     - `before` / `after` — full JSON snapshots of the struct
//!       (including the embedded phi-core sections), so a reviewer
//!       can see exactly which `ExecutionLimits`/`AgentProfile`/
//!       `ContextConfig`/`RetryConfig` fields changed.
//!
//! **Non-retroactive invariant.** The event carries the platform-level
//! change only; per-org snapshots (M3+) are not mutated. The invariant
//! is enforced structurally — the handler only writes to the
//! `platform_defaults` table — and pinned by the
//! `platform_defaults_non_retroactive_props` proptest.
//!
//! phi-core leverage: the `before` / `after` snapshots use
//! `serde_json::to_value(PlatformDefaults)` directly. Because every
//! phi-core-overlapping field is wrapped via `phi_core::...` types
//! (single source of truth), the wire diff always matches phi-core's
//! canonical serde shape — no phi transcoding involved.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};
use crate::model::PlatformDefaults;

/// Stable resource URI for the platform-defaults singleton. Matches
/// the catalogue seed done by the P7 handler so Permission-Check
/// Step 0 resolves on the URI.
pub const PLATFORM_DEFAULTS_URI: &str = "platform_defaults:singleton";

/// Deterministic NodeId for the platform-defaults singleton — the
/// row has no natural UUID (it's a fixed-key singleton), so every
/// audit event targeting it uses this stable id. Derived from a
/// keyed hash of the URI (plan §G11's spirit — Blake3 is already a
/// domain dep via the audit hash-chain, so we reuse rather than
/// pulling in sha2).
fn platform_defaults_target() -> NodeId {
    let digest = blake3::hash(PLATFORM_DEFAULTS_URI.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest.as_bytes()[..16]);
    NodeId::from_uuid(uuid::Uuid::from_bytes(bytes))
}

/// `platform.defaults.updated` — the singleton changed. Alerted tier.
///
/// `before` is `None` for the very first write (PUT from factory →
/// first persisted row); subsequent writes carry the full prior
/// snapshot for diff.
pub fn platform_defaults_updated(
    actor: AgentId,
    before: Option<&PlatformDefaults>,
    after: &PlatformDefaults,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let before_json = match before {
        Some(b) => serde_json::to_value(b).unwrap_or(serde_json::Value::Null),
        None => serde_json::Value::Null,
    };
    let after_json = serde_json::to_value(after).unwrap_or(serde_json::Value::Null);
    let diff = serde_json::json!({
        "before": before_json,
        "after":  after_json,
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform.defaults.updated".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(platform_defaults_target()),
        timestamp,
        diff,
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id,
        // Platform-scope writes chain under `None` (the root chain).
        org_scope: Option::<OrgId>::None,
        prev_event_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updated_event_shape_and_class() {
        let now = Utc::now();
        let before = PlatformDefaults::factory(now);
        let mut after = PlatformDefaults::factory(now);
        after.version = 1;
        after.default_retention_days = 60;
        let ev = platform_defaults_updated(AgentId::new(), Some(&before), &after, None, now);
        assert_eq!(ev.event_type, "platform.defaults.updated");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["before"]["version"], 0);
        assert_eq!(ev.diff["after"]["version"], 1);
        assert_eq!(ev.diff["after"]["default_retention_days"], 60);
    }

    #[test]
    fn first_write_reports_null_before() {
        let now = Utc::now();
        let after = PlatformDefaults::factory(now);
        let ev = platform_defaults_updated(AgentId::new(), None, &after, None, now);
        assert_eq!(ev.diff["before"], serde_json::Value::Null);
        assert!(ev.diff["after"].is_object());
    }

    #[test]
    fn target_entity_id_is_deterministic() {
        // Two calls produce events with the same target entity id —
        // guarantees every defaults audit event chains under the same
        // target, so a reviewer can `WHERE target_entity_id = …` to
        // recover the full revision history.
        let now = Utc::now();
        let pd = PlatformDefaults::factory(now);
        let ev1 = platform_defaults_updated(AgentId::new(), None, &pd, None, now);
        let ev2 = platform_defaults_updated(AgentId::new(), None, &pd, None, now);
        assert_eq!(ev1.target_entity_id, ev2.target_entity_id);
    }

    #[test]
    fn phi_core_nested_fields_round_trip_in_diff() {
        // Mutating a phi-core nested field (execution_limits.max_turns)
        // must surface in the diff without any phi-side
        // transcoding — this verifies the "phi-core serde is the
        // single source of truth" invariant.
        let now = Utc::now();
        let before = PlatformDefaults::factory(now);
        let mut after = PlatformDefaults::factory(now);
        after.execution_limits.max_turns = 999;
        after.version = 1;
        let ev = platform_defaults_updated(AgentId::new(), Some(&before), &after, None, now);
        assert_eq!(
            ev.diff["after"]["execution_limits"]["max_turns"], 999,
            "expected phi-core field to surface verbatim; diff was {:?}",
            ev.diff
        );
    }

    #[test]
    fn builder_leaves_prev_event_hash_unset() {
        let now = Utc::now();
        let pd = PlatformDefaults::factory(now);
        let ev = platform_defaults_updated(AgentId::new(), None, &pd, None, now);
        assert!(
            ev.prev_event_hash.is_none(),
            "emitter fills prev_event_hash; builder leaves it None"
        );
    }
}
