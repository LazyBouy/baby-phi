//! M4 composite + value-object types — the embedded records M4's new
//! vertical slices persist on top of M3's compound-tx infrastructure.
//!
//! Ontology source of truth: `docs/specs/v0/concepts/project.md §OKR` +
//! `docs/specs/v0/concepts/project.md §ResourceBoundaries`.
//!
//! ## phi-core leverage
//!
//! This file is the **single** locus of direct `phi_core::` imports added
//! in M4/P1. The [`AgentExecutionLimitsOverride`] composite wraps
//! [`phi_core::context::execution::ExecutionLimits`] directly — identical
//! discipline to M3/P1's [`crate::model::composites_m3::OrganizationDefaultsSnapshot`]
//! four-way wrap. Every other M4/P1 type is pure phi governance
//! (OKRs and resource boundaries are planning/governance concepts
//! phi-core has no counterpart for).
//!
//! See [ADR-0027](../../../../../../docs/specs/v0/implementation/m4/decisions/0027-per-agent-execution-limits-override.md)
//! for why `ExecutionLimits` ships a per-agent override layer on top of
//! ADR-0023's inherit-from-snapshot default.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{AgentId, McpServerId, ModelProviderId, NodeId};

// ============================================================================
// Objective + KeyResult — embedded OKR value objects on the Project node.
// ============================================================================

/// An Objective value object — a high-level goal the project pursues.
///
/// Embedded on [`crate::model::Project.objectives`] (not an independent
/// graph node per [project.md §OKR](../../../../../../docs/specs/v0/concepts/project.md)).
///
/// **phi-core leverage**: none — OKRs are a planning/governance concept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objective {
    /// Stable id scoped to the owning project — referenced by
    /// [`KeyResult.objective_id`] and by parent-project roll-ups.
    pub objective_id: String,
    pub name: String,
    pub description: String,
    pub status: ObjectiveStatus,
    /// The agent accountable for the objective.
    pub owner: AgentId,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
    /// Links to this project's [`KeyResult`]s by `kr_id`.
    #[serde(default)]
    pub key_result_ids: Vec<String>,
}

/// Objective lifecycle state — explicit transitions only (no silent expiry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveStatus {
    Draft,
    Active,
    Achieved,
    Missed,
    Cancelled,
}

impl ObjectiveStatus {
    pub const ALL: [ObjectiveStatus; 5] = [
        ObjectiveStatus::Draft,
        ObjectiveStatus::Active,
        ObjectiveStatus::Achieved,
        ObjectiveStatus::Missed,
        ObjectiveStatus::Cancelled,
    ];
}

/// A Key Result — measurable outcome that indicates progress toward an
/// [`Objective`]. Each KR tracked on its own schedule; aggregation into the
/// parent Objective's progress is explicit (not inferred).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyResult {
    pub kr_id: String,
    /// The Objective this KR measures (must match an
    /// [`Objective.objective_id`] on the same project).
    pub objective_id: String,
    pub name: String,
    pub description: String,
    pub measurement_type: MeasurementType,
    pub target_value: OkrValue,
    #[serde(default)]
    pub current_value: Option<OkrValue>,
    pub owner: AgentId,
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
    pub status: KeyResultStatus,
}

/// How a [`KeyResult`] is measured. Gates the shape of
/// [`KeyResult.target_value`] / [`KeyResult.current_value`] via
/// [`MeasurementType::is_valid_value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementType {
    /// Integer target/current (e.g. "ship 5 features").
    Count,
    /// Boolean target/current (e.g. "security review passed").
    Boolean,
    /// 0.0–1.0 target/current (e.g. "coverage ≥ 0.85").
    Percentage,
    /// Free-form `Value` for domain-specific metrics.
    Custom,
}

impl MeasurementType {
    pub const ALL: [MeasurementType; 4] = [
        MeasurementType::Count,
        MeasurementType::Boolean,
        MeasurementType::Percentage,
        MeasurementType::Custom,
    ];

    /// True if `v` is a valid representation for this measurement kind.
    ///
    /// Called by the M4/P6 project-creation orchestrator as part of OKR
    /// validation (server-side); rejection surfaces as
    /// `OKR_VALIDATION_FAILED` to the operator.
    pub fn is_valid_value(self, v: &OkrValue) -> bool {
        match (self, v) {
            (MeasurementType::Count, OkrValue::Integer(_)) => true,
            (MeasurementType::Boolean, OkrValue::Bool(_)) => true,
            (MeasurementType::Percentage, OkrValue::Percentage(p)) => (&0.0..=&1.0).contains(&p),
            (MeasurementType::Custom, _) => true,
            _ => false,
        }
    }
}

/// Typed `Value` carried by [`KeyResult.target_value`] /
/// [`KeyResult.current_value`]. Shape gated by
/// [`MeasurementType::is_valid_value`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum OkrValue {
    Integer(i64),
    Bool(bool),
    /// 0.0–1.0 inclusive. Enforced by [`MeasurementType::is_valid_value`].
    Percentage(f64),
    /// Free-form serde value for `MeasurementType::Custom`. Not
    /// `PartialEq`-compatible with itself structurally; custom KRs
    /// compare by `kr_id`.
    Custom(serde_json::Value),
}

