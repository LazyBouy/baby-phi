//! Domain-event listener implementations.
//!
//! At M4/P3 this module shipped a single listener —
//! [`TemplateAFireListener`] — that reacts to
//! [`DomainEvent::HasLeadEdgeCreated`] by minting the lead's grant.
//!
//! M5/P3 + CH-22 extend the set with four more:
//! - [`TemplateCFireListener`] — full body; reacts to
//!   [`DomainEvent::ManagesEdgeCreated`], calls
//!   [`crate::templates::c::fire_grant_on_manages_edge`], persists the
//!   Grant, emits `template.c.grant_fired` audit.
//! - [`TemplateDFireListener`] — full body; reacts to
//!   [`DomainEvent::HasAgentSupervisorEdgeCreated`], calls
//!   [`crate::templates::d::fire_grant_on_has_agent_supervisor`].
//! - [`MemoryExtractionListener`] — **stub at M5/P3**. Subscribes to
//!   [`DomainEvent::SessionEnded`]; body lands at CH-21.
//! - [`AgentCatalogListener`] — **CH-22 body shipped**. Subscribes to
//!   8 DomainEvent variants; on each fire upserts the
//!   `agent_catalog_entry` row and advances the catalog system
//!   agent's `system_agent_runtime_status` tile via
//!   [`record_system_agent_fire`] (drift D6.1 second call site).
//!   Honors ADR-0034 §D34.5 — consults `Agent.active` /
//!   `Agent.archived_at` via [`Repository::get_agent`] (archive wins
//!   ties) and is read-only on agent lifecycle.
//!
//! All five listeners share the same fail-safe semantics (ADR-0028):
//! events emit AFTER the owning compound-tx commits; listener errors
//! log + drop (no auto-retry at M5; M7b adds the retry fabric).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::audit::AuditEmitter;
use crate::events::{DomainEvent, EventHandler};
use crate::model::composites_m5::{AgentCatalogEntry, SystemAgentRuntimeStatus};
use crate::model::ids::{
    AgentCatalogEntryId, AgentId, AuthRequestId, OrgId, ProjectId, SystemAgentRuntimeStatusId,
};
use crate::templates::a::{fire_grant_on_lead_assignment, FireArgs};
use crate::Repository;

/// Stable display-name slug for the per-org agent-catalog system agent
/// provisioned at M3 org creation (`build_system_agents` in
/// `server::platform::orgs::create`). The listener resolves the
/// catalog system agent by walking
/// [`Organization::system_agents`](crate::model::nodes::Organization::system_agents)
/// and matching on this name. Tests that bypass the M3 provisioner
/// must reproduce the same display name to be observed by the
/// listener.
pub const AGENT_CATALOG_SYSTEM_AGENT_DISPLAY_NAME: &str = "agent-catalog";

/// Audit emission mode for the agent-catalog listener (ADR-0035 / CH-22).
///
/// The listener fires on up to 8 `DomainEvent` variants per session
/// lifecycle. Emitting an audit event on every fire would 10–100×
/// audit-log volume on busy orgs without governance benefit (catalog
/// refresh is observability data, not permission-relevant). Default
/// is [`CatalogAuditMode::Silent`] — no audit emission. Operators
/// flip to [`CatalogAuditMode::Debug`] (via
/// `[listeners.catalog] audit_mode = "debug"` in `config/<profile>.toml`
/// or `PHI_LISTENERS__CATALOG__AUDIT_MODE=debug`) for end-to-end
/// traceability during dev or acceptance testing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CatalogAuditMode {
    #[default]
    Silent,
    Debug,
}

/// Shared helper every page-13-aware listener uses to upsert its
/// `SystemAgentRuntimeStatus` tile on each fire (M5/P6 +
/// R-ADMIN-13-R2 / N3).
///
/// Each listener supplies the `(org, agent, effective_parallelize)`
/// triple it has on hand + optional error. The helper writes via
/// `repo.upsert_system_agent_runtime_status`; any repo error is
/// logged + swallowed (listener errors never propagate per
/// ADR-0028 fail-safe semantics).
///
/// **M5/P6 note**: call sites for Template A/C/D fire listeners
/// are deferred (those listeners write grants, not system-agent
/// telemetry). The helper is ready for the M5/P8 memory-extraction
/// + agent-catalog listener bodies which DO target system agents.
///
/// See drift **D6.1** in the plan archive.
pub async fn record_system_agent_fire(
    repo: &dyn Repository,
    org: OrgId,
    agent: AgentId,
    effective_parallelize: u32,
    last_error: Option<String>,
    now: DateTime<Utc>,
) {
    let id = SystemAgentRuntimeStatusId::new();
    let tags =
        crate::model::composites::auto_tags_for("system_agent_runtime_status", &id.to_string())
            .to_vec();
    let status = SystemAgentRuntimeStatus {
        id,
        agent_id: agent,
        owning_org: org,
        queue_depth: 0, // P8 bodies will compute; P6 helper seeds idle.
        last_fired_at: Some(now),
        effective_parallelize,
        last_error,
        updated_at: now,
        tags,
    };
    if let Err(e) = repo.upsert_system_agent_runtime_status(&status).await {
        tracing::error!(
            org = %org,
            agent = %agent,
            error = %e,
            "record_system_agent_fire: upsert failed — runtime-status tile stale",
        );
    }
}

/// Reactive subscriber that fires the Template A lead grant every
/// time a `HAS_LEAD` edge is reported on the event bus.
///
/// ## Resolution of the required ids
///
/// The pure-fn grant builder needs the adoption AR id + the actor id
/// for the audit event. Both come from the **org** the project
/// belongs to:
/// - `adoption_ar_resolver` is a pluggable callback supplied at
///   construction so the listener can ask "what adoption AR did this
///   org's Template A self-approval produce?" without hardcoding a
///   repo query. Production wires it to
///   `Repository::list_adoption_auth_requests_for_org` + a filter on
///   `TemplateKind::A`.
/// - `actor_for_org` returns "who on behalf of this org issued the
///   grant?" — M4/P3 wires this to the org's CEO by default. Swap it
///   for a service-account agent at M7+ if audit provenance needs to
///   name a system agent instead.
///
/// ## phi-core leverage
///
/// None. Listener logic is pure phi governance reactive flow.
pub struct TemplateAFireListener {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    adoption_ar_resolver: Arc<dyn AdoptionArResolver>,
    actor_resolver: Arc<dyn ActorResolver>,
}

