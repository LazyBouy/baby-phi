//! CH-21 / ADR-0040 + ADR-0041 — end-to-end memory-extraction
//! acceptance test. Exercises the full `MemoryExtractionListener`
//! body wired through `build_event_bus_with_m5_listeners` + the
//! in-memory Repository + a capturing audit emitter, asserting the
//! six P2-locked scenarios from the chunk plan §7 P2.
//!
//! Scenarios:
//! 1. Happy path: SessionEnded → Memory minted, Identity counter
//!    bumped, two audits emitted in order.
//! 2. SessionAborted: governance_state == Aborted → no extraction.
//! 3. Disabled extractor system agent → no extraction + no telemetry
//!    fire (ADR-0040 §D40.3 SKIP-BOTH).
//! 4. Human-kind working agent → no extraction (CH-16: no Identity).
//! 5. Scope public bucket: `#public` in session.tags → public bucket.
//! 6. Audit chain ordering: memory.extracted precedes identity.updated.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};

use domain::audit::events::m5_2::memory::ExtractionScope;
use domain::audit::{AuditClass, AuditEmitter, AuditEvent, NoopAuditEmitter};
use domain::events::{DomainEvent, EventBus, MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME};
use domain::in_memory::InMemoryRepository;
use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, AuditEventId, LoopId, OrgId, ProjectId, SessionId};
use domain::model::nodes::{
    Agent, AgentKind, AgentRole, Identity, LoopRecordNode, Organization, Session,
    SessionGovernanceState,
};
use domain::repository::{Repository, RepositoryResult};
use server::state::build_event_bus_with_m5_listeners;

#[derive(Default)]
struct CapturingAudit {
    events: Mutex<Vec<AuditEvent>>,
}

