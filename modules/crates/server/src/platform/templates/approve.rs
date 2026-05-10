//! POST `/api/v0/orgs/:org/authority-templates/:kind/approve` —
//! transition a pending adoption AR to Approved (R-ADMIN-12-W1).
//!
//! At M3 org-creation, adoption ARs are created as pre-approved
//! (Template-E shape). The page 12 approve handler still exists
//! for forward-compat + for adoption ARs created via other paths
//! that start Pending. It refuses (409) when the AR is already
//! terminal.

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
pub struct ApproveInput {
    pub org_id: OrgId,
    pub kind: TemplateKind,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveOutcome {
    pub adoption_auth_request_id: domain::model::ids::AuthRequestId,
    pub new_state: AuthRequestState,
    pub audit_event_id: AuditEventId,
}

pub async fn approve_adoption_ar(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: ApproveInput,
) -> Result<ApproveOutcome, TemplateError> {
    if matches!(input.kind, TemplateKind::E) {
        return Err(TemplateError::TemplateEAlwaysAvailable);
    }
    if !is_adoptable_kind(input.kind) {
        return Err(TemplateError::KindNotAdoptable(input.kind));
    }

    let ar = find_adoption_ar(&*repo, input.org_id, input.kind)
        .await?
        .ok_or(TemplateError::AdoptionNotFound {
            org: input.org_id,
            kind: input.kind,
        })?;

    match ar.state {
        AuthRequestState::Approved => {
            return Err(TemplateError::AdoptionAlreadyActive(ar.id));
        }
        AuthRequestState::Denied
        | AuthRequestState::Expired
        | AuthRequestState::Revoked
        | AuthRequestState::Cancelled => {
            return Err(TemplateError::AdoptionTerminal {
                ar: ar.id,
                state: state_name(ar.state),
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
        check_auth_request_access(&ar, &PrincipalRef::Agent(input.actor), IntendedOp::Approve)
    {
        let event = auth_request_access_denied(
            input.actor,
            ar.id,
            input.org_id,
            &access_err,
            IntendedOp::Approve,
            input.now,
        );
        audit
            .emit(event)
            .await
            .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;
        return Err(TemplateError::AccessDenied(access_err));
    }

    // Adoption ARs have a single resource slot + a single
    // approver slot. Fill it with Approved — the aggregation
    // flips the AR state to Approved.
    let next = transition_slot(&ar, 0, 0, ApproverSlotState::Approved, input.now)
        .map_err(|e| TemplateError::StateTransitionFailed(e.to_string()))?;
    repo.update_auth_request(&next).await?;

    let event = super::audit_events::template_adopted_approved(
        input.actor,
        input.org_id,
        input.kind,
        next.id,
        input.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| TemplateError::AuditEmit(e.to_string()))?;

    Ok(ApproveOutcome {
        adoption_auth_request_id: next.id,
        new_state: next.state,
        audit_event_id: event_id,
    })
}

fn state_name(s: AuthRequestState) -> &'static str {
    match s {
        AuthRequestState::Draft => "draft",
        AuthRequestState::Pending => "pending",
        AuthRequestState::InProgress => "in_progress",
        AuthRequestState::Approved => "approved",
        AuthRequestState::Denied => "denied",
        AuthRequestState::Partial => "partial",
        AuthRequestState::Expired => "expired",
        AuthRequestState::Revoked => "revoked",
        AuthRequestState::Cancelled => "cancelled",
    }
}
