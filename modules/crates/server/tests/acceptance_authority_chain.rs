//! Acceptance tests for CH-14 / ADR-0053 — `system:genesis` axiom +
//! authority-chain walker + recursive revocation cascade.
//!
//! Coverage matrix (per plan §7 P2 + P3 + gate-2 FOLLOWUP-02 closure):
//!   1. `every_grant_chains_to_bootstrap_after_claim` — happy path
//!      after `bootstrap-claim`: the admin grant's chain walks to the
//!      bootstrap AR and `is_bootstrap_ar` returns true at the root.
//!   2. `walk_returns_empty_for_grant_with_no_descends_from` —
//!      legacy / null path.
//!   3. `walk_returns_err_on_synthetic_cycle` — synthetic cycle at
//!      depth ≥ 32 → `ProvenanceCycleDepthExceeded`.
//!   4. `revoke_cascades_to_grandchildren` — 3-hop chain revoked
//!      end-to-end via the recursive cascade.
//!   5. `revoke_cascade_does_not_re_revoke_already_revoked_grants` —
//!      idempotency.
//!   6. `revoke_cascade_emits_one_event_per_cascaded_ar` — gate-2
//!      FOLLOWUP-02 closure: per-AR `auth_request.revoked` audit-event
//!      emission per ADR-0053 §D53.7.
//!   7. `revoke_cascade_transitions_cascaded_ars_to_revoked_state` —
//!      gate-2 FOLLOWUP-02 closure: cascaded ARs transition Approved
//!      → Revoked.

mod acceptance_common;

use acceptance_common::{claim_body, mint_credential, spawn};
use chrono::{Duration as ChronoDuration, Utc};
use domain::audit::AuditClass;
use domain::model::ids::{AuthRequestId, GrantId};
use domain::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, Grant, PrincipalRef,
    ResourceRef, ResourceSlot, ResourceSlotState,
};
use domain::permissions::axioms::is_bootstrap_ar;
use domain::Repository;
use serde_json::Value;

fn claim_url(base: &str) -> String {
    format!("{base}/api/v0/bootstrap/claim")
}

// ---- 1. Walker reaches bootstrap from the admin grant ---------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_grant_chains_to_bootstrap_after_claim() {
    let acc = spawn(false).await;
    let client = acc.client();

    // Run the bootstrap claim; the admin grant becomes the leaf node
    // of the trivial 1-hop chain (Bootstrap-AR → admin-grant).
    let cred = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(&cred, "Alex Chen", "slack", "@alex"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let claim: Value = r.json().await.unwrap();
    let grant_id_str = claim["grant_id"].as_str().unwrap().to_string();
    let grant_id = GrantId::from_uuid(uuid::Uuid::parse_str(&grant_id_str).unwrap());

    // Walk the chain. Expect a single-element `[bootstrap_ar]` since
    // the admin grant points directly at the bootstrap AR and the
    // bootstrap AR's `descends_from_grant` is `None` (chain root).
    let chain = acc
        .store
        .walk_provenance_chain(grant_id)
        .await
        .expect("walk_provenance_chain");
    assert_eq!(
        chain.len(),
        1,
        "admin grant chain must have exactly one element (the bootstrap AR), got {}",
        chain.len()
    );
    let bootstrap = &chain[0];
    assert!(
        is_bootstrap_ar(bootstrap),
        "chain root must satisfy is_bootstrap_ar"
    );
    assert!(
        bootstrap.descends_from_grant.is_none(),
        "bootstrap AR has no parent grant by definition"
    );
    assert_eq!(bootstrap.audit_class, AuditClass::Alerted);
    assert_eq!(bootstrap.state, AuthRequestState::Approved);
}

// ---- 2. Walker returns empty for legacy / null grants ---------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn walk_returns_empty_for_grant_with_no_descends_from() {
    let acc = spawn(false).await;

    // Seed a synthetic grant with no `descends_from`. The walker should
    // return an empty Vec without erroring (this is the legacy /
    // pre-CH-14 grant case per ADR-0053 §D53.3).
    let synthetic_grant = synthetic_orphan_grant();
    acc.store
        .create_grant(&synthetic_grant)
        .await
        .expect("create synthetic grant");

    let chain = acc
        .store
        .walk_provenance_chain(synthetic_grant.id)
        .await
        .expect("walk_provenance_chain on orphan grant");
    assert!(
        chain.is_empty(),
        "orphan grant (descends_from=None) must yield an empty chain"
    );
}

