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

// phi-core session-tier types wrapped at M5/P1 per ADR-0029. Aliased
// where they collide with baby-phi's governance wrap (the wrap
// struct's name IS `Session` — the alias keeps both reachable in this
// module). `LoopRecord` and `Turn` don't collide (baby-phi wraps are
// `LoopRecordNode` and `TurnNode`) but aliasing uniformly pins the
// phi-core origin on every reference. Split into three `use`
// statements so the positive close-audit grep
// `grep -En '^use phi_core::session::model' domain/src/model/nodes.rs`
// returns exactly **3** lines (matching the M5 plan §Part 1.5 P1
// prediction).
use phi_core::session::model::LoopRecord as PhiCoreLoopRecord;
use phi_core::session::model::Session as PhiCoreSession;
use phi_core::session::model::Turn as PhiCoreTurn;

use super::ids::{
    AgentId, AuthRequestId, ConsentId, GrantId, LoopId, MemoryId, NodeId, OrgId, ProjectId,
    SessionId, TemplateId, TurnNodeId, UserId,
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
    /// Governance role within the owning org. `None` for pre-M4 agents
    /// deserialised from storage (treated as "Unclassified" by the
    /// dashboard roll-up). Enforced by `AgentRole::is_valid_for(kind)` at
    /// agent-creation + edit time.
    #[serde(default)]
    pub role: Option<AgentRole>,
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

/// An Agent's governance role within an org. Applies to both Human and LLM
/// agents via the `is_valid_for(kind)` rule. Source of truth for the 6-variant
/// enumeration: `docs/specs/v0/concepts/agent.md §Agent Roles`.
///
/// **Human-only roles** (must satisfy `kind == AgentKind::Human`):
/// - [`AgentRole::Executive`] — top-tier governance authority (e.g. CEO).
/// - [`AgentRole::Admin`] — operational governance (create/edit agents,
///   manage projects, adopt templates).
/// - [`AgentRole::Member`] — ordinary human participant; project contributor.
///
/// **LLM-only roles** (must satisfy `kind == AgentKind::Llm`):
/// - [`AgentRole::Intern`] — new worker agent, pre-token-economy.
/// - [`AgentRole::Contract`] — promoted agent, full token-economy participant.
/// - [`AgentRole::System`] — platform-provisioned system agent (read-only on
///   page 09; lifecycle managed by platform).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Executive,
    Admin,
    Member,
    Intern,
    Contract,
    System,
}

impl AgentRole {
    /// Every variant in a stable order (Human-side first, LLM-side second).
    pub const ALL: [AgentRole; 6] = [
        AgentRole::Executive,
        AgentRole::Admin,
        AgentRole::Member,
        AgentRole::Intern,
        AgentRole::Contract,
        AgentRole::System,
    ];

    /// Canonical string form — matches `#[serde(rename_all = "snake_case")]`.
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentRole::Executive => "executive",
            AgentRole::Admin => "admin",
            AgentRole::Member => "member",
            AgentRole::Intern => "intern",
            AgentRole::Contract => "contract",
            AgentRole::System => "system",
        }
    }

    /// Validation rule enforced at agent-creation time + edit time.
    ///
    /// Rejects Human agents carrying LLM roles and vice versa. Enforced by
    /// the server handler (page 09 create/edit) with stable code
    /// `AGENT_ROLE_INVALID_FOR_KIND` on violation.
    pub fn is_valid_for(self, kind: AgentKind) -> bool {
        match self {
            AgentRole::Executive | AgentRole::Admin | AgentRole::Member => kind == AgentKind::Human,
            AgentRole::Intern | AgentRole::Contract | AgentRole::System => kind == AgentKind::Llm,
        }
    }
}

