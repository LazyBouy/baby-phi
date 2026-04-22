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
//! ## M3 scope — adoption only
//!
//! [`build_adoption_request`] mints the **adoption AR** the CEO
//! self-approves at org creation time. The AR authorises the template
//! to fire within the org's scope; it does NOT itself issue grants.
//!
//! The trigger-fire path (one AR per `HAS_LEAD` edge firing, descending
//! from this adoption AR, issuing the actual session-read grant) lands
//! in **M5** alongside the event-subscription machinery that watches
//! the edge graph for `HasLead` emissions. The M3/P1 `Edge::HasLead`
//! variant was pre-wired so this module can reference it in docs
//! without an M5-blocking edge addition.

use crate::model::nodes::{AuthRequest, TemplateKind};

pub use super::adoption::AdoptionArgs;

/// Mint a Template-A adoption AR for the given org + CEO.
///
/// Pure fn — no I/O. See the shared helper
/// [`super::adoption::build_adoption_request`] for the underlying
/// construction details.
pub fn build_adoption_request(args: AdoptionArgs) -> AuthRequest {
    super::adoption::build_adoption_request(TemplateKind::A, args)
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
}
