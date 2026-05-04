//! CH-12 / ADR-0049 §D49.3 + §D49.4 + §D49.7 — acceptance suite for
//! the runtime tag-write validator + the paired audit-event builder.
//!
//! Two layers exercised end-to-end here.
//!
//! Layer 1 — Runtime path (Tests 1-7). Direct calls to
//! [`domain::permissions::validate_tag_write_on_session`] over
//! representative tag-set transitions. No production callsite emits
//! these errors today (no `update_session_tags` Repository method
//! exists at CH-12); the validator is forward-defensive symmetric
//! to CH-05's publish-time validator. The pure-fn rejection contract
//! is the unit of truth that future wiring chunks will rely on.
//!
//! Layer 2 — Audit-event integration path (Tests 8-10, F5.B). End-to-end
//! proof that the
//! `domain::audit::events::m5_2::tool_authority::frozen_tag_write_rejected`
//! builder produces an `AuditEvent` whose hash-chain semantics match
//! every other CH-21 event-type. Tests 8-9 exercise the validator +
//! audit-event pairing contract documented on the `Repository` trait
//! module-level docstring (`validate_tag_write_on_session` +
//! `frozen_tag_write_rejected` paired at any future tag-write
//! callsite). Test 10 pins per-org chain isolation for the new
//! event_type: different `org_scope` values produce events with
//! different hash-chain ancestry, mirroring CH-21's
//! `audit_emitter_chain_test::chains_are_scoped_per_org`.
//!
//! Concept-doc anchors:
//! - `permissions/05-memory-sessions.md` §"Frozen-at-creation tags
//!   (immutability)" lines 220–231 — the prefix list.
//! - `permissions/05-memory-sessions.md` Example 7 lines 516–525 —
//!   "worker tries to retag... Denied. The request never reaches the
//!   storage layer."

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tempfile::tempdir;

use domain::audit::events::m5_2::tool_authority::frozen_tag_write_rejected;
use domain::audit::{hash_event, AuditClass, AuditEmitter, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId, SessionId};
use domain::permissions::{validate_tag_write_on_session, FrozenTagViolation};
use domain::Repository;
use store::{SurrealAuditEmitter, SurrealStore};

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

fn fixed_timestamp() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp")
}

async fn fresh_emitter() -> (
    Arc<SurrealAuditEmitter>,
    Arc<dyn Repository>,
    tempfile::TempDir,
) {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");
    let repo: Arc<dyn Repository> = Arc::new(store);
    let emitter = Arc::new(SurrealAuditEmitter::new(repo.clone()));
    (emitter, repo, dir)
}

// ============================================================================
// Tests 1–7 — Runtime validator path
// ============================================================================

#[test]
fn test_1_no_op_tag_set_lifecycle_flip_passes() {
    // Lifecycle flip (`#active` -> `#archived`) on a session whose
    // structural-tag set is unchanged. `#active` / `#archived` are not
    // in SESSION_FROZEN_TAG_PREFIXES, so the validator returns Ok(()).
    let session = SessionId::new();
    let current = vec![
        "#kind:session".to_string(),
        format!("session:{}", session),
        "#active".to_string(),
    ];
    let proposed = vec![
        "#kind:session".to_string(),
        format!("session:{}", session),
        "#archived".to_string(),
    ];
    validate_tag_write_on_session(session, &current, &proposed)
        .expect("lifecycle flip with stable structural tags must pass");
}

#[test]
fn test_2_adding_agent_tag_triggers_frozen_tag_added() {
    // Worker tries to add a new `agent:bob` tag to an existing session.
    // `agent:` is in SESSION_FROZEN_TAG_PREFIXES → FrozenTagAdded.
    let session = SessionId::new();
    let current = vec![
        "#kind:session".to_string(),
        format!("session:{}", session),
        "agent:claude".to_string(),
    ];
    let proposed = vec![
        "#kind:session".to_string(),
        format!("session:{}", session),
        "agent:claude".to_string(),
        "agent:bob".to_string(),
    ];
    let err = validate_tag_write_on_session(session, &current, &proposed)
        .expect_err("adding agent:bob must reject");
    match err {
        FrozenTagViolation::FrozenTagAdded { session_id, tag } => {
            assert_eq!(session_id, session);
            assert_eq!(tag, "agent:bob");
        }
        other => panic!("expected FrozenTagAdded{{ agent:bob }}, got {:?}", other),
    }
}