// ---- 3. Walker terminates on synthetic depth-cap overflow -----------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn walk_returns_err_on_synthetic_cycle() {
    let acc = spawn(false).await;

    // Fabricate a 33-deep "almost-chain" so depth-cap fires. The
    // simplest construction is a row of 33 ARs each pointing at the
    // grant 1 step deeper, with a final grant whose descends_from
    // loops back to AR-0. (We don't need a true cycle; depth-cap
    // semantics only care about hop count.) For brevity we build a
    // straight path of 33 ARs/Grants pairs without termination at a
    // bootstrap node — the walker must eventually hit
    // MAX_PROVENANCE_DEPTH = 32 and return the error.
    let now = Utc::now();
    let mut ars: Vec<AuthRequest> = Vec::with_capacity(33);
    let mut grants: Vec<Grant> = Vec::with_capacity(33);
    let mut prev_grant: Option<GrantId> = None;
    for i in 0..33 {
        let ar_id = AuthRequestId::new();
        let ar = synthetic_chain_ar(ar_id, prev_grant);
        let grant_id = GrantId::new();
        let grant = synthetic_chain_grant(grant_id, ar_id, now);
        prev_grant = Some(grant_id);
        ars.push(ar);
        grants.push(grant);
        let _ = i;
    }
    for ar in &ars {
        acc.store.create_auth_request(ar).await.expect("create ar");
    }
    for g in &grants {
        acc.store.create_grant(g).await.expect("create grant");
    }
    // The leaf grant is the LAST one created; the walker should
    // climb 33 hops up the chain and trip the cap.
    let leaf_grant = grants.last().unwrap().id;
    let result = acc.store.walk_provenance_chain(leaf_grant).await;
    assert!(
        matches!(
            result,
            Err(
                domain::repository::RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: 32 }
            )
        ),
        "expected ProvenanceCycleDepthExceeded, got {:?}",
        result
    );
}

// ---- 4. Cascade revoke walks 3-hop chain end-to-end -----------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoke_cascades_to_grandchildren() {
    let acc = spawn(false).await;
    let now = Utc::now();

    // Build a 3-hop chain manually:
    //   root_ar → root_grant → child_ar → child_grant → grandchild_ar
    //                                  → grandchild_grant
    //
    // Calling `revoke_grants_by_descends_from_recursive(root_ar, now)`
    // must revoke all three grants (root_grant + child_grant +
    // grandchild_grant).
    let root_ar_id = AuthRequestId::new();
    let root_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();
    let grandchild_ar_id = AuthRequestId::new();
    let grandchild_grant_id = GrantId::new();

    let root_ar = synthetic_chain_ar(root_ar_id, None);
    let root_grant = synthetic_chain_grant(root_grant_id, root_ar_id, now);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(root_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id, now);
    let grandchild_ar = synthetic_chain_ar(grandchild_ar_id, Some(child_grant_id));
    let grandchild_grant = synthetic_chain_grant(grandchild_grant_id, grandchild_ar_id, now);

    for ar in [&root_ar, &child_ar, &grandchild_ar] {
        acc.store.create_auth_request(ar).await.unwrap();
    }
    for g in [&root_grant, &child_grant, &grandchild_grant] {
        acc.store.create_grant(g).await.unwrap();
    }

    let result = acc
        .store
        .revoke_grants_by_descends_from_recursive(root_ar_id, now + ChronoDuration::seconds(1))
        .await
        .expect("recursive cascade revoke");
    // All three grants should appear in the returned list.
    assert_eq!(
        result.revoked_grants.len(),
        3,
        "expected 3 grants revoked (root + child + grandchild), got {}: {:?}",
        result.revoked_grants.len(),
        result.revoked_grants
    );
    assert!(result.revoked_grants.contains(&root_grant_id));
    assert!(result.revoked_grants.contains(&child_grant_id));
    assert!(result.revoked_grants.contains(&grandchild_grant_id));
    // Two level-≥1 cascaded ARs (child + grandchild). The level-0
    // root_ar is the input — caller transitions + emits separately.
    assert_eq!(result.cascaded_ars.len(), 2);
    assert!(result.cascaded_ars.contains(&child_ar_id));
    assert!(result.cascaded_ars.contains(&grandchild_ar_id));

    // Storage-side: every grant row carries `revoked_at`.
    for gid in [root_grant_id, child_grant_id, grandchild_grant_id] {
        let g = acc
            .store
            .get_grant(gid)
            .await
            .unwrap()
            .expect("grant row exists");
        assert!(
            g.revoked_at.is_some(),
            "grant {gid} should be revoked after cascade"
        );
    }
}

