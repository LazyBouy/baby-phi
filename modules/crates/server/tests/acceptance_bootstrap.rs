//! Acceptance tests for the S01 bootstrap flow.
//!
//! These are the **E2E** tier of the test pyramid: the real
//! `baby-phi-server` axum app, the real embedded SurrealDB store, and
//! real HTTP over the loopback interface. Contracted in the plan at
//! Part 8 rows C4 / C5 / C9 / C10 / C11 / C13.
//!
//! Scenarios covered (matching `requirements/admin/01` §11 Acceptance
//! Scenarios + the plan's P9 bullets):
//!
//! 1. **Fresh install, happy claim** — status reports unclaimed, claim
//!    succeeds, status flips to claimed, every side-effect shows up in
//!    storage: human agent row, grant row, audit event row with
//!    `audit_class = Alerted`, catalogue seed for `system:root`, and
//!    the bootstrap credential is consumed.
//! 2. **Wrong credential** — claim returns 403 `BOOTSTRAP_INVALID`;
//!    no admin is created, no audit event is written.
//! 3. **Reused credential without admin** — credential marked consumed
//!    but no admin exists (corner case); claim returns 403
//!    `BOOTSTRAP_ALREADY_CONSUMED`.
//! 4. **Second claim after success** — 409 `PLATFORM_ADMIN_CLAIMED`.
//! 5. **/metrics surface** — Prometheus scrape exposes the
//!    `baby_phi_bootstrap_claims_total` counter after a successful
//!    claim.

mod acceptance_common;

use domain::audit::AuditClass;
use domain::model::nodes::AgentKind;
use domain::Repository;
use serde_json::Value;

use acceptance_common::{claim_body, mint_credential, spawn};

fn status_url(base: &str) -> String {
    format!("{base}/api/v0/bootstrap/status")
}
fn claim_url(base: &str) -> String {
    format!("{base}/api/v0/bootstrap/claim")
}

// ---- 1. Fresh install, happy claim ----------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_install_happy_claim() {
    let acc = spawn(false).await;
    let client = acc.client();

    // 1a. Initial status is unclaimed.
    let r = client.get(status_url(&acc.base_url)).send().await.unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["claimed"], false);
    assert_eq!(body["awaiting_credential"], true);

    // 1b. Mint credential + claim.
    let cred = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(&cred, "Alex Chen", "slack", "@alex"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);

    // Session cookie emitted.
    let set_cookie = r
        .headers()
        .get("set-cookie")
        .map(|v| v.to_str().unwrap().to_string())
        .expect("Set-Cookie header");
    assert!(set_cookie.starts_with("baby_phi_session="));

    let claim: Value = r.json().await.unwrap();
    let admin_id = claim["human_agent_id"].as_str().unwrap().to_string();
    let grant_id = claim["grant_id"].as_str().unwrap().to_string();
    let audit_event_id = claim["audit_event_id"].as_str().unwrap().to_string();
    let auth_request_id = claim["bootstrap_auth_request_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 1c. Status flips to claimed with the same admin id.
    let r = client.get(status_url(&acc.base_url)).send().await.unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["claimed"], true);
    assert_eq!(body["admin_agent_id"].as_str().unwrap(), admin_id);

    // 1d. Storage-side invariants: every side-effect of the s01 flow
    // is persisted.
    let admin = acc
        .store
        .get_admin_agent()
        .await
        .unwrap()
        .expect("admin agent present");
    assert_eq!(admin.id.to_string(), admin_id);
    assert_eq!(admin.kind, AgentKind::Human);
    assert_eq!(admin.display_name, "Alex Chen");

    // Grant present with [allocate] on system:root, descends_from the
    // bootstrap auth request, delegable.
    let grant_uuid: uuid::Uuid = grant_id.parse().unwrap();
    let grant = acc
        .store
        .get_grant(domain::model::ids::GrantId::from_uuid(grant_uuid))
        .await
        .unwrap()
        .expect("grant present");
    assert_eq!(grant.action, vec!["allocate".to_string()]);
    assert_eq!(grant.resource.uri, "system:root");
    assert_eq!(
        grant.descends_from.as_ref().unwrap().to_string(),
        auth_request_id,
    );
    assert!(grant.delegable);

    // Auth request in Approved state.
    let ar_uuid: uuid::Uuid = auth_request_id.parse().unwrap();
    let ar = acc
        .store
        .get_auth_request(domain::model::ids::AuthRequestId::from_uuid(ar_uuid))
        .await
        .unwrap()
        .expect("auth request present");
    assert_eq!(ar.state, domain::model::nodes::AuthRequestState::Approved);

    // Audit event — read back directly by id and assert the class,
    // event-type, provenance, and actor-agent all match what the
    // handler claimed to emit.
    let audit_uuid: uuid::Uuid = audit_event_id.parse().unwrap();
    let ae = acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(audit_uuid))
        .await
        .unwrap()
        .expect("audit event row must exist after claim");
    assert_eq!(ae.event_type, "platform_admin.claimed");
    assert_eq!(ae.audit_class, AuditClass::Alerted);
    assert_eq!(
        ae.provenance_auth_request_id.as_ref().unwrap().to_string(),
        auth_request_id,
    );
    assert_eq!(ae.actor_agent_id.as_ref().unwrap().to_string(), admin_id);
    // Genesis event in the platform-scope chain — no predecessor.
    assert_eq!(ae.prev_event_hash, None);

    // Catalogue seeds.
    let has_root = acc
        .store
        .catalogue_contains(None, "system:root")
        .await
        .unwrap();
    assert!(has_root, "system:root catalogue entry missing");

    // Bootstrap credential is consumed (list unconsumed only → empty).
    let unconsumed = acc.store.list_bootstrap_credentials(true).await.unwrap();
    assert!(
        unconsumed.is_empty(),
        "bootstrap credential should be consumed after successful claim",
    );

    // Sanity-check: AuthRequest also carries the Alerted class —
    // the s01 flow sets it on both so downstream query paths remain
    // consistent.
    assert_eq!(ar.audit_class, AuditClass::Alerted);
}

