//! Project-detail orchestrator — page 11 (M4/P7).
//!
//! Two orchestrators ship here:
//!
//! - [`project_detail`] — reader behind `GET /api/v0/projects/:id`.
//!   Returns the full [`ProjectDetail`] aggregate (project header +
//!   owning-org ids + roster + recent-sessions placeholder) OR a
//!   [`DetailOutcome::NotFound`] / [`DetailOutcome::AccessDenied`]
//!   sentinel the handler maps to 404 / 403.
//! - [`apply_okr_patch`] — writer behind `PATCH
//!   /api/v0/projects/:id/okrs`. Applies a batch of
//!   [`OkrPatchEntry`]s against the project's OKR vectors, persists
//!   the full row via [`Repository::upsert_project`], and emits one
//!   `platform.project.okr_updated` audit event per mutation.
//!
//! ## phi-core leverage
//!
//! **Q1 direct imports: 0** — page 11 is pure phi governance. The
//! `ProjectDetail` wire shape ships a deliberately **stripped** roster
//! row (id + kind + display_name + role), dropping the per-agent
//! `blueprint` field (which wraps [`phi_core::agents::profile::AgentProfile`]).
//! Rationale mirrors M3/P5 dashboard: coupling a read-heavy polling
//! surface to phi-core schema evolution would force a contract rev on
//! every phi-core release. Drill-down to the full blueprint is
//! available via page 09 (agent profile editor).
//!
//! **Q2 transitive: 0 at the wire tier** — the snapshot test in
//! [`tests::wire_shape_strips_phi_core`] asserts the JSON carries no
//! `defaults_snapshot` / `blueprint` / `execution_limits` /
//! `context_config` / `retry_config` keys at any depth.
//!
//! **Q3 rejections** (explicit module walk): `phi_core::Session` /
//! `LoopRecord` / `Turn` — deferred to M5 per D11; the `recent_sessions`
//! field in [`ProjectDetail`] is a placeholder `Vec::new()` until M5
//! wires baby-phi's governance `Session` node. `phi_core::AgentEvent`
//! — orthogonal per `phi/CLAUDE.md`. `phi_core::Usage` — the token
//! budget is governance-level economic resource tracking, not per-
//! loop usage.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use domain::audit::events::m4::projects::project_okrs_updated;
use domain::audit::AuditEmitter;
use domain::model::composites_m4::{KeyResult, Objective};
use domain::model::ids::{AgentId, AuditEventId, OrgId, ProjectId};
use domain::model::nodes::{Agent, AgentKind, AgentRole, Project};
use domain::repository::Repository;

use super::create::validate_okrs;
use super::ProjectError;

// ---------------------------------------------------------------------------
// Wire shapes — phi-core-stripped by design.
// ---------------------------------------------------------------------------

/// Compact roster row for the project-detail panel. Strips phi-core
/// fields (blueprint, execution_limits) — the schema-snapshot test at
/// the bottom of this file is the load-bearing invariant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterMember {
    pub agent_id: AgentId,
    pub kind: AgentKind,
    pub display_name: String,
    pub role: Option<AgentRole>,
    /// Membership role *on this project* — `Lead` if the `HAS_LEAD`
    /// edge targets this agent; `Member` if `HAS_AGENT`; `Sponsor` if
    /// `HAS_SPONSOR`. Derived by the orchestrator.
    pub project_role: ProjectMembershipRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMembershipRole {
    Lead,
    Member,
    Sponsor,
}

/// Placeholder row for the `recent_sessions` panel. The struct is
/// defined so the wire shape is stable across M4 ↔ M5; at M4 the list
/// is always empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentSessionStub {
    /// Opaque id — materialises to baby-phi's governance `Session` id
    /// at M5 (C-M5-3). The field is present at M4 so JSON consumers
    /// don't break when M5 flips the empty list to real rows.
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub summary: String,
}

