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
    /// CH-01 / ADR-0034 D34.1 — durable disable flag. `false` = the agent
    /// is disabled (system-agent disable handler flipped it). Pre-CH-01
    /// rows deserialise as `true` via `default_agent_active`; new agents
    /// land as `true` since `apply_agent_creation` constructs the struct
    /// directly. AgentCatalogListener (CH-22 body) reads this to compute
    /// `SystemAgentRuntimeStatus.is_paused`.
    #[serde(default = "default_agent_active")]
    pub active: bool,
    /// CH-01 / ADR-0034 D34.2 — optional archive timestamp. `None` = not
    /// archived. Set by the system-agent archive handler. Pre-CH-01 rows
    /// deserialise as `None`. Stored as RFC3339 string in SurrealDB
    /// (matching `grant.revoked_at` / `consent.revoked_at` convention).
    #[serde(default)]
    pub archived_at: Option<DateTime<Utc>>,
}

fn default_agent_active() -> bool {
    true
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
    /// CH-11 / ADR-0048 §D48.4 — how long a `Requested` consent waits
    /// before the sweeper auto-flips it to `TimedOut`. Default is
    /// `ProjectDuration` per concept doc 06 line 322. The launch
    /// handler reads this at consent-mint time to compute the row's
    /// `deadline_at`.
    ///
    /// `#[serde(default)]` shields pre-CH-11 wire payloads — they
    /// decode as `ApprovalTimeout::ProjectDuration`.
    #[serde(default)]
    pub approval_timeout: crate::model::ApprovalTimeout,
    /// CH-11 / ADR-0048 §D48.4 — what the engine returns when a
    /// consent crosses into `TimedOut`. Default is `Deny` per concept
    /// doc 06 line 349 ("absence of consent is not consent").
    ///
    /// `#[serde(default = "Organization::default_timeout_response")]`
    /// shields pre-CH-11 wire payloads — they decode as
    /// `TimeoutResponse::Deny`.
    #[serde(default = "Organization::default_timeout_response")]
    pub approval_timeout_default_response: TimeoutResponse,
    /// CH-26 / ADR-0061 §D61.2 — instance-identity + governance tags
    /// carried at the row level. At-creation-time the
    /// `apply_org_creation` compound-tx populates this with
    /// `[format!("organization:{}", id)]` (the self-identity tag per
    /// `concepts/permissions/01-resource-ontology.md` §"Instance Identity
    /// Tags"). Backfill migration `0018_org_project_tags.surql`
    /// populates extant pre-CH-26 rows.
    ///
    /// `#[serde(default)]` shields pre-CH-26 wire payloads — they decode
    /// as `Vec::new()`. Precedent: `Session.tags`, `Memory.tags`,
    /// `Channel.tags`, `AgentCredential.tags` (4 existing tag-bearing
    /// node-types).
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl Organization {
    fn default_consent_policy() -> crate::model::ConsentPolicy {
        crate::model::ConsentPolicy::Implicit
    }
    fn default_audit_class() -> crate::audit::AuditClass {
        crate::audit::AuditClass::Logged
    }
    /// Default response for a `TimedOut` consent on this org. Used by
    /// `#[serde(default = "...")]` on
    /// [`Organization::approval_timeout_default_response`] so pre-CH-11
    /// wire payloads decode cleanly.
    fn default_timeout_response() -> TimeoutResponse {
        TimeoutResponse::Deny
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
    /// CH-26 / ADR-0061 §D61.2 — instance-identity + governance tags
    /// carried at the row level. At-creation-time the
    /// `apply_project_creation` compound-tx populates this with
    /// `[format!("project:{}", id)]` (the self-identity tag per
    /// `concepts/permissions/01-resource-ontology.md` §"Instance Identity
    /// Tags"). Backfill migration `0018_org_project_tags.surql`
    /// populates extant pre-CH-26 rows.
    ///
    /// `#[serde(default)]` shields pre-CH-26 wire payloads — they decode
    /// as `Vec::new()`. Precedent: `Session.tags`, `Memory.tags`,
    /// `Channel.tags`, `AgentCredential.tags`.
    #[serde(default)]
    pub tags: Vec<String>,
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
    pub action: Vec<crate::permissions::action::Action>,
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
    /// CH-11 / ADR-0048 §D48.1 — approval-mode at the time the grant
    /// was issued. Templates issuing per-session-policy grants set this
    /// to `SubordinateRequired { policy: PerSession }` so the engine's
    /// Step 6 doesn't have to re-look-up the org node at check time.
    /// Pre-CH-11 grants decode as `ApprovalMode::Implicit` via serde's
    /// default — Step 6 short-circuits to Allow for those.
    ///
    /// Concept doc anchor:
    /// `permissions/06-multi-scope-consent.md` line 244.
    #[serde(default)]
    pub approval_mode: ApprovalMode,
    /// CH-13 / ADR-0050 §D50.5 — strictest-wins composed audit class
    /// at the time the grant was issued. The 3 production fire
    /// listeners (Template A/C/D) call
    /// [`crate::permissions::compose_audit_class_with_source`] with
    /// (org_default, template_ar, optional_override) and stamp the
    /// resolved class here.
    ///
    /// `#[serde(default = "Grant::default_audit_class")]` shielding
    /// produces `AuditClass::Silent` (the loosest — preserves the
    /// "no escalation by silent migration" property per concept-doc 07
    /// line 71) for pre-CH-13 grant rows.
    ///
    /// Concept doc anchor:
    /// `permissions/07-templates-and-tools.md` §"audit_class
    /// Composition Through Templates" line 69 — *"The resolved
    /// `audit_class` is recorded on the Grant at issuance time"*.
    #[serde(default = "Grant::default_audit_class")]
    pub audit_class: crate::audit::AuditClass,
    /// CH-08 / ADR-0052 §D52.3 — typed allocate-refinement constraints per concept-doc
    /// 02 line 197. `None` for non-`[allocate]` grants AND for legacy/pre-CH-08 grants
    /// (serde-default-shielded). When `Some(_)`, the contained `AllocateRefinement`
    /// narrows the allocate sub-capabilities (`no_further_delegation`, `max_depth`)
    /// per concept-doc 02 line 197 + concept-doc 03 line 54. Mirrors the CH-11 D48.1
    /// (Grant.approval_mode) + CH-13 D50.5 (Grant.audit_class) field-add pattern.
    #[serde(default)]
    pub allocate_refinement: Option<crate::permissions::AllocateRefinement>,
}

impl Grant {
    /// Default `audit_class` for pre-CH-13 grants deserialised from
    /// storage. Loosest class (`Silent`) is the safe default — it
    /// preserves the concept-doc 07 line 71 invariant that adopting a
    /// template *can never silently downgrade* an org's audit posture
    /// (a `Silent` placeholder cannot mask a stricter org default that
    /// the composer would otherwise apply).
    pub fn default_audit_class() -> crate::audit::AuditClass {
        crate::audit::AuditClass::Silent
    }
}

/// Per-grant approval mode — captures whether a read under this grant
/// requires runtime subordinate approval, and (when so) which
/// org-level [`ConsentPolicy`] applies.
///
/// CH-11 / ADR-0048 §D48.1 — denormalises the issuing org's
/// [`ConsentPolicy`] into the grant so the Permission Check engine's
/// Step 6 can branch without re-reading the org node.
///
/// Variants:
/// - [`ApprovalMode::Implicit`] (default) — no runtime gating; the
///   grant's holder reads without further consent. Pre-CH-11 grants
///   decode as this via serde's `#[serde(default)]` shield on
///   [`Grant::approval_mode`].
/// - [`ApprovalMode::SubordinateRequired`] — runtime consent required
///   from the read target ("subordinate"). The carried
///   [`ConsentPolicy`] (Implicit / OneTime / PerSession) governs how
///   often the prompt fires. Concept doc 06 line 244 — "Templates
///   auto-issue grants with `approval_mode: subordinate_required`".
/// - [`ApprovalMode::HumanApprovalRequired`] — reserved for future
///   human-in-the-loop flows. Engine returns `Pending` with a marker
///   reason; no real handling at CH-11 (placeholder per §3
///   "What this chunk does NOT do").
///
/// Wire format is a tag-payload object:
/// - `{"kind": "implicit"}`
/// - `{"kind": "subordinate_required", "policy": "per_session"}`
/// - `{"kind": "human_approval_required"}`
///
/// **phi-core leverage**: none — phi-core has no consent / governance
/// concept.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApprovalMode {
    /// No runtime consent gate — the grant's holder reads directly.
    /// Default for pre-CH-11 grants.
    #[default]
    Implicit,
    /// Read target's consent required at runtime. The carried
    /// [`ConsentPolicy`] governs how often the prompt fires (Implicit
    /// auto-acks; OneTime prompts once per (subordinate, org) pair;
    /// PerSession prompts on every new session).
    SubordinateRequired { policy: crate::model::ConsentPolicy },
    /// Reserved for future human-in-the-loop flows. CH-11 records the
    /// variant for forward-compat; the engine returns `Pending` with a
    /// marker reason but has no real handling.
    HumanApprovalRequired,
}

/// Default response when a `Requested` consent crosses its
/// `deadline_at` and the sweeper auto-flips it to `TimedOut`.
///
/// CH-11 / ADR-0048 §D48.4 — `Organization.approval_timeout_default_response`
/// carries this. Concept doc 06 line 349 — "Default response on
/// timeout: `deny`, on the principle that absence of consent is not
/// consent."
///
/// Wire format is a snake-case JSON string: `"deny"` or `"allow"`.
///
/// **phi-core leverage**: none — phi-only governance primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutResponse {
    /// Treat a `TimedOut` consent as a denial. Concept-doc default
    /// (line 349 — "absence of consent is not consent").
    #[default]
    Deny,
    /// Treat a `TimedOut` consent as an approval. Operators that want
    /// the opposite of the default opt in via
    /// `Organization.approval_timeout_default_response: allow`
    /// (concept doc 06 line 349).
    Allow,
}

