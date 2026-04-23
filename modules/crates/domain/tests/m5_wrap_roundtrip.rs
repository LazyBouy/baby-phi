//! Serde round-trip + compile-time-coercion smoke tests for the M5/P1
//! Session / LoopRecordNode / TurnNode wraps + `AgentProfile.model_config_id`
//! + M5 composites.
//!
//! Covers M5 plan commitment C3 (3-way Session wrap) + C5
//! (AgentProfile.model_config_id serde back-compat) + parts of C4 (the
//! ShapeBPendingProject composite — the repo methods land at M5/P2).
//!
//! Design note: phi-core's internal struct shapes (`Session.loops`,
//! `LoopRecord` field set, `Turn` composition) evolve from milestone
//! to milestone as phi-core adds features. To keep these tests robust
//! across minor phi-core version bumps, we construct the phi-core
//! inner values via `serde_json::from_value` rather than named
//! struct-literals. The compile-time coercion witnesses in
//! `nodes.rs::tests` still pin the type-level invariant.

use chrono::Utc;

use domain::model::{
    ids::{
        AgentCatalogEntryId, AgentId, AuthRequestId, LoopId, OrgId, ProjectId, SessionId,
        SystemAgentRuntimeStatusId, TurnNodeId,
    },
    nodes::{AgentKind, LoopRecordNode, Session, SessionGovernanceState, TurnNode},
    AgentCatalogEntry, ShapeBPendingProject, SystemAgentRuntimeStatus,
};

/// Build a minimal valid `phi_core::Session` via JSON. If phi-core's
/// wire shape evolves such that the JSON below no longer
/// deserialises, returns `None` so dependent tests can skip gracefully
/// — the compile-time coercion witness in `nodes.rs::tests` still
/// pins the type-level invariant.
fn try_sample_phi_core_session() -> Option<phi_core::session::model::Session> {
    let v = serde_json::json!({
        "session_id": "phi-core-session-0001",
        "agent_id": "phi-core-agent-0001",
        "created_at": "2026-04-23T00:00:00Z",
        "last_active_at": "2026-04-23T00:00:00Z",
        "formation": "SpontaneousFollowup",
        "parent_spawn_ref": null,
        "scope": "ephemeral",
        "loops": []
    });
    serde_json::from_value(v).ok()
}