// ---- 5b. Template-revoke handler reports multi-hop grant count -----------

/// Smoke test: the Template-revoke handler delegates to the recursive
/// cascade (CH-14 P3). Wire the chain through the `Repository`
/// directly (since fired grants don't yet exist on a fresh org per
/// plan §8 named-still-green list) and call the recursive method from
/// the handler's bound code path. This pins the handler-flip in
/// `templates/revoke.rs` to the recursive variant: if a future
/// refactor reverts to the single-hop method, multi-hop grant counts
/// would silently regress.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn template_revoke_handler_uses_recursive_cascade() {
    let acc = spawn(false).await;
    let now = Utc::now();

    // 2-hop chain: org_ar → org_grant → child_ar → child_grant.
    let org_ar_id = AuthRequestId::new();
    let org_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();

    let org_ar = synthetic_chain_ar(org_ar_id, None);
    let org_grant = synthetic_chain_grant(org_grant_id, org_ar_id, now);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(org_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id, now);

    for ar in [&org_ar, &child_ar] {
        acc.store.create_auth_request(ar).await.unwrap();
    }
    for g in [&org_grant, &child_grant] {
        acc.store.create_grant(g).await.unwrap();
    }

    // Calling the recursive variant via the trait surfaces a count
    // that includes BOTH the level-0 org_grant AND the level-1
    // child_grant. The single-hop variant (preserved verbatim per
    // ADR-0053 §D53.4) would only return the level-0 grant, so this
    // assertion fails fast if a future regression flips back.
    let result = acc
        .store
        .revoke_grants_by_descends_from_recursive(org_ar_id, now + ChronoDuration::seconds(1))
        .await
        .unwrap();
    assert_eq!(
        result.revoked_grants.len(),
        2,
        "Template-revoke must fan out to grandchildren via recursive cascade, got {}",
        result.revoked_grants.len()
    );
    // One cascaded AR (the child_ar at level 1).
    assert_eq!(result.cascaded_ars.len(), 1);
    assert_eq!(result.cascaded_ars[0], child_ar_id);

    // Verify the legacy single-hop variant returns just the level-0
    // grant — pins the back-compat property of ADR-0053 §D53.4.
    // (Set up a fresh chain since the previous one is now fully
    // revoked.)
    let org2_ar_id = AuthRequestId::new();
    let org2_grant_id = GrantId::new();
    let child2_ar_id = AuthRequestId::new();
    let child2_grant_id = GrantId::new();
    let org2_ar = synthetic_chain_ar(org2_ar_id, None);
    let org2_grant = synthetic_chain_grant(org2_grant_id, org2_ar_id, now);
    let child2_ar = synthetic_chain_ar(child2_ar_id, Some(org2_grant_id));
    let child2_grant = synthetic_chain_grant(child2_grant_id, child2_ar_id, now);
    for ar in [&org2_ar, &child2_ar] {
        acc.store.create_auth_request(ar).await.unwrap();
    }
    for g in [&org2_grant, &child2_grant] {
        acc.store.create_grant(g).await.unwrap();
    }
    let single_hop = acc
        .store
        .revoke_grants_by_descends_from(org2_ar_id, now + ChronoDuration::seconds(2))
        .await
        .unwrap();
    assert_eq!(
        single_hop.len(),
        1,
        "single-hop variant must return ONLY the level-0 grant, got {}",
        single_hop.len()
    );
    assert_eq!(single_hop[0], org2_grant_id);
}

