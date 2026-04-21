//! Audit-event builders for page 04 (credentials vault).
//!
//! Each builder is a **pure function** — no I/O, no hidden clock access
//! — returning a fully-populated [`AuditEvent`] whose `prev_event_hash`
//! is left `None` for the emitter to fill at persist time.
//!
//! Diff shapes are stable wire format: the web tier + acceptance tests
//! read these directly. If you change a field name here, update
//! [`modules/web/lib/api/audit.ts`] and the fixtures in
//! `server/tests/platform_secrets_test.rs` together.
//!
//! Audit-class policy:
//! - Every write (add / rotate / reveal / reassign / archive) is
//!   [`AuditClass::Alerted`] — platform-admin writes on sensitive
//!   material deserve immediate visibility (nfr-observability §Alerted).
//! - `SecretRevealAttemptDenied` is also `Alerted`: a denied reveal is
//!   the signal a custodian lost a grant, or an impostor is probing.
//! - `SecretListRead` is [`AuditClass::Logged`] — metadata-only read,
//!   keeps the chain live without flooding alert channels.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId, SecretId};
use crate::model::{SecretCredential, SecretRef};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a base-shaped [`AuditEvent`] with every field the secrets
/// builders share wired up. The caller overrides `event_type`, `diff`,
/// and `audit_class`.
fn scaffold(
    event_type: &str,
    actor: AgentId,
    target: SecretId,
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
        // Platform-scope writes chain under `None` (the root chain) —
        // orgs don't own platform-wide secrets. When M3+ introduces
        // per-org vault partitions, pass the org id here.
        org_scope: None,
        prev_event_hash: None,
    }
}

/// Render a `SecretCredential` into the "after" snapshot every builder
/// uses. Keeps the field order stable across event types. Never includes
/// ciphertext — the domain struct doesn't carry it.
fn after_snapshot(cred: &SecretCredential) -> serde_json::Value {
    serde_json::json!({
        "secret_id": cred.id.to_string(),
        "slug": cred.slug.as_str(),
        "custodian_id": cred.custodian.to_string(),
        "sensitive": cred.sensitive,
        "last_rotated_at": cred.last_rotated_at,
    })
}

// ---------------------------------------------------------------------------
// Builders — one per event type (7 total; plan §P4 D4).
// ---------------------------------------------------------------------------

/// `vault.secret.added` — a new vault entry just landed.
///
/// Emitted by [`server::platform::secrets::add`] after both the row +
/// catalogue seed are persisted. Alerted.
pub fn secret_added(
    actor: AgentId,
    cred: &SecretCredential,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": after_snapshot(cred),
    });
    scaffold(
        "vault.secret.added",
        actor,
        cred.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `vault.secret.rotated` — ciphertext material replaced; `last_rotated_at`
/// bumped. The diff captures only the rotation timestamp — plaintext
/// never appears in the audit trail.
pub fn secret_rotated(
    actor: AgentId,
    cred: &SecretCredential,
    prev_rotated_at: Option<DateTime<Utc>>,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": { "last_rotated_at": prev_rotated_at },
        "after": {
            "secret_id": cred.id.to_string(),
            "slug": cred.slug.as_str(),
            "last_rotated_at": cred.last_rotated_at,
        },
    });
    scaffold(
        "vault.secret.rotated",
        actor,
        cred.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `vault.secret.archived` — row marked archived. Not wired from a
/// handler in M2/P4 (no archive route); reserved for M3+. Ships now so
/// the event taxonomy + stable wire shape can be verified up-front.
pub fn secret_archived(
    actor: AgentId,
    cred: &SecretCredential,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": after_snapshot(cred),
        "after": { "archived_at": timestamp },
    });
    scaffold(
        "vault.secret.archived",
        actor,
        cred.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `vault.secret.custody_reassigned` — custodian agent changed. Old and
/// new custodians both captured so the reviewer can reconstruct the
/// delegation chain without cross-referencing.
pub fn secret_custody_reassigned(
    actor: AgentId,
    cred: &SecretCredential,
    previous_custodian: AgentId,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": { "custodian_id": previous_custodian.to_string() },
        "after": {
            "secret_id": cred.id.to_string(),
            "slug": cred.slug.as_str(),
            "custodian_id": cred.custodian.to_string(),
        },
    });
    scaffold(
        "vault.secret.custody_reassigned",
        actor,
        cred.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `vault.secret.revealed` — plaintext was handed to the actor.
///
/// Emitted **before** the handler streams plaintext back so a crash
/// between emit + send still leaves a trail. Alerted; carries the
/// purpose string the caller asserted in `constraint_context`.
pub fn secret_revealed(
    actor: AgentId,
    cred: &SecretCredential,
    purpose: &str,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "secret_id": cred.id.to_string(),
            "slug": cred.slug.as_str(),
            "custodian_id": cred.custodian.to_string(),
            "purpose": purpose,
        },
    });
    scaffold(
        "vault.secret.revealed",
        actor,
        cred.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `vault.secret.reveal_attempt_denied` — a caller attempted reveal and
/// the Permission Check rejected them. Alerted; the `failed_step` is
/// included so the reviewer knows whether the denial was catalogue /
/// grant / constraint / etc.
pub fn secret_reveal_attempt_denied(
    actor: AgentId,
    secret_id: SecretId,
    slug: &SecretRef,
    failed_step: &str,
    reason: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "secret_id": secret_id.to_string(),
            "slug": slug.as_str(),
            "failed_step": failed_step,
            "reason": reason,
        },
    });
    scaffold(
        "vault.secret.reveal_attempt_denied",
        actor,
        secret_id,
        timestamp,
        diff,
        AuditClass::Alerted,
        None,
    )
}