/// Blueprint — who an agent IS.
///
/// Per [`concepts/phi-core-mapping.md`](../../../../../docs/specs/v0/concepts/phi-core-mapping.md)
/// phi's `AgentProfile` node maps to phi-core's `AgentProfile` type.
/// Rather than duplicating phi-core's field set, this struct **wraps**
/// [`phi_core::agents::profile::AgentProfile`] as `blueprint` and adds
/// only the phi-specific governance fields on top.
///
/// Fields defined here (phi-only):
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
    /// phi-specific governance field — not on `phi_core::AgentProfile`.
    pub parallelize: u32,
    /// The phi-core execution blueprint. Source of truth for
    /// `system_prompt`, `thinking_level`, `skills`, etc.
    pub blueprint: phi_core::agents::profile::AgentProfile,
    /// Per-agent `ModelConfig` binding (M5 / C-M5-5). References a row in
    /// the owning org's `ModelRuntime` catalogue (M2/P6 surface).
    /// `None` = inherit from the org snapshot's default model
    /// (ADR-0023 default path). `#[serde(default)]` so pre-M5 stored
    /// rows round-trip cleanly.
    #[serde(default)]
    pub model_config_id: Option<String>,
    /// Optional dev/test override of the MockProvider response at M5
    /// (CH-02 / ADR-0032 D32.2). `None` → `provider_for` returns
    /// `MockProvider::text("Acknowledged.")`; `Some(s)` →
    /// `MockProvider::text(s)`. Governance field on the baby-phi
    /// wrapper; never placed on `blueprint` (phi-core inner). Bypassed
    /// at M7 when real providers dispatch via `ProviderRegistry`.
    /// `#[serde(default)]` so pre-CH-02 stored rows round-trip cleanly.
    #[serde(default)]
    pub mock_response: Option<String>,
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
    /// the M3 org-creation wizard, step 1). Purely phi —
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

/// Project node — M4's first-class container for work.
///
/// Source of truth: [`docs/specs/v0/concepts/project.md`] §Properties. M4
/// materialises the full struct (M1–M3 kept it scaffolded as id-only).
///
/// **phi-core leverage**: none — Project is a pure phi governance
/// composite. phi-core has no Project / OKR / planning concept (see
/// Part 1.5 of the M4 plan §Q3 rejections).
///
/// OKRs + resource boundaries are **embedded value objects** (not sibling
/// nodes) — see [`crate::model::composites_m4`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: String,
    /// Optional one-line goal. Complementary to `objectives` — a project
    /// may have a `goal` (framing) *and* OKRs (measurement).
    #[serde(default)]
    pub goal: Option<String>,
    pub status: ProjectStatus,
    pub shape: ProjectShape,
    /// Total tokens allocated for this project (complementary to the
    /// org-level [`crate::model::TokenBudgetPool`]).
    #[serde(default)]
    pub token_budget: Option<u64>,
    /// Running total of tokens consumed across all sessions attributed
    /// to this project. Invariant: `tokens_spent <= token_budget` when
    /// `token_budget.is_some()`. Enforced at session-end time (M5+).
    #[serde(default)]
    pub tokens_spent: u64,
    #[serde(default)]
    pub objectives: Vec<crate::model::composites_m4::Objective>,
    #[serde(default)]
    pub key_results: Vec<crate::model::composites_m4::KeyResult>,
    /// Subset of the owning org's `resources_catalogue` that this project
    /// operates within. Narrows the grantable resource set for
    /// project-scoped grants.
    #[serde(default)]
    pub resource_boundaries: Option<crate::model::composites_m4::ResourceBoundaries>,
    pub created_at: DateTime<Utc>,
}

/// The 4-state project lifecycle per `concepts/project.md §Project Status`.
///
/// Transitions:
/// ```text
/// Planned ──▶ InProgress ──▶ Finished
///                │   ▲
///                ▼   │
///              OnHold
/// ```
/// Every transition carries a reason (surfaced via an audit event; not
/// embedded on the variant to keep the enum serde-flat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Planned,
    InProgress,
    OnHold,
    Finished,
}

impl ProjectStatus {
    pub const ALL: [ProjectStatus; 4] = [
        ProjectStatus::Planned,
        ProjectStatus::InProgress,
        ProjectStatus::OnHold,
        ProjectStatus::Finished,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Planned => "planned",
            ProjectStatus::InProgress => "in_progress",
            ProjectStatus::OnHold => "on_hold",
            ProjectStatus::Finished => "finished",
        }
    }
}

