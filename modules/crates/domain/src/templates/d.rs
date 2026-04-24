//! Template D — **Project-Scoped Role**.
//!
//! When Agent Y is assigned a `supervisor` role on Project P (edge
//! property on `HAS_AGENT`), the system auto-issues a grant giving
//! Agent Y `[read, inspect]` on sessions tagged
//! `project:P AND role_at_creation:worker`. Honours project
//! boundaries — no cross-project surveillance; only sees worker
//! sessions, not other supervisors'.
//!
//! See
//! [`../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md`](../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md)
//! §Template D.
//!
//! ## Two-stage lifecycle (mirrors Template A + C)
//!
//! - **Adoption** (M3/P2): [`build_adoption_request`] mints the
//!   CEO-self-approved adoption AR at org creation time.
//! - **Fire** (M5/P3 pure-fn):
//!   [`fire_grant_on_has_agent_supervisor`] constructs the
//!   project-scoped `[read, inspect]` Grant. M5/P3's
//!   [`crate::events::listeners::TemplateDFireListener`] wires it
//!   to the domain event bus.

use chrono::{DateTime, Utc};

use crate::model::ids::{AgentId, AuthRequestId, GrantId, ProjectId};
use crate::model::nodes::{AuthRequest, Grant, PrincipalRef, ResourceRef, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-D adoption AR for the given org + CEO.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::D, args)
}

// ---------------------------------------------------------------------------
// M5/P3 — pure-fn grant builder for the Template-D firing path.
// ---------------------------------------------------------------------------

/// Inputs the M5/P3 `TemplateDFireListener` supplies each time a
/// `HAS_AGENT_SUPERVISOR` edge is emitted on the domain event bus.
#[derive(Debug, Clone)]
pub struct FireArgs {
    /// The project the supervisor relationship is scoped to.
    pub project: ProjectId,
    /// The agent that just became the project supervisor.
    pub supervisor: AgentId,
    /// The agent the supervisor now oversees (within this project).
    pub supervisee: AgentId,
    /// The Template-D adoption AR id that authorises this firing.
    pub adoption_auth_request_id: AuthRequestId,
    /// Wall-clock time for `issued_at`.
    pub now: DateTime<Utc>,
}

/// Build the `[read, inspect]` Grant for the supervisor on
/// `project:<p>/agent:<supervisee>`. Pure — no I/O, no Repository.
///
/// Grant shape (invariants pinned by the
/// `template_d_fire_grant_shape_props` proptest at 50 cases):
/// - `holder == PrincipalRef::Agent(args.supervisor)`.
/// - `action == ["read", "inspect"]`.
/// - `resource.uri == "project:<puuid>/agent:<suuid>"` — the
///   project-scoped instance URI (no cross-project surveillance).
/// - `descends_from == Some(args.adoption_auth_request_id)`.
/// - `delegable == false`.
/// - `fundamentals == [Tag]`.
/// - `revoked_at == None`.
pub fn fire_grant_on_has_agent_supervisor(args: FireArgs) -> Grant {
    let FireArgs {
        project,
        supervisor,
        supervisee,
        adoption_auth_request_id,
        now,
    } = args;
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(supervisor),
        action: vec!["read".to_string(), "inspect".to_string()],
        resource: ResourceRef {
            uri: format!("project:{project}/agent:{supervisee}"),
        },
        fundamentals: vec![crate::model::Fundamental::Tag],
        descends_from: Some(adoption_auth_request_id),
        delegable: false,
        issued_at: now,
        revoked_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, AuthRequestId, OrgId};
    use crate::model::nodes::{AuthRequestState, PrincipalRef};
    use chrono::Utc;

    fn args() -> AdoptionArgs {
        AdoptionArgs {
            org_id: OrgId::new(),
            ceo: PrincipalRef::Agent(AgentId::new()),
            now: Utc::now(),
        }
    }

    #[test]
    fn template_d_adoption_carries_template_d_tag() {
        let ar = build_adoption_request(args());
        assert!(ar.kinds.iter().any(|k| k == "#template:d"));
        assert_eq!(ar.state, AuthRequestState::Approved);
    }

    #[test]
    fn template_d_adoption_references_org_scoped_resource_uri() {
        let a = args();
        let expected_uri = format!("org:{}/template:d", a.org_id);
        let ar = build_adoption_request(a);
        assert_eq!(ar.resource_slots[0].resource.uri, expected_uri);
    }

    // ---- M5/P3: fire_grant_on_has_agent_supervisor ---------------------

    fn fire_args() -> FireArgs {
        FireArgs {
            project: ProjectId::new(),
            supervisor: AgentId::new(),
            supervisee: AgentId::new(),
            adoption_auth_request_id: AuthRequestId::new(),
            now: Utc::now(),
        }
    }

    #[test]
    fn fire_grant_holder_is_the_supervisor() {
        let a = fire_args();
        let expected_supervisor = a.supervisor;
        let g = fire_grant_on_has_agent_supervisor(a);
        match g.holder {
            PrincipalRef::Agent(id) => assert_eq!(id, expected_supervisor),
            other => panic!("expected Agent holder, got {other:?}"),
        }
    }

    #[test]
    fn fire_grant_action_is_read_inspect_in_stable_order() {
        let g = fire_grant_on_has_agent_supervisor(fire_args());
        assert_eq!(g.action, vec!["read", "inspect"]);
    }

    #[test]
    fn fire_grant_resource_uri_names_project_then_supervisee() {
        let a = fire_args();
        let expected_uri = format!("project:{}/agent:{}", a.project, a.supervisee);
        let g = fire_grant_on_has_agent_supervisor(a);
        assert_eq!(g.resource.uri, expected_uri);
    }

    #[test]
    fn fire_grant_descends_from_the_adoption_ar() {
        let a = fire_args();
        let expected_ar = a.adoption_auth_request_id;
        let g = fire_grant_on_has_agent_supervisor(a);
        assert_eq!(g.descends_from, Some(expected_ar));
    }

    #[test]
    fn fire_grant_is_non_delegable_and_unrevoked() {
        let g = fire_grant_on_has_agent_supervisor(fire_args());
        assert!(!g.delegable);
        assert!(g.revoked_at.is_none());
    }

    #[test]
    fn fire_grant_carries_tag_fundamental() {
        let g = fire_grant_on_has_agent_supervisor(fire_args());
        assert_eq!(g.fundamentals, vec![crate::model::Fundamental::Tag]);
    }
}
