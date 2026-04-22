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
//! ## M3 scope — adoption only
//!
//! [`build_adoption_request`] mints the CEO-self-approved adoption
//! AR at org creation. Per-appointment trigger-fire ARs (one per
//! agent assignment to an org-tree node) are M5 work.

use crate::model::nodes::{AuthRequest, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-C adoption AR for the given org + CEO.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::C, args)
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
}
