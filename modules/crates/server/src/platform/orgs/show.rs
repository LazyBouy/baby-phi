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

use std::sync::Arc;

use domain::model::ids::OrgId;
use domain::model::nodes::Organization;
use domain::repository::Repository;

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
pub async fn show_organization(
    repo: Arc<dyn Repository>,
    id: OrgId,
) -> Result<Option<OrganizationDetail>, OrgError> {
    let org = match repo
        .get_organization(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?
    {
        Some(o) => o,
        None => return Ok(None),
    };
    let members = repo
        .list_agents_in_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let projects = repo
        .list_projects_in_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let adoptions = repo
        .list_adoption_auth_requests_for_org(id)
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    Ok(Some(OrganizationDetail {
        organization: org,
        member_count: members.len(),
        project_count: projects.len(),
        adopted_template_count: adoptions.len(),
    }))
}
