//! GET `/api/v0/sessions/:id` — full `SessionDetail`.
//!
//! Access gate: the viewer must either have started the session
//! (`session.started_by == viewer`) OR be a Human-kind member of the
//! owning org. Cross-org viewing is forbidden (`FORBIDDEN`).
//!
//! Response carries the wrapped phi-core types via serde flatten —
//! the schema-snapshot test in `acceptance_sessions_show.rs` pins
//! the depth so future phi-core evolution doesn't silently leak new
//! fields.

use std::sync::Arc;

use domain::model::composites_m5::SessionDetail;
use domain::model::ids::{AgentId, SessionId};
use domain::model::nodes::AgentKind;
use domain::Repository;

use super::SessionError;

/// View returned by [`show_session`].
///
/// At P4 this is a thin alias of [`SessionDetail`]; the access gate
/// above restricts *who* sees it but the shape is the full wrap
/// (session + loops + turns-by-loop). The list endpoint returns the
/// header strip instead.
pub type SessionView = SessionDetail;

/// Look up a session + enforce the access gate.
pub async fn show_session(
    repo: Arc<dyn Repository>,
    session_id: SessionId,
    viewer: AgentId,
) -> Result<SessionView, SessionError> {
    let detail = repo
        .fetch_session(session_id)
        .await?
        .ok_or(SessionError::SessionNotFound(session_id))?;

    if detail.session.started_by == viewer {
        return Ok(detail);
    }

    // Viewer must be a Human member of the session's owning org.
    let agents = repo.list_agents_in_org(detail.session.owning_org).await?;
    let is_org_human_member = agents
        .iter()
        .any(|a| a.id == viewer && a.kind == AgentKind::Human);
    if is_org_human_member {
        return Ok(detail);
    }

    Err(SessionError::Forbidden(session_id))
}