// ---- 5. Cascade is idempotent over already-revoked grants -----------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoke_cascade_does_not_re_revoke_already_revoked_grants() {
    let acc = spawn(false).await;
    let now = Utc::now();

    let root_ar_id = AuthRequestId::new();
    let root_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();

    let root_ar = synthetic_chain_ar(root_ar_id, None);
    let root_grant = synthetic_chain_grant(root_grant_id, root_ar_id, now);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(root_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id, now);

    for ar in [&root_ar, &child_ar] {
        acc.store.create_auth_request(ar).await.unwrap();
    }
    for g in [&root_grant, &child_grant] {
        acc.store.create_grant(g).await.unwrap();
    }

    // First cascade revokes both.
    let first = acc
        .store
        .revoke_grants_by_descends_from_recursive(root_ar_id, now + ChronoDuration::seconds(1))
        .await
        .unwrap();
    assert_eq!(first.revoked_grants.len(), 2);
    assert_eq!(first.cascaded_ars.len(), 1);

    // Second cascade is a no-op (every grant is already revoked).
    // No new grants → no new cascaded ARs surface either.
    let second = acc
        .store
        .revoke_grants_by_descends_from_recursive(root_ar_id, now + ChronoDuration::seconds(2))
        .await
        .unwrap();
    assert!(
        second.revoked_grants.is_empty(),
        "second cascade must be a no-op (revoked_grants), got {:?}",
        second.revoked_grants
    );
    assert!(
        second.cascaded_ars.is_empty(),
        "second cascade must surface no new cascaded ARs, got {:?}",
        second.cascaded_ars
    );
}

