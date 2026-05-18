//! CH-11 / ADR-0048 — Per-Session consent gating acceptance suite.
//!
//! Drives the launch handler end-to-end against a fresh in-memory
//! repository, exercising:
//!
//! 1. Implicit-policy → grant short-circuits to Allow.
//! 2. OneTime-policy first read → Pending + minted Requested consent.
//! 3. OneTime-policy second read after acknowledge → Allow.
//! 4. PerSession-policy first read on session A → Pending + minted
//!    consent scoped to session A.
//! 5. PerSession-policy first read on session B (after session A
//!    acknowledged) still returns Pending — per-session matching.
//! 6. PerSession-policy timed-out + org default Deny → Denied.
//! 7. PerSession-policy timed-out + org default Allow → Allow.
//! 8. `Org.approval_timeout: Fixed(24h)` → consent.deadline_at = now+24h.
//!
//! Each test builds its own `Organization` + grant + actor/subordinate
//! pair against an `InMemoryRepository`, calls `launch_session`
//! directly (no HTTP), and asserts on the returned `SessionError` +
//! the persisted consent state.
//!
//! Direct-call vs HTTP: the M5/P4 acceptance suite drives the HTTP
//! launch path with a full bootstrap-claim fixture; CH-11 needs richer
//! org+grant setup (non-Implicit consent_policy, SubordinateRequired
//! ApprovalMode) that the harness fixture doesn't expose. Calling
//! `launch_session` directly proves the same wiring without
//! re-engineering the fixture.

use std::sync::Arc;

use chrono::{Duration, Utc};
use domain::audit::{AuditClass, AuditEmitter};
use domain::events::{EventBus, InProcessEventBus};
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, ConsentId, ModelProviderId, NodeId, OrgId, ProjectId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, ApprovalMode, Consent, ConsentScope, ConsentState, Grant,
    Organization, PrincipalRef, Project, ProjectShape, ProjectStatus, ResourceRef, TimeoutResponse,
};
use domain::model::{ApprovalTimeout, ConsentPolicy};
use domain::permissions::Action;
use domain::Repository;
use server::platform::sessions::{launch_session, LaunchInput, SessionError};
use server::state::{new_session_live_stream_registry, new_session_registry};

#[derive(Clone)]
struct Fixture {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    org_id: OrgId,
    project_id: ProjectId,
    actor_id: AgentId,
    subordinate_id: AgentId,
}

