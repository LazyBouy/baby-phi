//! GET `/api/v0/projects/:project/sessions` — session header list.
//!
//! Returns the project's sessions as a stripped `SessionHeader`
//! shape — no phi-core `inner` leak. The JSON-schema-snapshot test
//! in `acceptance_sessions_list.rs` asserts the response does NOT
//! contain the `inner` / `blueprint` / `loops` keys (those are the
//! detail-endpoint's concern).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::model::ids::{AgentId, ProjectId, SessionId};
use domain::model::nodes::SessionGovernanceState;
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::SessionError;

/// Stripped session header — the shape `list_sessions_in_project`
/// returns + the shape page 11's "Recent sessions" panel consumes.
///
/// Deliberately carries NO phi-core types (no `inner`, no
/// `blueprint`, no `loops`). A reviewer spotting a phi-core type in
/// this struct should reject the PR.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionHeader {
    pub id: SessionId,
    pub agent_id: AgentId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub governance_state: SessionGovernanceState,
    /// Total turns across all the session's loops. Denormalised
    /// here so the list endpoint doesn't need to read every loop.
    pub turn_count: u32,
    /// Running-token total at the session's current state. For
    /// finalised sessions this is the final value; for active
    /// sessions it's the snapshot at last recorder flush.
    pub tokens_spent: u64,
}

/// List sessions in a project, newest-first.
pub async fn list_sessions_in_project(
    repo: Arc<dyn Repository>,
    project: ProjectId,
) -> Result<Vec<SessionHeader>, SessionError> {
    let sessions = repo.list_sessions_in_project(project).await?;
    let mut out: Vec<SessionHeader> = sessions
        .into_iter()
        .map(|s| {
            // Turn count is only discoverable via the full
            // `fetch_session` — the list surface exposes the
            // tokens_spent field + a `None` turn_count stub. At
            // P4 we accept the partial shape; a later phase
            // denormalises turn_count onto the session row
            // directly (C-M6 follow-up).
            SessionHeader {
                id: s.id,
                agent_id: s.started_by,
                started_at: s.started_at,
                ended_at: s.ended_at,
                governance_state: s.governance_state,
                turn_count: s.inner.loops.iter().map(|l| l.turns.len() as u32).sum(),
                tokens_spent: s.tokens_spent,
            }
        })
        .collect();
    out.sort_by_key(|h| std::cmp::Reverse(h.started_at));
    Ok(out)
}
