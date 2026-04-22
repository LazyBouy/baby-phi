//! All 37 node types the v0 ontology defines.
//!
//! Each node has an `id`-only struct for M1; nodes that are load-bearing for
//! the Permission Check spine (Agent, AgentProfile, Grant, AuthRequest,
//! Template, Organization, Consent, User, InboxObject, OutboxObject,
//! ToolAuthorityManifest, Channel, Memory) carry their full M1 field shape.
//! The remaining nodes are scaffolded with `id` only; later milestones flesh
//! out their field sets as they come into scope.
//!
//! Source of truth: `docs/specs/v0/concepts/ontology.md` §Node Types.
//!
//! The [`NodeKind`] enum mirrors this inventory; the unit tests below assert
//! the canonical count (37) and matching string forms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{
    AgentId, AuthRequestId, ConsentId, GrantId, MemoryId, NodeId, OrgId, ProjectId, SessionId,
    TemplateId, UserId,
};

// ============================================================================
// NodeKind — the authoritative inventory of all 37 node type names.
// ============================================================================

/// Every node type the v0 ontology defines, by name.
///
/// Count: **37** (invariant asserted in [`tests`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // Core Identity (4)
    Agent,
    AgentProfile,
    User,
    Identity,
    // Execution History (5)
    Session,
    Loop,
    Turn,
    Message,
    Event,
    // Capability (8)
    ModelConfig,
    ToolDefinition,
    ToolImplementation,
    Skill,
    McpServer,
    OpenApiSpec,
    SystemPrompt,
    EvaluationStrategy,
    // Governance (9)
    ExecutionLimits,
    Grant,
    AuthRequest,
    Template,
    ToolAuthorityManifest,
    Consent,
    CompactionPolicy,
    RetryPolicy,
    CachePolicy,
    // Social Structure (11)
    Project,
    Task,
    Bid,
    Rating,
    Organization,
    Channel,
    Memory,
    AgentConfig,
    PromptBlock,
    InboxObject,
    OutboxObject,
}

impl NodeKind {
    /// Every node kind in a stable order, grouped by the concept doc's sections.
    pub const ALL: [NodeKind; 37] = [
        // Core Identity (4)
        NodeKind::Agent,
        NodeKind::AgentProfile,
        NodeKind::User,
        NodeKind::Identity,
        // Execution History (5)
        NodeKind::Session,
        NodeKind::Loop,
        NodeKind::Turn,
        NodeKind::Message,
        NodeKind::Event,
        // Capability (8)
        NodeKind::ModelConfig,
        NodeKind::ToolDefinition,
        NodeKind::ToolImplementation,
        NodeKind::Skill,
        NodeKind::McpServer,
        NodeKind::OpenApiSpec,
        NodeKind::SystemPrompt,
        NodeKind::EvaluationStrategy,
        // Governance (9)
        NodeKind::ExecutionLimits,
        NodeKind::Grant,
        NodeKind::AuthRequest,
        NodeKind::Template,
        NodeKind::ToolAuthorityManifest,
        NodeKind::Consent,
        NodeKind::CompactionPolicy,
        NodeKind::RetryPolicy,
        NodeKind::CachePolicy,
        // Social Structure (11)
        NodeKind::Project,
        NodeKind::Task,
        NodeKind::Bid,
        NodeKind::Rating,
        NodeKind::Organization,
        NodeKind::Channel,
        NodeKind::Memory,
        NodeKind::AgentConfig,
        NodeKind::PromptBlock,
        NodeKind::InboxObject,
        NodeKind::OutboxObject,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Agent => "agent",
            NodeKind::AgentProfile => "agent_profile",
            NodeKind::User => "user",
            NodeKind::Identity => "identity",
            NodeKind::Session => "session",
            NodeKind::Loop => "loop",
            NodeKind::Turn => "turn",
            NodeKind::Message => "message",
            NodeKind::Event => "event",
            NodeKind::ModelConfig => "model_config",
            NodeKind::ToolDefinition => "tool_definition",
            NodeKind::ToolImplementation => "tool_implementation",
            NodeKind::Skill => "skill",
            NodeKind::McpServer => "mcp_server",
            NodeKind::OpenApiSpec => "open_api_spec",
            NodeKind::SystemPrompt => "system_prompt",
            NodeKind::EvaluationStrategy => "evaluation_strategy",
            NodeKind::ExecutionLimits => "execution_limits",
            NodeKind::Grant => "grant",
            NodeKind::AuthRequest => "auth_request",
            NodeKind::Template => "template",
            NodeKind::ToolAuthorityManifest => "tool_authority_manifest",
            NodeKind::Consent => "consent",
            NodeKind::CompactionPolicy => "compaction_policy",
            NodeKind::RetryPolicy => "retry_policy",
            NodeKind::CachePolicy => "cache_policy",
            NodeKind::Project => "project",
            NodeKind::Task => "task",
            NodeKind::Bid => "bid",
            NodeKind::Rating => "rating",
            NodeKind::Organization => "organization",
            NodeKind::Channel => "channel",
            NodeKind::Memory => "memory",
            NodeKind::AgentConfig => "agent_config",
            NodeKind::PromptBlock => "prompt_block",
            NodeKind::InboxObject => "inbox_object",
            NodeKind::OutboxObject => "outbox_object",
        }
    }
}