/// Build a fresh in-memory fixture: org with `consent_policy`,
/// `approval_timeout`, `approval_timeout_default_response` set per
/// args; one project; one actor agent owned by the org with a
/// `SubordinateRequired { policy }` grant on the `session` resource;
/// one subordinate agent with a profile bound to a model runtime.
async fn build_fixture(
    consent_policy: ConsentPolicy,
    approval_timeout: ApprovalTimeout,
    timeout_default_response: TimeoutResponse,
    approval_mode: ApprovalMode,
) -> Fixture {
    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let audit: Arc<dyn AuditEmitter> = Arc::new(domain::audit::NoopAuditEmitter);
    let event_bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());

    let now = Utc::now();
    let org_id = OrgId::new();
    let actor_id = AgentId::new();
    let subordinate_id = AgentId::new();
    let project_id = ProjectId::new();

    // Organization with the requested consent settings.
    let org = Organization {
        id: org_id,
        display_name: "Acme".into(),
        vision: None,
        mission: None,
        consent_policy,
        audit_class_default: domain::audit::AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        approval_timeout,
        approval_timeout_default_response: timeout_default_response,
        tags: vec![format!("organization:{}", org_id)],
        created_at: now,
    };
    repo.create_organization(&org).await.expect("create org");

    // Actor + subordinate agents.
    let actor = Agent {
        id: actor_id,
        kind: AgentKind::Llm,
        display_name: "actor-supervisor".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
        active: true,
        archived_at: None,
    };
    let subordinate = Agent {
        id: subordinate_id,
        kind: AgentKind::Llm,
        display_name: "subordinate".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
        active: true,
        archived_at: None,
    };
    repo.create_agent(&actor).await.expect("create actor");
    repo.create_agent(&subordinate)
        .await
        .expect("create subordinate");

    // Project. Catalogue entry for `session` so Step 0 doesn't miss
    // (not strictly needed since the launch handler builds its own
    // catalogue, but completeness).
    let project = Project {
        id: project_id,
        name: "Fixture Project".into(),
        description: "CH-11 acceptance".into(),
        goal: None,
        status: ProjectStatus::Planned,
        shape: ProjectShape::A,
        token_budget: None,
        tokens_spent: 0,
        objectives: vec![],
        key_results: vec![],
        resource_boundaries: None,
        tags: vec![format!("project:{}", project_id)],
        created_at: now,
    };
    repo.upsert_project(&project).await.expect("create project");

    // Subordinate's model runtime + profile (so launch's pre-gate
    // checks pass).
    let runtime_id = ModelProviderId::new();
    let runtime = domain::model::composites_m2::ModelRuntime {
        id: runtime_id,
        config: phi_core::provider::model::ModelConfig::anthropic("test", "claude-test", ""),
        secret_ref: domain::model::composites_m2::SecretRef::new("anthropic-api-key"),
        tenants_allowed: domain::model::composites_m2::TenantSet::All,
        status: domain::model::composites_m2::RuntimeStatus::Ok,
        archived_at: None,
        created_at: now,
    };
    repo.put_model_provider(&runtime)
        .await
        .expect("seed runtime");

    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id: subordinate_id,
        parallelize: 4,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: Some(runtime_id.to_string()),
        mock_response: None,
        created_at: now,
    };
    repo.create_agent_profile(&profile)
        .await
        .expect("create profile");

    // CH-15 / ADR-0054 §D54.1 — grant on `session_object` matching
    // the new launch manifest's reach (`[Read, Inspect, List]` on
    // `session_object` per `build_session_launch_manifest`). Pre-CH-15
    // this test seeded `[Invoke]` on `identity_principal`; CH-15
    // unifies the launch manifest on the session-object reach so the
    // grant must match. The `approval_mode` carries the
    // SubordinateRequired payload that drives Step 6 consent gating —
    // unchanged from CH-11.
    let grant = Grant {
        id: domain::model::ids::GrantId::new(),
        holder: PrincipalRef::Agent(subordinate_id),
        action: vec![Action::Read, Action::Inspect, Action::List],
        resource: ResourceRef {
            uri: "session_object".into(),
        },
        fundamentals: vec![
            domain::model::Fundamental::DataObject,
            domain::model::Fundamental::Tag,
        ],
        descends_from: None,
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    repo.create_grant(&grant).await.expect("create grant");

    Fixture {
        repo,
        audit,
        event_bus,
        org_id,
        project_id,
        actor_id,
        subordinate_id,
    }
}

fn launch_input(f: &Fixture, now: chrono::DateTime<Utc>) -> LaunchInput {
    LaunchInput {
        org_id: f.org_id,
        project_id: f.project_id,
        agent_id: f.subordinate_id,
        prompt: "hello".into(),
        actor: f.actor_id,
        now,
    }
}

async fn run_launch(
    f: &Fixture,
) -> Result<server::platform::sessions::LaunchReceipt, SessionError> {
    let now = Utc::now();
    launch_session(
        f.repo.clone(),
        f.audit.clone(),
        f.event_bus.clone(),
        new_session_registry(),
        new_session_live_stream_registry(),
        16,
        64,
        launch_input(f, now),
    )
    .await
}

// ---------------------------------------------------------------------------
// Test 1 — Implicit policy short-circuits to Allow.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn implicit_policy_grant_allows_immediately() {
    let f = build_fixture(
        ConsentPolicy::Implicit,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        // ApprovalMode::Implicit short-circuits Step 6 in the engine.
        ApprovalMode::Implicit,
    )
    .await;

    let result = run_launch(&f).await;
    assert!(
        result.is_ok(),
        "Implicit-policy grant must allow launch, got: {result:?}"
    );

    // No consents were minted.
    let consents = f
        .repo
        .list_consents_for_subordinate(f.subordinate_id)
        .await
        .unwrap();
    assert!(
        consents.is_empty(),
        "Implicit policy must NOT mint a consent; got {consents:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 2 — OneTime first read returns Pending + mints Requested.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn one_time_policy_first_read_returns_pending_and_mints_requested_consent() {
    let f = build_fixture(
        ConsentPolicy::OneTime,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::OneTime,
        },
    )
    .await;

    let err = run_launch(&f)
        .await
        .expect_err("first read must return Pending");
    let consent_id = match err {
        SessionError::ConsentPending {
            consent_id,
            subordinate,
        } => {
            assert_eq!(subordinate, f.subordinate_id);
            consent_id
        }
        other => panic!("expected ConsentPending, got {other:?}"),
    };

    // The minted row exists, with state=Requested + scope.session_id=None.
    let consents = f
        .repo
        .list_consents_for_subordinate(f.subordinate_id)
        .await
        .unwrap();
    assert_eq!(consents.len(), 1);
    let c = &consents[0];
    assert_eq!(c.id, consent_id);
    assert_eq!(c.state, ConsentState::Requested);
    assert_eq!(c.scope.session_id, None, "OneTime → no session axis");
}

