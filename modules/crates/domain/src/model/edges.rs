//! The 74 edge types the v0 ontology defines (67 at M3 close; M4/P1 adds
//! `HAS_SUBPROJECT` + `HAS_CONFIG` per the project-node edge table in
//! `concepts/project.md §Project Edges`; CH-23 adds `MANAGES` +
//! `HAS_AGENT_SUPERVISOR` per ADR-0046 Template C/D HTTP edges; CH-25
//! adds `OWNS` per ADR-0060 §D60.1 for Agent→Org/Project ownership with
//! typed [`crate::model::ids::OwnedResourceId`] payload, F1.b USER-LOCKED;
//! CH-28 adds `AGENT_PROFILE_USES_BLUEPRINT` + `AGENT_USES_BLUEPRINT_OVERRIDE`
//! per ADR-0063 §D63.1 + §D63.3 for the hybrid Blueprint table —
//! `AgentProfile→Blueprint(template)` + `Agent→Blueprint(override)` —
//! F1.c USER-LOCKED).
//!
//! Edges are modelled as a single tagged enum [`Edge`]. Each variant's payload
//! carries the edge's ID and the IDs of its `from` and `to` nodes. Where the
//! concept doc lists distinct source/target type pairs for the same edge
//! *name* (e.g. `CONNECTS_TO` with either `McpServer` or `OpenApiSpec` as
//! target; `HOLDS_GRANT` from Agent/Project/Org; `PROVIDES_TOOL` from
//! McpServer/OpenApiSpec; `OWNED_BY` both as Agent→User and generic
//! Resource→Principal), we model each source/target type pair as a distinct
//! variant — this is what gets the count to 72.
//!
//! Source of truth: `docs/specs/v0/concepts/ontology.md` §Edge Types.

use serde::{Deserialize, Serialize};

use super::ids::{
    AgentId, AuthRequestId, ConsentId, EdgeId, GrantId, MemoryId, NodeId, OrgId, OwnedResourceId,
    ProjectId, SessionId, TemplateId, UserId,
};
use super::nodes::BlueprintId;