/// Who holds a grant / submits a request / is an approver.
///
/// Serde uses the externally-tagged default format (rather than internal
/// tagging) because several variants wrap primitives — `Agent(AgentId)` and
/// `System(String)` are newtype tuple variants, which serde rejects under
/// `#[serde(tag = ...)]`. The wire format is `{"agent": "<uuid>"}` /
/// `{"system": "system:genesis"}`, which round-trips cleanly through JSON
/// and through SurrealDB's `object` storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// CH-06 / D-new-11: instance-identity tags
    /// (`#kind:auth_request`, `auth_request:<id>`).
    #[serde(default)]
    pub tags: Vec<String>,
    /// CH-14 / ADR-0053 §D53.5 — the `Grant` under whose authority this
    /// AR was submitted. `None` for bootstrap ARs (the chain root) and
    /// for legacy / pre-CH-14 rows (decoded as `None` via
    /// `#[serde(default)]`). Forms the AR-to-Grant link of the
    /// authority chain; the Grant-to-AR link continues to live on
    /// [`Grant::descends_from`]. Adoption-AR-side wiring (Template
    /// A/B/C/D/E adoption-AR builders setting `Some(firing_grant_id)`)
    /// is deferred to a successor chunk per the ADR's §D53.5
    /// scope-control decision; the field exists from CH-14 forward so
    /// successor chunks need not migrate.
    #[serde(default)]
    pub descends_from_grant: Option<GrantId>,
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
///
/// Concept doc: [`docs/specs/v0/concepts/permissions/06-multi-scope-consent.md`]
/// §"Consent Node (New Node Type)" lines 351–363 + §"Consent Lifecycle"
/// lines 369–414. CH-09 lifts the 11-field shape into typed Rust; CH-10
/// will add the state-transition function + lifecycle invariants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub id: ConsentId,
    pub agent_id: AgentId,
    pub scope: ConsentScope,
    /// Lifecycle state. `#[serde(default)]` so any pre-CH-09 wire
    /// payload that omits the field decodes to `Acknowledged` per
    /// `Default for ConsentState` (ADR-0045 §D45.4).
    #[serde(default)]
    pub state: ConsentState,
    pub requested_at: DateTime<Utc>,
    #[serde(default)]
    pub responded_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocable: bool,
    pub provenance: String,
    /// CH-10 / ADR-0047 §D47.6 — When a `Requested` consent should be
    /// auto-flipped to `TimedOut` by the sweeper. `None` for legacy
    /// rows and `Acknowledged`-by-default (implicit-policy) consents
    /// that have no deadline. The sweeper only flips rows where
    /// `state = Requested AND deadline_at <= now`. Population of this
    /// field at consent creation time lands at CH-11 (per-policy
    /// minter rewrite).
    #[serde(default)]
    pub deadline_at: Option<DateTime<Utc>>,
}