// ============================================================================
// Load-bearing (full-field) node structs — used by the M1 spine.
// ============================================================================

/// A principal: human or LLM. Agents are both principals and resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub kind: AgentKind,
    pub display_name: String,
    /// Optional owning org once membership is established (None for fresh
    /// platform-admin at bootstrap time).
    pub owning_org: Option<OrgId>,
    pub created_at: DateTime<Utc>,
}

/// Human vs LLM agent. Human agents have no `ModelConfig` / `ExecutionLimits`
/// attached per `concepts/agent.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Human,
    Llm,
}

/// Blueprint — who an agent IS.
///
/// Per [`concepts/phi-core-mapping.md`](../../../../../docs/specs/v0/concepts/phi-core-mapping.md)
/// baby-phi's `AgentProfile` node maps to phi-core's `AgentProfile` type.
/// Rather than duplicating phi-core's field set, this struct **wraps**
/// [`phi_core::agents::profile::AgentProfile`] as `blueprint` and adds
/// only the baby-phi-specific governance fields on top.
///
/// Fields defined here (baby-phi-only):
/// - `id` — graph-node identity.
/// - `agent_id` — owning Agent node.
/// - `parallelize` — concurrent session cap (enforced at session-start);
///   this is a platform governance concern that phi-core doesn't model.
/// - `created_at` — persistence timestamp.
///
/// Everything else (`system_prompt`, `thinking_level`, `temperature`,
/// `max_tokens`, `config_id`, `skills`, `workspace`, human-readable
/// `name`, `description`) lives on `blueprint` — the single source of
/// truth, imported directly from phi-core. This keeps the two
/// representations in lock-step and matches the M2 reuse mandate
/// (ADR-0015 / §1.6 of the M2 plan).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: NodeId,
    pub agent_id: AgentId,
    /// Concurrent session cap (default 1); enforced at session-start time.
    /// baby-phi-specific governance field — not on `phi_core::AgentProfile`.
    pub parallelize: u32,
    /// The phi-core execution blueprint. Source of truth for
    /// `system_prompt`, `thinking_level`, `skills`, etc.
    pub blueprint: phi_core::agents::profile::AgentProfile,
    pub created_at: DateTime<Utc>,
}

/// A human owner of agents. Distinct from an Agent(kind=Human): a User is a
/// platform-level identity, an Agent(kind=Human) is the workspace-level
/// principal that represents them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