/// Project governance shape — Shape A (single-org, immediate materialisation)
/// vs Shape B (co-owned by two orgs, two-approver Auth Request flow).
///
/// Wire-form: `"shape_a"` / `"shape_b"` — pins the dashboard's
/// `ProjectsSummary.shape_a` / `.shape_b` counter field names (M3 carryover
/// C-M4-3). Alternative names (`single_org` / `co_owned`) rejected at plan
/// close (D2) to avoid a downstream rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectShape {
    #[serde(rename = "shape_a")]
    A,
    #[serde(rename = "shape_b")]
    B,
}

impl ProjectShape {
    pub const ALL: [ProjectShape; 2] = [ProjectShape::A, ProjectShape::B];

    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectShape::A => "shape_a",
            ProjectShape::B => "shape_b",
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
// ============================================================================
// Session / LoopRecordNode / TurnNode — 3-way wrap of phi-core's session tier.
//
// Landed at M5/P1 per ADR-0029. Each wrap carries a nested `inner` field
// holding the corresponding `phi_core::session::model::*` type verbatim;
// governance extensions (owning_org, owning_project, started_by, etc.)
// sit alongside. Nested (not `#[serde(flatten)]`) to avoid field-name
// collisions between phi-core's `session_id: String` / `agent_id: String`
// and baby-phi's `id: SessionId` / `started_by: AgentId` newtype UUIDs —
// the identical discipline M3 used for `OrganizationDefaultsSnapshot`.
//
// Compile-time coercion witnesses in the tests module pin the invariant
// that `inner` holds phi-core's type (see
// `tests::wraps_carry_phi_core_types`).
// ============================================================================

/// One logical task/conversation. Wraps `phi_core::session::model::Session`.
///
/// `inner` carries phi-core's full Session tree verbatim via serde;
/// governance extensions (owning_org, started_by, governance_state,
/// tokens_spent) sit alongside. See [ADR-0029](../../../../../../docs/specs/v0/implementation/m5/decisions/0029-session-persistence-and-recorder-wrap.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    /// The phi-core session tree (session-id, agent-id, loops, formation).
    pub inner: PhiCoreSession,
    pub owning_org: OrgId,
    pub owning_project: ProjectId,
    pub started_by: AgentId,
    pub governance_state: SessionGovernanceState,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    /// Running total of tokens the session consumed across every
    /// turn. Maintained by `BabyPhiSessionRecorder` at M5/P3 (zero
    /// at M5/P1 when the wrap lands without the recorder).
    #[serde(default)]
    pub tokens_spent: u64,
}

/// Session lifecycle state. Explicit transitions only (no silent
/// expiry). `Running` / `Completed` / `Aborted` / `FailedLaunch`.
///
/// - `Running` — set at launch; flipped by the recorder on `AgentEnd`.
/// - `Completed` — natural termination (`AgentEnd.rejection = None`,
///   final turn hit `StopReason::Stop` with no follow-up).
/// - `Aborted` — operator-triggered terminate OR rejection from the
///   input filter OR cancellation-token fire.
/// - `FailedLaunch` — panic scopeguard fallback (task crashed before
///   producing a valid `AgentEnd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionGovernanceState {
    Running,
    Completed,
    Aborted,
    FailedLaunch,
}

impl SessionGovernanceState {
    pub const ALL: [SessionGovernanceState; 4] = [
        SessionGovernanceState::Running,
        SessionGovernanceState::Completed,
        SessionGovernanceState::Aborted,
        SessionGovernanceState::FailedLaunch,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SessionGovernanceState::Running => "running",
            SessionGovernanceState::Completed => "completed",
            SessionGovernanceState::Aborted => "aborted",
            SessionGovernanceState::FailedLaunch => "failed_launch",
        }
    }

    /// True if the session is no longer accepting turns (Completed /
    /// Aborted / FailedLaunch). `Running` alone counts against the
    /// per-agent parallelize cap + the platform-wide registry.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, SessionGovernanceState::Running)
    }
}

/// One `agent_loop()` call. Wraps `phi_core::session::model::LoopRecord`.
///
/// Named `LoopRecordNode` (not `LoopRecord`) so the wrap sits alongside
/// phi-core's `LoopRecord` without a word-boundary collision on the
/// `check-phi-core-reuse.sh` denylist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopRecordNode {
    pub id: LoopId,
    /// The phi-core loop-record (messages, usage, status, timing).
    pub inner: PhiCoreLoopRecord,
    pub session_id: SessionId,
    /// Zero-based index of this loop within its session.
    pub loop_index: u32,
}

