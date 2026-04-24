//! Template C — **Hierarchical Org Chart**.
//!
//! When Agent X is appointed to a node in the organisation tree
//! (e.g. `org:acme/eng/web/lead`), the system auto-issues a grant
//! giving Agent X `[read, inspect, list]` on sessions matching
//! `tags any_match org:acme/eng/web/**`. Matches real-world multi-
//! level supervision via subtree prefix matching; requires explicit
//! org-tree modelling (`HAS_SUBORGANIZATION` edges).
//!
//! See
//! [`../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md`](../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md)
//! §Template C.
//!
//! ## Two-stage lifecycle (mirrors Template A)
//!
//! - **Adoption** (M3/P2): [`build_adoption_request`] mints the
//!   CEO-self-approved adoption AR at org creation time.
//! - **Fire** (M5/P3 pure-fn): [`fire_grant_on_manages_edge`]
//!   constructs the `[read, inspect]` Grant as a pure function. The
//!   M5/P3 [`crate::events::listeners::TemplateCFireListener`] wires
//!   it to the domain event bus so every `MANAGES` edge emission
//!   triggers grant issuance automatically.

use chrono::{DateTime, Utc};

use crate::model::ids::{AgentId, AuthRequestId, GrantId};
use crate::model::nodes::{AuthRequest, Grant, PrincipalRef, ResourceRef, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-C adoption AR for the given org + CEO.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::C, args)
}

// ---------------------------------------------------------------------------
// M5/P3 — pure-fn grant builder for the Template-C firing path.
// ---------------------------------------------------------------------------

/// Inputs the M5/P3 `TemplateCFireListener` supplies each time a
/// `MANAGES` edge is emitted on the domain event bus.
#[derive(Debug, Clone)]
pub struct FireArgs {
    /// The agent that just became the manager.
    pub manager: AgentId,
    /// The agent that just became the subordinate.
    pub subordinate: AgentId,
    /// The Template-C adoption AR id that authorises this firing.
    /// Revoking the adoption AR cascades to every grant that has
    /// fired under it.
    pub adoption_auth_request_id: AuthRequestId,
    /// Wall-clock time for `issued_at`.
    pub now: DateTime<Utc>,
}

/// Build the `[read, inspect]` Grant for the manager on
/// `agent:<subordinate>`. Pure — no I/O, no Repository. Callers
/// persist via [`crate::Repository::create_grant`] + emit the
/// companion audit event.
///
/// Grant shape (invariants pinned by the
/// `template_c_fire_grant_shape_props` proptest at 50 cases):
/// - `holder == PrincipalRef::Agent(args.manager)`.
/// - `action == ["read", "inspect"]` (stable order — no `list`;
///   Template C is narrower than Template A because a subordinate
///   is a single resource, not a project-wide cohort).
/// - `resource.uri == "agent:<uuid>"` with the subordinate's UUID
///   fully expanded.
/// - `descends_from == Some(args.adoption_auth_request_id)`.
/// - `delegable == false`.
/// - `fundamentals == [Tag]` — instance URI, needs explicit Tag
///   fundamentals for the resolver (ADR-0018).
/// - `revoked_at == None`.
pub fn fire_grant_on_manages_edge(args: FireArgs) -> Grant {
    let FireArgs {
        manager,
        subordinate,
        adoption_auth_request_id,
        now,
    } = args;
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(manager),
        action: vec!["read".to_string(), "inspect".to_string()],
        resource: ResourceRef {
            uri: format!("agent:{subordinate}"),
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
    fn template_c_adoption_carries_template_c_tag() {
        let ar = build_adoption_request(args());
        assert!(ar.kinds.iter().any(|k| k == "#template:c"));
        assert_eq!(ar.state, AuthRequestState::Approved);
    }

    #[test]
    fn template_c_adoption_references_org_scoped_resource_uri() {
        let a = args();
        let expected_uri = format!("org:{}/template:c", a.org_id);
        let ar = build_adoption_request(a);
        assert_eq!(ar.resource_slots[0].resource.uri, expected_uri);
    }

    // ---- M5/P3: fire_grant_on_manages_edge ------------------------------

    fn fire_args() -> FireArgs {
        FireArgs {
            manager: AgentId::new(),
            subordinate: AgentId::new(),
            adoption_auth_request_id: AuthRequestId::new(),
            now: Utc::now(),
        }
    }

    #[test]
    fn fire_grant_holder_is_the_manager() {
        let a = fire_args();
        let expected_manager = a.manager;
        let g = fire_grant_on_manages_edge(a);
        match g.holder {
            PrincipalRef::Agent(id) => assert_eq!(id, expected_manager),
            other => panic!("expected Agent holder, got {other:?}"),
        }
    }

    #[test]
    fn fire_grant_action_is_read_inspect_in_stable_order() {
        let g = fire_grant_on_manages_edge(fire_args());
        assert_eq!(g.action, vec!["read", "inspect"]);
    }

    #[test]
    fn fire_grant_resource_uri_names_the_subordinate_uuid() {
        let a = fire_args();
        let expected_uri = format!("agent:{}", a.subordinate);
        let g = fire_grant_on_manages_edge(a);
        assert_eq!(g.resource.uri, expected_uri);
    }

    #[test]
    fn fire_grant_descends_from_the_adoption_ar() {
        let a = fire_args();
        let expected_ar = a.adoption_auth_request_id;
        let g = fire_grant_on_manages_edge(a);
        assert_eq!(g.descends_from, Some(expected_ar));
    }

    #[test]
    fn fire_grant_is_non_delegable_and_unrevoked() {
        let g = fire_grant_on_manages_edge(fire_args());
        assert!(!g.delegable);
        assert!(g.revoked_at.is_none());
    }

    #[test]
    fn fire_grant_carries_tag_fundamental() {
        let g = fire_grant_on_manages_edge(fire_args());
        assert_eq!(g.fundamentals, vec![crate::model::Fundamental::Tag]);
    }
}
