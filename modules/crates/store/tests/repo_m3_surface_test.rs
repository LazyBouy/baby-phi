//! Integration tests for M3 additions to the `SurrealStore: Repository`
//! surface. Mirrors `domain/tests/in_memory_m3_test.rs` against the
//! real SurrealDB backend so the two impls stay in lock-step.
//!
//! Commitment C5 in the M3 plan.

use chrono::Utc;
use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, NodeId, OrgId};
use domain::model::nodes::{Agent, AgentKind, Organization, PrincipalRef};
use domain::model::ConsentPolicy;
use domain::repository::Repository;
use domain::templates::a;
use store::SurrealStore;
use tempfile::TempDir;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

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

// ---- list_agents_in_org ----------------------------------------------------

#[tokio::test]
async fn list_agents_in_org_filters_by_owning_org() {
    let (store, _dir) = fresh_store().await;
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    store
        .create_organization(&minimal_org(org_a))
        .await
        .unwrap();
    store
        .create_organization(&minimal_org(org_b))
        .await
        .unwrap();

    for _ in 0..3 {
        store
            .create_agent(&Agent {
                id: AgentId::new(),
                kind: AgentKind::Llm,
                display_name: "worker".into(),
                owning_org: Some(org_a),
                role: None,
                created_at: Utc::now(),
            })
            .await
            .unwrap();
    }
    store
        .create_agent(&Agent {
            id: AgentId::new(),
            kind: AgentKind::Human,
            display_name: "ceo-b".into(),
            owning_org: Some(org_b),
            role: None,
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    let agents_a = store.list_agents_in_org(org_a).await.unwrap();
    assert_eq!(agents_a.len(), 3);

    let agents_b = store.list_agents_in_org(org_b).await.unwrap();
    assert_eq!(agents_b.len(), 1);

    // Unknown org → empty Vec, NOT NotFound.
    let unknown = store.list_agents_in_org(OrgId::new()).await.unwrap();
    assert!(unknown.is_empty());
}

// ---- list_projects_in_org -------------------------------------------------

#[tokio::test]
async fn list_projects_in_org_empty_for_m3() {
    let (store, _dir) = fresh_store().await;
    let projects = store.list_projects_in_org(OrgId::new()).await.unwrap();
    assert!(projects.is_empty(), "projects land in M4");
}

// ---- list_active_auth_requests_for_org ------------------------------------

#[tokio::test]
async fn list_active_auth_requests_for_org_terminal_adoption_excluded() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let ceo_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "ceo".into(),
        owning_org: Some(org),
        role: None,
        created_at: Utc::now(),
    };
    store.create_organization(&minimal_org(org)).await.unwrap();
    store.create_agent(&ceo_agent).await.unwrap();

    // Adoption AR = Approved (terminal). Must NOT surface in "active"
    // results.
    let ar = a::build_adoption_request(a::AdoptionArgs {
        org_id: org,
        ceo: PrincipalRef::Agent(ceo_agent.id),
        now: Utc::now(),
    });
    store.create_auth_request(&ar).await.unwrap();

    let active = store.list_active_auth_requests_for_org(org).await.unwrap();
    assert!(active.is_empty());
}

// ---- list_recent_audit_events_for_org -------------------------------------

#[tokio::test]
async fn list_recent_audit_events_for_org_orders_and_bounds() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();

    // Write 4 events with increasing timestamps. The DESC ORDER BY in
    // the SurrealQL query should return them newest-first.
    let base = Utc::now();
    for i in 0..4 {
        let ev = AuditEvent {
            event_id: AuditEventId::new(),
            event_type: format!("test.event.{}", i),
            actor_agent_id: None,
            target_entity_id: Some(NodeId::new()),
            timestamp: base + chrono::Duration::seconds(i),
            diff: serde_json::json!({}),
            audit_class: AuditClass::Logged,
            provenance_auth_request_id: None,
            org_scope: Some(org),
            prev_event_hash: None,
        };
        store.write_audit_event(&ev).await.unwrap();
    }

    let events = store
        .list_recent_audit_events_for_org(org, 3)
        .await
        .unwrap();
    assert_eq!(events.len(), 3, "limit caps at 3");
    assert!(
        events[0].timestamp >= events[1].timestamp,
        "newest-first ordering"
    );
    assert!(events[1].timestamp >= events[2].timestamp);
}

#[tokio::test]
async fn list_recent_audit_events_for_org_excludes_platform_root_chain() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();

    // One event in the org's chain, one in the platform root chain
    // (org_scope = None). The org-scoped list must only return the
    // first.
    store
        .write_audit_event(&AuditEvent {
            event_id: AuditEventId::new(),
            event_type: "for.org".into(),
            actor_agent_id: None,
            target_entity_id: None,
            timestamp: Utc::now(),
            diff: serde_json::json!({}),
            audit_class: AuditClass::Logged,
            provenance_auth_request_id: None,
            org_scope: Some(org),
            prev_event_hash: None,
        })
        .await
        .unwrap();
    store
        .write_audit_event(&AuditEvent {
            event_id: AuditEventId::new(),
            event_type: "for.platform".into(),
            actor_agent_id: None,
            target_entity_id: None,
            timestamp: Utc::now(),
            diff: serde_json::json!({}),
            audit_class: AuditClass::Logged,
            provenance_auth_request_id: None,
            org_scope: None,
            prev_event_hash: None,
        })
        .await
        .unwrap();

    let events = store
        .list_recent_audit_events_for_org(org, 10)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "for.org");
}

// ---- list_adoption_auth_requests_for_org ----------------------------------

#[tokio::test]
async fn list_adoption_auth_requests_for_org_matches_uri_prefix_on_surrealdb() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let ceo = PrincipalRef::Agent(AgentId::new());

    for _ in 0..2 {
        store
            .create_auth_request(&a::build_adoption_request(a::AdoptionArgs {
                org_id: org,
                ceo: ceo.clone(),
                now: Utc::now(),
            }))
            .await
            .unwrap();
    }

    let adoptions = store
        .list_adoption_auth_requests_for_org(org)
        .await
        .unwrap();
    assert_eq!(adoptions.len(), 2);

    // Different org → no cross-contamination.
    let other_org = OrgId::new();
    let other_adoptions = store
        .list_adoption_auth_requests_for_org(other_org)
        .await
        .unwrap();
    assert!(other_adoptions.is_empty());
}
