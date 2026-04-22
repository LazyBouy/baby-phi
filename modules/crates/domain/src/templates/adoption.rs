//! Shared adoption-AR builder for Templates A/B/C/D.
//!
//! Template A, B, C, D all share the same **adoption-time** shape: the
//! CEO (who holds `[allocate]` on the org's control-plane objects at
//! creation time) self-approves "this org adopts Template X." The
//! per-template modules ([`super::a`], [`super::b`], [`super::c`],
//! [`super::d`]) are thin wrappers over [`build_adoption_request`] that
//! preset the [`TemplateKind`] discriminator and carry template-specific
//! documentation.
//!
//! ## Why one shared helper?
//!
//! Adoption is kind-agnostic in shape (same approver, same scope, same
//! audit class, same active window). The semantic difference between
//! templates shows up at **trigger-fire** time (M5 work) — not at
//! adoption. Duplicating four near-identical files would amplify drift
//! risk; a single shared helper with four per-template wrappers
//! preserves the discoverable per-template module layout the plan
//! commits to without the copy-paste cost.
//!
//! ## Relationship to Template E
//!
//! Mechanically, adoption-ARs are Template-E-shaped (requestor ==
//! approver, pre-approved, `terminal_state_entered_at = Some(now)`).
//! This helper delegates to [`super::e::build_auto_approved_request`]
//! and overlays adoption-specific kinds + scope + justification.
//!
//! The returned AR carries `provenance_template = None`. Callers that
//! also persist the `Template` graph node (M3/P4's compound tx does
//! this) should set `ar.provenance_template = Some(template.id)`
//! before persisting — matching Template E's "set provenance after
//! persist" convention.

use chrono::{DateTime, Utc};

use crate::audit::AuditClass;
use crate::model::ids::OrgId;
use crate::model::nodes::{AuthRequest, PrincipalRef, ResourceRef, TemplateKind};

/// Inputs every per-template adoption builder consumes. Trigger-fire
/// context (project lead, delegation loop id, role name) is **M5**
/// work and lives in a separate per-template fire-AR builder; it does
/// not belong here.
#[derive(Debug, Clone)]
pub struct AdoptionArgs {
    /// The org adopting the template.
    pub org_id: OrgId,
    /// The CEO principal (or, equivalently, any principal holding
    /// `[allocate]` on the org's control-plane objects). Used as both
    /// requestor and sole approver.
    pub ceo: PrincipalRef,
    /// Wall-clock time for `submitted_at` + `terminal_state_entered_at`.
    pub now: DateTime<Utc>,
}

