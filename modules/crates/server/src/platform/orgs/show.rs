//! `GET /api/v0/orgs/:id` — single-org detail (dashboard-preview shape).
//!
//! ## phi-core leverage
//!
//! Q1 **none**, Q2 **yes — via `defaults_snapshot`**, Q3 **none new**:
//!
//! - **Q2**: The detail payload includes `Organization.defaults_snapshot`
//!   verbatim, which wraps 4 phi-core types. This is **intentional** —
//!   operators drilling into a specific org want to see the frozen
//!   phi-core config (execution limits, retry, agent-profile defaults).
//!   Contrast with `list.rs` which omits the snapshot for summary compactness.
//! - The schema-snapshot test in P5's dashboard phase **will allow**
//!   snapshot fields in the show payload when that phase lands — this
//!   endpoint's JSON contract is stable across M3→M4 per D11/Q6.
//!
//! ## Scope at M3
//!
//! Returns the full `Organization` struct + `member_count` +
//! `project_count` + `adopted_template_count`. M3/P5's dashboard
//! endpoint returns a richer aggregate (recent audit events, token
//! budget utilisation, CTA cards); this endpoint stays minimal so CLI
//! `org show` + web's org-preview panel can both consume it.

use std::collections::HashSet;
use std::sync::Arc;

use domain::auth_requests::access::{check_auth_request_access, IntendedOp};
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Organization, PrincipalRef};
use domain::permissions::catalogue::StaticCatalogue;
use domain::permissions::manifest::{CheckContext, ConsentIndex, Manifest, ToolCall};
use domain::permissions::metrics::NoopMetrics;
use domain::permissions::Action;
use domain::repository::Repository;

use crate::handler_support::permission::check_permission;

use super::OrgError;

#[derive(Debug, Clone)]
pub struct OrganizationDetail {
    pub organization: Organization,
    pub member_count: usize,
    pub project_count: usize,
    pub adopted_template_count: usize,
}

/// Returns `Ok(None)` when the org is not found — the HTTP layer
/// maps that to 404.
///
/// `viewer` is used to apply the per-state access matrix (CH-18 /
/// ADR-0056 §D56.5 / F3.B.list-filter.a) to the adoption-AR list:
/// `adopted_template_count` reflects only the ARs the viewer is
/// authorised to read. List-side reads are passive observation; no
/// audit-event is emitted per filtered AR (silent post-filter — same
/// UX as `archived_at IS NOT NULL` filtering).
///
/// CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — the viewer is
/// gated through the Permission Check engine on `org:<id>`
/// (Action::Inspect) before the org body is returned. The CH-25
/// synth-owner-grant rule (widened to 4-verb scope at CH-27 / ADR-0062
/// §D62.2) resolves Allow for the owning Agent; strangers receive
/// Denied → propagated as `OrgError::PermissionDenied` mapped to HTTP
/// 403 NO_GRANTS_HELD per CH-25 wire convention (`denial_to_api_error`).
/// The bespoke AR-filter on the adoptions list stays as defence-in-depth.
pub async fn show_organization(
    repo: Arc<dyn Repository>,
    id: OrgId,
    viewer: AgentId,
) -> Result<Option<OrganizationDetail>, OrgError> {
    let org = match repo
        .get_organization(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?
    {
        Some(o) => o,
        None => return Ok(None),
    };

    // CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — engine-routed
    // Inspect check on `org:<id>`. **BLOCKING** as of CH-27: engine deny
    // propagates as `OrgError::PermissionDenied` → HTTP 403 NO_GRANTS_HELD.
    // The CH-25 synth-owner-grant rule (widened to 4-verb scope at CH-27
    // / ADR-0062 §D62.2) resolves Allow for the owning Agent; cross-org
    // strangers resolve Deny. The M3 per-AR filter at line ~80 below
    // stays as defence-in-depth.
    is_inspect_allowed(&*repo, viewer, id).await?;

    let members = repo
        .list_agents_in_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let projects = repo
        .list_projects_in_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    // CH-18 / ADR-0056 §D56.5 + F3.B.list-filter.a — silent post-filter
    // on adoption ARs (same rationale as dashboard.rs:273,293). Kept as
    // defence-in-depth alongside the new engine gate above.
    let adoptions: Vec<_> = repo
        .list_adoption_auth_requests_for_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?
        .into_iter()
        .filter(|ar| {
            check_auth_request_access(ar, &PrincipalRef::Agent(viewer), IntendedOp::Read).is_ok()
        })
        .collect();
    Ok(Some(OrganizationDetail {
        organization: org,
        member_count: members.len(),
        project_count: projects.len(),
        adopted_template_count: adoptions.len(),
    }))
}

/// CH-26 / ADR-0061 §D61.5 + CH-27 / ADR-0062 §D62.1 — engine-routed
/// Inspect gate for org show. **BLOCKING** as of CH-27.
///
/// Builds the standard CheckContext shape used across CH-25 + CH-26 +
/// CH-27 handler refactors: explicit grants + CH-25 owned-resource
/// slices + catalogue seeded with the instance URI. Returns `Ok(())`
/// iff the engine returns Allowed; engine denials propagate as
/// `OrgError::PermissionDenied` mapped to HTTP 403 NO_GRANTS_HELD per
/// CH-25 wire convention.
async fn is_inspect_allowed(
    repo: &dyn Repository,
    viewer: AgentId,
    org_id: OrgId,
) -> Result<(), OrgError> {
    let uri = format!("org:{}", org_id);
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
    let mut catalogue = StaticCatalogue::empty();
    catalogue.seed(Some(org_id), &uri);
    let consents = ConsentIndex::empty();
    let gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();
    let ctx = CheckContext {
        agent: viewer,
        current_org: Some(org_id),
        current_project: None,
        current_session: None,
        agent_grants: &agent_grants,
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
        agent_owned_orgs: &agent_owned_orgs,
        agent_owned_projects: &agent_owned_projects,
        call: ToolCall {
            target_uri: uri.clone(),
            target_tags: vec![format!("organization:{}", org_id)],
            ..Default::default()
        },
    };
    let manifest = Manifest {
        actions: vec![Action::Inspect],
        resource: vec!["identity_principal".to_string()],
        ..Default::default()
    };
    check_permission(&ctx, &manifest, &NoopMetrics)
        .map(|_resolved| ())
        .map_err(|api_err| OrgError::PermissionDenied(api_err.message))
}
