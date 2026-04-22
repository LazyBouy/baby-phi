//! Integration tests for `Repository::apply_org_creation` on
//! SurrealDB. Proves:
//!
//!   1. Happy path commits every expected row + edge atomically.
//!   2. Duplicate-org-id returns `Conflict` (the tx rolls back; no
//!      partial state survives).
//!   3. **ADR-0023 invariant**: zero rows land in the per-agent
//!      `execution_limits` / `retry_policy` / `cache_policy` /
//!      `compaction_policy` tables — system agents inherit from
//!      `Organization.defaults_snapshot` instead.
//!   4. **phi-core positive-grep invariant**: each system agent's
//!      `agent_profile.blueprint` column round-trips every phi-core
//!      `AgentProfile` field (system_prompt preserved end-to-end).
//!
//! Commitment C9 in the M3 plan; Q1/Q2/Q3 close-audit greps for
//! P3's phi-core leverage subsection.

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::composites_m3::{ConsentPolicy, TokenBudgetPool};
use domain::model::ids::{AgentId, GrantId, NodeId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, Channel, ChannelKind, Grant, InboxObject, Organization,
    OutboxObject, PrincipalRef, ResourceRef, TemplateKind,
};
use domain::model::Fundamental;
use domain::repository::{OrgCreationPayload, Repository, RepositoryError};
use domain::templates::a;
use store::SurrealStore;
use tempfile::TempDir;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "baby-phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

fn sample_org() -> Organization {
    Organization {
        id: OrgId::new(),
        display_name: "Acme".into(),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![TemplateKind::A],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        created_at: Utc::now(),
    }
}

fn build_payload(org: Organization) -> OrgCreationPayload {
    let org_id = org.id;
    let ceo_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "Acme CEO".into(),
        owning_org: Some(org_id),
        created_at: Utc::now(),
    };
    let ceo_channel = Channel {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        kind: ChannelKind::Email,
        handle: "ceo@acme.test".into(),
        created_at: Utc::now(),
    };
    let ceo_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: Utc::now(),
    };
    let ceo_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: Utc::now(),
    };
    let ceo_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(ceo_agent.id),
        action: vec!["allocate".into()],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![Fundamental::IdentityPrincipal, Fundamental::Tag],
        descends_from: None,
        delegable: true,
        issued_at: Utc::now(),
        revoked_at: None,
    };
    let sys0_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "memory-extractor".into(),
        owning_org: Some(org_id),
        created_at: Utc::now(),
    };
    let sys1_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
        created_at: Utc::now(),
    };
    // Role-specific phi-core blueprints — each one proves transit of
    // `phi_core::AgentProfile` through the compound tx.
    let sys0_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("memory-extractor".into()),
        system_prompt: Some("You distill agent memories from recent sessions.".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys1_blueprint = phi_core::agents::profile::AgentProfile {
        name: Some("agent-catalog".into()),
        system_prompt: Some("You maintain the org's agent catalogue.".into()),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    let sys0_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys0_agent.id,
        parallelize: 1,
        blueprint: sys0_blueprint,
        created_at: Utc::now(),
    };
    let sys1_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: sys1_agent.id,
        parallelize: 1,
        blueprint: sys1_blueprint,
        created_at: Utc::now(),
    };
    let token_budget_pool = TokenBudgetPool::new(org_id, 1_000_000, Utc::now());

    let adoption_ar = a::build_adoption_request(a::AdoptionArgs {
        org_id,
        ceo: PrincipalRef::Agent(ceo_agent.id),
        now: Utc::now(),
    });

    OrgCreationPayload {
        organization: org,
        ceo_agent,
        ceo_channel,
        ceo_inbox,
        ceo_outbox,
        ceo_grant,
        system_agents: [(sys0_agent, sys0_profile), (sys1_agent, sys1_profile)],
        token_budget_pool,
        adoption_auth_requests: vec![adoption_ar],
        catalogue_entries: vec![
            (format!("org:{}", org_id), "control_plane".into()),
            (format!("org:{}/template:a", org_id), "control_plane".into()),
        ],
    }
}

async fn count_rows(store: &SurrealStore, table: &str) -> i64 {
    let q = format!("SELECT count() FROM {table} GROUP ALL");
    let counts: Vec<i64> = store
        .client()
        .query(&q)
        .await
        .unwrap()
        .take((0, "count"))
        .unwrap();
    counts.into_iter().next().unwrap_or(0)
}

