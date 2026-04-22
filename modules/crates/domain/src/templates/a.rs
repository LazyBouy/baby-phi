//! Template A — **Project Lead Authority**.
//!
//! When a `HAS_LEAD` edge is created from Project P to Agent X, the
//! system auto-issues a grant giving Agent X `[read, inspect, list]`
//! on every session tagged `project:P`. The "team lead" mental model:
//! simple edge-lookup policy, naturally cleaned up when leadership
//! changes (the edge getting deleted revokes the grant).
//!
//! See
//! [`../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md`](../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md)
//! §Template A for the full semantic rules.
//!
//! ## Two-stage lifecycle
//!
//! - **Adoption** (M3/P2): [`build_adoption_request`] mints the
//!   adoption AR the CEO self-approves at org creation time. The AR
//!   authorises the template to fire within the org's scope; it does
//!   NOT itself issue grants.
//! - **Fire** (M4/P2 pure-fn + M4/P3 listener wiring):
//!   [`fire_grant_on_lead_assignment`] constructs the lead-assignment
//!   Grant as a pure function. M4/P3's [`crate::events::listeners`]
//!   wires it to the domain event bus so every `HAS_LEAD` edge write
//!   via `apply_project_creation` triggers grant issuance
//!   automatically. See ADR-0028.
//!
//! Keeping the grant builder pure (no I/O, no Repository reference)
//! is what lets the listener re-call it idempotently + what lets the
//! proptest fuzz the grant shape across 50 cases without standing up
//! a fake bus.

use chrono::{DateTime, Utc};

use crate::model::ids::{AgentId, AuthRequestId, GrantId, ProjectId};
use crate::model::nodes::{AuthRequest, Grant, PrincipalRef, ResourceRef, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-A adoption AR for the given org + CEO.
///
/// Pure fn — no I/O. See the shared helper
/// [`super::adoption::build_adoption_request`] for the underlying
/// construction details.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::A, args)
}

// ---------------------------------------------------------------------------
// M4/P2 — pure-fn grant builder for the Template-A firing path.
// ---------------------------------------------------------------------------

/// Inputs the M4/P3 `TemplateAFireListener` supplies each time a
/// `HAS_LEAD` edge is emitted on the domain event bus. Separating the
/// call site from the pure-fn keeps the builder easily proptest-able.
#[derive(Debug, Clone)]
pub struct FireArgs {
    /// The project the lead was just assigned to.
    pub project: ProjectId,
    /// The agent that just became project lead.
    pub lead: AgentId,
    /// The Template-A adoption AR id that authorises this firing.
    /// The emitted Grant's `descends_from` field points here —
    /// revoking the adoption AR cascades to every grant that has
    /// fired under it.
    pub adoption_auth_request_id: AuthRequestId,
    /// Wall-clock time for `issued_at`. Injected so tests can pin
    /// deterministic timestamps.
    pub now: DateTime<Utc>,
}

/// Build the `[read, inspect, list]` Grant for the lead on
/// `project:<id>`. Pure — no I/O, no Repository. Callers persist via
/// [`crate::Repository::create_grant`] + emit the companion audit
/// event.
///
/// Grant shape (invariants pinned by the
/// `template_a_fire_grant_shape_props` proptest at 50 cases):
/// - `holder == PrincipalRef::Agent(args.lead)`.
/// - `action == ["read", "inspect", "list"]` (stable order — the
///   engine's grant-resolution path is order-insensitive but tests
///   + audit diffs assert on the fixed ordering).
/// - `resource.uri == "project:<uuid>"` with the project's UUID
///   fully expanded (no `project_id` placeholder).
/// - `descends_from == Some(args.adoption_auth_request_id)` — the
///   adoption AR provenance that authorises the fire.
/// - `delegable == false` — the lead cannot hand off the baton
///   transitively; leadership is renewed per-assignment (the edge
///   drives the grant, so delegation would decouple from edge
///   state).
/// - `fundamentals == [Tag]` — the resource URI is an instance URI
///   under the `#kind:project` tag; explicit fundamentals lets the
///   engine resolve the grant without relying on URI-derivation.
/// - `revoked_at == None` — a fresh grant.
pub fn fire_grant_on_lead_assignment(args: FireArgs) -> Grant {
    let FireArgs {
        project,
        lead,
        adoption_auth_request_id,
        now,
    } = args;
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(lead),
        action: vec![
            "read".to_string(),
            "inspect".to_string(),
            "list".to_string(),
        ],
        resource: ResourceRef {
            uri: format!("project:{project}"),
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
    use crate::model::ids::{AgentId, OrgId};
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
    fn template_a_adoption_carries_template_a_tag() {
        let ar = build_adoption_request(args());
        assert!(ar.kinds.iter().any(|k| k == "#template:a"));
        assert_eq!(ar.state, AuthRequestState::Approved);
    }

    #[test]
    fn template_a_adoption_references_org_scoped_resource_uri() {
        let a = args();
        let expected_uri = format!("org:{}/template:a", a.org_id);
        let ar = build_adoption_request(a);
        assert_eq!(ar.resource_slots[0].resource.uri, expected_uri);
    }

    // ---- M4/P2: fire_grant_on_lead_assignment ---------------------------

    fn fire_args() -> FireArgs {
        FireArgs {
            project: ProjectId::new(),
            lead: AgentId::new(),
            adoption_auth_request_id: AuthRequestId::new(),
            now: Utc::now(),
        }
    }

    #[test]
    fn fire_grant_holder_is_the_lead_agent() {
        let a = fire_args();
        let expected_lead = a.lead;
        let g = fire_grant_on_lead_assignment(a);
        match g.holder {
            PrincipalRef::Agent(id) => assert_eq!(id, expected_lead),
            other => panic!("expected Agent holder, got {other:?}"),
        }
    }

    #[test]
    fn fire_grant_action_is_read_inspect_list_in_stable_order() {
        let g = fire_grant_on_lead_assignment(fire_args());
        assert_eq!(g.action, vec!["read", "inspect", "list"]);
    }

    #[test]
    fn fire_grant_resource_uri_names_the_project_uuid() {
        let a = fire_args();
        let expected_uri = format!("project:{}", a.project);
        let g = fire_grant_on_lead_assignment(a);
        assert_eq!(g.resource.uri, expected_uri);
    }

    #[test]
    fn fire_grant_descends_from_the_adoption_ar() {
        let a = fire_args();
        let expected_ar = a.adoption_auth_request_id;
        let g = fire_grant_on_lead_assignment(a);
        assert_eq!(g.descends_from, Some(expected_ar));
    }

    #[test]
    fn fire_grant_is_non_delegable_and_unrevoked() {
        let g = fire_grant_on_lead_assignment(fire_args());
        assert!(!g.delegable, "lead grants are non-delegable by design");
        assert!(g.revoked_at.is_none());
    }

    #[test]
    fn fire_grant_carries_tag_fundamental_for_instance_uri_resolution() {
        // Instance URIs (e.g. `project:<uuid>`) need explicit
        // fundamentals for the resolver; see
        // `crate::permissions::expansion::resolve_grant` + ADR-0018.
        let g = fire_grant_on_lead_assignment(fire_args());
        assert_eq!(g.fundamentals, vec![crate::model::Fundamental::Tag]);
    }

    #[test]
    fn fire_grant_ids_are_distinct_across_calls() {
        let a = fire_grant_on_lead_assignment(fire_args());
        let b = fire_grant_on_lead_assignment(fire_args());
        assert_ne!(a.id, b.id, "every fire must mint a fresh GrantId");
    }
}
