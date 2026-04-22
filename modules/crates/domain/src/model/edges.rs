//! The 69 edge types the v0 ontology defines (67 at M3 close; M4/P1 adds
//! `HAS_SUBPROJECT` + `HAS_CONFIG` per the project-node edge table in
//! `concepts/project.md §Project Edges`).
//!
//! Edges are modelled as a single tagged enum [`Edge`]. Each variant's payload
//! carries the edge's ID and the IDs of its `from` and `to` nodes. Where the
//! concept doc lists distinct source/target type pairs for the same edge
//! *name* (e.g. `CONNECTS_TO` with either `McpServer` or `OpenApiSpec` as
//! target; `HOLDS_GRANT` from Agent/Project/Org; `PROVIDES_TOOL` from
//! McpServer/OpenApiSpec; `OWNED_BY` both as Agent→User and generic
//! Resource→Principal), we model each source/target type pair as a distinct
//! variant — this is what gets the count to 69.
//!
//! Source of truth: `docs/specs/v0/concepts/ontology.md` §Edge Types.

use serde::{Deserialize, Serialize};

use super::ids::{
    AgentId, AuthRequestId, ConsentId, EdgeId, GrantId, MemoryId, NodeId, OrgId, ProjectId,
    SessionId, TemplateId, UserId,
};

/// Every edge type in the v0 ontology.
///
/// Count: **69** (invariant asserted in [`tests`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "edge")]
pub enum Edge {
    // --- Agent-Centric (22) ----------------------------------------------
    HasProfile {
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

    // --- Governance — Ownership (3) --------------------------------------
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
}

impl Edge {
    /// Stable, canonical name for this edge kind — matches the concept doc's
    /// UPPER_SNAKE_CASE form.
    pub fn name(&self) -> &'static str {
        match self {
            Edge::HasProfile { .. } => "HAS_PROFILE",
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
        }
    }
}

/// Every edge kind name, in the same order as the concept doc's tables.
///
/// Used by tests to assert the 69 count (67 at M3 close + 2 added at M4/P1).
/// Strings here mirror [`Edge::name`] outputs for the same variant order.
pub const EDGE_KIND_NAMES: [&str; 69] = [
    "HAS_PROFILE",
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, MemoryId, OrgId, UserId};
    use std::collections::HashSet;

    #[test]
    fn edge_kind_names_is_exactly_69() {
        assert_eq!(EDGE_KIND_NAMES.len(), 69);
    }

    #[test]
    fn edge_kind_names_are_distinct() {
        let set: HashSet<_> = EDGE_KIND_NAMES.iter().collect();
        assert_eq!(set.len(), 69);
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
}