/// Build an adoption-shaped AR for an arbitrary `TemplateKind`.
///
/// Panics (via `debug_assert!`) in debug builds when called with
/// `TemplateKind::SystemBootstrap` or `TemplateKind::E` — those
/// templates use their own construction paths
/// ([`super::e::build_auto_approved_request`] for E; the bootstrap
/// claim flow for SystemBootstrap). Templates F is reserved for M6
/// break-glass and should not be adopted at org creation.
///
/// Release builds treat the misuse as a best-effort construction
/// (returns an adoption-shaped AR with the supplied kind), rather than
/// panicking the server. M3/P4's wizard payload validation rejects F
/// upstream so this fallback never fires in practice.
pub fn build_adoption_request(kind: TemplateKind, args: AdoptionArgs) -> AuthRequest {
    debug_assert!(
        matches!(
            kind,
            TemplateKind::A | TemplateKind::B | TemplateKind::C | TemplateKind::D
        ),
        "adoption::build_adoption_request should only be called for A/B/C/D; got {:?}",
        kind
    );

    let AdoptionArgs { org_id, ceo, now } = args;

    // Resource URI: `org:<id>/template:<kind>` — stable, org-scoped,
    // unique per (org, kind) pair. The M3/P4 handler seeds a
    // catalogue entry at this URI so Permission-Check Step 0
    // resolves when a template-fire descending grant is evaluated.
    let resource_uri = format!("org:{}/template:{}", org_id, kind.as_str());
    // `#template:<kind>` tag for grant-ceiling matching; the
    // control-plane kind tag keeps the adoption AR's scope aligned
    // with the org's control-plane object catalogue.
    let kinds = vec![
        format!("#template:{}", kind.as_str()),
        "#kind:control_plane".to_string(),
    ];
    // Scope the resulting grants can carry. A single org-scoped
    // scope string is sufficient for adoption; trigger-fire ARs
    // will carry richer scopes (e.g. `project:<id>`) in M5.
    let scope = vec![format!("org:{}", org_id)];
    let justification = Some(format!(
        "self-approved by CEO: org `{}` adopts authority template `{}`",
        org_id,
        kind.as_str()
    ));

    super::e::build_auto_approved_request(super::e::BuildArgs {
        requestor_and_approver: ceo,
        resource: ResourceRef { uri: resource_uri },
        kinds,
        scope,
        justification,
        audit_class: AuditClass::Alerted,
        now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::AgentId;
    use crate::model::nodes::{AuthRequestState, ResourceSlotState};

    fn sample_args() -> AdoptionArgs {
        AdoptionArgs {
            org_id: OrgId::new(),
            ceo: PrincipalRef::Agent(AgentId::new()),
            now: Utc::now(),
        }
    }

    #[test]
    fn adoption_ar_is_already_approved() {
        let ar = build_adoption_request(TemplateKind::A, sample_args());
        assert_eq!(ar.state, AuthRequestState::Approved);
        assert_eq!(ar.resource_slots[0].state, ResourceSlotState::Approved);
    }

    #[test]
    fn adoption_ar_carries_template_specific_kind_tag() {
        for kind in [
            TemplateKind::A,
            TemplateKind::B,
            TemplateKind::C,
            TemplateKind::D,
        ] {
            let ar = build_adoption_request(kind, sample_args());
            let expected_tag = format!("#template:{}", kind.as_str());
            assert!(
                ar.kinds.contains(&expected_tag),
                "adoption AR for {kind:?} must contain {expected_tag}; got {:?}",
                ar.kinds
            );
        }
    }

    #[test]
    fn adoption_ar_scopes_to_owning_org() {
        let args = sample_args();
        let expected = format!("org:{}", args.org_id);
        let ar = build_adoption_request(TemplateKind::A, args);
        assert!(ar.scope.contains(&expected));
    }

    #[test]
    fn adoption_ar_audit_class_is_alerted() {
        let ar = build_adoption_request(TemplateKind::B, sample_args());
        assert_eq!(ar.audit_class, AuditClass::Alerted);
    }

    #[test]
    fn each_call_mints_a_fresh_ar_id() {
        let a = build_adoption_request(TemplateKind::C, sample_args());
        let b = build_adoption_request(TemplateKind::C, sample_args());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn requestor_equals_approver_is_ceo() {
        let args = sample_args();
        let ceo = args.ceo.clone();
        let ar = build_adoption_request(TemplateKind::D, args);
        match (
            &ar.requestor,
            &ar.resource_slots[0].approvers[0].approver,
            &ceo,
        ) {
            (PrincipalRef::Agent(a), PrincipalRef::Agent(b), PrincipalRef::Agent(c)) => {
                assert_eq!(a, b);
                assert_eq!(a, c);
            }
            _ => panic!("requestor/approver/CEO must all be the same Agent"),
        }
    }

    #[test]
    fn provenance_template_starts_unset() {
        // Callers set `provenance_template = Some(template_node.id)`
        // AFTER persisting the Template graph node. This lets the
        // M3/P4 compound tx fill the link without requiring the
        // pure-fn builder to know about Template node ids.
        let ar = build_adoption_request(TemplateKind::A, sample_args());
        assert!(ar.provenance_template.is_none());
    }
}
