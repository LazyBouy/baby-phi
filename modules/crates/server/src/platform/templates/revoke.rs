//! POST `/api/v0/orgs/:org/authority-templates/:kind/revoke` —
//! transition the active adoption AR to Revoked + cascade-revoke
//! every grant whose `descends_from == adoption_ar.id`
//! (R-ADMIN-12-W4, forward-only per
//! [system/s04-auth-request-state-transitions.md]).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m5_2::auth_request_access::auth_request_access_denied;
use domain::audit::AuditEmitter;
use domain::auth_requests::access::{check_auth_request_access, IntendedOp};
use domain::auth_requests::revoke as revoke_ar;
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, OrgId};
use domain::model::nodes::{AuthRequestState, PrincipalRef, TemplateKind};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{find_adoption_ar, is_adoptable_kind, TemplateError};

#[derive(Debug, Clone)]
pub struct RevokeInput {
    pub org_id: OrgId,
    pub kind: TemplateKind,
    pub reason: String,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeOutcome {
    pub adoption_auth_request_id: AuthRequestId,
    pub grants_revoked: Vec<GrantId>,
    pub grant_count_revoked: u32,
    pub audit_event_id: AuditEventId,
}

pub async fn revoke_template(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: RevokeInput,
) -> Result<RevokeOutcome, TemplateError> {
    if matches!(input.kind, TemplateKind::E) {
        return Err(TemplateError::TemplateEAlwaysAvailable);
    }
    if !is_adoptable_kind(input.kind) {
        return Err(TemplateError::KindNotAdoptable(input.kind));
    }
    if input.reason.trim().is_empty() {
        return Err(TemplateError::InputInvalid(
            "reason must not be empty".into(),
        ));
    }

    let ar = find_adoption_ar(&*repo, input.org_id, input.kind)
        .await?
        .ok_or(TemplateError::AdoptionNotFound {
            org: input.org_id,
            kind: input.kind,
        })?;

    // Only Approved adoptions can be revoked (R-ADMIN-12-W4 +
    // s04 §revoke rules). Pending / Denied / already-Revoked
    // surface as TEMPLATE_ADOPTION_TERMINAL.
    if !matches!(ar.state, AuthRequestState::Approved) {
        let state = match ar.state {
            AuthRequestState::Draft => "draft",
            AuthRequestState::Pending => "pending",
            AuthRequestState::InProgress => "in_progress",
            AuthRequestState::Partial => "partial",
            AuthRequestState::Denied => "denied",
            AuthRequestState::Expired => "expired",
            AuthRequestState::Revoked => "revoked",
            AuthRequestState::Cancelled => "cancelled",
            AuthRequestState::Approved => "approved",
        };
        return Err(TemplateError::AdoptionTerminal { ar: ar.id, state });
    }

    // CH-18 / ADR-0056 §D56.6 — gate the mutation on the per-state
    // access matrix. On Err, emit the Alerted-class
    // `auth_request.access_denied` audit event, then surface the typed
    // error to the caller.
    if let Err(access_err) =
        check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Revoke)
    {
        let event = auth_request_access_denied(
            input.actor,
            ar.id,
            input.org_id,
            &access_err,
            IntendedOp::Revoke,
            input.now,
        );
        audit
            .emit(event)
            .await
            .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;
        return Err(TemplateError::AccessDenied(access_err));
    }

    // Transition the AR → Revoked via the domain state machine.
    // The helper returns the new AR + a companion audit event; we
    // persist the AR now + emit our own template-level audit after
    // the grant cascade (template.revoked carries grant_count).
    let (next, _auth_ar_audit_event) = revoke_ar(&ar, Some(input.actor), &input.reason, input.now)
        .map_err(|e| TemplateError::StateTransitionFailed(e.to_string()))?;
    repo.update_auth_request(&next).await?;

    // Forward-only tree-wide grant cascade: every live grant in the
    // descend-tree rooted at `next.id` flips to `revoked_at = now`,
    // including grandchildren minted under intermediate-AR authority
    // (CH-14 / ADR-0053 §D53.4). The single-hop sibling
    // `revoke_grants_by_descends_from` is preserved verbatim for the
    // M2 `narrow_mcp_tenants` caller per ADR-0033 contract.
    //
    // The Repository returns a `CascadeResult` with both
    // `revoked_grants` (for the `template.revoked` summary event's
    // `grant_count`) and `cascaded_ars` (every level-≥1 AR in the
    // descent tree, for per-AR `auth_request.revoked` emission per
    // ADR-0053 §D53.7).
    let cascade = repo
        .revoke_grants_by_descends_from_recursive(next.id, input.now)
        .await?;
    let grants_revoked = cascade.revoked_grants;
    let grant_count_revoked = grants_revoked.len() as u32;

    // CH-14 / ADR-0053 §D53.7 — for each level-≥1 cascaded AR, mirror
    // the level-0 flow: build the (next_ar, audit_event) pair via the
    // domain helper, persist the AR-state-transition (Approved →
    // Revoked) via `update_auth_request`, and emit the per-AR
    // `auth_request.revoked` audit event. Ordering inside the loop
    // preserves the BFS insertion order from the Repository so the
    // audit chain hashes deterministically per cascade run. Already-
    // closed-terminal ARs are skipped (idempotency) — the Repository
    // surfaces them anyway when their parent grant flips during
    // re-runs, so the handler must defend against double-revoke.
    for cascaded_ar_id in cascade.cascaded_ars.iter().copied() {
        let cascaded_ar = match repo.get_auth_request(cascaded_ar_id).await? {
            Some(a) => a,
            None => continue, // defensive: row absent → skip
        };
        if !matches!(
            cascaded_ar.state,
            AuthRequestState::Approved | AuthRequestState::Partial
        ) {
            // Already closed-terminal (e.g. previously cascaded
            // through a prior revoke or manual revoke); skip to keep
            // the cascade idempotent.
            continue;
        }
        let (next_cascaded, cascaded_event) =
            revoke_ar(&cascaded_ar, Some(input.actor), &input.reason, input.now)
                .map_err(|e| TemplateError::StateTransitionFailed(e.to_string()))?;
        repo.update_auth_request(&next_cascaded).await?;
        audit
            .emit(cascaded_event)
            .await
            .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;
    }

    // Emit the template.revoked audit event. `grant_count` makes
    // the revocation count visible in-line with the audit record
    // so operators don't need to cross-reference the cascade
    // report.
    let event = super::audit_events::template_revoked(
        input.actor,
        input.org_id,
        input.kind,
        next.id,
        grant_count_revoked,
        &input.reason,
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;

    Ok(RevokeOutcome {
        adoption_auth_request_id: next.id,
        grants_revoked,
        grant_count_revoked,
        audit_event_id: event_id,
    })
}
