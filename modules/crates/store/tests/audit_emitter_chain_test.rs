//! Hash-chain continuity test for `SurrealAuditEmitter`.
//!
//! Verifies M2 plan commitment C6:
//!   1. The first event within an org has `prev_event_hash == None`.
//!   2. Every subsequent event's `prev_event_hash` equals
//!      `hash_event(previous)`.
//!   3. Events in one org's chain DO NOT leak into another org's chain
//!      (per-org scoping).
//!
//! Uses real SurrealDB (tempdir RocksDB) so the test exercises the
//! exact path the production binary takes.

use std::sync::Arc;

use chrono::Utc;
use tempfile::tempdir;

use domain::audit::{hash_event, AuditClass, AuditEmitter, AuditEvent};
use domain::model::ids::{AuditEventId, OrgId};
use domain::Repository;
use store::{SurrealAuditEmitter, SurrealStore};

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

fn sample_event(org: Option<OrgId>, event_type: &str) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::json!({"n": event_type}),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: org,
        prev_event_hash: None, // emitter will populate
    }
}

#[tokio::test]
async fn first_event_for_org_has_no_prev_hash() {
    let (emitter, repo, _dir) = fresh_emitter().await;
    let org = Some(OrgId::new());

    let e1 = sample_event(org, "event.one");
    let id1 = e1.event_id;
    emitter.emit(e1).await.unwrap();

    let back = repo.get_audit_event(id1).await.unwrap().unwrap();
    assert!(
        back.prev_event_hash.is_none(),
        "first event must not carry a prev hash"
    );
}

#[tokio::test]
async fn second_event_links_to_first_event_hash() {
    let (emitter, repo, _dir) = fresh_emitter().await;
    let org = Some(OrgId::new());

    let e1 = sample_event(org, "event.one");
    let id1 = e1.event_id;
    emitter.emit(e1).await.unwrap();
    let stored1 = repo.get_audit_event(id1).await.unwrap().unwrap();
    let expected_prev = hash_event(&stored1);

    let e2 = sample_event(org, "event.two");
    let id2 = e2.event_id;
    emitter.emit(e2).await.unwrap();
    let stored2 = repo.get_audit_event(id2).await.unwrap().unwrap();

    assert_eq!(
        stored2.prev_event_hash,
        Some(expected_prev),
        "second event's prev_event_hash must equal hash(first)"
    );
}

#[tokio::test]
async fn three_event_chain_is_continuous() {
    let (emitter, repo, _dir) = fresh_emitter().await;
    let org = Some(OrgId::new());

    let e1 = sample_event(org, "event.one");
    let id1 = e1.event_id;
    emitter.emit(e1).await.unwrap();

    // Wait 1ms so the SurrealDB `timestamp DESC` ORDER BY tiebreaks
    // deterministically. In practice `timestamp` is the wall clock,
    // which may collide at ms resolution under fast tests.
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    let e2 = sample_event(org, "event.two");
    let id2 = e2.event_id;
    emitter.emit(e2).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    let e3 = sample_event(org, "event.three");
    let id3 = e3.event_id;
    emitter.emit(e3).await.unwrap();

    let s1 = repo.get_audit_event(id1).await.unwrap().unwrap();
    let s2 = repo.get_audit_event(id2).await.unwrap().unwrap();
    let s3 = repo.get_audit_event(id3).await.unwrap().unwrap();

    assert!(s1.prev_event_hash.is_none());
    assert_eq!(s2.prev_event_hash, Some(hash_event(&s1)));
    assert_eq!(s3.prev_event_hash, Some(hash_event(&s2)));
}

#[tokio::test]
async fn chains_are_scoped_per_org() {
    let (emitter, repo, _dir) = fresh_emitter().await;
    let org_a = Some(OrgId::new());
    let org_b = Some(OrgId::new());

    // Seed org_a with one event.
    let a1 = sample_event(org_a, "a.one");
    emitter.emit(a1).await.unwrap();

    // Emit into org_b — must see an empty chain (first-in-scope).
    let b1 = sample_event(org_b, "b.one");
    let id_b1 = b1.event_id;
    emitter.emit(b1).await.unwrap();
    let stored_b1 = repo.get_audit_event(id_b1).await.unwrap().unwrap();
    assert!(
        stored_b1.prev_event_hash.is_none(),
        "org_b's first event must not link to org_a's chain"
    );
}
