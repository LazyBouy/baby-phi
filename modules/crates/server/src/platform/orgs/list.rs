//! `GET /api/v0/orgs` — list every org the caller can see.
//!
//! ## phi-core leverage
//!
//! Q1 **none**, Q2 **none**, Q3 **none** — the return type is
//! `Vec<OrganizationSummary>` with phi-native governance fields
//! only (`id`, `display_name`, `consent_policy`,
//! `authority_templates_enabled`, member count). Per the leverage
//! checklist §5, the summary payload **deliberately omits** the
//! `defaults_snapshot` to keep list responses small and to avoid
//! surfacing phi-core internals on the index view; the detail
//! endpoint (`show.rs`) carries the full snapshot.
//!
//! ## Scope at M3
//!
//! No filtering / pagination yet — M3's dashboard fits 10s of orgs
//! comfortably on a single page, and pagination would need a stable
//! cursor token that M4 can design once project volume forces it.

use std::collections::HashSet;
use std::sync::Arc;

use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{PrincipalRef, TemplateKind};
use domain::permissions::catalogue::StaticCatalogue;
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::permissions::Action;
use domain::repository::Repository;

use crate::handler_support::permission::check_permission;

use super::OrgError;

#[derive(Debug, Clone)]
pub struct OrganizationSummary {
    pub id: OrgId,
    pub display_name: String,
    pub consent_policy: ConsentPolicy,
    pub authority_templates_enabled: Vec<TemplateKind>,
    pub member_count: usize,
}

pub async fn list_organizations(
    repo: Arc<dyn Repository>,
    viewer: AgentId,
) -> Result<Vec<OrganizationSummary>, OrgError> {
    // The repo surface doesn't have a `list_organizations` method at
    // M3 (dashboard consumption is per-org); we derive the list from
    // the admin-facing path: iterate every audit event with
    // event_type = "platform.organization.created" and collect the
    // unique org_scopes. This is cheap for M3 (tens of orgs). M4
    // will add a proper `list_orgs` repo method + index once volume
    // warrants.
    //
    // CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — engine-routed
    // Observe check per-row. **BLOCKING-FILTER** as of CH-27: rows where
    // the engine returns Deny are dropped from the returned summary list
    // (the engine result IS the filter). The CH-25 synth-owner-grant
    // rule (widened to 4-verb scope at CH-27 / ADR-0062 §D62.2) resolves
    // Allow for the owning Agent of each org; rows the viewer does not
    // own and has no explicit Observe grant on are silently filtered out
    // (silent post-filter — same UX shape as the existing AR-list
    // post-filter at `show_organization`).
    let all = repo
        .list_all_orgs()
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    // Pre-load CH-25 owned-resource slices ONCE so the per-row engine
    // invocation doesn't re-issue the same repo queries N times.
    let agent_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(viewer))
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let agent_owned_orgs = repo
        .list_agent_owned_orgs(viewer)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let agent_owned_projects = repo
        .list_agent_owned_projects(viewer)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let mut summaries = Vec::with_capacity(all.len());
    for org in all {
        // CH-27 / ADR-0062 §D62.1 — engine result IS the filter; rows
        // where the engine returns Deny are skipped silently.
        if !engine_observe_allowed(
            viewer,
            org.id,
            &agent_grants,
            &agent_owned_orgs,
            &agent_owned_projects,
        ) {
            continue;
        }
        let members = repo
            .list_agents_in_org(org.id)
            .await
            .map_err(|e| OrgError::Repository(e.to_string()))?;
        summaries.push(OrganizationSummary {
            id: org.id,
            display_name: org.display_name,
            consent_policy: org.consent_policy,
            authority_templates_enabled: org.authority_templates_enabled,
            member_count: members.len(),
        });
    }
    Ok(summaries)
}

/// CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — synchronous
/// per-row engine gate. Builds CheckContext from pre-loaded slices
/// (callsite is responsible for fetching them once); invokes
/// `check_permission` on `org:<id>` with `Action::Observe`. **CH-27**:
/// the bool result is now a **filter** — the caller drops rows where
/// this returns `false` (silent post-filter; same UX as the AR-list
/// post-filter shape).
fn engine_observe_allowed(
    viewer: AgentId,
    org_id: OrgId,
    agent_grants: &[domain::model::nodes::Grant],
    agent_owned_orgs: &[OrgId],
    agent_owned_projects: &[domain::model::ids::ProjectId],
) -> bool {
    let uri = format!("org:{}", org_id);
    let mut catalogue = StaticCatalogue::empty();
    catalogue.seed(Some(org_id), &uri);
    let consents = ConsentIndex::empty();
    let gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();
    let ctx = CheckContext {
        agent: viewer,
        current_org: Some(org_id),
        current_project: None,
        current_session: None,
        agent_grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        session_org_tags: &[],
        session_project_tags: &[],
        agent_owned_orgs,
        agent_owned_projects,
        call: ToolCall {
            target_uri: uri.clone(),
            target_tags: vec![format!("organization:{}", org_id)],
            ..Default::default()
        },
    };
    let manifest = Manifest {
        actions: vec![Action::Observe],
        resource: vec!["identity_principal".to_string()],
        ..Default::default()
    };
    check_permission(&ctx, &manifest, &NoopMetrics).is_ok()
}
