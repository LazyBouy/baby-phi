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
    AgentId, AuditEventId, AuthRequestId, EdgeId, GrantId, McpServerId, ModelProviderId, NodeId,
    OrgId, SecretId,
};
use crate::model::nodes::{
    Agent, AgentKind, AgentProfile, AuthRequest, Channel, Consent, Grant, InboxObject, Memory,
    Organization, OutboxObject, PrincipalRef, ResourceRef, Template, ToolAuthorityManifest, User,
};
use crate::model::{
    Composite, ExternalService, ModelRuntime, PlatformDefaults, SecretCredential, SecretRef,
    TenantSet,
};
use crate::repository::{
    BootstrapCredentialRow, Repository, RepositoryError, RepositoryResult, SealedBlob,
    TenantRevocation,
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

    async fn create_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()> {
        self.lock()?
            .agent_profiles
            .insert(profile.id, profile.clone());
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