/// Organization node — the top of the social-structure hierarchy.
///
/// M3 extends the M1/M2 baseline with governance fields (consent
/// policy, default audit class, adopted authority templates) and a
/// frozen snapshot of platform defaults (per ADR-0019's
/// non-retroactive invariant). Fields added in M3 carry
/// `#[serde(default)]` so pre-M3 orgs deserialised from storage keep
/// round-tripping without a manual backfill — new fields resolve to
/// safe defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrgId,
    pub display_name: String,
    /// Operator-facing org vision statement (optional; supplied by
    /// the M3 org-creation wizard, step 1). Purely baby-phi —
    /// phi-core has no org-governance concept.
    #[serde(default)]
    pub vision: Option<String>,
    /// Operator-facing org mission statement (same provenance as
    /// `vision`).
    #[serde(default)]
    pub mission: Option<String>,
    /// Consent policy governing cross-session data access. Captured
    /// at creation time and frozen (the non-retroactive invariant).
    /// Defaults to `Implicit` for pre-M3 orgs deserialised from
    /// storage.
    #[serde(default = "Organization::default_consent_policy")]
    pub consent_policy: crate::model::ConsentPolicy,
    /// Default audit tier applied when no event-specific class is
    /// specified. Defaults to `Logged` for pre-M3 deserialisation
    /// compatibility.
    #[serde(default = "Organization::default_audit_class")]
    pub audit_class_default: crate::audit::AuditClass,
    /// Authority templates enabled for this org (subset of
    /// `TemplateKind::{A, B, C, D}`; E is always-available, F is
    /// reserved for M6). Applied at creation time with the org's CEO
    /// as the approver for each template-adoption Auth Request.
    #[serde(default)]
    pub authority_templates_enabled: Vec<TemplateKind>,
    /// Snapshot of platform defaults at org-creation time. Frozen —
    /// later `PlatformDefaults` PUTs do not mutate this field. `None`
    /// for pre-M3 orgs deserialised from storage (those orgs fall
    /// back to reading the current platform defaults at invocation
    /// time; this path is M3-backward-compat only).
    #[serde(default)]
    pub defaults_snapshot: Option<crate::model::OrganizationDefaultsSnapshot>,
    /// Preferred model provider for this org (M3 wizard step 5). All
    /// agents in the org default to this provider unless overridden
    /// at agent-profile time. `None` means "use the platform default
    /// at invocation time."
    #[serde(default)]
    pub default_model_provider: Option<crate::model::ids::ModelProviderId>,
    /// System agents provisioned at org-creation time (M3 spawns two:
    /// memory-extraction-agent + agent-catalog-agent). Used by the
    /// dashboard's AgentsSummary panel + M5's event-subscription
    /// wiring.
    #[serde(default)]
    pub system_agents: Vec<AgentId>,
    pub created_at: DateTime<Utc>,
}

impl Organization {
    fn default_consent_policy() -> crate::model::ConsentPolicy {
        crate::model::ConsentPolicy::Implicit
    }
    fn default_audit_class() -> crate::audit::AuditClass {
        crate::audit::AuditClass::Logged
    }
}

/// A reusable permission pattern whose adoption emits an Auth Request that
/// serves as the provenance for subsequent grants the template fires.
///
/// The [`kind`](Template::kind) discriminator identifies which lifecycle
/// pattern the template implements. v0 currently defines:
/// `SystemBootstrap`, `A`, `B`, `C`, `D`, `E`, `F`. M2 exercises
/// `SystemBootstrap` (already landed in M1) and `E` (self-interested
/// auto-approve — the platform admin both submits and approves their own
/// write). The other variants land as their owning milestones need them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: TemplateId,
    /// Stable name — e.g. `template:system_bootstrap`.
    pub name: String,
    /// Lifecycle family this template implements. Pre-M2 rows default to
    /// [`TemplateKind::SystemBootstrap`] via serde's `default`.
    #[serde(default)]
    pub kind: TemplateKind,
    pub created_at: DateTime<Utc>,
}

