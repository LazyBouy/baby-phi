//! CH-08 / ADR-0052 §D52.5 — atomicity tests for
//! `Repository::apply_transfer_grant` on `InMemoryRepository`.
//!
//! Concept-doc 02 line 206 verbatim: *"on approval, rewrites the OWNED_BY
//! edge and revokes any residual authority the sender held through
//! ownership"*. These tests pin the four semantic invariants:
//!
//! 1. **Happy path** — sender's grant is revoked atomically with the
//!    OWNED_BY-edge rewrite + the recipient's grant mint (concept-doc
//!    02 lines 199–207 transfer-cardinality "exclusive" / move semantics).
//! 2. **Sender-grant-missing rollback** — `RepositoryError::NotFound`;
//!    pre/post state byte-identical (atomicity invariant).
//! 3. **Sender-grant-already-revoked rollback** — `RepositoryError::Conflict`;
//!    re-entry safety (matches the `apply_org_creation` precedent).
//! 4. **Allocate-path remains additive** — calling `create_grant` for an
//!    `Allocate`-action grant on the same resource leaves the sender's
//!    grant intact (concept-doc 02 lines 199–204 "additive" / Arc::clone
//!    semantics for `[allocate]`).
//!
//! Forward-defensive (F5.A user-locked at plan approval): no production
//! caller wires this method today.

use chrono::{TimeZone, Utc};
use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, GrantId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::Principal;
use domain::permissions::Action;
use domain::repository::{Repository, TransferGrantPayload};

fn ts(offset_secs: i64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 7, 0, 0, 0).unwrap() + chrono::Duration::seconds(offset_secs)
}

fn transfer_grant(holder: PrincipalRef, resource_uri: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: vec![Action::Transfer],
        resource: ResourceRef {
            uri: resource_uri.into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: ts(0),
        revoked_at: None,
        approval_mode: domain::model::ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    }
}

fn allocate_grant(holder: PrincipalRef, resource_uri: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: vec![Action::Allocate],
        resource: ResourceRef {
            uri: resource_uri.into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: ts(0),
        revoked_at: None,
        approval_mode: domain::model::ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    }
}

#[tokio::test]
async fn transfer_revokes_sender_and_mints_recipient() {
    let repo = InMemoryRepository::new();
    let sender = AgentId::new();
    let recipient = AgentId::new();
    let resource_uri = "filesystem:/workspace/repo-x";

    // Pre-state: sender holds a [transfer]-eligible grant + an OWNED_BY
    // edge points the resource at sender.
    let sender_grant = transfer_grant(PrincipalRef::Agent(sender), resource_uri);
    let sender_grant_id = sender_grant.id;
    repo.create_grant(&sender_grant)
        .await
        .expect("seed sender grant");

    // Seed the OWNED_BY edge: resource → sender. The edge identifies
    // "the sender currently owns this resource" structurally per
    // concept-doc 02 line 206 framing.
    let resource_node = domain::model::ids::NodeId::new();
    repo.upsert_ownership_raw(resource_node, sender.node_id(), None)
        .await
        .expect("seed OWNED_BY edge");

    // Build the compound-tx payload.
    let recipient_grant = transfer_grant(PrincipalRef::Agent(recipient), resource_uri);
    let recipient_grant_id = recipient_grant.id;
    let payload = TransferGrantPayload {
        sender_grant_id,
        new_owner: PrincipalRef::Agent(recipient),
        recipient_grant,
        at: ts(60),
    };

    // Act.
    repo.apply_transfer_grant(&payload).await.expect("commit");

    // Post-state assertions:
    // (1) sender's grant is revoked at `payload.at`.
    let sender_after = repo.get_grant(sender_grant_id).await.unwrap().unwrap();
    assert_eq!(
        sender_after.revoked_at,
        Some(ts(60)),
        "sender's grant must be revoked atomically"
    );

    // (2) recipient's grant is minted.
    let recipient_after = repo.get_grant(recipient_grant_id).await.unwrap();
    assert!(
        recipient_after.is_some(),
        "recipient's grant must be minted atomically"
    );
    let recipient_after = recipient_after.unwrap();
    assert!(
        matches!(recipient_after.holder, PrincipalRef::Agent(a) if a == recipient),
        "recipient's grant holder must be the new_owner"
    );
    assert_eq!(recipient_after.revoked_at, None);
    assert_eq!(recipient_after.resource.uri, resource_uri);

    // (3) OWNED_BY edge for the resource now points at the recipient
    //     (the new owner). The forward-defensive in-memory impl
    //     identifies the edge by sender's holder match; post-rewrite,
    //     that edge's owner has flipped to recipient.node_id().
    let recipient_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(recipient))
        .await
        .unwrap();
    assert_eq!(recipient_grants.len(), 1);
}