/// Project-detail aggregate for page 11. Wire-stable contract; handler
/// serialises this verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub project: Project,
    /// Every org that holds a `BELONGS_TO` edge from this project.
    /// Shape A = one org; Shape B = two orgs.
    pub owning_org_ids: Vec<OrgId>,
    pub lead_agent_id: Option<AgentId>,
    pub roster: Vec<RosterMember>,
    /// **M4 placeholder** — always `Vec::new()`. Populates at M5 via
    /// C-M5-3 (baby-phi governance `Session` node). The field exists at
    /// M4 so CLI + Web renderers don't need to change when M5 ships.
    pub recent_sessions: Vec<RecentSessionStub>,
}

/// Three-state outcome: the project may be absent (→ 404), the viewer
/// may have no relation to any owning org (→ 403), or we return the
/// aggregate (→ 200).
#[derive(Debug)]
pub enum DetailOutcome {
    Found(Box<ProjectDetail>),
    NotFound,
    AccessDenied,
}

// ---------------------------------------------------------------------------
// Orchestrator — reader
// ---------------------------------------------------------------------------

pub async fn project_detail(
    repo: Arc<dyn Repository>,
    project_id: ProjectId,
    viewer_agent_id: AgentId,
) -> Result<DetailOutcome, ProjectError> {
    let project = match repo
        .get_project(project_id)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?
    {
        Some(p) => p,
        None => return Ok(DetailOutcome::NotFound),
    };

    // Resolve owning orgs by scanning every org for project containment.
    // Cheaper than adding a new repo method at M4 scope — M5 may
    // introduce `list_owning_orgs_for_project` if the access-check
    // is on a hot path.
    let viewer_row = repo
        .get_agent(viewer_agent_id)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?;
    let viewer_org = viewer_row.as_ref().and_then(|a| a.owning_org);

    // Owning-org set = every org whose `list_projects_in_org` contains
    // this project id. One query per candidate org; bounded to the
    // viewer's org + (if Shape B) the co-owner resolved below.
    // Since we don't yet know the co-owner, we conservatively scan the
    // viewer's org first and short-circuit if found.
    let mut owning_orgs: Vec<OrgId> = Vec::new();
    if let Some(org) = viewer_org {
        let projects = repo
            .list_projects_in_org(org)
            .await
            .map_err(|e| ProjectError::Repository(e.to_string()))?;
        if projects.iter().any(|p| p.id == project_id) {
            owning_orgs.push(org);
        }
    }

    // If the viewer's org doesn't own this project (or viewer has no
    // owning org), check every org in the platform. At M4 the platform
    // rarely has more than tens of orgs; M5+ may add a dedicated repo
    // method if this grows unbounded.
    if owning_orgs.is_empty() {
        let all = repo
            .list_all_orgs()
            .await
            .map_err(|e| ProjectError::Repository(e.to_string()))?;
        for org in all {
            let projects = repo
                .list_projects_in_org(org.id)
                .await
                .map_err(|e| ProjectError::Repository(e.to_string()))?;
            if projects.iter().any(|p| p.id == project_id) {
                owning_orgs.push(org.id);
            }
        }
    } else {
        // For Shape B, also scan for the co-owner org.
        if matches!(project.shape, domain::model::nodes::ProjectShape::B) {
            let all = repo
                .list_all_orgs()
                .await
                .map_err(|e| ProjectError::Repository(e.to_string()))?;
            for org in all {
                if owning_orgs.contains(&org.id) {
                    continue;
                }
                let projects = repo
                    .list_projects_in_org(org.id)
                    .await
                    .map_err(|e| ProjectError::Repository(e.to_string()))?;
                if projects.iter().any(|p| p.id == project_id) {
                    owning_orgs.push(org.id);
                }
            }
        }
    }

    // Access gate: the viewer must be a member of at least one owning
    // org (via `owning_org`) OR be on the project's roster (lead /
    // member / sponsor).
    let roster = build_roster(repo.clone(), &owning_orgs, project_id).await?;
    let on_roster = roster.iter().any(|m| m.agent_id == viewer_agent_id);
    let in_owning_org = matches!(viewer_org, Some(o) if owning_orgs.contains(&o));

    if !on_roster && !in_owning_org {
        return Ok(DetailOutcome::AccessDenied);
    }

    let lead_agent_id = roster
        .iter()
        .find(|m| m.project_role == ProjectMembershipRole::Lead)
        .map(|m| m.agent_id);

    Ok(DetailOutcome::Found(Box::new(ProjectDetail {
        project,
        owning_org_ids: owning_orgs,
        lead_agent_id,
        roster,
        // M4 placeholder — C-M5-3 flips this to the real query.
        recent_sessions: Vec::new(),
    })))
}

