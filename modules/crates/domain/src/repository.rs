//! Persistence boundary.
//!
//! The `store` crate implements [`Repository`] against SurrealDB. The
//! domain layer only ever talks to `Repository`, keeping SurrealDB as a
//! swappable adapter (see ADR-0001 / ADR-0007 in M0).
//!
//! ## Object-safety + generics
//!
//! `Repository` is `Arc<dyn Repository>`-dispatchable by M0 convention, so
//! its methods cannot carry generic type parameters. The three typed
//! ownership helpers ([`upsert_ownership`], [`upsert_creation`],
//! [`upsert_allocation`]) are therefore **free functions** on this module
//! rather than trait methods. They use the sealed
//! [`crate::model::Principal`] / [`crate::model::Resource`] marker traits
//! to reject wrong-pair ID types at compile time, then delegate to the
//! trait's `*_raw` methods (which take `NodeId`). See ADR-0015 for the
//! rationale.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::AuditEvent;
use crate::model::ids::{
    AgentId, AuditEventId, AuthRequestId, EdgeId, GrantId, McpServerId, ModelProviderId, NodeId,
    OrgId, ProjectId, SecretId, SessionId,
};
use crate::model::nodes::{
    Agent, AgentProfile, AgentRole, AuthRequest, Channel, Consent, Grant, InboxObject,
    LoopRecordNode, Memory, Organization, OutboxObject, PrincipalRef, Project, ProjectShape,
    ResourceRef, Session, SessionGovernanceState, Template, ToolAuthorityManifest, TurnNode, User,
};
use crate::model::{
    AgentCatalogEntry, AgentExecutionLimitsOverride, Composite, ExternalService, ModelRuntime,
    PlatformDefaults, Principal, Resource, SecretCredential, SecretRef, SessionDetail,
    ShapeBPendingProject, SystemAgentRuntimeStatus, TenantSet,
};

// ----------------------------------------------------------------------------
// Error type
// ----------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("backend error: {0}")]
    Backend(String),
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

// ----------------------------------------------------------------------------
// ProjectShapeCounts — returned by count_projects_by_shape_in_org.
// ----------------------------------------------------------------------------

/// Project-shape bucketed counts for one org. Powers the M4/P8
/// dashboard rewrite's `ProjectsSummary.shape_a` + `.shape_b` tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProjectShapeCounts {
    pub shape_a: u32,
    pub shape_b: u32,
}

impl ProjectShapeCounts {
    pub fn total(self) -> u32 {
        self.shape_a.saturating_add(self.shape_b)
    }
}

// ----------------------------------------------------------------------------
// Sealed-material envelope — domain-side projection of the crypto layer's
// output (`store::SealedSecret`) that the repository trait can reference
// without depending on the `store` crate. The two base64 strings are
// stored directly on the `secrets_vault` row.
// ----------------------------------------------------------------------------

/// The persisted sealed form of a secret: AES-GCM ciphertext + nonce,
/// both base64-encoded (standard alphabet, no padding — see
/// [`crate::model::composites_m2`] docs and the vault schema in
/// `store/migrations/0001_initial.surql` for the rationale).
///
/// The `store::crypto` layer produces the bytes; the repository stores
/// them. The domain layer never holds plaintext.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedBlob {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
}

// ----------------------------------------------------------------------------
// Bootstrap-credential row — lightweight projection for the handful of
// columns the bootstrap flow actually needs. Full field shape lives on the
// SurrealDB `bootstrap_credentials` table.
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BootstrapCredentialRow {
    /// SurrealDB record id, opaque to callers.
    pub record_id: String,
    pub digest: String,
    pub created_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

// ----------------------------------------------------------------------------
// Bootstrap claim payload — the full set of entities apply_bootstrap_claim
// must commit atomically.
// ----------------------------------------------------------------------------

/// All the writes the S01 flow commits in one atomic batch.
///
/// The caller (server::bootstrap::claim) assembles the entities from the
/// submitted credential + validated form input; the repository runs them
/// in a single SurrealDB transaction.
#[derive(Debug, Clone)]
pub struct BootstrapClaim {
    /// SurrealDB record id of the unconsumed credential to mark `consumed_at`.
    pub credential_record_id: String,
    /// The new Human Agent node (R-SYS-s01-2).
    pub human_agent: crate::model::nodes::Agent,
    /// The Human Agent's channel (Slack / email / web).
    pub channel: crate::model::nodes::Channel,
    /// The Human Agent's inbox (R-SYS-s01-3).
    pub inbox: crate::model::nodes::InboxObject,
    /// The Human Agent's outbox (R-SYS-s01-3).
    pub outbox: crate::model::nodes::OutboxObject,
    /// The Bootstrap Auth Request, pre-built in `Approved` state with the
    /// `system:genesis` slot already filled (R-SYS-s01-1).
    pub auth_request: crate::model::nodes::AuthRequest,
    /// The `[allocate]`-on-`system:root` Grant (R-SYS-s01-4).
    pub grant: crate::model::nodes::Grant,
    /// Platform-level catalogue seeds — `(uri, kind)` pairs (R-SYS-s01-3).
    /// Must include at minimum `system:root` and the new inbox/outbox URIs.
    pub catalogue_entries: Vec<(String, String)>,
    /// The `PlatformAdminClaimed` audit event (R-ADMIN-01-N1 /
    /// R-SYS-s01 side-effects).
    pub audit_event: crate::audit::AuditEvent,
}

// ----------------------------------------------------------------------------
// Org-creation payload (M3/P3) — everything `apply_org_creation` commits
// atomically. See ADR-0022 (compound transaction) + ADR-0023
// (inherit-from-snapshot — no per-agent ExecutionLimits/ContextConfig/
// RetryConfig nodes are created).
// ----------------------------------------------------------------------------

/// Full payload for `Repository::apply_org_creation`.
///
/// Constructed by the M3/P4 wizard orchestrator from the validated POST
/// body + resolved platform defaults. The orchestrator clones
/// `phi_core::AgentProfile` out of `organization.defaults_snapshot`,
/// overrides `name` / `system_prompt` per role, and bundles everything
/// here before handing the payload to the repo for a single-transaction
/// commit.
///
/// Ordering of the internal vectors is preserved — adoption auth
/// requests commit in the order the orchestrator supplies them, which
/// is also the order their companion `authority_template.adopted`
/// audit events emit in `emit_audit_batch`.
#[derive(Debug, Clone)]
pub struct OrgCreationPayload {
    /// The new org node. Orchestrator freezes `defaults_snapshot` here
    /// per ADR-0019 before calling; repo persists as-is.
    pub organization: crate::model::nodes::Organization,
    /// The CEO Human Agent.
    pub ceo_agent: crate::model::nodes::Agent,
    /// CEO's reach-me channel (Slack / email / web).
    pub ceo_channel: crate::model::nodes::Channel,
    /// CEO's inbox (governance inbox, not phi-core runtime inbox).
    pub ceo_inbox: crate::model::nodes::InboxObject,
    /// CEO's outbox.
    pub ceo_outbox: crate::model::nodes::OutboxObject,
    /// CEO's `[allocate]`-on-`org:<id>` grant — the root authority
    /// over the org's control-plane surface.
    pub ceo_grant: crate::model::nodes::Grant,
    /// Two system agents + their `AgentProfile` nodes. Each profile's
    /// `blueprint: phi_core::agents::profile::AgentProfile` carries
    /// the role-specific system prompt.
    pub system_agents: [(
        crate::model::nodes::Agent,
        crate::model::nodes::AgentProfile,
    ); 2],
    /// Org-level token budget pool (1:1 with org).
    pub token_budget_pool: crate::model::composites_m3::TokenBudgetPool,
    /// One Template-E-shaped adoption AR per enabled template (subset
    /// of A / B / C / D). Orchestrator is free to supply empty if the
    /// org adopts no templates.
    pub adoption_auth_requests: Vec<crate::model::nodes::AuthRequest>,
    /// Catalogue seeds: `(resource_uri, kind)` pairs scoped to this org.
    /// Must include at minimum the org's control-plane URI
    /// (`org:<id>`), each adoption AR's template URI
    /// (`org:<id>/template:<kind>`), and the CEO's inbox/outbox URIs.
    pub catalogue_entries: Vec<(String, String)>,
}

/// Everything the caller (M3/P4 handler) needs after a successful
/// `apply_org_creation` commit — ids to emit audit events against,
/// ids to include in the HTTP response, ids to thread into the
/// post-commit message-delivery hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgCreationReceipt {
    pub org_id: OrgId,
    pub ceo_agent_id: AgentId,
    pub ceo_channel_id: NodeId,
    pub ceo_inbox_id: NodeId,
    pub ceo_outbox_id: NodeId,
    pub ceo_grant_id: GrantId,
    pub system_agent_ids: [AgentId; 2],
    pub system_agent_profile_ids: [NodeId; 2],
    pub token_budget_pool_id: NodeId,
    /// Adoption AR ids, in the same order as
    /// `OrgCreationPayload.adoption_auth_requests` (so the caller can
    /// pair each with its companion audit event).
    pub adoption_auth_request_ids: Vec<AuthRequestId>,
}