#[async_trait]
impl AuditEmitter for CapturingAudit {
    async fn emit(&self, event: AuditEvent) -> RepositoryResult<()> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

fn sample_phi_core_session() -> phi_core::session::model::Session {
    serde_json::from_value(serde_json::json!({
        "session_id": "s-acc-mem",
        "agent_id": "a-acc-mem",
        "created_at": "2026-04-28T00:00:00Z",
        "last_active_at": "2026-04-28T00:00:00Z",
        "formation": {"Explicit": {"timestamp": "2026-04-28T00:00:00Z"}},
        "parent_spawn_ref": null,
        "scope": "ephemeral",
        "loops": []
    }))
    .expect("phi-core Session JSON deserialises")
}

fn sample_phi_core_loop_record() -> phi_core::session::model::LoopRecord {
    serde_json::from_value(serde_json::json!({
        "loop_id": "l-acc-mem",
        "session_id": "s-acc-mem",
        "agent_id": "a-acc-mem",
        "parent_loop_id": null,
        "started_at": "2026-04-28T00:00:00Z",
        "ended_at": null,
        "status": "Running",
        "rejection": null,
        "config": null,
        "messages": [],
        "usage": {"input": 0, "output": 0, "reasoning": 0, "cache_read": 0, "cache_write": 0, "total_tokens": 0},
        "metadata": null,
        "events": [],
        "children_loop_ids": [],
        "child_loop_refs": [],
        "parallel_group": null
    }))
    .expect("phi-core LoopRecord JSON deserialises")
}

struct AcceptanceFixture {
    repo: Arc<dyn Repository>,
    audit: Arc<CapturingAudit>,
    bus: Arc<dyn EventBus>,
    org_id: OrgId,
    extractor_agent_id: AgentId,
    working_agent_id: AgentId,
    project_id: ProjectId,
    session_id: SessionId,
    ended_at: chrono::DateTime<Utc>,
}

async fn fixture(
    extractor_active: bool,
    working_kind: AgentKind,
    session_extra_tags: Vec<String>,
    final_state: SessionGovernanceState,
) -> AcceptanceFixture {
    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let audit = Arc::new(CapturingAudit::default());
    let now = Utc.with_ymd_and_hms(2026, 4, 28, 12, 0, 0).unwrap();

    let org_id = OrgId::new();
    let extractor_agent_id = AgentId::new();
    let working_agent_id = AgentId::new();
    let project_id = ProjectId::new();
    let session_id = SessionId::new();

    let extractor = Agent {
        id: extractor_agent_id,
        kind: AgentKind::Llm,
        display_name: MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME.to_string(),
        owning_org: Some(org_id),
        role: Some(AgentRole::System),
        created_at: now,
        active: extractor_active,
        archived_at: None,
    };
    repo.create_agent(&extractor).await.unwrap();

    let working = Agent {
        id: working_agent_id,
        kind: working_kind,
        display_name: "test-worker".into(),
        owning_org: Some(org_id),
        role: Some(AgentRole::Member),
        created_at: now,
        active: true,
        archived_at: None,
    };
    repo.create_agent(&working).await.unwrap();

    if working_kind == AgentKind::Llm {
        let iden = Identity::default_for_llm(working_agent_id, now);
        repo.upsert_identity(&iden).await.unwrap();
    }

    let org = Organization {
        id: org_id,
        display_name: "test-org".into(),
        vision: None,
        mission: None,
        consent_policy: ConsentPolicy::Implicit,
        audit_class_default: AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![extractor_agent_id],
        created_at: now,
    };
    repo.create_organization(&org).await.unwrap();

    let mut session_tags = vec![format!("session:{}", session_id), "#kind:session".into()];
    session_tags.extend(session_extra_tags);
    let session = Session {
        id: session_id,
        inner: sample_phi_core_session(),
        owning_org: org_id,
        owning_project: project_id,
        started_by: working_agent_id,
        governance_state: SessionGovernanceState::Running,
        started_at: now,
        ended_at: None,
        tokens_spent: 0,
        tags: session_tags,
    };
    let lr = LoopRecordNode {
        id: LoopId::new(),
        inner: sample_phi_core_loop_record(),
        session_id,
        loop_index: 0,
    };
    repo.persist_session(&session, &lr).await.unwrap();
    repo.mark_session_ended(session_id, now, final_state)
        .await
        .unwrap();

    // Wire the full M5 + CH-21 listener set via the production
    // helper. Verifies the listener body is actually invoked through
    // the bus the server uses, not via a hand-built test bus.
    let bus = build_event_bus_with_m5_listeners(
        repo.clone(),
        audit.clone() as Arc<dyn AuditEmitter>,
        domain::events::CatalogAuditMode::default(),
    ) as Arc<dyn EventBus>;

    AcceptanceFixture {
        repo,
        audit,
        bus,
        org_id,
        extractor_agent_id,
        working_agent_id,
        project_id,
        session_id,
        ended_at: now,
    }
}

fn session_ended_event(f: &AcceptanceFixture) -> DomainEvent {
    DomainEvent::SessionEnded {
        session_id: f.session_id,
        agent_id: f.working_agent_id,
        project_id: f.project_id,
        ended_at: f.ended_at,
        duration_ms: 1_000,
        turn_count: 2,
        tokens_spent: 256,
        event_id: AuditEventId::new(),
    }
}

// ---- Scenario 1 — happy path ------------------------------------------------

#[tokio::test]
async fn scenario_1_session_ended_mints_memory_and_bumps_identity() {
    let f = fixture(
        true,
        AgentKind::Llm,
        vec![],
        SessionGovernanceState::Completed,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    // Memory minted.
    let memories = f
        .repo
        .list_memories_for_agent(f.working_agent_id)
        .await
        .unwrap();
    assert_eq!(memories.len(), 1, "exactly one Memory per session at v0");
    assert!(memories[0]
        .tags
        .iter()
        .any(|t| t == &format!("agent:{}", f.working_agent_id)));
    assert!(memories[0]
        .tags
        .iter()
        .any(|t| t == &format!("session:{}", f.session_id)));
    assert!(memories[0]
        .tags
        .iter()
        .any(|t| t == &format!("project:{}", f.project_id)));
    assert!(memories[0]
        .tags
        .iter()
        .any(|t| t == &format!("org:{}", f.org_id)));

    // Identity counter advanced.
    let iden = f
        .repo
        .get_identity(f.working_agent_id)
        .await
        .unwrap()
        .expect("identity row present");
    assert_eq!(iden.witnessed.memories_extracted, 1);
    assert_eq!(iden.witnessed.extraction_scope_distribution.private, 1);
    assert_eq!(iden.witnessed.extraction_scope_distribution.public, 0);

    // Two audit events emitted.
    let events = f.audit.events.lock().unwrap().clone();
    let mem_count = events
        .iter()
        .filter(|e| e.event_type == "platform.memory.extracted")
        .count();
    let id_count = events
        .iter()
        .filter(|e| e.event_type == "platform.identity.updated")
        .count();
    assert_eq!(mem_count, 1);
    assert_eq!(id_count, 1);
}

// ---- Scenario 2 — Aborted session skips extraction --------------------------

#[tokio::test]
async fn scenario_2_aborted_session_skips_extraction() {
    let f = fixture(
        true,
        AgentKind::Llm,
        vec![],
        SessionGovernanceState::Aborted,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    let memories = f
        .repo
        .list_memories_for_agent(f.working_agent_id)
        .await
        .unwrap();
    assert!(memories.is_empty(), "aborted session: no Memory");

    let iden = f
        .repo
        .get_identity(f.working_agent_id)
        .await
        .unwrap()
        .expect("identity row present");
    assert_eq!(iden.witnessed.memories_extracted, 0);

    // The capturing emitter sees zero memory.extracted + zero
    // identity.updated audits from this fire (catalog listener may
    // emit unrelated events if Debug mode were on; default Silent).
    let events = f.audit.events.lock().unwrap().clone();
    assert!(events
        .iter()
        .all(|e| e.event_type != "platform.memory.extracted"));
    assert!(events
        .iter()
        .all(|e| e.event_type != "platform.identity.updated"));
}

// ---- Scenario 3 — disabled extractor → SKIP BOTH ----------------------------

#[tokio::test]
async fn scenario_3_disabled_extractor_skips_both_extraction_and_telemetry() {
    let f = fixture(
        false, /* extractor_active */
        AgentKind::Llm,
        vec![],
        SessionGovernanceState::Completed,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    let memories = f
        .repo
        .list_memories_for_agent(f.working_agent_id)
        .await
        .unwrap();
    assert!(
        memories.is_empty(),
        "disabled extractor: no Memory (ADR-0040 §D40.3 SKIP-BOTH)"
    );

    let tile = f
        .repo
        .fetch_system_agent_runtime_status_for_org(f.org_id)
        .await
        .unwrap()
        .into_iter()
        .find(|t| t.agent_id == f.extractor_agent_id);
    assert!(
        tile.is_none(),
        "disabled extractor: no telemetry fire either"
    );

    let events = f.audit.events.lock().unwrap().clone();
    assert!(events
        .iter()
        .all(|e| e.event_type != "platform.memory.extracted"));
}

// ---- Scenario 4 — Human working agent → graceful skip ----------------------

#[tokio::test]
async fn scenario_4_human_working_agent_skips_extraction() {
    let f = fixture(
        true,
        AgentKind::Human,
        vec![],
        SessionGovernanceState::Completed,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    // Human has no Identity row per CH-16.
    assert!(f
        .repo
        .get_identity(f.working_agent_id)
        .await
        .unwrap()
        .is_none());

    // Listener short-circuits before minting Memory or emitting
    // memory-extraction audits.
    let memories = f
        .repo
        .list_memories_for_agent(f.working_agent_id)
        .await
        .unwrap();
    assert!(
        memories.is_empty(),
        "Human-kind: no Memory (extraction is LLM-only at v0)",
    );
    let events = f.audit.events.lock().unwrap().clone();
    assert!(events
        .iter()
        .all(|e| e.event_type != "platform.memory.extracted"));
}

// ---- Scenario 5 — `#public` tag → public bucket ----------------------------

#[tokio::test]
async fn scenario_5_public_tag_increments_public_bucket() {
    let f = fixture(
        true,
        AgentKind::Llm,
        vec!["#public".into()],
        SessionGovernanceState::Completed,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    let iden = f
        .repo
        .get_identity(f.working_agent_id)
        .await
        .unwrap()
        .expect("identity row present");
    assert_eq!(iden.witnessed.memories_extracted, 1);
    assert_eq!(iden.witnessed.extraction_scope_distribution.public, 1);
    assert_eq!(iden.witnessed.extraction_scope_distribution.private, 0);

    // Audit reflects the public bucket.
    let events = f.audit.events.lock().unwrap().clone();
    let mem_audit = events
        .iter()
        .find(|e| e.event_type == "platform.memory.extracted")
        .expect("memory.extracted audit present");
    assert_eq!(
        mem_audit.diff["scope_bucket"].as_str().unwrap(),
        ExtractionScope::Public.as_str()
    );
}

// ---- Scenario 6 — audit chain ordering -------------------------------------

#[tokio::test]
async fn scenario_6_audit_chain_orders_memory_extracted_before_identity_updated() {
    let f = fixture(
        true,
        AgentKind::Llm,
        vec![],
        SessionGovernanceState::Completed,
    )
    .await;
    f.bus.emit(session_ended_event(&f)).await;

    let events = f.audit.events.lock().unwrap().clone();
    let mem_idx = events
        .iter()
        .position(|e| e.event_type == "platform.memory.extracted")
        .expect("memory.extracted present");
    let id_idx = events
        .iter()
        .position(|e| e.event_type == "platform.identity.updated")
        .expect("identity.updated present");
    assert!(
        mem_idx < id_idx,
        "platform.memory.extracted must precede platform.identity.updated \
         in the audit log within the same fire"
    );
    // Both events are scoped to the same org.
    assert_eq!(events[mem_idx].org_scope, Some(f.org_id));
    assert_eq!(events[id_idx].org_scope, Some(f.org_id));
}

// ---- Bonus: the listener-count invariant survives CH-21 -------------------

#[tokio::test]
async fn ch_21_preserves_5_listener_invariant() {
    let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
    let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
    let bus =
        build_event_bus_with_m5_listeners(repo, audit, domain::events::CatalogAuditMode::default());
    assert_eq!(
        bus.handler_count(),
        5,
        "CH-21 changes listener body, not subscription count",
    );
}
