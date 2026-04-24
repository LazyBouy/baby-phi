//! POST `/api/v0/orgs/:org/authority-templates/:kind/revoke` —
//! transition the active adoption AR to Revoked + cascade-revoke
//! every grant whose `descends_from == adoption_ar.id`
//! (R-ADMIN-12-W4, forward-only per
//! [system/s04-auth-request-state-transitions.md]).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::auth_requests::revoke as revoke_ar;
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, OrgId};
use domain::model::nodes::{AuthRequestState, TemplateKind};
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

    // Transition the AR → Revoked via the domain state machine.
    // The helper returns the new AR + a companion audit event; we
    // persist the AR now + emit our own template-level audit after
    // the grant cascade (template.revoked carries grant_count).
    let (next, _auth_ar_audit_event) = revoke_ar(&ar, Some(input.actor), &input.reason, input.now)
        .map_err(|e| TemplateError::StateTransitionFailed(e.to_string()))?;
    repo.update_auth_request(&next).await?;

    // Forward-only grant cascade: every live grant whose
    // `descends_from == ar.id` flips to `revoked_at = now`.
    let grants_revoked = repo
        .revoke_grants_by_descends_from(next.id, input.now)
        .await?;
    let grant_count_revoked = grants_revoked.len() as u32;

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