impl Eq for OkrValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyResultStatus {
    NotStarted,
    InProgress,
    Achieved,
    Missed,
    Cancelled,
}

impl KeyResultStatus {
    pub const ALL: [KeyResultStatus; 5] = [
        KeyResultStatus::NotStarted,
        KeyResultStatus::InProgress,
        KeyResultStatus::Achieved,
        KeyResultStatus::Missed,
        KeyResultStatus::Cancelled,
    ];
}

// ============================================================================
// ResourceBoundaries — embedded governance-scope value object on Project.
// ============================================================================

/// A subset of the owning org's `resources_catalogue` that a project
/// operates within. Narrows the grantable resource set for
/// project-scoped grants.
///
/// **phi-core leverage**: none — phi governance catalogue primitive.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBoundaries {
    /// Model-provider ids from the org's catalogue. Empty = all providers
    /// allowed (the inherit default).
    #[serde(default)]
    pub allowed_model_providers: Vec<ModelProviderId>,
    /// MCP-server ids from the org's catalogue. Empty = all allowed.
    #[serde(default)]
    pub allowed_mcp_servers: Vec<McpServerId>,
    /// Named skills (string keys, not ids — skills are string-named in
    /// phi-core's skill registry). Empty = all allowed.
    #[serde(default)]
    pub allowed_skills: Vec<String>,
    /// Project-scoped token-budget ceiling (independent of the
    /// project-node-level `token_budget`). Empty = no project-scope cap.
    #[serde(default)]
    pub token_budget: Option<u64>,
}

// ============================================================================
// AgentExecutionLimitsOverride — the ONLY phi-core wrap added at M4/P1.
// ============================================================================

/// Per-agent `ExecutionLimits` override row. **Opt-in** extension on top
/// of the ADR-0023 inherit-from-snapshot default.
///
/// **phi-core leverage**: direct wrap of
/// [`phi_core::context::execution::ExecutionLimits`] — the single load-bearing
/// phi-core import introduced in M4/P1 (the compile-time coercion test in
/// `tests` pins the type identity at compile time).
///
/// **Invariant** (enforced at repo layer by `M4/P3`'s compound tx + pinned
/// by a 50-case proptest): every field on [`limits`] must be
/// `≤` the corresponding field on the owning org's
/// `Organization.defaults_snapshot.execution_limits`. Operators can only
/// *tighten* per-agent ceilings; raising them above the org ceiling is
/// forbidden.
///
/// **Resolution**: `resolve_effective_execution_limits(agent_id)` (M4/P2)
/// returns this override if a row exists, else falls back to the
/// org-snapshot value. Default path (no row) matches ADR-0023.
///
/// See [ADR-0027](../../../../../../docs/specs/v0/implementation/m4/decisions/0027-per-agent-execution-limits-override.md).
///
/// [`limits`]: AgentExecutionLimitsOverride::limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionLimitsOverride {
    /// Stable graph-node identity for the override row.
    pub id: NodeId,
    /// The agent this override applies to (1:1 — enforced by a UNIQUE
    /// index on the `agent_execution_limits` table per migration 0004).
    pub owning_agent: AgentId,
    /// The phi-core limits — source of truth for max_turns / tokens /
    /// duration / cost. Wrapped directly, not re-declared.
    pub limits: phi_core::context::execution::ExecutionLimits,
    pub created_at: DateTime<Utc>,
}

