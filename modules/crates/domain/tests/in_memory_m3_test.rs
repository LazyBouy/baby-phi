//! Smoke tests for the M3 additions to `InMemoryRepository`.
//!
//! Covers the 5 org-scoped list methods that M3/P5's dashboard
//! handler reads from. Each method gets a populated-org case + an
//! empty-org case (or unknown-org where applicable) so the "empty
//! returns empty Vec, not error" contract stays pinned.
//!
//! Commitment C5 in the M3 plan.

#![cfg(feature = "in-memory-repo")]

use chrono::Utc;

use domain::audit::{AuditClass, AuditEvent};
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId};
use domain::model::nodes::{Agent, AgentKind, Organization, PrincipalRef, TemplateKind};
use domain::model::ConsentPolicy;
use domain::repository::Repository;
use domain::templates::a;

fn minimal_org(id: OrgId) -> Organization {
    Organization {
        id,
        display_name: format!("org-{}", id),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        created_at: Utc::now(),
    }
}

fn agent_in(org: OrgId, kind: AgentKind) -> Agent {
    Agent {
        id: AgentId::new(),
        kind,
        display_name: "test-agent".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
    }
}

// ---- list_agents_in_org ----------------------------------------------------

#[tokio::test]
async fn list_agents_in_org_returns_members_and_ignores_others() {
    let repo = InMemoryRepository::new();
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    repo.create_organization(&minimal_org(org_a)).await.unwrap();
    repo.create_organization(&minimal_org(org_b)).await.unwrap();

    // 3 agents in org_a + 1 in org_b.
    for _ in 0..3 {
        repo.create_agent(&agent_in(org_a, AgentKind::Llm))
            .await
            .unwrap();
    }
    repo.create_agent(&agent_in(org_b, AgentKind::Human))
        .await
        .unwrap();

    let a = repo.list_agents_in_org(org_a).await.unwrap();
    assert_eq!(a.len(), 3, "org_a has 3 agents");

    let b = repo.list_agents_in_org(org_b).await.unwrap();
    assert_eq!(b.len(), 1, "org_b has 1 agent");
}

#[tokio::test]
async fn list_agents_in_org_returns_empty_for_unknown_org() {
    let repo = InMemoryRepository::new();
    let orphan = OrgId::new();
    let agents = repo.list_agents_in_org(orphan).await.unwrap();
    assert!(
        agents.is_empty(),
        "unknown org returns empty Vec, not error"
    );
}

// ---- list_projects_in_org -------------------------------------------------

#[tokio::test]
async fn list_projects_in_org_always_empty_for_m3() {
    // M3 wires the method with an empty stub — M4 will persist
    // projects. The contract test pins the placeholder behaviour so
    // the M3/P5 dashboard panel renders "0 projects" deterministically.
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    let projects = repo.list_projects_in_org(org).await.unwrap();
    assert!(projects.is_empty());
}

// ---- list_active_auth_requests_for_org ------------------------------------

#[tokio::test]
async fn list_active_auth_requests_for_org_filters_by_org_and_state() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    let ceo_id = AgentId::new();
    repo.create_organization(&minimal_org(org)).await.unwrap();
    repo.create_agent(&Agent {
        id: ceo_id,
        kind: AgentKind::Human,
        display_name: "CEO".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
    })
    .await
    .unwrap();

    // One Template-A adoption AR for the org's CEO — state Approved
    // is terminal, so it should NOT appear in "active" list.
    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id: org,
        ceo: PrincipalRef::Agent(ceo_id),
        now: Utc::now(),
    });
    repo.create_auth_request(&adoption_ar).await.unwrap();

    let active = repo.list_active_auth_requests_for_org(org).await.unwrap();
    assert!(
        active.is_empty(),
        "Approved adoption AR is terminal — not in active list"
    );
}

#[tokio::test]
async fn list_active_auth_requests_for_org_empty_for_unknown_org() {
    let repo = InMemoryRepository::new();
    let orphan = OrgId::new();
    assert!(repo
        .list_active_auth_requests_for_org(orphan)
        .await
        .unwrap()
        .is_empty());
}

// ---- list_recent_audit_events_for_org -------------------------------------