/// What a Consent record covers — the org under whose policy the consent
/// applies, and (optionally) the Authority Templates and Actions inside
/// that scope.
///
/// Per concept doc 06 §"Consent Node" lines 355–357, the `templates` /
/// `actions` lists narrow the consent to a subset of the org's template
/// catalog; an empty list means "any template / any action covered by
/// the org-level policy". The Permission Check engine intersects this
/// scope with the grant's claimed reach at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub org: OrgId,
    #[serde(default)]
    pub templates: Vec<TemplateId>,
    #[serde(default)]
    pub actions: Vec<crate::permissions::action::Action>,
    /// CH-11 / ADR-0048 §D48.2 — Per-Session axis. `None` means the
    /// consent is not session-scoped (Implicit / OneTime policies).
    /// `Some(id)` means the consent only applies to reads on the
    /// named session (Per-Session policy). The
    /// [`crate::permissions::manifest::ConsentIndex`] projection keys
    /// on `(subordinate, org, Option<session_id>)` to discriminate.
    ///
    /// `#[serde(default)]` shields pre-CH-11 wire payloads — they
    /// decode as `None`. Migration 0013 leaves the SurrealDB schema
    /// untouched here because `scope` is already
    /// `FLEXIBLE TYPE object` per ADR-0045 §D45.5 — the new field
    /// rides under the FLEXIBLE shield.
    #[serde(default)]
    pub session_id: Option<SessionId>,
}

/// Lifecycle state of a [`Consent`] record.
///
/// Per concept doc 06 §"Consent Lifecycle" lines 369–414. CH-09 ships
/// the type only; CH-10 (drift D-new-05) wires the transition function
/// + forward-only revocation invariant + timeout handling.
///
/// `Default = Acknowledged` matches the `implicit` consent policy
/// semantic (concept-doc line 410 — "Consent is auto-Acknowledged at
/// agent creation, no request is sent") and acts as the safe back-compat
/// default for any pre-CH-09 wire payload that omits the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsentState {
    Requested,
    #[default]
    Acknowledged,
    Declined,
    Revoked,
    TimedOut,
    Expired,
}

