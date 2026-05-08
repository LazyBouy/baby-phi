//! CH-08 / ADR-0052 §D52.5 — atomicity tests for
//! `Repository::apply_transfer_grant` on `SurrealStore` (in-memory mode).
//!
//! Concept-doc 02 line 206 verbatim: *"on approval, rewrites the OWNED_BY
//! edge and revokes any residual authority the sender held through
//! ownership"*. These tests pin the three-write atomicity guarantee on
//! the production storage adapter using SurrealDB's `BEGIN TRANSACTION
//! ... COMMIT TRANSACTION` block.
//!
//! Forward-defensive (F5.A user-locked at plan approval): no production
//! caller wires this method today.

use chrono::{TimeZone, Utc};
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, GrantId, NodeId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::Principal;
use domain::permissions::Action;
use domain::repository::{Repository, RepositoryError, TransferGrantPayload};
use store::SurrealStore;
use tempfile::TempDir;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

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

async fn seed_owned_by(store: &SurrealStore, resource: NodeId, owner: NodeId) {
    Repository::upsert_ownership_raw(store, resource, owner, None)
        .await
        .expect("seed OWNED_BY edge");
}

#[tokio::test]
async fn transfer_grant_atomic_round_trip_surreal() {
    // Happy path: the three writes (OWNED_BY edge rewrite + sender
    // grant revocation + recipient grant mint) commit atomically under
    // SurrealDB's BEGIN/COMMIT.
    let (store, _dir) = fresh_store().await;
    let sender = AgentId::new();
    let recipient = AgentId::new();
    let resource_uri = "filesystem:/workspace/repo-x";

    // Pre-state: seed sender's grant + an OWNED_BY edge pointing at sender.
    let sender_grant = transfer_grant(PrincipalRef::Agent(sender), resource_uri);
    let sender_grant_id = sender_grant.id;
    store
        .create_grant(&sender_grant)
        .await
        .expect("seed sender");

    let resource_node = NodeId::new();
    seed_owned_by(&store, resource_node, sender.node_id()).await;

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
    store.apply_transfer_grant(&payload).await.expect("commit");

    // Post-state assertions:
    // (1) sender's grant is revoked at `payload.at`.
    let sender_after = store.get_grant(sender_grant_id).await.unwrap().unwrap();
    assert_eq!(
        sender_after.revoked_at,
        Some(ts(60)),
        "sender's grant must be revoked atomically"
    );

    // (2) recipient's grant is durable.
    let recipient_after = store.get_grant(recipient_grant_id).await.unwrap();
    assert!(
        recipient_after.is_some(),
        "recipient's grant must be minted durably"
    );
    let recipient_after = recipient_after.unwrap();
    assert!(matches!(recipient_after.holder, PrincipalRef::Agent(a) if a == recipient));
    assert_eq!(recipient_after.revoked_at, None);
    assert_eq!(recipient_after.resource.uri, resource_uri);

    // (3) The OWNED_BY edge for the resource now points at the recipient
    //     (the new owner). Probe via SurrealQL — this confirms the edge
    //     row's `out` got rewritten by the DELETE + RELATE pair inside
    //     the BEGIN/COMMIT.
    let recipient_id_str = recipient.to_string();
    let mut probe = store
        .client()
        .query(
            "SELECT VALUE record::id(out) AS owner_id FROM owned_by \
             WHERE record::id(out) = $owner LIMIT 1",
        )
        .bind(("owner", recipient_id_str.clone()))
        .await
        .unwrap();
    let owners: Vec<String> = probe.take(0).unwrap();
    assert_eq!(
        owners.first().map(String::as_str),
        Some(recipient_id_str.as_str()),
        "OWNED_BY edge must be rewritten to point at new_owner"
    );

    // The total count of owned_by edges must remain at 1 — DELETE +
    // RELATE under the BEGIN/COMMIT must keep cardinality intact.
    let mut count_probe = store
        .client()
        .query("SELECT count() FROM owned_by GROUP ALL")
        .await
        .unwrap();
    let counts: Vec<i64> = count_probe.take((0, "count")).unwrap();
    assert_eq!(
        counts.into_iter().next(),
        Some(1),
        "owned_by edge count must remain 1 after rewrite (atomicity invariant)"
    );
}

#[tokio::test]
async fn transfer_rolls_back_on_already_revoked_surreal() {
    // Atomicity: when sender's grant is already revoked, the compound
    // tx rejects with Conflict and NO partial state survives. The
    // recipient's grant must NOT exist; the OWNED_BY edge stays
    // pointing at sender.
    let (store, _dir) = fresh_store().await;
    let sender = AgentId::new();
    let recipient = AgentId::new();
    let resource_uri = "filesystem:/workspace/repo-y";

    let mut sender_grant = transfer_grant(PrincipalRef::Agent(sender), resource_uri);
    sender_grant.revoked_at = Some(ts(0));
    let sender_grant_id = sender_grant.id;
    store.create_grant(&sender_grant).await.expect("seed");

    let resource_node = NodeId::new();
    seed_owned_by(&store, resource_node, sender.node_id()).await;

    let recipient_grant = transfer_grant(PrincipalRef::Agent(recipient), resource_uri);
    let recipient_grant_id = recipient_grant.id;
    let payload = TransferGrantPayload {
        sender_grant_id,
        new_owner: PrincipalRef::Agent(recipient),
        recipient_grant,
        at: ts(60),
    };

    let result = store.apply_transfer_grant(&payload).await;
    match result {
        Err(RepositoryError::Conflict(msg)) => {
            assert!(
                msg.contains("already revoked"),
                "Conflict reason must mention re-entry safety: {msg}"
            );
        }
        other => panic!("expected Conflict, got {other:?}"),
    }

    // Post-state: recipient's grant must NOT be minted.
    let recipient_after = store.get_grant(recipient_grant_id).await.unwrap();
    assert!(
        recipient_after.is_none(),
        "recipient's grant must NOT be minted on rollback"
    );

    // Post-state: OWNED_BY edge still points at sender.
    let sender_id_str = sender.to_string();
    let mut probe = store
        .client()
        .query(
            "SELECT VALUE record::id(out) AS owner_id FROM owned_by \
             WHERE record::id(out) = $owner LIMIT 1",
        )
        .bind(("owner", sender_id_str.clone()))
        .await
        .unwrap();
    let owners: Vec<String> = probe.take(0).unwrap();
    assert_eq!(
        owners.first().map(String::as_str),
        Some(sender_id_str.as_str()),
        "OWNED_BY edge must NOT be rewritten on rollback"
    );
}