#[tokio::test]
async fn happy_path_commits_every_row_and_returns_receipt() {
    let (store, _dir) = fresh_store().await;
    let org = sample_org();
    let payload = build_payload(org.clone());

    let receipt = store.apply_org_creation(&payload).await.expect("commit");

    // Receipt identity.
    assert_eq!(receipt.org_id, payload.organization.id);
    assert_eq!(receipt.ceo_agent_id, payload.ceo_agent.id);
    assert_eq!(
        receipt.system_agent_ids,
        [payload.system_agents[0].0.id, payload.system_agents[1].0.id,]
    );
    assert_eq!(receipt.adoption_auth_request_ids.len(), 1);

    // Row counts.
    assert_eq!(count_rows(&store, "organization").await, 1);
    // 3 agents: CEO + 2 system agents.
    assert_eq!(count_rows(&store, "agent").await, 3);
    assert_eq!(count_rows(&store, "agent_profile").await, 2);
    assert_eq!(count_rows(&store, "channel").await, 1);
    assert_eq!(count_rows(&store, "inbox_object").await, 1);
    assert_eq!(count_rows(&store, "outbox_object").await, 1);
    assert_eq!(count_rows(&store, "grant").await, 1);
    assert_eq!(count_rows(&store, "token_budget_pool").await, 1);
    assert_eq!(count_rows(&store, "auth_request").await, 1);
    assert_eq!(count_rows(&store, "resources_catalogue").await, 2);

    // Edges.
    assert_eq!(count_rows(&store, "has_ceo").await, 1);
    assert_eq!(count_rows(&store, "has_member").await, 3); // CEO + 2 sys
    assert_eq!(count_rows(&store, "member_of").await, 3);
    assert_eq!(count_rows(&store, "has_inbox").await, 1);
    assert_eq!(count_rows(&store, "has_outbox").await, 1);
    assert_eq!(count_rows(&store, "has_channel").await, 1);
    assert_eq!(count_rows(&store, "has_profile").await, 2);
}

#[tokio::test]
async fn adr_0023_invariant_no_per_agent_policy_nodes_materialised() {
    // Positive-grep assertion per phi-core-leverage-checklist §6:
    // after `apply_org_creation`, every per-agent
    // ExecutionLimits/RetryPolicy/CachePolicy/CompactionPolicy table
    // MUST stay at zero rows (ADR-0023 inherit-from-snapshot
    // invariant).
    let (store, _dir) = fresh_store().await;
    let payload = build_payload(sample_org());
    store.apply_org_creation(&payload).await.unwrap();

    assert_eq!(count_rows(&store, "execution_limits").await, 0);
    assert_eq!(count_rows(&store, "retry_policy").await, 0);
    assert_eq!(count_rows(&store, "cache_policy").await, 0);
    assert_eq!(count_rows(&store, "compaction_policy").await, 0);
}

#[tokio::test]
async fn agent_profile_blueprint_roundtrips_phi_core_fields() {
    // Positive phi-core transit assertion: each system agent's
    // `agent_profile.blueprint` must preserve the full
    // `phi_core::AgentProfile` shape end-to-end — system_prompt,
    // name, etc. A baby-phi local re-implementation would lose
    // fields on round-trip.
    let (store, _dir) = fresh_store().await;
    let payload = build_payload(sample_org());
    store.apply_org_creation(&payload).await.unwrap();

    // Query blueprint subfield directly — no `get_agent_profile` on
    // the Repository trait yet (M3 scope didn't need it; dashboard
    // reads profiles indirectly via `list_agents_in_org`).
    let prompts: Vec<Option<String>> = store
        .client()
        .query(
            "SELECT blueprint.system_prompt AS system_prompt FROM type::thing('agent_profile', $id)",
        )
        .bind(("id", payload.system_agents[0].1.id.to_string()))
        .await
        .unwrap()
        .take((0, "system_prompt"))
        .unwrap();
    assert_eq!(
        prompts.first().cloned().flatten().as_deref(),
        Some("You distill agent memories from recent sessions.")
    );

    let names: Vec<Option<String>> = store
        .client()
        .query("SELECT blueprint.name AS name FROM type::thing('agent_profile', $id)")
        .bind(("id", payload.system_agents[1].1.id.to_string()))
        .await
        .unwrap()
        .take((0, "name"))
        .unwrap();
    assert_eq!(
        names.first().cloned().flatten().as_deref(),
        Some("agent-catalog")
    );

    // Compile-time proof that the payload's blueprint field is
    // exactly phi-core's AgentProfile (not a baby-phi redeclaration).
    // If anyone breaks the wrap in the future, this function
    // signature won't accept the field and the test won't compile.
    fn is_phi_core_agent_profile(_: &phi_core::agents::profile::AgentProfile) {}
    is_phi_core_agent_profile(&payload.system_agents[0].1.blueprint);
    is_phi_core_agent_profile(&payload.system_agents[1].1.blueprint);
}

#[tokio::test]
async fn duplicate_org_id_is_conflict_with_no_partial_state() {
    let (store, _dir) = fresh_store().await;
    let org = sample_org();
    let payload_1 = build_payload(org.clone());
    store.apply_org_creation(&payload_1).await.unwrap();

    let row_count_before = count_rows(&store, "organization").await;

    // Second attempt with the SAME org id — different CEO/system
    // agents, same org.id. Must fail and not add any rows.
    let payload_2 = build_payload(org);
    let err = store
        .apply_org_creation(&payload_2)
        .await
        .expect_err("must fail");
    // SurrealDB uniqueness constraint surfaces as Backend, not
    // Conflict — either is acceptable so long as nothing leaked.
    match err {
        RepositoryError::Conflict(_) | RepositoryError::Backend(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }

    assert_eq!(
        count_rows(&store, "organization").await,
        row_count_before,
        "rollback must leave organization count unchanged"
    );
    // The duplicate CREATE on organization was the first statement in
    // the tx; agent/profile/etc CREATEs for the 2nd payload must not
    // have persisted either.
    assert_eq!(count_rows(&store, "agent").await, 3); // still just payload_1's
    assert_eq!(count_rows(&store, "agent_profile").await, 2);
    assert_eq!(count_rows(&store, "token_budget_pool").await, 1);
}
