//! POST `/api/v0/sessions/:id/terminate` — operator-initiated
//! session abort.
//!
//! Fires the `CancellationToken` in [`SessionRegistry`], flips the
//! Session row's `governance_state` to `Aborted`, removes the
//! registry entry (so `session_registry.len()` drops below the
//! `max_concurrent` cap), emits the
//! [`DomainEvent::SessionAborted`] governance event.
//!
//! Idempotent at the **already-terminal** layer: terminating a
//! session that's already `Aborted` / `Completed` / `FailedLaunch`
//! returns `SESSION_ALREADY_TERMINAL` (409).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::AuditEmitter;
use domain::events::{DomainEvent, EventBus};
use domain::model::ids::{AgentId, AuditEventId, SessionId};
use domain::model::nodes::{AgentKind, SessionGovernanceState};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::SessionError;
use crate::state::SessionRegistry;

/// Input for [`terminate_session`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminateInput {
    pub session_id: SessionId,
    pub reason: String,
    pub terminated_by: AgentId,
    pub now: DateTime<Utc>,
}

/// Outcome returned on successful terminate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminateOutcome {
    pub session_id: SessionId,
    pub final_state: SessionGovernanceState,
    pub event_id: AuditEventId,
}

/// Terminate a running session.
pub async fn terminate_session(
    repo: Arc<dyn Repository>,
    _audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
    registry: SessionRegistry,
    input: TerminateInput,
) -> Result<TerminateOutcome, SessionError> {
    if input.reason.trim().is_empty() {
        return Err(SessionError::TerminateReasonRequired);
    }

    let detail = repo
        .fetch_session(input.session_id)
        .await?
        .ok_or(SessionError::SessionNotFound(input.session_id))?;

    if detail.session.governance_state.is_terminal() {
        return Err(SessionError::SessionAlreadyTerminal(input.session_id));
    }

    // Access gate — either the session's starter OR a Human member
    // of the owning org may terminate.
    let starter_is_caller = detail.session.started_by == input.terminated_by;
    let caller_is_org_human = if starter_is_caller {
        true
    } else {
        let agents = repo.list_agents_in_org(detail.session.owning_org).await?;
        agents
            .iter()
            .any(|a| a.id == input.terminated_by && a.kind == AgentKind::Human)
    };
    if !caller_is_org_human {
        return Err(SessionError::TerminateForbidden(input.session_id));
    }

    // Fire the cancellation token (if a launch task is running).
    // `remove` returns the previously-stored token so we can cancel
    // it; a missing entry means the launch chain already cleaned up
    // (e.g. the task completed between the fetch and this line).
    if let Some((_, token)) = registry.remove(&input.session_id) {
        token.cancel();
    }

    // Flip the governance state. The `reason` + `terminated_by`
    // fields live on the audit + governance event emissions below,
    // NOT on the durable session row itself (per the repo's
    // `terminate_session` signature).
    repo.terminate_session(input.session_id, input.now).await?;

    // Governance event (post-commit; bus is fail-safe).
    let event_id = AuditEventId::new();
    event_bus
        .emit(DomainEvent::SessionAborted {
            session_id: input.session_id,
            reason: input.reason,
            terminated_by: input.terminated_by,
            at: input.now,
            event_id,
        })
        .await;

    Ok(TerminateOutcome {
        session_id: input.session_id,
        final_state: SessionGovernanceState::Aborted,
        event_id,
    })
}