/// The lifecycle pattern family a [`Template`] implements.
///
/// Source of truth: `docs/specs/v0/concepts/permissions/02-templates.md`.
/// The variants match the concept doc's A–F table plus the pre-M2
/// `SystemBootstrap` shipping template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    /// The platform-install bootstrap template seeded by `--bootstrap-init`.
    /// Pre-dates the A–F taxonomy; shipped with M1's claim flow.
    SystemBootstrap,
    /// Explicit human-to-human request (human agent asks human approver).
    A,
    /// Delegated approval (agent asks on behalf of another principal).
    B,
    /// Pre-authorized template (grants auto-issue, no approval step).
    C,
    /// Consent-gated template (consent record is the approval).
    D,
    /// Self-interested auto-approve — the requestor fills their own
    /// approver slot. Used by all M2 admin writes (pages 02–05) because
    /// the platform admin owns both the write and the approval.
    E,
    /// Break-glass (elevated audit-class, mandatory post-incident review).
    F,
}

impl TemplateKind {
    /// Every variant in a stable order (matches the concept doc's
    /// §Templates table A–F, preceded by `SystemBootstrap`).
    pub const ALL: [TemplateKind; 7] = [
        TemplateKind::SystemBootstrap,
        TemplateKind::A,
        TemplateKind::B,
        TemplateKind::C,
        TemplateKind::D,
        TemplateKind::E,
        TemplateKind::F,
    ];

    /// Canonical string form, matches the `#[serde(rename_all)]` output.
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateKind::SystemBootstrap => "system_bootstrap",
            TemplateKind::A => "a",
            TemplateKind::B => "b",
            TemplateKind::C => "c",
            TemplateKind::D => "d",
            TemplateKind::E => "e",
            TemplateKind::F => "f",
        }
    }
}

impl Default for TemplateKind {
    /// Pre-M2 templates are all the bootstrap template; serde falls back
    /// here when deserializing rows written before the `kind` field
    /// existed on disk.
    fn default() -> Self {
        TemplateKind::SystemBootstrap
    }
}

/// Capability-based access control record (the 5-tuple from `permissions/03`).
///
/// `fundamentals` (added in M2/P4.5, G19 / D17) lets callers bind a grant
/// to explicit fundamental classes when the `resource.uri` is an **instance
/// URI** (e.g. `secret:anthropic-api-key`, `provider:42`) — the engine's
/// `resolve_grant` function has no other way to derive the fundamental
/// from such a URI. Leaving the field empty (the serde default) preserves
/// pre-M2/P4.5 semantics: the engine falls back to URI-derived expansion
/// (class name → fundamental; composite name → constituents; `system:root`
/// → every fundamental; opaque URI → empty set, i.e. never matches).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub id: GrantId,
    pub holder: PrincipalRef,
    pub action: Vec<String>,
    pub resource: ResourceRef,
    /// Explicit fundamental classes this grant covers — authoritative
    /// when non-empty. Empty means "use legacy URI-derivation" (M1
    /// semantics). See `domain::permissions::expansion::resolve_grant`.
    #[serde(default)]
    pub fundamentals: Vec<crate::model::Fundamental>,
    /// Auth Request that produced this grant (structural provenance).
    pub descends_from: Option<AuthRequestId>,
    pub delegable: bool,
    pub issued_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Who holds a grant / submits a request / is an approver.
///
/// Serde uses the externally-tagged default format (rather than internal
/// tagging) because several variants wrap primitives — `Agent(AgentId)` and
/// `System(String)` are newtype tuple variants, which serde rejects under
/// `#[serde(tag = ...)]`. The wire format is `{"agent": "<uuid>"}` /
/// `{"system": "system:genesis"}`, which round-trips cleanly through JSON
/// and through SurrealDB's `object` storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalRef {
    Agent(AgentId),
    User(UserId),
    Organization(OrgId),
    Project(ProjectId),
    /// System axioms — `system:genesis`, `system:root`, etc.
    System(String),
}

