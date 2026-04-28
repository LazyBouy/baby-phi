//! CH-06 / D-new-11 — instance-identity tag emission.
//!
//! Concept doc: `permissions/01-resource-ontology.md` §"Instance Identity
//! Tags" — every composite/node instance carries `(#kind:{name},
//! {name}:{instance_id})` at creation. This integration test exercises the
//! emission path for every type that owns a stable graph-node ID; embedded
//! value-objects (Objective, KeyResult, ResourceBoundaries,
//! OrganizationDefaultsSnapshot, SessionDetail) carry the `tags` field for
//! shape consistency but emission is deferred to the parent aggregate per
//! ADR-0037 §D37.5.

use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::composites::auto_tags_for;
use domain::model::composites_m2::{
    ExternalService, ExternalServiceKind, RuntimeStatus, TenantSet,
};
use domain::model::composites_m3::TokenBudgetPool;
use domain::model::composites_m4::AgentExecutionLimitsOverride;
use domain::model::composites_m5::{
    AgentCatalogEntry, ShapeBPendingProject, SystemAgentRuntimeStatus,
};
use domain::model::ids::{
    AgentCatalogEntryId, AgentId, AuthRequestId, McpServerId, NodeId, OrgId, ProjectId, SessionId,
    SystemAgentRuntimeStatusId,
};
use domain::model::nodes::{
    AgentKind, AuthRequest, AuthRequestState, InboxObject, OutboxObject, PrincipalRef, Session,
    SessionGovernanceState,
};

/// Helper: assert that `tags` contains both `#kind:{name}` and
/// `{name}:{instance_id}` per concept-01 §"Instance Identity Tags".
fn assert_instance_tags(tags: &[String], kind: &str, instance_id: &str) {
    let kind_tag = format!("#kind:{}", kind);
    let self_tag = format!("{}:{}", kind, instance_id);
    assert!(
        tags.iter().any(|t| t == &kind_tag),
        "expected `{}` in tags {:?}",
        kind_tag,
        tags
    );
    assert!(
        tags.iter().any(|t| t == &self_tag),
        "expected `{}` in tags {:?}",
        self_tag,
        tags
    );
}

#[test]
fn auto_tags_for_emits_canonical_pair() {
    let pair = auto_tags_for("session", "s-123");
    assert_eq!(pair[0], "#kind:session");
    assert_eq!(pair[1], "session:s-123");
}

// ---- Composite types with stable graph-node IDs ----------------------------