impl TemplateAFireListener {
    pub fn new(
        repo: Arc<dyn Repository>,
        audit: Arc<dyn AuditEmitter>,
        adoption_ar_resolver: Arc<dyn AdoptionArResolver>,
        actor_resolver: Arc<dyn ActorResolver>,
    ) -> Self {
        Self {
            repo,
            audit,
            adoption_ar_resolver,
            actor_resolver,
        }
    }
}

/// Resolves the Template A adoption AR id for a project's owning org.
/// Production wires this to the repository; tests can stub it.
#[async_trait]
pub trait AdoptionArResolver: Send + Sync {
    /// Resolve the adoption AR that authorises Template A fires for
    /// the project identified by `project`. Returns `None` when the
    /// project's owning org has not adopted Template A — in which
    /// case the listener skips the grant issuance (logs a warning).
    async fn resolve(
        &self,
        project: crate::model::ids::ProjectId,
    ) -> Option<(OrgId, AuthRequestId)>;
}

/// Resolves the actor Agent id for an audit event scoped to `org`.
/// Typically the org's CEO; the governance layer may delegate this
/// to a dedicated system agent at M7+.
#[async_trait]
pub trait ActorResolver: Send + Sync {
    async fn resolve(&self, org: OrgId) -> Option<AgentId>;
}

#[async_trait]
impl EventHandler for TemplateAFireListener {
    async fn on_event(&self, event: &DomainEvent) {
        // Only the Template-A trigger variant is handled here;
        // every other variant is ignored (the match's catch-all
        // arm keeps the match total as new M5+ variants land).
        match event {
            DomainEvent::HasLeadEdgeCreated {
                project,
                lead,
                event_id,
                ..
            } => {
                let Some((org, adoption_ar)) = self.adoption_ar_resolver.resolve(*project).await
                else {
                    tracing::warn!(
                        project = %project,
                        event_id = %event_id,
                        "TemplateAFireListener: no Template-A adoption AR for project's org; \
                         skipping grant issuance",
                    );
                    return;
                };
                let Some(actor) = self.actor_resolver.resolve(org).await else {
                    tracing::warn!(
                        project = %project,
                        org = %org,
                        event_id = %event_id,
                        "TemplateAFireListener: no actor resolver result for org; \
                         skipping grant issuance",
                    );
                    return;
                };

                let now = Utc::now();
                let grant = fire_grant_on_lead_assignment(FireArgs {
                    project: *project,
                    lead: *lead,
                    adoption_auth_request_id: adoption_ar,
                    now,
                });
                let grant_id = grant.id;

                // Fail-safe semantics (ADR-0028): listener errors are
                // logged + dropped, not propagated. The compound tx is
                // already durable; a grant miss means the operator
                // must manually replay via M7b's retry machinery
                // (lands later). Re-entry on restart is safe because
                // `fire_grant_on_lead_assignment` mints a fresh
                // `GrantId` and the duplicate is easily detected.
                if let Err(e) = self.repo.create_grant(&grant).await {
                    tracing::error!(
                        project = %project,
                        event_id = %event_id,
                        error = %e,
                        "TemplateAFireListener: create_grant failed — operator must replay",
                    );
                    return;
                }

                let audit_event = crate::audit::events::m4::templates::template_a_grant_fired(
                    actor,
                    org,
                    *project,
                    *lead,
                    grant_id,
                    adoption_ar,
                    now,
                );
                if let Err(e) = self.audit.emit(audit_event).await {
                    tracing::error!(
                        project = %project,
                        event_id = %event_id,
                        error = %e,
                        "TemplateAFireListener: audit emit failed after grant persisted — \
                         grant is durable but audit trail has a gap",
                    );
                }
            }
            _ => {
                // Template A reacts to HAS_LEAD only; other M5
                // variants have their own listeners.
            }
        }
    }
}

// ===========================================================================
// M5/P3 — Template C fire listener
// ===========================================================================

/// Resolves the Template C adoption AR id for an org.
///
/// Production wires this to
/// `Repository::list_adoption_auth_requests_for_org` filtered on
/// `TemplateKind::C`; tests can stub it.
#[async_trait]
pub trait TemplateCAdoptionArResolver: Send + Sync {
    async fn resolve(&self, org: OrgId) -> Option<AuthRequestId>;
}

/// Reactive subscriber that fires the Template C manager grant every
/// time a `MANAGES` edge is reported on the event bus.
pub struct TemplateCFireListener {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    adoption_ar_resolver: Arc<dyn TemplateCAdoptionArResolver>,
    actor_resolver: Arc<dyn ActorResolver>,
}

impl TemplateCFireListener {
    pub fn new(
        repo: Arc<dyn Repository>,
        audit: Arc<dyn AuditEmitter>,
        adoption_ar_resolver: Arc<dyn TemplateCAdoptionArResolver>,
        actor_resolver: Arc<dyn ActorResolver>,
    ) -> Self {
        Self {
            repo,
            audit,
            adoption_ar_resolver,
            actor_resolver,
        }
    }
}

#[async_trait]
impl EventHandler for TemplateCFireListener {
    async fn on_event(&self, event: &DomainEvent) {
        let DomainEvent::ManagesEdgeCreated {
            org_id,
            manager,
            subordinate,
            event_id,
            ..
        } = event
        else {
            return;
        };

        let Some(adoption_ar) = self.adoption_ar_resolver.resolve(*org_id).await else {
            tracing::warn!(
                org = %org_id,
                event_id = %event_id,
                "TemplateCFireListener: no Template-C adoption AR for org; skipping grant issuance",
            );
            return;
        };
        let Some(actor) = self.actor_resolver.resolve(*org_id).await else {
            tracing::warn!(
                org = %org_id,
                event_id = %event_id,
                "TemplateCFireListener: no actor resolver result for org; skipping grant issuance",
            );
            return;
        };

        let now = Utc::now();
        let grant =
            crate::templates::c::fire_grant_on_manages_edge(crate::templates::c::FireArgs {
                manager: *manager,
                subordinate: *subordinate,
                adoption_auth_request_id: adoption_ar,
                now,
            });
        let grant_id = grant.id;

        if let Err(e) = self.repo.create_grant(&grant).await {
            tracing::error!(
                org = %org_id,
                event_id = %event_id,
                error = %e,
                "TemplateCFireListener: create_grant failed — operator must replay",
            );
            return;
        }

        let audit_event = crate::audit::events::m5::templates::template_c_grant_fired(
            actor,
            *org_id,
            *manager,
            *subordinate,
            grant_id,
            adoption_ar,
            now,
        );
        if let Err(e) = self.audit.emit(audit_event).await {
            tracing::error!(
                org = %org_id,
                event_id = %event_id,
                error = %e,
                "TemplateCFireListener: audit emit failed after grant persisted — \
                 grant is durable but audit trail has a gap",
            );
        }
    }
}

