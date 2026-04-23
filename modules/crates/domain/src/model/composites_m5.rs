//! M5 composite + value-object types — the embedded records M5's new
//! surfaces persist on top of M4's compound-tx + event-bus
//! infrastructure.
//!
//! Ontology source of truth:
//! - `docs/specs/v0/concepts/project.md §Shape B` (pending sidecar)
//! - `docs/specs/v0/concepts/system-agents.md` (catalog + runtime status)
//!
//! ## phi-core leverage
//!
//! This file is **phi-core-import-free by design**. The Session / LoopRecord
//! / Turn wraps live in [`crate::model::nodes`] (the governance-node
//! tier); everything here is pure phi governance plumbing
//! (`ShapeBPendingProject` payload sidecar, `AgentCatalogEntry` s03
//! cache row, `SystemAgentRuntimeStatus` page 13 live-status tile).
//!
//! See [ADR-0029](../../../../../../docs/specs/v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md)
//! for the Session/LoopRecord/Turn wrap decision,
//! [ADR-0030](../../../../../../docs/specs/v0/implementation/m5/decisions/0030-template-node-uniqueness.md)
//! for Template uniqueness, and
//! [ADR-0031](../../../../../../docs/specs/v0/implementation/m5/decisions/0031-session-cancellation-and-concurrency.md)
//! for session cancellation + concurrency.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::ids::{
    AgentCatalogEntryId, AgentId, AuthRequestId, LoopId, OrgId, SystemAgentRuntimeStatusId,
};
use super::nodes::{AgentKind, LoopRecordNode, Session, TurnNode};

// ============================================================================
// SessionDetail — drill-down aggregate (session + loops + turns-by-loop).
// ============================================================================

/// Full drill-down for a single `Session`. Returned by
/// [`Repository::fetch_session`] to populate the per-session detail
/// view at M5/P4 + page 11's "Recent sessions" flyout panel.
///
/// Reconstructs the nested `phi_core::Session.loops: Vec<LoopRecord>`
/// tree from baby-phi's flattened storage layout (one row per tier
/// in SurrealDB per migration 0005). Per-Turn queries against the
/// flat `turn` table stay O(1); materialising the full tree is
/// one compound SELECT per tier + an in-process group-by.
///
/// `turns_by_loop` uses `BTreeMap` so iteration order is stable
/// across drill-downs regardless of SurrealDB's row-return order —
/// acceptance tests can pin the exact output shape.
///
/// Does NOT derive `PartialEq` — the inner phi-core `Session` /
/// `LoopRecord` / `Turn` types don't derive equality, and deep
/// equality on a full session tree is rarely what tests want
/// anyway. Tests compare ids + field-level invariants instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    pub session: Session,
    pub loops: Vec<LoopRecordNode>,
    /// Turns keyed by their parent `LoopId`. Every `LoopId` present
    /// in `loops` appears as a key (possibly with an empty vec
    /// for loops that ended before any turn completed).
    pub turns_by_loop: BTreeMap<LoopId, Vec<TurnNode>>,
}

// ============================================================================
// ShapeBPendingProject — sidecar payload for Shape B project creation (C-M5-6).
// ============================================================================

/// The pending-payload sidecar Shape B project creation writes at submit
/// time, read back at both-approve time to call `materialise_project`.
///
/// Lives in the `shape_b_pending_projects` SurrealDB table (migration
/// 0005) with `UNIQUE(auth_request_id)`. Absence of a row for an AR
/// means either (a) the AR was never Shape B or (b) the Approved
/// branch already consumed + deleted the sidecar. Callers that
/// observe `AR = Approved` + `sidecar = None` treat the project as
/// already materialised (idempotent).
///
/// `payload` is stored as a [`serde_json::Value`] because the full
/// `CreateProjectInput` type lives in the `server` crate; pushing the
/// serialised JSON through the governance tier avoids an upward
/// dependency. The server's submit + approve handlers (wired at M5/P4)
/// serialise / deserialise the `CreateProjectInput` here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeBPendingProject {
    pub auth_request_id: AuthRequestId,
    /// Serialised `CreateProjectInput`. FLEXIBLE on the storage side
    /// (migration 0005) so the input shape can evolve without a
    /// migration 0006.
    pub payload: JsonValue,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// AgentCatalogEntry — s03 catalogue cache row.
// ============================================================================

