//! Audit-event builders for page 10 (project creation wizard) + page
//! 11 (project detail).
//!
//! Three event types ship with M4/P2:
//!
//! - `platform.project.created` (Alerted) — Shape A happy path, or
//!   Shape B post-both-approve materialisation. Diff carries the
//!   Project struct + shape + lead_agent_id + co-owner list for
//!   Shape B.
//! - `platform.project_creation.pending` (Logged) — Shape B submit:
//!   AR created with two approver slots, no project yet. Diff
//!   carries the AR id so the dashboard's `PendingAuthRequests`
//!   panel can render the waiting state.
//! - `platform.project_creation.denied` (Alerted) — Shape B both-
//!   deny outcome (or mixed A/D / D/A landing in Partial). Diff
//!   carries the AR id + denying-approver reasons for the incident
//!   trail.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId, ProjectId};
use crate::model::nodes::{Project, ProjectShape};

#[allow(clippy::too_many_arguments)]
fn scaffold(
    event_type: &str,
    actor: AgentId,
    target: NodeId,
    org: OrgId,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    audit_class: AuditClass,
    provenance_auth_request_id: Option<AuthRequestId>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(target),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id,
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

/// `platform.project.created` — Alerted.
///
/// Emitted after M4/P6's `apply_project_creation` compound tx commits
/// (Shape A) or after the post-both-approve materialisation path
/// (Shape B). `co_owner_orgs` carries both org ids for Shape B;
/// Shape A leaves it as a single-element Vec with the owning org id.
pub fn project_created(
    actor: AgentId,
    project: &Project,
    owning_org: OrgId,
    co_owner_orgs: &[OrgId],
    lead_agent_id: AgentId,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "project_id":   project.id.to_string(),
            "name":         project.name,
            "description":  project.description,
            "goal":         project.goal,
            "shape":        project.shape.as_str(),
            "status":       project.status.as_str(),
            "token_budget": project.token_budget,
            "objectives_count":  project.objectives.len(),
            "key_results_count": project.key_results.len(),
            "owning_org":   owning_org.to_string(),
            "co_owner_orgs": co_owner_orgs
                                .iter()
                                .map(|o| o.to_string())
                                .collect::<Vec<_>>(),
            "lead_agent_id": lead_agent_id.to_string(),
            "created_at":   project.created_at,
        },
    });
    scaffold(
        "platform.project.created",
        actor,
        NodeId::from_uuid(*project.id.as_uuid()),
        owning_org,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.project_creation.pending` — Logged.
///
/// Emitted at Shape B submit: two-approver AR created, project not
/// yet materialised. The `auth_request_id` + `approver_agent_ids`
/// fields let the M4/P5 dashboard render the waiting state without
/// a second query.
pub fn project_creation_pending(
    actor: AgentId,
    org: OrgId,
    auth_request_id: AuthRequestId,
    proposed_project_name: String,
    co_owner_orgs: &[OrgId],
    approver_agent_ids: &[AgentId],
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "auth_request_id":  auth_request_id.to_string(),
            "shape":            ProjectShape::B.as_str(),
            "proposed_name":    proposed_project_name,
            "co_owner_orgs":    co_owner_orgs
                                    .iter()
                                    .map(|o| o.to_string())
                                    .collect::<Vec<_>>(),
            "approver_agent_ids": approver_agent_ids
                                      .iter()
                                      .map(|a| a.to_string())
                                      .collect::<Vec<_>>(),
        },
    });
    scaffold(
        "platform.project_creation.pending",
        actor,
        NodeId::from_uuid(*auth_request_id.as_uuid()),
        org,
        timestamp,
        diff,
        AuditClass::Logged,
        Some(auth_request_id),
    )
}

/// `platform.project_creation.denied` — Alerted.
///
/// Emitted on Shape B both-deny OR mixed outcome (`Partial` terminal
/// state with at least one deny). `denying_approvers` captures who
/// denied + their reason text so the incident trail is complete.
pub fn project_creation_denied(
    actor: AgentId,
    org: OrgId,
    auth_request_id: AuthRequestId,
    proposed_project_name: String,
    denying_approvers: &[(AgentId, Option<String>)],
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "auth_request_id":  auth_request_id.to_string(),
            "proposed_name":    proposed_project_name,
            "denying_approvers": denying_approvers
                                     .iter()
                                     .map(|(id, reason)| serde_json::json!({
                                         "agent_id": id.to_string(),
                                         "reason":   reason,
                                     }))
                                     .collect::<Vec<_>>(),
        },
    });
    scaffold(
        "platform.project_creation.denied",
        actor,
        NodeId::from_uuid(*auth_request_id.as_uuid()),
        org,
        timestamp,
        diff,
        AuditClass::Alerted,
        Some(auth_request_id),
    )
}

/// Wrap a `ProjectId` as a `NodeId` — project rows live in the
/// entity namespace, not the edge namespace.
fn project_node_id(id: ProjectId) -> NodeId {
    NodeId::from_uuid(*id.as_uuid())
}

/// `platform.project.okr_updated` — Logged.
///
/// Emitted once per OKR mutation applied via the M4/P7 in-place OKR
/// editor (`PATCH /api/v0/projects/:id/okrs`). One event per entry in
/// the patch array — operators see the sequence in the per-org audit
/// chain and can replay it row by row.
///
/// The `op` field records `create` / `update` / `delete`; the `kind`
/// field records `objective` / `key_result`. `before` carries the
/// pre-image (None on create) and `after` the post-image (None on
/// delete). The reader drives diff rendering from this pair.
#[allow(clippy::too_many_arguments)]
pub fn project_okrs_updated(
    actor: AgentId,
    project_id: ProjectId,
    org: OrgId,
    kind: &str,
    op: &str,
    entity_id: &str,
    before: serde_json::Value,
    after: serde_json::Value,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "kind":       kind,
        "op":         op,
        "entity_id":  entity_id,
        "before":     before,
        "after":      after,
    });
    scaffold(
        "platform.project.okr_updated",
        actor,
        project_node_id(project_id),
        org,
        timestamp,
        diff,
        AuditClass::Logged,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::composites_m4::ResourceBoundaries;
    use crate::model::nodes::{ProjectShape, ProjectStatus};

    fn sample_project(shape: ProjectShape) -> Project {
        Project {
            id: ProjectId::new(),
            name: "Atlas".into(),
            description: "Moonshot memory benchmark".into(),
            goal: Some("0.85 recall at 10k tokens".into()),
            status: ProjectStatus::Planned,
            shape,
            token_budget: Some(1_000_000),
            tokens_spent: 0,
            objectives: vec![],
            key_results: vec![],
            resource_boundaries: Some(ResourceBoundaries::default()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn project_created_is_alerted_and_carries_shape_lead() {
        let p = sample_project(ProjectShape::A);
        let lead = AgentId::new();
        let org = OrgId::new();
        let ev = project_created(AgentId::new(), &p, org, &[org], lead, None, Utc::now());
        assert_eq!(ev.event_type, "platform.project.created");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.diff["after"]["shape"], "shape_a");
        assert_eq!(ev.diff["after"]["lead_agent_id"], lead.to_string());
    }

    #[test]
    fn project_created_shape_b_records_two_co_owners() {
        let p = sample_project(ProjectShape::B);
        let org_a = OrgId::new();
        let org_b = OrgId::new();
        let ev = project_created(
            AgentId::new(),
            &p,
            org_a,
            &[org_a, org_b],
            AgentId::new(),
            None,
            Utc::now(),
        );
        let owners = ev.diff["after"]["co_owner_orgs"]
            .as_array()
            .expect("co_owner_orgs serialises as array");
        assert_eq!(owners.len(), 2);
        assert_eq!(ev.diff["after"]["shape"], "shape_b");
    }

    #[test]
    fn project_creation_pending_is_logged_and_links_ar() {
        let ar = AuthRequestId::new();
        let ev = project_creation_pending(
            AgentId::new(),
            OrgId::new(),
            ar,
            "Atlas".into(),
            &[OrgId::new(), OrgId::new()],
            &[AgentId::new(), AgentId::new()],
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.project_creation.pending");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.provenance_auth_request_id, Some(ar));
        let approvers = ev.diff["after"]["approver_agent_ids"]
            .as_array()
            .expect("approver_agent_ids serialises as array");
        assert_eq!(approvers.len(), 2);
    }

    #[test]
    fn project_okrs_updated_is_logged_and_carries_before_after() {
        let pid = ProjectId::new();
        let org = OrgId::new();
        let ev = project_okrs_updated(
            AgentId::new(),
            pid,
            org,
            "objective",
            "update",
            "obj-1",
            serde_json::json!({ "status": "draft" }),
            serde_json::json!({ "status": "active" }),
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.project.okr_updated");
        assert_eq!(ev.audit_class, AuditClass::Logged);
        assert_eq!(ev.org_scope, Some(org));
        assert_eq!(ev.diff["kind"], "objective");
        assert_eq!(ev.diff["op"], "update");
        assert_eq!(ev.diff["entity_id"], "obj-1");
        assert_eq!(ev.diff["before"]["status"], "draft");
        assert_eq!(ev.diff["after"]["status"], "active");
    }

    #[test]
    fn project_creation_denied_is_alerted_and_carries_reasons() {
        let ar = AuthRequestId::new();
        let denier = AgentId::new();
        let ev = project_creation_denied(
            AgentId::new(),
            OrgId::new(),
            ar,
            "Atlas".into(),
            &[(denier, Some("scope too wide".into()))],
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.project_creation.denied");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        let deniers = ev.diff["after"]["denying_approvers"]
            .as_array()
            .expect("denying_approvers is an array");
        assert_eq!(deniers.len(), 1);
        assert_eq!(deniers[0]["agent_id"], denier.to_string());
        assert_eq!(deniers[0]["reason"], "scope too wide");
    }
}