// ===========================================================================
// M5/P3 — Template D fire listener
// ===========================================================================

/// Resolves the Template D adoption AR id for a project by walking
/// `project → belongs_to → org` and looking up the Template-D adoption
/// AR on the owning org. Same shape as M4's `AdoptionArResolver`; the
/// new trait lets the production impl filter by `TemplateKind::D`.
#[async_trait]
pub trait TemplateDAdoptionArResolver: Send + Sync {
    async fn resolve(&self, project: ProjectId) -> Option<(OrgId, AuthRequestId)>;
}

/// Reactive subscriber that fires the Template D supervisor grant
/// every time a `HAS_AGENT_SUPERVISOR` edge is reported on the event
/// bus.
pub struct TemplateDFireListener {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    adoption_ar_resolver: Arc<dyn TemplateDAdoptionArResolver>,
    actor_resolver: Arc<dyn ActorResolver>,
}

impl TemplateDFireListener {
    pub fn new(
        repo: Arc<dyn Repository>,
        audit: Arc<dyn AuditEmitter>,
        adoption_ar_resolver: Arc<dyn TemplateDAdoptionArResolver>,
        actor_resolver: Arc<dyn ActorResolver>,
    ) -> Self {
        Self {
            repo,
            audit,
            adoption_ar_resolver,
            actor_resolver,
        }
    }
}

#[async_trait]
impl EventHandler for TemplateDFireListener {
    async fn on_event(&self, event: &DomainEvent) {
        let DomainEvent::HasAgentSupervisorEdgeCreated {
            project_id,
            supervisor,
            supervisee,
            event_id,
            ..
        } = event
        else {
            return;
        };

        let Some((org, adoption_ar)) = self.adoption_ar_resolver.resolve(*project_id).await else {
            tracing::warn!(
                project = %project_id,
                event_id = %event_id,
                "TemplateDFireListener: no Template-D adoption AR for project's org; skipping",
            );
            return;
        };
        let Some(actor) = self.actor_resolver.resolve(org).await else {
            tracing::warn!(
                project = %project_id,
                org = %org,
                event_id = %event_id,
                "TemplateDFireListener: no actor resolver result for org; skipping",
            );
            return;
        };

        let now = Utc::now();
        let grant = crate::templates::d::fire_grant_on_has_agent_supervisor(
            crate::templates::d::FireArgs {
                project: *project_id,
                supervisor: *supervisor,
                supervisee: *supervisee,
                adoption_auth_request_id: adoption_ar,
                now,
            },
        );
        let grant_id = grant.id;

        if let Err(e) = self.repo.create_grant(&grant).await {
            tracing::error!(
                project = %project_id,
                event_id = %event_id,
                error = %e,
                "TemplateDFireListener: create_grant failed — operator must replay",
            );
            return;
        }

        let audit_event = crate::audit::events::m5::templates::template_d_grant_fired(
            actor,
            org,
            *project_id,
            *supervisor,
            *supervisee,
            grant_id,
            adoption_ar,
            now,
        );
        if let Err(e) = self.audit.emit(audit_event).await {
            tracing::error!(
                project = %project_id,
                event_id = %event_id,
                error = %e,
                "TemplateDFireListener: audit emit failed after grant persisted — \
                 grant is durable but audit trail has a gap",
            );
        }
    }
}

// ===========================================================================
// M5/P3 — Stub listeners (bodies at M5/P8)
// ===========================================================================

/// Memory-extraction listener — **stub at M5/P3**.
///
/// Subscribes to [`DomainEvent::SessionEnded`]; the extraction body
/// (supervisor `agent_loop` run + `MemoryExtracted` audit emission)
/// lands at M5/P8. Pre-registering the subscription here lets the
/// handler-count invariant assert 5 listeners at P3 close without
/// changing the boot-time wiring at P8.
pub struct MemoryExtractionListener {
    _repo: Arc<dyn Repository>,
    _audit: Arc<dyn AuditEmitter>,
}

impl MemoryExtractionListener {
    pub fn new(repo: Arc<dyn Repository>, audit: Arc<dyn AuditEmitter>) -> Self {
        Self {
            _repo: repo,
            _audit: audit,
        }
    }
}

#[async_trait]
impl EventHandler for MemoryExtractionListener {
    async fn on_event(&self, event: &DomainEvent) {
        if let DomainEvent::SessionEnded { event_id, .. } = event {
            tracing::debug!(
                event_id = %event_id,
                "MemoryExtractionListener (stub): SessionEnded received — body ships at M5/P8",
            );
        }
    }
}

/// Agent-catalog listener — body shipped at CH-22.
///
/// Subscribes to the 8 `DomainEvent` variants that drive catalog
/// upserts (`AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged`,
/// `HasLeadEdgeCreated`, `ManagesEdgeCreated`,
/// `HasAgentSupervisorEdgeCreated`, `SessionStarted`, `SessionEnded`).
/// `SessionAborted` is in the subscription set but is a documented
/// no-op (not part of the plan's trigger set — preserves the M5/P3
/// stub semantics so an aborted session does not cause a catalog
/// row to be created).
///
/// On every fire the body:
/// 1. Reads the canonical [`Agent`](crate::model::nodes::Agent) row
///    via [`Repository::get_agent`] (ADR-0034 §D34.5 #1 — never infer
///    lifecycle from the event payload).
/// 2. Computes `catalog_active = agent.active && agent.archived_at.is_none()`
///    (D34.5 #2 + #3 — archive wins ties).
/// 3. Upserts the `agent_catalog_entry` row via
///    [`Repository::upsert_agent_catalog_entry`].
/// 4. Resolves the catalog system agent for the agent's org and
///    advances its `system_agent_runtime_status` tile via
///    [`record_system_agent_fire`] (drift D6.1 second call site).
/// 5. When [`CatalogAuditMode::Debug`] is configured, emits an
///    `agent_catalog_refreshed` audit event for the fire (ADR-0035).
///
/// The body is read-only on agent lifecycle (D34.5 #4) — it never
/// calls [`Repository::set_agent_active`] /
/// [`Repository::set_agent_archived_at`].
pub struct AgentCatalogListener {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    audit_mode: CatalogAuditMode,
}

