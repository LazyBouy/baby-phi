//! Template B — **Direct Delegation Authority**.
//!
//! When Agent A spawns Agent B via a `DELEGATES_TO` edge at loop
//! `Ln`, the system auto-issues a grant giving Agent A `[read,
//! inspect]` on sessions tagged `delegated_from:Ln`. Fine-grained:
//! only the actual delegation chain matters. Composes with Template
//! A (a team lead who delegates still sees the delegatee's work).
//!
//! See
//! [`../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md`](../../../../docs/specs/v0/concepts/permissions/05-memory-sessions.md)
//! §Template B.
//!
//! ## M3 scope — adoption only
//!
//! [`build_adoption_request`] mints the CEO-self-approved adoption
//! AR at org creation. Per-`DELEGATES_TO`-edge trigger-fire ARs are
//! M5 work.

use crate::model::nodes::{AuthRequest, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-B adoption AR for the given org + CEO.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::B, args)
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
    fn template_b_adoption_carries_template_b_tag() {
        let ar = build_adoption_request(args());
        assert!(ar.kinds.iter().any(|k| k == "#template:b"));
        assert_eq!(ar.state, AuthRequestState::Approved);
    }

    #[test]
    fn template_b_adoption_references_org_scoped_resource_uri() {
        let a = args();
        let expected_uri = format!("org:{}/template:b", a.org_id);
        let ar = build_adoption_request(a);
        assert_eq!(ar.resource_slots[0].resource.uri, expected_uri);
    }
}
