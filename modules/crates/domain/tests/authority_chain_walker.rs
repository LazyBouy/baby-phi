//! Unit tests for the authority-chain walker against
//! `InMemoryRepository`.
//!
//! Pairs with `acceptance_authority_chain.rs` (server crate, E2E
//! against a live SurrealDB) — these in-memory tests exercise the
//! BFS / chain-walk loops without standing up a real DB. Per CH-14
//! plan §8 P2 walker-unit-tests bullet.

use chrono::Utc;
use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AuthRequestId, GrantId, TemplateId};
use domain::model::nodes::{
    ApprovalMode, ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, Grant,
    PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState,
};
use domain::permissions::axioms::{
    is_bootstrap_ar, system_bootstrap_template_id, system_genesis_principal,
};
use domain::Repository;

fn bootstrap_shaped_ar(id: AuthRequestId) -> AuthRequest {
    AuthRequest {
        id,
        requestor: system_genesis_principal(),
        kinds: vec!["control_plane_object".into()],
        scope: vec!["allocate".into()],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "system:root".into(),
            },
            approvers: vec![ApproverSlot {
                approver: system_genesis_principal(),
                state: ApproverSlotState::Approved,
                responded_at: Some(Utc::now()),
                reconsidered_at: None,
            }],
            state: ResourceSlotState::Approved,
        }],
        justification: Some("bootstrap".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: Some(Utc::now()),
        archived: false,
        active_window_days: 3650,
        provenance_template: Some(system_bootstrap_template_id()),
        tags: vec![],
        descends_from_grant: None,
    }
}

fn synthetic_chain_ar(id: AuthRequestId, parent_grant: Option<GrantId>) -> AuthRequest {
    AuthRequest {
        id,
        requestor: PrincipalRef::System("system:test-walker".into()),
        kinds: vec![],
        scope: vec![],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![],
        justification: None,
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: Some(Utc::now()),
        archived: false,
        active_window_days: 30,
        provenance_template: None,
        tags: vec![],
        descends_from_grant: parent_grant,
    }
}

fn synthetic_chain_grant(id: GrantId, parent_ar: AuthRequestId) -> Grant {
    Grant {
        id,
        holder: PrincipalRef::System("system:test-walker".into()),
        action: vec![domain::permissions::Action::Read],
        resource: ResourceRef {
            uri: "test:r".into(),
        },
        fundamentals: vec![],
        descends_from: Some(parent_ar),
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
        approval_mode: ApprovalMode::default(),
        audit_class: AuditClass::Logged,
        allocate_refinement: None,
    }
}

#[tokio::test]
async fn walker_returns_single_element_chain_for_bootstrap_grant() {
    let repo = InMemoryRepository::new();
    let bootstrap_ar_id = AuthRequestId::new();
    let bootstrap_ar = bootstrap_shaped_ar(bootstrap_ar_id);
    repo.create_auth_request(&bootstrap_ar).await.unwrap();

    let grant_id = GrantId::new();
    let grant = synthetic_chain_grant(grant_id, bootstrap_ar_id);
    repo.create_grant(&grant).await.unwrap();

    let chain = repo.walk_provenance_chain(grant_id).await.unwrap();
    assert_eq!(chain.len(), 1);
    assert!(is_bootstrap_ar(&chain[0]));
}

#[tokio::test]
async fn walker_returns_empty_for_orphan_grant() {
    let repo = InMemoryRepository::new();
    let grant_id = GrantId::new();
    let grant = Grant {
        descends_from: None,
        ..synthetic_chain_grant(grant_id, AuthRequestId::new())
    };
    repo.create_grant(&grant).await.unwrap();
    let chain = repo.walk_provenance_chain(grant_id).await.unwrap();
    assert!(chain.is_empty());
}

#[tokio::test]
async fn walker_climbs_two_hop_chain_root_to_leaf() {
    let repo = InMemoryRepository::new();
    // Bootstrap-AR ← bootstrap-grant ← child-AR ← child-grant
    let bootstrap_ar_id = AuthRequestId::new();
    let bootstrap_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();

    let bootstrap_ar = bootstrap_shaped_ar(bootstrap_ar_id);
    let bootstrap_grant = synthetic_chain_grant(bootstrap_grant_id, bootstrap_ar_id);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(bootstrap_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id);

    repo.create_auth_request(&bootstrap_ar).await.unwrap();
    repo.create_auth_request(&child_ar).await.unwrap();
    repo.create_grant(&bootstrap_grant).await.unwrap();
    repo.create_grant(&child_grant).await.unwrap();

    let chain = repo.walk_provenance_chain(child_grant_id).await.unwrap();
    assert_eq!(chain.len(), 2);
    // Index 0 is the root (bootstrap), last is the leaf (direct AR).
    assert!(is_bootstrap_ar(&chain[0]));
    assert_eq!(chain[1].id, child_ar_id);
}

