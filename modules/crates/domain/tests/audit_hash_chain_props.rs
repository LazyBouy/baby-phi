//! Property tests for [`domain::audit`] hash-chain continuity.
//!
//! Commitment-ledger row **C6** calls for unit + proptest coverage of
//! the per-org hash-chain semantics. Unit tests in `audit.rs` already
//! cover the basic contracts (canonical bytes exclude `prev_event_hash`;
//! identical content ⇒ identical hash; content-change ⇒ hash-change).
//! These proptests cover the harder invariants:
//!
//! 1. **Determinism under `prev_event_hash` variation.** The canonical
//!    bytes (and hence `hash_event`) must not depend on the previous
//!    event's digest — otherwise the chain self-references and a
//!    tamper to any earlier event would rewrite its own hash, defeating
//!    the chain's tamper-evidence property.
//! 2. **Forward-chain linkage.** When event *N+1*'s `prev_event_hash`
//!    is set to `hash_event(N)`, flipping any captured field in *N*
//!    changes *N*'s hash — which any downstream verifier will catch by
//!    re-computing *N*'s hash and comparing to *N+1*'s `prev_event_hash`.
//! 3. **Org-scope isolation.** Two events with different `org_scope`
//!    values produce distinct hashes even when every other field is
//!    identical — so chains cannot accidentally merge across tenants.

use chrono::{DateTime, Utc};
use domain::audit::{hash_event, AuditClass, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, OrgId};

use proptest::prelude::*;

fn event_with(
    event_id: AuditEventId,
    event_type: &str,
    timestamp_secs: i64,
    prev: Option<[u8; 32]>,
    org_scope: Option<OrgId>,
) -> AuditEvent {
    AuditEvent {
        event_id,
        event_type: event_type.to_string(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: DateTime::<Utc>::from_timestamp(timestamp_secs, 0).unwrap(),
        diff: serde_json::json!({"after": {"id": "x"}}),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope,
        prev_event_hash: prev,
    }
}

fn arb_event_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("platform_admin.claimed".to_string()),
        Just("grant.issued".to_string()),
        Just("grant.revoked".to_string()),
        Just("auth_request.transitioned".to_string()),
        Just("auth_request.revoked".to_string()),
    ]
}

fn arb_hash() -> impl Strategy<Value = [u8; 32]> {
    prop::array::uniform32(any::<u8>())
}

proptest! {
    /// `hash_event` must not depend on `prev_event_hash`. If it did, the
    /// chain would self-reference and a tampered earlier event would
    /// rewrite its own hash to match the downstream reference, defeating
    /// tamper-evidence.
    #[test]
    fn hash_is_independent_of_prev_event_hash(
        ty in arb_event_type(),
        ts in 0i64..4_000_000_000i64,
        prev_a in prop::option::of(arb_hash()),
        prev_b in prop::option::of(arb_hash()),
    ) {
        let id = AuditEventId::new();
        let a = event_with(id, &ty, ts, prev_a, None);
        let b = event_with(id, &ty, ts, prev_b, None);
        prop_assert_eq!(hash_event(&a), hash_event(&b),
            "hash_event must be invariant under prev_event_hash changes");
    }

    /// Forward-chain linkage: if any captured field of event N changes,
    /// event N's hash changes, and therefore any downstream verifier
    /// comparing `prev_event_hash(N+1)` against a recomputed `hash(N)`
    /// will detect the tamper. This proptest flips one captured field
    /// at random and asserts the hash diverges.
    #[test]
    fn changing_any_captured_field_changes_hash(
        ty in arb_event_type(),
        new_ty in arb_event_type(),
        ts in 0i64..4_000_000_000i64,
        ts_delta in 1i64..1_000_000i64,
        field in 0usize..4,
    ) {
        let id = AuditEventId::new();
        let orig = event_with(id, &ty, ts, None, None);

        // Build a tampered copy by mutating exactly one captured field.
        let mut tampered = orig.clone();
        match field {
            0 => {
                // Event type.
                prop_assume!(new_ty != ty);
                tampered.event_type = new_ty;
            }
            1 => {
                // Timestamp.
                tampered.timestamp =
                    DateTime::<Utc>::from_timestamp(ts + ts_delta, 0).unwrap();
            }
            2 => {
                // Diff payload.
                tampered.diff = serde_json::json!({"after": {"id": "y"}});
            }
            _ => {
                // Actor agent id.
                tampered.actor_agent_id = Some(AgentId::new());
            }
        }

        prop_assert_ne!(hash_event(&orig), hash_event(&tampered),
            "tampering with a captured field must change hash (field = {})", field);
    }

    /// Org-scope isolation: two otherwise-identical events in different
    /// org scopes hash differently, so chains cannot accidentally merge
    /// across tenants.
    #[test]
    fn org_scope_is_captured_in_hash(
        ty in arb_event_type(),
        ts in 0i64..4_000_000_000i64,
    ) {
        let id = AuditEventId::new();
        let org_a = OrgId::new();
        let org_b = OrgId::new();
        prop_assume!(org_a != org_b);
        let a = event_with(id, &ty, ts, None, Some(org_a));
        let b = event_with(id, &ty, ts, None, Some(org_b));
        prop_assert_ne!(hash_event(&a), hash_event(&b),
            "different org_scope must produce different hash");
    }

    /// Two-event chain linkage: given events N and N+1 where N+1 copies
    /// hash(N) into its prev_event_hash, a verifier can detect tamper
    /// by recomputing hash(N) and checking equality. This is the
    /// "chain verification" operation downstream code performs; we
    /// check it here by simulating the check after a tamper.
    #[test]
    fn two_event_chain_detects_tamper_on_predecessor(
        ty0 in arb_event_type(),
        ty1 in arb_event_type(),
        ts0 in 0i64..2_000_000_000i64,
        ts1_delta in 1i64..1_000_000i64,
        tampered_diff_seed in any::<u8>(),
    ) {
        let id0 = AuditEventId::new();
        let id1 = AuditEventId::new();
        let e0 = event_with(id0, &ty0, ts0, None, None);
        let h0 = hash_event(&e0);
        let e1 = event_with(id1, &ty1, ts0 + ts1_delta, Some(h0), None);

        // Honest verification.
        prop_assert_eq!(e1.prev_event_hash.unwrap(), hash_event(&e0));

        // Now tamper: mutate e0's diff in a way the seed makes
        // unambiguously different.
        let mut e0_tampered = e0.clone();
        e0_tampered.diff = serde_json::json!({"after": {"tampered": tampered_diff_seed}});
        // The stored prev_event_hash on e1 no longer matches the
        // recomputed hash(e0_tampered).
        prop_assert_ne!(e1.prev_event_hash.unwrap(), hash_event(&e0_tampered),
            "chain verifier must flag tampered predecessor");
    }
}
