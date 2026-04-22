//! Property tests for Template D adoption builder (M3/P2 commitment C6).

use chrono::{TimeZone, Utc};
use proptest::prelude::*;

use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{ApproverSlotState, AuthRequestState, PrincipalRef, ResourceSlotState};
use domain::templates::d::{build_adoption_request, AdoptionArgs};

fn arb_ceo() -> impl Strategy<Value = PrincipalRef> {
    prop_oneof![
        Just(PrincipalRef::Agent(AgentId::new())),
        Just(PrincipalRef::User(domain::model::ids::UserId::new())),
    ]
}

proptest! {
    #[test]
    fn template_d_adoption_is_always_approved(ceo in arb_ceo()) {
        let ar = build_adoption_request(AdoptionArgs {
            org_id: OrgId::new(),
            ceo,
            now: Utc.with_ymd_and_hms(2026, 4, 22, 12, 0, 0).unwrap(),
        });
        prop_assert_eq!(ar.state, AuthRequestState::Approved);
        prop_assert_eq!(ar.resource_slots[0].state, ResourceSlotState::Approved);
        prop_assert_eq!(
            ar.resource_slots[0].approvers[0].state,
            ApproverSlotState::Approved
        );
    }

    #[test]
    fn template_d_carries_template_d_kind_tag(ceo in arb_ceo()) {
        let ar = build_adoption_request(AdoptionArgs {
            org_id: OrgId::new(),
            ceo,
            now: Utc::now(),
        });
        prop_assert!(ar.kinds.contains(&"#template:d".to_string()));
    }

    #[test]
    fn template_d_resource_uri_encodes_org_id(org_id_seed in any::<u128>()) {
        let org_id = OrgId::from_uuid(uuid::Uuid::from_u128(org_id_seed));
        let ar = build_adoption_request(AdoptionArgs {
            org_id,
            ceo: PrincipalRef::Agent(AgentId::new()),
            now: Utc::now(),
        });
        let expected_uri = format!("org:{}/template:d", org_id);
        prop_assert_eq!(&ar.resource_slots[0].resource.uri, &expected_uri);
    }

    #[test]
    fn template_d_serde_round_trip_is_identity(ceo in arb_ceo()) {
        let ar = build_adoption_request(AdoptionArgs {
            org_id: OrgId::new(),
            ceo,
            now: Utc::now(),
        });
        let json = serde_json::to_string(&ar).unwrap();
        let back: domain::model::nodes::AuthRequest =
            serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.id, ar.id);
    }
}
