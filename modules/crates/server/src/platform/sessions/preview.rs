//! POST `/api/v0/orgs/:org/projects/:project/sessions/preview` —
//! Permission Check preview (D5, server-side).
//!
//! Runs the M1 [`domain::permissions::check`] engine over steps 0–6
//! so the operator can see the decision BEFORE firing a real launch.
//! Keeping this server-side single-sources the permission algorithm
//! + lets the CLI reuse the endpoint verbatim.
//!
//! ## Synthetic manifest
//!
//! A session launch doesn't have a concrete `ToolCall` at preview
//! time, so we build a minimal synthetic manifest declaring:
//! - `actions = ["launch_session"]`
//! - `resource = ["session"]`
//! - `transitive = []`
//!
//! M6+ phases that introduce finer-grained launch policies refine
//! this. At M5 the endpoint proves the engine is wired end-to-end.

use std::collections::HashSet;
use std::sync::Arc;

use domain::model::ids::{AgentId, OrgId, ProjectId};
use domain::model::nodes::PrincipalRef;
use domain::permissions::{
    check, CheckContext, ConsentIndex, Decision, Manifest, NoopMetrics, StaticCatalogue, ToolCall,
};
use domain::Repository;
use serde::{Deserialize, Serialize};

use super::SessionError;

/// Input to [`preview_session`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewInput {
    pub org_id: OrgId,
    pub project_id: ProjectId,
    pub agent_id: AgentId,
}

/// Outcome returned by [`preview_session`].
///
/// The full [`Decision`] is surfaced so the UI + CLI can render the
/// worked trace without re-running the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewOutcome {
    pub agent_id: AgentId,
    pub project_id: ProjectId,
    pub decision: Decision,
}

/// Run the preview.
pub async fn preview_session(
    repo: Arc<dyn Repository>,
    input: PreviewInput,
) -> Result<PreviewOutcome, SessionError> {
    // Confirm the agent + project exist; the UI's error messages are
    // more actionable with an explicit 404 than a "denied at step 2"
    // opaque fail.
    let agent = repo
        .get_agent(input.agent_id)
        .await?
        .ok_or(SessionError::AgentNotFound(input.agent_id))?;
    let _project = repo
        .get_project(input.project_id)
        .await?
        .ok_or(SessionError::ProjectNotFound(input.project_id))?;

    // Gather grants.
    let agent_grants = repo
        .list_grants_for_principal(&PrincipalRef::Agent(input.agent_id))
        .await?;
    let project_grants = repo
        .list_grants_for_principal(&PrincipalRef::Project(input.project_id))
        .await?;
    let org_grants = repo
        .list_grants_for_principal(&PrincipalRef::Organization(input.org_id))
        .await?;
    // Ceiling grants are the org's root grants at M5 — a richer
    // ceiling hierarchy lands at M7+.
    let ceiling_grants: Vec<_> = org_grants.clone();

    // Build synthetic launch manifest. `session` is a composite in
    // the M2 ontology + the resource catalogue carries it per org.
    let manifest = Manifest {
        actions: vec!["launch_session".to_string()],
        resource: vec!["session".to_string()],
        transitive: vec![],
        constraints: vec![],
        constraint_requirements: std::collections::HashMap::new(),
        kinds: vec![],
    };

    // Preview is stateless — no consents recorded in-band.
    let consents = ConsentIndex::empty();
    // Seed the catalogue with the session resource so Step 0
    // doesn't mis-miss on the synthetic URI.
    let catalogue = StaticCatalogue::with_entries([(Some(input.org_id), "session".to_string())]);
    let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

    let ctx = CheckContext {
        agent: agent.id,
        current_org: Some(input.org_id),
        current_project: Some(input.project_id),
        agent_grants: &agent_grants,
        project_grants: &project_grants,
        org_grants: &org_grants,
        ceiling_grants: &ceiling_grants,
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &template_gated,
        call: ToolCall::default(),
    };

    let decision = check(&ctx, &manifest, &NoopMetrics);

    Ok(PreviewOutcome {
        agent_id: input.agent_id,
        project_id: input.project_id,
        decision,
    })
}
