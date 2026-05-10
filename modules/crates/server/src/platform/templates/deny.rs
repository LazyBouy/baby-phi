//! POST `/api/v0/orgs/:org/authority-templates/:kind/deny` —
//! transition a pending adoption AR to Denied (R-ADMIN-12-W2).
//!
//! Denying closes the adoption Auth Request without activating
//! the template. The template remains available for re-adoption.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m5_2::auth_request_access::auth_request_access_denied;
use domain::audit::AuditEmitter;
use domain::auth_requests::access::{check_auth_request_access, IntendedOp};
use domain::auth_requests::transition_slot;
use domain::model::ids::{AgentId, AuditEventId, OrgId};
use domain::model::nodes::{ApproverSlotState, AuthRequestState, PrincipalRef, TemplateKind};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::{find_adoption_ar, is_adoptable_kind, TemplateError};

#[derive(Debug, Clone)]
pub struct DenyInput {
    pub org_id: OrgId,
    pub kind: TemplateKind,
    pub reason: String,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenyOutcome {
    pub adoption_auth_request_id: domain::model::ids::AuthRequestId,
    pub new_state: AuthRequestState,
    pub audit_event_id: AuditEventId,
}

pub async fn deny_adoption_ar(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: DenyInput,
) -> Result<DenyOutcome, TemplateError> {
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

    match ar.state {
        AuthRequestState::Approved
        | AuthRequestState::Denied
        | AuthRequestState::Expired
        | AuthRequestState::Revoked
        | AuthRequestState::Cancelled => {
            return Err(TemplateError::AdoptionTerminal {
                ar: ar.id,
                state: match ar.state {
                    AuthRequestState::Approved => "approved",
                    AuthRequestState::Denied => "denied",
                    AuthRequestState::Expired => "expired",
                    AuthRequestState::Revoked => "revoked",
                    AuthRequestState::Cancelled => "cancelled",
                    _ => "terminal",
                },
            });
        }
        AuthRequestState::Draft
        | AuthRequestState::Pending
        | AuthRequestState::InProgress
        | AuthRequestState::Partial => {}
    }

    // CH-18 / ADR-0056 §D56.6 — gate the mutation on the per-state
    // access matrix. On Err, emit the Alerted-class
    // `auth_request.access_denied` audit event, then surface the typed
    // error to the caller.
    if let Err(access_err) =
        check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Deny)
    {
        let event = auth_request_access_denied(
            input.actor,
            ar.id,
            input.org_id,
            &access_err,
            IntendedOp::Deny,
            input.now,
        );
        audit
            .emit(event)
            .await
            .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;
        return Err(TemplateError::AccessDenied(access_err));
    }

    let next = transition_slot(&ar, 0, 0, ApproverSlotState::Denied, input.now)
        .map_err(|e| TemplateError::StateTransitionFailed(e.to_string()))?;
    repo.update_auth_request(&next).await?;

    let event = super::audit_events::template_adoption_denied(
        input.actor,
        input.org_id,
        input.kind,
        next.id,
        &input.reason,
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;

    Ok(DenyOutcome {
        adoption_auth_request_id: next.id,
        new_state: next.state,
        audit_event_id: event_id,
    })
}
