//! In-memory [`Repository`] fake.
//!
//! Only compiled when `cfg(any(test, feature = "in-memory-repo"))` — the
//! release binary gets no-op complexity from carrying it. Tests in other
//! crates enable the feature via `domain = { ..., features = ["in-memory-repo"] }`
//! in their `[dev-dependencies]`.
//!
//! This is deliberately a minimal, HashMap-backed impl: enough to exercise
//! the repository contract in unit/handler/acceptance tests without
//! standing up SurrealDB. P3 and P4 proptests also lean on it.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::audit::AuditEvent;
use crate::model::ids::{
    AgentId, AuditEventId, AuthRequestId, EdgeId, GrantId, LoopId, McpServerId, ModelProviderId,
    NodeId, OrgId, ProjectId, SecretId, SessionId, TurnNodeId,
};
use crate::model::nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, AuthRequest, AuthRequestState, Channel, Consent,
    Grant, InboxObject, LoopRecordNode, Memory, Organization, OutboxObject, PrincipalRef, Project,
    ProjectShape, ResourceRef, Session, SessionGovernanceState, Template, ToolAuthorityManifest,
    TurnNode, User,
};
use crate::model::{
    AgentCatalogEntry, AgentExecutionLimitsOverride, Composite, ExternalService, ModelRuntime,
    PlatformDefaults, SecretCredential, SecretRef, SessionDetail, ShapeBPendingProject,
    SystemAgentRuntimeStatus, TenantSet,
};
use crate::repository::{
    BootstrapCredentialRow, OrgCreationPayload, OrgCreationReceipt, ProjectShapeCounts, Repository,
    RepositoryError, RepositoryResult, SealedBlob, TenantRevocation,
};

#[derive(Default)]
struct State {
    agents: HashMap<AgentId, Agent>,
    agent_profiles: HashMap<NodeId, AgentProfile>,
    users: HashMap<crate::model::ids::UserId, User>,
    organizations: HashMap<OrgId, Organization>,
    templates: HashMap<crate::model::ids::TemplateId, Template>,
    channels: HashMap<NodeId, Channel>,
    inboxes: HashMap<NodeId, InboxObject>,
    outboxes: HashMap<NodeId, OutboxObject>,
    memories: HashMap<crate::model::ids::MemoryId, Memory>,
    consents: HashMap<crate::model::ids::ConsentId, Consent>,
    manifests: HashMap<NodeId, ToolAuthorityManifest>,
    grants: HashMap<GrantId, Grant>,
    auth_requests: HashMap<AuthRequestId, AuthRequest>,
    ownership_edges: Vec<OwnershipEdge>,
    creation_edges: Vec<CreationEdge>,
    allocation_edges: Vec<AllocationEdge>,
    bootstrap_credentials: Vec<BootstrapCredentialRow>,
    catalogue: Vec<(Option<OrgId>, String, String)>, // (owning_org, uri, kind)
    audit_events: Vec<AuditEvent>,
    // M2 additions (P2 — one HashMap per new composite-instance table).
    secrets: HashMap<SecretId, (SecretCredential, SealedBlob)>,
    model_providers: HashMap<ModelProviderId, ModelRuntime>,
    mcp_servers: HashMap<McpServerId, ExternalService>,
    platform_defaults: Option<PlatformDefaults>,
    // M3/P3 additions.
    token_budget_pools: HashMap<NodeId, crate::model::composites_m3::TokenBudgetPool>,
    // M4/P1 additions — Project node + Project lead edges + per-agent
    // ExecutionLimits override rows. `has_lead_edges` gains production
    // writers at M4/P3's `apply_project_creation`; until then the vec
    // stays empty and `list_projects_led_by_agent` returns empty.
    projects: HashMap<ProjectId, Project>,
    /// `HAS_LEAD` edges (project → lead agent). Kept as a (project,
    /// lead) Vec rather than a HashMap because an agent may lead
    /// multiple projects.
    has_lead_edges: Vec<(ProjectId, AgentId)>,
    /// `BELONGS_TO` edges (project → owning org). Shape A projects
    /// have exactly one entry; Shape B projects have two entries
    /// (one per co-owner).
    project_belongs_to_edges: Vec<(ProjectId, OrgId)>,
    /// Per-agent `ExecutionLimits` override rows, keyed by owning
    /// agent (1:1 per migration 0004's UNIQUE index).
    agent_execution_limits: HashMap<AgentId, AgentExecutionLimitsOverride>,
    // M5/P2 additions — Session tier + sidecar + catalog + runtime status.
    /// Governance `Session` rows keyed by session id.
    sessions: HashMap<SessionId, Session>,
    /// Governance `LoopRecordNode` rows keyed by loop id.
    loop_records: HashMap<LoopId, LoopRecordNode>,
    /// Governance `TurnNode` rows keyed by turn node id.
    turn_nodes: HashMap<TurnNodeId, TurnNode>,
    /// `runs_session` edges (session → project). Allows
    /// [`list_sessions_in_project`] to walk the edge set.
    runs_session_edges: Vec<(SessionId, ProjectId)>,
    /// Materialised `phi_core::AgentEvent` stream per session.
    /// Stored as serde `Value` on the repo boundary to keep
    /// phi-core types from crossing the trait (Q3 rejection —
    /// preserves the repo's phi-core-free trait shape).
    session_agent_events: HashMap<SessionId, Vec<serde_json::Value>>,
    /// Shape B pending-project sidecar rows, keyed by the AR id.
    /// UNIQUE per migration 0005's `shape_b_pending_projects_ar`
    /// index — enforced here via the HashMap key.
    shape_b_pending_projects: HashMap<AuthRequestId, ShapeBPendingProject>,
    /// Agent catalogue cache rows — s03 output. UNIQUE per
    /// `agent_id` per migration 0005.
    agent_catalog_entries: HashMap<AgentId, AgentCatalogEntry>,
    /// System-agent runtime-status tiles (page 13 live feed).
    /// UNIQUE per `agent_id` per migration 0005.
    system_agent_runtime_status: HashMap<AgentId, SystemAgentRuntimeStatus>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct OwnershipEdge {
    id: EdgeId,
    resource: NodeId,
    owner: NodeId,
    auth_request: Option<AuthRequestId>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct CreationEdge {
    id: EdgeId,
    creator: NodeId,
    resource: NodeId,
}

#[derive(Clone)]
#[allow(dead_code)]
struct AllocationEdge {
    id: EdgeId,
    from: NodeId,
    to: NodeId,
    resource: ResourceRef,
    auth_request: AuthRequestId,
}

/// In-memory `Repository`. Cheap to construct; `Clone` via the inner
/// `Arc<Mutex<...>>` is cheap.
///
/// The `unhealthy` toggle is a test-only knob that makes [`Repository::ping`]
/// return `Err` — useful for exercising `/healthz/ready` failure paths
/// without standing up a broken database.
pub struct InMemoryRepository {
    state: std::sync::Arc<Mutex<State>>,
    unhealthy: std::sync::atomic::AtomicBool,
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self {
            state: std::sync::Arc::new(Mutex::new(State::default())),
            unhealthy: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test knob: make `ping()` return `Err` on subsequent calls. Used by
    /// server health-endpoint tests.
    pub fn set_unhealthy(&self, unhealthy: bool) {
        self.unhealthy
            .store(unhealthy, std::sync::atomic::Ordering::SeqCst);
    }

    fn lock(&self) -> RepositoryResult<std::sync::MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|e| RepositoryError::Backend(format!("in-memory lock poisoned: {e}")))
    }

    // ---- M4/P2 test-only seed helpers ---------------------------------
    //
    // The M4/P2 Repository trait ships only READ methods for Project +
    // the `HAS_LEAD` / `BELONGS_TO` edges. Write surfaces land at M4/P3
    // via the `apply_project_creation` compound tx — which doesn't
    // exist yet. These helpers let unit tests exercise the populated
    // branches of the new readers without waiting for P3.
    //
    // Gated on `#[cfg(any(test, feature = "in-memory-repo"))]` already
    // via the module, so the release binary gets none of this.

    /// Seed a Project row + its `BELONGS_TO` edges into the in-memory
    /// state. `owning_orgs` is a single-org slice for Shape A and a
    /// two-org slice for Shape B.
    pub fn test_seed_project(&self, project: Project, owning_orgs: &[OrgId]) {
        let mut state = self
            .state
            .lock()
            .expect("in-memory lock poisoned in test helper");
        let pid = project.id;
        state.projects.insert(pid, project);
        for org in owning_orgs {
            state.project_belongs_to_edges.push((pid, *org));
        }
    }

    /// Seed a `HAS_LEAD` edge from project to lead agent. Appends
    /// rather than replacing — callers that model "lead changes"
    /// should clear the existing edge first.
    pub fn test_seed_project_lead(&self, project: ProjectId, lead: AgentId) {
        let mut state = self
            .state
            .lock()
            .expect("in-memory lock poisoned in test helper");
        state.has_lead_edges.push((project, lead));
    }
}

#[async_trait]
impl Repository for InMemoryRepository {
    async fn ping(&self) -> RepositoryResult<()> {
        if self.unhealthy.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(RepositoryError::Backend(
                "in-memory repo: simulated unhealthy".into(),
            ));
        }
        // Take the lock just long enough to prove the mutex is healthy;
        // matches the SurrealDB `health()` probe semantics. Scoped so
        // Clippy doesn't flag the `_` binding as held-too-long.
        {
            let _guard = self.lock()?;
        }
        Ok(())
    }

