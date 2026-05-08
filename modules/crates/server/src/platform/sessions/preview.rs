//! POST `/api/v0/orgs/:org/projects/:project/sessions/preview` —
//! Permission Check preview (D5, server-side).
//!
//! Runs the M1 [`domain::permissions::check`] engine over steps 0–6
//! so the operator can see the decision BEFORE firing a real launch.
//! Keeping this server-side single-sources the permission algorithm
//! + lets the CLI reuse the endpoint verbatim.
//!
//! ## Synthetic manifest (CH-15 / ADR-0054 §D54.1 + §D54.7)
//!
//! Both preview + launch call
//! [`domain::permissions::build_session_launch_manifest`] for the
//! synthetic launch manifest:
//! - `actions = [Action::Read, Action::Inspect, Action::List]` —
//!   matches Template A's `[Read, Inspect, List]` issuance shape
//!   exactly.
//! - `resource = ["session_object"]` — the composite expanding to
//!   `[DataObject, Tag]` per concept doc 01.
//!
//! Pre-CH-15 preview used `actions = [Action::Invoke]`, `resource =
//! ["session"]`; CH-15 unifies on the typed builder so preview's
//! Decision matches launch's Decision when grants are stable.

use std::collections::HashSet;
use std::sync::Arc;

use domain::model::ids::{AgentId, OrgId, ProjectId};
use domain::model::nodes::PrincipalRef;
use domain::permissions::{
    build_session_launch_manifest, check, CheckContext, ConsentIndex, Decision, NoopMetrics,
    StaticCatalogue, ToolCall,
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

    // CH-15 / ADR-0054 §D54.1 + §D54.7 — synthetic launch manifest
    // from the typed builder. Preview + launch share identical
    // manifest shape so preview's Decision matches launch's Decision
    // when grants are stable.
    let manifest = build_session_launch_manifest(input.project_id);

    // Preview is stateless — no consents recorded in-band.
    let consents = ConsentIndex::empty();
    // Seed the catalogue with the session_object resource so Step 0
    // doesn't mis-miss on the synthetic URI.
    let catalogue =
        StaticCatalogue::with_entries([(Some(input.org_id), "session_object".to_string())]);
    let template_gated: HashSet<domain::model::ids::AuthRequestId> = HashSet::new();

    let ctx = CheckContext {
        agent: agent.id,
        current_org: Some(input.org_id),
        current_project: Some(input.project_id),
        // Preview is stateless — no live session ambient context. The
        // launch handler at P3 populates `current_session = Some(s)`
        // for real launches. CH-11 / ADR-0048 §D48.3.
        current_session: None,
        agent_grants: &agent_grants,
        project_grants: &project_grants,
        org_grants: &org_grants,
        ceiling_grants: &ceiling_grants,
        catalogue: &catalogue,
        consents: &consents,
        // Preview doesn't observe org timeout-default; the
        // approval_timeout_default_response axis only applies to
        // TimedOut consents, which preview never has. Default to
        // `Deny` per concept doc 06 line 349 ("absence of consent is
        // not consent").
        timeout_default_response: domain::model::TimeoutResponse::Deny,
        template_gated_auth_requests: &template_gated,
        set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY,
        // CH-07 / ADR-0051 §D51.4 — preview-path is Shape A/D-only
        // today (read-only, single-scope). Empty slices preserve
        // today's single-tier cascade behaviour exactly. Multi-scope
        // preview support lands with CH-15.
        session_org_tags: &[],
        session_project_tags: &[],
        // CH-15 / ADR-0054 §D54.1 — synthetic ToolCall is class-level
        // (`target_uri = ""` skips Step 0 catalogue) but carries the
        // would-be session's tags so the engine's
        // [`expansion::resolve_grant`] kind_refinement
        // (`#kind:session`) on the `session_object` composite matches
        // at Step 3 + Step 5's scope cascade has the `project:<id>`
        // predicate to bind the lead's session-object grant against.
        call: ToolCall {
            target_uri: String::new(),
            target_tags: vec![
                "#kind:session".to_string(),
                format!("project:{}", input.project_id),
                format!("org:{}", input.org_id),
            ],
            target_agent: None,
            constraint_context: std::collections::HashMap::new(),
        },
    };

    let decision = check(&ctx, &manifest, &NoopMetrics);

    Ok(PreviewOutcome {
        agent_id: input.agent_id,
        project_id: input.project_id,
        decision,
    })
}
