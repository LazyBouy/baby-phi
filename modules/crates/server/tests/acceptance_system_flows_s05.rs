//! CH-23 / ADR-0046 — system-flow s05 acceptance tests.
//!
//! Closes the loop on Template C and Template D end-to-end: the
//! production HTTP code paths
//! (`POST /api/v0/orgs/:org/agents/:agent/manager` and
//! `POST /api/v0/projects/:project/agents/:agent/supervisor`) emit the
//! `MANAGES` and `HAS_AGENT_SUPERVISOR` trigger events the
//! `TemplateCFireListener` and `TemplateDFireListener` subscribe to,
//! and a real listener wired to the harness's event bus mints the
//! `[read, inspect]` grant per concept-doc §"Template C/D".
//!
//! Scenarios:
//!  1. **Template C end-to-end on MANAGES.** POST manager → edge row
//!     persisted + `template.c.grant_fired` audit event + grant on
//!     `agent:<subordinate>` held by manager.
//!  2. **Template D end-to-end on HAS_AGENT_SUPERVISOR.** POST
//!     supervisor → edge row persisted + `template.d.grant_fired`
//!     audit event + grant on `project:<p>/agent:<supervisee>` held
//!     by supervisor.
//!  3. **A + C + D simultaneous.** A project lead with a manager
//!     relationship and a supervisor relationship triggers all three
//!     listeners; all three grants exist + each audit fires exactly
//!     once.
//!  4. **Cross-listener ordering.** Two listeners subscribed to the
//!     same `ManagesEdgeCreated` variant dispatch in
//!     subscription-registration order; an atomic counter records the
//!     observed order to defend against future bus refactors that
//!     would silently change semantics.

mod acceptance_common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use acceptance_common::admin::{
    spawn_claimed_with_org, spawn_claimed_with_org_and_project, ClaimedOrg, ClaimedProject,
};
use chrono::Utc;
use domain::audit::AuditEmitter;
use domain::events::{
    DomainEvent, EventHandler, TemplateAFireListener, TemplateCFireListener, TemplateDFireListener,
};
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, AgentRole, AuthRequest, InboxObject, OutboxObject};
use domain::repository::AgentCreationPayload;
use domain::Repository;
use serde_json::{json, Value};
use server::platform::projects::{
    RepoActorResolver, RepoAdoptionArResolver, RepoTemplateCAdoptionArResolver,
    RepoTemplateDAdoptionArResolver,
};

// ---------------------------------------------------------------------------
// Listener wiring helpers
// ---------------------------------------------------------------------------

/// Subscribe a real `TemplateCFireListener` to the harness's event bus
/// so it observes `ManagesEdgeCreated` emits from production handlers.
fn subscribe_template_c(org: &ClaimedOrg) {
    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let audit: Arc<dyn AuditEmitter> = Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    let listener = Arc::new(TemplateCFireListener::new(
        repo.clone(),
        audit,
        Arc::new(RepoTemplateCAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo)),
    ));
    org.acc().event_bus.subscribe(listener);
}

/// Subscribe a real `TemplateDFireListener`.
fn subscribe_template_d(org: &ClaimedOrg) {
    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let audit: Arc<dyn AuditEmitter> = Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    let listener = Arc::new(TemplateDFireListener::new(
        repo.clone(),
        audit,
        Arc::new(RepoTemplateDAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo)),
    ));
    org.acc().event_bus.subscribe(listener);
}

/// Subscribe a real `TemplateAFireListener` for scenario 3 — wires the
/// `HasLeadEdgeCreated` path that production-create_project emits.
fn subscribe_template_a(org: &ClaimedOrg) {
    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let audit: Arc<dyn AuditEmitter> = Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    let listener = Arc::new(TemplateAFireListener::new(
        repo.clone(),
        audit,
        Arc::new(RepoAdoptionArResolver::new(repo.clone())),
        Arc::new(RepoActorResolver::new(repo)),
    ));
    org.acc().event_bus.subscribe(listener);
}

/// Convenience accessor — `ClaimedOrg.admin.acc` is what every test
/// reaches for; bundle into one call.
trait ClaimedOrgExt {
    fn acc(&self) -> &acceptance_common::Acceptance;
}

impl ClaimedOrgExt for ClaimedOrg {
    fn acc(&self) -> &acceptance_common::Acceptance {
        &self.admin.acc
    }
}