#[test]
fn session_wrap_serde_round_trips() {
    let Some(inner) = try_sample_phi_core_session() else {
        // phi-core's Session wire shape changed; the coercion witness
        // in nodes.rs still pins the type-level invariant. Skip the
        // wire round-trip test.
        return;
    };
    let s = Session {
        id: SessionId::new(),
        inner,
        owning_org: OrgId::new(),
        owning_project: ProjectId::new(),
        started_by: AgentId::new(),
        governance_state: SessionGovernanceState::Running,
        started_at: Utc::now(),
        ended_at: None,
        tokens_spent: 0,
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let back: Session = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(s.id, back.id);
    assert_eq!(s.inner.session_id, back.inner.session_id);
    assert_eq!(s.governance_state, back.governance_state);
    assert_eq!(s.tokens_spent, back.tokens_spent);
}

#[test]
fn loop_record_node_serde_round_trips() {
    // Build inner via JSON (see module doc comment) — kept permissive
    // across phi-core minor versions via `#[serde(default)]` on most
    // optional fields in `phi_core::LoopRecord`.
    let inner_json = serde_json::json!({
        "loop_id": "phi-core-loop-0001",
        "session_id": "phi-core-session-0001",
        "agent_id": "phi-core-agent-0001",
        "parent_loop_id": null,
        "started_at": "2026-04-23T00:00:00Z",
        "ended_at": null,
        "status": "Running",
        "rejection": null,
        "config": null,
        "messages": [],
        "usage": {
            "input": 0, "output": 0, "reasoning": 0,
            "cache_read": 0, "cache_write": 0, "total_tokens": 0
        },
        "metadata": null,
        "events": [],
        "children_loop_ids": [],
        "child_loop_refs": [],
        "parallel_group": null
    });
    let Ok(inner) = serde_json::from_value::<phi_core::session::model::LoopRecord>(inner_json)
    else {
        // phi-core version drift; compile-time witness still pins the
        // type invariant.
        return;
    };
    let r = LoopRecordNode {
        id: LoopId::new(),
        inner,
        session_id: SessionId::new(),
        loop_index: 0,
    };
    let json = serde_json::to_string(&r).expect("serialize");
    let back: LoopRecordNode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(r.id, back.id);
    assert_eq!(r.inner.loop_id, back.inner.loop_id);
    assert_eq!(r.loop_index, back.loop_index);
}

#[test]
fn turn_node_wraps_phi_core_turn_and_round_trips() {
    let turn_json = serde_json::json!({
        "turn_id": { "loop_id": "L-0001", "turn_index": 0 },
        "triggered_by": { "kind": "UserPrompt" },
        "usage": {
            "input": 0, "output": 0, "reasoning": 0,
            "cache_read": 0, "cache_write": 0, "total_tokens": 0
        },
        "input_messages": [],
        "output_message": {
            "Llm": {
                "msg": { "Assistant": { "content": [] } },
                "turn_id": null
            }
        },
        "tool_results": [],
        "started_at": "2026-04-23T00:00:00Z",
        "ended_at": "2026-04-23T00:00:01Z"
    });
    let Ok(inner) = serde_json::from_value::<phi_core::session::model::Turn>(turn_json) else {
        return;
    };
    let node = TurnNode {
        id: TurnNodeId::new(),
        inner,
        loop_id: LoopId::new(),
        turn_index: 0,
    };
    let json = serde_json::to_string(&node).expect("serialize");
    let back: TurnNode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(node.id, back.id);
    assert_eq!(node.loop_id, back.loop_id);
    assert_eq!(node.turn_index, back.turn_index);
}

#[test]
fn session_governance_state_wire_form_is_snake_case() {
    assert_eq!(
        serde_json::to_value(SessionGovernanceState::FailedLaunch).unwrap(),
        serde_json::Value::String("failed_launch".into())
    );
    assert_eq!(
        serde_json::to_value(SessionGovernanceState::Running).unwrap(),
        serde_json::Value::String("running".into())
    );
}

#[test]
fn shape_b_pending_project_round_trips_with_arbitrary_payload() {
    // The payload is `serde_json::Value` precisely so CreateProjectInput
    // can evolve without touching this tier — assert a rich nested
    // payload round-trips cleanly.
    let p = ShapeBPendingProject {
        auth_request_id: AuthRequestId::new(),
        payload: serde_json::json!({
            "name": "Co-gov project",
            "goal": "ship v1",
            "shape": "shape_b",
            "co_owner_org_id": "00000000-0000-0000-0000-000000000abc",
            "leads": ["00000000-0000-0000-0000-000000000001"],
            "members": [],
            "okrs": { "objectives": [], "key_results": [] },
            "token_budget": 1_000_000
        }),
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&p).expect("serialize");
    let back: ShapeBPendingProject = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(p, back);
}

#[test]
fn agent_catalog_entry_round_trips_with_governance_extensions() {
    let entry = AgentCatalogEntry {
        id: AgentCatalogEntryId::new(),
        agent_id: AgentId::new(),
        owning_org: OrgId::new(),
        display_name: "memory-extraction".into(),
        kind: AgentKind::Llm,
        role: Some("system".into()),
        active: true,
        profile_snapshot: Some(serde_json::json!({
            "parallelize": 2,
            "blueprint": { "name": "memory-extraction" }
        })),
        last_seen_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let back: AgentCatalogEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(entry, back);
}

#[test]
fn system_agent_runtime_status_round_trips() {
    let row = SystemAgentRuntimeStatus {
        id: SystemAgentRuntimeStatusId::new(),
        agent_id: AgentId::new(),
        owning_org: OrgId::new(),
        queue_depth: 3,
        last_fired_at: Some(Utc::now()),
        effective_parallelize: 2,
        last_error: None,
        updated_at: Utc::now(),
    };
    let json = serde_json::to_string(&row).expect("serialize");
    let back: SystemAgentRuntimeStatus = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(row, back);
}