/// What a grant / request targets. For v0.1 we identify resources by a
/// namespaced string; richer forms land as composites grow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    /// e.g. `system:root`, `filesystem:/workspace/**`, `auth_request:req-7102`.
    pub uri: String,
}

/// First-class workflow composite mediating grant creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub id: AuthRequestId,
    pub requestor: PrincipalRef,
    pub kinds: Vec<String>,
    pub scope: Vec<String>,
    pub state: AuthRequestState,
    pub valid_until: Option<DateTime<Utc>>,
    pub submitted_at: DateTime<Utc>,
    pub resource_slots: Vec<ResourceSlot>,
    pub justification: Option<String>,
    pub audit_class: crate::audit::AuditClass,
    pub terminal_state_entered_at: Option<DateTime<Utc>>,
    pub archived: bool,
    pub active_window_days: u32,
    pub provenance_template: Option<TemplateId>,
}

/// One resource within an Auth Request, with its per-approver slots. P4 fills
/// in transition logic over these; for P1 we only need the shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSlot {
    pub resource: ResourceRef,
    pub approvers: Vec<ApproverSlot>,
    pub state: ResourceSlotState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverSlot {
    pub approver: PrincipalRef,
    pub state: ApproverSlotState,
    pub responded_at: Option<DateTime<Utc>>,
    pub reconsidered_at: Option<DateTime<Utc>>,
}

/// The 9-state Auth Request lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthRequestState {
    Draft,
    Pending,
    InProgress,
    Approved,
    Denied,
    Partial,
    Expired,
    Revoked,
    Cancelled,
}

impl AuthRequestState {
    pub const ALL: [AuthRequestState; 9] = [
        AuthRequestState::Draft,
        AuthRequestState::Pending,
        AuthRequestState::InProgress,
        AuthRequestState::Approved,
        AuthRequestState::Denied,
        AuthRequestState::Partial,
        AuthRequestState::Expired,
        AuthRequestState::Revoked,
        AuthRequestState::Cancelled,
    ];
}

/// Per-resource aggregation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSlotState {
    InProgress,
    Approved,
    Denied,
    Partial,
    Expired,
}

/// Per-approver slot state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApproverSlotState {
    Unfilled,
    Approved,
    Denied,
}

/// Subordinate consent record gating Authority Template grants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub id: ConsentId,
    pub subordinate: AgentId,
    pub scoped_to: OrgId,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Publish-time authority declaration for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAuthorityManifest {
    pub id: NodeId,
    /// Name of the `ToolDefinition` this manifest is attached to.
    pub tool_name: String,
    pub resource: Vec<String>,
    pub transitive: Vec<String>,
    pub actions: Vec<String>,
    pub constraints: Vec<String>,
    pub kinds: Vec<String>,
    pub target_kinds: Vec<String>,
    pub delegable: bool,
    pub approval: String,
}

/// Channel describing how to reach a Human Agent (Slack/email/web).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: NodeId,
    pub agent_id: AgentId,
    pub kind: ChannelKind,
    pub handle: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKind {
    Slack,
    Email,
    Web,
}

/// An Agent's inbox (received `AgentMessage`s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxObject {
    pub id: NodeId,
    pub agent_id: AgentId,
    pub created_at: DateTime<Utc>,
}

/// An Agent's outbox (sent `AgentMessage`s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxObject {
    pub id: NodeId,
    pub agent_id: AgentId,
    pub created_at: DateTime<Utc>,
}

/// Persistent knowledge across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    pub owning_agent: AgentId,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Scaffolded (id-only) node structs — to be fleshed out in later milestones.
// ============================================================================
//
// Each of these exists as a named type so that the 37-node inventory is real
// at the type level. The struct shape is intentionally minimal (`id` only);
// later milestones will add fields when the node comes into scope.

macro_rules! scaffold_node {
    ($(#[$meta:meta])* $name:ident, $id_ty:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            pub id: $id_ty,
        }
    };
}