impl ClaimedOrgExt for ClaimedProject {
    fn acc(&self) -> &acceptance_common::Acceptance {
        &self.claimed_org.admin.acc
    }
}

fn ceo_client(org: &ClaimedOrg) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(&org.admin, org.ceo_agent_id)
        .expect("mint CEO session")
}

/// Insert a Template-C (or Template-D) adoption AR into the test org so
/// the listener's resolver can find it. The fixture only seeds Template
/// A by default.
async fn seed_template_c_adoption(org: &ClaimedOrg) {
    let ar: AuthRequest =
        domain::templates::c::build_adoption_request(domain::templates::c::AdoptionArgs {
            org_id: org.org_id,
            ceo: domain::model::nodes::PrincipalRef::Agent(org.ceo_agent_id),
            now: Utc::now(),
        });
    org.acc()
        .store
        .create_auth_request(&ar)
        .await
        .expect("seed template C adoption AR");
}

async fn seed_template_d_adoption(org: &ClaimedOrg) {
    let ar: AuthRequest =
        domain::templates::d::build_adoption_request(domain::templates::d::AdoptionArgs {
            org_id: org.org_id,
            ceo: domain::model::nodes::PrincipalRef::Agent(org.ceo_agent_id),
            now: Utc::now(),
        });
    org.acc()
        .store
        .create_auth_request(&ar)
        .await
        .expect("seed template D adoption AR");
}

async fn create_member_agent(repo: &Arc<dyn Repository>, org_id: OrgId, name: &str) -> AgentId {
    let now = Utc::now();
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: name.into(),
        owning_org: Some(org_id),
        role: Some(AgentRole::Intern),
        created_at: now,
        active: true,
        archived_at: None,
    };
    let inbox = InboxObject {
        id: domain::model::ids::NodeId::new(),
        agent_id: agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let outbox = OutboxObject {
        id: domain::model::ids::NodeId::new(),
        agent_id: agent.id,
        created_at: now,
        tags: Vec::new(),
    };
    let aid = agent.id;
    repo.apply_agent_creation(&AgentCreationPayload {
        agent,
        inbox,
        outbox,
        profile: None,
        default_grants: vec![],
        initial_execution_limits_override: None,
        catalogue_entries: vec![],
        identity: Some(domain::model::nodes::Identity::default_for_llm(aid, now)),
    })
    .await
    .expect("create member agent");
    aid
}

// ---------------------------------------------------------------------------
// Scenario 1 — Template C end-to-end on MANAGES
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_1_template_c_fires_on_manages_edge_creation() {
    let org = spawn_claimed_with_org(false).await;
    seed_template_c_adoption(&org).await;
    subscribe_template_c(&org);

    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let manager = create_member_agent(&repo, org.org_id, "manager").await;
    let subordinate = create_member_agent(&repo, org.org_id, "subordinate").await;

    let url = org.url(&format!(
        "/api/v0/orgs/{}/agents/{}/manager",
        org.org_id, subordinate
    ));
    let body = json!({"manager_agent_id": manager.to_string()});
    let res = ceo_client(&org).post(url).json(&body).send().await.unwrap();
    assert_eq!(
        res.status().as_u16(),
        201,
        "create returned {:?}",
        res.text().await
    );
    let resp: Value = res.json().await.unwrap();
    assert_eq!(resp["created"], json!(true));

    // Bus is in-process synchronous — listener has fired by now.

    // (a) Template C grant exists on the manager principal.
    let grants = repo
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(manager))
        .await
        .unwrap();
    assert!(
        grants.iter().any(|g| g.resource.uri.starts_with("agent:")
            && g.resource.uri.contains(&subordinate.to_string())),
        "Template C grant on agent:<subordinate> not found; got {:?}",
        grants.iter().map(|g| &g.resource.uri).collect::<Vec<_>>()
    );

    // (b) Audit chain has both edge-created + template-c-grant-fired
    //     events with org_scope = test org.
    let events = repo
        .list_recent_audit_events_for_org(org.org_id, 100)
        .await
        .unwrap();
    let types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert!(
        types.contains(&"platform.manages.edge.created"),
        "missing edge-created audit; got {:?}",
        types
    );
    assert!(
        types.contains(&"template.c.grant_fired"),
        "missing template.c.grant_fired audit; got {:?}",
        types
    );
}