impl AgentCatalogListener {
    pub fn new(
        repo: Arc<dyn Repository>,
        audit: Arc<dyn AuditEmitter>,
        audit_mode: CatalogAuditMode,
    ) -> Self {
        Self {
            repo,
            audit,
            audit_mode,
        }
    }

    /// Walk `org.system_agents` and return the first entry whose
    /// `display_name` matches [`AGENT_CATALOG_SYSTEM_AGENT_DISPLAY_NAME`].
    /// Returns `None` if the org cannot be loaded, has no system
    /// agents, or none match. Up to 2 lookups in the M5 baseline
    /// (memory-extractor + agent-catalog).
    async fn resolve_catalog_system_agent(&self, org_id: OrgId) -> Option<AgentId> {
        let org = self.repo.get_organization(org_id).await.ok().flatten()?;
        for sys_agent_id in &org.system_agents {
            if let Ok(Some(sys_agent)) = self.repo.get_agent(*sys_agent_id).await {
                if sys_agent.display_name == AGENT_CATALOG_SYSTEM_AGENT_DISPLAY_NAME {
                    return Some(*sys_agent_id);
                }
            }
        }
        None
    }
}

/// Pull `(agent_id, event_at)` out of any of the 8 subscribed
/// variants. Returns `None` for `SessionAborted` (documented no-op
/// per the plan) so the caller can early-return.
fn agent_id_and_timestamp_for(event: &DomainEvent) -> Option<(AgentId, DateTime<Utc>)> {
    match event {
        DomainEvent::AgentCreated { agent_id, at, .. } => Some((*agent_id, *at)),
        DomainEvent::AgentArchived { agent_id, at, .. } => Some((*agent_id, *at)),
        DomainEvent::HasProfileEdgeChanged { agent_id, at, .. } => Some((*agent_id, *at)),
        DomainEvent::HasLeadEdgeCreated { lead, at, .. } => Some((*lead, *at)),
        DomainEvent::ManagesEdgeCreated { manager, at, .. } => Some((*manager, *at)),
        DomainEvent::HasAgentSupervisorEdgeCreated { supervisor, at, .. } => {
            Some((*supervisor, *at))
        }
        DomainEvent::SessionStarted {
            agent_id,
            started_at,
            ..
        } => Some((*agent_id, *started_at)),
        DomainEvent::SessionEnded {
            agent_id, ended_at, ..
        } => Some((*agent_id, *ended_at)),
        DomainEvent::SessionAborted { .. } => None,
    }
}

