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
use crate::model::ids::{AgentId, AuthRequestId, EdgeId, GrantId, NodeId, OrgId};
use crate::model::nodes::{
    Agent, AgentKind, AgentProfile, AuthRequest, Channel, Consent, Grant, InboxObject, Memory,
    Organization, OutboxObject, PrincipalRef, ResourceRef, Template, ToolAuthorityManifest, User,
};
use crate::repository::{BootstrapCredentialRow, Repository, RepositoryError, RepositoryResult};

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
        match state
            .bootstrap_credentials
            .iter_mut()
            .find(|r| r.record_id == record_id)
        {
            Some(row) => {
                row.consumed_at = Some(Utc::now());
                Ok(())
            }
            None => Err(RepositoryError::NotFound),
        }
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

    async fn last_event_hash_for_org(
        &self,
        org: Option<OrgId>,
    ) -> RepositoryResult<Option<[u8; 32]>> {
        Ok(self
            .lock()?
            .audit_events
            .iter()
            .rev()
            .find(|e| e.org_scope == org)
            .and_then(|e| e.prev_event_hash))
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