#[test]
fn test_3_removing_session_tag_triggers_frozen_tag_removed() {
    // Worker tries to remove the `session:<id>` instance-identity tag.
    // `session:` is in SESSION_FROZEN_TAG_PREFIXES → FrozenTagRemoved.
    let session = SessionId::new();
    let session_tag = format!("session:{}", session);
    let current = vec![
        "#kind:session".to_string(),
        session_tag.clone(),
        "agent:claude".to_string(),
    ];
    let proposed = vec![
        "#kind:session".to_string(),
        // session_tag deliberately removed.
        "agent:claude".to_string(),
    ];
    let err = validate_tag_write_on_session(session, &current, &proposed)
        .expect_err("removing session:<id> must reject");
    match err {
        FrozenTagViolation::FrozenTagRemoved { session_id, tag } => {
            assert_eq!(session_id, session);
            assert_eq!(tag, session_tag);
        }
        other => panic!(
            "expected FrozenTagRemoved{{ session:<id> }}, got {:?}",
            other
        ),
    }
}

#[test]
fn test_4_concept_doc_example_7_worker_retag_attempt_rejected() {
    // Concept-doc 05 Example 7 fixture (lines 516–525):
    // Worker has session `s1` tagged `["session:s1", "agent:claude-coder-9"]`
    // and tries to retag to add `project:other-project`. `project:` is
    // in SESSION_FROZEN_TAG_PREFIXES → FrozenTagAdded. The validator
    // surfaces the rejection BEFORE the storage layer is touched.
    let session = SessionId::new();
    let session_tag = format!("session:{}", session);
    let current = vec![session_tag.clone(), "agent:claude-coder-9".to_string()];
    let proposed = vec![
        session_tag,
        "agent:claude-coder-9".to_string(),
        "project:other-project".to_string(),
    ];
    let err = validate_tag_write_on_session(session, &current, &proposed)
        .expect_err("Example 7 retag attempt must reject");
    assert!(
        matches!(
            err,
            FrozenTagViolation::FrozenTagAdded {
                ref tag,
                ..
            } if tag == "project:other-project"
        ),
        "expected FrozenTagAdded{{ project:other-project }}, got {:?}",
        err
    );
}

#[test]
fn test_5_lifecycle_only_change_passes() {
    // Lifecycle-only happy path: structural tags identical, only
    // `#active` -> `#archived` flip. Validates that
    // SESSION_FROZEN_TAG_PREFIXES does NOT include `#active` /
    // `#archived` (per concept doc 05 line 541 — agent-mutable).
    let session = SessionId::new();
    let session_tag = format!("session:{}", session);
    let current = vec![
        "#kind:session".to_string(),
        session_tag.clone(),
        "agent:claude".to_string(),
        "#active".to_string(),
    ];
    let proposed = vec![
        "#kind:session".to_string(),
        session_tag,
        "agent:claude".to_string(),
        "#archived".to_string(),
    ];
    validate_tag_write_on_session(session, &current, &proposed)
        .expect("pure lifecycle flip must pass");
}

#[test]
fn test_6_round_trip_identity_passes() {
    // Round-trip: current == proposed exactly. No frozen tag added or
    // removed. Validates the function is total over the no-change input.
    let session = SessionId::new();
    let tags = vec![
        "#kind:session".to_string(),
        format!("session:{}", session),
        "agent:claude".to_string(),
        "project:phi".to_string(),
        "#active".to_string(),
    ];
    validate_tag_write_on_session(session, &tags, &tags).expect("round-trip identity must pass");
}

#[test]
fn test_7_empty_tags_edge_case_passes() {
    // Edge case: both current and proposed tag sets empty. No frozen
    // namespace ever appears → Ok(()). Validates the empty-input
    // boundary doesn't regress to a panic / unexpected error.
    let session = SessionId::new();
    let current: Vec<String> = vec![];
    let proposed: Vec<String> = vec![];
    validate_tag_write_on_session(session, &current, &proposed)
        .expect("empty-tags edge case must pass");
}

