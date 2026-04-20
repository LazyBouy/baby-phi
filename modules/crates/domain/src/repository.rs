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

use crate::audit::AuditEvent;
use crate::model::ids::{AgentId, AuthRequestId, EdgeId, GrantId, NodeId, OrgId};
use crate::model::nodes::{
    Agent, AgentProfile, AuthRequest, Channel, Consent, Grant, InboxObject, Memory, Organization,
    OutboxObject, PrincipalRef, ResourceRef, Template, ToolAuthorityManifest, User,
};
use crate::model::{Principal, Resource};

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

    /// Returns the hash of the most recent event within `org_scope`, or
    /// `None` if no events exist yet for that scope.
    async fn last_event_hash_for_org(
        &self,
        org: Option<OrgId>,
    ) -> RepositoryResult<Option<[u8; 32]>>;
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