    // ---- Node CRUD ----

    async fn create_agent(&self, agent: &Agent) -> RepositoryResult<()> {
        self.lock()?.agents.insert(agent.id, agent.clone());
        Ok(())
    }

    async fn get_agent(&self, id: AgentId) -> RepositoryResult<Option<Agent>> {
        Ok(self.lock()?.agents.get(&id).cloned())
    }

    async fn upsert_agent(&self, agent: &Agent) -> RepositoryResult<()> {
        self.lock()?.agents.insert(agent.id, agent.clone());
        Ok(())
    }

    async fn create_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()> {
        self.lock()?
            .agent_profiles
            .insert(profile.id, profile.clone());
        Ok(())
    }

    async fn get_agent_profile_for_agent(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentProfile>> {
        Ok(self
            .lock()?
            .agent_profiles
            .values()
            .find(|p| p.agent_id == agent)
            .cloned())
    }

    async fn upsert_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()> {
        let mut g = self.lock()?;
        // Remove any existing profile for the same agent (1:1 invariant)
        // before inserting, so we don't accumulate duplicates when the
        // caller passes a fresh `id` for an edit.
        let existing: Vec<_> = g
            .agent_profiles
            .iter()
            .filter_map(|(k, v)| {
                if v.agent_id == profile.agent_id && *k != profile.id {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect();
        for k in existing {
            g.agent_profiles.remove(&k);
        }
        g.agent_profiles.insert(profile.id, profile.clone());
        Ok(())
    }

    async fn create_user(&self, user: &User) -> RepositoryResult<()> {
        self.lock()?.users.insert(user.id, user.clone());
        Ok(())
    }

    async fn create_organization(&self, org: &Organization) -> RepositoryResult<()> {
        self.lock()?.organizations.insert(org.id, org.clone());
        Ok(())
    }

    async fn get_organization(&self, id: OrgId) -> RepositoryResult<Option<Organization>> {
        Ok(self.lock()?.organizations.get(&id).cloned())
    }

    async fn create_template(&self, template: &Template) -> RepositoryResult<()> {
        self.lock()?.templates.insert(template.id, template.clone());
        Ok(())
    }

    async fn create_channel(&self, channel: &Channel) -> RepositoryResult<()> {
        self.lock()?.channels.insert(channel.id, channel.clone());
        Ok(())
    }

    async fn create_inbox(&self, inbox: &InboxObject) -> RepositoryResult<()> {
        self.lock()?.inboxes.insert(inbox.id, inbox.clone());
        Ok(())
    }

    async fn create_outbox(&self, outbox: &OutboxObject) -> RepositoryResult<()> {
        self.lock()?.outboxes.insert(outbox.id, outbox.clone());
        Ok(())
    }

    async fn create_memory(&self, memory: &Memory) -> RepositoryResult<()> {
        self.lock()?.memories.insert(memory.id, memory.clone());
        Ok(())
    }

    async fn create_consent(&self, consent: &Consent) -> RepositoryResult<()> {
        self.lock()?.consents.insert(consent.id, consent.clone());
        Ok(())
    }

    async fn create_tool_authority_manifest(
        &self,
        manifest: &ToolAuthorityManifest,
    ) -> RepositoryResult<()> {
        self.lock()?.manifests.insert(manifest.id, manifest.clone());
        Ok(())
    }

    async fn get_admin_agent(&self) -> RepositoryResult<Option<Agent>> {
        Ok(self
            .lock()?
            .agents
            .values()
            .find(|a| a.kind == AgentKind::Human)
            .cloned())
    }

    // ---- Grants ----

    async fn create_grant(&self, grant: &Grant) -> RepositoryResult<()> {
        self.lock()?.grants.insert(grant.id, grant.clone());
        Ok(())
    }

    async fn get_grant(&self, id: GrantId) -> RepositoryResult<Option<Grant>> {
        Ok(self.lock()?.grants.get(&id).cloned())
    }

    async fn revoke_grant(&self, id: GrantId, revoked_at: DateTime<Utc>) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.grants.get_mut(&id) {
            Some(g) => {
                g.revoked_at = Some(revoked_at);
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn list_grants_for_principal(
        &self,
        principal: &PrincipalRef,
    ) -> RepositoryResult<Vec<Grant>> {
        let state = self.lock()?;
        Ok(state
            .grants
            .values()
            .filter(|g| principals_equal(&g.holder, principal))
            .cloned()
            .collect())
    }

    // ---- Auth Requests ----

    async fn create_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()> {
        self.lock()?.auth_requests.insert(req.id, req.clone());
        Ok(())
    }

    async fn get_auth_request(&self, id: AuthRequestId) -> RepositoryResult<Option<AuthRequest>> {
        Ok(self.lock()?.auth_requests.get(&id).cloned())
    }

    async fn update_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        if !state.auth_requests.contains_key(&req.id) {
            return Err(RepositoryError::NotFound);
        }
        state.auth_requests.insert(req.id, req.clone());
        Ok(())
    }

    async fn list_active_auth_requests_for_resource(
        &self,
        resource: &ResourceRef,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        let state = self.lock()?;
        Ok(state
            .auth_requests
            .values()
            .filter(|r| {
                !r.archived
                    && r.resource_slots
                        .iter()
                        .any(|s| s.resource.uri == resource.uri)
            })
            .cloned()
            .collect())
    }

    // ---- Ownership edges (raw) ----

    async fn upsert_ownership_raw(
        &self,
        resource_id: NodeId,
        owner_id: NodeId,
        auth_request: Option<AuthRequestId>,
    ) -> RepositoryResult<EdgeId> {
        let id = EdgeId::new();
        self.lock()?.ownership_edges.push(OwnershipEdge {
            id,
            resource: resource_id,
            owner: owner_id,
            auth_request,
        });
        Ok(id)
    }

    async fn upsert_creation_raw(
        &self,
        creator_id: NodeId,
        resource_id: NodeId,
    ) -> RepositoryResult<EdgeId> {
        let id = EdgeId::new();
        self.lock()?.creation_edges.push(CreationEdge {
            id,
            creator: creator_id,
            resource: resource_id,
        });
        Ok(id)
    }

    async fn upsert_allocation_raw(
        &self,
        from_id: NodeId,
        to_id: NodeId,
        resource: &ResourceRef,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<EdgeId> {
        let id = EdgeId::new();
        self.lock()?.allocation_edges.push(AllocationEdge {
            id,
            from: from_id,
            to: to_id,
            resource: resource.clone(),
            auth_request,
        });
        Ok(id)
    }

    // ---- Bootstrap credentials ----

    async fn put_bootstrap_credential(
        &self,
        digest: String,
    ) -> RepositoryResult<BootstrapCredentialRow> {
        let row = BootstrapCredentialRow {
            record_id: format!("mem:{}", uuid::Uuid::new_v4()),
            digest,
            created_at: Utc::now(),
            consumed_at: None,
        };
        self.lock()?.bootstrap_credentials.push(row.clone());
        Ok(row)
    }

    async fn find_unconsumed_credential(
        &self,
        digest: &str,
    ) -> RepositoryResult<Option<BootstrapCredentialRow>> {
        Ok(self
            .lock()?
            .bootstrap_credentials
            .iter()
            .find(|r| r.digest == digest && r.consumed_at.is_none())
            .cloned())
    }

    async fn consume_bootstrap_credential(&self, record_id: &str) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        if let Some(row) = state
            .bootstrap_credentials
            .iter_mut()
            .find(|r| r.record_id == record_id)
        {
            row.consumed_at = Some(Utc::now());
        }
        // Mirrors SurrealStore's UPDATE-no-op-on-missing semantics. The
        // repository integration test `consume_missing_credential_is_noop`
        // pins this contract.
        Ok(())
    }

    async fn list_bootstrap_credentials(
        &self,
        unconsumed_only: bool,
    ) -> RepositoryResult<Vec<BootstrapCredentialRow>> {
        let state = self.lock()?;
        Ok(state
            .bootstrap_credentials
            .iter()
            .filter(|r| !unconsumed_only || r.consumed_at.is_none())
            .cloned()
            .collect())
    }

    // ---- Resources Catalogue ----

    async fn seed_catalogue_entry(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        kind: &str,
    ) -> RepositoryResult<()> {
        self.lock()?
            .catalogue
            .push((owning_org, resource_uri.to_string(), kind.to_string()));
        Ok(())
    }

    async fn catalogue_contains(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
    ) -> RepositoryResult<bool> {
        Ok(self
            .lock()?
            .catalogue
            .iter()
            .any(|(org, uri, _)| *org == owning_org && uri == resource_uri))
    }

    // ---- Audit ----

    async fn write_audit_event(&self, event: &AuditEvent) -> RepositoryResult<()> {
        self.lock()?.audit_events.push(event.clone());
        Ok(())
    }

    async fn get_audit_event(&self, id: AuditEventId) -> RepositoryResult<Option<AuditEvent>> {
        Ok(self
            .lock()?
            .audit_events
            .iter()
            .find(|e| e.event_id == id)
            .cloned())
    }

    async fn last_event_hash_for_org(
        &self,
        org: Option<OrgId>,
    ) -> RepositoryResult<Option<[u8; 32]>> {
        // Hash the LAST event's content (canonical bytes exclude
        // `prev_event_hash` itself). The next event within this org
        // copies the returned digest into its `prev_event_hash` field,
        // forming the chain:
        //
        //   event_n.prev_event_hash = hash_event(event_{n-1})
        //
        // Returning `event.prev_event_hash` instead would propagate the
        // second-to-last event's hash, not the last — that's not a chain.
        Ok(self
            .lock()?
            .audit_events
            .iter()
            .rev()
            .find(|e| e.org_scope == org)
            .map(crate::audit::hash_event))
    }

    async fn apply_bootstrap_claim(
        &self,
        claim: &crate::repository::BootstrapClaim,
    ) -> RepositoryResult<()> {
        // The single write-lock gives us pseudo-atomicity: if the guard
        // returns mid-batch (panic / error), the other thread sees a
        // poisoned mutex. For the happy path we apply every write; any
        // validation error is surfaced before mutation begins.
        let mut state = self.lock()?;

        // Pre-flight validation so partial state doesn't survive:
        // credential must still exist and be unconsumed.
        let cred = state
            .bootstrap_credentials
            .iter_mut()
            .find(|c| c.record_id == claim.credential_record_id)
            .ok_or(RepositoryError::NotFound)?;
        if cred.consumed_at.is_some() {
            return Err(RepositoryError::Conflict(
                "bootstrap credential already consumed".into(),
            ));
        }

        // Writes in order matching the SurrealStore transaction.
        state
            .agents
            .insert(claim.human_agent.id, claim.human_agent.clone());
        state
            .channels
            .insert(claim.channel.id, claim.channel.clone());
        state.inboxes.insert(claim.inbox.id, claim.inbox.clone());
        state.outboxes.insert(claim.outbox.id, claim.outbox.clone());
        state
            .auth_requests
            .insert(claim.auth_request.id, claim.auth_request.clone());
        state.grants.insert(claim.grant.id, claim.grant.clone());
        state.audit_events.push(claim.audit_event.clone());
        for (uri, kind) in &claim.catalogue_entries {
            state.catalogue.push((None, uri.clone(), kind.clone()));
        }

        // Mark credential consumed LAST — this is the moment the claim
        // becomes irreversible from the caller's perspective.
        let cred = state
            .bootstrap_credentials
            .iter_mut()
            .find(|c| c.record_id == claim.credential_record_id)
            .expect("credential located above");
        cred.consumed_at = Some(chrono::Utc::now());
        Ok(())
    }

    // ================================================================
    // M2 additions — match the new trait methods in the same order as
    // `repository.rs`.
    // ================================================================

    // ---- Secrets vault ---------------------------------------------

    async fn put_secret(
        &self,
        credential: &SecretCredential,
        sealed: &SealedBlob,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        // Slug uniqueness — mirrors `secrets_vault_slug` UNIQUE INDEX.
        if state
            .secrets
            .values()
            .any(|(c, _)| c.slug == credential.slug)
        {
            return Err(RepositoryError::Conflict(format!(
                "vault slug already in use: {}",
                credential.slug
            )));
        }
        state
            .secrets
            .insert(credential.id, (credential.clone(), sealed.clone()));
        Ok(())
    }

    async fn get_secret_by_slug(
        &self,
        slug: &SecretRef,
    ) -> RepositoryResult<Option<(SecretCredential, SealedBlob)>> {
        Ok(self
            .lock()?
            .secrets
            .values()
            .find(|(c, _)| c.slug == *slug)
            .cloned())
    }

    async fn list_secrets(&self) -> RepositoryResult<Vec<SecretCredential>> {
        Ok(self
            .lock()?
            .secrets
            .values()
            .map(|(c, _)| c.clone())
            .collect())
    }

    async fn rotate_secret(
        &self,
        id: SecretId,
        new_sealed: &SealedBlob,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.secrets.get_mut(&id) {
            Some((cred, sealed)) => {
                cred.last_rotated_at = Some(at);
                *sealed = new_sealed.clone();
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn reassign_secret_custodian(
        &self,
        id: SecretId,
        new_custodian: AgentId,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.secrets.get_mut(&id) {
            Some((cred, _)) => {
                cred.custodian = new_custodian;
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    // ---- Model providers -------------------------------------------

    async fn put_model_provider(&self, provider: &ModelRuntime) -> RepositoryResult<()> {
        self.lock()?
            .model_providers
            .insert(provider.id, provider.clone());
        Ok(())
    }

    async fn list_model_providers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ModelRuntime>> {
        Ok(self
            .lock()?
            .model_providers
            .values()
            .filter(|p| include_archived || p.archived_at.is_none())
            .cloned()
            .collect())
    }

    async fn archive_model_provider(
        &self,
        id: ModelProviderId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.model_providers.get_mut(&id) {
            Some(p) => {
                p.archived_at = Some(at);
                p.status = crate::model::RuntimeStatus::Archived;
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    // ---- MCP servers -----------------------------------------------

    async fn put_mcp_server(&self, server: &ExternalService) -> RepositoryResult<()> {
        self.lock()?.mcp_servers.insert(server.id, server.clone());
        Ok(())
    }

    async fn list_mcp_servers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ExternalService>> {
        Ok(self
            .lock()?
            .mcp_servers
            .values()
            .filter(|s| include_archived || s.archived_at.is_none())
            .cloned()
            .collect())
    }

    async fn patch_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.mcp_servers.get_mut(&id) {
            Some(s) => {
                s.tenants_allowed = new_allowed.clone();
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn archive_mcp_server(&self, id: McpServerId, at: DateTime<Utc>) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        match state.mcp_servers.get_mut(&id) {
            Some(s) => {
                s.archived_at = Some(at);
                s.status = crate::model::RuntimeStatus::Archived;
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
    }

    // ---- Platform defaults -----------------------------------------

    async fn get_platform_defaults(&self) -> RepositoryResult<Option<PlatformDefaults>> {
        Ok(self.lock()?.platform_defaults.clone())
    }

    async fn put_platform_defaults(&self, defaults: &PlatformDefaults) -> RepositoryResult<()> {
        self.lock()?.platform_defaults = Some(defaults.clone());
        Ok(())
    }

    // ---- Cascade ---------------------------------------------------

    async fn narrow_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TenantRevocation>> {
        let mut state = self.lock()?;

        // 1. Compute the dropped-orgs diff against the current set.
        let server = state
            .mcp_servers
            .get_mut(&id)
            .ok_or(RepositoryError::NotFound)?;
        let dropped = dropped_orgs(&server.tenants_allowed, new_allowed);
        if dropped.is_empty() {
            // No shrink — treat as a no-op patch so callers that
            // mis-route stay correct.
            server.tenants_allowed = new_allowed.clone();
            return Ok(vec![]);
        }
        server.tenants_allowed = new_allowed.clone();

        // 2. For each dropped org, find every live Auth Request whose
        //    requestor was that org and revoke every grant
        //    descending from it.
        let mut out: Vec<TenantRevocation> = Vec::new();
        for org in dropped {
            // Collect candidate Auth Requests (the requestor was this
            // org). Cloned so we can mutate `state.grants` below
            // without aliasing the iterator.
            let ars: Vec<AuthRequestId> = state
                .auth_requests
                .values()
                .filter(|ar| matches!(ar.requestor, PrincipalRef::Organization(o) if o == org))
                .map(|ar| ar.id)
                .collect();
            for ar in ars {
                let mut revoked: Vec<GrantId> = Vec::new();
                for g in state.grants.values_mut() {
                    if g.descends_from == Some(ar) && g.revoked_at.is_none() {
                        g.revoked_at = Some(at);
                        revoked.push(g.id);
                    }
                }
                if !revoked.is_empty() {
                    out.push(TenantRevocation {
                        org,
                        auth_request: ar,
                        revoked_grants: revoked,
                    });
                }
            }
        }
        Ok(out)
    }

    async fn revoke_grants_by_descends_from(
        &self,
        ar: AuthRequestId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<GrantId>> {
        let mut state = self.lock()?;
        let mut out: Vec<GrantId> = Vec::new();
        for g in state.grants.values_mut() {
            if g.descends_from == Some(ar) && g.revoked_at.is_none() {
                g.revoked_at = Some(at);
                out.push(g.id);
            }
        }
        Ok(out)
    }

    // ---- Catalogue -------------------------------------------------

    async fn seed_catalogue_entry_for_composite(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        composite: Composite,
    ) -> RepositoryResult<()> {
        self.seed_catalogue_entry(owning_org, resource_uri, composite.kind_name())
            .await
    }

    // ---- M3 org-scoped reads --------------------------------------

    async fn list_agents_in_org(&self, org: OrgId) -> RepositoryResult<Vec<Agent>> {
        let state = self.lock()?;
        Ok(state
            .agents
            .values()
            .filter(|a| a.owning_org == Some(org))
            .cloned()
            .collect())
    }

    async fn list_all_orgs(&self) -> RepositoryResult<Vec<Organization>> {
        let state = self.lock()?;
        Ok(state.organizations.values().cloned().collect())
    }

    async fn list_projects_in_org(&self, org: OrgId) -> RepositoryResult<Vec<Project>> {
        // M4/P1 fleshes the Project struct; M4/P3's compound tx adds
        // production writers. Until then `state.projects` stays empty
        // and this method returns `vec![]` — which matches the
        // pre-M4 behaviour the dashboard already handles.
        let state = self.lock()?;
        let project_ids: std::collections::HashSet<ProjectId> = state
            .project_belongs_to_edges
            .iter()
            .filter(|(_, owning)| *owning == org)
            .map(|(pid, _)| *pid)
            .collect();
        Ok(state
            .projects
            .values()
            .filter(|p| project_ids.contains(&p.id))
            .cloned()
            .collect())
    }

    // ---- M4/P2 surface --------------------------------------------

    async fn list_agents_in_org_by_role(
        &self,
        org: OrgId,
        role: Option<AgentRole>,
    ) -> RepositoryResult<Vec<Agent>> {
        let state = self.lock()?;
        Ok(state
            .agents
            .values()
            .filter(|a| a.owning_org == Some(org))
            .filter(|a| match role {
                None => true,
                Some(target) => a.role == Some(target),
            })
            .cloned()
            .collect())
    }

    async fn get_project(&self, id: ProjectId) -> RepositoryResult<Option<Project>> {
        let state = self.lock()?;
        Ok(state.projects.get(&id).cloned())
    }

    async fn list_projects_by_shape_in_org(
        &self,
        org: OrgId,
        shape: ProjectShape,
    ) -> RepositoryResult<Vec<Project>> {
        let state = self.lock()?;
        let project_ids: std::collections::HashSet<ProjectId> = state
            .project_belongs_to_edges
            .iter()
            .filter(|(_, owning)| *owning == org)
            .map(|(pid, _)| *pid)
            .collect();
        Ok(state
            .projects
            .values()
            .filter(|p| project_ids.contains(&p.id) && p.shape == shape)
            .cloned()
            .collect())
    }

    async fn count_projects_by_shape_in_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<ProjectShapeCounts> {
        let state = self.lock()?;
        let project_ids: std::collections::HashSet<ProjectId> = state
            .project_belongs_to_edges
            .iter()
            .filter(|(_, owning)| *owning == org)
            .map(|(pid, _)| *pid)
            .collect();
        let mut counts = ProjectShapeCounts::default();
        for p in state.projects.values() {
            if !project_ids.contains(&p.id) {
                continue;
            }
            match p.shape {
                ProjectShape::A => counts.shape_a = counts.shape_a.saturating_add(1),
                ProjectShape::B => counts.shape_b = counts.shape_b.saturating_add(1),
            }
        }
        Ok(counts)
    }

    async fn list_projects_led_by_agent(&self, agent: AgentId) -> RepositoryResult<Vec<Project>> {
        let state = self.lock()?;
        let project_ids: std::collections::HashSet<ProjectId> = state
            .has_lead_edges
            .iter()
            .filter(|(_, lead)| *lead == agent)
            .map(|(pid, _)| *pid)
            .collect();
        Ok(state
            .projects
            .values()
            .filter(|p| project_ids.contains(&p.id))
            .cloned()
            .collect())
    }

    async fn upsert_project(&self, project: &Project) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        state.projects.insert(project.id, project.clone());
        Ok(())
    }

    async fn get_agent_execution_limits_override(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentExecutionLimitsOverride>> {
        let state = self.lock()?;
        Ok(state.agent_execution_limits.get(&agent).cloned())
    }

    async fn set_agent_execution_limits_override(
        &self,
        row: &AgentExecutionLimitsOverride,
    ) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        // The domain-layer invariant (`≤ org snapshot`) is NOT enforced
        // here — that belongs to the handler so violations surface as a
        // stable 400 code rather than a backend error. The UNIQUE index
        // at storage is simulated by HashMap overwrite semantics.
        state
            .agent_execution_limits
            .insert(row.owning_agent, row.clone());
        Ok(())
    }

    async fn clear_agent_execution_limits_override(&self, agent: AgentId) -> RepositoryResult<()> {
        let mut state = self.lock()?;
        state.agent_execution_limits.remove(&agent);
        Ok(())
    }

    async fn resolve_effective_execution_limits(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<phi_core::context::execution::ExecutionLimits>> {
        let state = self.lock()?;
        if let Some(row) = state.agent_execution_limits.get(&agent) {
            return Ok(Some(row.limits.clone()));
        }
        let Some(a) = state.agents.get(&agent) else {
            return Ok(None);
        };
        let Some(org_id) = a.owning_org else {
            return Ok(None);
        };
        let Some(org) = state.organizations.get(&org_id) else {
            return Ok(None);
        };
        Ok(org
            .defaults_snapshot
            .as_ref()
            .map(|snap| snap.execution_limits.clone()))
    }

    async fn list_active_auth_requests_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        let state = self.lock()?;
        // Non-terminal states = not-yet-decided. The 9-state machine's
        // terminal set is {Approved, Denied, Partial, Expired,
        // Withdrawn, Escalated, Archived}; Draft / Pending /
        // InProgress are the dashboard-visible active states.
        use crate::model::nodes::AuthRequestState as S;
        let agents_in_org: std::collections::HashSet<AgentId> = state
            .agents
            .values()
            .filter(|a| a.owning_org == Some(org))
            .map(|a| a.id)
            .collect();
        Ok(state
            .auth_requests
            .values()
            .filter(|r| {
                !r.archived
                    && matches!(r.state, S::Draft | S::Pending | S::InProgress)
                    && match &r.requestor {
                        PrincipalRef::Organization(o) => *o == org,
                        PrincipalRef::Agent(a) => agents_in_org.contains(a),
                        _ => false,
                    }
            })
            .cloned()
            .collect())
    }

    async fn list_recent_audit_events_for_org(
        &self,
        org: OrgId,
        limit: usize,
    ) -> RepositoryResult<Vec<AuditEvent>> {
        let state = self.lock()?;
        let mut matching: Vec<AuditEvent> = state
            .audit_events
            .iter()
            .filter(|e| e.org_scope == Some(org))
            .cloned()
            .collect();
        // Newest-first — matches the SurrealDB `ORDER BY timestamp
        // DESC` in the sibling implementation.
        matching.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        matching.truncate(limit);
        Ok(matching)
    }

    async fn list_adoption_auth_requests_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        let state = self.lock()?;
        // Match by URI prefix — adoption ARs carry
        // `org:<id>/template:<kind>` as their resource URI (see
        // `domain::templates::adoption::build_adoption_request`).
        let prefix = format!("org:{}/template:", org);
        Ok(state
            .auth_requests
            .values()
            .filter(|r| {
                r.resource_slots
                    .iter()
                    .any(|s| s.resource.uri.starts_with(&prefix))
            })
            .cloned()
            .collect())
    }

    async fn get_token_budget_pool_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Option<crate::model::composites_m3::TokenBudgetPool>> {
        let state = self.lock()?;
        Ok(state
            .token_budget_pools
            .values()
            .find(|p| p.owning_org == org)
            .cloned())
    }

    async fn count_alerted_events_for_org_since(
        &self,
        org: OrgId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryResult<u32> {
        let state = self.lock()?;
        let n = state
            .audit_events
            .iter()
            .filter(|e| {
                e.org_scope == Some(org)
                    && matches!(e.audit_class, crate::audit::AuditClass::Alerted)
                    && e.timestamp >= since
            })
            .count();
        Ok(n as u32)
    }

    async fn apply_org_creation(
        &self,
        payload: &OrgCreationPayload,
    ) -> RepositoryResult<OrgCreationReceipt> {
        // Pseudo-atomicity via the single write-lock (same pattern as
        // `apply_bootstrap_claim`). Pre-flight validates every
        // invariant before mutating, so a rejection leaves zero
        // partial state.
        let mut state = self.lock()?;

        let org_id = payload.organization.id;

        // ---- pre-flight validation ------------------------------
        if state.organizations.contains_key(&org_id) {
            return Err(RepositoryError::Conflict(format!(
                "organization already exists: {org_id}"
            )));
        }
        if state
            .token_budget_pools
            .values()
            .any(|p| p.owning_org == org_id)
        {
            return Err(RepositoryError::Conflict(format!(
                "token_budget_pool already exists for org: {org_id}"
            )));
        }
        if payload.ceo_agent.owning_org != Some(org_id) {
            return Err(RepositoryError::InvalidArgument(
                "ceo_agent.owning_org must match organization.id".into(),
            ));
        }
        for (agent, _profile) in &payload.system_agents {
            if agent.owning_org != Some(org_id) {
                return Err(RepositoryError::InvalidArgument(
                    "system_agent.owning_org must match organization.id".into(),
                ));
            }
        }
        if payload.token_budget_pool.owning_org != org_id {
            return Err(RepositoryError::InvalidArgument(
                "token_budget_pool.owning_org must match organization.id".into(),
            ));
        }

        // ---- commit --------------------------------------------
        state
            .organizations
            .insert(org_id, payload.organization.clone());
        state
            .agents
            .insert(payload.ceo_agent.id, payload.ceo_agent.clone());
        state
            .channels
            .insert(payload.ceo_channel.id, payload.ceo_channel.clone());
        state
            .inboxes
            .insert(payload.ceo_inbox.id, payload.ceo_inbox.clone());
        state
            .outboxes
            .insert(payload.ceo_outbox.id, payload.ceo_outbox.clone());
        state
            .grants
            .insert(payload.ceo_grant.id, payload.ceo_grant.clone());

        let mut system_agent_ids = [payload.system_agents[0].0.id, payload.system_agents[1].0.id];
        let mut system_agent_profile_ids =
            [payload.system_agents[0].1.id, payload.system_agents[1].1.id];
        // `Vec<Agent>` iteration order is unspecified but `[T; 2]` is
        // stable — we emit ids in the order the caller supplied to
        // keep `OrgCreationReceipt.system_agent_ids` deterministic.
        for (agent, profile) in &payload.system_agents {
            state.agents.insert(agent.id, agent.clone());
            state.agent_profiles.insert(profile.id, profile.clone());
        }
        // Defensive: we assigned the pair from indices 0/1, so no-op
        // here; kept to document the stable-order invariant.
        system_agent_ids.sort_by_key(|id| {
            payload
                .system_agents
                .iter()
                .position(|(a, _)| a.id == *id)
                .expect("id came from payload")
        });
        system_agent_profile_ids.sort_by_key(|id| {
            payload
                .system_agents
                .iter()
                .position(|(_, p)| p.id == *id)
                .expect("id came from payload")
        });

        state.token_budget_pools.insert(
            payload.token_budget_pool.id,
            payload.token_budget_pool.clone(),
        );

        let mut adoption_ar_ids = Vec::with_capacity(payload.adoption_auth_requests.len());
        for ar in &payload.adoption_auth_requests {
            state.auth_requests.insert(ar.id, ar.clone());
            adoption_ar_ids.push(ar.id);
        }

        for (uri, kind) in &payload.catalogue_entries {
            state
                .catalogue
                .push((Some(org_id), uri.clone(), kind.clone()));
        }

        // ADR-0023 invariant check (defensive): no per-agent
        // ExecutionLimits / RetryPolicy / CachePolicy / CompactionPolicy
        // nodes are materialised here. The in-memory backend has no
        // HashMap slots for those at all, which enforces the invariant
        // structurally. This comment documents the intent for future
        // readers / bug-fixers.

        Ok(OrgCreationReceipt {
            org_id,
            ceo_agent_id: payload.ceo_agent.id,
            ceo_channel_id: payload.ceo_channel.id,
            ceo_inbox_id: payload.ceo_inbox.id,
            ceo_outbox_id: payload.ceo_outbox.id,
            ceo_grant_id: payload.ceo_grant.id,
            system_agent_ids,
            system_agent_profile_ids,
            token_budget_pool_id: payload.token_budget_pool.id,
            adoption_auth_request_ids: adoption_ar_ids,
        })
    }

    async fn apply_project_creation(
        &self,
        payload: &crate::repository::ProjectCreationPayload,
    ) -> RepositoryResult<crate::repository::ProjectCreationReceipt> {
        let mut state = self.lock()?;
        let pid = payload.project.id;

        // ---- pre-flight validation ------------------------------
        if state.projects.contains_key(&pid) {
            return Err(RepositoryError::Conflict(format!(
                "project already exists: {pid}"
            )));
        }
        use crate::model::nodes::ProjectShape;
        match payload.project.shape {
            ProjectShape::A => {
                if payload.owning_orgs.len() != 1 {
                    return Err(RepositoryError::InvalidArgument(
                        "shape_a requires exactly 1 owning org".into(),
                    ));
                }
            }
            ProjectShape::B => {
                if payload.owning_orgs.len() != 2 {
                    return Err(RepositoryError::InvalidArgument(
                        "shape_b requires exactly 2 owning orgs".into(),
                    ));
                }
            }
        }
        for org in &payload.owning_orgs {
            if !state.organizations.contains_key(org) {
                return Err(RepositoryError::InvalidArgument(format!(
                    "owning org not found: {org}"
                )));
            }
        }
        if !state.agents.contains_key(&payload.lead_agent_id) {
            return Err(RepositoryError::InvalidArgument(format!(
                "lead agent not found: {}",
                payload.lead_agent_id
            )));
        }
        for m in &payload.member_agent_ids {
            if !state.agents.contains_key(m) {
                return Err(RepositoryError::InvalidArgument(format!(
                    "member agent not found: {m}"
                )));
            }
        }
        for s in &payload.sponsor_agent_ids {
            if !state.agents.contains_key(s) {
                return Err(RepositoryError::InvalidArgument(format!(
                    "sponsor agent not found: {s}"
                )));
            }
        }

        // ---- commit --------------------------------------------
        state.projects.insert(pid, payload.project.clone());
        for org in &payload.owning_orgs {
            state.project_belongs_to_edges.push((pid, *org));
        }
        state.has_lead_edges.push((pid, payload.lead_agent_id));
        // `HAS_AGENT` / `HAS_SPONSOR` edges are tracked structurally
        // via the project row; in-memory repo does not (yet) carry a
        // dedicated Vec for them. Surface-level reads live on the
        // SurrealDB impl in P4+. At M4/P3 the in-memory impl's scope
        // is proving the compound-tx ordering + rollback semantics.
        for (uri, kind) in &payload.catalogue_entries {
            state
                .catalogue
                .push((Some(payload.owning_orgs[0]), uri.clone(), kind.clone()));
        }

        Ok(crate::repository::ProjectCreationReceipt {
            project_id: pid,
            owning_org_ids: payload.owning_orgs.clone(),
            lead_agent_id: payload.lead_agent_id,
            has_lead_edge_id: EdgeId::new(),
        })
    }

    async fn apply_agent_creation(
        &self,
        payload: &crate::repository::AgentCreationPayload,
    ) -> RepositoryResult<crate::repository::AgentCreationReceipt> {
        let mut state = self.lock()?;
        let aid = payload.agent.id;

        // ---- pre-flight validation ------------------------------
        if state.agents.contains_key(&aid) {
            return Err(RepositoryError::Conflict(format!(
                "agent already exists: {aid}"
            )));
        }
        let Some(org_id) = payload.agent.owning_org else {
            return Err(RepositoryError::InvalidArgument(
                "agent.owning_org must be Some".into(),
            ));
        };
        if !state.organizations.contains_key(&org_id) {
            return Err(RepositoryError::InvalidArgument(format!(
                "owning org not found: {org_id}"
            )));
        }
        if let Some(role) = payload.agent.role {
            if !role.is_valid_for(payload.agent.kind) {
                return Err(RepositoryError::InvalidArgument(format!(
                    "role {:?} invalid for kind {:?}",
                    role, payload.agent.kind
                )));
            }
        }
        for g in &payload.default_grants {
            match &g.holder {
                crate::model::nodes::PrincipalRef::Agent(a) if *a == aid => {}
                _ => {
                    return Err(RepositoryError::InvalidArgument(
                        "default_grants[n].holder must be Agent(payload.agent.id)".into(),
                    ));
                }
            }
        }
        if let Some(ref ovr) = payload.initial_execution_limits_override {
            if ovr.owning_agent != aid {
                return Err(RepositoryError::InvalidArgument(
                    "initial_execution_limits_override.owning_agent must match payload.agent.id"
                        .into(),
                ));
            }
        }
        if payload.inbox.agent_id != aid {
            return Err(RepositoryError::InvalidArgument(
                "inbox.agent_id must match payload.agent.id".into(),
            ));
        }
        if payload.outbox.agent_id != aid {
            return Err(RepositoryError::InvalidArgument(
                "outbox.agent_id must match payload.agent.id".into(),
            ));
        }
        if let Some(ref p) = payload.profile {
            if p.agent_id != aid {
                return Err(RepositoryError::InvalidArgument(
                    "profile.agent_id must match payload.agent.id".into(),
                ));
            }
        }

        // ---- commit --------------------------------------------
        state.agents.insert(aid, payload.agent.clone());
        state
            .inboxes
            .insert(payload.inbox.id, payload.inbox.clone());
        state
            .outboxes
            .insert(payload.outbox.id, payload.outbox.clone());
        let profile_id = payload.profile.as_ref().map(|p| {
            state.agent_profiles.insert(p.id, p.clone());
            p.id
        });
        let mut default_grant_ids = Vec::with_capacity(payload.default_grants.len());
        for g in &payload.default_grants {
            state.grants.insert(g.id, g.clone());
            default_grant_ids.push(g.id);
        }
        let execution_limits_override_id =
            payload
                .initial_execution_limits_override
                .as_ref()
                .map(|ovr| {
                    state.agent_execution_limits.insert(aid, ovr.clone());
                    ovr.id
                });
        for (uri, kind) in &payload.catalogue_entries {
            state
                .catalogue
                .push((Some(org_id), uri.clone(), kind.clone()));
        }

        Ok(crate::repository::AgentCreationReceipt {
            agent_id: aid,
            owning_org_id: org_id,
            inbox_id: payload.inbox.id,
            outbox_id: payload.outbox.id,
            profile_id,
            default_grant_ids,
            execution_limits_override_id,
        })
    }

    // ---- M5/P2 — Session + sidecar + catalog + status surface ---------

    async fn persist_session(
        &self,
        session: &Session,
        first_loop: &LoopRecordNode,
    ) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        if s.sessions.contains_key(&session.id) {
            return Err(RepositoryError::Conflict(format!(
                "session {} already persisted",
                session.id
            )));
        }
        s.sessions.insert(session.id, session.clone());
        s.loop_records.insert(first_loop.id, first_loop.clone());
        s.runs_session_edges
            .push((session.id, session.owning_project));
        Ok(())
    }

    async fn append_loop_record(&self, loop_record: &LoopRecordNode) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        if !s.sessions.contains_key(&loop_record.session_id) {
            return Err(RepositoryError::NotFound);
        }
        s.loop_records.insert(loop_record.id, loop_record.clone());
        Ok(())
    }

    async fn append_turn(&self, turn: &TurnNode) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        if !s.loop_records.contains_key(&turn.loop_id) {
            return Err(RepositoryError::NotFound);
        }
        s.turn_nodes.insert(turn.id, turn.clone());
        Ok(())
    }

    async fn append_agent_event(
        &self,
        session: SessionId,
        event: serde_json::Value,
    ) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        if !s.sessions.contains_key(&session) {
            return Err(RepositoryError::NotFound);
        }
        s.session_agent_events
            .entry(session)
            .or_default()
            .push(event);
        Ok(())
    }

    async fn fetch_session(&self, session: SessionId) -> RepositoryResult<Option<SessionDetail>> {
        let s = self.lock()?;
        let Some(sess) = s.sessions.get(&session).cloned() else {
            return Ok(None);
        };
        let mut loops: Vec<LoopRecordNode> = s
            .loop_records
            .values()
            .filter(|r| r.session_id == session)
            .cloned()
            .collect();
        loops.sort_by_key(|r| r.loop_index);
        let mut turns_by_loop: std::collections::BTreeMap<LoopId, Vec<TurnNode>> =
            std::collections::BTreeMap::new();
        for lr in &loops {
            turns_by_loop.insert(lr.id, Vec::new());
        }
        for t in s.turn_nodes.values() {
            if let Some(bucket) = turns_by_loop.get_mut(&t.loop_id) {
                bucket.push(t.clone());
            }
        }
        for bucket in turns_by_loop.values_mut() {
            bucket.sort_by_key(|t| t.turn_index);
        }
        Ok(Some(SessionDetail {
            session: sess,
            loops,
            turns_by_loop,
        }))
    }

    async fn list_sessions_in_project(&self, project: ProjectId) -> RepositoryResult<Vec<Session>> {
        let s = self.lock()?;
        let mut out: Vec<Session> = s
            .sessions
            .values()
            .filter(|sess| sess.owning_project == project)
            .cloned()
            .collect();
        // Newest-first by started_at.
        out.sort_by_key(|s| std::cmp::Reverse(s.started_at));
        Ok(out)
    }

    async fn list_active_sessions_for_agent(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Vec<Session>> {
        let s = self.lock()?;
        let out: Vec<Session> = s
            .sessions
            .values()
            .filter(|sess| {
                sess.started_by == agent && sess.governance_state == SessionGovernanceState::Running
            })
            .cloned()
            .collect();
        Ok(out)
    }

    async fn count_active_sessions_for_agent(&self, agent: AgentId) -> RepositoryResult<u32> {
        let s = self.lock()?;
        let n = s
            .sessions
            .values()
            .filter(|sess| {
                sess.started_by == agent && sess.governance_state == SessionGovernanceState::Running
            })
            .count();
        Ok(n as u32)
    }

    async fn mark_session_ended(
        &self,
        session: SessionId,
        at: DateTime<Utc>,
        state: SessionGovernanceState,
    ) -> RepositoryResult<()> {
        if state == SessionGovernanceState::Running {
            return Err(RepositoryError::InvalidArgument(
                "mark_session_ended requires a terminal state".into(),
            ));
        }
        let mut s = self.lock()?;
        let Some(row) = s.sessions.get_mut(&session) else {
            return Err(RepositoryError::NotFound);
        };
        if row.governance_state.is_terminal() {
            return Err(RepositoryError::Conflict(format!(
                "session {} already terminal ({:?})",
                session, row.governance_state
            )));
        }
        row.governance_state = state;
        row.ended_at = Some(at);
        Ok(())
    }

    async fn terminate_session(
        &self,
        session: SessionId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        // Delegate to mark_session_ended — reason + actor live on
        // the audit event emitted by the handler, not on the
        // persistent session row.
        self.mark_session_ended(session, at, SessionGovernanceState::Aborted)
            .await
    }

    async fn persist_shape_b_pending(&self, row: &ShapeBPendingProject) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        if s.shape_b_pending_projects
            .contains_key(&row.auth_request_id)
        {
            return Err(RepositoryError::Conflict(format!(
                "shape_b_pending_projects already exists for ar {}",
                row.auth_request_id
            )));
        }
        s.shape_b_pending_projects
            .insert(row.auth_request_id, row.clone());
        Ok(())
    }

    async fn fetch_shape_b_pending(
        &self,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<Option<ShapeBPendingProject>> {
        let s = self.lock()?;
        Ok(s.shape_b_pending_projects.get(&auth_request).cloned())
    }

    async fn delete_shape_b_pending(&self, auth_request: AuthRequestId) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        s.shape_b_pending_projects.remove(&auth_request);
        Ok(())
    }

    async fn upsert_agent_catalog_entry(&self, entry: &AgentCatalogEntry) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        s.agent_catalog_entries
            .insert(entry.agent_id, entry.clone());
        Ok(())
    }

    async fn list_agent_catalog_entries_in_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AgentCatalogEntry>> {
        let s = self.lock()?;
        let mut out: Vec<AgentCatalogEntry> = s
            .agent_catalog_entries
            .values()
            .filter(|e| e.owning_org == org)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(out)
    }

    async fn get_agent_catalog_entry(
        &self,
        agent: AgentId,
    ) -> RepositoryResult<Option<AgentCatalogEntry>> {
        let s = self.lock()?;
        Ok(s.agent_catalog_entries.get(&agent).cloned())
    }

    async fn upsert_system_agent_runtime_status(
        &self,
        status: &SystemAgentRuntimeStatus,
    ) -> RepositoryResult<()> {
        let mut s = self.lock()?;
        s.system_agent_runtime_status
            .insert(status.agent_id, status.clone());
        Ok(())
    }

    async fn fetch_system_agent_runtime_status_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<SystemAgentRuntimeStatus>> {
        let s = self.lock()?;
        let mut out: Vec<SystemAgentRuntimeStatus> = s
            .system_agent_runtime_status
            .values()
            .filter(|st| st.owning_org == org)
            .cloned()
            .collect();
        out.sort_by_key(|st| st.agent_id);
        Ok(out)
    }

    async fn list_authority_templates_for_org(
        &self,
        _org: OrgId,
    ) -> RepositoryResult<Vec<Template>> {
        // ADR-0030: Template rows are platform-level. Return every
        // Template row (ordered by kind) — the page 12 handler
        // augments with per-org adoption state at the HTTP tier.
        let s = self.lock()?;
        let mut out: Vec<Template> = s.templates.values().cloned().collect();
        out.sort_by_key(|t| t.kind);
        Ok(out)
    }

    async fn count_grants_fired_by_adoption(
        &self,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<u32> {
        let s = self.lock()?;
        let n = s
            .grants
            .values()
            .filter(|g| g.descends_from.as_ref() == Some(&auth_request))
            .count();
        Ok(n as u32)
    }

    async fn list_revoked_adoptions_for_org(
        &self,
        org: OrgId,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        // Adoption ARs carry `org:<id>/template:<kind>` as one of
        // their `resource_slots[n].resource.uri` values — matches the
        // SurrealDB-side `list_adoption_ars_for_org` prefix filter.
        let prefix = format!("org:{}/template:", org);
        let s = self.lock()?;
        let mut out: Vec<AuthRequest> = s
            .auth_requests
            .values()
            .filter(|ar| {
                ar.state == AuthRequestState::Revoked
                    && ar
                        .resource_slots
                        .iter()
                        .any(|slot| slot.resource.uri.starts_with(&prefix))
            })
            .cloned()
            .collect();
        out.sort_by_key(|ar| std::cmp::Reverse(ar.submitted_at));
        Ok(out)
    }
}

/// Compute the set of orgs that are in `old` but not in `new`. Used by
/// `narrow_mcp_tenants` to drive the cascade. `TenantSet::All` on
/// either side is treated as "every known org" — but since we don't
/// hold a platform-wide org index here, we approximate:
///
/// - `old = All`, `new = Only(ids)` → dropped is implicitly "everything
///   NOT in `ids`". Since we can't enumerate "everything", we return an
///   empty list here and rely on the handler (which CAN enumerate org
///   rows via `list_all_orgs`) to compute the full dropped set. M2/P6
///   restricts this path to handlers that enumerate explicitly.
/// - `old = Only(a)`, `new = Only(b)` → dropped = `a \ b`.
/// - `old = Only(_)`, `new = All` → widening; no drops.
/// - `old = All`, `new = All` → no change.
fn dropped_orgs(old: &TenantSet, new: &TenantSet) -> Vec<OrgId> {
    match (old, new) {
        (TenantSet::Only(old_ids), TenantSet::Only(new_ids)) => old_ids
            .iter()
            .filter(|o| !new_ids.contains(o))
            .copied()
            .collect(),
        (TenantSet::Only(_), TenantSet::All) => vec![],
        (TenantSet::All, TenantSet::All) => vec![],
        (TenantSet::All, TenantSet::Only(_)) => vec![],
    }
}

fn principals_equal(a: &PrincipalRef, b: &PrincipalRef) -> bool {
    match (a, b) {
        (PrincipalRef::Agent(x), PrincipalRef::Agent(y)) => x == y,
        (PrincipalRef::User(x), PrincipalRef::User(y)) => x == y,
        (PrincipalRef::Organization(x), PrincipalRef::Organization(y)) => x == y,
        (PrincipalRef::Project(x), PrincipalRef::Project(y)) => x == y,
        (PrincipalRef::System(x), PrincipalRef::System(y)) => x == y,
        _ => false,
    }
}