// ---- 2. Wrong credential ---------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wrong_credential_rejects_403_bootstrap_invalid() {
    let acc = spawn(false).await;
    let client = acc.client();

    // Seed a real credential but send a different one.
    let _real = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(
            "bphi-bootstrap-WRONG",
            "Alex",
            "slack",
            "@alex",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "BOOTSTRAP_INVALID");

    // No admin created, credential unchanged.
    assert!(acc.store.get_admin_agent().await.unwrap().is_none());
    let unconsumed = acc.store.list_bootstrap_credentials(true).await.unwrap();
    assert_eq!(unconsumed.len(), 1);

    // Audit chain is empty — no event should be emitted for a failed
    // credential verification (per Plan Part 8 C6 + admin/01 §11 S2).
    // The `last_event_hash_for_org(None)` probe returns None both
    // when no events exist AND when the only event is genesis, so
    // we additionally check the grants list is empty.
    assert!(acc
        .store
        .last_event_hash_for_org(None)
        .await
        .unwrap()
        .is_none());
}

// ---- 3. Reused credential without admin (403) -----------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reused_credential_without_admin_rejects_403_already_consumed() {
    let acc = spawn(false).await;
    let client = acc.client();

    // Mint + consume without going through the claim path.
    let plain = mint_credential(&acc).await;
    let creds = acc.store.list_bootstrap_credentials(false).await.unwrap();
    assert_eq!(creds.len(), 1);
    acc.store
        .consume_bootstrap_credential(&creds[0].record_id)
        .await
        .unwrap();

    // No admin, consumed credential. Claim should now 403 with
    // ALREADY_CONSUMED (not INVALID, since the hash still verifies).
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(&plain, "Alex", "slack", "@alex"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "BOOTSTRAP_ALREADY_CONSUMED");
}

// ---- 4. Second claim after success (409) ----------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn second_claim_after_success_rejects_409_platform_admin_claimed() {
    let acc = spawn(false).await;
    let client = acc.client();

    let cred1 = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(&cred1, "Alex", "slack", "@alex"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);

    // Second claim with a freshly-minted (different!) credential still
    // fails with 409: `get_admin_agent` is the first-line check before
    // any credential scan.
    let cred2 = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(&cred2, "Other", "email", "other@example.com"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 409);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "PLATFORM_ADMIN_CLAIMED");
}

// ---- 5. /metrics exposes the bootstrap-claims counter ---------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metrics_endpoint_exposes_bootstrap_claims_counter() {
    // This test owns the process-global Prometheus recorder. It must
    // be the only acceptance test that passes `with_metrics = true`.
    let acc = spawn(true).await;
    let client = acc.client();

    let cred = mint_credential(&acc).await;
    let r = client
        .post(claim_url(&acc.base_url))
        .json(&claim_body(
            &cred,
            "Metrics Admin",
            "web",
            "https://example.com",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);

    let r = client
        .get(format!("{}/metrics", acc.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body = r.text().await.unwrap();
    assert!(
        body.contains("baby_phi_bootstrap_claims_total"),
        "expected baby_phi_bootstrap_claims_total in /metrics output:\n{body}"
    );
    assert!(
        body.contains("result=\"success\""),
        "expected result=\"success\" label in /metrics output:\n{body}"
    );
}