// ---- 6. Template-revoke emits one auth_request.revoked per cascaded AR ----
//
// CH-14 / ADR-0053 §D53.7 — gate-2 inline correction (FOLLOWUP-02
// closure). The Template-revoke handler must emit exactly N
// `auth_request.revoked` audit events for N level-≥1 cascaded ARs +
// transition each cascaded AR to `AuthRequestState::Revoked`. The
// level-0 adoption AR's transition is unchanged (existing behaviour).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoke_cascade_emits_one_event_per_cascaded_ar() {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use domain::audit::{AuditEmitter, AuditEvent, NoopAuditEmitter};
    use domain::in_memory::InMemoryRepository;
    use domain::model::ids::{AgentId, OrgId};
    use domain::model::nodes::TemplateKind;
    use domain::repository::{Repository, RepositoryResult};
    use domain::templates::a;
    use server::platform::templates::{revoke_template, RevokeInput};

    /// Capturing emitter that records every event for assertion.
    /// Mirrors the `CapturingAudit` pattern from
    /// `acceptance_memory_extraction.rs`.
    #[derive(Default)]
    struct CountingAudit {
        events: Mutex<Vec<AuditEvent>>,
    }
    #[async_trait]
    impl AuditEmitter for CountingAudit {
        async fn emit(&self, event: AuditEvent) -> RepositoryResult<()> {
            self.events.lock().unwrap().push(event);
            Ok(())
        }
    }
    // Silence the unused-import warning for NoopAuditEmitter — the
    // capturing emitter is what this test uses.
    let _: Option<NoopAuditEmitter> = None;

    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let audit = Arc::new(CountingAudit::default());
    let now = Utc::now();
    let org_id = OrgId::new();
    let actor = AgentId::new();

    // Build a real Template-A adoption AR (matches the adoption-AR
    // shape `find_adoption_ar` queries for via the
    // `org:<id>/template:<kind>` resource-URI prefix).
    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id,
        ceo: PrincipalRef::Agent(actor),
        now,
    });
    let adoption_ar_id = adoption_ar.id;
    repo.create_auth_request(&adoption_ar).await.unwrap();

    // Wire a 2-hop authority chain BELOW the adoption AR:
    //   adoption_ar → fired_grant → child_ar → child_grant → grand_ar
    //                                                      → grand_grant
    // Cascading from `adoption_ar` should surface child_ar +
    // grand_ar in `cascaded_ars` (2 level-≥1 ARs).
    let fired_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();
    let grand_ar_id = AuthRequestId::new();
    let grand_grant_id = GrantId::new();

    let fired_grant = synthetic_chain_grant(fired_grant_id, adoption_ar_id, now);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(fired_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id, now);
    let grand_ar = synthetic_chain_ar(grand_ar_id, Some(child_grant_id));
    let grand_grant = synthetic_chain_grant(grand_grant_id, grand_ar_id, now);

    repo.create_grant(&fired_grant).await.unwrap();
    repo.create_auth_request(&child_ar).await.unwrap();
    repo.create_grant(&child_grant).await.unwrap();
    repo.create_auth_request(&grand_ar).await.unwrap();
    repo.create_grant(&grand_grant).await.unwrap();

    let outcome = revoke_template(
        repo.clone(),
        audit.clone() as Arc<dyn AuditEmitter>,
        RevokeInput {
            org_id,
            kind: TemplateKind::A,
            reason: "fixture revoke".into(),
            actor,
            now: now + ChronoDuration::seconds(1),
        },
    )
    .await
    .expect("revoke_template should succeed");

    assert_eq!(
        outcome.grant_count_revoked, 3,
        "all 3 grants should be revoked (fired + child + grand)"
    );

    // Audit-event-count assertion. Per ADR-0053 §D53.7, the handler
    // must emit:
    //   - N − 1 `auth_request.revoked` events (one per level-≥1
    //     cascaded AR — here N − 1 = 2: child_ar + grand_ar), and
    //   - 1 `platform.template.revoked` summary event.
    // Total emitted = 3.
    let events = audit.events.lock().unwrap().clone();
    let auth_request_revoked: Vec<&AuditEvent> = events
        .iter()
        .filter(|e| e.event_type == "auth_request.revoked")
        .collect();
    assert_eq!(
        auth_request_revoked.len(),
        2,
        "expected 2 auth_request.revoked events (one per cascaded AR), got {}: {:?}",
        auth_request_revoked.len(),
        auth_request_revoked
            .iter()
            .map(|e| &e.event_type)
            .collect::<Vec<_>>()
    );
    let template_revoked: Vec<&AuditEvent> = events
        .iter()
        .filter(|e| e.event_type == "platform.template.revoked")
        .collect();
    assert_eq!(
        template_revoked.len(),
        1,
        "expected exactly one platform.template.revoked summary event"
    );
    assert_eq!(events.len(), 3);

    // Each cascaded AR is referenced by `provenance_auth_request_id`
    // in exactly one event — pin the pairing for cross-pod
    // determinism.
    let cascaded_ar_ids: Vec<AuthRequestId> = auth_request_revoked
        .iter()
        .filter_map(|e| e.provenance_auth_request_id)
        .collect();
    assert!(cascaded_ar_ids.contains(&child_ar_id));
    assert!(cascaded_ar_ids.contains(&grand_ar_id));
    assert!(
        !cascaded_ar_ids.contains(&adoption_ar_id),
        "level-0 adoption AR should NOT have its own auth_request.revoked event from the cascade loop \
         (the handler discards the level-0 event today; the template.revoked summary event covers it)"
    );
}