#[tokio::test]
async fn walker_returns_err_at_depth_cap() {
    let repo = InMemoryRepository::new();
    // 33 hops without termination → MAX_PROVENANCE_DEPTH = 32 trips.
    let mut ars: Vec<AuthRequest> = Vec::with_capacity(33);
    let mut grants: Vec<Grant> = Vec::with_capacity(33);
    let mut prev_grant: Option<GrantId> = None;
    for _ in 0..33 {
        let ar_id = AuthRequestId::new();
        let ar = synthetic_chain_ar(ar_id, prev_grant);
        let grant_id = GrantId::new();
        let grant = synthetic_chain_grant(grant_id, ar_id);
        prev_grant = Some(grant_id);
        ars.push(ar);
        grants.push(grant);
    }
    for ar in &ars {
        repo.create_auth_request(ar).await.unwrap();
    }
    for g in &grants {
        repo.create_grant(g).await.unwrap();
    }
    let leaf = grants.last().unwrap().id;
    let result = repo.walk_provenance_chain(leaf).await;
    assert!(matches!(
        result,
        Err(domain::repository::RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: 32 })
    ));
}

// ---- recursive cascade unit tests -----------------------------------------

#[tokio::test]
async fn recursive_cascade_returns_empty_for_ar_with_no_children() {
    let repo = InMemoryRepository::new();
    let ar_id = AuthRequestId::new();
    let ar = synthetic_chain_ar(ar_id, None);
    repo.create_auth_request(&ar).await.unwrap();

    let result = repo
        .revoke_grants_by_descends_from_recursive(ar_id, Utc::now())
        .await
        .unwrap();
    assert!(result.revoked_grants.is_empty());
    assert!(result.cascaded_ars.is_empty());
}

#[tokio::test]
async fn recursive_cascade_walks_3_hop_chain() {
    let repo = InMemoryRepository::new();
    let now = Utc::now();
    // root_ar → root_grant → child_ar → child_grant → grand_ar → grand_grant.
    let root_ar_id = AuthRequestId::new();
    let root_grant_id = GrantId::new();
    let child_ar_id = AuthRequestId::new();
    let child_grant_id = GrantId::new();
    let grand_ar_id = AuthRequestId::new();
    let grand_grant_id = GrantId::new();

    let root_ar = synthetic_chain_ar(root_ar_id, None);
    let root_grant = synthetic_chain_grant(root_grant_id, root_ar_id);
    let child_ar = synthetic_chain_ar(child_ar_id, Some(root_grant_id));
    let child_grant = synthetic_chain_grant(child_grant_id, child_ar_id);
    let grand_ar = synthetic_chain_ar(grand_ar_id, Some(child_grant_id));
    let grand_grant = synthetic_chain_grant(grand_grant_id, grand_ar_id);

    for ar in [&root_ar, &child_ar, &grand_ar] {
        repo.create_auth_request(ar).await.unwrap();
    }
    for g in [&root_grant, &child_grant, &grand_grant] {
        repo.create_grant(g).await.unwrap();
    }
    let result = repo
        .revoke_grants_by_descends_from_recursive(root_ar_id, now)
        .await
        .unwrap();
    assert_eq!(result.revoked_grants.len(), 3);
    for gid in [root_grant_id, child_grant_id, grand_grant_id] {
        assert!(result.revoked_grants.contains(&gid));
        let g = repo.get_grant(gid).await.unwrap().unwrap();
        assert_eq!(g.revoked_at, Some(now));
    }
    // Two cascaded ARs (level-1 child_ar + level-2 grand_ar). The
    // level-0 root_ar is the input — handled by the caller.
    assert_eq!(result.cascaded_ars.len(), 2);
    assert!(result.cascaded_ars.contains(&child_ar_id));
    assert!(result.cascaded_ars.contains(&grand_ar_id));
    assert!(!result.cascaded_ars.contains(&root_ar_id));
}

// ---- predicate negative cases -----------------------------------------------

#[tokio::test]
async fn is_bootstrap_ar_rejects_template_ar_with_zero_uuid_but_wrong_requestor() {
    let mut ar = bootstrap_shaped_ar(AuthRequestId::new());
    // Zero-UUID template id stays, but requestor flips to a Template
    // AR's typical requestor (an Agent). Predicate must reject.
    ar.requestor = PrincipalRef::Agent(domain::model::ids::AgentId::new());
    assert!(!is_bootstrap_ar(&ar));
}

#[tokio::test]
async fn is_bootstrap_ar_rejects_genesis_requestor_with_non_zero_template_uuid() {
    let mut ar = bootstrap_shaped_ar(AuthRequestId::new());
    ar.provenance_template = Some(TemplateId::new());
    assert!(!is_bootstrap_ar(&ar));
}