// ----------------------------------------------------------------------------
// M4/P3 — compound-tx payloads for project + agent creation.
// ----------------------------------------------------------------------------

/// Full payload for [`Repository::apply_project_creation`].
///
/// **Shape A** (single-org, immediate): caller supplies one element in
/// `owning_orgs`; the compound tx materialises the project + edges in
/// a single BEGIN/COMMIT. Shape B's pending-approval submit (pre
/// materialisation) uses `Repository::create_auth_request` + an
/// AR-builder directly — this payload is for the **materialisation**
/// step, which runs identically for Shape A's happy path and Shape
/// B's both-approve outcome. In both cases, `owning_orgs` carries
/// every co-owner (1 for A; 2 for B).
///
/// Every referenced principal (lead, members, sponsors, co-owners)
/// must already exist as an Agent / Organization row — the compound
/// tx does not create them. The caller should invoke
/// `apply_agent_creation` first for any net-new lead/member agents.
///
/// ## phi-core leverage
///
/// None — Project is a pure phi governance composite; the
/// payload carries no phi-core types.
#[derive(Debug, Clone)]
pub struct ProjectCreationPayload {
    /// The new Project row. `id`, `name`, `shape`, `status`, `created_at`
    /// must be set by the orchestrator before calling.
    pub project: crate::model::nodes::Project,
    /// Owning orgs — exactly 1 for Shape A, exactly 2 for Shape B. The
    /// in-memory + SurrealDB impls emit one `BELONGS_TO` edge per entry.
    pub owning_orgs: Vec<OrgId>,
    /// The designated project lead. An existing Agent whose
    /// `owning_org` must match one of `owning_orgs`. A `HAS_LEAD` edge
    /// is emitted from the project to this agent; the caller typically
    /// pairs the compound tx with a post-commit domain event on the
    /// bus so Template A's fire-listener issues the lead grant.
    pub lead_agent_id: AgentId,
    /// Additional agents on the project (`HAS_AGENT` edges). May be
    /// empty.
    pub member_agent_ids: Vec<AgentId>,
    /// Optional sponsor agents (`HAS_SPONSOR` edges). Typically the
    /// CEO of the owning org.
    pub sponsor_agent_ids: Vec<AgentId>,
    /// Catalogue seeds for project-scoped grants. Must include at
    /// minimum `project:<id>`.
    pub catalogue_entries: Vec<(String, String)>,
}

/// Ids the caller (M4/P6 handler) needs after a successful
/// [`Repository::apply_project_creation`] commit — to emit the
/// `platform.project.created` audit event, the `HasLeadEdgeCreated`
/// domain event (so Template A fires), and to build the HTTP
/// response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCreationReceipt {
    pub project_id: ProjectId,
    /// Owning-org ids, in the same order the caller supplied them.
    pub owning_org_ids: Vec<OrgId>,
    pub lead_agent_id: AgentId,
    /// `HAS_LEAD` edge id — included so the caller can trace the
    /// domain event back to its source edge at incident time.
    pub has_lead_edge_id: EdgeId,
}

/// Full payload for [`Repository::apply_agent_creation`].
///
/// Atomic write: Agent + inbox + outbox + optional profile + optional
/// initial `ExecutionLimits` override + default grants + the edge set
/// (`HAS_INBOX`, `HAS_OUTBOX`, `MEMBER_OF`, optional `HAS_PROFILE`).
///
/// ## phi-core leverage
///
/// `initial_execution_limits_override` carries
/// `phi_core::context::execution::ExecutionLimits` via the M4/P1
/// wrap. `profile.blueprint` carries `phi_core::AgentProfile` via the
/// existing wrap. Both transit without re-declaration.
#[derive(Debug, Clone)]
pub struct AgentCreationPayload {
    /// The new Agent row. `owning_org` must be `Some(_)`.
    pub agent: crate::model::nodes::Agent,
    pub inbox: crate::model::nodes::InboxObject,
    pub outbox: crate::model::nodes::OutboxObject,
    /// Optional profile. Required for LLM-kind agents in typical
    /// flows; pre-M4 style also lets Humans carry one for
    /// consistency.
    pub profile: Option<crate::model::nodes::AgentProfile>,
    /// Default grants issued at creation time — e.g. `read` on its
    /// own inbox. The orchestrator assembles these per-role.
    pub default_grants: Vec<crate::model::nodes::Grant>,
    /// Optional per-agent `ExecutionLimits` override row. Absent =
    /// inherit from the org snapshot per ADR-0023; present = ADR-0027
    /// opt-in override path. Must satisfy
    /// [`AgentExecutionLimitsOverride::is_bounded_by`] against the
    /// owning org's snapshot before calling — the compound tx does
    /// not re-check.
    pub initial_execution_limits_override: Option<AgentExecutionLimitsOverride>,
    /// Catalogue seeds (e.g. the new inbox/outbox URIs under the
    /// owning org).
    pub catalogue_entries: Vec<(String, String)>,
}

