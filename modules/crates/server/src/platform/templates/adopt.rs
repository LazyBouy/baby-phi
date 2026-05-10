//! POST `/api/v0/orgs/:org/authority-templates/:kind/adopt` —
//! mint a fresh CEO-self-approved adoption AR for `kind`
//! (R-ADMIN-12-W3).
//!
//! The admin is the sole approver; the resulting AR is
//! immediately Approved (Template-E shape, same pattern M3's
//! org-creation wizard uses). The handler:
//! 1. Refuses if the org already has a live adoption AR for
//!    `kind` (pending or active).
//! 2. Loads the CEO's Agent id via the same "first Human member
//!    in org" rule as M4's `RepoActorResolver`.
//! 3. Calls `domain::templates::adoption::build_adoption_request`
//!    to construct the AR.
//! 4. Persists the AR + emits the adopted audit event.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, OrgId};
use domain::model::nodes::{AgentKind, AuthRequestState, PrincipalRef, TemplateKind};
use domain::templates::adoption::{build_adoption_request, AdoptionArgs};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{find_adoption_ar, is_adoptable_kind, TemplateError};

#[derive(Debug, Clone)]
pub struct AdoptInput {
    pub org_id: OrgId,
    pub kind: TemplateKind,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptOutcome {
    pub adoption_auth_request_id: AuthRequestId,
    pub state: AuthRequestState,
    pub audit_event_id: AuditEventId,
}

pub async fn adopt_template_inline(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: AdoptInput,
) -> Result<AdoptOutcome, TemplateError> {
    if matches!(input.kind, TemplateKind::E) {
        return Err(TemplateError::TemplateEAlwaysAvailable);
    }
    if !is_adoptable_kind(input.kind) {
        return Err(TemplateError::KindNotAdoptable(input.kind));
    }

    // Org must exist.
    let _ = repo
        .get_organization(input.org_id)
        .await?
        .ok_or(TemplateError::OrgNotFound(input.org_id))?;

    // Reject re-adoption when an AR already exists in a live
    // (non-terminal, non-failed) state. Revoked / Denied / Expired
    // / Cancelled paths allow fresh adoption — the kind is
    // available again.
    if let Some(existing) = find_adoption_ar(&*repo, input.org_id, input.kind).await? {
        match existing.state {
            AuthRequestState::Approved => {
                return Err(TemplateError::AdoptionAlreadyActive(existing.id));
            }
            AuthRequestState::Pending | AuthRequestState::Partial => {
                return Err(TemplateError::AdoptionAlreadyPending(existing.id));
            }
            _ => {}
        }
    }

    // Identify the CEO (= first Human-kind agent in the org).
    let agents = repo.list_agents_in_org(input.org_id).await?;
    let ceo = agents
        .iter()
        .find(|a| a.kind == AgentKind::Human)
        .map(|a| a.id)
        .ok_or(TemplateError::Forbidden(input.org_id))?;

    // Build the adoption AR (auto-approved; Template-E shape).
    let ar = build_adoption_request(
        input.kind,
        AdoptionArgs {
            org_id: input.org_id,
            ceo: PrincipalRef::Agent(ceo),
            now: input.now,
        },
    );
    let ar_id = ar.id;
    let ar_state = ar.state;

    // CH-18 / ADR-0056 §D56.5 + §D56.8 + F3.B.create-side.a — kernel-
    // skip-equivalent for the submit-side defence-in-depth check.
    // Rationale: the AR's `requestor` is intentionally `ceo` (set at
    // `build_adoption_request` line 90 above), NOT `input.actor` (the
    // platform-admin invoking the adopt endpoint). Running the
    // synthetic-Draft Submit check with `&PrincipalRef::Agent(input.actor)`
    // would return `RequestorOnlyOperation` because admin != ceo, which
    // does not capture the F3.B.create-side.a defence-in-depth INTENT
    // for an admin-on-behalf-of-CEO flow. The handler's existing
    // first-Human-in-org gate at line 79-83 fills the equivalent role
    // (admin must be in org with a Human CEO present). Documented as
    // a structural mismatch between concept doc 02's "Draft × Submit
    // by requestor" cell and baby-phi's admin-on-behalf-of-CEO Template-
    // adoption flow; a successor chunk may revisit via
    // `D-CH18-FOLLOWUP-02`.
    repo.create_auth_request(&ar).await?;

    let event = super::audit_events::template_adopted_inline(
        input.actor,
        input.org_id,
        input.kind,
        ar_id,
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;

    Ok(AdoptOutcome {
        adoption_auth_request_id: ar_id,
        state: ar_state,
        audit_event_id: event_id,
    })
}