/// One LLM round-trip. Wraps `phi_core::session::model::Turn`.
///
/// Named `TurnNode` (not `Turn`) for the same denylist-hygiene reason
/// as `LoopRecordNode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnNode {
    pub id: TurnNodeId,
    /// The phi-core turn (input_messages, output_message, tool_results, usage).
    pub inner: PhiCoreTurn,
    pub loop_id: LoopId,
    /// Zero-based index of this turn within its loop.
    pub turn_index: u32,
}
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
// Project is a full M4 node (not a scaffold) — defined below.

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

    // ---- M4: AgentRole ----------------------------------------------------

    #[test]
    fn agent_role_all_has_six_variants() {
        assert_eq!(AgentRole::ALL.len(), 6);
    }

    #[test]
    fn agent_role_serde_roundtrip() {
        for r in AgentRole::ALL {
            let j = serde_json::to_string(&r).expect("serialize");
            let back: AgentRole = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, r);
        }
    }

    #[test]
    fn agent_role_wire_names_match_as_str() {
        // `as_str()` must render the same wire-form as serde — the
        // migration 0004 ASSERT clause uses serde's snake_case form.
        for r in AgentRole::ALL {
            let serde_form = serde_json::to_value(r)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .expect("AgentRole serialises as JSON string");
            assert_eq!(r.as_str(), serde_form);
        }
    }

    #[test]
    fn agent_role_human_variants_reject_llm_kind() {
        for r in [AgentRole::Executive, AgentRole::Admin, AgentRole::Member] {
            assert!(r.is_valid_for(AgentKind::Human), "{:?} Human", r);
            assert!(!r.is_valid_for(AgentKind::Llm), "{:?} Llm", r);
        }
    }

    #[test]
    fn agent_role_llm_variants_reject_human_kind() {
        for r in [AgentRole::Intern, AgentRole::Contract, AgentRole::System] {
            assert!(r.is_valid_for(AgentKind::Llm), "{:?} Llm", r);
            assert!(!r.is_valid_for(AgentKind::Human), "{:?} Human", r);
        }
    }

    // ---- M4: ProjectStatus + ProjectShape ---------------------------------

    #[test]
    fn project_status_all_has_four_variants_and_roundtrips() {
        assert_eq!(ProjectStatus::ALL.len(), 4);
        for s in ProjectStatus::ALL {
            let j = serde_json::to_string(&s).expect("serialize");
            let back: ProjectStatus = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn project_shape_serialises_with_shape_prefix() {
        // Dashboard's `ProjectsSummary.shape_a` / `.shape_b` field names
        // depend on this wire form (M3 carryover C-M4-3).
        assert_eq!(
            serde_json::to_value(ProjectShape::A).unwrap(),
            serde_json::Value::String("shape_a".into())
        );
        assert_eq!(
            serde_json::to_value(ProjectShape::B).unwrap(),
            serde_json::Value::String("shape_b".into())
        );
    }

    #[test]
    fn project_shape_roundtrips() {
        for s in ProjectShape::ALL {
            let j = serde_json::to_string(&s).expect("serialize");
            let back: ProjectShape = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn agent_role_field_defaults_to_none_for_pre_m4_rows() {
        // Deserialising an Agent row written before M4 (no `role` column)
        // must round-trip with `role = None` via `#[serde(default)]`.
        let pre_m4 = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "kind": "human",
            "display_name": "legacy admin",
            "owning_org": null,
            "created_at": "2026-01-01T00:00:00Z"
        }"#;
        let agent: Agent = serde_json::from_str(pre_m4).expect("deserialize pre-M4 row");
        assert_eq!(agent.role, None);
    }

    // ---- M5: Session / LoopRecordNode / TurnNode wraps --------------------

    /// Compile-time coercion witnesses. A rename or shape change in
    /// phi-core's `Session` / `LoopRecord` / `Turn` breaks the baby-phi
    /// build immediately. Identical discipline to M3's
    /// `OrganizationDefaultsSnapshot` wrap + M4's `AgentProfile.blueprint`
    /// wrap. See ADR-0029.
    #[allow(dead_code)]
    fn _is_phi_core_session(_: &PhiCoreSession) {}
    #[allow(dead_code)]
    fn _is_phi_core_loop_record(_: &PhiCoreLoopRecord) {}
    #[allow(dead_code)]
    fn _is_phi_core_turn(_: &PhiCoreTurn) {}

    #[allow(dead_code)]
    fn _coerces_session_inner(s: &Session) {
        _is_phi_core_session(&s.inner);
    }
    #[allow(dead_code)]
    fn _coerces_loop_record_inner(r: &LoopRecordNode) {
        _is_phi_core_loop_record(&r.inner);
    }
    #[allow(dead_code)]
    fn _coerces_turn_inner(t: &TurnNode) {
        _is_phi_core_turn(&t.inner);
    }

    #[test]
    fn session_governance_state_has_four_variants_and_roundtrips() {
        assert_eq!(SessionGovernanceState::ALL.len(), 4);
        for s in SessionGovernanceState::ALL {
            let j = serde_json::to_string(&s).expect("serialize");
            let back: SessionGovernanceState = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn session_governance_state_is_terminal_matches_running_guard() {
        assert!(!SessionGovernanceState::Running.is_terminal());
        assert!(SessionGovernanceState::Completed.is_terminal());
        assert!(SessionGovernanceState::Aborted.is_terminal());
        assert!(SessionGovernanceState::FailedLaunch.is_terminal());
    }

    #[test]
    fn session_governance_state_wire_names_match_as_str() {
        // Migration 0005's `session.governance_state` ASSERT clause uses
        // serde's snake_case form. Pin parity with `as_str()`.
        for s in SessionGovernanceState::ALL {
            let serde_form = serde_json::to_value(s)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .expect("serialises as JSON string");
            assert_eq!(s.as_str(), serde_form);
        }
    }

    #[test]
    fn agent_profile_model_config_id_defaults_to_none_for_pre_m5_rows() {
        // Pre-M5 agent_profile rows had no `model_config_id` column.
        // `#[serde(default)]` must keep the round-trip clean.
        let pre_m5 = serde_json::json!({
            "id": "00000000-0000-0000-0000-00000000aaaa",
            "agent_id": "00000000-0000-0000-0000-00000000bbbb",
            "parallelize": 1,
            "blueprint": phi_core::agents::profile::AgentProfile::default(),
            "created_at": "2026-01-01T00:00:00Z"
        });
        let profile: AgentProfile = serde_json::from_value(pre_m5).expect("deserialize pre-M5 row");
        assert_eq!(profile.model_config_id, None);
    }

    #[test]
    fn agent_profile_mock_response_defaults_to_none_for_pre_ch02_rows() {
        // Pre-CH-02 agent_profile rows had no `mock_response` column
        // (ADR-0032 D32.2 added it at migration 0006). `#[serde(default)]`
        // must keep the round-trip clean.
        let pre_ch02 = serde_json::json!({
            "id": "00000000-0000-0000-0000-00000000cccc",
            "agent_id": "00000000-0000-0000-0000-00000000dddd",
            "parallelize": 1,
            "blueprint": phi_core::agents::profile::AgentProfile::default(),
            "model_config_id": null,
            "created_at": "2026-01-01T00:00:00Z"
        });
        let profile: AgentProfile =
            serde_json::from_value(pre_ch02).expect("deserialize pre-CH-02 row");
        assert_eq!(profile.mock_response, None);
    }

    #[test]
    fn agent_profile_mock_response_roundtrip_preserves_some_value() {
        // Post-CH-02 rows with an explicit mock_response must survive
        // a serialize → deserialize cycle.
        let original = AgentProfile {
            id: NodeId::new(),
            agent_id: AgentId::new(),
            parallelize: 1,
            blueprint: phi_core::agents::profile::AgentProfile::default(),
            model_config_id: None,
            mock_response: Some("Test fixture response".to_string()),
            created_at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&original).expect("serialize");
        let back: AgentProfile = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(
            back.mock_response,
            Some("Test fixture response".to_string())
        );
    }
}
