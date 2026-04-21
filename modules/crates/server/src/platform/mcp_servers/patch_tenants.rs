//! `patch_mcp_tenants` — update `tenants_allowed`, cascading revocation
//! when the new set is a strict subset of the old.
//!
//! Flow (plan §P6 + D7):
//! 1. Load the current row (404 if unknown id).
//! 2. Classify the delta:
//!    - Same set → no-op (no repo write, no cascade, no audit).
//!    - `new ⊇ old` (widening / no drops) → [`Repository::patch_mcp_tenants`]
//!      (overwrite-only; no cascade events).
//!    - `new ⊂ old` (narrowing) → [`Repository::narrow_mcp_tenants`],
//!      which runs the grant-revocation sweep in-txn and returns the
//!      list of [`TenantRevocation`] entries. The handler then emits:
//!        * one `platform.mcp_server.tenant_access_revoked` summary
//!          (carrying the full `revoked_orgs` list + counts);
//!        * one `auth_request.revoked` per affected AR (Alerted).
//! 3. Return the [`PatchTenantsOutcome`] so the HTTP layer can surface
//!    the blast-radius summary to the operator.
//!
//! `All → Only(ids)` edge case: for M2 the in-memory + SurrealDB impl
//! returns an empty cascade because we don't enumerate "every org on
//! the platform" at this point. M3 will supply the full org index and
//! walk it here.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::mcp as mcp_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::{AgentId, McpServerId};
use domain::model::nodes::{PrincipalRef, ResourceRef};
use domain::model::TenantSet;
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{mcp_server_uri, McpError, PatchTenantsOutcome, KIND_TAG};

pub struct PatchTenantsInput {
    pub mcp_server_id: McpServerId,
    pub new_allowed: TenantSet,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

/// Decide whether `new` is a strict subset of `old` — i.e. narrowing
/// that requires the cascade. `All → Only(_)` counts as narrowing but
/// the repo impl currently returns empty cascade (see module docs).
fn is_narrowing(old: &TenantSet, new: &TenantSet) -> bool {
    match (old, new) {
        (TenantSet::All, TenantSet::Only(_)) => true,
        (TenantSet::Only(old_ids), TenantSet::Only(new_ids)) => {
            old_ids.iter().any(|o| !new_ids.contains(o))
        }
        _ => false,
    }
}

pub async fn patch_mcp_tenants(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: PatchTenantsInput,
) -> Result<PatchTenantsOutcome, McpError> {
    // 1. Load current row. The list query is cheap at M2 scale (~tens
    //    of rows) and also gives us a clean 404 path.
    let existing = repo
        .list_mcp_servers(true)
        .await?
        .into_iter()
        .find(|s| s.id == input.mcp_server_id)
        .ok_or(McpError::NotFound(input.mcp_server_id))?;

    // 2. Classify.
    if existing.tenants_allowed == input.new_allowed {
        // No-op — return silently with no audit event. Handlers can
        // detect this by the empty cascade + None audit id.
        return Ok(PatchTenantsOutcome {
            mcp_server_id: input.mcp_server_id,
            cascade: vec![],
            audit_event_id: None,
        });
    }

    // Template E AR — self-approved platform-admin write — covers both
    // the widen and narrow arms (both mutate the row).
    let uri = mcp_server_uri(input.mcp_server_id);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec!["patch_tenants".to_string()],
        justification: Some(format!(
            "self-approved platform-admin write: patch tenants_allowed on MCP server `{}`",
            input.mcp_server_id
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;
    repo.create_auth_request(&ar).await?;

    if !is_narrowing(&existing.tenants_allowed, &input.new_allowed) {
        // 2a. Widening (or sideways move in the TenantSet::All space) —
        //     overwrite; no cascade, no cascade audit event.
        repo.patch_mcp_tenants(input.mcp_server_id, &input.new_allowed)
            .await?;
        return Ok(PatchTenantsOutcome {
            mcp_server_id: input.mcp_server_id,
            cascade: vec![],
            audit_event_id: None,
        });
    }

    // 2b. Narrowing — run the cascade.
    let cascade = repo
        .narrow_mcp_tenants(input.mcp_server_id, &input.new_allowed, input.now)
        .await?;

    // Reload the row so the summary event carries the POST-cascade
    // snapshot (tenants_allowed reflects the new set).
    let updated = repo
        .list_mcp_servers(true)
        .await?
        .into_iter()
        .find(|s| s.id == input.mcp_server_id)
        .ok_or(McpError::NotFound(input.mcp_server_id))?;

    // Per-AR `auth_request.revoked` events — one per entry. Emitted
    // BEFORE the summary event so a reader walking the chain sees the
    // individual revocations grouped under the summary.
    for rev in &cascade {
        let event = mcp_events::auth_request_revoked_by_mcp_cascade(
            input.actor,
            input.mcp_server_id,
            rev.org,
            rev,
            Some(auth_request_id),
            input.now,
        );
        audit
            .emit(event)
            .await
            .map_err(|e| McpError::AuditEmit(e.to_string()))?;
    }

    // Summary event.
    let summary = mcp_events::mcp_server_tenant_access_revoked(
        input.actor,
        &updated,
        &existing.tenants_allowed,
        &cascade,
        Some(auth_request_id),
        input.now,
    );
    let summary_id = summary.event_id;
    audit
        .emit(summary)
        .await
        .map_err(|e| McpError::AuditEmit(e.to_string()))?;

    Ok(PatchTenantsOutcome {
        mcp_server_id: input.mcp_server_id,
        cascade,
        audit_event_id: Some(summary_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::ids::OrgId;

    #[test]
    fn is_narrowing_detects_strict_subset() {
        let a = OrgId::new();
        let b = OrgId::new();
        assert!(is_narrowing(
            &TenantSet::Only(vec![a, b]),
            &TenantSet::Only(vec![a]),
        ));
        assert!(!is_narrowing(
            &TenantSet::Only(vec![a]),
            &TenantSet::Only(vec![a, b]),
        ));
        assert!(!is_narrowing(
            &TenantSet::Only(vec![a]),
            &TenantSet::Only(vec![a]),
        ));
    }

    #[test]
    fn is_narrowing_treats_all_to_only_as_narrow() {
        // Even though M2's repo impl can't enumerate "every other org",
        // semantically `All → Only(subset)` IS narrowing. The handler
        // still audits the Template E AR; the cascade just comes back
        // empty.
        let a = OrgId::new();
        assert!(is_narrowing(&TenantSet::All, &TenantSet::Only(vec![a])));
    }

    #[test]
    fn is_narrowing_treats_only_to_all_as_widen() {
        let a = OrgId::new();
        assert!(!is_narrowing(&TenantSet::Only(vec![a]), &TenantSet::All));
        assert!(!is_narrowing(&TenantSet::All, &TenantSet::All));
    }
}