// ============================================================================
// Tests 8–10 — Audit-event integration path (F5.B)
//
// These tests exercise the validator + audit-event pairing contract
// from the Repository trait module-level docstring:
// `validate_tag_write_on_session` rejection paired with
// `frozen_tag_write_rejected(...)` emission. End-to-end demonstration
// that future Repository wiring will produce a chain-linked AuditEvent
// for every frozen-tag-write rejection.
// ============================================================================

#[tokio::test]
async fn test_8_added_violation_paired_with_audit_event_persists_with_chain() {
    // (1) Pure-fn rejection: validate_tag_write_on_session returns
    //     Err(FrozenTagAdded).
    // (2) Pair with audit-event: frozen_tag_write_rejected(...) builds
    //     an AuditEvent with the F5.B contract fields.
    // (3) Persist via SurrealAuditEmitter; verify it round-trips with
    //     prev_event_hash threaded from a prior event in the same
    //     org_scope (CH-21 chain symmetry holds for the new event_type).
    let (emitter, repo, _dir) = fresh_emitter().await;
    let actor = AgentId::new();
    let session = SessionId::new();
    let org = OrgId::new();

    // Seed the org chain with one event, then emit the rejection.
    let seed = AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "platform_admin.claimed".to_string(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::json!({"before": null, "after": {"seed": true}}),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    };
    let seed_id = seed.event_id;
    emitter.emit(seed).await.unwrap();
    let stored_seed = repo.get_audit_event(seed_id).await.unwrap().unwrap();
    let expected_prev = hash_event(&stored_seed);

    // SAFETY pause for SurrealDB ms-resolution timestamp tiebreak.
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;

    // (1) Pure-fn rejection.
    let session_tag = format!("session:{}", session);
    let current = vec![session_tag.clone(), "agent:claude".to_string()];
    let proposed = vec![
        session_tag,
        "agent:claude".to_string(),
        "agent:rogue".to_string(),
    ];
    let violation = validate_tag_write_on_session(session, &current, &proposed)
        .expect_err("FrozenTagAdded expected");

    // (2) Pair with audit-event.
    let evt = frozen_tag_write_rejected(actor, session, org, &violation, fixed_timestamp());
    let evt_id = evt.event_id;
    assert_eq!(evt.event_type, "tool.frozen_tag_write_rejected");
    assert_eq!(evt.audit_class, AuditClass::Alerted);
    assert_eq!(
        evt.target_entity_id,
        Some(NodeId::from_uuid(*session.as_uuid()))
    );
    assert_eq!(
        evt.diff["after"]["tag"].as_str().unwrap(),
        "agent:rogue",
        "diff.after.tag must match the rejected tag"
    );

    // (3) Persist.
    emitter.emit(evt).await.unwrap();
    let stored = repo.get_audit_event(evt_id).await.unwrap().unwrap();
    assert_eq!(
        stored.prev_event_hash,
        Some(expected_prev),
        "F5.B: rejection event's prev_event_hash must thread from the prior event in the same org_scope"
    );
    assert_eq!(stored.event_type, "tool.frozen_tag_write_rejected");
    assert_eq!(stored.audit_class, AuditClass::Alerted);
    assert_eq!(stored.org_scope, Some(org));
}

