//! Audit-event builders for page 02 (model providers).
//!
//! Three event types ship with M2/P5:
//! - `platform.model_provider.registered` (Alerted) — a new runtime was bound.
//! - `platform.model_provider.archived` (Alerted) — a runtime was soft-deleted.
//! - `platform.model_provider.health_degraded` (Alerted) — health probe saw a
//!   failure. M2 ships the **event shape only** — the real scheduled probe
//!   lands in M7b (plan G5). The builder exists so M7b can wire the probe
//!   path without redefining the diff shape.
//!
//! Diff-shape convention:
//! - `before` is `null` on `registered`; a snapshot of the catalogue row
//!   on `archived` / `health_degraded`.
//! - `after` always carries the identifying fields — `provider_id`,
//!   `provider_kind` (stringified `phi_core::provider::model::ApiProtocol`),
//!   `model_id`, `secret_ref` — plus the event-specific delta.
//! - Plaintext API keys are **never** in the diff. `secret_ref` is the
//!   vault slug; the actual key stays sealed in the vault.
//!
//! phi-core leverage: the `provider_kind` field is the `Display` form of
//! `phi_core::provider::model::ApiProtocol`; callers stringify the
//! wrapped-provider's `config.api` rather than re-normalising names.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, ModelProviderId, NodeId, OrgId};
use crate::model::ModelRuntime;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn scaffold(
    event_type: &str,
    actor: AgentId,
    target: ModelProviderId,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    audit_class: AuditClass,
    provenance_auth_request_id: Option<AuthRequestId>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*target.as_uuid())),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id,
        // Platform-scope writes chain under `None` (the root chain).
        // Per-org runtimes in M3+ will pass the owning org.
        org_scope: Option::<OrgId>::None,
        prev_event_hash: None,
    }
}

/// Shared "after" fields every provider event emits. Pulls the provider
/// kind off the embedded `phi_core::provider::model::ModelConfig` so
/// the diff's wire shape always matches phi-core's canonical
/// snake_case name for the protocol.
fn after_snapshot(rt: &ModelRuntime) -> serde_json::Value {
    serde_json::json!({
        "provider_id":   rt.id.to_string(),
        "model_id":      rt.config.id,
        "model_name":    rt.config.name,
        "provider_kind": rt.config.api.to_string(),
        "provider":      rt.config.provider,
        "base_url":      rt.config.base_url,
        "secret_ref":    rt.secret_ref.as_str(),
        "status":        rt.status,
        "archived_at":   rt.archived_at,
        "created_at":    rt.created_at,
    })
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

/// `platform.model_provider.registered` — a new LLM runtime was bound.
///
/// Emitted by the P5 `register_provider` handler after the Template E
/// AR + catalogue seed + per-instance grant are durable. Alerted.
pub fn model_provider_registered(
    actor: AgentId,
    runtime: &ModelRuntime,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after":  after_snapshot(runtime),
    });
    scaffold(
        "platform.model_provider.registered",
        actor,
        runtime.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.model_provider.archived` — runtime soft-deleted.
///
/// Archive in M2 is a metadata flip (`archived_at = now`) — existing
/// grants are left in place. A downstream revocation cascade is M3
/// work (see plan Part 11 Q8).
pub fn model_provider_archived(
    actor: AgentId,
    runtime: &ModelRuntime,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": after_snapshot(runtime),
        "after": {
            "provider_id": runtime.id.to_string(),
            "archived_at": timestamp,
        },
    });
    scaffold(
        "platform.model_provider.archived",
        actor,
        runtime.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.model_provider.health_degraded` — a probe of the provider
/// failed. Ships event-shape-only in M2 (no probe runs); the M7b
/// scheduled probe wires the emit call. Builder carries the old status
/// + new status + reason so the reviewer can correlate the incident.
pub fn model_provider_health_degraded(
    actor: AgentId,
    runtime: &ModelRuntime,
    previous_status: crate::model::RuntimeStatus,
    reason: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": { "status": previous_status },
        "after": {
            "provider_id": runtime.id.to_string(),
            "status":      runtime.status,
            "reason":      reason,
        },
    });
    scaffold(
        "platform.model_provider.health_degraded",
        actor,
        runtime.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        None,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::composites_m2::{RuntimeStatus, TenantSet};
    use crate::model::ids::ModelProviderId;
    use crate::model::{ModelRuntime, SecretRef};

    fn sample_runtime() -> ModelRuntime {
        let config = phi_core::provider::model::ModelConfig::anthropic(
            "claude-sonnet-4-20250514",
            "Claude Sonnet 4",
            "__vault__",
        );
        ModelRuntime {
            id: ModelProviderId::new(),
            config,
            secret_ref: SecretRef::new("anthropic-api-key"),
            tenants_allowed: TenantSet::All,
            status: RuntimeStatus::Ok,
            archived_at: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn registered_event_shape_and_class() {
        let rt = sample_runtime();
        let ev = model_provider_registered(AgentId::new(), &rt, None, Utc::now());
        assert_eq!(ev.event_type, "platform.model_provider.registered");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["before"], serde_json::Value::Null);
        // phi-core's Display for ApiProtocol::AnthropicMessages is
        // `anthropic_messages` — verify we pass that through.
        assert_eq!(ev.diff["after"]["provider_kind"], "anthropic_messages");
        assert_eq!(ev.diff["after"]["secret_ref"], "anthropic-api-key");
        // Plaintext api_key MUST NOT leak into the diff.
        let diff_str = serde_json::to_string(&ev.diff).unwrap();
        assert!(!diff_str.contains("__vault__"));
    }

    #[test]
    fn archived_event_captures_before_snapshot() {
        let rt = sample_runtime();
        let ev = model_provider_archived(AgentId::new(), &rt, None, Utc::now());
        assert_eq!(ev.event_type, "platform.model_provider.archived");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["before"]["provider_kind"], "anthropic_messages");
        assert!(ev.diff["after"]["archived_at"].is_string());
    }

    #[test]
    fn health_degraded_records_status_transition() {
        let rt = sample_runtime();
        let ev = model_provider_health_degraded(
            AgentId::new(),
            &rt,
            RuntimeStatus::Ok,
            "timeout contacting provider",
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.model_provider.health_degraded");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert!(ev.diff["after"]["reason"]
            .as_str()
            .unwrap()
            .contains("timeout"));
    }

    #[test]
    fn no_builder_sets_prev_event_hash() {
        let rt = sample_runtime();
        let now = Utc::now();
        for ev in [
            model_provider_registered(AgentId::new(), &rt, None, now),
            model_provider_archived(AgentId::new(), &rt, None, now),
            model_provider_health_degraded(AgentId::new(), &rt, RuntimeStatus::Ok, "timeout", now),
        ] {
            assert!(
                ev.prev_event_hash.is_none(),
                "emitter fills prev_event_hash; builder leaves it `None`"
            );
        }
    }
}
