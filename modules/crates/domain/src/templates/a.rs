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

use crate::audit::AuditClass;
use crate::model::ids::{AgentId, AuthRequestId, GrantId, ProjectId};
use crate::model::nodes::{AuthRequest, Grant, PrincipalRef, ResourceRef, TemplateKind};
use crate::model::Fundamental;

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
    /// CH-13 / ADR-0050 §D50.5 — strictest-wins composed
    /// [`AuditClass`] for the minted Grant. The P2 listener wiring
    /// computes this from
    /// [`crate::permissions::compose_audit_class_with_source`] over
    /// (org_default, template_ar, optional_override) and passes it
    /// through to the pure-fn at fire time. Pure-fn discipline
    /// preserved (no I/O, no Repository — composition happens above
    /// the pure-fn boundary in the listener body).
    pub audit_class: AuditClass,
}

/// Build the paired Grants for the lead at lead-assignment time.
/// Pure — no I/O, no Repository. Callers persist via
/// [`crate::Repository::create_grant`] + emit one companion audit
/// event per Grant.
///
/// **CH-15 / ADR-0054 §D54.3** — the return type is now
/// [`Vec<Grant>`] (CascadeResult-style typed-multi-value precedent
/// per CH-14 retro v7). Two grants are minted per `HAS_LEAD` edge:
///
/// 1. `grants[0]` — the **project-resource grant** (preserved
///    verbatim from pre-CH-15 shape; `[read, inspect, list]` on
///    `project:<uuid>` with `fundamentals = [Tag]`).
/// 2. `grants[1]` — the NEW **session-object grant** covering session
///    instances tagged `project:<uuid>` (`[read, inspect, list]` on
///    `session_object/project:<uuid>` selector form with
///    `fundamentals = [DataObject, Tag]` per concept doc 01 §"Composite
///    Classes" — `SessionObject` expands to `[DataObject, Tag]`).
///
/// Both grants carry the same `holder`, `action` ordering,
/// `descends_from` (the adoption AR), `delegable = false`,
/// `issued_at`, `revoked_at = None`, `approval_mode`, `audit_class`,
/// and `allocate_refinement = None`. They differ only in
/// `resource.uri` + `fundamentals`. The `id` is freshly minted for
/// each grant.
///
/// Grant-shape invariants pinned by the
/// `template_a_fire_grant_shape_props` proptest at 50 cases (now
/// asserting `grants.len() == 2` + per-grant invariants):
/// - `grants[0].holder == grants[1].holder == PrincipalRef::Agent(args.lead)`.
/// - Both grants' `action == [Read, Inspect, List]` (stable order).
/// - `grants[0].resource.uri == "project:<uuid>"`.
/// - `grants[1].resource.uri == "session_object/project:<uuid>"` —
///   the selector-form URI matches concept doc 07 §"Template A —
///   Project Lead Authority" verbatim ("every session tagged
///   `project:P`"). The engine reads `tags contains project:<uuid>`
///   on the resolution path; the URI carries the same predicate as
///   a literal so `expansion::resolve_grant` can match instance URIs
///   carrying that tag.
/// - Both `descends_from == Some(args.adoption_auth_request_id)` —
///   shared provenance; the Authority Chain (concept doc 04
///   §"The Authority Chain") is preserved.
/// - Both `delegable == false`.
/// - `grants[0].fundamentals == [Tag]` (matches pre-CH-15 shape).
/// - `grants[1].fundamentals == [DataObject, Tag]` (matches
///   `SessionObject` composite expansion per concept doc 01).
/// - Both `revoked_at == None`.
/// - `grants[0].id != grants[1].id` — fresh GrantId per grant.
pub fn fire_grant_on_lead_assignment(args: FireArgs) -> Vec<Grant> {
    let FireArgs {
        project,
        lead,
        adoption_auth_request_id,
        now,
        audit_class,
    } = args;
    let project_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(lead),
        action: vec![
            crate::permissions::action::Action::Read,
            crate::permissions::action::Action::Inspect,
            crate::permissions::action::Action::List,
        ],
        resource: ResourceRef {
            uri: format!("project:{project}"),
        },
        fundamentals: vec![Fundamental::Tag],
        descends_from: Some(adoption_auth_request_id),
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode: crate::model::ApprovalMode::Implicit,
        audit_class,
        allocate_refinement: None,
    };
    // CH-15 / ADR-0054 §D54.3 — paired session_object grant covering
    // session instances tagged `project:<uuid>`. Concept doc 07
    // §"Template A — Project Lead Authority" specifies the lead has
    // [read, inspect, list] on every session tagged `project:P`; this
    // grant materialises that on the wire as a selector-grammar
    // predicate (`tags contains "project:<uuid>"` AND `tags contains
    // #kind:session`). The project-tag predicate uses the
    // string-literal form because the grammar's `namespace_tag`
    // identifier requires `[A-Za-z_][A-Za-z0-9_-]*` and UUIDs that
    // start with a digit fail to parse (concept-09 §"Tag Forms"). The
    // selector parser at
    // [`crate::permissions::selector::parse_selector`] resolves the
    // expression to a `Selector::Bool` that the engine evaluates
    // against the synthetic launch ToolCall's `target_tags` set
    // (which the launch handler populates with `project:<uuid>` +
    // `org:<uuid>` + `#kind:session` per ADR-0054 §D54.1).
    let session_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(lead),
        action: vec![
            crate::permissions::action::Action::Read,
            crate::permissions::action::Action::Inspect,
            crate::permissions::action::Action::List,
        ],
        resource: ResourceRef {
            uri: format!(r#"tags contains "project:{project}" AND tags contains #kind:session"#),
        },
        fundamentals: vec![Fundamental::DataObject, Fundamental::Tag],
        descends_from: Some(adoption_auth_request_id),
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode: crate::model::ApprovalMode::Implicit,
        audit_class,
        allocate_refinement: None,
    };
    vec![project_grant, session_grant]
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
            audit_class: AuditClass::Silent,
        }
    }

    #[test]
    fn fire_grant_holder_is_the_lead_agent() {
        let a = fire_args();
        let expected_lead = a.lead;
        let grants = fire_grant_on_lead_assignment(a);
        for g in &grants {
            match g.holder {
                PrincipalRef::Agent(id) => assert_eq!(id, expected_lead),
                ref other => panic!("expected Agent holder, got {other:?}"),
            }
        }
    }

    #[test]
    fn fire_grant_action_is_read_inspect_list_in_stable_order() {
        let grants = fire_grant_on_lead_assignment(fire_args());
        for g in &grants {
            assert_eq!(
                g.action,
                vec![
                    crate::permissions::action::Action::Read,
                    crate::permissions::action::Action::Inspect,
                    crate::permissions::action::Action::List,
                ]
            );
        }
    }

    #[test]
    fn fire_grant_resource_uri_names_the_project_uuid() {
        let a = fire_args();
        let expected_uri = format!("project:{}", a.project);
        let grants = fire_grant_on_lead_assignment(a);
        // grants[0] is the project-resource grant (CH-15 / ADR-0054
        // §D54.3 — first is the project grant, second is the
        // session_object grant).
        assert_eq!(grants[0].resource.uri, expected_uri);
    }

    #[test]
    fn fire_grant_descends_from_the_adoption_ar() {
        let a = fire_args();
        let expected_ar = a.adoption_auth_request_id;
        let grants = fire_grant_on_lead_assignment(a);
        for g in &grants {
            assert_eq!(g.descends_from, Some(expected_ar));
        }
    }

    #[test]
    fn fire_grant_is_non_delegable_and_unrevoked() {
        let grants = fire_grant_on_lead_assignment(fire_args());
        for g in &grants {
            assert!(!g.delegable, "lead grants are non-delegable by design");
            assert!(g.revoked_at.is_none());
        }
    }

    #[test]
    fn fire_grant_carries_tag_fundamental_for_instance_uri_resolution() {
        // Instance URIs (e.g. `project:<uuid>`) need explicit
        // fundamentals for the resolver; see
        // `crate::permissions::expansion::resolve_grant` + ADR-0018.
        let grants = fire_grant_on_lead_assignment(fire_args());
        // grants[0] is the project-resource grant; pre-CH-15 shape
        // pinned `fundamentals == [Tag]`.
        assert_eq!(grants[0].fundamentals, vec![Fundamental::Tag]);
    }

    #[test]
    fn fire_grant_ids_are_distinct_across_calls() {
        let a = fire_grant_on_lead_assignment(fire_args());
        let b = fire_grant_on_lead_assignment(fire_args());
        assert_ne!(a[0].id, b[0].id, "every fire must mint a fresh GrantId");
        assert_ne!(a[1].id, b[1].id, "every fire must mint a fresh GrantId");
    }

    // ---- CH-15 / ADR-0054 §D54.3 — Vec<Grant> + paired session grant ----

    /// CH-15 invariant: every fire mints exactly TWO grants.
    #[test]
    fn fire_grant_returns_two_grants_per_call() {
        let grants = fire_grant_on_lead_assignment(fire_args());
        assert_eq!(
            grants.len(),
            2,
            "CH-15: each fire mints (project_grant, session_grant) pair"
        );
    }

    /// CH-15 back-compat: the first grant's shape matches the
    /// pre-CH-15 single-grant shape exactly.
    #[test]
    fn fire_grant_first_grant_unchanged_from_pre_ch15_shape() {
        let a = fire_args();
        let expected_uri = format!("project:{}", a.project);
        let grants = fire_grant_on_lead_assignment(a.clone());
        let g0 = &grants[0];
        assert_eq!(g0.resource.uri, expected_uri);
        assert_eq!(g0.fundamentals, vec![Fundamental::Tag]);
        assert_eq!(
            g0.action,
            vec![
                crate::permissions::action::Action::Read,
                crate::permissions::action::Action::Inspect,
                crate::permissions::action::Action::List,
            ]
        );
        match g0.holder {
            PrincipalRef::Agent(id) => assert_eq!(id, a.lead),
            ref other => panic!("expected Agent holder, got {other:?}"),
        }
    }

    /// CH-15: the second grant's URI is a selector-grammar predicate
    /// targeting sessions tagged `project:<id>` AND `#kind:session`.
    /// The project-tag predicate uses the string-literal form so
    /// UUIDs starting with a digit parse cleanly.
    #[test]
    fn fire_grant_second_grant_targets_session_object() {
        let a = fire_args();
        let expected_uri = format!(
            r#"tags contains "project:{}" AND tags contains #kind:session"#,
            a.project
        );
        let grants = fire_grant_on_lead_assignment(a);
        assert_eq!(grants[1].resource.uri, expected_uri);
    }

    /// CH-15: the second grant's selector targets sessions filtered
    /// by `project:<id>` tag. The URI is a selector-grammar predicate
    /// the engine parses into `Selector::Bool(...)` at Step 3.
    #[test]
    fn fire_grant_second_grant_selector_filters_by_project_tag() {
        let a = fire_args();
        let project_tag = format!("project:{}", a.project);
        let grants = fire_grant_on_lead_assignment(a);
        assert!(
            grants[1].resource.uri.contains(&project_tag),
            "session-object grant URI must carry project:<id>; got {:?}",
            grants[1].resource.uri
        );
        assert!(
            grants[1].resource.uri.contains("tags contains"),
            "session-object grant URI must use selector-grammar form; got {:?}",
            grants[1].resource.uri
        );
        assert!(
            grants[1].resource.uri.contains("#kind:session"),
            "session-object grant URI must constrain on #kind:session; got {:?}",
            grants[1].resource.uri
        );
        // Composite SessionObject expands to [DataObject, Tag] per
        // concept doc 01.
        assert_eq!(
            grants[1].fundamentals,
            vec![Fundamental::DataObject, Fundamental::Tag]
        );
    }

    /// CH-15: both grants share the same adoption AR provenance.
    /// `walk_provenance_chain` traverses both transparently.
    #[test]
    fn fire_grant_both_grants_descend_from_same_ar() {
        let a = fire_args();
        let expected_ar = a.adoption_auth_request_id;
        let grants = fire_grant_on_lead_assignment(a);
        assert_eq!(grants[0].descends_from, Some(expected_ar));
        assert_eq!(grants[1].descends_from, Some(expected_ar));
    }

    /// CH-15: the two grants always have distinct ids — they are
    /// independent records on the wire even though they share a
    /// provenance.
    #[test]
    fn fire_grant_both_grants_have_distinct_ids() {
        let grants = fire_grant_on_lead_assignment(fire_args());
        assert_ne!(
            grants[0].id, grants[1].id,
            "paired grants must carry distinct GrantIds"
        );
    }
}
