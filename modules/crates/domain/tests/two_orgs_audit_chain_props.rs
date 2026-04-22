//! Property test for per-org audit-chain isolation (M3/P3, C8).
//!
//! 50 proptest cases interleave emits across two orgs and assert
//! that each org's hash chain is completely independent of the other:
//! chain A's hash sequence never appears as chain B's `prev_event_hash`
//! and vice versa.
//!
//! The test runs against `InMemoryRepository` but replicates
//! `SurrealAuditEmitter`'s 3-line chain-link logic inline (look up
//! last hash for `event.org_scope`, assign `event.prev_event_hash`,
//! write). Keeping the emitter logic here (rather than dev-depping
//! `store`) avoids a workspace-layer cycle. The equivalent
//! SurrealDB-layer integration test is
//! `store/tests/audit_emitter_chain_test.rs`.

#![cfg(feature = "in-memory-repo")]

use chrono::Utc;
use proptest::prelude::*;

use domain::audit::{hash_event, AuditClass, AuditEvent};
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AuditEventId, OrgId};
use domain::repository::Repository;

/// Replicates `store::SurrealAuditEmitter::emit`'s chain-link step
/// against any `Repository`. Pure inline reproduction — keeps the
/// domain crate dependency-free while still exercising the exact
/// sequence the production emitter uses.
async fn emit_with_chain(repo: &InMemoryRepository, event: &mut AuditEvent) {
    let prev = repo
        .last_event_hash_for_org(event.org_scope)
        .await
        .expect("last_event_hash lookup");
    event.prev_event_hash = prev;
    repo.write_audit_event(event)
        .await
        .expect("write audit event");
}

fn mk_event(org: OrgId, tag: &str) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: tag.to_string(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::json!({"tag": tag}),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// Which org each emit in the schedule targets. `true` = org_a,
/// `false` = org_b.
fn arb_schedule() -> impl Strategy<Value = Vec<bool>> {
    prop::collection::vec(any::<bool>(), 4..=12)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// For any interleaving of emits across two orgs, the two hash
    /// chains must be independent:
    ///   - Every event in chain A has `prev_event_hash` pointing at
    ///     the previous A event (or `None` for the first A event).
    ///   - Every event in chain B has `prev_event_hash` pointing at
    ///     the previous B event (or `None` for the first B event).
    ///   - No hash from chain A ever appears as a `prev_event_hash`
    ///     in chain B, and vice versa.
    #[test]
    fn two_orgs_hash_chains_are_independent(schedule in arb_schedule()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async {
            let repo = InMemoryRepository::new();
            let org_a = OrgId::new();
            let org_b = OrgId::new();

            // Emit according to the schedule, record the order each
            // org saw its events in (for chain-continuity checks).
            let mut a_chain: Vec<AuditEvent> = Vec::new();
            let mut b_chain: Vec<AuditEvent> = Vec::new();
            for (i, is_a) in schedule.iter().enumerate() {
                let org = if *is_a { org_a } else { org_b };
                let tag = if *is_a {
                    format!("a.{i}")
                } else {
                    format!("b.{i}")
                };
                let mut ev = mk_event(org, &tag);
                emit_with_chain(&repo, &mut ev).await;
                if *is_a {
                    a_chain.push(ev);
                } else {
                    b_chain.push(ev);
                }
            }

            // --- Invariant 1: first event of each chain has
            //     prev_event_hash = None.
            if let Some(first) = a_chain.first() {
                prop_assert!(
                    first.prev_event_hash.is_none(),
                    "first A event must have prev_event_hash = None, got {:?}",
                    first.prev_event_hash
                );
            }
            if let Some(first) = b_chain.first() {
                prop_assert!(
                    first.prev_event_hash.is_none(),
                    "first B event must have prev_event_hash = None, got {:?}",
                    first.prev_event_hash
                );
            }

            // --- Invariant 2: every subsequent event in chain A
            //     points at the previous A event; same for B.
            for win in a_chain.windows(2) {
                let expected = hash_event(&win[0]);
                let got = win[1].prev_event_hash.expect("non-first event");
                prop_assert_eq!(
                    got, expected,
                    "A[{}].prev_event_hash != hash_event(A[{}])",
                    win[1].event_type, win[0].event_type
                );
            }
            for win in b_chain.windows(2) {
                let expected = hash_event(&win[0]);
                let got = win[1].prev_event_hash.expect("non-first event");
                prop_assert_eq!(
                    got, expected,
                    "B[{}].prev_event_hash != hash_event(B[{}])",
                    win[1].event_type, win[0].event_type
                );
            }

            // --- Invariant 3: no hash from chain A appears as a
            //     prev_event_hash in chain B, and vice versa.
            let a_hashes: std::collections::HashSet<[u8; 32]> =
                a_chain.iter().map(hash_event).collect();
            let b_hashes: std::collections::HashSet<[u8; 32]> =
                b_chain.iter().map(hash_event).collect();
            for ev in &b_chain {
                if let Some(prev) = ev.prev_event_hash {
                    prop_assert!(
                        !a_hashes.contains(&prev),
                        "B chain contains prev_event_hash from A chain — leak!"
                    );
                }
            }
            for ev in &a_chain {
                if let Some(prev) = ev.prev_event_hash {
                    prop_assert!(
                        !b_hashes.contains(&prev),
                        "A chain contains prev_event_hash from B chain — leak!"
                    );
                }
            }

            Ok::<_, TestCaseError>(())
        })?;
    }
}