#[tokio::test]
async fn test_9_removed_violation_paired_with_audit_event_persists_with_chain() {
    // Symmetric to Test 8 but for the FrozenTagRemoved branch — pin
    // both validator output paths through the audit-event integration.
    let (emitter, repo, _dir) = fresh_emitter().await;
    let actor = AgentId::new();
    let session = SessionId::new();
    let org = OrgId::new();

    // (1) Pure-fn rejection: remove session:<id>.
    let session_tag = format!("session:{}", session);
    let current = vec![session_tag.clone(), "agent:claude".to_string()];
    let proposed = vec![
        // session_tag deliberately removed.
        "agent:claude".to_string(),
    ];
    let violation = validate_tag_write_on_session(session, &current, &proposed)
        .expect_err("FrozenTagRemoved expected");
    assert!(
        matches!(violation, FrozenTagViolation::FrozenTagRemoved { .. }),
        "expected FrozenTagRemoved branch"
    );

    // (2) Pair with audit-event.
    let evt = frozen_tag_write_rejected(actor, session, org, &violation, fixed_timestamp());
    let evt_id = evt.event_id;
    assert_eq!(
        evt.diff["after"]["violation_kind"].as_str().unwrap(),
        "frozen_tag_removed"
    );
    assert_eq!(
        evt.diff["after"]["tag"].as_str().unwrap(),
        session_tag,
        "diff.after.tag must match the removed tag"
    );

    // (3) Persist as the FIRST event in this org's chain →
    // prev_event_hash must be None.
    emitter.emit(evt).await.unwrap();
    let stored = repo.get_audit_event(evt_id).await.unwrap().unwrap();
    assert!(
        stored.prev_event_hash.is_none(),
        "first event in a new org_scope must not carry a prev hash"
    );
    assert_eq!(stored.event_type, "tool.frozen_tag_write_rejected");
    assert_eq!(stored.audit_class, AuditClass::Alerted);
    assert_eq!(stored.org_scope, Some(org));
}

#[tokio::test]
async fn test_10_two_orgs_produce_independent_chains_for_new_event_type() {
    // Cross-org chain isolation: emit one frozen_tag_write_rejected in
    // org_a, then one in org_b. Both must surface as the FIRST event
    // in their respective `org_scope` chains (prev_event_hash == None
    // for both). Mirrors CH-21's
    // `audit_emitter_chain_test::chains_are_scoped_per_org` for the
    // additive event_type. Pins F5.B's per-org isolation claim.
    let (emitter, repo, _dir) = fresh_emitter().await;
    let actor = AgentId::new();

    // Org A: reject one frozen-tag-add.
    let org_a = OrgId::new();
    let session_a = SessionId::new();
    let session_tag_a = format!("session:{}", session_a);
    let current_a = vec![session_tag_a.clone()];
    let proposed_a = vec![session_tag_a, "agent:rogue".to_string()];
    let v_a = validate_tag_write_on_session(session_a, &current_a, &proposed_a)
        .expect_err("FrozenTagAdded in org_a");
    let evt_a = frozen_tag_write_rejected(actor, session_a, org_a, &v_a, fixed_timestamp());
    let id_a = evt_a.event_id;
    emitter.emit(evt_a).await.unwrap();

    // Wait so SurrealDB's timestamp ORDER BY tiebreaks deterministically.
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;

    // Org B: reject one frozen-tag-remove. Must see an empty chain
    // (first-in-scope for org_b), even though org_a already has one
    // event in its chain.
    let org_b = OrgId::new();
    let session_b = SessionId::new();
    let session_tag_b = format!("session:{}", session_b);
    let current_b = vec![session_tag_b];
    let proposed_b: Vec<String> = vec![]; // remove the structural tag
    let v_b = validate_tag_write_on_session(session_b, &current_b, &proposed_b)
        .expect_err("FrozenTagRemoved in org_b");
    let evt_b = frozen_tag_write_rejected(actor, session_b, org_b, &v_b, fixed_timestamp());
    let id_b = evt_b.event_id;
    emitter.emit(evt_b).await.unwrap();

    // Both stored events must have org_scope set to their respective
    // orgs and DIFFERENT org_scope values.
    let stored_a = repo.get_audit_event(id_a).await.unwrap().unwrap();
    let stored_b = repo.get_audit_event(id_b).await.unwrap().unwrap();

    assert_eq!(stored_a.org_scope, Some(org_a));
    assert_eq!(stored_b.org_scope, Some(org_b));
    assert_ne!(
        stored_a.org_scope, stored_b.org_scope,
        "cross-org chain isolation: org_scope values must differ"
    );

    // Org_b's event must NOT chain to org_a's event — both are first
    // in their own org_scope chain.
    assert!(
        stored_a.prev_event_hash.is_none(),
        "org_a's first event must not carry a prev hash"
    );
    assert!(
        stored_b.prev_event_hash.is_none(),
        "org_b's first event must not link to org_a's chain"
    );
}
