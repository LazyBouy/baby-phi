//! CH-10 / ADR-0047 — Repository tests for the 5 per-transition Consent
//! methods on the in-memory backend (D-new-05 closure).
//!
//! Covered:
//! - Happy-path: each of the 5 transitions persists the new state +
//!   timestamp field + audit event.
//! - Illegal-transition rejection on the wrong source state.
//! - Forward-only revocation: a `Revoked` consent rejects all transitions.
//! - `revocable = false` blocks `revoke_consent` even from a legal state.
//! - Audit-event content: org_scope + from/to states + timestamp fields.
//!
//! Store-parity coverage for the SurrealDB backend lives in
//! `store/tests/repository_test.rs` (CH-10 section).

use chrono::{Duration, Utc};

use domain::audit::AuditClass;
use domain::consents::ConsentTransitionError;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, ConsentId, OrgId};
use domain::model::nodes::{Consent, ConsentScope, ConsentState};
use domain::repository::{Repository, RepositoryError};

fn fresh_consent(state: ConsentState, revocable: bool) -> Consent {
    let now = Utc::now();
    Consent {
        id: ConsentId::new(),
        agent_id: AgentId::new(),
        scope: ConsentScope {
            org: OrgId::new(),
            templates: vec![],
            actions: vec![],
        },
        state,
        requested_at: now,
        responded_at: None,
        revoked_at: None,
        revocable,
        provenance: "agent:test@unit".into(),
        deadline_at: None,
    }
}

async fn seed(repo: &InMemoryRepository, c: &Consent) {
    repo.create_consent(c).await.expect("seed consent");
}

// ---- acknowledge_consent ---------------------------------------------------

#[tokio::test]
async fn acknowledge_consent_happy_path_returns_audit_id_and_emits_audit() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Requested, true);
    seed(&repo, &c).await;
    let actor = AgentId::new();
    let at = Utc::now();

    let audit_id = repo
        .acknowledge_consent(c.id, at, actor)
        .await
        .expect("acknowledge succeeds");

    let events = repo
        .list_recent_audit_events_for_org(c.scope.org, 100)
        .await
        .unwrap();
    let ev = events
        .iter()
        .find(|e| e.event_id == audit_id)
        .expect("audit event landed");
    assert_eq!(ev.event_type, "consent.acknowledged");
    assert_eq!(ev.audit_class, AuditClass::Logged);
    assert_eq!(ev.actor_agent_id, Some(actor));
}

#[tokio::test]
async fn acknowledge_consent_from_terminal_state_returns_consent_transition_error() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Declined, true);
    seed(&repo, &c).await;

    let err = repo
        .acknowledge_consent(c.id, Utc::now(), AgentId::new())
        .await
        .expect_err("declined→acknowledged must reject");
    match err {
        RepositoryError::ConsentTransition {
            source: ConsentTransitionError::IllegalTransition { from, to },
        } => {
            assert_eq!(from, ConsentState::Declined);
            assert_eq!(to, ConsentState::Acknowledged);
        }
        other => panic!("expected ConsentTransition::IllegalTransition, got {other:?}"),
    }
}

// ---- decline_consent -------------------------------------------------------

#[tokio::test]
async fn decline_consent_happy_path_emits_audit() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Requested, true);
    seed(&repo, &c).await;

    let _audit_id = repo
        .decline_consent(c.id, Utc::now(), AgentId::new())
        .await
        .expect("decline succeeds");

    let events = repo
        .list_recent_audit_events_for_org(c.scope.org, 100)
        .await
        .unwrap();
    assert!(
        events.iter().any(|e| e.event_type == "consent.declined"),
        "consent.declined audit must be present"
    );
}

// ---- revoke_consent --------------------------------------------------------

#[tokio::test]
async fn revoke_consent_happy_path_revocable_emits_audit() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Acknowledged, true);
    seed(&repo, &c).await;

    let _audit_id = repo
        .revoke_consent(c.id, Utc::now(), AgentId::new())
        .await
        .expect("revoke succeeds");

    let events = repo
        .list_recent_audit_events_for_org(c.scope.org, 100)
        .await
        .unwrap();
    let ev = events
        .iter()
        .find(|e| e.event_type == "consent.revoked")
        .expect("revoked audit");
    assert_eq!(ev.diff["after"]["from_state"], "acknowledged");
    assert_eq!(ev.diff["after"]["to_state"], "revoked");
    assert!(ev.diff["after"]["revoked_at"].is_string());
}

#[tokio::test]
async fn revoke_consent_non_revocable_returns_not_revocable_error() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Acknowledged, false);
    seed(&repo, &c).await;

    let err = repo
        .revoke_consent(c.id, Utc::now(), AgentId::new())
        .await
        .expect_err("revoke on non-revocable consent must reject");
    match err {
        RepositoryError::ConsentTransition {
            source: ConsentTransitionError::NotRevocable { consent_id },
        } => {
            assert_eq!(consent_id, c.id);
        }
        other => panic!("expected NotRevocable, got {other:?}"),
    }
}

