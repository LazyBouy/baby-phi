//! Authority-chain axioms — the typed root of the permissions tree.
//!
//! Concept doc 02 §"System Bootstrap Template" lines 449–487 specifies that
//! every Grant + Auth Request in the system traces back through the
//! authority tree to a hardcoded bootstrap node approved by the
//! `system:genesis` axiomatic principal. Concept doc 04 §"The Authority
//! Chain" lines 510–547 names the traversal algorithm. CH-14 / ADR-0053
//! codifies these claims in typed Rust.
//!
//! Three pieces ship here:
//! 1. [`SYSTEM_GENESIS_PRINCIPAL`] — the magic-string const, single source
//!    of truth for the literal `"system:genesis"`.
//! 2. [`system_genesis_principal`] — canonical constructor for the
//!    [`PrincipalRef::System`] axiom.
//! 3. [`is_bootstrap_ar`] — the two-witness termination predicate the
//!    [`crate::Repository::walk_provenance_chain`] walker (D53.3) consults
//!    at the chain root.
//!
//! Per ADR-0053 §D53.1 / §D53.2.

use crate::model::ids::TemplateId;
use crate::model::nodes::{AuthRequest, PrincipalRef};

/// The magic-string identifier of the axiomatic root principal.
///
/// Concept doc 02 line 458 names this principal as the only approver of
/// the System Bootstrap Template's adoption AR. Wire format is
/// `{"system": "system:genesis"}` — see [`PrincipalRef`] docstring at
/// `model/nodes.rs:789-796` for the serde shape.
pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis";

/// Canonical constructor for the axiomatic [`PrincipalRef::System`]
/// principal. Production callsites at `bootstrap/claim.rs` (requestor +
/// approver) and test fixtures across `repository_test.rs` +
/// `template_e_props.rs` use this helper instead of constructing the
/// `PrincipalRef::System("system:genesis".into())` literal manually,
/// so the magic string lives in exactly one place per ADR-0053 §D53.1.
pub fn system_genesis_principal() -> PrincipalRef {
    PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL.into())
}

/// The hardcoded `TemplateId` used as the bootstrap AR's
/// `provenance_template`. By convention the all-zero UUID stands for
/// "the bootstrap template" — see `bootstrap/claim.rs:218` for the
/// production minting site.
///
/// Not a `const` because [`TemplateId::from_uuid`] is a regular `fn`
/// (not `const fn`); calling this helper at every termination check is
/// cheap (a single `Uuid::nil()` plus a transparent newtype wrap).
pub fn system_bootstrap_template_id() -> TemplateId {
    TemplateId::from_uuid(uuid::Uuid::nil())
}

/// The walker / cascade termination predicate.
///
/// An [`AuthRequest`] is the bootstrap node iff BOTH conditions hold:
///
/// 1. `requestor == system_genesis_principal()` (magic-string match on
///    the `system:genesis` principal).
/// 2. `provenance_template == Some(system_bootstrap_template_id())`
///    (the all-zero-UUID Template marker).
///
/// Two witnesses, AND-combined: a future code path that mints a
/// non-bootstrap AR with `system:genesis` requestor (e.g., for an
/// auditor-bot system-internal op) cannot alias the bootstrap; a future
/// Template AR that happens to use the all-zero UUID cannot alias the
/// bootstrap either. ADR-0053 §D53.2 / F5.A.
pub fn is_bootstrap_ar(ar: &AuthRequest) -> bool {
    let requestor_is_genesis = matches!(
        &ar.requestor,
        PrincipalRef::System(s) if s == SYSTEM_GENESIS_PRINCIPAL,
    );
    let provenance_is_bootstrap = ar.provenance_template == Some(system_bootstrap_template_id());
    requestor_is_genesis && provenance_is_bootstrap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::ids::AuthRequestId;
    use crate::model::nodes::AuthRequestState;
    use chrono::Utc;

    fn bootstrap_ar() -> AuthRequest {
        AuthRequest {
            id: AuthRequestId::new(),
            requestor: system_genesis_principal(),
            kinds: vec!["control_plane_object".into()],
            scope: vec!["allocate".into()],
            state: AuthRequestState::Approved,
            valid_until: None,
            submitted_at: Utc::now(),
            resource_slots: vec![],
            justification: None,
            audit_class: AuditClass::Alerted,
            terminal_state_entered_at: None,
            archived: false,
            active_window_days: 90,
            provenance_template: Some(system_bootstrap_template_id()),
            tags: vec![],
            descends_from_grant: None,
        }
    }

    #[test]
    fn const_round_trips_through_helper() {
        let p = system_genesis_principal();
        match p {
            PrincipalRef::System(s) => assert_eq!(s, SYSTEM_GENESIS_PRINCIPAL),
            _ => panic!("expected System variant"),
        }
    }

    #[test]
    fn helper_equals_manual_literal_construction() {
        let helper = system_genesis_principal();
        let manual = PrincipalRef::System("system:genesis".into());
        let helper_str = match &helper {
            PrincipalRef::System(s) => s.clone(),
            _ => panic!("expected System variant"),
        };
        let manual_str = match &manual {
            PrincipalRef::System(s) => s.clone(),
            _ => panic!("expected System variant"),
        };
        assert_eq!(helper_str, manual_str);
    }

    #[test]
    fn is_bootstrap_ar_positive_on_claim_shaped_ar() {
        let ar = bootstrap_ar();
        assert!(is_bootstrap_ar(&ar));
    }

    #[test]
    fn is_bootstrap_ar_negative_on_template_ar_with_non_zero_uuid() {
        let mut ar = bootstrap_ar();
        ar.provenance_template = Some(TemplateId::new());
        assert!(!is_bootstrap_ar(&ar));
    }

    #[test]
    fn is_bootstrap_ar_negative_on_non_genesis_system_ar() {
        let mut ar = bootstrap_ar();
        ar.requestor = PrincipalRef::System("system:auditor-bot".into());
        assert!(!is_bootstrap_ar(&ar));
    }
}