impl ConsentState {
    /// Every variant in declaration order. Used by tests + by future
    /// CH-10 transition logic to enumerate the closed set without
    /// reflection.
    pub const ALL: [ConsentState; 6] = [
        ConsentState::Requested,
        ConsentState::Acknowledged,
        ConsentState::Declined,
        ConsentState::Revoked,
        ConsentState::TimedOut,
        ConsentState::Expired,
    ];
}

/// Publish-time authority declaration for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAuthorityManifest {
    pub id: NodeId,
    /// Name of the `ToolDefinition` this manifest is attached to.
    pub tool_name: String,
    pub resource: Vec<String>,
    pub transitive: Vec<String>,
    pub actions: Vec<crate::permissions::action::Action>,
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
    /// CH-06 / D-new-11: instance-identity tags
    /// (`#kind:inbox`, `inbox:<id>`).
    #[serde(default)]
    pub tags: Vec<String>,
}

/// An Agent's outbox (sent `AgentMessage`s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxObject {
    pub id: NodeId,
    pub agent_id: AgentId,
    pub created_at: DateTime<Utc>,
    /// CH-06 / D-new-11: instance-identity tags
    /// (`#kind:outbox`, `outbox:<id>`).
    #[serde(default)]
    pub tags: Vec<String>,
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

// ============================================================================
// Identity — emergent self of an LLM Agent. CH-16 / D-new-01 closure.
//
// Materialized per concept-`agent.md` § "Identity (Emergent, Event-Driven)" +
// § "Identity Node Content — Provisional Direction" — the four-field v0
// commitment: `self_description` + `lived` + `witnessed` + `embedding`.
// Updated reactively on session end / memory extraction / skill change /
// rating received (CH-21 wires the first emitter via the memory-extraction
// listener body). LLM agents only — Human Agents have no system-computed
// Identity per concept-`human-agent.md` § "No Identity"; the guard lives
// in `Repository::upsert_identity` (defensive) + `apply_agent_creation`
// (preventive) per ADR-0039.
// ============================================================================

/// Emergent self of an LLM Agent. Materialized at agent creation with the
/// 4-field v0 commitment shape; reactively updated by future writers
/// (CH-21 memory-extraction listener; M6 skill/rating events).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    pub id: NodeId,
    /// Keyed-by; UNIQUE per migration 0009. One row per LLM agent.
    pub agent_id: AgentId,
    /// Agent-authored natural-language bio (≤ 500 tokens). Default empty
    /// at create per concept-`agent.md` line 325 ("Rewritten by the agent
    /// or by a system-agent synthesiser on session end / skill change /
    /// rating event"); CH-21 populates via the synthesiser path.
    #[serde(default)]
    pub self_description: String,
    /// Direct-doing metrics. Defaults to zeroed struct at create.
    #[serde(default)]
    pub lived: LivedExperience,
    /// Supervised-doing metrics. Defaults to zeroed struct at create.
    /// Empty for non-supervisor agents per concept-`agent.md` line 327.
    #[serde(default)]
    pub witnessed: WitnessedExperience,
    /// Vector derived from `self_description`. Empty at create per
    /// ADR-0038 §D38.3; populated when the embedding provider integrates
    /// (M6-DEFERRED-03).
    #[serde(default)]
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
    /// Bumped on every reactive update — preserves "incrementally updated"
    /// semantics from concept-`agent.md` § "Materialization".
    pub updated_at: DateTime<Utc>,
}

