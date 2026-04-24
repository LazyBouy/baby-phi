//! Repo-backed resolvers for the Template A / C / D fire-listeners.
//!
//! - [`RepoAdoptionArResolver`] (Template A, project-scoped) walks
//!   `project → belongs_to → org` then calls
//!   `Repository::list_adoption_auth_requests_for_org` and filters
//!   for the Template A entry.
//! - [`RepoTemplateCAdoptionArResolver`] (Template C, org-scoped)
//!   looks up the Template C adoption AR directly by org.
//! - [`RepoTemplateDAdoptionArResolver`] (Template D, project-scoped)
//!   mirrors the Template A path but filters for the Template D
//!   entry.
//! - [`RepoActorResolver`] picks the org's CEO (the first
//!   `Human`-kind agent whose `owning_org == Some(org)`). A separate
//!   service-account path is reserved for M7+.
//!
//! All resolvers accept `Arc<dyn Repository>` so the wiring in
//! `main.rs` + tests can share one repo handle.

use async_trait::async_trait;
use std::sync::Arc;

use domain::events::{
    ActorResolver, AdoptionArResolver, TemplateCAdoptionArResolver, TemplateDAdoptionArResolver,
};
use domain::model::ids::{AgentId, AuthRequestId, OrgId, ProjectId};
use domain::model::nodes::{AgentKind, TemplateKind};
use domain::Repository;

pub struct RepoAdoptionArResolver {
    repo: Arc<dyn Repository>,
}

impl RepoAdoptionArResolver {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AdoptionArResolver for RepoAdoptionArResolver {
    async fn resolve(&self, project: ProjectId) -> Option<(OrgId, AuthRequestId)> {
        // Simple linear scan at M4 volume (tens of orgs): for every
        // org, check if it lists the project under its
        // `BELONGS_TO` reverse-walk + look up the Template-A
        // adoption AR. A dedicated edge-walk repo method can replace
        // this at M7+ if org count grows.
        let orgs = self.repo.list_all_orgs().await.ok()?;
        for org in orgs {
            let projects = self.repo.list_projects_in_org(org.id).await.ok()?;
            if !projects.iter().any(|p| p.id == project) {
                continue;
            }
            let ars = self
                .repo
                .list_adoption_auth_requests_for_org(org.id)
                .await
                .ok()?;
            for ar in ars {
                let is_template_a = ar.resource_slots.first().is_some_and(|rs| {
                    rs.resource
                        .uri
                        .ends_with(&format!("/template:{}", TemplateKind::A.as_str()))
                });
                if is_template_a {
                    return Some((org.id, ar.id));
                }
            }
        }
        None
    }
}

pub struct RepoActorResolver {
    repo: Arc<dyn Repository>,
}

impl RepoActorResolver {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ActorResolver for RepoActorResolver {
    async fn resolve(&self, org: OrgId) -> Option<AgentId> {
        // Pick the first Human-kind member — the org's CEO (at
        // org-creation time, the CEO is the only Human-kind member).
        // M7+ may introduce a dedicated "org service account" that
        // takes precedence.
        let agents = self.repo.list_agents_in_org(org).await.ok()?;
        agents
            .into_iter()
            .find(|a| a.kind == AgentKind::Human)
            .map(|a| a.id)
    }
}

// ===========================================================================
// M5/P3 — Template C / D adoption-AR resolvers
// ===========================================================================

/// Resolve the Template C adoption AR for an org.
///
/// Org-scoped (Template C fires on `MANAGES` edges within an org's
/// reporting tree — no project hop required). Filters
/// `list_adoption_auth_requests_for_org` by `TemplateKind::C`.
pub struct RepoTemplateCAdoptionArResolver {
    repo: Arc<dyn Repository>,
}

impl RepoTemplateCAdoptionArResolver {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TemplateCAdoptionArResolver for RepoTemplateCAdoptionArResolver {
    async fn resolve(&self, org: OrgId) -> Option<AuthRequestId> {
        let ars = self
            .repo
            .list_adoption_auth_requests_for_org(org)
            .await
            .ok()?;
        ars.into_iter()
            .find(|ar| {
                ar.resource_slots.first().is_some_and(|rs| {
                    rs.resource
                        .uri
                        .ends_with(&format!("/template:{}", TemplateKind::C.as_str()))
                })
            })
            .map(|ar| ar.id)
    }
}

/// Resolve the Template D adoption AR for the org that owns a
/// project. Mirrors [`RepoAdoptionArResolver`] but filters for
/// Template D.
pub struct RepoTemplateDAdoptionArResolver {
    repo: Arc<dyn Repository>,
}

impl RepoTemplateDAdoptionArResolver {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TemplateDAdoptionArResolver for RepoTemplateDAdoptionArResolver {
    async fn resolve(&self, project: ProjectId) -> Option<(OrgId, AuthRequestId)> {
        let orgs = self.repo.list_all_orgs().await.ok()?;
        for org in orgs {
            let projects = self.repo.list_projects_in_org(org.id).await.ok()?;
            if !projects.iter().any(|p| p.id == project) {
                continue;
            }
            let ars = self
                .repo
                .list_adoption_auth_requests_for_org(org.id)
                .await
                .ok()?;
            for ar in ars {
                let is_template_d = ar.resource_slots.first().is_some_and(|rs| {
                    rs.resource
                        .uri
                        .ends_with(&format!("/template:{}", TemplateKind::D.as_str()))
                });
                if is_template_d {
                    return Some((org.id, ar.id));
                }
            }
        }
        None
    }
}
