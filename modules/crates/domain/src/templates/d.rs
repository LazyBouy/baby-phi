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
//! ## M3 scope — adoption only
//!
//! [`build_adoption_request`] mints the CEO-self-approved adoption
//! AR at org creation. Per-role-assignment trigger-fire ARs are M5
//! work.

use crate::model::nodes::{AuthRequest, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-D adoption AR for the given org + CEO.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::D, args)
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
}