/// Every edge type in the v0 ontology.
///
/// Count: **74** (invariant asserted in [`tests`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "edge")]
pub enum Edge {
    // --- Agent-Centric (22) ----------------------------------------------
    /// Renamed from `HasProfile` per CH-28 / ADR-0063 §D63.7 (F2.b USER-LOCKED
    /// DIVERGENT). The `serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")`
    /// attribute pins the wire-format edge tag to `USES_PROFILE` for new
    /// writes while accepting `HAS_PROFILE` for legacy reads (back-compat
    /// per ADR-0063 §D63.7).
    #[serde(rename = "USES_PROFILE", alias = "HAS_PROFILE")]
    UsesProfile {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    UsesModel {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    HasTool {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    HasSkill {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    AgentHoldsGrant {
        id: EdgeId,
        from: AgentId,
        to: GrantId,
    },
    GovernedBy {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    UsesCompaction {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    UsesRetry {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    UsesCache {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    UsesEvaluation {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    HasSystemPrompt {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    AgentConnectsToMcpServer {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    AgentConnectsToOpenApiSpec {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    RunsSession {
        id: EdgeId,
        from: AgentId,
        to: SessionId,
    },
    DelegatesTo {
        id: EdgeId,
        from: AgentId,
        to: AgentId,
    },
    /// Specific case of the generic `OWNED_BY`: an Agent is owned by a User.
    AgentOwnedByUser {
        id: EdgeId,
        from: AgentId,
        to: UserId,
    },
    HasMemory {
        id: EdgeId,
        from: AgentId,
        to: MemoryId,
    },
    HasInbox {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    HasOutbox {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    HasChannel {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    LoadedFrom {
        id: EdgeId,
        from: AgentId,
        to: NodeId,
    },
    MemberOf {
        id: EdgeId,
        from: AgentId,
        to: OrgId,
    },

    // --- Execution Chain (7) ---------------------------------------------
    ContainsLoop {
        id: EdgeId,
        from: SessionId,
        to: NodeId,
    },
    ContinuesFrom {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    ContainsTurn {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    ConfiguredWith {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    Emits {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    Produces {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    ExecutesTool {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },

    // --- Cross-Agent (3) -------------------------------------------------
    SpawnedFrom {
        id: EdgeId,
        from: SessionId,
        to: SessionId,
    },
    SpawnedChild {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    ParallelWith {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },

    // --- Capability Wiring (5) -------------------------------------------
    ImplementedBy {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    HasManifest {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    McpProvidesTool {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    OpenApiProvidesTool {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    ContainsBlock {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },

    // --- Social Structure (14) -------------------------------------------
    HasBoard {
        id: EdgeId,
        from: OrgId,
        to: AgentId,
    },
    HasCeo {
        id: EdgeId,
        from: OrgId,
        to: AgentId,
    },
    HasProject {
        id: EdgeId,
        from: OrgId,
        to: ProjectId,
    },
    HasMember {
        id: EdgeId,
        from: OrgId,
        to: AgentId,
    },
    HasSuborganization {
        id: EdgeId,
        from: OrgId,
        to: OrgId,
    },
    HasSponsor {
        id: EdgeId,
        from: ProjectId,
        to: AgentId,
    },
    HasAgent {
        id: EdgeId,
        from: ProjectId,
        to: AgentId,
    },
    /// Project P → Agent X: X is the designated lead of P. M3/P1 adds
    /// the variant so the Template A pure-fn constructor (M3/P2) can
    /// name it as its trigger condition; M5 wires the template firing.
    HasLead {
        id: EdgeId,
        from: ProjectId,
        to: AgentId,
    },
    /// Agent M → Agent S: M manages S within `org`. CH-23 adds this
    /// variant so the Template C listener has a real production
    /// trigger; the carrier `org` field lets the listener populate
    /// `DomainEvent::ManagesEdgeCreated.org_id` without a follow-up
    /// `member_of` lookup.
    Manages {
        id: EdgeId,
        from: AgentId,
        to: AgentId,
        org: OrgId,
    },
    /// Agent S → Agent T: S supervises T within `project`. CH-23 adds
    /// this variant so the Template D listener has a real production
    /// trigger; the carrier `project` field lets the listener populate
    /// `DomainEvent::HasAgentSupervisorEdgeCreated.project_id` without
    /// a follow-up project-membership lookup.
    HasAgentSupervisor {
        id: EdgeId,
        from: AgentId,
        to: AgentId,
        project: ProjectId,
    },
    HasTask {
        id: EdgeId,
        from: ProjectId,
        to: NodeId,
    },
    /// Project P → Project Q: Q is a sub-project of P. M4/P1 adds the
    /// variant so M4/P6's `apply_project_creation` can name it when an
    /// operator nests a child project under an existing parent.
    HasSubproject {
        id: EdgeId,
        from: ProjectId,
        to: ProjectId,
    },
    /// Project P → AgentConfig C: the project-level root configuration
    /// document governing agents that run within P. M4/P1 adds the
    /// variant; first production writes land at M5 (session-launch)
    /// when agent-config resolution extends to project scope.
    HasConfig {
        id: EdgeId,
        from: ProjectId,
        to: NodeId,
    },
    BelongsTo {
        id: EdgeId,
        from: ProjectId,
        to: OrgId,
    },
    AssignedTo {
        id: EdgeId,
        from: NodeId,
        to: AgentId,
    },
    HasBid {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    BidSubmittedBy {
        id: EdgeId,
        from: NodeId,
        to: AgentId,
    },
    Rates {
        id: EdgeId,
        from: NodeId,
        to: AgentId,
    },
    GivenBy {
        id: EdgeId,
        from: NodeId,
        to: AgentId,
    },

    // --- Governance — Ownership (4) --------------------------------------
    /// Generic form: any Resource is owned by any Principal. The `from` and
    /// `to` carry the respective node IDs via `NodeId` since Resource /
    /// Principal are type unions, not single types.
    OwnedBy {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    Created {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    AllocatedTo {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
    },
    /// CH-25 / ADR-0060 §D60.1 (F1.b USER-LOCKED, DIVERGENT) — Agent
    /// owns an Organization or a Project. The payload is typed at both
    /// ends (Agent → OwnedResourceId) so that callers cannot accidentally
    /// emit this edge with the wrong principal-end or resource-end kind.
    /// Distinct from the generic [`Edge::OwnedBy`] variant (which stays
    /// Memory-as-Resource focused per ADR-0015 / M1 close).
    ///
    /// Org/Project STAY Principal-only per the v0 ontology invariant at
    /// `principal_resource.rs:182-186`; the typed `OwnedResourceId`
    /// payload is the resource-end carrier, NOT a `Resource`-trait
    /// relaxation.
    Owns {
        id: EdgeId,
        from: AgentId,
        to: OwnedResourceId,
    },

    // --- Governance — Grant + Auth Request (10) --------------------------
    IssuedGrant {
        id: EdgeId,
        from: UserId,
        to: GrantId,
    },
    DescendsFrom {
        id: EdgeId,
        from: GrantId,
        to: AuthRequestId,
    },
    AppliesTo {
        id: EdgeId,
        from: GrantId,
        to: AgentId,
    },
    ProjectHoldsGrant {
        id: EdgeId,
        from: ProjectId,
        to: GrantId,
    },
    OrgHoldsGrant {
        id: EdgeId,
        from: OrgId,
        to: GrantId,
    },
    RequestsOn {
        id: EdgeId,
        from: AuthRequestId,
        to: NodeId,
    },
    ApprovedBy {
        id: EdgeId,
        from: AuthRequestId,
        to: NodeId,
    },
    AuthRequestSubmittedBy {
        id: EdgeId,
        from: AuthRequestId,
        to: NodeId,
    },
    EmittedBy {
        id: EdgeId,
        from: AuthRequestId,
        to: TemplateId,
    },
    /// Explicit listing for the Agent→Grant HOLDS_GRANT variant in the
    /// Governance section (the concept doc lists it in both Agent-Centric
    /// and Governance tables; we keep the Agent-Centric variant authoritative
    /// and flag this one as a dedupe-marker).
    AgentHoldsGrantDedupe {
        id: EdgeId,
        from: AgentId,
        to: GrantId,
    },

    // --- Consent (2) ------------------------------------------------------
    HasConsent {
        id: EdgeId,
        from: AgentId,
        to: ConsentId,
    },
    ScopedTo {
        id: EdgeId,
        from: ConsentId,
        to: OrgId,
    },

    // --- Hybrid Blueprint table (2 — CH-28 / ADR-0063 §D63.1 + §D63.3) ----
    /// CH-28 / ADR-0063 §D63.1 + §D63.3 (F1.c USER-LOCKED DIVERGENT) —
    /// AgentProfile → template-Blueprint relation. 1:1 from the
    /// AgentProfile side; N:1 from the Blueprint side (many AgentProfile
    /// rows MAY share one template Blueprint — the load-bearing sharing
    /// semantic). Wire-name: `AGENT_PROFILE_USES_BLUEPRINT`.
    AgentProfileUsesBlueprint {
        id: EdgeId,
        from: NodeId,
        to: BlueprintId,
    },
    /// CH-28 / ADR-0063 §D63.1 + §D63.3 (F1.c USER-LOCKED DIVERGENT) —
    /// Agent → per-agent override-Blueprint relation. 1:zero-or-one
    /// from the Agent side; 1:1 from the Blueprint side. The 1:1
    /// invariant on override rows is enforced at the repository tier
    /// per ADR-0063 §D63.5 + §D63.12 (SurrealDB 2.6.5 does NOT support
    /// filtered UNIQUE indexes). Wire-name: `AGENT_USES_BLUEPRINT_OVERRIDE`.
    AgentUsesBlueprintOverride {
        id: EdgeId,
        from: AgentId,
        to: BlueprintId,
    },
}

impl Edge {
    /// Stable, canonical name for this edge kind — matches the concept doc's
    /// UPPER_SNAKE_CASE form.
    pub fn name(&self) -> &'static str {
        match self {
            Edge::UsesProfile { .. } => "USES_PROFILE",
            Edge::UsesModel { .. } => "USES_MODEL",
            Edge::HasTool { .. } => "HAS_TOOL",
            Edge::HasSkill { .. } => "HAS_SKILL",
            Edge::AgentHoldsGrant { .. } => "HOLDS_GRANT(agent)",
            Edge::GovernedBy { .. } => "GOVERNED_BY",
            Edge::UsesCompaction { .. } => "USES_COMPACTION",
            Edge::UsesRetry { .. } => "USES_RETRY",
            Edge::UsesCache { .. } => "USES_CACHE",
            Edge::UsesEvaluation { .. } => "USES_EVALUATION",
            Edge::HasSystemPrompt { .. } => "HAS_SYSTEM_PROMPT",
            Edge::AgentConnectsToMcpServer { .. } => "CONNECTS_TO(mcp)",
            Edge::AgentConnectsToOpenApiSpec { .. } => "CONNECTS_TO(openapi)",
            Edge::RunsSession { .. } => "RUNS_SESSION",
            Edge::DelegatesTo { .. } => "DELEGATES_TO",
            Edge::AgentOwnedByUser { .. } => "OWNED_BY(agent->user)",
            Edge::HasMemory { .. } => "HAS_MEMORY",
            Edge::HasInbox { .. } => "HAS_INBOX",
            Edge::HasOutbox { .. } => "HAS_OUTBOX",
            Edge::HasChannel { .. } => "HAS_CHANNEL",
            Edge::LoadedFrom { .. } => "LOADED_FROM",
            Edge::MemberOf { .. } => "MEMBER_OF",

            Edge::ContainsLoop { .. } => "CONTAINS_LOOP",
            Edge::ContinuesFrom { .. } => "CONTINUES_FROM",
            Edge::ContainsTurn { .. } => "CONTAINS_TURN",
            Edge::ConfiguredWith { .. } => "CONFIGURED_WITH",
            Edge::Emits { .. } => "EMITS",
            Edge::Produces { .. } => "PRODUCES",
            Edge::ExecutesTool { .. } => "EXECUTES_TOOL",

            Edge::SpawnedFrom { .. } => "SPAWNED_FROM",
            Edge::SpawnedChild { .. } => "SPAWNED_CHILD",
            Edge::ParallelWith { .. } => "PARALLEL_WITH",

            Edge::ImplementedBy { .. } => "IMPLEMENTED_BY",
            Edge::HasManifest { .. } => "HAS_MANIFEST",
            Edge::McpProvidesTool { .. } => "PROVIDES_TOOL(mcp)",
            Edge::OpenApiProvidesTool { .. } => "PROVIDES_TOOL(openapi)",
            Edge::ContainsBlock { .. } => "CONTAINS_BLOCK",

            Edge::HasBoard { .. } => "HAS_BOARD",
            Edge::HasCeo { .. } => "HAS_CEO",
            Edge::HasProject { .. } => "HAS_PROJECT",
            Edge::HasMember { .. } => "HAS_MEMBER",
            Edge::HasSuborganization { .. } => "HAS_SUBORGANIZATION",
            Edge::HasSponsor { .. } => "HAS_SPONSOR",
            Edge::HasAgent { .. } => "HAS_AGENT",
            Edge::HasLead { .. } => "HAS_LEAD",
            Edge::Manages { .. } => "MANAGES",
            Edge::HasAgentSupervisor { .. } => "HAS_AGENT_SUPERVISOR",
            Edge::HasTask { .. } => "HAS_TASK",
            Edge::HasSubproject { .. } => "HAS_SUBPROJECT",
            Edge::HasConfig { .. } => "HAS_CONFIG",
            Edge::BelongsTo { .. } => "BELONGS_TO",
            Edge::AssignedTo { .. } => "ASSIGNED_TO",
            Edge::HasBid { .. } => "HAS_BID",
            Edge::BidSubmittedBy { .. } => "SUBMITTED_BY(bid->agent)",
            Edge::Rates { .. } => "RATES",
            Edge::GivenBy { .. } => "GIVEN_BY",

            Edge::OwnedBy { .. } => "OWNED_BY",
            Edge::Created { .. } => "CREATED",
            Edge::AllocatedTo { .. } => "ALLOCATED_TO",
            Edge::Owns { .. } => "OWNS",

            Edge::IssuedGrant { .. } => "ISSUED_GRANT",
            Edge::DescendsFrom { .. } => "DESCENDS_FROM",
            Edge::AppliesTo { .. } => "APPLIES_TO",
            Edge::ProjectHoldsGrant { .. } => "HOLDS_GRANT(project)",
            Edge::OrgHoldsGrant { .. } => "HOLDS_GRANT(org)",
            Edge::RequestsOn { .. } => "REQUESTS_ON",
            Edge::ApprovedBy { .. } => "APPROVED_BY",
            Edge::AuthRequestSubmittedBy { .. } => "SUBMITTED_BY(auth_request->principal)",
            Edge::EmittedBy { .. } => "EMITTED_BY",
            Edge::AgentHoldsGrantDedupe { .. } => "HOLDS_GRANT(agent-governance-listing)",

            Edge::HasConsent { .. } => "HAS_CONSENT",
            Edge::ScopedTo { .. } => "SCOPED_TO",

            Edge::AgentProfileUsesBlueprint { .. } => "AGENT_PROFILE_USES_BLUEPRINT",
            Edge::AgentUsesBlueprintOverride { .. } => "AGENT_USES_BLUEPRINT_OVERRIDE",
        }
    }
}

/// Every edge kind name, in the same order as the concept doc's tables.
///
/// Used by tests to assert the 74 count: 67 at M3 close, +2 at M4/P1
/// (`HasSubproject`, `HasConfig`), +2 at CH-23 for Template C/D
/// triggers (`Manages`, `HasAgentSupervisor`), +1 at CH-25 (`Owns`)
/// per ADR-0060 §D60.1 for Agent→Org/Project ownership (F1.b
/// USER-LOCKED DIVERGENT), +2 at CH-28
/// (`AgentProfileUsesBlueprint`, `AgentUsesBlueprintOverride`) per
/// ADR-0063 §D63.1 + §D63.3 for the hybrid Blueprint table edges
/// (F1.c USER-LOCKED DIVERGENT). Strings here mirror [`Edge::name`]
/// outputs for the same variant order.
pub const EDGE_KIND_NAMES: [&str; 74] = [
    "USES_PROFILE",
    "USES_MODEL",
    "HAS_TOOL",
    "HAS_SKILL",
    "HOLDS_GRANT(agent)",
    "GOVERNED_BY",
    "USES_COMPACTION",
    "USES_RETRY",
    "USES_CACHE",
    "USES_EVALUATION",
    "HAS_SYSTEM_PROMPT",
    "CONNECTS_TO(mcp)",
    "CONNECTS_TO(openapi)",
    "RUNS_SESSION",
    "DELEGATES_TO",
    "OWNED_BY(agent->user)",
    "HAS_MEMORY",
    "HAS_INBOX",
    "HAS_OUTBOX",
    "HAS_CHANNEL",
    "LOADED_FROM",
    "MEMBER_OF",
    "CONTAINS_LOOP",
    "CONTINUES_FROM",
    "CONTAINS_TURN",
    "CONFIGURED_WITH",
    "EMITS",
    "PRODUCES",
    "EXECUTES_TOOL",
    "SPAWNED_FROM",
    "SPAWNED_CHILD",
    "PARALLEL_WITH",
    "IMPLEMENTED_BY",
    "HAS_MANIFEST",
    "PROVIDES_TOOL(mcp)",
    "PROVIDES_TOOL(openapi)",
    "CONTAINS_BLOCK",
    "HAS_BOARD",
    "HAS_CEO",
    "HAS_PROJECT",
    "HAS_MEMBER",
    "HAS_SUBORGANIZATION",
    "HAS_SPONSOR",
    "HAS_AGENT",
    "HAS_LEAD",
    "MANAGES",
    "HAS_AGENT_SUPERVISOR",
    "HAS_TASK",
    "HAS_SUBPROJECT",
    "HAS_CONFIG",
    "BELONGS_TO",
    "ASSIGNED_TO",
    "HAS_BID",
    "SUBMITTED_BY(bid->agent)",
    "RATES",
    "GIVEN_BY",
    "OWNED_BY",
    "CREATED",
    "ALLOCATED_TO",
    "OWNS",
    "ISSUED_GRANT",
    "DESCENDS_FROM",
    "APPLIES_TO",
    "HOLDS_GRANT(project)",
    "HOLDS_GRANT(org)",
    "REQUESTS_ON",
    "APPROVED_BY",
    "SUBMITTED_BY(auth_request->principal)",
    "EMITTED_BY",
    "HOLDS_GRANT(agent-governance-listing)",
    "HAS_CONSENT",
    "SCOPED_TO",
    "AGENT_PROFILE_USES_BLUEPRINT",
    "AGENT_USES_BLUEPRINT_OVERRIDE",
];

// ============================================================================
// Typed constructors for the three untyped-RELATION edges.
// ============================================================================
//
// `owned_by`, `created`, `allocated_to` accept the `Resource`/`Principal`
// type unions. The `Edge` enum payload carries `NodeId` for both ends
// because the variants must be a single Rust type. These constructors add
// compile-time safety so callers can't cross-paste the wrong ID kind.
//
// See ADR-0015 for the full rationale.

use super::principal_resource::{Principal, Resource};

impl Edge {
    /// Typed constructor for `owned_by` — a Resource is owned by a
    /// Principal. Compile-time rejects wrong pairs (e.g. a `ConsentId`
    /// as the Principal).
    pub fn new_owned_by<R: Resource, P: Principal>(resource: &R, principal: &P) -> Edge {
        Edge::OwnedBy {
            id: EdgeId::new(),
            from: resource.node_id(),
            to: principal.node_id(),
        }
    }

    /// Typed constructor for `created` — a Principal created a Resource
    /// (creation provenance).
    pub fn new_created<P: Principal, R: Resource>(creator: &P, resource: &R) -> Edge {
        Edge::Created {
            id: EdgeId::new(),
            from: creator.node_id(),
            to: resource.node_id(),
        }
    }

    /// Typed constructor for `allocated_to` — Principal A allocates some
    /// scope of authority over a resource to Principal B.
    ///
    /// Note: the concept-doc edge properties (`resource_ref`, `scope`,
    /// `provenance_auth_request`) live on the edge row at persistence
    /// time. They are not part of the `Edge` enum payload (which holds
    /// only graph shape); the repository's `upsert_allocation` helper
    /// carries them alongside.
    pub fn new_allocated_to<P1: Principal, P2: Principal>(from: &P1, to: &P2) -> Edge {
        Edge::AllocatedTo {
            id: EdgeId::new(),
            from: from.node_id(),
            to: to.node_id(),
        }
    }

    /// Typed constructor for `owns` — an Agent owns an Organization or
    /// Project (CH-25 / ADR-0060 §D60.1, F1.b USER-LOCKED).
    ///
    /// The payload is typed at both ends: `from: AgentId` is concrete
    /// (NOT generic `Principal`) and `to: OwnedResourceId` is the
    /// closed-set enum carrying either `Org(OrgId)` or `Project(ProjectId)`.
    /// This is more restrictive than `new_owned_by` / `new_created` /
    /// `new_allocated_to` (which use the Principal/Resource trait
    /// dispatch) — the typed variant payload makes wrong cross-pastes
    /// (e.g., `OwnedResourceId::Org(uid)` where `uid` is `UserId`)
    /// impossible at compile time.
    pub fn new_owns(agent: &AgentId, owned: OwnedResourceId) -> Edge {
        Edge::Owns {
            id: EdgeId::new(),
            from: *agent,
            to: owned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, MemoryId, OrgId, UserId};
    use std::collections::HashSet;

    #[test]
    fn edge_kind_names_cardinality_is_74_pinned_at_compile_time() {
        // 67 at M3 close + 2 at M4/P1 (HasSubproject, HasConfig) + 2
        // at CH-23 (Manages, HasAgentSupervisor — Template C/D
        // production triggers, ADR-0046) + 1 at CH-25 (Owns — Agent
        // ownership of Org/Project per ADR-0060 §D60.1, F1.b USER-LOCKED) +
        // 2 at CH-28 (AgentProfileUsesBlueprint, AgentUsesBlueprintOverride
        // — hybrid Blueprint table edges per ADR-0063 §D63.1 + §D63.3,
        // F1.c USER-LOCKED).
        assert_eq!(EDGE_KIND_NAMES.len(), 74);
    }

    #[test]
    fn edge_kind_names_are_distinct() {
        let set: HashSet<_> = EDGE_KIND_NAMES.iter().collect();
        assert_eq!(set.len(), 74);
    }

    #[test]
    fn manages_and_has_agent_supervisor_variants_have_correct_names() {
        // CH-23 / ADR-0046 — sanity check that the new variants emit
        // the concept-doc-mandated names from `Edge::name()`.
        let manages = Edge::Manages {
            id: EdgeId::new(),
            from: AgentId::new(),
            to: AgentId::new(),
            org: OrgId::new(),
        };
        let supervisor = Edge::HasAgentSupervisor {
            id: EdgeId::new(),
            from: AgentId::new(),
            to: AgentId::new(),
            project: crate::model::ids::ProjectId::new(),
        };
        assert_eq!(manages.name(), "MANAGES");
        assert_eq!(supervisor.name(), "HAS_AGENT_SUPERVISOR");
        assert!(EDGE_KIND_NAMES.contains(&"MANAGES"));
        assert!(EDGE_KIND_NAMES.contains(&"HAS_AGENT_SUPERVISOR"));
    }

    #[test]
    fn typed_owned_by_constructs_valid_edge() {
        let mem = MemoryId::new();
        let user = UserId::new();
        let edge = Edge::new_owned_by(&mem, &user);
        match edge {
            Edge::OwnedBy { from, to, .. } => {
                assert_eq!(from.as_uuid(), mem.as_uuid());
                assert_eq!(to.as_uuid(), user.as_uuid());
            }
            other => panic!("expected OwnedBy variant, got {:?}", other),
        }
    }

    #[test]
    fn typed_created_carries_principal_then_resource() {
        let agent = AgentId::new();
        let mem = MemoryId::new();
        let edge = Edge::new_created(&agent, &mem);
        match edge {
            Edge::Created { from, to, .. } => {
                assert_eq!(from.as_uuid(), agent.as_uuid());
                assert_eq!(to.as_uuid(), mem.as_uuid());
            }
            other => panic!("expected Created variant, got {:?}", other),
        }
    }

    #[test]
    fn typed_allocated_to_accepts_two_principals() {
        let org = OrgId::new();
        let agent = AgentId::new();
        let edge = Edge::new_allocated_to(&org, &agent);
        match edge {
            Edge::AllocatedTo { from, to, .. } => {
                assert_eq!(from.as_uuid(), org.as_uuid());
                assert_eq!(to.as_uuid(), agent.as_uuid());
            }
            other => panic!("expected AllocatedTo variant, got {:?}", other),
        }
    }

    #[test]
    fn agent_works_as_both_principal_and_resource() {
        // Regression guard for the dual-role invariant (see
        // principal_resource.rs). If the Resource impl on AgentId were
        // ever removed, this test stops compiling.
        let owner = AgentId::new();
        let owned = AgentId::new();
        let edge = Edge::new_owned_by(&owned, &owner);
        assert!(matches!(edge, Edge::OwnedBy { .. }));
    }

    #[test]
    fn owns_variant_has_correct_name() {
        // CH-25 / ADR-0060 §D60.1 (F1.b USER-LOCKED) — sanity check that
        // the new Owns variant emits the concept-doc-mandated name "OWNS"
        // from `Edge::name()` and is present in `EDGE_KIND_NAMES`.
        use crate::model::ids::OwnedResourceId;
        let agent = AgentId::new();
        let org = OrgId::new();
        let edge = Edge::Owns {
            id: EdgeId::new(),
            from: agent,
            to: OwnedResourceId::Org(org),
        };
        assert_eq!(edge.name(), "OWNS");
        assert!(EDGE_KIND_NAMES.contains(&"OWNS"));
    }

    #[test]
    fn typed_new_owns_org_constructs_valid_edge() {
        // CH-25 / ADR-0060 §D60.1 — Edge::new_owns typed constructor.
        // F1.b user-lock: payload is typed at both ends; AgentId on the
        // principal-end, OwnedResourceId on the resource-end.
        use crate::model::ids::OwnedResourceId;
        let agent = AgentId::new();
        let org = OrgId::new();
        let edge = Edge::new_owns(&agent, OwnedResourceId::Org(org));
        match edge {
            Edge::Owns { from, to, .. } => {
                assert_eq!(from.as_uuid(), agent.as_uuid());
                match to {
                    OwnedResourceId::Org(o) => assert_eq!(o.as_uuid(), org.as_uuid()),
                    other => panic!("expected Org variant, got {:?}", other),
                }
            }
            other => panic!("expected Owns variant, got {:?}", other),
        }
    }

    #[test]
    fn typed_new_owns_project_constructs_valid_edge() {
        use crate::model::ids::{OwnedResourceId, ProjectId};
        let agent = AgentId::new();
        let project = ProjectId::new();
        let edge = Edge::new_owns(&agent, OwnedResourceId::Project(project));
        match edge {
            Edge::Owns { from, to, .. } => {
                assert_eq!(from.as_uuid(), agent.as_uuid());
                match to {
                    OwnedResourceId::Project(p) => assert_eq!(p.as_uuid(), project.as_uuid()),
                    other => panic!("expected Project variant, got {:?}", other),
                }
            }
            other => panic!("expected Owns variant, got {:?}", other),
        }
    }

    // ---- CH-28 / ADR-0063 §D63.1 + §D63.3 — hybrid Blueprint table edges --

    #[test]
    fn agent_profile_uses_blueprint_edge_name_is_canonical() {
        // F1.c USER-LOCKED — verify the AGENT_PROFILE_USES_BLUEPRINT
        // edge variant emits the concept-doc-mandated wire name from
        // `Edge::name()` and is present in `EDGE_KIND_NAMES`.
        let edge = Edge::AgentProfileUsesBlueprint {
            id: EdgeId::new(),
            from: crate::model::ids::NodeId::new(),
            to: crate::model::nodes::BlueprintId::new(),
        };
        assert_eq!(edge.name(), "AGENT_PROFILE_USES_BLUEPRINT");
        assert!(EDGE_KIND_NAMES.contains(&"AGENT_PROFILE_USES_BLUEPRINT"));
    }

    #[test]
    fn agent_uses_blueprint_override_edge_name_is_canonical() {
        // F1.c USER-LOCKED — verify the AGENT_USES_BLUEPRINT_OVERRIDE
        // edge variant emits the concept-doc-mandated wire name from
        // `Edge::name()` and is present in `EDGE_KIND_NAMES`.
        let edge = Edge::AgentUsesBlueprintOverride {
            id: EdgeId::new(),
            from: AgentId::new(),
            to: crate::model::nodes::BlueprintId::new(),
        };
        assert_eq!(edge.name(), "AGENT_USES_BLUEPRINT_OVERRIDE");
        assert!(EDGE_KIND_NAMES.contains(&"AGENT_USES_BLUEPRINT_OVERRIDE"));
    }
}