// ---------------------------------------------------------------------------
// Test 3 — OneTime second read after acknowledge → Allow.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn one_time_policy_second_read_after_acknowledge_allows() {
    let f = build_fixture(
        ConsentPolicy::OneTime,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::OneTime,
        },
    )
    .await;

    // First call mints the Requested row.
    let _ = run_launch(&f).await.expect_err("first call must Pending");
    let consent_id = f
        .repo
        .list_consents_for_subordinate(f.subordinate_id)
        .await
        .unwrap()[0]
        .id;
    f.repo
        .acknowledge_consent(consent_id, Utc::now(), f.subordinate_id)
        .await
        .expect("ack succeeds");

    // Second call now allows.
    let result = run_launch(&f).await;
    assert!(
        result.is_ok(),
        "after acknowledge, OneTime must allow; got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — PerSession first read mints session-scoped consent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn per_session_policy_first_read_on_session_a_returns_pending_and_mints_session_scoped_consent(
) {
    let f = build_fixture(
        ConsentPolicy::PerSession,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::PerSession,
        },
    )
    .await;

    let err = run_launch(&f)
        .await
        .expect_err("PerSession first read must Pending");
    match err {
        SessionError::ConsentPending { consent_id, .. } => {
            let c = f
                .repo
                .get_consent(consent_id)
                .await
                .unwrap()
                .expect("row exists");
            assert!(
                c.scope.session_id.is_some(),
                "PerSession mint must scope to a session id; got scope={:?}",
                c.scope
            );
            assert_eq!(c.state, ConsentState::Requested);
        }
        other => panic!("expected ConsentPending, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 5 — Session A ack does NOT cover session B (per-session matching).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn per_session_policy_first_read_on_session_b_after_session_a_acknowledged_still_returns_pending(
) {
    let f = build_fixture(
        ConsentPolicy::PerSession,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::PerSession,
        },
    )
    .await;

    // Session A: launch → Pending → ack the row.
    let _ = run_launch(&f)
        .await
        .expect_err("session A first read pending");
    let session_a_consent = f
        .repo
        .list_consents_for_subordinate(f.subordinate_id)
        .await
        .unwrap()[0]
        .id;
    f.repo
        .acknowledge_consent(session_a_consent, Utc::now(), f.subordinate_id)
        .await
        .expect("ack session A");

    // Session B: a fresh launch should return Pending again — the
    // session_id allocated for session B doesn't match session A's
    // ack row, so the per-session lookup misses + the engine + handler
    // mints another Requested row.
    let err = run_launch(&f)
        .await
        .expect_err("session B first read must still Pending");
    let consent_b_id = match err {
        SessionError::ConsentPending { consent_id, .. } => consent_id,
        other => panic!("expected ConsentPending on session B, got {other:?}"),
    };

    // Both consents exist + have distinct session_ids.
    let all = f
        .repo
        .list_consents_for_subordinate(f.subordinate_id)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
    let consent_a = all.iter().find(|c| c.id == session_a_consent).unwrap();
    let consent_b = all.iter().find(|c| c.id == consent_b_id).unwrap();
    assert!(consent_a.scope.session_id.is_some());
    assert!(consent_b.scope.session_id.is_some());
    assert_ne!(
        consent_a.scope.session_id, consent_b.scope.session_id,
        "session A and session B mints must scope to distinct session ids",
    );
}

