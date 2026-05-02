//! CH-09 / ADR-0045 — acceptance suite for the 11-field Consent node
//! shape (drift D-new-04 closure).
//!
//! Confirms a fully-populated Consent record persists cleanly via
//! `Repository::create_consent` against both backends — `InMemoryRepository`
//! and `SurrealStore::open_embedded` (the migration 0010 redefine).
//!
//! Today's `Repository` trait surface for Consent is a single
//! `create_consent(&Consent) -> RepositoryResult<()>` method. There are
//! no read or update methods at this chunk (CH-10 adds
//! `update_consent_state`; CH-11 adds the read methods). These tests
//! therefore confirm the create-path accepts the full shape; future
//! chunks add round-trip read assertions when those methods land.
//!
//! Cross-impl consistency: both backends accept identical inputs and
//! return `Ok(())`.

use std::sync::Arc;

use chrono::Utc;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, ConsentId, OrgId, TemplateId};
use domain::model::nodes::{Consent, ConsentScope, ConsentState};
use domain::permissions::Action;
use domain::repository::Repository;
use store::SurrealStore;

fn full_consent() -> Consent {
    let now = Utc::now();
    Consent {
        id: ConsentId::new(),
        agent_id: AgentId::new(),
        scope: ConsentScope {
            org: OrgId::new(),
            templates: vec![TemplateId::new()],
            actions: vec![Action::Read, Action::List],
        },
        state: ConsentState::Acknowledged,
        requested_at: now,
        responded_at: Some(now),
        revoked_at: None,
        revocable: true,
        provenance: "agent:test@acceptance".into(),
        deadline_at: None,
    }
}

#[tokio::test]
async fn in_memory_persists_full_eleven_field_consent() {
    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let consent = full_consent();
    repo.create_consent(&consent)
        .await
        .expect("InMemoryRepository accepts the full 11-field Consent shape");
}

#[tokio::test]
async fn surreal_persists_full_eleven_field_consent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded surreal");
    let consent = full_consent();
    store
        .create_consent(&consent)
        .await
        .expect("SurrealStore accepts the full 11-field Consent shape post-migration-0010");
}

#[tokio::test]
async fn cross_impl_consistency_both_backends_accept_identical_consent() {
    let consent = full_consent();

    let in_memory: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let in_memory_result = in_memory.create_consent(&consent).await;

    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded surreal");
    let surreal_result = store.create_consent(&consent).await;

    assert!(
        in_memory_result.is_ok() && surreal_result.is_ok(),
        "both backends must accept the same Consent shape: in_memory={:?} surreal={:?}",
        in_memory_result,
        surreal_result
    );
}