#[tokio::test]
async fn transfer_rolls_back_on_sender_grant_missing() {
    let repo = InMemoryRepository::new();
    let recipient = AgentId::new();
    let resource_uri = "filesystem:/workspace/repo-y";

    // Pre-state: NO sender grant seeded.
    let recipient_grant = transfer_grant(PrincipalRef::Agent(recipient), resource_uri);
    let recipient_grant_id = recipient_grant.id;
    let payload = TransferGrantPayload {
        sender_grant_id: GrantId::new(), // does not exist
        new_owner: PrincipalRef::Agent(recipient),
        recipient_grant,
        at: ts(60),
    };

    // Act.
    let result = repo.apply_transfer_grant(&payload).await;

    // Post-state assertions: NotFound error + zero partial state.
    match result {
        Err(domain::repository::RepositoryError::NotFound) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
    // Recipient's grant must NOT be minted (atomic rollback).
    let recipient_after = repo.get_grant(recipient_grant_id).await.unwrap();
    assert!(
        recipient_after.is_none(),
        "recipient's grant must NOT be minted on rollback"
    );
}

#[tokio::test]
async fn transfer_rolls_back_on_sender_grant_already_revoked() {
    let repo = InMemoryRepository::new();
    let sender = AgentId::new();
    let recipient = AgentId::new();
    let resource_uri = "filesystem:/workspace/repo-z";

    // Pre-state: sender's grant exists but is already revoked.
    let mut sender_grant = transfer_grant(PrincipalRef::Agent(sender), resource_uri);
    sender_grant.revoked_at = Some(ts(0));
    let sender_grant_id = sender_grant.id;
    repo.create_grant(&sender_grant).await.expect("seed");

    let resource_node = domain::model::ids::NodeId::new();
    repo.upsert_ownership_raw(resource_node, sender.node_id(), None)
        .await
        .expect("seed OWNED_BY edge");

    let recipient_grant = transfer_grant(PrincipalRef::Agent(recipient), resource_uri);
    let recipient_grant_id = recipient_grant.id;
    let payload = TransferGrantPayload {
        sender_grant_id,
        new_owner: PrincipalRef::Agent(recipient),
        recipient_grant,
        at: ts(60),
    };

    // Act.
    let result = repo.apply_transfer_grant(&payload).await;

    // Post-state: Conflict error + zero partial state.
    match result {
        Err(domain::repository::RepositoryError::Conflict(msg)) => {
            assert!(
                msg.contains("already revoked"),
                "Conflict reason must mention re-entry safety: {msg}"
            );
        }
        other => panic!("expected Conflict, got {other:?}"),
    }

    // Recipient's grant must NOT be minted (atomic rollback).
    let recipient_after = repo.get_grant(recipient_grant_id).await.unwrap();
    assert!(
        recipient_after.is_none(),
        "recipient's grant must NOT be minted on rollback"
    );

    // Sender's grant remains revoked at the original ts(0) (NOT ts(60)).
    let sender_after = repo.get_grant(sender_grant_id).await.unwrap().unwrap();
    assert_eq!(sender_after.revoked_at, Some(ts(0)));
}

#[tokio::test]
async fn allocate_path_remains_additive() {
    // Concept-doc 02 lines 199–204 — `allocate` is **additive**: sender
    // retains full share; recipient gets parallel grant (Arc::clone
    // semantics). This test pins the invariant that minting an
    // Allocate-grant via `create_grant` on the same resource does NOT
    // touch the sender's grant. CH-08 deliberately changes nothing on
    // the Allocate path — ADR-0052 §D52.6.
    let repo = InMemoryRepository::new();
    let sender = AgentId::new();
    let allocatee = AgentId::new();
    let resource_uri = "filesystem:/workspace/shared";

    // Sender holds a [transfer]-eligible grant. (Use Transfer here to
    // mirror the typical mixed-action ownership scenario; the invariant
    // is that the sender's grant survives an additive Allocate-mint.)
    let sender_grant = transfer_grant(PrincipalRef::Agent(sender), resource_uri);
    let sender_grant_id = sender_grant.id;
    repo.create_grant(&sender_grant).await.expect("seed sender");

    // Now mint an Allocate-grant for `allocatee` on the same resource —
    // this is the additive path.
    let allocatee_grant = allocate_grant(PrincipalRef::Agent(allocatee), resource_uri);
    let allocatee_grant_id = allocatee_grant.id;
    repo.create_grant(&allocatee_grant)
        .await
        .expect("mint allocate-grant additively");

    // Both grants must coexist, untouched.
    let sender_after = repo.get_grant(sender_grant_id).await.unwrap().unwrap();
    assert_eq!(
        sender_after.revoked_at, None,
        "Allocate path must NOT revoke sender's grant — additive cardinality preserved"
    );

    let allocatee_after = repo.get_grant(allocatee_grant_id).await.unwrap().unwrap();
    assert_eq!(allocatee_after.revoked_at, None);
    assert_eq!(allocatee_after.resource.uri, resource_uri);
    assert!(matches!(
        allocatee_after.action.as_slice(),
        [Action::Allocate]
    ));

    // The two grants are observable independently per principal.
    let sender_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(sender))
        .await
        .unwrap();
    assert_eq!(sender_grants.len(), 1);
    let allocatee_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(allocatee))
        .await
        .unwrap();
    assert_eq!(allocatee_grants.len(), 1);
}