#[tokio::test]
async fn revoke_consent_from_revoked_is_illegal_forward_only() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Revoked, true);
    seed(&repo, &c).await;

    let err = repo
        .revoke_consent(c.id, Utc::now(), AgentId::new())
        .await
        .expect_err("re-revoke must reject (forward-only)");
    assert!(matches!(
        err,
        RepositoryError::ConsentTransition {
            source: ConsentTransitionError::IllegalTransition { .. }
        }
    ));
}

// ---- mark_consent_timed_out ------------------------------------------------

#[tokio::test]
async fn mark_consent_timed_out_from_requested_emits_audit() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Requested, true);
    seed(&repo, &c).await;

    let _audit_id = repo
        .mark_consent_timed_out(c.id, Utc::now(), AgentId::new())
        .await
        .expect("timed_out succeeds");

    let events = repo
        .list_recent_audit_events_for_org(c.scope.org, 100)
        .await
        .unwrap();
    let ev = events
        .iter()
        .find(|e| e.event_type == "consent.timed_out")
        .expect("timed_out audit");
    assert_eq!(ev.diff["after"]["from_state"], "requested");
    assert_eq!(ev.diff["after"]["to_state"], "timed_out");
}

// ---- mark_consent_expired --------------------------------------------------

#[tokio::test]
async fn mark_consent_expired_from_acknowledged_records_caller_supplied_from_state() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::Acknowledged, true);
    seed(&repo, &c).await;

    let _audit_id = repo
        .mark_consent_expired(c.id, Utc::now(), AgentId::new())
        .await
        .expect("expire succeeds");

    let events = repo
        .list_recent_audit_events_for_org(c.scope.org, 100)
        .await
        .unwrap();
    let ev = events
        .iter()
        .find(|e| e.event_type == "consent.expired")
        .expect("expired audit");
    assert_eq!(ev.diff["after"]["from_state"], "acknowledged");
    assert_eq!(ev.diff["after"]["to_state"], "expired");
}

#[tokio::test]
async fn mark_consent_expired_from_terminal_is_illegal() {
    let repo = InMemoryRepository::new();
    let c = fresh_consent(ConsentState::TimedOut, true);
    seed(&repo, &c).await;

    let err = repo
        .mark_consent_expired(c.id, Utc::now(), AgentId::new())
        .await
        .expect_err("expire on terminal must reject");
    assert!(matches!(
        err,
        RepositoryError::ConsentTransition {
            source: ConsentTransitionError::IllegalTransition { .. }
        }
    ));
}

// ---- not_found path --------------------------------------------------------

#[tokio::test]
async fn acknowledge_consent_on_missing_id_returns_not_found() {
    let repo = InMemoryRepository::new();
    let err = repo
        .acknowledge_consent(ConsentId::new(), Utc::now(), AgentId::new())
        .await
        .expect_err("missing consent must reject");
    assert!(matches!(err, RepositoryError::NotFound));
}

// ---- sweeper ---------------------------------------------------------------

#[tokio::test]
async fn sweep_consent_timeouts_flips_eligible_rows_and_returns_ids() {
    let repo = InMemoryRepository::new();
    let now = Utc::now();
    // Eligible: Requested with deadline_at in the past.
    let mut eligible = fresh_consent(ConsentState::Requested, true);
    eligible.deadline_at = Some(now - Duration::seconds(60));
    seed(&repo, &eligible).await;
    // Ineligible: deadline in the future.
    let mut future = fresh_consent(ConsentState::Requested, true);
    future.deadline_at = Some(now + Duration::seconds(3600));
    seed(&repo, &future).await;
    // Ineligible: no deadline.
    let no_deadline = fresh_consent(ConsentState::Requested, true);
    seed(&repo, &no_deadline).await;
    // Ineligible: not Requested.
    let mut ack = fresh_consent(ConsentState::Acknowledged, true);
    ack.deadline_at = Some(now - Duration::seconds(60));
    seed(&repo, &ack).await;

    let flipped = repo.sweep_consent_timeouts(now).await.unwrap();
    assert_eq!(flipped.len(), 1);
    assert_eq!(flipped[0], eligible.id);
}

#[tokio::test]
async fn sweep_consent_timeouts_re_run_returns_empty_when_no_more_eligible() {
    let repo = InMemoryRepository::new();
    let now = Utc::now();
    let mut c = fresh_consent(ConsentState::Requested, true);
    c.deadline_at = Some(now - Duration::seconds(60));
    seed(&repo, &c).await;

    let first = repo.sweep_consent_timeouts(now).await.unwrap();
    assert_eq!(first.len(), 1);
    let second = repo.sweep_consent_timeouts(now).await.unwrap();
    assert!(
        second.is_empty(),
        "no rows should remain eligible after first sweep"
    );
}