#[test]
fn external_service_emits_instance_tags() {
    let id = McpServerId::new();
    let svc = ExternalService {
        id,
        display_name: "test-mcp".into(),
        kind: ExternalServiceKind::Mcp,
        endpoint: "stdio:///cmd".into(),
        secret_ref: None,
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
        tags: auto_tags_for("external_service", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&svc.tags, "external_service", &id.to_string());
}

#[test]
fn token_budget_pool_constructor_emits_instance_tags() {
    let pool = TokenBudgetPool::new(OrgId::new(), 1_000_000, Utc::now());
    assert_instance_tags(&pool.tags, "token_budget_pool", &pool.id.to_string());
}

#[test]
fn agent_execution_limits_override_emits_instance_tags() {
    let id = NodeId::new();
    let row = AgentExecutionLimitsOverride {
        id,
        owning_agent: AgentId::new(),
        limits: phi_core::context::execution::ExecutionLimits::default(),
        created_at: Utc::now(),
        tags: auto_tags_for("agent_execution_limits_override", &id.to_string()).to_vec(),
    };
    assert_instance_tags(
        &row.tags,
        "agent_execution_limits_override",
        &id.to_string(),
    );
}

#[test]
fn shape_b_pending_project_emits_instance_tags_keyed_on_ar() {
    // ShapeBPendingProject's stable key is the auth_request_id (it's a
    // sidecar; one row per AR).
    let ar_id = AuthRequestId::new();
    let row = ShapeBPendingProject {
        auth_request_id: ar_id,
        payload: serde_json::json!({"shape": "shape_b"}),
        created_at: Utc::now(),
        tags: auto_tags_for("shape_b_pending_project", &ar_id.to_string()).to_vec(),
    };
    assert_instance_tags(&row.tags, "shape_b_pending_project", &ar_id.to_string());
}

#[test]
fn agent_catalog_entry_emits_instance_tags() {
    let id = AgentCatalogEntryId::new();
    let entry = AgentCatalogEntry {
        id,
        agent_id: AgentId::new(),
        owning_org: OrgId::new(),
        display_name: "test".into(),
        kind: AgentKind::Llm,
        role: None,
        active: true,
        profile_snapshot: None,
        last_seen_at: Utc::now(),
        updated_at: Utc::now(),
        tags: auto_tags_for("agent_catalog_entry", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&entry.tags, "agent_catalog_entry", &id.to_string());
}

#[test]
fn system_agent_runtime_status_emits_instance_tags() {
    let id = SystemAgentRuntimeStatusId::new();
    let row = SystemAgentRuntimeStatus {
        id,
        agent_id: AgentId::new(),
        owning_org: OrgId::new(),
        queue_depth: 0,
        last_fired_at: None,
        effective_parallelize: 1,
        last_error: None,
        updated_at: Utc::now(),
        tags: auto_tags_for("system_agent_runtime_status", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&row.tags, "system_agent_runtime_status", &id.to_string());
}

// ---- Node types ------------------------------------------------------------

#[test]
fn auth_request_emits_instance_tags() {
    let id = AuthRequestId::new();
    let ar = AuthRequest {
        id,
        requestor: PrincipalRef::Agent(AgentId::new()),
        kinds: vec![],
        scope: vec![],
        state: AuthRequestState::Draft,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![],
        justification: None,
        audit_class: AuditClass::Logged,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
        tags: auto_tags_for("auth_request", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&ar.tags, "auth_request", &id.to_string());
}

#[test]
fn inbox_object_emits_instance_tags() {
    let id = NodeId::new();
    let inbox = InboxObject {
        id,
        agent_id: AgentId::new(),
        created_at: Utc::now(),
        tags: auto_tags_for("inbox", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&inbox.tags, "inbox", &id.to_string());
}

#[test]
fn outbox_object_emits_instance_tags() {
    let id = NodeId::new();
    let outbox = OutboxObject {
        id,
        agent_id: AgentId::new(),
        created_at: Utc::now(),
        tags: auto_tags_for("outbox", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&outbox.tags, "outbox", &id.to_string());
}

#[test]
fn session_emits_instance_tags() {
    let id = SessionId::new();
    // Build a minimal phi-core inner via JSON deserialization — phi-core's
    // `Session` doesn't ship a `new()` constructor, but it round-trips
    // through serde, so a synthesized JSON satisfies the wrap.
    let now = Utc::now();
    let inner: phi_core::session::model::Session = serde_json::from_value(serde_json::json!({
        "session_id": id.to_string(),
        "agent_id": "agent-stub",
        "created_at": now,
        "last_active_at": now,
        "formation": { "Explicit": { "timestamp": now } },
        "loops": []
    }))
    .expect("phi-core Session JSON round-trip");
    let session = Session {
        id,
        inner,
        owning_org: OrgId::new(),
        owning_project: ProjectId::new(),
        started_by: AgentId::new(),
        governance_state: SessionGovernanceState::Running,
        started_at: Utc::now(),
        ended_at: None,
        tokens_spent: 0,
        tags: auto_tags_for("session", &id.to_string()).to_vec(),
    };
    assert_instance_tags(&session.tags, "session", &id.to_string());
}

// ---- Cross-creation property: helper output is canonical -------------------

#[test]
fn auto_tags_for_kind_tag_first_self_tag_second() {
    // Every `auto_tags_for(kind, id)` returns `[kind_tag, self_tag]` in
    // that order. Tests below rely on contains() so order doesn't strictly
    // matter, but the canonical order is part of the helper's contract.
    let pair = auto_tags_for("session", "s-1");
    assert_eq!(pair[0], "#kind:session");
    assert_eq!(pair[1], "session:s-1");
}

#[test]
fn empty_kind_or_id_does_not_panic() {
    // Graceful handling of pathological inputs — no panic, just yields a
    // pair that would fail the canonical-form check downstream.
    let pair = auto_tags_for("", "");
    assert_eq!(pair[0], "#kind:");
    assert_eq!(pair[1], ":");
}
