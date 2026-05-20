//! CH-28 / ADR-0063 §D63.13 + §D63.14 + §D63.15 — factory helper for the
//! acceptance suite covering the AgentProfile cardinality redesign
//! (1:1 → N:1 hybrid Blueprint table).
//!
//! Per P2-ACCEPTANCE deliverable 1 (iter-5 plan §7) this helper is
//! `OPTIONAL but planner-recommended` — the per-test setup brevity is
//! the only motivation. Tests that don't need to exercise the override-
//! Blueprint round-trip path (e.g., the cardinality-flip + edge-rename
//! tests) construct `AgentProfile` directly via struct literals per
//! §D63.13.
//!
//! Signature shape: `make_test_profile_with_overrides(agent_id,
//! parallelize, model_config_id, mock_response) -> (AgentProfile,
//! Blueprint)`. The returned `AgentProfile` carries the 3 override
//! fields populated verbatim; the returned `Blueprint` is the
//! override-tier (`agent_id = Some(agent_id)`) row that the composite-
//! write at `upsert_agent_profile` would emit. Callers `assert_eq!`
//! the synthesized read against the input to verify §D63.15.

#![allow(dead_code)]

use chrono::Utc;
use domain::model::ids::{AgentId, NodeId};
use domain::model::nodes::{AgentProfile, Blueprint, BlueprintId};

/// Build an (AgentProfile, Blueprint) pair carrying the supplied
/// override field values. The Blueprint row mirrors the override-tier
/// shape (`agent_id = Some(_)`) that the composite-write at
/// `upsert_agent_profile` would persist per ADR-0063 §D63.14 + §D63.15.
///
/// The returned `AgentProfile` and `Blueprint` share the same
/// `created_at` timestamp + the same `agent_id`, mirroring the
/// in-process invariant the composite-write maintains.
pub fn make_test_profile_with_overrides(
    agent_id: AgentId,
    parallelize: u32,
    model_config_id: Option<String>,
    mock_response: Option<String>,
) -> (AgentProfile, Blueprint) {
    let now = Utc::now();
    let profile = AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: model_config_id.clone(),
        mock_response: mock_response.clone(),
        created_at: now,
    };
    let override_bp = Blueprint {
        id: BlueprintId::new(),
        agent_id: Some(agent_id),
        parallelize: Some(parallelize),
        model_config_id,
        mock_response,
        created_at: now,
    };
    (profile, override_bp)
}
