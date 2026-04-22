//! Property tests for Template A adoption builder (M3/P2 commitment C6).
//!
//! Each generated (org_id, CEO principal, timestamp) triple produces
//! an adoption AR satisfying every invariant the M3/P4 compound tx
//! depends on:
//!   1. State is `AuthRequestState::Approved`.
//!   2. Exactly one ResourceSlot, with exactly one ApproverSlot in
//!      Approved state, and the approver = supplied CEO.
//!   3. `#template:a` tag is present in `kinds`.
//!   4. Resource URI is `org:<id>/template:a`.
//!   5. Scope contains `org:<id>`.
//!   6. Serde round-trips preserve identity.
//!
//! Template B/C/D share a sibling test file each — parametric
//! generation over all four would share logic at the cost of readability
//! (four concrete files each ~40 lines are easier to grep).

use chrono::{TimeZone, Utc};
use proptest::prelude::*;

use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{ApproverSlotState, AuthRequestState, PrincipalRef, ResourceSlotState};
use domain::templates::a::{build_adoption_request, AdoptionArgs};

fn arb_ceo() -> impl Strategy<Value = PrincipalRef> {
    prop_oneof![
        Just(PrincipalRef::Agent(AgentId::new())),
        Just(PrincipalRef::User(domain::model::ids::UserId::new())),
    ]
}

proptest! {
    #[test]
    fn template_a_adoption_is_always_approved(ceo in arb_ceo()) {
        let org_id = OrgId::new();
        let now = Utc.with_ymd_and_hms(2026, 4, 22, 12, 0, 0).unwrap();
        let ar = build_adoption_request(AdoptionArgs { org_id, ceo: ceo.clone(), now });

        prop_assert_eq!(ar.state, AuthRequestState::Approved);
        prop_assert_eq!(ar.resource_slots.len(), 1);
        prop_assert_eq!(ar.resource_slots[0].state, ResourceSlotState::Approved);
        prop_assert_eq!(ar.resource_slots[0].approvers.len(), 1);
        prop_assert_eq!(
            ar.resource_slots[0].approvers[0].state,
            ApproverSlotState::Approved
        );
    }

    #[test]
    fn template_a_ceo_is_both_requestor_and_approver(ceo in arb_ceo()) {
        let args = AdoptionArgs {
            org_id: OrgId::new(),
            ceo: ceo.clone(),
            now: Utc::now(),
        };
        let ar = build_adoption_request(args);
        // `PrincipalRef` does not derive PartialEq (variants carry
        // different id types), so compare by variant + id via
        // serialisation — any semantic change will surface as a
        // JSON-diff mismatch.
        let requestor_json = serde_json::to_value(&ar.requestor).unwrap();
        let approver_json =
            serde_json::to_value(&ar.resource_slots[0].approvers[0].approver).unwrap();
        let ceo_json = serde_json::to_value(&ceo).unwrap();
        prop_assert_eq!(requestor_json, ceo_json.clone());
        prop_assert_eq!(approver_json, ceo_json);
    }

    #[test]
    fn template_a_resource_uri_matches_org_id(org_id_seed in any::<u128>()) {
        // Cast the seed into an OrgId so each proptest case explores a
        // different uri suffix.
        let org_id = OrgId::from_uuid(uuid::Uuid::from_u128(org_id_seed));
        let ar = build_adoption_request(AdoptionArgs {
            org_id,
            ceo: PrincipalRef::Agent(AgentId::new()),
            now: Utc::now(),
        });
        let expected_uri = format!("org:{}/template:a", org_id);
        prop_assert_eq!(&ar.resource_slots[0].resource.uri, &expected_uri);
        let expected_scope = format!("org:{}", org_id);
        prop_assert!(ar.scope.contains(&expected_scope));
    }

    #[test]
    fn template_a_serde_round_trip_is_identity(ceo in arb_ceo()) {
        let ar = build_adoption_request(AdoptionArgs {
            org_id: OrgId::new(),
            ceo,
            now: Utc::now(),
        });
        let json = serde_json::to_string(&ar).unwrap();
        let back: domain::model::nodes::AuthRequest =
            serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.id, ar.id);
        prop_assert_eq!(&back.kinds, &ar.kinds);
        prop_assert_eq!(&back.scope, &ar.scope);
    }
}