#[tokio::test]
async fn scenario_1_re_post_is_idempotent_and_no_double_grant() {
    let org = spawn_claimed_with_org(false).await;
    seed_template_c_adoption(&org).await;
    subscribe_template_c(&org);

    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let manager = create_member_agent(&repo, org.org_id, "manager").await;
    let subordinate = create_member_agent(&repo, org.org_id, "subordinate").await;

    let url = org.url(&format!(
        "/api/v0/orgs/{}/agents/{}/manager",
        org.org_id, subordinate
    ));
    let body = json!({"manager_agent_id": manager.to_string()});
    let first = ceo_client(&org)
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status().as_u16(), 201);

    let second = ceo_client(&org)
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        second.status().as_u16(),
        200,
        "idempotent re-POST returned {:?}",
        second.text().await
    );
    let body: Value = second.json().await.unwrap();
    assert_eq!(body["created"], json!(false));

    // Only one Template C grant on the manager principal.
    let grants = repo
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(manager))
        .await
        .unwrap();
    let template_c_grants: Vec<_> = grants
        .iter()
        .filter(|g| g.resource.uri.contains(&subordinate.to_string()))
        .collect();
    assert_eq!(
        template_c_grants.len(),
        1,
        "idempotent path must not double-fire Template C; got {:?}",
        template_c_grants
    );
}

// ---------------------------------------------------------------------------
// Scenario 2 — Template D end-to-end on HAS_AGENT_SUPERVISOR
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_2_template_d_fires_on_has_agent_supervisor_edge_creation() {
    let project = spawn_claimed_with_org_and_project(false).await;
    seed_template_d_adoption(&project.claimed_org).await;
    subscribe_template_d(&project.claimed_org);

    let supervisor = project.project_member; // member is the supervisor
    let supervisee = project.project_lead; // lead becomes supervisee here

    let url = project.url(&format!(
        "/api/v0/projects/{}/agents/{}/supervisor",
        project.project_id, supervisee
    ));
    let body = json!({"supervisor_agent_id": supervisor.to_string()});
    let res = ceo_client(&project.claimed_org)
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status().as_u16(),
        201,
        "create returned {:?}",
        res.text().await
    );
    let resp: Value = res.json().await.unwrap();
    assert_eq!(resp["created"], json!(true));

    let repo: Arc<dyn Repository> = project.acc().store.clone();
    let grants = repo
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(supervisor))
        .await
        .unwrap();
    assert!(
        grants.iter().any(|g| {
            g.resource
                .uri
                .starts_with(&format!("project:{}/agent:", project.project_id))
                && g.resource.uri.contains(&supervisee.to_string())
        }),
        "Template D grant on project:<p>/agent:<supervisee> not found; got {:?}",
        grants.iter().map(|g| &g.resource.uri).collect::<Vec<_>>()
    );

    let events = repo
        .list_recent_audit_events_for_org(project.claimed_org.org_id, 100)
        .await
        .unwrap();
    let types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert!(types.contains(&"platform.has_agent_supervisor.edge.created"));
    assert!(types.contains(&"template.d.grant_fired"));
}

