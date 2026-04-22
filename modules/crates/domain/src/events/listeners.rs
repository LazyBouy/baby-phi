//! Domain-event listener implementations.
//!
//! At M4/P3 this module ships a single listener —
//! [`TemplateAFireListener`] — that reacts to
//! [`DomainEvent::HasLeadEdgeCreated`] by minting the lead's grant
//! (via the M4/P2 pure-fn [`crate::templates::a::fire_grant_on_lead_assignment`]),
//! persisting it, and emitting the companion audit event.
//!
//! Future reactive listeners (M5+: memory-extractor, agent-catalog,
//! etc.) plug into the same bus.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::audit::AuditEmitter;
use crate::events::{DomainEvent, EventHandler};
use crate::model::ids::{AgentId, AuthRequestId, OrgId};
use crate::templates::a::{fire_grant_on_lead_assignment, FireArgs};
use crate::Repository;

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
        // Extensible match — future variants can be handled here.
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
}