impl Identity {
    /// Build a fresh Identity for a newly-created LLM agent. All four
    /// content fields are zero/empty per ADR-0038 §D38.2; both timestamps
    /// are set to `now`. Callers MUST verify the owning agent's `kind`
    /// is `AgentKind::Llm` before calling — the defensive guard at
    /// `Repository::upsert_identity` is the seatbelt, not the API
    /// contract.
    pub fn default_for_llm(agent_id: AgentId, now: DateTime<Utc>) -> Self {
        Self {
            id: NodeId::new(),
            agent_id,
            self_description: String::new(),
            lived: LivedExperience::default(),
            witnessed: WitnessedExperience::default(),
            embedding: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Direct-doing metrics. Field names + types match concept-`agent.md`
/// line 326 + concept-`ontology.md` line 29 verbatim.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LivedExperience {
    #[serde(default)]
    pub sessions_completed: u64,
    #[serde(default)]
    pub sessions_successful: u64,
    /// Last 20 ratings per the rolling window in concept-`token-economy.md`.
    #[serde(default)]
    pub ratings_window: Vec<RatingPoint>,
    #[serde(default)]
    pub skills: Vec<SkillRef>,
    /// Top tags by frequency across completed work.
    #[serde(default)]
    pub specializations: Vec<String>,
}

/// Supervised-doing metrics. Field names match concept-`agent.md` line 327.
/// Empty for non-supervisor agents.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WitnessedExperience {
    #[serde(default)]
    pub memories_extracted: u64,
    #[serde(default)]
    pub subordinates_observed: Vec<AgentId>,
    /// Tracks how many extractions were private vs public (see
    /// `concepts/permissions/05-memory-sessions.md` § "Memory as
    /// Resource Class").
    #[serde(default)]
    pub extraction_scope_distribution: ExtractionScopeDistribution,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionScopeDistribution {
    #[serde(default)]
    pub private: u64,
    #[serde(default)]
    pub public: u64,
}

/// Lightweight rating point — agent-id of rater + score + timestamp.
/// Full token-economy semantics (rater-weight, decay) deferred to M6+
/// per concept-`token-economy.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RatingPoint {
    pub rater: AgentId,
    pub score: f32,
    pub at: DateTime<Utc>,
}

/// Reference to a skill the agent currently has. M6 swaps to a typed
/// `SkillId` once the Skill node lands; v0.1 placeholder is name-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRef {
    pub name: String,
}
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
    /// CH-06 / D-new-11: instance-identity tags
    /// (`#kind:session`, `session:<id>`).
    #[serde(default)]
    pub tags: Vec<String>,
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

    // ---- CH-26 / ADR-0061 §D61.2 — `tags: Vec<String>` field on
    // Organization + Project carries `#[serde(default)]` for backward
    // compat with pre-CH-26 wire payloads. The test below pins the
    // round-trip behaviour: deserialising a pre-CH-26 JSON blob (without
    // the `tags` field) MUST resolve to `tags = vec![]` rather than
    // erroring on the missing field.

    #[test]
    fn organization_tags_serde_default_round_trips_pre_ch26_payload() {
        // Pre-CH-26 wire payload — no `tags` field.
        let pre_ch26_json = serde_json::json!({
            "id": OrgId::new(),
            "display_name": "Acme",
            "consent_policy": "implicit",
            "audit_class_default": "logged",
            "approval_timeout_default_response": "deny",
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        let org: Organization =
            serde_json::from_value(pre_ch26_json).expect("pre-CH-26 payload must deserialise");
        assert_eq!(
            org.tags,
            Vec::<String>::new(),
            "tags must default to vec![] when missing from wire payload"
        );
    }

    #[test]
    fn project_tags_serde_default_round_trips_pre_ch26_payload() {
        // Pre-CH-26 wire payload — no `tags` field.
        let pre_ch26_json = serde_json::json!({
            "id": ProjectId::new(),
            "name": "Atlas",
            "description": "memory benchmark",
            "status": "planned",
            "shape": "shape_a",
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        let project: Project =
            serde_json::from_value(pre_ch26_json).expect("pre-CH-26 payload must deserialise");
        assert_eq!(
            project.tags,
            Vec::<String>::new(),
            "tags must default to vec![] when missing from wire payload"
        );
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

    // ---- CH-16 / D-new-01 — Identity struct unit tests --------------------

    #[test]
    fn identity_default_for_llm_zeroes_all_content_fields() {
        let agent_id = AgentId::new();
        let now = chrono::Utc::now();
        let iden = Identity::default_for_llm(agent_id, now);
        assert_eq!(iden.agent_id, agent_id);
        assert_eq!(iden.self_description, "");
        assert_eq!(iden.lived, LivedExperience::default());
        assert_eq!(iden.witnessed, WitnessedExperience::default());
        assert!(iden.embedding.is_empty());
        assert_eq!(iden.created_at, now);
        assert_eq!(iden.updated_at, now);
    }

    #[test]
    fn identity_serde_round_trips() {
        let iden = Identity::default_for_llm(AgentId::new(), chrono::Utc::now());
        let j = serde_json::to_string(&iden).expect("serialize");
        let back: Identity = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, iden);
    }

    #[test]
    fn lived_experience_default_is_zeroed() {
        let lived = LivedExperience::default();
        assert_eq!(lived.sessions_completed, 0);
        assert_eq!(lived.sessions_successful, 0);
        assert!(lived.ratings_window.is_empty());
        assert!(lived.skills.is_empty());
        assert!(lived.specializations.is_empty());
    }

    #[test]
    fn witnessed_experience_default_is_zeroed() {
        let w = WitnessedExperience::default();
        assert_eq!(w.memories_extracted, 0);
        assert!(w.subordinates_observed.is_empty());
        assert_eq!(w.extraction_scope_distribution.private, 0);
        assert_eq!(w.extraction_scope_distribution.public, 0);
    }

    #[test]
    fn lived_experience_serde_round_trips_with_populated_values() {
        let lived = LivedExperience {
            sessions_completed: 42,
            sessions_successful: 38,
            ratings_window: vec![
                RatingPoint {
                    rater: AgentId::new(),
                    score: 0.95,
                    at: chrono::Utc::now(),
                },
                RatingPoint {
                    rater: AgentId::new(),
                    score: 0.7,
                    at: chrono::Utc::now(),
                },
            ],
            skills: vec![SkillRef {
                name: "rust".into(),
            }],
            specializations: vec!["devops".into(), "security".into()],
        };
        let j = serde_json::to_string(&lived).expect("serialize");
        let back: LivedExperience = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, lived);
    }

    #[test]
    fn witnessed_experience_serde_round_trips_with_populated_values() {
        let w = WitnessedExperience {
            memories_extracted: 17,
            subordinates_observed: vec![AgentId::new(), AgentId::new()],
            extraction_scope_distribution: ExtractionScopeDistribution {
                private: 12,
                public: 5,
            },
        };
        let j = serde_json::to_string(&w).expect("serialize");
        let back: WitnessedExperience = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, w);
    }

    #[test]
    fn rating_point_serde_round_trips() {
        let rp = RatingPoint {
            rater: AgentId::new(),
            score: 0.875,
            at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&rp).expect("serialize");
        let back: RatingPoint = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, rp);
    }

    #[test]
    fn identity_pre_ch16_row_with_missing_fields_deserialises() {
        // Pre-migration rows have only id + agent_id + created_at +
        // updated_at; the four content fields must default-fill via
        // serde defaults.
        let now = chrono::Utc::now();
        let aid = AgentId::new();
        let pre_ch16 = serde_json::json!({
            "id": NodeId::new().to_string(),
            "agent_id": aid.to_string(),
            "created_at": now,
            "updated_at": now,
        });
        let iden: Identity = serde_json::from_value(pre_ch16).expect("deserialize pre-CH-16 row");
        assert_eq!(iden.agent_id, aid);
        assert_eq!(iden.self_description, "");
        assert!(iden.embedding.is_empty());
    }

    // ---- CH-09: Consent full shape (D-new-04 closure) ----------------------

    fn sample_consent() -> Consent {
        let now = Utc::now();
        Consent {
            id: ConsentId::new(),
            agent_id: AgentId::new(),
            scope: ConsentScope {
                org: OrgId::new(),
                templates: vec![],
                actions: vec![],
                session_id: None,
            },
            state: ConsentState::Acknowledged,
            requested_at: now,
            responded_at: Some(now),
            revoked_at: None,
            revocable: true,
            provenance: "agent:test@unit".into(),
            deadline_at: None,
        }
    }

    #[test]
    fn consent_struct_carries_concept_doc_fields() {
        // CH-09 closes drift D-new-04 by lifting concept doc 06 lines
        // 351-363 into the typed Consent struct. This test pins the
        // 11-field shape via construction (a missing or extra field
        // wouldn't compile).
        let c = sample_consent();
        // 11 logical fields per concept doc — `scope` is nested
        // (3 sub-fields). Touch every leaf to defend against silent
        // field removal.
        let _ = (
            c.id,
            c.agent_id,
            c.scope.org,
            c.scope.templates.clone(),
            c.scope.actions.clone(),
            c.state,
            c.requested_at,
            c.responded_at,
            c.revoked_at,
            c.revocable,
            c.provenance.clone(),
        );
    }

    #[test]
    fn consent_state_all_has_six_variants() {
        assert_eq!(ConsentState::ALL.len(), 6);
        let set: std::collections::HashSet<_> = ConsentState::ALL.iter().collect();
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn consent_state_default_is_acknowledged() {
        // Per ADR-0045 §D45.4: matches the implicit-policy semantic
        // (concept doc 06 line 410 — auto-Acknowledged at agent
        // creation) and acts as safe back-compat default for legacy
        // wire payloads that omit the field.
        assert_eq!(ConsentState::default(), ConsentState::Acknowledged);
    }

    #[test]
    fn consent_state_serializes_as_snake_case() {
        // Migration 0010 stores `state` as a string with DEFAULT
        // "acknowledged"; this test pins the wire form so the migration
        // and the Rust enum can never drift.
        for (state, wire) in [
            (ConsentState::Requested, "requested"),
            (ConsentState::Acknowledged, "acknowledged"),
            (ConsentState::Declined, "declined"),
            (ConsentState::Revoked, "revoked"),
            (ConsentState::TimedOut, "timed_out"),
            (ConsentState::Expired, "expired"),
        ] {
            let j = serde_json::to_value(state).expect("serialize");
            assert_eq!(j, serde_json::Value::String(wire.into()));
            let back: ConsentState = serde_json::from_value(j).expect("deserialize");
            assert_eq!(back, state);
        }
    }

    #[test]
    fn consent_scope_serde_round_trip() {
        let scope = ConsentScope {
            org: OrgId::new(),
            templates: vec![TemplateId::new(), TemplateId::new()],
            actions: vec![
                crate::permissions::action::Action::Read,
                crate::permissions::action::Action::List,
            ],
            session_id: None,
        };
        let j = serde_json::to_value(&scope).expect("serialize");
        let back: ConsentScope = serde_json::from_value(j).expect("deserialize");
        assert_eq!(back.org, scope.org);
        assert_eq!(back.templates, scope.templates);
        assert_eq!(back.actions, scope.actions);
    }

    #[test]
    fn consent_full_serde_round_trip() {
        let c = sample_consent();
        let j = serde_json::to_value(&c).expect("serialize");
        let back: Consent = serde_json::from_value(j).expect("deserialize");
        assert_eq!(back.id, c.id);
        assert_eq!(back.agent_id, c.agent_id);
        assert_eq!(back.scope.org, c.scope.org);
        assert_eq!(back.state, c.state);
        assert_eq!(back.revocable, c.revocable);
        assert_eq!(back.provenance, c.provenance);
    }

    #[test]
    fn consent_legacy_wire_format_decodes_with_defaults() {
        // Pre-CH-09 wire payloads carried only 5 fields; #[serde(default)]
        // on `responded_at` + `revoked_at` and Default::Acknowledged on
        // `state` ensure such payloads (or any future payload with
        // missing optional fields) still decode cleanly.
        let now = Utc::now();
        let payload = serde_json::json!({
            "id": ConsentId::new().to_string(),
            "agent_id": AgentId::new().to_string(),
            "scope": {
                "org": OrgId::new().to_string(),
            },
            // No `state` — must default to Acknowledged.
            "requested_at": now,
            // No `responded_at`, no `revoked_at` — both Option, default None.
            "revocable": true,
            "provenance": "",
        });
        let c: Consent = serde_json::from_value(payload).expect("legacy decode");
        assert_eq!(c.state, ConsentState::Acknowledged);
        assert!(c.responded_at.is_none());
        assert!(c.revoked_at.is_none());
        assert!(c.scope.templates.is_empty());
        assert!(c.scope.actions.is_empty());
    }

    #[test]
    fn consent_with_action_in_scope_serializes_action_as_snake_case() {
        // Cross-check: ConsentScope.actions uses the typed Action from
        // ADR-0043. Action serializes snake_case; this test pins the
        // ConsentScope wire form for `[Read]`.
        let scope = ConsentScope {
            org: OrgId::new(),
            templates: vec![],
            actions: vec![crate::permissions::action::Action::Read],
            session_id: None,
        };
        let j = serde_json::to_value(&scope).expect("serialize");
        assert_eq!(
            j.get("actions"),
            Some(&serde_json::json!(["read"])),
            "ConsentScope.actions must serialize as snake_case strings"
        );
    }

    // ----------------------------------------------------------------
    // CH-11 / ADR-0048 — ApprovalMode + TimeoutResponse + ConsentScope
    // .session_id + Organization defaults coverage.
    // ----------------------------------------------------------------

    #[test]
    fn approval_mode_default_is_implicit() {
        // Pre-CH-11 grants decode without an `approval_mode` key; the
        // `#[serde(default)]` shield on `Grant.approval_mode` resolves
        // to `ApprovalMode::Implicit`, so Step 6 short-circuits to
        // Allow for legacy grants. ADR-0048 §D48.1.
        assert_eq!(ApprovalMode::default(), ApprovalMode::Implicit);
    }

    #[test]
    fn approval_mode_serde_roundtrip_all_variants() {
        // Tag-payload wire format: `{"kind": "implicit"}`, etc. The
        // `subordinate_required` variant carries `policy: ConsentPolicy`
        // verbatim alongside the tag.
        let cases = [
            (
                ApprovalMode::Implicit,
                serde_json::json!({"kind": "implicit"}),
            ),
            (
                ApprovalMode::SubordinateRequired {
                    policy: crate::model::ConsentPolicy::Implicit,
                },
                serde_json::json!({"kind": "subordinate_required", "policy": "implicit"}),
            ),
            (
                ApprovalMode::SubordinateRequired {
                    policy: crate::model::ConsentPolicy::OneTime,
                },
                serde_json::json!({"kind": "subordinate_required", "policy": "one_time"}),
            ),
            (
                ApprovalMode::SubordinateRequired {
                    policy: crate::model::ConsentPolicy::PerSession,
                },
                serde_json::json!({"kind": "subordinate_required", "policy": "per_session"}),
            ),
            (
                ApprovalMode::HumanApprovalRequired,
                serde_json::json!({"kind": "human_approval_required"}),
            ),
        ];
        for (variant, expected_wire) in cases {
            let j = serde_json::to_value(&variant).expect("serialize");
            assert_eq!(j, expected_wire, "wire form for {variant:?}");
            let back: ApprovalMode = serde_json::from_value(j).expect("deserialize");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn timeout_response_default_is_deny() {
        // Concept doc 06 line 349 — "Default response on timeout: deny,
        // on the principle that absence of consent is not consent."
        assert_eq!(TimeoutResponse::default(), TimeoutResponse::Deny);
    }

    #[test]
    fn timeout_response_serde_roundtrip() {
        // Snake-case JSON-string wire form so migration 0013's
        // `TYPE string DEFAULT "deny"` ASSERT clause matches the
        // serde rendering exactly.
        for (variant, wire) in [
            (TimeoutResponse::Deny, "deny"),
            (TimeoutResponse::Allow, "allow"),
        ] {
            let j = serde_json::to_value(variant).expect("serialize");
            assert_eq!(j, serde_json::json!(wire));
            let back: TimeoutResponse =
                serde_json::from_value(serde_json::json!(wire)).expect("deserialize");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn consent_scope_session_id_serde_default_is_none() {
        // Pre-CH-11 wire payloads carry no `session_id` key on
        // ConsentScope; the `#[serde(default)]` shield resolves the
        // missing field to `None`. ADR-0048 §D48.2 — the
        // FLEXIBLE-shielded scope object accepts the new field
        // optionally.
        let pre_ch11 = serde_json::json!({
            "org": OrgId::new().to_string(),
            "templates": [],
            "actions": [],
        });
        let scope: ConsentScope = serde_json::from_value(pre_ch11).expect("decode legacy scope");
        assert!(scope.session_id.is_none());
    }

    #[test]
    fn organization_approval_timeout_default_is_project_duration() {
        // CH-11 / ADR-0048 §D48.4 — the `approval_timeout` field
        // defaults to ProjectDuration via `#[serde(default)]` on
        // Organization.approval_timeout. Concept doc 06 line 322.
        // We assert via the default-instance path: the field's
        // `#[serde(default)]` shield resolves missing keys to the
        // enum's Default impl.
        let now = Utc::now();
        let org_pre_ch11 = serde_json::json!({
            "id": OrgId::new().to_string(),
            "display_name": "legacy-org",
            "consent_policy": "implicit",
            "audit_class_default": "logged",
            "created_at": now,
        });
        let org: Organization =
            serde_json::from_value(org_pre_ch11).expect("decode legacy organization");
        assert_eq!(
            org.approval_timeout,
            crate::model::ApprovalTimeout::ProjectDuration
        );
    }

    #[test]
    fn organization_approval_timeout_default_response_default_is_deny() {
        // CH-11 / ADR-0048 §D48.4 — the
        // `approval_timeout_default_response` field defaults to Deny
        // via `#[serde(default = "Organization::default_timeout_response")]`.
        // Concept doc 06 line 349.
        let now = Utc::now();
        let org_pre_ch11 = serde_json::json!({
            "id": OrgId::new().to_string(),
            "display_name": "legacy-org",
            "consent_policy": "implicit",
            "audit_class_default": "logged",
            "created_at": now,
        });
        let org: Organization =
            serde_json::from_value(org_pre_ch11).expect("decode legacy organization");
        assert_eq!(org.approval_timeout_default_response, TimeoutResponse::Deny);
    }

    #[test]
    fn grant_approval_mode_default_is_implicit_via_legacy_decode() {
        // CH-11 / ADR-0048 §D48.1 — Pre-CH-11 grants decode as
        // `ApprovalMode::Implicit`, preserving the engine's existing
        // back-compat path through Step 6.
        let now = Utc::now();
        let legacy_grant = serde_json::json!({
            "id": GrantId::new().to_string(),
            "holder": {"agent": AgentId::new().to_string()},
            "action": ["read"],
            "resource": {"uri": "system:root"},
            "fundamentals": [],
            "descends_from": null,
            "delegable": false,
            "issued_at": now,
            "revoked_at": null,
        });
        let g: Grant = serde_json::from_value(legacy_grant).expect("decode legacy grant");
        assert_eq!(g.approval_mode, ApprovalMode::Implicit);
    }

    #[test]
    fn principal_ref_partial_eq_round_trips() {
        // CH-18 / ADR-0056 §D56.4 — `PrincipalRef` derives `PartialEq` so
        // `classify_principal` can compare against `ar.requestor` and
        // `slot.approver` via `==` rather than via destructure-match.
        // Each of the 5 variants must be `==` to a freshly-constructed
        // copy carrying the same payload.
        let agent_id = AgentId::new();
        assert_eq!(PrincipalRef::Agent(agent_id), PrincipalRef::Agent(agent_id),);

        let user_id = UserId::new();
        assert_eq!(PrincipalRef::User(user_id), PrincipalRef::User(user_id));

        let org_id = OrgId::new();
        assert_eq!(
            PrincipalRef::Organization(org_id),
            PrincipalRef::Organization(org_id),
        );

        let project_id = ProjectId::new();
        assert_eq!(
            PrincipalRef::Project(project_id),
            PrincipalRef::Project(project_id),
        );

        assert_eq!(
            PrincipalRef::System("system:genesis".into()),
            PrincipalRef::System("system:genesis".into()),
        );

        // Sanity: distinct payloads are NOT equal.
        let other_agent = AgentId::new();
        assert_ne!(
            PrincipalRef::Agent(agent_id),
            PrincipalRef::Agent(other_agent),
        );
    }
}