/// `vault.secret.list_read` — metadata-only read of the vault catalogue.
/// Logged (not Alerted) — operators listing the vault is routine.
///
/// `target_entity_id` is not meaningful for a list read; the builder
/// emits a zero-UUID placeholder so downstream tooling can still
/// dereference the field without branching.
pub fn secret_list_read(
    actor: AgentId,
    listed_count: usize,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "vault.secret.list_read".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(uuid::Uuid::nil())),
        timestamp,
        diff: serde_json::json!({
            "before": serde_json::Value::Null,
            "after": { "listed_count": listed_count },
        }),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Option::<OrgId>::None,
        prev_event_hash: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_credential() -> SecretCredential {
        SecretCredential {
            id: SecretId::new(),
            slug: SecretRef::new("anthropic-api-key"),
            custodian: AgentId::new(),
            last_rotated_at: None,
            sensitive: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn secret_added_is_alerted_and_names_event_correctly() {
        let actor = AgentId::new();
        let cred = sample_credential();
        let now = Utc::now();
        let ev = secret_added(actor, &cred, None, now);
        assert_eq!(ev.event_type, "vault.secret.added");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.actor_agent_id, Some(actor));
        assert_eq!(ev.timestamp, now);
        assert!(ev.prev_event_hash.is_none());
        assert_eq!(ev.diff["before"], serde_json::Value::Null);
        assert_eq!(ev.diff["after"]["slug"], "anthropic-api-key");
    }

    #[test]
    fn secret_rotated_captures_prev_and_new_timestamps() {
        let cred = sample_credential();
        let prev = DateTime::from_timestamp(1_700_000_000, 0);
        let ev = secret_rotated(AgentId::new(), &cred, prev, None, Utc::now());
        assert_eq!(ev.event_type, "vault.secret.rotated");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(
            ev.diff["before"]["last_rotated_at"],
            serde_json::json!(prev)
        );
    }

    #[test]
    fn secret_archived_snapshot_is_before_and_archived_at_is_after() {
        let cred = sample_credential();
        let now = Utc::now();
        let ev = secret_archived(AgentId::new(), &cred, None, now);
        assert_eq!(ev.event_type, "vault.secret.archived");
        assert_eq!(ev.diff["before"]["slug"], "anthropic-api-key");
        assert!(ev.diff["after"]["archived_at"].is_string());
    }

    #[test]
    fn secret_custody_reassigned_records_both_custodians() {
        let previous = AgentId::new();
        let cred = sample_credential();
        let ev = secret_custody_reassigned(AgentId::new(), &cred, previous, None, Utc::now());
        assert_eq!(ev.event_type, "vault.secret.custody_reassigned");
        assert_eq!(
            ev.diff["before"]["custodian_id"].as_str().unwrap(),
            previous.to_string()
        );
        assert_eq!(
            ev.diff["after"]["custodian_id"].as_str().unwrap(),
            cred.custodian.to_string()
        );
    }

    #[test]
    fn secret_revealed_includes_purpose() {
        let cred = sample_credential();
        let ev = secret_revealed(AgentId::new(), &cred, "rotate-downstream", None, Utc::now());
        assert_eq!(ev.event_type, "vault.secret.revealed");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["after"]["purpose"], "rotate-downstream");
    }

    #[test]
    fn secret_reveal_attempt_denied_captures_failed_step_and_reason() {
        let ev = secret_reveal_attempt_denied(
            AgentId::new(),
            SecretId::new(),
            &SecretRef::new("anthropic-api-key"),
            "Constraint",
            "constraint `purpose` was not satisfied",
            Utc::now(),
        );
        assert_eq!(ev.event_type, "vault.secret.reveal_attempt_denied");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["after"]["failed_step"], "Constraint");
        assert!(ev.diff["after"]["reason"]
            .as_str()
            .unwrap()
            .contains("purpose"));
    }

    #[test]
    fn secret_list_read_is_logged_tier() {
        let ev = secret_list_read(AgentId::new(), 3, Utc::now());
        assert_eq!(ev.event_type, "vault.secret.list_read");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.diff["after"]["listed_count"], 3);
    }

    #[test]
    fn every_builder_leaves_prev_event_hash_unset() {
        let cred = sample_credential();
        let now = Utc::now();
        let events = [
            secret_added(AgentId::new(), &cred, None, now),
            secret_rotated(AgentId::new(), &cred, None, None, now),
            secret_archived(AgentId::new(), &cred, None, now),
            secret_custody_reassigned(AgentId::new(), &cred, AgentId::new(), None, now),
            secret_revealed(AgentId::new(), &cred, "probe", None, now),
            secret_reveal_attempt_denied(
                AgentId::new(),
                cred.id,
                &cred.slug,
                "Catalogue",
                "miss",
                now,
            ),
            secret_list_read(AgentId::new(), 0, now),
        ];
        for ev in events {
            assert!(
                ev.prev_event_hash.is_none(),
                "builder `{}` must leave prev_event_hash unset",
                ev.event_type
            );
        }
    }
}