impl AgentExecutionLimitsOverride {
    /// Bounds-check: return true iff every field on `self.limits` is
    /// `≤` the corresponding field on the supplied org-snapshot value.
    ///
    /// Called by the M4/P5 edit handler before the override is persisted;
    /// violation surfaces as `EXECUTION_LIMITS_EXCEED_ORG_CEILING`.
    pub fn is_bounded_by(&self, ceiling: &phi_core::context::execution::ExecutionLimits) -> bool {
        self.limits.max_turns <= ceiling.max_turns
            && self.limits.max_total_tokens <= ceiling.max_total_tokens
            && self.limits.max_duration <= ceiling.max_duration
            && match (self.limits.max_cost, ceiling.max_cost) {
                // If the ceiling has no cost cap, any per-agent cost is OK.
                (_, None) => true,
                // If the ceiling has a cost cap and the override has none,
                // the override is *unbounded* — reject.
                (None, Some(_)) => false,
                (Some(agent), Some(org)) => agent <= org,
            }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_status_all_has_five_variants_and_roundtrips() {
        assert_eq!(ObjectiveStatus::ALL.len(), 5);
        for s in ObjectiveStatus::ALL {
            let j = serde_json::to_string(&s).expect("serialize");
            let back: ObjectiveStatus = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn key_result_status_all_has_five_variants_and_roundtrips() {
        assert_eq!(KeyResultStatus::ALL.len(), 5);
        for s in KeyResultStatus::ALL {
            let j = serde_json::to_string(&s).expect("serialize");
            let back: KeyResultStatus = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn measurement_type_validates_values() {
        assert!(MeasurementType::Count.is_valid_value(&OkrValue::Integer(5)));
        assert!(!MeasurementType::Count.is_valid_value(&OkrValue::Bool(true)));
        assert!(MeasurementType::Boolean.is_valid_value(&OkrValue::Bool(false)));
        assert!(MeasurementType::Percentage.is_valid_value(&OkrValue::Percentage(0.85)));
        assert!(!MeasurementType::Percentage.is_valid_value(&OkrValue::Percentage(1.5)));
        assert!(!MeasurementType::Percentage.is_valid_value(&OkrValue::Percentage(-0.1)));
        assert!(MeasurementType::Custom.is_valid_value(&OkrValue::Integer(42)));
    }

    #[test]
    fn okr_value_serde_roundtrip() {
        for v in [
            OkrValue::Integer(7),
            OkrValue::Bool(true),
            OkrValue::Percentage(0.42),
            OkrValue::Custom(serde_json::json!({"sla": 99.9})),
        ] {
            let j = serde_json::to_string(&v).expect("serialize");
            let back: OkrValue = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, v);
        }
    }

    #[test]
    fn resource_boundaries_default_is_empty() {
        let rb = ResourceBoundaries::default();
        assert!(rb.allowed_model_providers.is_empty());
        assert!(rb.allowed_mcp_servers.is_empty());
        assert!(rb.allowed_skills.is_empty());
        assert!(rb.token_budget.is_none());
    }

    #[test]
    fn agent_execution_limits_override_wraps_phi_core_type() {
        let ovr = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: AgentId::new(),
            limits: phi_core::context::execution::ExecutionLimits::default(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&ovr).expect("serialize");
        let back: AgentExecutionLimitsOverride = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.limits.max_turns, ovr.limits.max_turns);
    }

    /// Compile-time coercion test — proves the `limits` field *is* the
    /// phi-core type, not a phi redeclaration. Mirrors the discipline
    /// from M3's `OrganizationDefaultsSnapshot`.
    #[test]
    fn agent_execution_limits_override_limits_is_phi_core_type() {
        fn is_phi_core_execution_limits(_: &phi_core::context::execution::ExecutionLimits) {}
        let ovr = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: AgentId::new(),
            limits: phi_core::context::execution::ExecutionLimits::default(),
            created_at: Utc::now(),
        };
        is_phi_core_execution_limits(&ovr.limits);
    }

    #[test]
    fn is_bounded_by_rejects_exceeding_org_ceiling() {
        let ceiling = phi_core::context::execution::ExecutionLimits {
            max_turns: 50,
            max_total_tokens: 1_000_000,
            max_duration: std::time::Duration::from_secs(600),
            max_cost: Some(10.0),
        };
        let ok = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: AgentId::new(),
            limits: phi_core::context::execution::ExecutionLimits {
                max_turns: 25,
                max_total_tokens: 500_000,
                max_duration: std::time::Duration::from_secs(300),
                max_cost: Some(5.0),
            },
            created_at: Utc::now(),
        };
        assert!(ok.is_bounded_by(&ceiling));

        let over_turns = AgentExecutionLimitsOverride {
            limits: phi_core::context::execution::ExecutionLimits {
                max_turns: 100, // > 50
                max_total_tokens: 500_000,
                max_duration: std::time::Duration::from_secs(300),
                max_cost: Some(5.0),
            },
            ..ok.clone()
        };
        assert!(!over_turns.is_bounded_by(&ceiling));

        let unbounded_cost = AgentExecutionLimitsOverride {
            limits: phi_core::context::execution::ExecutionLimits {
                max_turns: 25,
                max_total_tokens: 500_000,
                max_duration: std::time::Duration::from_secs(300),
                max_cost: None, // agent-unbounded while org capped — reject
            },
            ..ok.clone()
        };
        assert!(!unbounded_cost.is_bounded_by(&ceiling));
    }

    #[test]
    fn is_bounded_by_allows_cost_when_org_is_uncapped() {
        let uncapped_ceiling = phi_core::context::execution::ExecutionLimits {
            max_turns: 50,
            max_total_tokens: 1_000_000,
            max_duration: std::time::Duration::from_secs(600),
            max_cost: None,
        };
        let agent = AgentExecutionLimitsOverride {
            id: NodeId::new(),
            owning_agent: AgentId::new(),
            limits: phi_core::context::execution::ExecutionLimits {
                max_turns: 25,
                max_total_tokens: 500_000,
                max_duration: std::time::Duration::from_secs(300),
                max_cost: Some(5.0),
            },
            created_at: Utc::now(),
        };
        assert!(agent.is_bounded_by(&uncapped_ceiling));
    }
}