/// Ids the caller needs after a successful
/// [`Repository::apply_agent_creation`] commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCreationReceipt {
    pub agent_id: AgentId,
    pub owning_org_id: OrgId,
    pub inbox_id: NodeId,
    pub outbox_id: NodeId,
    pub profile_id: Option<NodeId>,
    pub default_grant_ids: Vec<GrantId>,
    pub execution_limits_override_id: Option<NodeId>,
}

// ----------------------------------------------------------------------------
// The trait
// ----------------------------------------------------------------------------

/// Object-safe persistence surface. All methods take plain IDs and data
/// structs; the typed ownership-edge helpers are free functions below.
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    // ---- Health -----------------------------------------------------------

    /// Readiness check used by `/healthz/ready`.
    async fn ping(&self) -> RepositoryResult<()>;

    // ---- Node CRUD (M1-critical) ------------------------------------------
    //
    // Each `create_*` method expects the caller to have allocated the ID.
    // Each `get_*` returns `Ok(None)` when the row is absent (rather than
    // `Err(NotFound)`), matching the "optional-find" convention. Callers
    // that need hard failure use `Repository::get_*` + `ok_or(NotFound)`.

    async fn create_agent(&self, agent: &Agent) -> RepositoryResult<()>;
    async fn get_agent(&self, id: AgentId) -> RepositoryResult<Option<Agent>>;
    /// Upsert the Agent row (matched on `id`). Used by M4/P5's profile
    /// editor to persist mutable-field changes (`display_name`).
    /// Immutable-field drift is caller-enforced — this method does not
    /// re-validate `kind` / `role` / `owning_org`.
    async fn upsert_agent(&self, agent: &Agent) -> RepositoryResult<()>;

    async fn create_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()>;
    /// Fetch the (single) [`AgentProfile`] row whose `agent_id == agent`.
    /// At M4 the schema enforces 1:1 via a UNIQUE index in migration
    /// 0004; this method returns `Ok(None)` for Human agents that
    /// never get a profile row.
    async fn get_agent_profile_for_agent(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentProfile>>;
    /// Upsert the profile row (matched on `id`). Used by M4/P5's
    /// profile editor to persist blueprint + parallelize edits.
    async fn upsert_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()>;

    async fn create_user(&self, user: &User) -> RepositoryResult<()>;

    async fn create_organization(&self, org: &Organization) -> RepositoryResult<()>;
    async fn get_organization(&self, id: OrgId) -> RepositoryResult<Option<Organization>>;

    async fn create_template(&self, template: &Template) -> RepositoryResult<()>;

    async fn create_channel(&self, channel: &Channel) -> RepositoryResult<()>;

    async fn create_inbox(&self, inbox: &InboxObject) -> RepositoryResult<()>;
    async fn create_outbox(&self, outbox: &OutboxObject) -> RepositoryResult<()>;

    async fn create_memory(&self, memory: &Memory) -> RepositoryResult<()>;

    async fn create_consent(&self, consent: &Consent) -> RepositoryResult<()>;

    async fn create_tool_authority_manifest(
        &self,
        manifest: &ToolAuthorityManifest,
    ) -> RepositoryResult<()>;

    /// Bootstrap status: returns the first Human-kind Agent if the
    /// platform admin has been claimed, otherwise `None`. Used by
    /// `GET /api/v0/bootstrap/status` in P6 and by the acceptance tests.
    async fn get_admin_agent(&self) -> RepositoryResult<Option<Agent>>;

    // ---- Grants -----------------------------------------------------------

    async fn create_grant(&self, grant: &Grant) -> RepositoryResult<()>;
    async fn get_grant(&self, id: GrantId) -> RepositoryResult<Option<Grant>>;
    async fn revoke_grant(&self, id: GrantId, revoked_at: DateTime<Utc>) -> RepositoryResult<()>;
    async fn list_grants_for_principal(
        &self,
        principal: &PrincipalRef,
    ) -> RepositoryResult<Vec<Grant>>;

    // ---- Auth Requests ----------------------------------------------------

    async fn create_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()>;
    async fn get_auth_request(&self, id: AuthRequestId) -> RepositoryResult<Option<AuthRequest>>;
    /// Full-replace update — P4 will build finer-grained helpers for slot
    /// transitions on top of this.
    async fn update_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()>;
    async fn list_active_auth_requests_for_resource(
        &self,
        resource: &ResourceRef,
    ) -> RepositoryResult<Vec<AuthRequest>>;

    // ---- Ownership edges (raw — use free-function wrappers) ---------------
    //
    // Typed entry points: [`upsert_ownership`], [`upsert_creation`],
    // [`upsert_allocation`] below. Callers should prefer those so endpoint
    // type errors fire at compile time.

    async fn upsert_ownership_raw(
        &self,
        resource_id: NodeId,
        owner_id: NodeId,
        auth_request: Option<AuthRequestId>,
    ) -> RepositoryResult<EdgeId>;
    async fn upsert_creation_raw(
        &self,
        creator_id: NodeId,
        resource_id: NodeId,
    ) -> RepositoryResult<EdgeId>;
    async fn upsert_allocation_raw(
        &self,
        from_id: NodeId,
        to_id: NodeId,
        resource: &ResourceRef,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<EdgeId>;

    // ---- Bootstrap credentials --------------------------------------------

    async fn put_bootstrap_credential(
        &self,
        digest: String,
    ) -> RepositoryResult<BootstrapCredentialRow>;
    async fn find_unconsumed_credential(
        &self,
        digest: &str,
    ) -> RepositoryResult<Option<BootstrapCredentialRow>>;
    async fn consume_bootstrap_credential(&self, record_id: &str) -> RepositoryResult<()>;
    /// List bootstrap credentials. Used by the s01 claim flow to verify
    /// a supplied plaintext against every stored argon2id hash (can't
    /// look up by hash directly because each row has its own salt).
    ///
    /// When `unconsumed_only` is true, filters out rows with a
    /// `consumed_at` timestamp. When false, returns every row (the
    /// caller filters as needed).
    async fn list_bootstrap_credentials(
        &self,
        unconsumed_only: bool,
    ) -> RepositoryResult<Vec<BootstrapCredentialRow>>;

    // ---- Resources Catalogue ----------------------------------------------

    async fn seed_catalogue_entry(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        kind: &str,
    ) -> RepositoryResult<()>;
    async fn catalogue_contains(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
    ) -> RepositoryResult<bool>;

    // ---- Bootstrap claim (atomic — the full s01 flow in one txn) ---------

    /// Apply the System Bootstrap Template adoption in a single atomic
    /// write. Per `concepts/permissions/02` §System Bootstrap Template
    /// and `requirements/system/s01-bootstrap-template-adoption.md`
    /// (R-SYS-s01-1 … R-SYS-s01-6), **all** of the following writes must
    /// succeed together or none at all:
    ///
    /// 1. Create the Human Agent node.
    /// 2. Create the Inbox and Outbox composites.
    /// 3. Seed the platform-level `resources_catalogue` entries.
    /// 4. Create the (auto-Approved) Bootstrap Auth Request.
    /// 5. Create the `[allocate]`-on-`system:root` Grant.
    /// 6. Write the `PlatformAdminClaimed` audit event.
    /// 7. Mark the bootstrap credential consumed.
    ///
    /// If the method returns `Ok(())`, the write is durably applied. If
    /// it returns `Err(_)`, **no** partial state survives — the
    /// credential remains unconsumed and the caller may retry. SurrealDB
    /// impl uses a `BEGIN TRANSACTION … COMMIT TRANSACTION` envelope;
    /// in-memory fake uses its single write-lock to serialise the batch.
    async fn apply_bootstrap_claim(&self, claim: &BootstrapClaim) -> RepositoryResult<()>;

    // ---- Audit ------------------------------------------------------------

    /// Writes the event to the primary store. Emitters are expected to
    /// populate `prev_event_hash` before calling; this method does not
    /// look up the chain (keeps the repository surface narrow).
    async fn write_audit_event(&self, event: &AuditEvent) -> RepositoryResult<()>;

    /// Look up a single audit event by id. Returns `None` when no row
    /// exists. Used by the acceptance suite to verify end-to-end that a
    /// handler's stated `audit_event_id` really landed in storage with
    /// the expected class + provenance.
    async fn get_audit_event(&self, id: AuditEventId) -> RepositoryResult<Option<AuditEvent>>;

    /// Returns the hash of the most recent event within `org_scope`, or
    /// `None` if no events exist yet for that scope.
    async fn last_event_hash_for_org(
        &self,
        org: Option<OrgId>,
    ) -> RepositoryResult<Option<[u8; 32]>>;

    // ================================================================
    // M2 additions — admin pages 02–05.
    //
    // These methods land with M2 / P2 (commitment C3 in the archived
    // plan). Each one is documented inline; handlers in P4–P7 wrap them
    // behind Permission Check + audit emission via `handler_support`.
    // ================================================================

    // ---- Secrets vault (M2 / P4 — credentials vault) ---------------

    /// Insert a new vault row. Fails with
    /// [`RepositoryError::Conflict`] when the slug is already in use
    /// (UNIQUE INDEX on `secrets_vault.slug`).
    async fn put_secret(
        &self,
        credential: &SecretCredential,
        sealed: &SealedBlob,
    ) -> RepositoryResult<()>;

    /// Look up a secret by its human-readable slug. Returns both the
    /// catalogue entry and the sealed bytes so the reveal path can
    /// unseal without a second round-trip.
    async fn get_secret_by_slug(
        &self,
        slug: &SecretRef,
    ) -> RepositoryResult<Option<(SecretCredential, SealedBlob)>>;

    /// List every vault entry. Ciphertext bytes are NOT returned —
    /// the list view is metadata-only (slug, custodian, sensitive,
    /// last_rotated_at). Callers wanting to unseal a specific entry
    /// follow up with [`Self::get_secret_by_slug`].
    async fn list_secrets(&self) -> RepositoryResult<Vec<SecretCredential>>;

    /// Rotate the sealed material + bump `last_rotated_at`. The slug
    /// and custodian are unchanged. `NotFound` when the id does not
    /// exist.
    async fn rotate_secret(
        &self,
        id: SecretId,
        new_sealed: &SealedBlob,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()>;

    /// Reassign custody of a secret to a different Agent. The sealed
    /// material is untouched — the Agent delegation is a governance
    /// concern, not a crypto one.
    async fn reassign_secret_custodian(
        &self,
        id: SecretId,
        new_custodian: AgentId,
    ) -> RepositoryResult<()>;

    // ---- Model providers (M2 / P5) ---------------------------------

    /// Upsert a model-runtime row. The embedded
    /// [`phi_core::provider::model::ModelConfig`] is stored as a
    /// flexible object so phi-core's field evolution does not force a
    /// phi migration.
    async fn put_model_provider(&self, provider: &ModelRuntime) -> RepositoryResult<()>;

    /// List model-runtime rows. When `include_archived` is `false`,
    /// rows whose `archived_at` is non-null are filtered out.
    async fn list_model_providers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ModelRuntime>>;

    /// Mark a model-runtime row archived (soft delete). Does not
    /// remove the row; audit + provenance still references it.
    async fn archive_model_provider(
        &self,
        id: ModelProviderId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()>;

    // ---- MCP servers (M2 / P6) -------------------------------------

    /// Upsert an external-service (`mcp_server`) row.
    async fn put_mcp_server(&self, server: &ExternalService) -> RepositoryResult<()>;

    async fn list_mcp_servers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ExternalService>>;

    /// Overwrite `tenants_allowed` without cascading. Used when the
    /// new set is a superset of the old (no revocations required).
    /// For shrinking, callers MUST use [`Self::narrow_mcp_tenants`]
    /// so the cascade runs in the same transaction.
    async fn patch_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
    ) -> RepositoryResult<()>;

    async fn archive_mcp_server(&self, id: McpServerId, at: DateTime<Utc>) -> RepositoryResult<()>;

    // ---- Platform defaults (M2 / P7) -------------------------------

    /// Read the singleton row. `None` when no row has been seeded yet
    /// (fresh install); handlers seed via [`Self::put_platform_defaults`]
    /// on first write.
    async fn get_platform_defaults(&self) -> RepositoryResult<Option<PlatformDefaults>>;

    /// Upsert the singleton row. The `singleton = 1` UNIQUE INDEX on
    /// the underlying table guarantees at most one row.
    async fn put_platform_defaults(&self, defaults: &PlatformDefaults) -> RepositoryResult<()>;

    // ---- Cascade (M2 / P6 — tenant narrowing + bulk revocation) ----

    /// Narrow an MCP server's `tenants_allowed`, cascading revocation
    /// to every grant whose provenance `descends_from` an Auth Request
    /// scoped to a now-excluded org.
    ///
    /// Returns one entry per affected (org, AR) pair, listing the
    /// grants that were revoked. The caller (M2/P6 handler) emits one
    /// summary `McpServerTenantAccessRevoked` event plus one
    /// `auth_request.revoked` event per entry. Revocation is
    /// forward-only: revoked grants carry `revoked_at = at`.
    ///
    /// Must be called only when `new_allowed` is STRICTLY SMALLER than
    /// the existing set — the handler validates this pre-flight. A
    /// no-shrink call returns an empty `Vec` and leaves the state
    /// unchanged (the handler routes those through
    /// [`Self::patch_mcp_tenants`] instead).
    async fn narrow_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TenantRevocation>>;

    /// Revoke every live grant whose `descends_from = ar`. Returns the
    /// list of affected grant ids so the caller can emit per-grant
    /// audit events. No-op when no matching grants exist.
    async fn revoke_grants_by_descends_from(
        &self,
        ar: AuthRequestId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<GrantId>>;

    // ---- Catalogue (M2 / P4 + P5 + P6) -----------------------------

    /// Seed a catalogue entry tagged with `composite.kind_name()`.
    /// Thin convenience wrapper over [`Self::seed_catalogue_entry`] —
    /// used by every M2 admin write that creates a composite instance
    /// so Permission-Check Step 0 resolves on the resulting URI.
    async fn seed_catalogue_entry_for_composite(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        composite: Composite,
    ) -> RepositoryResult<()>;

    // ================================================================
    // M3 additions — admin pages 06–07.
    //
    // Org-scoped list methods that page 07 (dashboard) relies on.
    // Every read is by `OrgId`; unknown-org returns `Ok(vec![])`
    // (not `Err(NotFound)`) so empty-org paths stay distinct from
    // repository failures. Landed in M3/P2 per commitment C5.
    // ================================================================

    /// List every Agent whose `owning_org == Some(org)`. Includes all
    /// kinds (Human / Intern / Contract / System). Returns an empty
    /// `Vec` when the org is unknown or has no members yet. Used by
    /// the M3/P5 dashboard's `AgentsSummary` panel.
    async fn list_agents_in_org(&self, org: OrgId) -> RepositoryResult<Vec<Agent>>;

    /// List every organization in the platform. Used by M3/P4's
    /// `GET /api/v0/orgs` index. No pagination at M3 (dashboard fits
    /// tens of orgs); M4 adds a cursor-based variant if platform
    /// volume demands.
    async fn list_all_orgs(&self) -> RepositoryResult<Vec<Organization>>;

    /// List every Project belonging to `org`. **M4/P2 note**: return
    /// type migrated from `Vec<ProjectId>` to `Vec<Project>` alongside
    /// the full [`Project`] struct landing at M4/P1. Dashboard
    /// call-sites (M3/P5) used only `.len()` so the change is
    /// mechanical at the call-site level.
    ///
    /// Belongs-to semantics: an entry is included if **any** of its
    /// `BELONGS_TO` edges targets `org`. Shape A projects have a
    /// single edge (to the owning org); Shape B projects have two
    /// edges (one per co-owner) so they appear in the list for both
    /// co-owning orgs.
    async fn list_projects_in_org(&self, org: OrgId) -> RepositoryResult<Vec<Project>>;

    // ---- M4/P2 surface -------------------------------------------------
    //
    // Six project-centric + four agent-centric reads land here; the
    // first six feed page 08 + 11 business logic at P4–P7, the last
    // four wrap the new per-agent `ExecutionLimits` override path
    // per ADR-0027. The trait-level docs capture the contract; the
    // in-memory + SurrealDB impls share it verbatim.

    /// List every Agent whose `owning_org == Some(org)`, optionally
    /// filtered by governance role. `None` returns everyone; passing
    /// `Some(AgentRole::X)` returns only agents whose `role ==
    /// Some(X)`. Agents with `role == None` (pre-M4 rows) match only
    /// when the filter is `None`.
    ///
    /// Used by page 08 (agent roster list) role-chip filters + by the
    /// M4/P8 dashboard rewrite's `AgentsSummary.{executive, admin,
    /// member, intern, contract, system, unclassified}` counters.
    async fn list_agents_in_org_by_role(
        &self,
        org: OrgId,
        role: Option<AgentRole>,
    ) -> RepositoryResult<Vec<Agent>>;

    /// Fetch a single Project by id. Returns `Ok(None)` when the
    /// project row is absent (matches the "optional-find" convention
    /// used by [`Repository::get_agent`] / [`Repository::get_organization`]).
    /// Used by page 11 (project detail) handler + M4/P3's compound-tx
    /// rollback verification.
    async fn get_project(&self, id: ProjectId) -> RepositoryResult<Option<Project>>;

    /// List every Project in `org` matching `shape`. Narrowing
    /// variant of [`Repository::list_projects_in_org`] — handlers use
    /// this when rendering shape-specific panels (e.g. the dashboard
    /// lists Shape A vs Shape B counts separately).
    async fn list_projects_by_shape_in_org(
        &self,
        org: OrgId,
        shape: ProjectShape,
    ) -> RepositoryResult<Vec<Project>>;

    /// Count every Project in `org` split by [`ProjectShape`].
    ///
    /// Powers the M4/P8 dashboard rewrite's `ProjectsSummary.shape_a`
    /// + `.shape_b` counters (M3 carryover C-M4-3). Cheap — the impl
    /// pushes the count into the backend rather than materialising
    /// the row set in Rust.
    async fn count_projects_by_shape_in_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<ProjectShapeCounts>;

    /// List every Project that `agent` is the designated lead of
    /// (`HAS_LEAD` edge from Project → Agent). Used by the dashboard
    /// viewer-role resolution (`ProjectLead` path) + by page 11's
    /// "projects I lead" panel.
    ///
    /// **M4/P2 note**: the `HAS_LEAD` edge has no production writers
    /// until M4/P3's `apply_project_creation` compound tx. This
    /// method returns `Vec::new()` for every agent until then; the
    /// trait surface exists now so P4 handlers can wire the call
    /// site.
    async fn list_projects_led_by_agent(&self, agent: AgentId) -> RepositoryResult<Vec<Project>>;

    /// Persist (create-or-replace) a project row. Shipped at M4/P7 for
    /// the in-place OKR editor (`PATCH /api/v0/projects/:id/okrs`) —
    /// the orchestrator validates the OKR patch in Rust, builds a
    /// fully-replaced [`Project`] struct, then calls this method.
    ///
    /// Invariants enforced at the callsite (not this method): the
    /// project already exists (callers read-then-write), the shape +
    /// owning-org edges don't change (OKR mutations never change
    /// governance), and `created_at` is preserved from the existing
    /// row.
    ///
    /// M5+ may extend this to accept a narrower `ProjectPatch` shape
    /// if status transitions + resource-boundary edits land as
    /// separate endpoints; at M4 the OKR editor is the only writer so
    /// full-row replacement is the minimum-surface path.
    async fn upsert_project(&self, project: &Project) -> RepositoryResult<()>;

    /// Look up the opt-in per-agent `ExecutionLimits` override row
    /// for `agent`. Returns `Ok(None)` when the agent inherits from
    /// the org snapshot (ADR-0023 default path).
    async fn get_agent_execution_limits_override(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentExecutionLimitsOverride>>;

    /// Persist (create-or-replace) the per-agent override. Enforces
    /// the UNIQUE `owning_agent` index at the storage layer; callers
    /// are responsible for bounds-checking against the owning org's
    /// snapshot before invoking (see
    /// [`AgentExecutionLimitsOverride::is_bounded_by`]).
    async fn set_agent_execution_limits_override(
        &self,
        row: &AgentExecutionLimitsOverride,
    ) -> RepositoryResult<()>;

    /// Idempotent DELETE of an agent's override row. Succeeds whether
    /// or not a row exists.
    async fn clear_agent_execution_limits_override(&self, agent: AgentId) -> RepositoryResult<()>;

    /// Resolve the effective `ExecutionLimits` for `agent`:
    /// 1. If a per-agent override row exists → that row's `limits`.
    /// 2. Else if the agent has an `owning_org` whose
    ///    `defaults_snapshot.execution_limits` is set → that value.
    /// 3. Else `None` (the agent has neither override nor org
    ///    snapshot — unusual; caller should treat as "use
    ///    `phi_core::ExecutionLimits::default()`").
    ///
    /// Single entry point every caller should use — prevents the
    /// "two sources of truth" drift ADR-0027 warns about.
    async fn resolve_effective_execution_limits(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<phi_core::context::execution::ExecutionLimits>>;

    /// Count an agent's in-flight sessions — used by the M4/P5 profile
    /// editor to gate the `ModelConfig` change path (D-M4-3): changing
    /// an agent's model while a session is running returns
    /// `409 ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`.
    ///
    /// **M5/P2 update**: both first-party impls (in-memory +
    /// SurrealDB) override this with a real count. The default
    /// body below stays as a safety net — `Ok(0)` — so thin test
    /// decorators that don't care about the 409 path still compile.
    /// Production Repositories MUST override; the flipped behaviour
    /// closes C-M5-5.
    ///
    /// **Test ergonomics**: acceptance tests that want to exercise
    /// the 409 path wrap the `Arc<dyn Repository>` with a thin
    /// decorator that returns a non-zero count for one specific
    /// agent id.
    async fn count_active_sessions_for_agent(&self, _agent: AgentId) -> RepositoryResult<u32> {
        Ok(0)
    }

    /// List every non-terminal Auth Request requested by a principal
    /// belonging to `org`. "Non-terminal" = state ∈ {Draft, Pending,
    /// InProgress} (terminal states are Approved / Denied / Partial
    /// / Expired / Withdrawn / Escalated / Archived). Excludes
    /// `archived = true` rows regardless of state.
    ///
    /// Used by the M3/P5 dashboard's `PendingAuthRequests` panel.
    async fn list_active_auth_requests_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>>;

    /// List up to `limit` most-recent audit events whose `org_scope ==
    /// Some(org)`. Results are ordered newest-first by `timestamp`.
    /// Used by the M3/P5 dashboard's `RecentAuditEvents` panel.
    async fn list_recent_audit_events_for_org(
        &self,
        org: OrgId,
        limit: usize,
    ) -> RepositoryResult<Vec<AuditEvent>>;

    /// List every adoption AR (template kinds A / B / C / D) for
    /// `org`. Filter: AR's resource URI starts with
    /// `org:<id>/template:` AND `provenance_template` points at a
    /// template node of one of the A-D kinds (M3/P4 wires the
    /// provenance on persist).
    ///
    /// Adoption ARs are terminal-Approved (Template-E-shaped); this
    /// method returns every one regardless of `archived` so the
    /// dashboard's `AdoptedTemplates` panel can display both active
    /// and revoked adoptions.
    ///
    /// Returns `Vec<AuthRequest>` so callers can render
    /// `template_kind` from the referenced Template node without a
    /// second query (the caller already has the Template rows cached
    /// from the org-creation flow).
    async fn list_adoption_auth_requests_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>>;

    /// Fetch the single [`TokenBudgetPool`] associated with `org`, or
    /// `None` if none exists (should not happen for orgs created via
    /// [`Repository::apply_org_creation`] — the compound tx always
    /// creates exactly one pool per org). Used by the M3/P5 dashboard's
    /// `TokenBudget` panel to render `used / total`.
    ///
    /// [`TokenBudgetPool`]: crate::model::composites_m3::TokenBudgetPool
    async fn get_token_budget_pool_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Option<crate::model::composites_m3::TokenBudgetPool>>;

    /// Count audit events whose `org_scope = Some(org)` AND
    /// `audit_class = Alerted` AND `timestamp >= since`. Used by the
    /// M3/P5 dashboard's `AlertedEventsCount` panel (R-ADMIN-07-R5 "in
    /// the last 24 hours"). The SurrealDB impl pushes the filter into
    /// the query — the in-memory impl walks the store under the lock.
    async fn count_alerted_events_for_org_since(
        &self,
        org: OrgId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryResult<u32>;

    // ---- Org creation compound tx (M3/P3) -----------------------------
    //
    // Single atomic write: Organization + CEO Agent/Channel/Inbox/Outbox/
    // Grant + 2 system agents + 2 AgentProfile nodes + TokenBudgetPool
    // + N adoption Auth Requests + the edge set
    // (HasCeo / HasMember / MemberOf / HasInbox / HasOutbox / HasChannel
    // / HasProfile) + catalogue seeds.
    //
    // See ADR-0022 for the compound-tx rationale; ADR-0023 pins the
    // inherit-from-snapshot invariant (no per-agent phi-core-wrap
    // nodes). `UsesModel` edge wiring is deferred to M5 session launch.

    /// Commit the full M3 org-creation write in one atomic transaction.
    ///
    /// On `Ok(OrgCreationReceipt)`, every entity in `payload` is
    /// durable and the caller may emit the `platform.organization.created`
    /// + N `authority_template.adopted` audit events via
    /// `handler_support::audit::emit_audit_batch`. On `Err(_)`, **no**
    /// partial state survives — the SurrealDB impl wraps every write
    /// in `BEGIN TRANSACTION … COMMIT TRANSACTION`; the in-memory impl
    /// validates first and then applies under a single write-lock.
    ///
    /// Pre-conditions (caller's responsibility):
    /// - `payload.organization.id` is fresh (unique).
    /// - Every `system_agents[n].0.owning_org == Some(payload.organization.id)`.
    /// - `payload.ceo_agent.owning_org == Some(payload.organization.id)`.
    /// - Every `adoption_auth_requests[n].state == AuthRequestState::Approved`.
    /// - `payload.token_budget_pool.owning_org == payload.organization.id`.
    async fn apply_org_creation(
        &self,
        payload: &OrgCreationPayload,
    ) -> RepositoryResult<OrgCreationReceipt>;

    // ---- M4/P3 — Project + Agent creation compound txns ---------------

    /// Commit the full M4 project-creation write in one atomic
    /// transaction (Shape A immediate, Shape B post-both-approve).
    ///
    /// On `Ok`, the following are durable:
    /// - The Project row.
    /// - One `BELONGS_TO` edge per entry in `payload.owning_orgs`.
    /// - `HAS_LEAD` edge (project → lead agent).
    /// - Zero or more `HAS_AGENT` edges (project → member agents).
    /// - Zero or more `HAS_SPONSOR` edges (project → sponsor agents).
    /// - One `HAS_PROJECT` edge per owning org (so the dashboard's
    ///   project count picks it up via the existing org-scoped read).
    /// - Catalogue seeds.
    ///
    /// On `Err(_)`, no partial state survives.
    ///
    /// **Post-commit**: the caller should emit
    /// `DomainEvent::HasLeadEdgeCreated { project, lead, ... }` on the
    /// event bus so Template A's fire-listener issues the lead grant.
    /// The compound tx intentionally does NOT emit — fail-safe
    /// semantics per ADR-0028 (commit durable before emit).
    ///
    /// Pre-conditions (caller's responsibility):
    /// - `payload.owning_orgs.len() == 1` (Shape A) or `== 2` (Shape B).
    /// - Every referenced Agent / Org row exists.
    /// - `payload.project.shape` matches the arity of `owning_orgs`.
    async fn apply_project_creation(
        &self,
        payload: &ProjectCreationPayload,
    ) -> RepositoryResult<ProjectCreationReceipt>;

    /// Commit the full M4 agent-creation write in one atomic
    /// transaction (pages 09 create mode + post-bootstrap CEO
    /// creation).
    ///
    /// On `Ok`, the following are durable:
    /// - The Agent row.
    /// - Inbox + Outbox rows.
    /// - Optional AgentProfile row.
    /// - Optional `agent_execution_limits` override row.
    /// - N default grants.
    /// - Edges: `HAS_INBOX`, `HAS_OUTBOX`, `MEMBER_OF` to owning org,
    ///   optional `HAS_PROFILE`.
    /// - Catalogue seeds.
    ///
    /// On `Err(_)`, no partial state survives.
    ///
    /// Pre-conditions (caller's responsibility):
    /// - `payload.agent.owning_org == Some(<existing org id>)`.
    /// - `payload.agent.role.is_valid_for(payload.agent.kind)` if `role`
    ///   is set.
    /// - Every default grant's `holder == PrincipalRef::Agent(payload.agent.id)`
    ///   (defensive — the handler assembles grants for the new agent).
    /// - If `initial_execution_limits_override` is present,
    ///   `is_bounded_by(org.defaults_snapshot.execution_limits)`.
    async fn apply_agent_creation(
        &self,
        payload: &AgentCreationPayload,
    ) -> RepositoryResult<AgentCreationReceipt>;

    // ---- M5/P2 — Session + sidecar + catalog + status surface ---------
    //
    // Landed at M5/P2 per the M5 plan §P2 Deliverables.
    // **phi-core leverage** (Q1 at P2): 0 new `use phi_core::*` imports
    // in this file — all phi-core types flow through the already-wrapped
    // `Session` / `LoopRecordNode` / `TurnNode` node structs imported
    // above. The M4/P1 wrap pattern absorbs every phi-core type
    // transit; no new direct imports cross the repo trait.

    /// Persist a fresh governance `Session` row + its first
    /// [`LoopRecordNode`] + the `runs_session` edge linking session
    /// to project — the writes the M5/P4 launch handler needs to
    /// make the session visible to page 11's "Recent sessions" panel.
    ///
    /// **Drift-addendum invariant (D1.1)**: the SurrealDB `session`
    /// table inherits a mandatory `created_at: string` column from
    /// 0001. Implementations MUST populate `created_at` (typically
    /// the same wall-clock value as `session.started_at`) inside the
    /// compound write or SurrealDB's SCHEMAFULL ASSERT will reject
    /// the row.
    async fn persist_session(
        &self,
        session: &Session,
        first_loop: &LoopRecordNode,
    ) -> RepositoryResult<()>;

    /// Append a new [`LoopRecordNode`] to an existing session.
    /// Called by [`BabyPhiSessionRecorder`] (M5/P3) on each
    /// `AgentStart` that opens a follow-on loop in the same
    /// session (continuation / rerun / branch).
    async fn append_loop_record(&self, loop_record: &LoopRecordNode) -> RepositoryResult<()>;

    /// Append a materialised [`TurnNode`] to an existing loop.
    /// Called by [`BabyPhiSessionRecorder`] on each `TurnEnd`. The
    /// same `created_at: string` mandatory-field invariant from the
    /// drift addendum (D1.1) applies to the `turn` table — impls
    /// populate it alongside the turn's own `started_at`.
    async fn append_turn(&self, turn: &TurnNode) -> RepositoryResult<()>;

    /// Persist one `phi_core::AgentEvent` for the audit replay tier.
    /// M5/P1 deliberately keeps this pass-through; the `AgentEvent`
    /// stream flows through [`BabyPhiSessionRecorder`]'s storage
    /// sink rather than surfacing phi-core types at the repo
    /// boundary (Q3 rejection — preserves the repo's phi-core-free
    /// trait shape). Implementations serialise the event via
    /// [`serde_json::Value`] on the way in so callers may evolve
    /// the `AgentEvent` shape without forcing a trait change.
    async fn append_agent_event(
        &self,
        session: SessionId,
        event: serde_json::Value,
    ) -> RepositoryResult<()>;

    /// Fetch the full session drill-down (session + every loop +
    /// every turn) keyed by `session`. Returns [`None`] when the
    /// session is unknown (404 on the HTTP tier).
    ///
    /// Reconstructs the nested `phi_core::Session.loops` tree from
    /// baby-phi's flattened storage layout. Per-Turn queries stay
    /// O(1) against the flat `turn` table; full drill-down is one
    /// compound SELECT per tier + in-process group-by.
    async fn fetch_session(&self, session: SessionId) -> RepositoryResult<Option<SessionDetail>>;

    /// List every session whose `owning_project == project`.
    /// Ordered newest-first by `started_at`. Used by page 11's
    /// "Recent sessions" panel + the M5/P4 `GET /projects/:id/sessions`
    /// endpoint (which strips to `SessionHeader` at the wire tier per
    /// the plan's §P4 schema-snapshot invariant).
    async fn list_sessions_in_project(&self, project: ProjectId) -> RepositoryResult<Vec<Session>>;

    /// List every session with `started_by == agent` and
    /// `governance_state == Running`. Drives the per-agent
    /// parallelize gate at M5/P4 launch time; also underpins the
    /// [`count_active_sessions_for_agent`] flip (the count is just
    /// `.len()` on the list for the in-memory impl).
    async fn list_active_sessions_for_agent(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Vec<Session>>;

    /// Mark a session terminal (Completed / Aborted / FailedLaunch).
    /// Rejects the call when the session is already terminal
    /// (`Conflict` — callers treat as idempotent "already ended").
    /// Atomically sets `ended_at` to the supplied `at` value.
    async fn mark_session_ended(
        &self,
        session: SessionId,
        at: DateTime<Utc>,
        state: SessionGovernanceState,
    ) -> RepositoryResult<()>;

    /// Terminate a session from the page-14 W3 action path. Thin
    /// wrapper around `mark_session_ended(..., Aborted)` + a
    /// governance `reason` string + `terminated_by` principal ref
    /// kept on the audit emit (not on the session row — reason /
    /// actor live in the audit event, not in the durable session
    /// row itself). Impls return `NotFound` if the session is
    /// unknown, `Conflict` if already terminal.
    async fn terminate_session(
        &self,
        session: SessionId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()>;

    /// Write the `UsesModel` edge that binds `agent` to the model
    /// runtime identified by `model_runtime` (as a generic
    /// [`NodeId`] because the edge's `to` slot carries the raw
    /// node id; callers pass `ModelProviderId::as_uuid` wrapped in
    /// `NodeId::from_uuid`).
    ///
    /// M5/P4 first-writer — closes C-M5-2. Migration 0005 retyped
    /// the `uses_model` RELATION to `agent → model_runtime` (from
    /// the legacy `agent → model_config`); this method is the
    /// idiomatic call site for the retype.
    ///
    /// SurrealDB impls MUST use the LET-first RELATE pattern (per
    /// the M5/P2 drift addendum D2.2) — `LET $a = type::thing(...);
    /// LET $m = type::thing(...); RELATE $a -> uses_model -> $m
    /// SET id = type::thing('uses_model', $edge) RETURN NONE`.
    /// Returns the [`EdgeId`] so the caller can thread it through
    /// audit / Launch receipts.
    async fn write_uses_model_edge(
        &self,
        agent: AgentId,
        model_runtime: NodeId,
    ) -> RepositoryResult<EdgeId> {
        let _ = (agent, model_runtime);
        Err(RepositoryError::Backend(
            "write_uses_model_edge not implemented by this Repository impl".into(),
        ))
    }

    /// Persist the Shape B pending-project payload sidecar. Called
    /// from `POST /projects` at submit time, alongside the AR
    /// creation in the same compound tx. Closes C-M5-6 at M5/P4.
    async fn persist_shape_b_pending(&self, row: &ShapeBPendingProject) -> RepositoryResult<()>;

    /// Read the sidecar by `auth_request`. Called by the
    /// `approve_pending_shape_b` Approved branch to reconstruct the
    /// full `CreateProjectInput` for the materialisation compound
    /// tx. Returns [`None`] after the sidecar has been deleted (the
    /// idempotent post-materialise state).
    async fn fetch_shape_b_pending(
        &self,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<Option<ShapeBPendingProject>>;

    /// Delete the sidecar after the Approved branch successfully
    /// materialises the project. No-op on a missing row (sidecar
    /// already deleted → idempotent).
    async fn delete_shape_b_pending(&self, auth_request: AuthRequestId) -> RepositoryResult<()>;

    /// Upsert an [`AgentCatalogEntry`] — the s03 catalogue cache
    /// row. Keyed by `agent_id` (UNIQUE per migration 0005's
    /// `agent_catalog_entry_identity` index). Called from
    /// [`AgentCatalogListener`] at M5/P8 on 8 trigger variants
    /// (AgentCreated / AgentArchived / edge-change events).
    async fn upsert_agent_catalog_entry(&self, entry: &AgentCatalogEntry) -> RepositoryResult<()>;

    /// List every catalogue entry for `org` — the page-07 dashboard
    /// roll-up source + M6 a05 grants-view underpinning. Ordered by
    /// `display_name` for stable operator-facing output.
    async fn list_agent_catalog_entries_in_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AgentCatalogEntry>>;

    /// Get a single catalogue entry by agent id. Returns [`None`]
    /// when the agent has no row yet (pre-first-upsert).
    async fn get_agent_catalog_entry(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentCatalogEntry>>;

    /// Upsert the [`SystemAgentRuntimeStatus`] tile for a system
    /// agent. Called by all 5 listener bodies (Template A / C / D
    /// fire + memory-extraction + agent-catalog) via a shared
    /// helper on every fire.
    async fn upsert_system_agent_runtime_status(
        &self,
        status: &SystemAgentRuntimeStatus,
    ) -> RepositoryResult<()>;

    /// Fetch every runtime-status row for `org` — the page-13
    /// listing endpoint's data source (R-ADMIN-13-R2 / N3). Ordered
    /// by `agent_id` for stable operator-facing output.
    async fn fetch_system_agent_runtime_status_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<SystemAgentRuntimeStatus>>;

    /// List every [`Template`] row for the org's adoption view.
    /// M5/P1 migration 0005 flipped `template` uniqueness from name
    /// to kind (ADR-0030) — Template rows are now platform-level;
    /// per-org adoption lives on the AR's `provenance_template`.
    /// This method therefore returns the **5 platform Template
    /// rows** (A/B/C/D/E) every org sees, filtered or augmented by
    /// adoption state at the server handler tier (M5/P5). The
    /// `org` parameter is kept for symmetry + future evolution but
    /// not used by the read itself at M5/P2.
    async fn list_authority_templates_for_org(&self, org: OrgId)
        -> RepositoryResult<Vec<Template>>;

    /// Count the [`Grant`] rows whose `DESCENDS_FROM` edge points at
    /// `auth_request`. Used by page 12 to surface the
    /// "grants that will be revoked" confirm-dialog number before
    /// the operator runs the revoke-cascade. Pre-cascade read —
    /// counts BOTH active and already-revoked grants for the full
    /// audit trail; the revoke action itself filters to non-terminal
    /// grants at M5/P5.
    async fn count_grants_fired_by_adoption(
        &self,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<u32>;

    /// List every adoption-AR row for `org` whose `state ==
    /// AuthRequestState::Revoked`. Powers page 12's "Revoked"
    /// bucket (R-ADMIN-12-R2). Ordered by `created_at` newest-first.
    async fn list_revoked_adoptions_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>>;
}

/// One cascade hit recorded by [`Repository::narrow_mcp_tenants`] —
/// the org whose access was dropped, the Auth Request whose grants
/// were revoked, and the grant ids that flipped to `revoked`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantRevocation {
    pub org: OrgId,
    pub auth_request: AuthRequestId,
    pub revoked_grants: Vec<GrantId>,
}

// ----------------------------------------------------------------------------
// Typed free-function wrappers — compile-time safe via marker traits.
// ----------------------------------------------------------------------------
//
// These are the entry points callers should normally use. Each delegates
// to the trait's raw method after calling the marker-trait's `node_id()`.
// The generics are on the free function, not on the trait, so
// `Arc<dyn Repository>` remains object-safe.

/// Record that `resource` is owned by `principal`. See ADR-0015.
pub async fn upsert_ownership<R, P>(
    repo: &(dyn Repository + '_),
    resource: &R,
    principal: &P,
    auth_request: Option<AuthRequestId>,
) -> RepositoryResult<EdgeId>
where
    R: Resource + ?Sized,
    P: Principal + ?Sized,
{
    repo.upsert_ownership_raw(resource.node_id(), principal.node_id(), auth_request)
        .await
}

/// Record that `creator` (a Principal) brought `resource` into existence.
pub async fn upsert_creation<P, R>(
    repo: &(dyn Repository + '_),
    creator: &P,
    resource: &R,
) -> RepositoryResult<EdgeId>
where
    P: Principal + ?Sized,
    R: Resource + ?Sized,
{
    repo.upsert_creation_raw(creator.node_id(), resource.node_id())
        .await
}

/// Record that `from` (a Principal) allocated scope of authority over
/// `resource` to `to` (another Principal), with provenance pointing at
/// `auth_request`.
pub async fn upsert_allocation<P1, P2>(
    repo: &(dyn Repository + '_),
    from: &P1,
    to: &P2,
    resource: &ResourceRef,
    auth_request: AuthRequestId,
) -> RepositoryResult<EdgeId>
where
    P1: Principal + ?Sized,
    P2: Principal + ?Sized,
{
    repo.upsert_allocation_raw(from.node_id(), to.node_id(), resource, auth_request)
        .await
}
