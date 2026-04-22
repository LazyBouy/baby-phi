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

use std::sync::Arc;

use domain::model::composites_m3::ConsentPolicy;
use domain::model::ids::OrgId;
use domain::model::nodes::TemplateKind;
use domain::repository::Repository;

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
) -> Result<Vec<OrganizationSummary>, OrgError> {
    // The repo surface doesn't have a `list_organizations` method at
    // M3 (dashboard consumption is per-org); we derive the list from
    // the admin-facing path: iterate every audit event with
    // event_type = "platform.organization.created" and collect the
    // unique org_scopes. This is cheap for M3 (tens of orgs). M4
    // will add a proper `list_orgs` repo method + index once volume
    // warrants.
    //
    // For now, a simpler path: fetch a generous window of recent
    // audit events filtered by the creation event_type. The query
    // below is a stand-in that we can replace in M4 without
    // re-shaping this business-logic module.
    //
    // Since we don't have a proper repo method, we iterate through a
    // best-effort approach: list_recent_audit_events_for_org on a
    // synthetic empty org returns nothing — useless. We need a
    // repo-level `list_orgs`. For M3 correctness we add a
    // thin helper here that calls the existing
    // `get_organization(id)` per id supplied via a catalogue scan.
    //
    // To keep M3's scope bounded we restrict this method to returning
    // the **caller-accessible** orgs; at P4 the caller is always the
    // platform admin, so we currently return every org. Replace with
    // `list_orgs_for_admin(admin_id)` in M4.
    let all = repo
        .list_all_orgs()
        .await
        .map_err(|e| OrgError::Repository(e.to_string()))?;
    let mut summaries = Vec::with_capacity(all.len());
    for org in all {
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