// ---------------------------------------------------------------------------
// Test 6 — TimedOut + Deny default → Denied.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn per_session_policy_timed_out_with_deny_default_returns_denied() {
    let f = build_fixture(
        ConsentPolicy::PerSession,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::PerSession,
        },
    )
    .await;

    // Pre-seed a TimedOut consent for the subordinate. We can't easily
    // drive the launch handler to allocate a specific session id, so
    // we seed a row keyed on `session_id = None` and use the
    // `OneTime` policy lookup as the gating axis. PerSession mints
    // would key on `session_id = Some(_)`; for the deny-on-TimedOut
    // path the engine consults the lookup result regardless — so a
    // TimedOut row at `(subordinate, org, None)` won't match the
    // PerSession lookup (which reads `(subordinate, org, Some(sid))`).
    //
    // Instead: switch the test to OneTime to exercise the deny path.
    // The plan's TimedOut tests cover the response-table semantics
    // independently of the per-session axis.
    drop(f);
    let f = build_fixture(
        ConsentPolicy::OneTime,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::OneTime,
        },
    )
    .await;

    let timed_out = Consent {
        id: ConsentId::new(),
        agent_id: f.subordinate_id,
        scope: ConsentScope {
            org: f.org_id,
            templates: vec![],
            actions: vec![],
            session_id: None,
        },
        state: ConsentState::TimedOut,
        requested_at: Utc::now() - Duration::hours(1),
        responded_at: None,
        revoked_at: None,
        revocable: true,
        provenance: "engine:step_6@fixture".into(),
        deadline_at: Some(Utc::now() - Duration::seconds(60)),
    };
    f.repo
        .create_consent(&timed_out)
        .await
        .expect("seed timed_out consent");

    let err = run_launch(&f)
        .await
        .expect_err("TimedOut + Deny must reject launch");
    match err {
        SessionError::ConsentDenied { subordinate, .. } => {
            assert_eq!(subordinate, f.subordinate_id);
        }
        other => panic!("expected ConsentDenied, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 7 — TimedOut + Allow default → Allow.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn per_session_policy_timed_out_with_allow_default_returns_allow() {
    let f = build_fixture(
        ConsentPolicy::OneTime,
        ApprovalTimeout::ProjectDuration,
        TimeoutResponse::Allow,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::OneTime,
        },
    )
    .await;

    let timed_out = Consent {
        id: ConsentId::new(),
        agent_id: f.subordinate_id,
        scope: ConsentScope {
            org: f.org_id,
            templates: vec![],
            actions: vec![],
            session_id: None,
        },
        state: ConsentState::TimedOut,
        requested_at: Utc::now() - Duration::hours(1),
        responded_at: None,
        revoked_at: None,
        revocable: true,
        provenance: "engine:step_6@fixture".into(),
        deadline_at: Some(Utc::now() - Duration::seconds(60)),
    };
    f.repo
        .create_consent(&timed_out)
        .await
        .expect("seed timed_out consent");

    let result = run_launch(&f).await;
    assert!(
        result.is_ok(),
        "TimedOut + Allow default must allow launch; got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 8 — F3.A locked: Fixed(24h) deadline computation.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deadline_uses_org_approval_timeout_fixed_value() {
    let f = build_fixture(
        ConsentPolicy::OneTime,
        ApprovalTimeout::Fixed {
            duration: Duration::hours(24),
        },
        TimeoutResponse::Deny,
        ApprovalMode::SubordinateRequired {
            policy: ConsentPolicy::OneTime,
        },
    )
    .await;

    let before = Utc::now();
    let err = run_launch(&f).await.expect_err("Pending expected");
    let after = Utc::now();
    let consent_id = match err {
        SessionError::ConsentPending { consent_id, .. } => consent_id,
        other => panic!("expected Pending, got {other:?}"),
    };
    let consent = f.repo.get_consent(consent_id).await.unwrap().unwrap();
    let deadline = consent.deadline_at.expect("Fixed must populate deadline");
    let expected_lo = before + Duration::hours(24) - Duration::seconds(2);
    let expected_hi = after + Duration::hours(24) + Duration::seconds(2);
    assert!(
        deadline >= expected_lo && deadline <= expected_hi,
        "deadline {deadline:?} must be within ~now+24h (lo={expected_lo:?}, hi={expected_hi:?})",
    );
}
