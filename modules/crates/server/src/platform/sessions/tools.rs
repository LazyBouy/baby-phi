//! GET `/api/v0/sessions/:id/tools` — resolve the agent's tool
//! summaries (C-M5-4 close).
//!
//! ## Scope
//!
//! At P4 this surface ships the **wire shape + resolver**. The
//! returned list is empty for every agent today because baby-phi
//! hasn't wired concrete phi-core [`AgentTool`] implementations onto
//! its governance model yet — the `HAS_TOOL` edge exists in the
//! ontology but no M4/M5 writer populates it. M7+ (concrete tool
//! catalogue) swaps the empty list for real `ToolSummary` rows
//! derived from the agent's `HAS_TOOL` edges.
//!
//! ## phi-core leverage
//!
//! One new import at P4:
//! - `use phi_core::types::tool::AgentTool;` — **compile-time
//!   witness** that phi-core's trait exists + a future M7+ phase can
//!   switch the return type to `Vec<Box<dyn AgentTool>>` without
//!   restructuring the call site. The witness is pinned by
//!   [`_is_phi_core_agent_tool_trait`] below.

use std::sync::Arc;

use domain::model::ids::{AgentId, SessionId};
use domain::Repository;
use phi_core::types::tool::AgentTool;
use serde::{Deserialize, Serialize};

use super::{show::show_session, SessionError};

/// Wire-level summary of a tool the agent can invoke.
///
/// Deliberately NOT a phi-core trait object — this is the HTTP
/// projection that the web UI + CLI consume.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSummary {
    pub name: String,
    pub label: String,
    pub description: String,
    /// JSON-Schema describing the tool's parameters. Matches
    /// phi-core's [`AgentTool::input_schema`] shape so the strip
    /// is lossless when M7+ switches to real trait objects.
    pub parameters_schema: serde_json::Value,
}

/// Return the tool summaries for `agent`.
///
/// At M5/P4 this returns an empty `Vec` — see module doc. The
/// signature is stable so M7+ can swap the body without touching
/// call sites.
pub async fn resolve_agent_tools(
    _repo: Arc<dyn Repository>,
    _agent: AgentId,
) -> Result<Vec<ToolSummary>, SessionError> {
    Ok(Vec::new())
}

/// Resolve tools for the agent that owns `session_id`.
/// Convenience wrapper used by the `/sessions/:id/tools` handler.
pub async fn resolve_tools_for_session(
    repo: Arc<dyn Repository>,
    session_id: SessionId,
    viewer: AgentId,
) -> Result<Vec<ToolSummary>, SessionError> {
    let detail = show_session(repo.clone(), session_id, viewer).await?;
    resolve_agent_tools(repo, detail.session.started_by).await
}

// ---------------------------------------------------------------------------
// Compile-time witness (M5 discipline)
// ---------------------------------------------------------------------------
//
// Never runs; exists so a phi-core rename of `AgentTool` (trait) or
// a breaking change to the trait's `input_schema` / `name` / `run`
// signatures surfaces as a baby-phi build failure rather than a
// silent drift.

#[allow(dead_code)]
fn _is_phi_core_agent_tool_trait<T: AgentTool + ?Sized>(_: &T) {}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::in_memory::InMemoryRepository;

    #[tokio::test]
    async fn resolve_agent_tools_returns_empty_list_at_m5() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let tools = resolve_agent_tools(repo, AgentId::new()).await.expect("ok");
        assert!(
            tools.is_empty(),
            "M5 ships the resolver shape; no HAS_TOOL writer yet"
        );
    }

    #[test]
    fn tool_summary_serde_round_trip_is_lossless() {
        let t = ToolSummary {
            name: "bash".to_string(),
            label: "Bash shell".to_string(),
            description: "run bash commands".to_string(),
            parameters_schema: serde_json::json!({"type": "object"}),
        };
        let j = serde_json::to_string(&t).expect("serialize");
        let back: ToolSummary = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(t, back);
    }
}