#[tokio::test]
async fn list_recent_audit_events_for_org_returns_newest_first_bounded_by_limit() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();

    // Write 3 events with increasing timestamps — the method must
    // return them in reverse-chronological order.
    for i in 0..3 {
        let ev = AuditEvent {
            event_id: AuditEventId::new(),
            event_type: format!("test.event.{}", i),
            actor_agent_id: None,
            target_entity_id: Some(NodeId::new()),
            timestamp: Utc::now(),
            diff: serde_json::Value::Null,
            audit_class: AuditClass::Logged,
            provenance_auth_request_id: None,
            org_scope: Some(org),
            prev_event_hash: None,
        };
        repo.write_audit_event(&ev).await.unwrap();
    }

    let events = repo.list_recent_audit_events_for_org(org, 2).await.unwrap();
    assert_eq!(events.len(), 2, "limit caps the result");
    assert!(
        events[0].timestamp >= events[1].timestamp,
        "newest-first ordering"
    );
}

#[tokio::test]
async fn list_recent_audit_events_for_org_excludes_other_orgs() {
    let repo = InMemoryRepository::new();
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    repo.write_audit_event(&AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "for.org_a".into(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::Value::Null,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org_a),
        prev_event_hash: None,
    })
    .await
    .unwrap();
    repo.write_audit_event(&AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "for.org_b".into(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::Value::Null,
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: Some(org_b),
        prev_event_hash: None,
    })
    .await
    .unwrap();

    let a = repo
        .list_recent_audit_events_for_org(org_a, 10)
        .await
        .unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].event_type, "for.org_a");
}

// ---- list_adoption_auth_requests_for_org ----------------------------------

#[tokio::test]
async fn list_adoption_auth_requests_for_org_matches_uri_prefix() {
    let repo = InMemoryRepository::new();
    let org = OrgId::new();
    let ceo = PrincipalRef::Agent(AgentId::new());

    // Persist 2 adoption ARs (Template A + Template B).
    for _ in 0..2 {
        let ar = a::build_adoption_request(a::AdoptionArgs {
            org_id: org,
            ceo: ceo.clone(),
            now: Utc::now(),
        });
        repo.create_auth_request(&ar).await.unwrap();
    }
    let b_ar = domain::templates::b::build_adoption_request(domain::templates::b::AdoptionArgs {
        org_id: org,
        ceo: ceo.clone(),
        now: Utc::now(),
    });
    repo.create_auth_request(&b_ar).await.unwrap();

    let adoptions = repo.list_adoption_auth_requests_for_org(org).await.unwrap();
    assert_eq!(adoptions.len(), 3);
}

#[tokio::test]
async fn list_adoption_auth_requests_for_org_does_not_cross_orgs() {
    let repo = InMemoryRepository::new();
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    let ceo = PrincipalRef::Agent(AgentId::new());
    // Adoption AR against org_a
    repo.create_auth_request(&a::build_adoption_request(a::AdoptionArgs {
        org_id: org_a,
        ceo: ceo.clone(),
        now: Utc::now(),
    }))
    .await
    .unwrap();

    let b_adoptions = repo
        .list_adoption_auth_requests_for_org(org_b)
        .await
        .unwrap();
    assert!(
        b_adoptions.is_empty(),
        "adoption ARs don't leak across orgs"
    );
}

// ---- M3 audit event builder smoke (domain layer) ---------------------------

#[tokio::test]
async fn m3_audit_event_builders_produce_org_scoped_events() {
    // Not a repository test per se — verifies the domain-layer
    // invariant that M3's builders set `org_scope = Some(org_id)`.
    use domain::audit::events::m3::orgs;

    let org = minimal_org(OrgId::new());
    let ceo = AgentId::new();
    let actor = AgentId::new();
    let ev = orgs::organization_created(actor, &org, ceo, None, Utc::now());
    assert_eq!(ev.org_scope, Some(org.id));

    let adoption_ar = domain::model::ids::AuthRequestId::new();
    let ev2 =
        orgs::authority_template_adopted(actor, org.id, TemplateKind::A, adoption_ar, Utc::now());
    assert_eq!(ev2.org_scope, Some(org.id));
}