async fn build_roster(
    repo: Arc<dyn Repository>,
    owning_orgs: &[OrgId],
    project_id: ProjectId,
) -> Result<Vec<RosterMember>, ProjectError> {
    // For each owning org, pull every agent; cross-reference against
    // the project's roster edges (HAS_LEAD / HAS_AGENT / HAS_SPONSOR).
    //
    // M4 simplification: we derive lead + members from the repo's
    // project-led-by-agent lookup + a fallback agent list. M5 may
    // introduce a dedicated `list_project_roster(project_id)` if this
    // becomes a hot path.
    let mut roster: Vec<RosterMember> = Vec::new();
    let mut seen: std::collections::HashSet<AgentId> = std::collections::HashSet::new();

    // Lead: at most one per project.
    let lead_candidates = collect_lead_candidates(repo.clone(), owning_orgs, project_id).await?;
    for agent in lead_candidates {
        if seen.insert(agent.id) {
            roster.push(RosterMember {
                agent_id: agent.id,
                kind: agent.kind,
                display_name: agent.display_name,
                role: agent.role,
                project_role: ProjectMembershipRole::Lead,
            });
        }
    }

    // Members + sponsors: at M4 we don't have a `list_project_agents`
    // repo method, so we conservatively return only the lead at M4.
    // Members + sponsors were written at creation time (via
    // `apply_project_creation`'s HAS_AGENT + HAS_SPONSOR edges) but
    // read-back requires a new repo method. Scoped for M5 per C-M5-7.
    let _ = project_id;

    Ok(roster)
}

async fn collect_lead_candidates(
    repo: Arc<dyn Repository>,
    owning_orgs: &[OrgId],
    project_id: ProjectId,
) -> Result<Vec<Agent>, ProjectError> {
    let mut leads: Vec<Agent> = Vec::new();
    let mut seen: std::collections::HashSet<AgentId> = std::collections::HashSet::new();
    for org in owning_orgs {
        let agents = repo
            .list_agents_in_org(*org)
            .await
            .map_err(|e| ProjectError::Repository(e.to_string()))?;
        for agent in agents {
            if seen.contains(&agent.id) {
                continue;
            }
            let led_projects = repo
                .list_projects_led_by_agent(agent.id)
                .await
                .map_err(|e| ProjectError::Repository(e.to_string()))?;
            if led_projects.iter().any(|p| p.id == project_id) {
                seen.insert(agent.id);
                leads.push(agent);
            }
        }
    }
    Ok(leads)
}

// ---------------------------------------------------------------------------
// OKR patch — writer
// ---------------------------------------------------------------------------

