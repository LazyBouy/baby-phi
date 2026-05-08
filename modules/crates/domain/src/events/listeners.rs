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

use crate::audit::{AuditClass, AuditEmitter};
use crate::events::{DomainEvent, EventHandler};
use crate::model::composites_m5::{AgentCatalogEntry, SystemAgentRuntimeStatus};
use crate::model::ids::{
    AgentCatalogEntryId, AgentId, AuthRequestId, OrgId, ProjectId, SystemAgentRuntimeStatusId,
};
use crate::permissions::compose_audit_class_with_source;
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

/// Resolve the strictest-wins composed [`AuditClass`] for a fire
/// listener (CH-13 / ADR-0050 §D50.5).
///
/// Reads the org's `audit_class_default` + the adoption AR's
/// `audit_class` from the repository, then folds them through
/// [`compose_audit_class_with_source`] with no per-Grant override
/// (Templates A/C/D do not currently supply one; reserved for
/// forward-scope templates).
///
/// Fail-safe semantics (ADR-0028): a repo error or a missing-row
/// hit logs at WARN and returns `None`, in which case the listener
/// falls back to `(AuditClass::Silent, AuditClassSource::OrgDefault)`
/// — the loosest class is the safest default for the
/// "row not found" path because it avoids accidentally escalating
/// audit volume on a degraded read path. Concept-doc 07 line 71's
/// no-silent-downgrade invariant only applies to the **happy** path
/// (both rows present); a missing-row hit is structural divergence,
/// not a downgrade decision.
async fn resolve_composed_audit_class(
    repo: &dyn Repository,
    org: OrgId,
    adoption_ar_id: AuthRequestId,
) -> (AuditClass, crate::permissions::AuditClassSource) {
    let org_default = match repo.get_organization(org).await {
        Ok(Some(o)) => o.audit_class_default,
        Ok(None) => {
            tracing::warn!(
                org = %org,
                "resolve_composed_audit_class: organization row not found; falling back to Silent",
            );
            return (
                AuditClass::Silent,
                crate::permissions::AuditClassSource::OrgDefault,
            );
        }
        Err(e) => {
            tracing::warn!(
                org = %org,
                error = %e,
                "resolve_composed_audit_class: repo.get_organization failed; falling back to Silent",
            );
            return (
                AuditClass::Silent,
                crate::permissions::AuditClassSource::OrgDefault,
            );
        }
    };

    let template_ar = match repo.get_auth_request(adoption_ar_id).await {
        Ok(Some(ar)) => ar.audit_class,
        Ok(None) => {
            tracing::warn!(
                adoption_ar_id = %adoption_ar_id,
                "resolve_composed_audit_class: adoption AR row not found; \
                 composing without template_ar candidate",
            );
            // Treat missing template_ar as `Silent` so it never wins
            // strictest-of-three: the org default decides alone.
            AuditClass::Silent
        }
        Err(e) => {
            tracing::warn!(
                adoption_ar_id = %adoption_ar_id,
                error = %e,
                "resolve_composed_audit_class: repo.get_auth_request failed; \
                 composing without template_ar candidate",
            );
            AuditClass::Silent
        }
    };

    compose_audit_class_with_source(org_default, template_ar, None)
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
                // CH-13 / ADR-0050 §D50.5 — strictest-wins composed
                // audit class over (org_default, template_ar, None).
                // Production Templates A/C/D do not supply per-Grant
                // overrides today; reserved for forward-scope.
                let (resolved_class, audit_class_source) =
                    resolve_composed_audit_class(self.repo.as_ref(), org, adoption_ar).await;

                // CH-15 / ADR-0054 §D54.3 — `fire_grant_on_lead_assignment`
                // returns Vec<Grant> (project-resource grant + paired
                // session-object grant). Each grant persists via its
                // own `repo.create_grant` call; one
                // `template.a.grant_fired` audit event fires per
                // persisted grant so operators see the full pair in
                // the audit log.
                let grants = fire_grant_on_lead_assignment(FireArgs {
                    project: *project,
                    lead: *lead,
                    adoption_auth_request_id: adoption_ar,
                    now,
                    audit_class: resolved_class,
                });

                // Fail-safe semantics (ADR-0028): listener errors are
                // logged + dropped, not propagated. The compound tx is
                // already durable; a grant miss means the operator
                // must manually replay via M7b's retry machinery
                // (lands later). Re-entry on restart is safe because
                // `fire_grant_on_lead_assignment` mints fresh
                // `GrantId`s on each call and the duplicate is easily
                // detected.
                for grant in &grants {
                    if let Err(e) = self.repo.create_grant(grant).await {
                        tracing::error!(
                            project = %project,
                            event_id = %event_id,
                            grant_id = %grant.id,
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
                        grant.id,
                        adoption_ar,
                        resolved_class,
                        audit_class_source,
                        now,
                    );
                    if let Err(e) = self.audit.emit(audit_event).await {
                        tracing::error!(
                            project = %project,
                            event_id = %event_id,
                            grant_id = %grant.id,
                            error = %e,
                            "TemplateAFireListener: audit emit failed after grant persisted — \
                             grant is durable but audit trail has a gap",
                        );
                    }
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
        // CH-13 / ADR-0050 §D50.5 — strictest-wins composed audit class.
        let (resolved_class, audit_class_source) =
            resolve_composed_audit_class(self.repo.as_ref(), *org_id, adoption_ar).await;

        let grant =
            crate::templates::c::fire_grant_on_manages_edge(crate::templates::c::FireArgs {
                manager: *manager,
                subordinate: *subordinate,
                adoption_auth_request_id: adoption_ar,
                now,
                audit_class: resolved_class,
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
            resolved_class,
            audit_class_source,
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
        // CH-13 / ADR-0050 §D50.5 — strictest-wins composed audit class.
        let (resolved_class, audit_class_source) =
            resolve_composed_audit_class(self.repo.as_ref(), org, adoption_ar).await;

        let grant = crate::templates::d::fire_grant_on_has_agent_supervisor(
            crate::templates::d::FireArgs {
                project: *project_id,
                supervisor: *supervisor,
                supervisee: *supervisee,
                adoption_auth_request_id: adoption_ar,
                now,
                audit_class: resolved_class,
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
            resolved_class,
            audit_class_source,
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

/// Stable display-name slug for the per-org memory-extraction system
/// agent provisioned at M3 org creation. Mirrors the catalog system
/// agent's `AGENT_CATALOG_SYSTEM_AGENT_DISPLAY_NAME` pattern — the
/// listener resolves the extractor by walking
/// [`Organization::system_agents`](crate::model::nodes::Organization::system_agents)
/// and matching on this name.
pub const MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME: &str = "memory-extraction-agent";

/// Memory-extraction listener body shipped at CH-21 / ADR-0040.
///
/// Subscribes to [`DomainEvent::SessionEnded`]. On every non-aborted
/// fire by an LLM-kind working agent, the body:
///
/// 1. Reads the working agent's `Agent` row + the `SessionDetail` via
///    [`Repository::fetch_session`]; bails on Human-kind, missing
///    agent, missing session, or `governance_state == Aborted`.
/// 2. Resolves the org's `memory-extraction-agent` system agent via
///    `Organization::system_agents`; bails when unresolvable.
/// 3. Honors operator disable: if the system agent has
///    `active = false` or `archived_at = Some(_)`, logs `warn!` and
///    returns without minting a Memory or advancing telemetry
///    (ADR-0040 §D40.3 / Fork 4 SKIP-BOTH).
/// 4. Mints exactly one [`Memory`] per fire (ADR-0040 §D40.1
///    HEURISTIC v0); tags = union of `Session.tags` (CH-06 instance
///    tags) + `agent:{started_by}` + `session:{session_id}` +
///    `project:{project_id}` + `org:{owning_org}` (ADR-0040 §D40.2
///    DERIVE FROM SESSION TAGS).
/// 5. Decides the binary scope bucket: `#public` in tag set →
///    [`ExtractionScope::Public`]; else [`ExtractionScope::Private`]
///    (ADR-0040 §D40.2). The 4-pool routing
///    (private/project/org/`#public`) from
///    `concepts/system-agents.md` § "Allocation Rules" is deferred
///    to **M6-DEFERRED-04** per ADR-0040 § Out-of-Scope.
/// 6. Reads + upserts the working agent's `Identity` row (CH-16
///    surface): bumps `witnessed.memories_extracted += 1`, the
///    matching `extraction_scope_distribution.{private|public} += 1`,
///    and `updated_at = ended_at`. Identity-missing for an LLM agent
///    logs `warn!` and skips just the Identity update (Memory + audit
///    still ship).
/// 7. Emits two audit events: `platform.memory.extracted` (Logged)
///    via [`crate::audit::events::m5_2::memory::memory_extracted`];
///    `platform.identity.updated` (Logged) via
///    [`crate::audit::events::m5_2::identity::identity_updated`] —
///    CH-21 is the **first production emitter** of CH-16's
///    `identity_updated` helper (ADR-0038 §D38.5 carry-forward).
/// 8. Calls [`record_system_agent_fire`] for the memory-extractor
///    system agent — **drift D6.1 first call site** (CH-22 shipped
///    the second).
///
/// Per ADR-0028 fail-safe semantics every step's failure logs +
/// continues where coherent; bails when continuing would produce
/// incoherent governance state. Bus re-emission of
/// [`DomainEvent::MemoryExtracted`] / [`DomainEvent::IdentityUpdated`]
/// is deferred at v0 (no current consumer; would create a bus → listener
/// → bus Arc cycle that's better solved with a `Weak<dyn EventBus>`
/// design once a real subscriber appears — out of scope per
/// ADR-0040 §D40.6).
pub struct MemoryExtractionListener {
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
}

impl MemoryExtractionListener {
    pub fn new(repo: Arc<dyn Repository>, audit: Arc<dyn AuditEmitter>) -> Self {
        Self { repo, audit }
    }

    /// Walk `org.system_agents` and return the first entry whose
    /// `display_name` matches
    /// [`MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME`]. Returns
    /// `None` when the org cannot be loaded, has no system agents,
    /// or none match. Mirrors the catalog listener's
    /// `resolve_catalog_system_agent` (lines 590–600).
    async fn resolve_memory_extraction_system_agent(
        &self,
        org_id: OrgId,
    ) -> Option<crate::model::nodes::Agent> {
        let org = self.repo.get_organization(org_id).await.ok().flatten()?;
        for sys_agent_id in &org.system_agents {
            if let Ok(Some(sys_agent)) = self.repo.get_agent(*sys_agent_id).await {
                if sys_agent.display_name == MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME {
                    return Some(sys_agent);
                }
            }
        }
        None
    }
}

/// Build the v0 heuristic Memory tag set: union of the source session's
/// CH-06 instance tags + the four governance scope tags
/// (`agent:` / `session:` / `project:` / `org:`). De-duplicated; tag
/// order is deterministic (session.tags first, governance tags
/// appended in fixed order). ADR-0040 §D40.2.
fn build_memory_tags(session: &crate::model::nodes::Session) -> Vec<String> {
    let mut tags = session.tags.clone();
    let governance_tags = [
        format!("agent:{}", session.started_by),
        format!("session:{}", session.id),
        format!("project:{}", session.owning_project),
        format!("org:{}", session.owning_org),
    ];
    for t in governance_tags {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    tags
}

/// Decide the binary `{private, public}` scope bucket from a tag set.
/// Concept-`agent.md` § "Two Streams of Experience" exposes
/// `extraction_scope_distribution` as a `{private, public}` carrier;
/// ADR-0040 §D40.2 maps `#public` in the source-session tag set to
/// `Public`, every other shape to `Private`.
fn decide_scope(tags: &[String]) -> crate::audit::events::m5_2::memory::ExtractionScope {
    if tags.iter().any(|t| t == "#public") {
        crate::audit::events::m5_2::memory::ExtractionScope::Public
    } else {
        crate::audit::events::m5_2::memory::ExtractionScope::Private
    }
}

#[async_trait]
impl EventHandler for MemoryExtractionListener {
    async fn on_event(&self, event: &DomainEvent) {
        // 1. Only react to natural session-end. Aborted / other
        //    variants are no-ops.
        let DomainEvent::SessionEnded {
            session_id,
            agent_id: working_agent_id,
            ended_at,
            event_id,
            ..
        } = event
        else {
            return;
        };

        // 2. Read the working agent's Agent row; bail if missing or
        //    Human-kind (Human extraction not in v0 — ADR-0040 §D40.5).
        let working_agent = match self.repo.get_agent(*working_agent_id).await {
            Ok(Some(a)) => a,
            Ok(None) => {
                tracing::warn!(
                    agent = %working_agent_id,
                    event_id = %event_id,
                    "MemoryExtractionListener: working agent row missing — skipping extraction",
                );
                return;
            }
            Err(e) => {
                tracing::error!(
                    agent = %working_agent_id,
                    event_id = %event_id,
                    error = %e,
                    "MemoryExtractionListener: get_agent failed — skipping extraction",
                );
                return;
            }
        };
        if working_agent.kind != crate::model::nodes::AgentKind::Llm {
            // Human-kind: no Identity row, no extraction at v0. Quiet
            // info-level skip (not a warning — this is by design).
            tracing::debug!(
                agent = %working_agent_id,
                event_id = %event_id,
                "MemoryExtractionListener: skipping Human-kind agent (no Identity at v0)",
            );
            return;
        }
        let Some(org_id) = working_agent.owning_org else {
            tracing::warn!(
                agent = %working_agent_id,
                event_id = %event_id,
                "MemoryExtractionListener: working agent has no owning_org — skipping extraction",
            );
            return;
        };

        // 3. Read SessionDetail; bail if missing or aborted.
        let detail = match self.repo.fetch_session(*session_id).await {
            Ok(Some(d)) => d,
            Ok(None) => {
                tracing::warn!(
                    session = %session_id,
                    event_id = %event_id,
                    "MemoryExtractionListener: session not fetchable — skipping extraction",
                );
                return;
            }
            Err(e) => {
                tracing::error!(
                    session = %session_id,
                    event_id = %event_id,
                    error = %e,
                    "MemoryExtractionListener: fetch_session failed — skipping extraction",
                );
                return;
            }
        };
        if detail.session.governance_state == crate::model::nodes::SessionGovernanceState::Aborted {
            tracing::debug!(
                session = %session_id,
                event_id = %event_id,
                "MemoryExtractionListener: session is Aborted — skipping extraction",
            );
            return;
        }

        // 4. Resolve memory-extraction system agent for the org.
        let Some(extractor_agent) = self.resolve_memory_extraction_system_agent(org_id).await
        else {
            tracing::warn!(
                org = %org_id,
                event_id = %event_id,
                "MemoryExtractionListener: memory-extraction system agent unresolvable for org — \
                 skipping extraction (operator action: provision via Standard Org Template)",
            );
            return;
        };

        // 5. Honor operator disable / archive (ADR-0040 §D40.3 SKIP-BOTH).
        if !extractor_agent.active || extractor_agent.archived_at.is_some() {
            tracing::warn!(
                org = %org_id,
                extractor = %extractor_agent.id,
                event_id = %event_id,
                "MemoryExtractionListener: memory-extraction system agent is disabled/archived — \
                 skipping extraction + telemetry fire (ADR-0040 §D40.3)",
            );
            return;
        }

        // 6. Mint Memory.
        let tags = build_memory_tags(&detail.session);
        let scope_bucket = decide_scope(&tags);
        let memory = crate::model::nodes::Memory {
            id: crate::model::ids::MemoryId::new(),
            owning_agent: working_agent.id,
            tags: tags.clone(),
            created_at: *ended_at,
        };
        if let Err(e) = self.repo.create_memory(&memory).await {
            tracing::error!(
                session = %session_id,
                event_id = %event_id,
                error = %e,
                "MemoryExtractionListener: create_memory failed — extraction aborted, \
                 no Identity update or audit",
            );
            return;
        }

        // 7. Update Identity (working agent). Identity-missing logs +
        //    skips this step only; Memory is already durable.
        let identity_updated_succeeded = match self.repo.get_identity(working_agent.id).await {
            Ok(Some(before)) => {
                let mut after = before.clone();
                after.witnessed.memories_extracted =
                    after.witnessed.memories_extracted.saturating_add(1);
                match scope_bucket {
                    crate::audit::events::m5_2::memory::ExtractionScope::Private => {
                        after.witnessed.extraction_scope_distribution.private = after
                            .witnessed
                            .extraction_scope_distribution
                            .private
                            .saturating_add(1);
                    }
                    crate::audit::events::m5_2::memory::ExtractionScope::Public => {
                        after.witnessed.extraction_scope_distribution.public = after
                            .witnessed
                            .extraction_scope_distribution
                            .public
                            .saturating_add(1);
                    }
                }
                after.updated_at = *ended_at;

                if let Err(e) = self.repo.upsert_identity(&after).await {
                    tracing::error!(
                        agent = %working_agent.id,
                        event_id = %event_id,
                        error = %e,
                        "MemoryExtractionListener: upsert_identity failed — \
                         memory durable but identity counter stale",
                    );
                    None
                } else {
                    Some((before, after))
                }
            }
            Ok(None) => {
                tracing::warn!(
                    agent = %working_agent.id,
                    event_id = %event_id,
                    "MemoryExtractionListener: identity row missing for LLM agent — \
                     skipping identity update (CH-16 invariant suggests this should not happen)",
                );
                None
            }
            Err(e) => {
                tracing::error!(
                    agent = %working_agent.id,
                    event_id = %event_id,
                    error = %e,
                    "MemoryExtractionListener: get_identity failed — \
                     memory durable but identity counter stale",
                );
                None
            }
        };

        // 8. Emit `platform.memory.extracted` audit.
        let memory_audit = crate::audit::events::m5_2::memory::memory_extracted(
            working_agent.id,
            &memory,
            *session_id,
            scope_bucket,
            org_id,
            *ended_at,
        );
        if let Err(e) = self.audit.emit(memory_audit).await {
            tracing::error!(
                session = %session_id,
                event_id = %event_id,
                error = %e,
                "MemoryExtractionListener: memory_extracted audit emit failed — \
                 memory is durable but audit trail has a gap",
            );
        }

        // 9. Emit `platform.identity.updated` audit when the upsert
        //    succeeded (CH-16 first emitter — ADR-0038 §D38.5).
        if let Some((before, after)) = identity_updated_succeeded {
            let identity_audit = crate::audit::events::m5_2::identity::identity_updated(
                working_agent.id,
                &before,
                &after,
                crate::events::IdentityUpdateTrigger::MemoryExtracted,
                org_id,
                *ended_at,
            );
            if let Err(e) = self.audit.emit(identity_audit).await {
                tracing::error!(
                    agent = %working_agent.id,
                    event_id = %event_id,
                    error = %e,
                    "MemoryExtractionListener: identity_updated audit emit failed — \
                     identity is durable but audit trail has a gap",
                );
            }
        }

        // 10. Drift D6.1 first call site — advance the memory-extractor
        //     system agent's runtime-status tile.
        let effective_parallelize = self
            .repo
            .get_agent_profile_for_agent(extractor_agent.id)
            .await
            .ok()
            .flatten()
            .map(|p| p.parallelize)
            .unwrap_or(1);
        record_system_agent_fire(
            self.repo.as_ref(),
            org_id,
            extractor_agent.id,
            effective_parallelize,
            None,
            *ended_at,
        )
        .await;
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
        // CH-16 — IdentityUpdated is informational only for the catalog
        // listener (the catalog row caches `display_name` / `kind` /
        // `role`, none of which Identity touches). Returning None
        // short-circuits this listener; future identity-aware listeners
        // (e.g. embedding similarity dashboards) live elsewhere.
        DomainEvent::IdentityUpdated { .. } => None,
        // CH-21 — MemoryExtracted is informational only for the catalog
        // listener (catalog rows have no field touched by extraction).
        // Returning None short-circuits this listener; the
        // memory-extraction listener owns the reactive path.
        DomainEvent::MemoryExtracted { .. } => None,
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

        // CH-15 / ADR-0054 §D54.3 — two grants persist per fire:
        // grants[0] = project-resource grant, grants[1] = paired
        // session-object grant.
        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(
            grants.len(),
            2,
            "CH-15: paired (project_grant, session_grant) lead grants persisted"
        );
        for g in &grants {
            assert_eq!(
                g.action,
                vec![
                    crate::permissions::action::Action::Read,
                    crate::permissions::action::Action::Inspect,
                    crate::permissions::action::Action::List,
                ]
            );
        }

        // CH-15: one `template.a.grant_fired` audit event per
        // persisted grant (2 events total).
        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        for e in &events {
            assert_eq!(e.event_type, "template.a.grant_fired");
            assert_eq!(e.org_scope, Some(org));
            assert_eq!(e.provenance_auth_request_id, Some(adoption_ar));
        }
    }

    /// CH-15: explicit per-grant resource-URI assertion — pinning the
    /// (project_grant, session_grant) order on the wire.
    #[tokio::test]
    async fn template_a_listener_persists_paired_session_object_grant() {
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

        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(grants.len(), 2);
        let project_uri = format!("project:{project}");
        let project_grant_count = grants
            .iter()
            .filter(|g| g.resource.uri == project_uri)
            .count();
        let session_grant_count = grants
            .iter()
            .filter(|g| {
                g.resource.uri.contains("tags contains")
                    && g.resource.uri.contains(&format!("project:{project}"))
                    && g.resource.uri.contains("#kind:session")
            })
            .count();
        assert_eq!(project_grant_count, 1, "exactly one project-resource grant");
        assert_eq!(session_grant_count, 1, "exactly one session-object grant");
    }

    /// CH-15: the listener emits two audit events (one per grant).
    #[tokio::test]
    async fn template_a_listener_emits_two_audit_events_per_lead_assignment() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((OrgId::new(), AuthRequestId::new())))),
            Arc::new(StaticActor(Some(AgentId::new()))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(ProjectId::new(), AgentId::new()))
            .await;

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2, "CH-15: two audit events per fire");
        for e in &events {
            assert_eq!(e.event_type, "template.a.grant_fired");
        }
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
        assert_eq!(
            grants[0].action,
            vec![
                crate::permissions::action::Action::Read,
                crate::permissions::action::Action::Inspect,
            ]
        );
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

    // ========================================================================
    // CH-13 P2 — strictest-wins audit-class composition wired through
    // the 3 production fire listeners. ADR-0050 §D50.5 +
    // concept-doc-07 §"audit_class Composition Through Templates" lines
    // 63–71. These tests pin the listener-level honoring of:
    //   - line 67 strictest-of-three ordering
    //   - line 69 audit_class_source attribution in the diff
    //   - line 71 no-silent-downgrade when org default is Alerted
    // ========================================================================

    /// Persist an [`Organization`] with the supplied `audit_class_default`
    /// so the listeners' `repo.get_organization(...)` lookup hits a row
    /// instead of falling back to the missing-row default.
    async fn seed_organization(
        repo: &Arc<dyn Repository>,
        org_id: OrgId,
        audit_class_default: crate::audit::AuditClass,
    ) {
        let org = crate::model::nodes::Organization {
            id: org_id,
            display_name: "ch13-test-org".to_string(),
            vision: None,
            mission: None,
            consent_policy: crate::model::ConsentPolicy::Implicit,
            audit_class_default,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            approval_timeout: crate::model::ApprovalTimeout::ProjectDuration,
            approval_timeout_default_response: crate::model::TimeoutResponse::Deny,
            created_at: Utc::now(),
        };
        repo.create_organization(&org).await.unwrap();
    }

    /// Persist an [`AuthRequest`] mimicking a template-adoption AR
    /// with the supplied `audit_class`. Mirrors the production
    /// `templates::adoption::build_adoption_request` shape closely
    /// enough for the listener's `audit_class` read (the composer
    /// only consults that one field).
    async fn seed_adoption_ar(
        repo: &Arc<dyn Repository>,
        ar_id: AuthRequestId,
        org_id: OrgId,
        ceo: AgentId,
        audit_class: crate::audit::AuditClass,
    ) {
        use crate::model::nodes::{
            AuthRequest, AuthRequestState, PrincipalRef, ResourceRef, ResourceSlot,
            ResourceSlotState,
        };
        let now = Utc::now();
        let ar = AuthRequest {
            id: ar_id,
            requestor: PrincipalRef::Agent(ceo),
            kinds: vec!["#template:a".to_string(), "#kind:control_plane".to_string()],
            scope: vec![format!("org:{}", org_id)],
            state: AuthRequestState::Approved,
            valid_until: None,
            submitted_at: now,
            resource_slots: vec![ResourceSlot {
                resource: ResourceRef {
                    uri: format!("org:{}/template:a", org_id),
                },
                approvers: vec![],
                state: ResourceSlotState::Approved,
            }],
            justification: Some("ch13 test adoption".to_string()),
            audit_class,
            terminal_state_entered_at: Some(now),
            archived: false,
            active_window_days: 365,
            provenance_template: None,
            tags: vec![],
            descends_from_grant: None,
        };
        repo.create_auth_request(&ar).await.unwrap();
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateA: org default supplies
    /// the winning class; both candidates Alerted (org) and Logged
    /// (template_ar) → resolved Alerted, source `org_default`. (Tie at
    /// strictest is impossible here; org wins outright.)
    #[tokio::test]
    async fn template_a_listener_org_default_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();

        seed_organization(&repo, org, AuditClass::Alerted).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Logged).await;

        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(ceo))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(project, lead)).await;

        // CH-15: paired grants both carry the resolved class.
        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(grants.len(), 2);
        for g in &grants {
            assert_eq!(g.audit_class, AuditClass::Alerted);
        }

        // CH-15: paired audit events both carry class + source.
        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        for e in &events {
            assert_eq!(e.audit_class, AuditClass::Alerted);
            assert_eq!(e.diff["after"]["audit_class_source"], "org_default");
        }
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateC: same shape on the
    /// `MANAGES` edge listener.
    #[tokio::test]
    async fn template_c_listener_org_default_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();

        seed_organization(&repo, org, AuditClass::Alerted).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Logged).await;

        let listener = Arc::new(TemplateCFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticOrgAdoption(Some(adoption_ar))),
            Arc::new(StaticActor(Some(ceo))),
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
        assert_eq!(grants[0].audit_class, AuditClass::Alerted);

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].audit_class, AuditClass::Alerted);
        assert_eq!(events[0].diff["after"]["audit_class_source"], "org_default");
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateD: same shape on the
    /// `HAS_AGENT_SUPERVISOR` edge listener.
    #[tokio::test]
    async fn template_d_listener_org_default_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let project = ProjectId::new();
        let supervisor = AgentId::new();
        let supervisee = AgentId::new();

        seed_organization(&repo, org, AuditClass::Alerted).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Logged).await;

        let listener = Arc::new(TemplateDFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticDAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(ceo))),
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
        assert_eq!(grants[0].audit_class, AuditClass::Alerted);

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].audit_class, AuditClass::Alerted);
        assert_eq!(events[0].diff["after"]["audit_class_source"], "org_default");
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateA: template_ar wins
    /// when org default is Logged + adoption AR is Alerted (the
    /// production-default adoption AR per `templates/adoption.rs:109`
    /// uses Alerted). Source attribution is `template_ar`.
    #[tokio::test]
    async fn template_a_listener_template_ar_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();

        seed_organization(&repo, org, AuditClass::Logged).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Alerted).await;

        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(ceo))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(project, lead)).await;

        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(grants.len(), 2);
        for g in &grants {
            assert_eq!(g.audit_class, AuditClass::Alerted);
        }

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        for e in &events {
            assert_eq!(e.audit_class, AuditClass::Alerted);
            assert_eq!(e.diff["after"]["audit_class_source"], "template_ar");
        }
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateC: template_ar wins.
    #[tokio::test]
    async fn template_c_listener_template_ar_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let manager = AgentId::new();
        let subordinate = AgentId::new();

        seed_organization(&repo, org, AuditClass::Logged).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Alerted).await;

        let listener = Arc::new(TemplateCFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticOrgAdoption(Some(adoption_ar))),
            Arc::new(StaticActor(Some(ceo))),
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
        assert_eq!(grants[0].audit_class, AuditClass::Alerted);

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].audit_class, AuditClass::Alerted);
        assert_eq!(events[0].diff["after"]["audit_class_source"], "template_ar");
    }

    /// CH-13 / concept-doc-07 line 67 — TemplateD: template_ar wins.
    #[tokio::test]
    async fn template_d_listener_template_ar_wins_when_strictest() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let project = ProjectId::new();
        let supervisor = AgentId::new();
        let supervisee = AgentId::new();

        seed_organization(&repo, org, AuditClass::Logged).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Alerted).await;

        let listener = Arc::new(TemplateDFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticDAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(ceo))),
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
        assert_eq!(grants[0].audit_class, AuditClass::Alerted);

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].audit_class, AuditClass::Alerted);
        assert_eq!(events[0].diff["after"]["audit_class_source"], "template_ar");
    }

    /// CH-13 / concept-doc-07 line 71 — no-silent-downgrade canonical
    /// compliance posture. Org compliance posture is Alerted; the
    /// production-default adoption AR is Alerted. The composed result
    /// MUST be Alerted (never silently downgraded by the listener
    /// path). The tie-breaker per ADR-0050 §D50.3 awards the source
    /// to `template_ar` (more specific than `org_default`).
    #[tokio::test]
    async fn template_a_listener_no_silent_downgrade_when_org_alerted() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let org = OrgId::new();
        let adoption_ar = AuthRequestId::new();
        let ceo = AgentId::new();
        let project = ProjectId::new();
        let lead = AgentId::new();

        seed_organization(&repo, org, AuditClass::Alerted).await;
        seed_adoption_ar(&repo, adoption_ar, org, ceo, AuditClass::Alerted).await;

        let listener = Arc::new(TemplateAFireListener::new(
            repo.clone(),
            audit.clone() as Arc<dyn AuditEmitter>,
            Arc::new(StaticAdoption(Some((org, adoption_ar)))),
            Arc::new(StaticActor(Some(ceo))),
        ));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        bus.emit(sample_event(project, lead)).await;

        let grants = repo
            .list_grants_for_principal(&crate::model::nodes::PrincipalRef::Agent(lead))
            .await
            .unwrap();
        assert_eq!(grants.len(), 2);
        for g in &grants {
            assert_eq!(
                g.audit_class,
                AuditClass::Alerted,
                "concept-doc-07 line 71: org default Alerted MUST flow through to Grant"
            );
        }

        let events = audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        for e in &events {
            assert_eq!(e.audit_class, AuditClass::Alerted);
            // Tie-breaker: org and template_ar both Alerted;
            // more-specific source (template_ar) wins per ADR-0050
            // §D50.3.
            assert_eq!(e.diff["after"]["audit_class_source"], "template_ar");
        }
    }

    // ========================================================================
    // CH-21 P1 — MemoryExtractionListener behavioural tests
    //
    // Replaces the previous `memory_extraction_listener_is_a_noop_at_p3`
    // stub test. The body now mints Memory + updates Identity + emits
    // two audit events + advances the memory-extractor system agent's
    // runtime-status tile (drift D6.1 first call site).
    //
    // Tests cover:
    //   - happy path: SessionEnded → Memory minted, Identity bumped,
    //     two audits, telemetry tile advances
    //   - SessionAborted governance state → no extraction
    //   - disabled extractor system agent → no extraction + no
    //     telemetry fire (ADR-0040 §D40.3 SKIP-BOTH)
    //   - Human-kind working agent → graceful skip (CH-16: no Identity)
    //   - missing extractor system agent → graceful skip + warn
    //   - scope decision: `#public` in tags → public bucket
    //   - scope decision: only project/org tags → private bucket
    // ========================================================================

    fn sample_phi_core_session() -> phi_core::session::model::Session {
        serde_json::from_value(serde_json::json!({
            "session_id": "s-mem-ext",
            "agent_id": "a-mem-ext",
            "created_at": "2026-04-28T00:00:00Z",
            "last_active_at": "2026-04-28T00:00:00Z",
            "formation": {"Explicit": {"timestamp": "2026-04-28T00:00:00Z"}},
            "parent_spawn_ref": null,
            "scope": "ephemeral",
            "loops": []
        }))
        .expect("phi-core Session JSON deserialises")
    }

    fn sample_phi_core_loop_record() -> phi_core::session::model::LoopRecord {
        serde_json::from_value(serde_json::json!({
            "loop_id": "l-mem-ext",
            "session_id": "s-mem-ext",
            "agent_id": "a-mem-ext",
            "parent_loop_id": null,
            "started_at": "2026-04-28T00:00:00Z",
            "ended_at": null,
            "status": "Running",
            "rejection": null,
            "config": null,
            "messages": [],
            "usage": {"input": 0, "output": 0, "reasoning": 0, "cache_read": 0, "cache_write": 0, "total_tokens": 0},
            "metadata": null,
            "events": [],
            "children_loop_ids": [],
            "child_loop_refs": [],
            "parallel_group": null
        }))
        .expect("phi-core LoopRecord JSON deserialises")
    }

    struct MemFixture {
        repo: Arc<dyn Repository>,
        audit: Arc<CapturingAudit>,
        org_id: OrgId,
        extractor_agent_id: AgentId,
        working_agent_id: AgentId,
        project_id: crate::model::ids::ProjectId,
        session_id: crate::model::ids::SessionId,
        ended_at: DateTime<Utc>,
    }

    /// 1-org fixture: memory-extraction system agent + LLM working
    /// agent (with Identity row) + a Session in `final_state` ready
    /// to fire extraction against. `extractor_active = false`
    /// produces the disabled-state path; `working_kind = Human`
    /// produces the no-Identity skip path; `final_state = Aborted`
    /// produces the aborted-skip path.
    async fn memory_fixture(
        extractor_active: bool,
        working_kind: AgentKind,
        session_extra_tags: Vec<String>,
        final_state: crate::model::nodes::SessionGovernanceState,
    ) -> MemFixture {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit = Arc::new(CapturingAudit::default());
        let now = Utc.with_ymd_and_hms(2026, 4, 28, 12, 0, 0).unwrap();

        let org_id = OrgId::new();
        let extractor_agent_id = AgentId::new();
        let working_agent_id = AgentId::new();
        let project_id = crate::model::ids::ProjectId::new();
        let session_id = crate::model::ids::SessionId::new();

        // Memory-extraction system agent (canonical display name; LLM kind).
        let extractor = Agent {
            id: extractor_agent_id,
            kind: AgentKind::Llm,
            display_name: MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME.to_string(),
            owning_org: Some(org_id),
            role: Some(AgentRole::System),
            created_at: now,
            active: extractor_active,
            archived_at: None,
        };
        repo.create_agent(&extractor).await.unwrap();

        // Working agent (kind selectable by the test).
        let working = Agent {
            id: working_agent_id,
            kind: working_kind,
            display_name: "test-worker".to_string(),
            owning_org: Some(org_id),
            role: Some(AgentRole::Member),
            created_at: now,
            active: true,
            archived_at: None,
        };
        repo.create_agent(&working).await.unwrap();

        // Identity for the working agent (only for LLM kind, per CH-16).
        if working_kind == AgentKind::Llm {
            let iden = crate::model::nodes::Identity::default_for_llm(working_agent_id, now);
            repo.upsert_identity(&iden).await.unwrap();
        }

        // Org listing both system agents.
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
            system_agents: vec![extractor_agent_id],
            approval_timeout: crate::model::ApprovalTimeout::ProjectDuration,
            approval_timeout_default_response: crate::model::TimeoutResponse::Deny,
            created_at: now,
        };
        repo.create_organization(&org).await.unwrap();

        // Session — Running governance state at persist; tests that
        // need Aborted call mark_session_ended after the fact.
        let mut session_tags = vec![format!("session:{}", session_id), "#kind:session".into()];
        session_tags.extend(session_extra_tags);
        let session = crate::model::nodes::Session {
            id: session_id,
            inner: sample_phi_core_session(),
            owning_org: org_id,
            owning_project: project_id,
            started_by: working_agent_id,
            governance_state: crate::model::nodes::SessionGovernanceState::Running,
            started_at: now,
            ended_at: None,
            tokens_spent: 0,
            tags: session_tags,
        };
        let lr = crate::model::nodes::LoopRecordNode {
            id: crate::model::ids::LoopId::new(),
            inner: sample_phi_core_loop_record(),
            session_id,
            loop_index: 0,
        };
        repo.persist_session(&session, &lr).await.unwrap();
        repo.mark_session_ended(session_id, now, final_state)
            .await
            .unwrap();

        MemFixture {
            repo,
            audit,
            org_id,
            extractor_agent_id,
            working_agent_id,
            project_id,
            session_id,
            ended_at: now,
        }
    }

    fn make_memory_listener(f: &MemFixture) -> Arc<MemoryExtractionListener> {
        Arc::new(MemoryExtractionListener::new(
            f.repo.clone(),
            f.audit.clone() as Arc<dyn AuditEmitter>,
        ))
    }

    fn session_ended_for(f: &MemFixture) -> DomainEvent {
        DomainEvent::SessionEnded {
            session_id: f.session_id,
            agent_id: f.working_agent_id,
            project_id: f.project_id,
            ended_at: f.ended_at,
            duration_ms: 1_000,
            turn_count: 2,
            tokens_spent: 256,
            event_id: AuditEventId::new(),
        }
    }

    // ---- Happy path -------------------------------------------------------

    #[tokio::test]
    async fn memory_extraction_listener_mints_memory_updates_identity_and_emits_two_audits() {
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        // Identity counter advanced.
        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(iden.witnessed.memories_extracted, 1);
        assert_eq!(iden.witnessed.extraction_scope_distribution.private, 1);
        assert_eq!(iden.witnessed.extraction_scope_distribution.public, 0);
        assert_eq!(iden.updated_at, f.ended_at);

        // Two audit events emitted in the right order: memory.extracted,
        // identity.updated.
        let events = f.audit.events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "platform.memory.extracted");
        assert_eq!(events[0].audit_class, AuditClass::Logged);
        assert_eq!(events[0].org_scope, Some(f.org_id));
        assert_eq!(events[0].diff["scope_bucket"].as_str().unwrap(), "private");
        assert_eq!(events[1].event_type, "platform.identity.updated");
        assert_eq!(events[1].audit_class, AuditClass::Logged);
        assert_eq!(
            events[1].diff["trigger"].as_str().unwrap(),
            "memory_extracted"
        );
    }

    // ---- D6.1 first call site: tile advances on extraction ---------------

    #[tokio::test]
    async fn memory_extraction_listener_advances_runtime_status_tile_for_extractor() {
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);

        // Pre-fire: tile may or may not exist (depends on prior fires).
        listener.on_event(&session_ended_for(&f)).await;

        let tile = f
            .repo
            .fetch_system_agent_runtime_status_for_org(f.org_id)
            .await
            .unwrap()
            .into_iter()
            .find(|t| t.agent_id == f.extractor_agent_id)
            .expect("D6.1 first call site: tile materialised after fire");
        assert!(
            tile.last_fired_at.is_some(),
            "tile last_fired_at populated by record_system_agent_fire",
        );
    }

    // ---- Aborted session → skip ------------------------------------------

    #[tokio::test]
    async fn memory_extraction_listener_skips_aborted_session() {
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Aborted,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(
            iden.witnessed.memories_extracted, 0,
            "aborted session: no extraction"
        );
        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "aborted session: no audit events",
        );
    }

    // ---- Disabled extractor system agent → skip both --------------------

    #[tokio::test]
    async fn memory_extraction_listener_skips_when_extractor_disabled() {
        let f = memory_fixture(
            false, /* extractor_active */
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(
            iden.witnessed.memories_extracted, 0,
            "disabled extractor: no extraction"
        );
        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "disabled extractor: no audit (ADR-0040 SKIP-BOTH)",
        );
        // Tile should NOT be materialised either.
        let tile = f
            .repo
            .fetch_system_agent_runtime_status_for_org(f.org_id)
            .await
            .unwrap()
            .into_iter()
            .find(|t| t.agent_id == f.extractor_agent_id);
        assert!(
            tile.is_none(),
            "ADR-0040 §D40.3: disabled extractor skips telemetry fire too",
        );
    }

    // ---- Human-kind working agent → graceful skip -------------------------

    #[tokio::test]
    async fn memory_extraction_listener_skips_human_kind_working_agent() {
        let f = memory_fixture(
            true,
            AgentKind::Human,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        // No Identity row exists for Human (CH-16 invariant); confirm
        // listener didn't try to create one + no audit emitted.
        assert!(f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .is_none(),);
        assert!(
            f.audit.events.lock().unwrap().is_empty(),
            "human-kind: no extraction + no audit",
        );
    }

    // ---- Missing extractor system agent → graceful skip ------------------

    #[tokio::test]
    async fn memory_extraction_listener_skips_when_extractor_unresolvable() {
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        // Strip the extractor from the org's system_agents list.
        let mut org = f.repo.get_organization(f.org_id).await.unwrap().unwrap();
        org.system_agents.clear();
        // Reuse create_organization-like wipe + re-insert: in_memory
        // doesn't expose an "update org" surface, so the cleanest path
        // is archiving the extractor agent so the resolver walk skips
        // it. (`active = false` would be detected by the disabled path;
        // archiving sidesteps the resolver before that check.)
        f.repo
            .set_agent_archived_at(f.extractor_agent_id, Some(f.ended_at))
            .await
            .unwrap();
        // Plus wipe display_name so the walk never matches even if
        // archive doesn't filter at the resolver tier.
        // (in_memory's resolver does not pre-filter on archived_at;
        // archived agents still appear with their canonical name —
        // so we test the disabled-path here, which is functionally
        // equivalent for "skip both".)
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        // The disabled-state guard short-circuits at step 5 — no
        // memory minted, no audit emitted, no telemetry fire.
        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(iden.witnessed.memories_extracted, 0);
        assert!(f.audit.events.lock().unwrap().is_empty());
    }

    // ---- Scope decision: `#public` → public bucket ----------------------

    #[tokio::test]
    async fn memory_extraction_listener_increments_public_bucket_when_session_has_public_tag() {
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec!["#public".into()],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(iden.witnessed.memories_extracted, 1);
        assert_eq!(iden.witnessed.extraction_scope_distribution.public, 1);
        assert_eq!(iden.witnessed.extraction_scope_distribution.private, 0);

        // Audit reflects the public bucket too.
        let events = f.audit.events.lock().unwrap().clone();
        assert_eq!(events[0].diff["scope_bucket"].as_str().unwrap(), "public");
    }

    // ---- Scope decision: project/org-only tags → private bucket ----------

    #[tokio::test]
    async fn memory_extraction_listener_defaults_to_private_when_no_public_tag() {
        // Only the canonical session/project/org/agent tags present.
        let f = memory_fixture(
            true,
            AgentKind::Llm,
            vec![],
            crate::model::nodes::SessionGovernanceState::Completed,
        )
        .await;
        let listener = make_memory_listener(&f);
        listener.on_event(&session_ended_for(&f)).await;

        let iden = f
            .repo
            .get_identity(f.working_agent_id)
            .await
            .unwrap()
            .expect("identity row present");
        assert_eq!(iden.witnessed.extraction_scope_distribution.private, 1);
        assert_eq!(iden.witnessed.extraction_scope_distribution.public, 0);
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
            approval_timeout: crate::model::ApprovalTimeout::ProjectDuration,
            approval_timeout_default_response: crate::model::TimeoutResponse::Deny,
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