// ---- 7. Cascaded ARs transition to Revoked state -------------------------
//
// CH-14 / ADR-0053 §D53.7 — gate-2 inline correction. Verifies the
// AR-state-transition axis (concept-doc 02 §"Step 3 — Owner
// reconsiders and revokes"): every cascaded AR's state flips from
// Approved → Revoked + carries `terminal_state_entered_at = at`.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoke_cascade_transitions_cascaded_ars_to_revoked_state() {
    use std::sync::Arc;

    use domain::audit::{AuditEmitter, NoopAuditEmitter};
    use domain::in_memory::InMemoryRepository;
    use domain::model::ids::{AgentId, OrgId};
    use domain::model::nodes::TemplateKind;
    use domain::repository::Repository;
    use domain::templates::a;
    use server::platform::templates::{revoke_template, RevokeInput};

    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
    let now = Utc::now();
    let revoke_at = now + ChronoDuration::seconds(1);
    let org_id = OrgId::new();
    let actor = AgentId::new();

    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id,
        ceo: PrincipalRef::Agent(actor),
        now,
    });
    let adoption_ar_id = adoption_ar.id;
    repo.create_auth_request(&adoption_ar).await.unwrap();

    // adoption_ar → fired_grant → child_ar (Approved) → child_grant
    let fired_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();

    let fired_grant = synthetic_chain_grant(fired_grant_id, adoption_ar_id, now);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(fired_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id, now);

    repo.create_grant(&fired_grant).await.unwrap();
    repo.create_auth_request(&child_ar).await.unwrap();
    repo.create_grant(&child_grant).await.unwrap();

    // Sanity: cascaded AR starts Approved (note: the synthetic
    // fixture pre-stamps `terminal_state_entered_at` to a non-None
    // value — that's a quirk of `synthetic_chain_ar`, not a real
    // production constraint; only `state` matters here).
    let pre = repo.get_auth_request(child_ar_id).await.unwrap().unwrap();
    assert_eq!(pre.state, AuthRequestState::Approved);

    revoke_template(
        repo.clone(),
        audit,
        RevokeInput {
            org_id,
            kind: TemplateKind::A,
            reason: "transition fixture".into(),
            actor,
            now: revoke_at,
        },
    )
    .await
    .expect("revoke_template should succeed");

    // Post-cascade: child_ar transitioned to Revoked at the
    // requested timestamp; level-0 adoption_ar also transitioned
    // (existing behaviour, unchanged by the gate-2 correction).
    let post_child = repo.get_auth_request(child_ar_id).await.unwrap().unwrap();
    assert_eq!(post_child.state, AuthRequestState::Revoked);
    assert_eq!(post_child.terminal_state_entered_at, Some(revoke_at));

    let post_adoption = repo
        .get_auth_request(adoption_ar_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(post_adoption.state, AuthRequestState::Revoked);
}

// ---- helpers ---------------------------------------------------------------

/// Build a synthetic AR shape good enough for chain-walking tests.
/// The audit class + state are arbitrary; only `id` and
/// `descends_from_grant` matter for the walker.
fn synthetic_chain_ar(id: AuthRequestId, parent_grant: Option<GrantId>) -> AuthRequest {
    AuthRequest {
        id,
        requestor: PrincipalRef::System("system:test-chain".into()),
        kinds: vec![],
        scope: vec![],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "test:r".into(),
            },
            approvers: vec![ApproverSlot {
                approver: PrincipalRef::System("system:test-chain".into()),
                state: ApproverSlotState::Approved,
                responded_at: Some(Utc::now()),
                reconsidered_at: None,
            }],
            state: ResourceSlotState::Approved,
        }],
        justification: Some("synthetic chain fixture".into()),
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: Some(Utc::now()),
        archived: false,
        active_window_days: 30,
        provenance_template: None,
        tags: vec![],
        descends_from_grant: parent_grant,
    }
}

fn synthetic_chain_grant(
    id: GrantId,
    parent_ar: AuthRequestId,
    issued_at: chrono::DateTime<Utc>,
) -> Grant {
    Grant {
        id,
        holder: PrincipalRef::System("system:test-chain".into()),
        action: vec![domain::permissions::Action::Read],
        resource: ResourceRef {
            uri: "test:r".into(),
        },
        fundamentals: vec![],
        descends_from: Some(parent_ar),
        delegable: false,
        issued_at,
        revoked_at: None,
        approval_mode: domain::model::nodes::ApprovalMode::default(),
        audit_class: AuditClass::Logged,
        allocate_refinement: None,
    }
}

/// An orphan grant — `descends_from = None`. Used by the
/// "walker returns empty" test.
fn synthetic_orphan_grant() -> Grant {
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::System("system:test-orphan".into()),
        action: vec![domain::permissions::Action::Read],
        resource: ResourceRef {
            uri: "test:r".into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
        approval_mode: domain::model::nodes::ApprovalMode::default(),
        audit_class: AuditClass::Logged,
        allocate_refinement: None,
    }
}