#[async_trait]
impl EventHandler for AgentCatalogListener {
    async fn on_event(&self, event: &DomainEvent) {
        let Some((agent_id, event_at)) = agent_id_and_timestamp_for(event) else {
            // SessionAborted: documented no-op per CH-22 plan.
            return;
        };

        // ADR-0034 D34.5 #1 — consult the durable Agent row via the repo.
        let agent = match self.repo.get_agent(agent_id).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                tracing::warn!(
                    agent = %agent_id,
                    event_kind = event.kind(),
                    event_id = %event.event_id(),
                    "AgentCatalogListener: agent row missing — skipping catalog upsert",
                );
                return;
            }
            Err(e) => {
                tracing::error!(
                    agent = %agent_id,
                    event_id = %event.event_id(),
                    error = %e,
                    "AgentCatalogListener: get_agent failed — skipping catalog upsert",
                );
                return;
            }
        };

        // The catalog row is org-scoped; orphan agents (no owning_org)
        // pre-date M3's org-creation invariant and have no place in
        // the catalog. Log + skip.
        let Some(org_id) = agent.owning_org else {
            tracing::warn!(
                agent = %agent_id,
                event_id = %event.event_id(),
                "AgentCatalogListener: agent has no owning_org — skipping catalog upsert",
            );
            return;
        };

        // ADR-0034 D34.5 #2+#3 — archive wins ties.
        let catalog_active = agent.active && agent.archived_at.is_none();

        // Read existing entry to preserve fields the current event
        // does not refresh (profile_snapshot, last_seen_at outside
        // session lifecycle, the row's id).
        let existing = self
            .repo
            .get_agent_catalog_entry(agent_id)
            .await
            .ok()
            .flatten();

        let profile_snapshot = match event {
            DomainEvent::HasProfileEdgeChanged { .. } => {
                match self.repo.get_agent_profile_for_agent(agent_id).await {
                    Ok(Some(profile)) => serde_json::to_value(&profile.blueprint).ok(),
                    _ => existing.as_ref().and_then(|e| e.profile_snapshot.clone()),
                }
            }
            _ => existing.as_ref().and_then(|e| e.profile_snapshot.clone()),
        };

        // last_seen_at — refreshed only on session lifecycle; otherwise
        // preserved (or seeded to event_at on first creation).
        let last_seen_at = match event {
            DomainEvent::SessionStarted { .. } | DomainEvent::SessionEnded { .. } => event_at,
            _ => existing
                .as_ref()
                .map(|e| e.last_seen_at)
                .unwrap_or(event_at),
        };

        let id = existing
            .as_ref()
            .map(|e| e.id)
            .unwrap_or_else(AgentCatalogEntryId::new);
        // CH-06: instance-identity tags. On upsert, preserve the
        // existing tags (they're a stable function of `id`); on first
        // insert, emit the canonical pair.
        let tags = match existing.as_ref() {
            Some(e) if !e.tags.is_empty() => e.tags.clone(),
            _ => crate::model::composites::auto_tags_for("agent_catalog_entry", &id.to_string())
                .to_vec(),
        };
        let entry = AgentCatalogEntry {
            id,
            agent_id,
            owning_org: org_id,
            display_name: agent.display_name.clone(),
            kind: agent.kind,
            role: agent.role.map(|r| r.as_str().to_string()),
            active: catalog_active,
            profile_snapshot,
            last_seen_at,
            updated_at: event_at,
            tags,
        };

        if let Err(e) = self.repo.upsert_agent_catalog_entry(&entry).await {
            tracing::error!(
                agent = %agent_id,
                event_id = %event.event_id(),
                error = %e,
                "AgentCatalogListener: upsert_agent_catalog_entry failed — catalog stale",
            );
            return;
        }

        // Drift D6.1 second call site — the catalog system agent's
        // runtime-status tile advances on every fire (whether the
        // refreshed agent IS the catalog system agent itself or not;
        // the catalog system agent is the actor of every catalog
        // refresh).
        let Some(catalog_sys_agent_id) = self.resolve_catalog_system_agent(org_id).await else {
            tracing::warn!(
                org = %org_id,
                event_id = %event.event_id(),
                "AgentCatalogListener: catalog system agent unresolvable for org — \
                 skipping runtime-status tile + audit (catalog upsert succeeded)",
            );
            return;
        };

        let effective_parallelize = self
            .repo
            .get_agent_profile_for_agent(catalog_sys_agent_id)
            .await
            .ok()
            .flatten()
            .map(|p| p.parallelize)
            .unwrap_or(1);

        record_system_agent_fire(
            self.repo.as_ref(),
            org_id,
            catalog_sys_agent_id,
            effective_parallelize,
            None,
            event_at,
        )
        .await;

        if self.audit_mode == CatalogAuditMode::Debug {
            let audit_event = crate::audit::events::m5::agent_catalog::agent_catalog_refreshed(
                catalog_sys_agent_id,
                org_id,
                agent_id,
                event.event_id(),
                event.kind(),
                event_at,
            );
            if let Err(e) = self.audit.emit(audit_event).await {
                tracing::error!(
                    agent = %agent_id,
                    event_id = %event.event_id(),
                    error = %e,
                    "AgentCatalogListener: debug-mode audit emit failed",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditEvent, NoopAuditEmitter};
    use crate::events::{DomainEvent, EventBus, InProcessEventBus};
    use crate::in_memory::InMemoryRepository;
    use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, OrgId, ProjectId};
    use crate::Repository;
    use std::sync::Mutex;

    struct StaticAdoption(Option<(OrgId, AuthRequestId)>);
    #[async_trait]
    impl AdoptionArResolver for StaticAdoption {
        async fn resolve(&self, _p: ProjectId) -> Option<(OrgId, AuthRequestId)> {
            self.0
        }
    }

    struct StaticActor(Option<AgentId>);
    #[async_trait]
    impl ActorResolver for StaticActor {
        async fn resolve(&self, _o: OrgId) -> Option<AgentId> {
            self.0
        }
    }

    /// Capturing audit emitter — lets tests assert the audit event
    /// shape without depending on the persistence layer.
    #[derive(Default)]
    struct CapturingAudit {
        events: Mutex<Vec<AuditEvent>>,
    }
    #[async_trait]
    impl AuditEmitter for CapturingAudit {
        async fn emit(&self, event: AuditEvent) -> crate::repository::RepositoryResult<()> {
            self.events.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn sample_event(project: ProjectId, lead: AgentId) -> DomainEvent {
        DomainEvent::HasLeadEdgeCreated {
            project,
            lead,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[tokio::test]
    async fn listener_fires_grant_and_emits_audit_on_matching_event() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let actor = AgentId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();

        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(actor))),
        ));

        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(project, lead)).await;

        // Grant persisted.
        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(grants.len(), 1, "exactly one lead grant persisted");
        assert_eq!(grants[0].action, vec!["read", "inspect", "list"]);

        // Audit event captured.
        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "template.a.grant_fired");
        assert_eq!(events[0].org_scope, Some(org));
        assert_eq!(events[0].provenance_auth_request_id, Some(adoption_ar));
    }

    #[tokio::test]
    async fn listener_skips_when_adoption_ar_is_absent() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(None)),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;

        assert_eq!(
            repo.list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(
                AgentId::new()
            ))
            .await
            .unwrap()
            .len(),
            0
        );
        assert!(audit.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn listener_skips_when_actor_is_absent() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((OrgId::new(), AuthRequestId::new())))),
            Arc::new(StaticActor(None)),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;

        assert!(audit.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn listener_with_noop_audit_does_not_panic() {
        // Belt-and-braces: wiring a NoopAuditEmitter in a
        // reactive-behaviour-irrelevant test (health probes, etc.)
        // must keep working.
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit,
            Arc::new(StaticAdoption(Some((OrgId::new(), AuthRequestId::new())))),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;
    }

    // ========================================================================
    // M5/P3 — Template C + D + stub listener tests
    // ========================================================================

    struct StaticOrgAdoption(Option<AuthRequestId>);
    #[async_trait]
    impl TemplateCAdoptionArResolver for StaticOrgAdoption {
        async fn resolve(&self, _o: OrgId) -> Option<AuthRequestId> {
            self.0
        }
    }

    struct StaticDAdoption(Option<(OrgId, AuthRequestId)>);
    #[async_trait]
    impl TemplateDAdoptionArResolver for StaticDAdoption {
        async fn resolve(&self, _p: ProjectId) -> Option<(OrgId, AuthRequestId)> {
            self.0
        }
    }

    fn manages_edge_event(org: OrgId, manager: AgentId, subordinate: AgentId) -> DomainEvent {
        DomainEvent::ManagesEdgeCreated {
            org_id: org,
            manager,
            subordinate,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    fn has_agent_supervisor_event(
        project: ProjectId,
        supervisor: AgentId,
        supervisee: AgentId,
    ) -> DomainEvent {
        DomainEvent::HasAgentSupervisorEdgeCreated {
            project_id: project,
            supervisor,
            supervisee,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    #[tokio::test]
    async fn template_c_listener_fires_grant_and_emits_audit() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let actor = AgentId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();

        let listener = Arc::new(TemplateCFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticOrgAdoption(Some(adoption_ar))),
            Arc::new(StaticActor(Some(actor))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(manages_edge_event(org, manager, subordinate))
            .await;

        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(manager))
            .await
            .unwrap();
        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0].action, vec!["read", "inspect"]);
        assert_eq!(grants[0].resource.uri, format!("agent:{}", subordinate));

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "template.c.grant_fired");
        assert_eq!(events[0].org_scope, Some(org));
        assert_eq!(events[0].provenance_auth_request_id, Some(adoption_ar));
    }

    #[tokio::test]
    async fn template_c_listener_skips_when_adoption_ar_absent() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateCFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticOrgAdoption(None)),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(manages_edge_event(
            OrgId::new(),
            AgentId::new(),
            AgentId::new(),
        ))
        .await;
        assert!(audit.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn template_c_listener_ignores_non_matching_event() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateCFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticOrgAdoption(Some(AuthRequestId::new()))),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;
        assert!(audit.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn template_d_listener_fires_project_scoped_grant() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let actor = AgentId::new();
        let project = ProjectId::new();
        let supervisor = AgentId::new();
        let supervisee = AgentId::new();

        let listener = Arc::new(TemplateDFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticDAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(actor))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(has_agent_supervisor_event(project, supervisor, supervisee))
            .await;

        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(supervisor))
            .await
            .unwrap();
        assert_eq!(grants.len(), 1);
        assert_eq!(
            grants[0].resource.uri,
            format!("project:{}/agent:{}", project, supervisee)
        );

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "template.d.grant_fired");
    }

    #[tokio::test]
    async fn template_d_listener_skips_when_project_lacks_adoption_ar() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateDFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticDAdoption(None)),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(has_agent_supervisor_event(
            ProjectId::new(),
            AgentId::new(),
            AgentId::new(),
        ))
        .await;
        assert!(audit.events.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn memory_extraction_listener_is_a_noop_at_p3() {
        // Confirms the stub doesn't panic on its subscribed variant
        // OR on unrelated variants. Full body ships at M5/P8.
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
        let listener = Arc::new(MemoryExtractionListener::new(repo.clone(), audit.clone()));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        // SessionEnded (the subscribed variant).
        bus.emit(DomainEvent::SessionEnded {
            session_id: crate::model::ids::SessionId::new(),
            agent_id: AgentId::new(),
            project_id: ProjectId::new(),
            ended_at: Utc::now(),
            duration_ms: 0,
            turn_count: 0,
            tokens_spent: 0,
            event_id: AuditEventId::new(),
        })
        .await;
        // HasLeadEdgeCreated (unrelated variant).
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;
    }

    // ========================================================================
    // CH-22 P2 — AgentCatalogListener behavioural tests
    //
    // The previous `agent_catalog_listener_is_a_noop_at_p3` test was
    // deleted: its contract (the listener silently logs on every
    // variant) no longer holds — the body upserts the catalog row +
    // advances the runtime-status tile.
    //
    // Tests cover:
    //   - all 8 trigger variants + the SessionAborted no-op
    //   - ADR-0034 §D34.5 conforming criteria 1-4 (durable read,
    //     archive-wins-ties, listener read-only on lifecycle)
    //   - CatalogAuditMode flag (silent default + debug emission)
    //   - drift D6.1 second call site (runtime-status tile advance)
    //   - missing-agent + role-index refresh edge cases
    // ========================================================================

    use crate::audit::AuditClass;
    use crate::model::nodes::{Agent, AgentKind, AgentProfile, AgentRole, Organization};
    use crate::model::ConsentPolicy;
    use chrono::{Duration, TimeZone};

    struct CatalogFixture {
        repo: Arc<dyn Repository>,
        audit: Arc<CapturingAudit>,
        org_id: OrgId,
        catalog_sys_agent_id: AgentId,
        subject_agent_id: AgentId,
    }

    /// Build a 1-org fixture with the catalog system agent registered
    /// in `org.system_agents` and a Human "subject" agent whose
    /// catalog row will be exercised by the tests.
    async fn catalog_fixture() -> CatalogFixture {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let now = Utc::now();
        let org_id = OrgId::new();
        let catalog_sys_agent_id = AgentId::new();
        let subject_agent_id = AgentId::new();

        // Catalog system agent — must use the canonical display_name
        // for the listener's resolver to pick it up.
        let catalog_sys_agent = Agent {
            id: catalog_sys_agent_id,
            kind: AgentKind::Llm,
            display_name: AGENT_CATALOG_SYSTEM_AGENT_DISPLAY_NAME.to_string(),
            owning_org: Some(org_id),
            role: Some(AgentRole::System),
            created_at: now,
            active: true,
            archived_at: None,
        };
        repo.create_agent(&catalog_sys_agent).await.unwrap();

        // Subject agent.
        let subject = Agent {
            id: subject_agent_id,
            kind: AgentKind::Human,
            display_name: "test-subject".to_string(),
            owning_org: Some(org_id),
            role: Some(AgentRole::Member),
            created_at: now,
            active: true,
            archived_at: None,
        };
        repo.create_agent(&subject).await.unwrap();

        // Org listing the catalog system agent.
        let org = Organization {
            id: org_id,
            display_name: "test-org".to_string(),
            vision: None,
            mission: None,
            consent_policy: ConsentPolicy::Implicit,
            audit_class_default: AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![catalog_sys_agent_id],
            created_at: now,
        };
        repo.create_organization(&org).await.unwrap();

        CatalogFixture {
            repo,
            audit,
            org_id,
            catalog_sys_agent_id,
            subject_agent_id,
        }
    }

    fn make_catalog_listener(
        f: &CatalogFixture,
        mode: CatalogAuditMode,
    ) -> Arc<AgentCatalogListener> {
        Arc::new(AgentCatalogListener::new(
            f.repo.clone(),
            f.audit.clone() as Arc<dyn AuditEmitter>,
            mode,
        ))
    }

    fn agent_created_event(agent_id: AgentId, org: OrgId) -> DomainEvent {
        DomainEvent::AgentCreated {
            agent_id,
            owning_org: org,
            agent_kind: AgentKind::Human,
            role: Some(AgentRole::Member),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    fn agent_archived_event(agent_id: AgentId) -> DomainEvent {
        DomainEvent::AgentArchived {
            agent_id,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    fn has_profile_changed_event(agent_id: AgentId) -> DomainEvent {
        DomainEvent::HasProfileEdgeChanged {
            agent_id,
            old_profile_id: None,
            new_profile_id: crate::model::ids::NodeId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        }
    }

    fn session_started_event(agent_id: AgentId, started_at: DateTime<Utc>) -> DomainEvent {
        DomainEvent::SessionStarted {
            session_id: crate::model::ids::SessionId::new(),
            agent_id,
            project_id: ProjectId::new(),
            started_at,
            event_id: AuditEventId::new(),
        }
    }

    fn session_ended_event(agent_id: AgentId, ended_at: DateTime<Utc>) -> DomainEvent {
        DomainEvent::SessionEnded {
            session_id: crate::model::ids::SessionId::new(),
            agent_id,
            project_id: ProjectId::new(),
            ended_at,
            duration_ms: 0,
            turn_count: 0,
            tokens_spent: 0,
            event_id: AuditEventId::new(),
        }
    }

    // ---- (1) AgentCreated upserts a fresh row -----------------------------

    #[tokio::test]
    async fn agent_catalog_listener_upserts_row_on_agent_created() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);

        listener
            .on_event(&agent_created_event(f.subject_agent_id, f.org_id))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row was upserted");
        assert!(entry.active, "newly-created agent is active");
        assert_eq!(entry.display_name, "test-subject");
        assert_eq!(entry.kind, AgentKind::Human);
        assert_eq!(entry.role.as_deref(), Some("member"));
        assert_eq!(entry.owning_org, f.org_id);
    }

    // ---- (2) AgentArchived flips active when the durable column does -----

    #[tokio::test]
    async fn agent_catalog_listener_flips_active_on_agent_archived() {
        let f = catalog_fixture().await;
        // Pre-seed durable lifecycle to mirror the disable+archive
        // handler's writes (CH-01) — the listener consults these.
        f.repo
            .set_agent_active(f.subject_agent_id, false)
            .await
            .unwrap();
        f.repo
            .set_agent_archived_at(f.subject_agent_id, Some(Utc::now()))
            .await
            .unwrap();

        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        listener
            .on_event(&agent_archived_event(f.subject_agent_id))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row exists");
        assert!(
            !entry.active,
            "archived agent's catalog row reflects active=false"
        );
    }

    // ---- (3) Archive wins when active=true but archived_at=Some ----------

    #[tokio::test]
    async fn agent_catalog_listener_archive_wins_when_active_true_but_archived_at_some() {
        let f = catalog_fixture().await;
        // Pre-seed: durable row says active=true but archived_at=Some.
        // ADR-0034 §D34.5 #3 — archive wins ties.
        f.repo
            .set_agent_archived_at(f.subject_agent_id, Some(Utc::now()))
            .await
            .unwrap();
        // Note: set_agent_active intentionally NOT called — the row's
        // active stays at its CREATE-time value of true.

        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        listener
            .on_event(&agent_created_event(f.subject_agent_id, f.org_id))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row exists");
        let durable = f
            .repo
            .get_agent(f.subject_agent_id)
            .await
            .unwrap()
            .expect("subject row exists");
        assert!(
            durable.active && durable.archived_at.is_some(),
            "fixture: durable shows active=true + archived_at=Some",
        );
        assert!(
            !entry.active,
            "ADR-0034 D34.5 #3: archived_at=Some forces catalog active=false \
             regardless of agent.active",
        );
    }

    // ---- (4) Listener is read-only on agent lifecycle (D34.5 #4) --------

    #[tokio::test]
    async fn agent_catalog_listener_is_read_only_on_lifecycle() {
        let f = catalog_fixture().await;
        // Subject starts active=true + archived_at=None (fixture default).
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);

        // Emit the AgentArchived signal — the listener must NOT
        // write back to agent.active or agent.archived_at.
        listener
            .on_event(&agent_archived_event(f.subject_agent_id))
            .await;

        let durable = f
            .repo
            .get_agent(f.subject_agent_id)
            .await
            .unwrap()
            .expect("subject row exists");
        assert!(
            durable.active,
            "ADR-0034 D34.5 #4: listener never writes Agent.active",
        );
        assert!(
            durable.archived_at.is_none(),
            "ADR-0034 D34.5 #4: listener never writes Agent.archived_at",
        );
    }

    // ---- (5) HasProfileEdgeChanged refreshes profile_snapshot ------------

    #[tokio::test]
    async fn agent_catalog_listener_refreshes_profile_snapshot_on_has_profile_edge_changed() {
        let f = catalog_fixture().await;

        // Seed an AgentProfile so get_agent_profile_for_agent succeeds.
        let blueprint = phi_core::agents::profile::AgentProfile {
            name: Some("test-subject-profile".to_string()),
            system_prompt: Some("you are a test subject".to_string()),
            ..Default::default()
        };
        let profile = AgentProfile {
            id: crate::model::ids::NodeId::new(),
            agent_id: f.subject_agent_id,
            parallelize: 2,
            blueprint,
            model_config_id: None,
            mock_response: None,
            created_at: Utc::now(),
        };
        f.repo.create_agent_profile(&profile).await.unwrap();

        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        listener
            .on_event(&has_profile_changed_event(f.subject_agent_id))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row exists");
        let snapshot = entry
            .profile_snapshot
            .expect("profile_snapshot populated on HasProfileEdgeChanged");
        assert_eq!(
            snapshot.get("system_prompt").and_then(|v| v.as_str()),
            Some("you are a test subject"),
            "snapshot carries the refreshed blueprint's system_prompt",
        );
    }

    // ---- (6 + 7) Session lifecycle touches last_seen_at -----------------

    #[tokio::test]
    async fn agent_catalog_listener_touches_last_seen_at_on_session_started() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let t1 = Utc.with_ymd_and_hms(2026, 4, 27, 10, 15, 30).unwrap();

        listener
            .on_event(&session_started_event(f.subject_agent_id, t1))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row exists");
        assert_eq!(
            entry.last_seen_at, t1,
            "SessionStarted bumps last_seen_at to event time",
        );
    }

    #[tokio::test]
    async fn agent_catalog_listener_touches_last_seen_at_on_session_ended() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let t2 = Utc.with_ymd_and_hms(2026, 4, 27, 11, 30, 0).unwrap();

        listener
            .on_event(&session_ended_event(f.subject_agent_id, t2))
            .await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row exists");
        assert_eq!(
            entry.last_seen_at, t2,
            "SessionEnded bumps last_seen_at to event time",
        );
    }

    // ---- (8) HasLeadEdgeCreated touches updated_at -----------------------

    #[tokio::test]
    async fn agent_catalog_listener_role_index_refresh_on_has_lead_edge_created() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let event_at = Utc::now();
        let event = DomainEvent::HasLeadEdgeCreated {
            project: ProjectId::new(),
            lead: f.subject_agent_id,
            at: event_at,
            event_id: AuditEventId::new(),
        };

        listener.on_event(&event).await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row touched on HasLeadEdgeCreated");
        assert_eq!(entry.updated_at, event_at);
    }

    // ---- (9) ManagesEdgeCreated touches updated_at ------------------------

    #[tokio::test]
    async fn agent_catalog_listener_role_index_refresh_on_manages_edge_created() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let event_at = Utc::now();
        let event = DomainEvent::ManagesEdgeCreated {
            org_id: f.org_id,
            manager: f.subject_agent_id,
            subordinate: AgentId::new(),
            at: event_at,
            event_id: AuditEventId::new(),
        };

        listener.on_event(&event).await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row touched on ManagesEdgeCreated");
        assert_eq!(entry.updated_at, event_at);
    }

    // ---- (10) HasAgentSupervisorEdgeCreated touches updated_at -----------

    #[tokio::test]
    async fn agent_catalog_listener_role_index_refresh_on_has_agent_supervisor_edge_created() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let event_at = Utc::now();
        let event = DomainEvent::HasAgentSupervisorEdgeCreated {
            project_id: ProjectId::new(),
            supervisor: f.subject_agent_id,
            supervisee: AgentId::new(),
            at: event_at,
            event_id: AuditEventId::new(),
        };

        listener.on_event(&event).await;

        let entry = f
            .repo
            .get_agent_catalog_entry(f.subject_agent_id)
            .await
            .unwrap()
            .expect("catalog row touched on HasAgentSupervisorEdgeCreated");
        assert_eq!(entry.updated_at, event_at);
    }

    // ---- (11) SessionAborted is a documented no-op -----------------------

    #[tokio::test]
    async fn agent_catalog_listener_silently_ignores_session_aborted() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Debug);

        let event = DomainEvent::SessionAborted {
            session_id: crate::model::ids::SessionId::new(),
            reason: "operator-cancel".to_string(),
            terminated_by: f.subject_agent_id,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        };
        listener.on_event(&event).await;

        assert!(
            f.repo
                .get_agent_catalog_entry(f.subject_agent_id)
                .await
                .unwrap()
                .is_none(),
            "SessionAborted does not create a catalog row",
        );
        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "SessionAborted does not emit even in debug mode",
        );
    }

    // ---- (12) Debug mode emits agent_catalog_refreshed -------------------

    #[tokio::test]
    async fn agent_catalog_listener_emits_audit_in_debug_mode() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Debug);
        let trigger_event = agent_created_event(f.subject_agent_id, f.org_id);
        let trigger_event_id = trigger_event.event_id();

        listener.on_event(&trigger_event).await;

        let events = f.audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1, "exactly one debug-mode audit emission");
        let evt = &events[0];
        assert_eq!(evt.event_type, "agent_catalog_refreshed");
        assert_eq!(evt.org_scope, Some(f.org_id));
        assert_eq!(evt.actor_agent_id, Some(f.catalog_sys_agent_id));
        assert_eq!(
            evt.diff["after"]["triggering_event_id"]
                .as_str()
                .map(str::to_string),
            Some(trigger_event_id.to_string()),
        );
        assert_eq!(
            evt.diff["after"]["triggering_event_kind"].as_str(),
            Some("agent_created"),
        );
    }

    // ---- (13) Silent default emits nothing -------------------------------

    #[tokio::test]
    async fn agent_catalog_listener_silent_in_default_mode() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::default());

        listener
            .on_event(&agent_created_event(f.subject_agent_id, f.org_id))
            .await;

        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "Silent mode (default) emits no audit events",
        );
        // Catalog row still upserted — silence applies only to the
        // audit chain, not the catalog mutation.
        assert!(
            f.repo
                .get_agent_catalog_entry(f.subject_agent_id)
                .await
                .unwrap()
                .is_some(),
            "Silent mode still upserts the catalog row",
        );
    }

    // ---- (14) Missing-agent path is warn-and-skip ------------------------

    #[tokio::test]
    async fn agent_catalog_listener_skips_when_agent_row_missing() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Debug);

        // Emit AgentArchived for an id that doesn't exist in the repo.
        let phantom = AgentId::new();
        listener.on_event(&agent_archived_event(phantom)).await;

        assert!(
            f.repo
                .get_agent_catalog_entry(phantom)
                .await
                .unwrap()
                .is_none(),
            "no catalog row created for missing agent",
        );
        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "no audit emitted when agent row is missing (warn-and-skip path)",
        );
    }

    // ---- (15) Runtime-status tile advances on every fire (D6.1) ----------

    #[tokio::test]
    async fn agent_catalog_listener_runtime_status_tile_advances_on_fire() {
        let f = catalog_fixture().await;
        let listener = make_catalog_listener(&f, CatalogAuditMode::Silent);
        let before = Utc::now() - Duration::seconds(1);

        listener
            .on_event(&agent_created_event(f.subject_agent_id, f.org_id))
            .await;

        let tiles = f
            .repo
            .fetch_system_agent_runtime_status_for_org(f.org_id)
            .await
            .unwrap();
        assert_eq!(
            tiles.len(),
            1,
            "exactly one runtime-status tile after one fire",
        );
        let tile = &tiles[0];
        assert_eq!(
            tile.agent_id, f.catalog_sys_agent_id,
            "tile is keyed on the catalog system agent (D6.1 second call site)",
        );
        let last_fired = tile.last_fired_at.expect("last_fired_at populated");
        assert!(
            last_fired >= before,
            "last_fired_at advanced after listener fired",
        );
    }
}