/// One catalogue cache row per agent per org, maintained by the s03
/// listener (`AgentCatalogListener`). Page 07 dashboard roll-ups + M6
/// a05 grants view read this instead of the raw `agent` table so
/// queries stay O(1) in the number of orgs.
///
/// Upsert triggers:
/// - `AgentCreated` — fresh row.
/// - `AgentArchived` — `active = false`.
/// - `HasLeadEdgeCreated` / `ManagesEdgeCreated` /
///   `HasAgentSupervisorEdgeCreated` — role-index refresh.
/// - `HasProfileEdgeChanged` — `profile_snapshot` refresh.
///
/// The `profile_snapshot` is a serialised snapshot of the agent's
/// `AgentProfile` at last-catalog-refresh time (kept as JSON to avoid
/// pulling phi-core types into the composites tier).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCatalogEntry {
    pub id: AgentCatalogEntryId,
    pub agent_id: AgentId,
    pub owning_org: OrgId,
    pub display_name: String,
    pub kind: AgentKind,
    /// Role per the 6-variant enum; `None` = unclassified.
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default = "default_active_true")]
    pub active: bool,
    /// Serialised snapshot of the agent's governance `AgentProfile` at
    /// last refresh. `None` before the first `HasProfileEdgeChanged`.
    #[serde(default)]
    pub profile_snapshot: Option<JsonValue>,
    pub last_seen_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_active_true() -> bool {
    true
}

// ============================================================================
// SystemAgentRuntimeStatus — page 13 live-status tile.
// ============================================================================

/// One live-status row per system agent per org. Upserted by all 5
/// listeners (Template A / C / D fire listeners + memory-extraction +
/// agent-catalog) via a shared helper at M5/P3 — whenever a listener
/// fires, it updates the corresponding status tile.
///
/// Page 13 polls this via the list endpoint (R-ADMIN-13-R2 / N3); live
/// queue-depth + last-fired-at drive the operator experience. Not to
/// be confused with the per-agent `ExecutionLimits` override row on
/// `agent_execution_limits` (M4/ADR-0027) — that is blueprint
/// configuration; this is runtime telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemAgentRuntimeStatus {
    pub id: SystemAgentRuntimeStatusId,
    pub agent_id: AgentId,
    pub owning_org: OrgId,
    /// Number of pending events currently queued against this agent.
    /// 0 = idle.
    #[serde(default)]
    pub queue_depth: u32,
    /// Wall-clock of the most recent listener fire that targeted this
    /// agent. `None` before the first fire.
    #[serde(default)]
    pub last_fired_at: Option<DateTime<Utc>>,
    /// Effective parallelize after org snapshot + per-agent override
    /// resolution — pinned here so page 13 can show the operator the
    /// currently-active cap without a second lookup.
    pub effective_parallelize: u32,
    /// Populated when the most recent fire errored; cleared on the
    /// next successful fire.
    #[serde(default)]
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentCatalogEntryId, AgentId, AuthRequestId, OrgId};

    #[test]
    fn shape_b_pending_project_round_trips() {
        let row = ShapeBPendingProject {
            auth_request_id: AuthRequestId::new(),
            payload: serde_json::json!({ "name": "demo", "shape": "shape_b" }),
            created_at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&row).expect("serialize");
        let back: ShapeBPendingProject = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(row, back);
    }

    #[test]
    fn agent_catalog_entry_round_trips() {
        let row = AgentCatalogEntry {
            id: AgentCatalogEntryId::new(),
            agent_id: AgentId::new(),
            owning_org: OrgId::new(),
            display_name: "Alex".into(),
            kind: AgentKind::Human,
            role: Some("executive".into()),
            active: true,
            profile_snapshot: None,
            last_seen_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&row).expect("serialize");
        let back: AgentCatalogEntry = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(row, back);
    }

    #[test]
    fn agent_catalog_entry_defaults_active_true_for_pre_m5_style_rows() {
        // An upsert path that omits the `active` field (e.g. a probe
        // writer that only knows agent_id + display_name) must land
        // with active = true via serde default.
        let partial = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "agent_id": "00000000-0000-0000-0000-000000000002",
            "owning_org": "00000000-0000-0000-0000-000000000003",
            "display_name": "probe",
            "kind": "llm",
            "last_seen_at": "2026-04-23T00:00:00Z",
            "updated_at": "2026-04-23T00:00:00Z"
        });
        let row: AgentCatalogEntry =
            serde_json::from_value(partial).expect("deserialize partial row");
        assert!(row.active);
        assert_eq!(row.role, None);
        assert_eq!(row.profile_snapshot, None);
    }

    #[test]
    fn system_agent_runtime_status_round_trips() {
        let row = SystemAgentRuntimeStatus {
            id: SystemAgentRuntimeStatusId::new(),
            agent_id: AgentId::new(),
            owning_org: OrgId::new(),
            queue_depth: 0,
            last_fired_at: None,
            effective_parallelize: 1,
            last_error: None,
            updated_at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&row).expect("serialize");
        let back: SystemAgentRuntimeStatus = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(row, back);
    }
}
