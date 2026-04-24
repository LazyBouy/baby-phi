//! Domain-event listener implementations.
//!
//! At M4/P3 this module shipped a single listener —
//! [`TemplateAFireListener`] — that reacts to
//! [`DomainEvent::HasLeadEdgeCreated`] by minting the lead's grant.
//!
//! M5/P3 extends the set with four more:
//! - [`TemplateCFireListener`] — full body; reacts to
//!   [`DomainEvent::ManagesEdgeCreated`], calls
//!   [`crate::templates::c::fire_grant_on_manages_edge`], persists the
//!   Grant, emits `template.c.grant_fired` audit.
//! - [`TemplateDFireListener`] — full body; reacts to
//!   [`DomainEvent::HasAgentSupervisorEdgeCreated`], calls
//!   [`crate::templates::d::fire_grant_on_has_agent_supervisor`].
//! - [`MemoryExtractionListener`] — **stub at M5/P3**. Subscribes to
//!   [`DomainEvent::SessionEnded`]; body lands at M5/P8.
//! - [`AgentCatalogListener`] — **stub at M5/P3**. Subscribes to 8
//!   DomainEvent variants; body lands at M5/P8.
//!
//! All five listeners share the same fail-safe semantics (ADR-0028):
//! events emit AFTER the owning compound-tx commits; listener errors
//! log + drop (no auto-retry at M5; M7b adds the retry fabric).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::audit::AuditEmitter;
use crate::events::{DomainEvent, EventHandler};
use crate::model::composites_m5::SystemAgentRuntimeStatus;
use crate::model::ids::{AgentId, AuthRequestId, OrgId, ProjectId, SystemAgentRuntimeStatusId};
use crate::templates::a::{fire_grant_on_lead_assignment, FireArgs};
use crate::Repository;

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
    let status = SystemAgentRuntimeStatus {
        id: SystemAgentRuntimeStatusId::new(),
        agent_id: agent,
        owning_org: org,
        queue_depth: 0, // P8 bodies will compute; P6 helper seeds idle.
        last_fired_at: Some(now),
        effective_parallelize,
        last_error,
        updated_at: now,
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

/// Agent-catalog listener — **stub at M5/P3**.
///
/// Subscribes to the 8 DomainEvent variants that drive catalog
/// upserts (`AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged`,
/// `HasLeadEdgeCreated`, `ManagesEdgeCreated`,
/// `HasAgentSupervisorEdgeCreated`, `SessionStarted`, `SessionEnded`).
/// The upsert body lands at M5/P8.
pub struct AgentCatalogListener {
    _repo: Arc<dyn Repository>,
    _audit: Arc<dyn AuditEmitter>,
}

impl AgentCatalogListener {
    pub fn new(repo: Arc<dyn Repository>, audit: Arc<dyn AuditEmitter>) -> Self {
        Self {
            _repo: repo,
            _audit: audit,
        }
    }
}

#[async_trait]
impl EventHandler for AgentCatalogListener {
    async fn on_event(&self, event: &DomainEvent) {
        // All handled variants are recognised at P3 so the log line
        // confirms the wiring; the actual catalog upsert ships at P8.
        match event {
            DomainEvent::AgentCreated { .. }
            | DomainEvent::AgentArchived { .. }
            | DomainEvent::HasProfileEdgeChanged { .. }
            | DomainEvent::HasLeadEdgeCreated { .. }
            | DomainEvent::ManagesEdgeCreated { .. }
            | DomainEvent::HasAgentSupervisorEdgeCreated { .. }
            | DomainEvent::SessionStarted { .. }
            | DomainEvent::SessionEnded { .. } => {
                tracing::debug!(
                    event_kind = event.kind(),
                    event_id = %event.event_id(),
                    "AgentCatalogListener (stub): upsert body ships at M5/P8",
                );
            }
            DomainEvent::SessionAborted { .. } => {
                // Not in the P8 trigger set per the plan.
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

    #[tokio::test]
    async fn agent_catalog_listener_is_a_noop_at_p3() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let audit: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
        let listener = Arc::new(AgentCatalogListener::new(repo.clone(), audit.clone()));
        let bus = InProcessEventBus::new();
        bus.subscribe(listener);
        // Three of the 8 subscribed variants.
        bus.emit(DomainEvent::AgentCreated {
            agent_id: AgentId::new(),
            owning_org: OrgId::new(),
            agent_kind: crate::model::nodes::AgentKind::Llm,
            role: None,
            at: Utc::now(),
            event_id: AuditEventId::new(),
        })
        .await;
        bus.emit(DomainEvent::AgentArchived {
            agent_id: AgentId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        })
        .await;
        bus.emit(DomainEvent::HasProfileEdgeChanged {
            agent_id: AgentId::new(),
            old_profile_id: None,
            new_profile_id: crate::model::ids::NodeId::new(),
            at: Utc::now(),
            event_id: AuditEventId::new(),
        })
        .await;
    }
}
