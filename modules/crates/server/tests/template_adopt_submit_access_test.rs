//! CH-18 / ADR-0056 §D56.5 + §D56.8 + F3.B.create-side.a — happy-path
//! regression-proof for `adopt_template_inline`.
//!
//! adopt.rs is **kernel-skip-equivalent** for the F3.B.create-side.a
//! submit-side defence-in-depth check (see §D56.8 follow-up note +
//! `D-CH18-FOLLOWUP-02`): the AR's `requestor` is intentionally the
//! org's CEO (built via `build_adoption_request` line 90), NOT
//! `input.actor` (the platform-admin). Running the synthetic-Draft
//! Submit check with `&PrincipalRef::Agent(input.actor)` would hit
//! `RequestorOnlyOperation` because admin != ceo, which does not
//! capture the F3.B.create-side.a defence-in-depth INTENT for an
//! admin-on-behalf-of-CEO flow.
//!
//! This test asserts the existing happy-path of admin-driven Template
//! adoption still works after CH-18's other 8 wiring sites land. It
//! exercises POST `/api/v0/orgs/:org/authority-templates/:kind/adopt`
//! and confirms the adoption AR is created Approved (Template-E
//! shape).

mod acceptance_common;

use acceptance_common::admin::{authed_client_for, spawn_claimed_with_org};
use serde_json::Value;

#[tokio::test]
async fn template_adopt_submit_remains_unwired_kernel_skip_happy_path() {
    let org = spawn_claimed_with_org(false).await;

    // Adopt Template C as the org's CEO (the adopt endpoint requires
    // a CEO-class caller; adopt.rs is kernel-skip-equivalent for the
    // F3.B.create-side.a Submit gate per ADR-0056 §D56.8).
    let ceo_client = authed_client_for(&org.admin, org.ceo_agent_id).expect("mint CEO session");
    let url = org.url(&format!(
        "/api/v0/orgs/{}/authority-templates/c/adopt",
        org.org_id
    ));
    let r = ceo_client.post(&url).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let body: Value = r.json().await.unwrap();
    assert!(body["adoption_auth_request_id"].as_str().is_some());
    // Template-E shape: AR is constructed Approved.
    assert_eq!(body["state"].as_str(), Some("approved"));
}