// ---------------------------------------------------------------------------
// Scenario 3 — A + C + D simultaneous
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scenario_3_a_c_d_all_fire_for_one_project_with_three_relationships() {
    // A project bundle gives us the Template A path live (lead +
    // HasLeadEdgeCreated); C and D are then triggered via our new
    // HTTP endpoints.
    let project = spawn_claimed_with_org_and_project(false).await;
    seed_template_c_adoption(&project.claimed_org).await;
    seed_template_d_adoption(&project.claimed_org).await;
    subscribe_template_a(&project.claimed_org);
    subscribe_template_c(&project.claimed_org);
    subscribe_template_d(&project.claimed_org);

    let repo: Arc<dyn Repository> = project.acc().store.clone();
    let lead = project.project_lead;
    let manager = project.project_member;

    // Trigger Template A by replaying the post-commit emit pattern
    // the M4/P6 production handler uses (the fixture sets up the
    // edge structurally but does not emit).
    project
        .acc()
        .event_bus
        .emit(DomainEvent::HasLeadEdgeCreated {
            project: project.project_id,
            lead,
            at: Utc::now(),
            event_id: domain::model::ids::AuditEventId::new(),
        })
        .await;

    // Template C — manager assignment for a fresh subordinate.
    let subordinate = create_member_agent(&repo, project.claimed_org.org_id, "subordinate-3").await;
    let c_url = project.url(&format!(
        "/api/v0/orgs/{}/agents/{}/manager",
        project.claimed_org.org_id, subordinate
    ));
    let res = ceo_client(&project.claimed_org)
        .post(c_url)
        .json(&json!({"manager_agent_id": manager.to_string()}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);

    // Template D — project supervisor assignment.
    let d_url = project.url(&format!(
        "/api/v0/projects/{}/agents/{}/supervisor",
        project.project_id, lead
    ));
    let res = ceo_client(&project.claimed_org)
        .post(d_url)
        .json(&json!({"supervisor_agent_id": manager.to_string()}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);

    // All three audit events must be present.
    let events = repo
        .list_recent_audit_events_for_org(project.claimed_org.org_id, 200)
        .await
        .unwrap();
    let types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert!(
        types.contains(&"template.a.grant_fired"),
        "Template A grant audit missing; got {:?}",
        types
    );
    assert!(
        types.contains(&"template.c.grant_fired"),
        "Template C grant audit missing; got {:?}",
        types
    );
    assert!(
        types.contains(&"template.d.grant_fired"),
        "Template D grant audit missing; got {:?}",
        types
    );

    // Each Template's grant exists on its expected holder.
    let lead_grants = repo
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(lead))
        .await
        .unwrap();
    assert!(
        lead_grants.iter().any(|g| g
            .resource
            .uri
            .starts_with(&format!("project:{}", project.project_id))),
        "Template A grant on project:<p> not minted on lead",
    );

    let manager_grants = repo
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(manager))
        .await
        .unwrap();
    let has_c = manager_grants
        .iter()
        .any(|g| g.resource.uri.contains(&subordinate.to_string()));
    let has_d = manager_grants.iter().any(|g| {
        g.resource
            .uri
            .starts_with(&format!("project:{}/agent:", project.project_id))
    });
    assert!(has_c, "Template C grant missing on manager");
    assert!(has_d, "Template D grant missing on manager (supervisor)");
}

// ---------------------------------------------------------------------------
// Scenario 4 — Cross-listener subscription order is deterministic
// ---------------------------------------------------------------------------

/// Test-only listener that records the order in which it's invoked
/// against a shared atomic counter.
struct OrderRecordingListener {
    counter: Arc<AtomicUsize>,
    seen_at: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl EventHandler for OrderRecordingListener {
    async fn on_event(&self, event: &DomainEvent) {
        if matches!(event, DomainEvent::ManagesEdgeCreated { .. }) {
            let pos = self.counter.fetch_add(1, Ordering::SeqCst);
            self.seen_at.store(pos, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn scenario_4_cross_listener_dispatch_order_is_subscription_order() {
    let org = spawn_claimed_with_org(false).await;
    seed_template_c_adoption(&org).await;

    // Subscribe in a known order: first OrderRecordingListener_A,
    // then TemplateCFireListener, then OrderRecordingListener_B. The
    // dispatch order must match — A fires at counter=0, C in between
    // (it doesn't increment the counter), B at counter=1.
    let counter = Arc::new(AtomicUsize::new(0));
    let a_seen = Arc::new(AtomicUsize::new(usize::MAX));
    let b_seen = Arc::new(AtomicUsize::new(usize::MAX));

    org.acc()
        .event_bus
        .subscribe(Arc::new(OrderRecordingListener {
            counter: counter.clone(),
            seen_at: a_seen.clone(),
        }));
    subscribe_template_c(&org);
    org.acc()
        .event_bus
        .subscribe(Arc::new(OrderRecordingListener {
            counter: counter.clone(),
            seen_at: b_seen.clone(),
        }));

    let repo: Arc<dyn Repository> = org.acc().store.clone();
    let manager = create_member_agent(&repo, org.org_id, "manager-4").await;
    let subordinate = create_member_agent(&repo, org.org_id, "subordinate-4").await;

    let url = org.url(&format!(
        "/api/v0/orgs/{}/agents/{}/manager",
        org.org_id, subordinate
    ));
    let res = ceo_client(&org)
        .post(url)
        .json(&json!({"manager_agent_id": manager.to_string()}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 201);

    let a_pos = a_seen.load(Ordering::SeqCst);
    let b_pos = b_seen.load(Ordering::SeqCst);
    assert_eq!(a_pos, 0, "A subscribed first must observe order=0");
    assert_eq!(b_pos, 1, "B subscribed last must observe order=1");
    assert!(
        a_pos < b_pos,
        "subscription order must be preserved (a={a_pos}, b={b_pos})"
    );
}