scaffold_node!(
    /// Emergent self of an LLM Agent. [PLANNED M5] — full field set (self_description,
    /// lived, witnessed, embedding) lands when memory-extraction wires in.
    Identity,
    NodeId
);
scaffold_node!(
    /// One logical task/conversation. [PLANNED M5] — phi-core `Session` wraps here.
    Session,
    SessionId
);
scaffold_node!(
    /// One `agent_loop()` call. [PLANNED M5].
    Loop,
    NodeId
);
scaffold_node!(
    /// One LLM round-trip. [PLANNED M5].
    Turn,
    NodeId
);
scaffold_node!(
    /// Atomic unit of conversation. [PLANNED M5].
    MessageNode,
    NodeId
);
scaffold_node!(
    /// Granular execution-trace event. [PLANNED M5].
    EventNode,
    NodeId
);
scaffold_node!(
    /// Which LLM backs the agent. [PLANNED M2] — model-provider page.
    ModelConfig,
    NodeId
);
scaffold_node!(
    /// Tool schema sent to the LLM. [PLANNED M2].
    ToolDefinition,
    NodeId
);
scaffold_node!(
    /// Concrete tool execution logic. [PLANNED M2].
    ToolImplementation,
    NodeId
);
scaffold_node!(
    /// Loaded skill with metadata. [PLANNED M4].
    Skill,
    NodeId
);
scaffold_node!(
    /// External tool server via MCP. [PLANNED M2].
    McpServer,
    NodeId
);
scaffold_node!(
    /// External OpenAPI spec. [PLANNED M2].
    OpenApiSpec,
    NodeId
);
scaffold_node!(
    /// Assembled system prompt. [PLANNED M4].
    SystemPrompt,
    NodeId
);
scaffold_node!(
    /// How parallel branches are evaluated. [PLANNED M4].
    EvaluationStrategy,
    NodeId
);
scaffold_node!(
    /// Constrains agent resources. [PLANNED M4].
    ExecutionLimits,
    NodeId
);
scaffold_node!(
    /// Context-management strategy. [PLANNED M4].
    CompactionPolicy,
    NodeId
);
scaffold_node!(
    /// Error-retry behavior. [PLANNED M4].
    RetryPolicy,
    NodeId
);
scaffold_node!(
    /// Prompt-caching behavior. [PLANNED M4].
    CachePolicy,
    NodeId
);
scaffold_node!(
    /// Container for work with goal, agents, governance. [PLANNED M4].
    Project,
    ProjectId
);
scaffold_node!(
    /// Biddable unit of work. [PLANNED M4].
    Task,
    NodeId
);
scaffold_node!(
    /// Agent proposal for a Task. [PLANNED M4].
    Bid,
    NodeId
);
scaffold_node!(
    /// Quality assessment of agent work. [PLANNED M5].
    Rating,
    NodeId
);
scaffold_node!(
    /// Root configuration document. [PLANNED M2].
    AgentConfig,
    NodeId
);
scaffold_node!(
    /// One block within a system-prompt strategy. [PLANNED M4].
    PromptBlock,
    NodeId
);
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn node_kind_all_is_exactly_37() {
        assert_eq!(NodeKind::ALL.len(), 37);
    }

    #[test]
    fn node_kind_variants_are_distinct() {
        let set: HashSet<_> = NodeKind::ALL.iter().collect();
        assert_eq!(set.len(), 37);
    }

    #[test]
    fn node_kind_as_str_is_distinct_per_variant() {
        let strs: HashSet<_> = NodeKind::ALL.iter().map(NodeKind::as_str).collect();
        assert_eq!(strs.len(), 37);
    }

    #[test]
    fn auth_request_state_all_is_exactly_nine() {
        assert_eq!(AuthRequestState::ALL.len(), 9);
    }

    #[test]
    fn node_kind_serde_roundtrip() {
        for k in NodeKind::ALL {
            let j = serde_json::to_string(&k).expect("serialize");
            let back: NodeKind = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, k);
        }
    }
}
