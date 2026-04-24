//! GET `/api/v0/orgs/:org/authority-templates` — page 12's
//! 4-bucket listing (pending / active / revoked / available).

use std::sync::Arc;

use domain::model::ids::OrgId;
use domain::model::nodes::{AuthRequestState, TemplateKind};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{find_adoption_ar, is_adoptable_kind, TemplateError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateRow {
    pub kind: TemplateKind,
    pub auth_request_id: domain::model::ids::AuthRequestId,
    /// Number of grants ever fired under this adoption (across
    /// active + revoked). Pre-cascade figure — Page 12's revoke
    /// confirmation dialog reads this.
    pub fires_count: u32,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesListing {
    pub pending: Vec<TemplateRow>,
    pub active: Vec<TemplateRow>,
    pub revoked: Vec<TemplateRow>,
    /// Kind slugs (as strings — `"c"`, `"d"`, `"e-always"`) the
    /// UI can render as "Available" CTAs. Per R-ADMIN-12-R3
    /// Template E surfaces as a fixed `"e-always"` sentinel.
    pub available: Vec<String>,
}

pub async fn list_templates_for_org(
    repo: Arc<dyn Repository>,
    org: OrgId,
) -> Result<TemplatesListing, TemplateError> {
    // Verify org exists; a missing org surfaces as 404 rather
    // than an empty payload.
    let _ = repo
        .get_organization(org)
        .await?
        .ok_or(TemplateError::OrgNotFound(org))?;

    let mut pending: Vec<TemplateRow> = Vec::new();
    let mut active: Vec<TemplateRow> = Vec::new();
    let mut revoked: Vec<TemplateRow> = Vec::new();
    let mut adopted_kinds: std::collections::BTreeSet<TemplateKind> =
        std::collections::BTreeSet::new();

    for kind in [
        TemplateKind::A,
        TemplateKind::B,
        TemplateKind::C,
        TemplateKind::D,
    ] {
        let Some(ar) = find_adoption_ar(&*repo, org, kind).await? else {
            continue;
        };
        let fires_count = repo.count_grants_fired_by_adoption(ar.id).await?;
        let row = TemplateRow {
            kind,
            auth_request_id: ar.id,
            fires_count,
            submitted_at: ar.submitted_at,
        };
        match ar.state {
            AuthRequestState::Approved => {
                adopted_kinds.insert(kind);
                active.push(row);
            }
            AuthRequestState::Revoked => {
                adopted_kinds.insert(kind);
                revoked.push(row);
            }
            AuthRequestState::Pending | AuthRequestState::Partial => {
                adopted_kinds.insert(kind);
                pending.push(row);
            }
            // Denied / Expired / Cancelled → the adoption was
            // abandoned; the kind is still available to be
            // re-adopted so NOT added to adopted_kinds.
            _ => {}
        }
    }

    // Every A-D kind NOT in adopted_kinds is available.
    let mut available: Vec<String> = TemplateKind::ALL
        .iter()
        .filter(|k| is_adoptable_kind(**k) && !adopted_kinds.contains(*k))
        .map(|k| k.as_str().to_string())
        .collect();
    // R-ADMIN-12-R3: Template E surfaces as a fixed sentinel
    // (always available, never adopted).
    available.push("e-always".to_string());

    // Sort each bucket newest-first by submitted_at for UI
    // consistency.
    pending.sort_by_key(|r| std::cmp::Reverse(r.submitted_at));
    active.sort_by_key(|r| std::cmp::Reverse(r.submitted_at));
    revoked.sort_by_key(|r| std::cmp::Reverse(r.submitted_at));

    Ok(TemplatesListing {
        pending,
        active,
        revoked,
        available,
    })
}
