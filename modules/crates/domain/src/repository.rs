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
    OrgId, ProjectId, SecretId,
};
use crate::model::nodes::{
    Agent, AgentProfile, AuthRequest, Channel, Consent, Grant, InboxObject, Memory, Organization,
    OutboxObject, PrincipalRef, ResourceRef, Template, ToolAuthorityManifest, User,
};
use crate::model::{
    Composite, ExternalService, ModelRuntime, PlatformDefaults, Principal, Resource,
    SecretCredential, SecretRef, TenantSet,
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

    async fn create_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()>;

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
    /// baby-phi migration.
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

    /// List every Project belonging to `org`. **M3 note**: v0's
    /// `Project` surface is minimal (no dedicated struct yet; M4
    /// fleshes it). Today the method returns `Vec<ProjectId>` so
    /// the M3/P5 dashboard's `ProjectsSummary` panel can render a
    /// count even before projects are persisted. When M4 adds the
    /// Project struct + persistence, the return type will migrate
    /// to `Vec<Project>` in a coordinated change.
    async fn list_projects_in_org(&self, org: OrgId) -> RepositoryResult<Vec<ProjectId>>;

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