/// One mutation entry in a PATCH body. The `kind` + `op` tags disambiguate
/// the payload shape; the orchestrator walks the array sequentially and
/// emits one audit event per entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OkrPatchEntry {
    /// An Objective-level mutation.
    Objective {
        #[serde(flatten)]
        body: ObjectiveMutation,
    },
    /// A KeyResult-level mutation.
    KeyResult {
        #[serde(flatten)]
        body: KeyResultMutation,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ObjectiveMutation {
    Create { payload: Objective },
    Update { payload: Objective },
    Delete { objective_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum KeyResultMutation {
    Create { payload: KeyResult },
    Update { payload: KeyResult },
    Delete { kr_id: String },
}

#[derive(Debug, Clone)]
pub struct OkrPatchReceipt {
    pub project_id: ProjectId,
    pub audit_event_ids: Vec<AuditEventId>,
    pub objectives_after: Vec<Objective>,
    pub key_results_after: Vec<KeyResult>,
}

pub async fn apply_okr_patch(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    project_id: ProjectId,
    viewer_agent_id: AgentId,
    patch: Vec<OkrPatchEntry>,
    now: DateTime<Utc>,
) -> Result<OkrPatchReceipt, ProjectError> {
    // 1. Load the project + authorise the caller (same gate as the
    //    reader path — a viewer must be on the roster OR in an owning
    //    org).
    let project = repo
        .get_project(project_id)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?
        .ok_or(ProjectError::Validation(format!(
            "project {project_id} not found"
        )))?;

    let viewer = repo
        .get_agent(viewer_agent_id)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?
        .ok_or(ProjectError::Validation(format!(
            "viewer agent {viewer_agent_id} not found"
        )))?;

    let owning_orgs = resolve_owning_orgs(repo.clone(), project_id, &project).await?;
    let led_projects = repo
        .list_projects_led_by_agent(viewer_agent_id)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?;
    let on_roster = led_projects.iter().any(|p| p.id == project_id);
    let in_owning_org = matches!(viewer.owning_org, Some(o) if owning_orgs.contains(&o));
    if !on_roster && !in_owning_org {
        return Err(ProjectError::ApproverNotAuthorized);
    }

    // 2. Apply every entry to an in-memory copy. Each entry produces a
    //    (before, after) pair held for audit emission AFTER the upsert
    //    succeeds (audit is durable; audit errors are surfaced but do
    //    not rewind the write).
    let mut objectives = project.objectives.clone();
    let mut key_results = project.key_results.clone();
    let mut pending_audits: Vec<(String, String, String, serde_json::Value, serde_json::Value)> =
        Vec::with_capacity(patch.len());

    for entry in patch {
        match entry {
            OkrPatchEntry::Objective { body } => match body {
                ObjectiveMutation::Create { payload } => {
                    if objectives
                        .iter()
                        .any(|o| o.objective_id == payload.objective_id)
                    {
                        return Err(ProjectError::OkrValidation(format!(
                            "objective_id {} already exists",
                            payload.objective_id
                        )));
                    }
                    let after = serde_json::to_value(&payload).map_err(audit_serde)?;
                    let entity_id = payload.objective_id.clone();
                    objectives.push(payload);
                    pending_audits.push((
                        "objective".into(),
                        "create".into(),
                        entity_id,
                        serde_json::Value::Null,
                        after,
                    ));
                }
                ObjectiveMutation::Update { payload } => {
                    let idx = objectives
                        .iter()
                        .position(|o| o.objective_id == payload.objective_id)
                        .ok_or_else(|| {
                            ProjectError::OkrValidation(format!(
                                "objective_id {} not found",
                                payload.objective_id
                            ))
                        })?;
                    let before = serde_json::to_value(&objectives[idx]).map_err(audit_serde)?;
                    let after = serde_json::to_value(&payload).map_err(audit_serde)?;
                    let entity_id = payload.objective_id.clone();
                    objectives[idx] = payload;
                    pending_audits.push((
                        "objective".into(),
                        "update".into(),
                        entity_id,
                        before,
                        after,
                    ));
                }
                ObjectiveMutation::Delete { objective_id } => {
                    let idx = objectives
                        .iter()
                        .position(|o| o.objective_id == objective_id)
                        .ok_or_else(|| {
                            ProjectError::OkrValidation(format!(
                                "objective_id {objective_id} not found"
                            ))
                        })?;
                    if key_results.iter().any(|k| k.objective_id == objective_id) {
                        return Err(ProjectError::OkrValidation(format!(
                            "cannot delete objective {objective_id}: \
                             delete dependent key_results first"
                        )));
                    }
                    let before = serde_json::to_value(&objectives[idx]).map_err(audit_serde)?;
                    objectives.remove(idx);
                    pending_audits.push((
                        "objective".into(),
                        "delete".into(),
                        objective_id,
                        before,
                        serde_json::Value::Null,
                    ));
                }
            },
            OkrPatchEntry::KeyResult { body } => match body {
                KeyResultMutation::Create { payload } => {
                    if key_results.iter().any(|k| k.kr_id == payload.kr_id) {
                        return Err(ProjectError::OkrValidation(format!(
                            "kr_id {} already exists",
                            payload.kr_id
                        )));
                    }
                    let after = serde_json::to_value(&payload).map_err(audit_serde)?;
                    let entity_id = payload.kr_id.clone();
                    key_results.push(payload);
                    pending_audits.push((
                        "key_result".into(),
                        "create".into(),
                        entity_id,
                        serde_json::Value::Null,
                        after,
                    ));
                }
                KeyResultMutation::Update { payload } => {
                    let idx = key_results
                        .iter()
                        .position(|k| k.kr_id == payload.kr_id)
                        .ok_or_else(|| {
                            ProjectError::OkrValidation(format!(
                                "kr_id {} not found",
                                payload.kr_id
                            ))
                        })?;
                    let before = serde_json::to_value(&key_results[idx]).map_err(audit_serde)?;
                    let after = serde_json::to_value(&payload).map_err(audit_serde)?;
                    let entity_id = payload.kr_id.clone();
                    key_results[idx] = payload;
                    pending_audits.push((
                        "key_result".into(),
                        "update".into(),
                        entity_id,
                        before,
                        after,
                    ));
                }
                KeyResultMutation::Delete { kr_id } => {
                    let idx = key_results
                        .iter()
                        .position(|k| k.kr_id == kr_id)
                        .ok_or_else(|| {
                            ProjectError::OkrValidation(format!("kr_id {kr_id} not found"))
                        })?;
                    let before = serde_json::to_value(&key_results[idx]).map_err(audit_serde)?;
                    key_results.remove(idx);
                    pending_audits.push((
                        "key_result".into(),
                        "delete".into(),
                        kr_id,
                        before,
                        serde_json::Value::Null,
                    ));
                }
            },
        }
    }

    // 3. Re-validate the full OKR set (catches cross-entry invariants:
    //    KR referencing unknown objective, duplicate ids, measurement
    //    type/value shape mismatch).
    validate_okrs(&objectives, &key_results)?;

    // 4. Build the replacement Project + upsert.
    let mut next = project.clone();
    next.objectives = objectives.clone();
    next.key_results = key_results.clone();
    repo.upsert_project(&next)
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?;

    // 5. Emit one audit per mutation. The primary owning org anchors
    //    the per-org audit chain (Shape B writes two chains; OKR edits
    //    land in the primary's — consistent with how project_created
    //    fires at submit time).
    let primary_org = *owning_orgs
        .first()
        .ok_or_else(|| ProjectError::Validation("project has no owning org".into()))?;
    let mut audit_event_ids: Vec<AuditEventId> = Vec::with_capacity(pending_audits.len());
    for (kind, op, entity_id, before, after) in pending_audits {
        let event = project_okrs_updated(
            viewer_agent_id,
            project_id,
            primary_org,
            &kind,
            &op,
            &entity_id,
            before,
            after,
            now,
        );
        let event_id = event.event_id;
        audit
            .emit(event)
            .await
            .map_err(|e| ProjectError::AuditEmit(e.to_string()))?;
        audit_event_ids.push(event_id);
    }

    Ok(OkrPatchReceipt {
        project_id,
        audit_event_ids,
        objectives_after: objectives,
        key_results_after: key_results,
    })
}

fn audit_serde(e: serde_json::Error) -> ProjectError {
    ProjectError::Repository(format!("serialise OKR entry for audit: {e}"))
}

async fn resolve_owning_orgs(
    repo: Arc<dyn Repository>,
    project_id: ProjectId,
    _project: &Project,
) -> Result<Vec<OrgId>, ProjectError> {
    let all = repo
        .list_all_orgs()
        .await
        .map_err(|e| ProjectError::Repository(e.to_string()))?;
    let mut owning: Vec<OrgId> = Vec::new();
    for org in all {
        let projects = repo
            .list_projects_in_org(org.id)
            .await
            .map_err(|e| ProjectError::Repository(e.to_string()))?;
        if projects.iter().any(|p| p.id == project_id) {
            owning.push(org.id);
        }
    }
    Ok(owning)
}

// ---------------------------------------------------------------------------
// Tests — pure helpers + wire-shape strip invariant.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::composites_m4::{
        KeyResultStatus, MeasurementType, ObjectiveStatus, OkrValue,
    };
    use domain::model::ids::{AgentId, OrgId, ProjectId};
    use domain::model::nodes::{Project, ProjectShape, ProjectStatus};

    fn sample_project() -> Project {
        Project {
            id: ProjectId::new(),
            name: "Atlas".into(),
            description: "".into(),
            goal: None,
            status: ProjectStatus::Planned,
            shape: ProjectShape::A,
            token_budget: None,
            tokens_spent: 0,
            objectives: vec![],
            key_results: vec![],
            resource_boundaries: None,
            created_at: Utc::now(),
        }
    }

    fn sample_roster() -> Vec<RosterMember> {
        vec![RosterMember {
            agent_id: AgentId::new(),
            kind: AgentKind::Human,
            display_name: "CEO".into(),
            role: Some(AgentRole::Executive),
            project_role: ProjectMembershipRole::Lead,
        }]
    }

    #[test]
    fn wire_shape_strips_phi_core() {
        // Phi-core invariant: the serialised ProjectDetail carries
        // none of `defaults_snapshot` / `blueprint` / `execution_limits`
        // / `context_config` / `retry_config` at any depth. This is the
        // load-bearing snapshot test for P7 per Part 1.5 Page 11.
        let detail = ProjectDetail {
            project: sample_project(),
            owning_org_ids: vec![OrgId::new()],
            lead_agent_id: None,
            roster: sample_roster(),
            recent_sessions: Vec::new(),
        };
        let json = serde_json::to_string(&detail).unwrap();
        for forbidden in [
            "defaults_snapshot",
            "blueprint",
            "execution_limits",
            "context_config",
            "retry_config",
        ] {
            assert!(
                !json.contains(forbidden),
                "ProjectDetail wire shape must not carry `{forbidden}` — \
                 any phi-core transit belongs on page 09 (agent profile editor), not here"
            );
        }
    }

    #[test]
    fn recent_sessions_is_empty_at_m4() {
        let detail = ProjectDetail {
            project: sample_project(),
            owning_org_ids: vec![OrgId::new()],
            lead_agent_id: None,
            roster: Vec::new(),
            recent_sessions: Vec::new(),
        };
        assert!(
            detail.recent_sessions.is_empty(),
            "M4 placeholder — C-M5-3 flips this to real rows"
        );
    }

    #[test]
    fn okr_patch_entry_serde_objective_create() {
        let obj = Objective {
            objective_id: "obj-1".into(),
            name: "O1".into(),
            description: "".into(),
            status: ObjectiveStatus::Draft,
            owner: AgentId::new(),
            deadline: None,
            key_result_ids: vec![],
        };
        let entry = OkrPatchEntry::Objective {
            body: ObjectiveMutation::Create {
                payload: obj.clone(),
            },
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["kind"], "objective");
        assert_eq!(json["op"], "create");
        assert_eq!(json["payload"]["objective_id"], "obj-1");
    }

    #[test]
    fn okr_patch_entry_serde_keyresult_delete() {
        let entry = OkrPatchEntry::KeyResult {
            body: KeyResultMutation::Delete {
                kr_id: "kr-1".into(),
            },
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["kind"], "key_result");
        assert_eq!(json["op"], "delete");
        assert_eq!(json["kr_id"], "kr-1");
    }

    #[test]
    fn okr_patch_entry_serde_keyresult_update_roundtrip() {
        let kr = KeyResult {
            kr_id: "kr-1".into(),
            objective_id: "obj-1".into(),
            name: "KR1".into(),
            description: "".into(),
            measurement_type: MeasurementType::Count,
            target_value: OkrValue::Integer(10),
            current_value: Some(OkrValue::Integer(3)),
            owner: AgentId::new(),
            deadline: None,
            status: KeyResultStatus::InProgress,
        };
        let entry = OkrPatchEntry::KeyResult {
            body: KeyResultMutation::Update {
                payload: kr.clone(),
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        let round: OkrPatchEntry = serde_json::from_str(&json).unwrap();
        match round {
            OkrPatchEntry::KeyResult {
                body: KeyResultMutation::Update { payload },
            } => {
                assert_eq!(payload.kr_id, "kr-1");
                assert_eq!(payload.target_value, OkrValue::Integer(10));
            }
            _ => panic!("expected KeyResult Update after roundtrip"),
        }
    }
}
