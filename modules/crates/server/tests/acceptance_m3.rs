//! M3 cross-page acceptance — the compose test.
//!
//! One end-to-end scenario per the M3 plan §P6 / commitment C13:
//!
//! 1. Boot a claimed admin environment (`spawn_claimed`).
//! 2. POST `/api/v0/orgs` with a wizard payload → receive CreatedOrg
//!    receipt + 2 audit events (organization_created +
//!    authority_template.adopted).
//! 3. Open a CEO-authed session and GET `/api/v0/orgs/:id/dashboard`
//!    → receive a populated `DashboardSummary`.
//! 4. Verify the per-org audit chain — both events share the same
//!    `org_scope = Some(org_id)`, are ordered by timestamp, and the
//!    second's `prev_event_hash` equals blake3(first). Proves pages
//!    06 and 07 share the audit-chain invariant that is M3's headline
//!    deliverable.
//! 5. Verify the dashboard's stable wire shape re-reads consistently
//!    on a second sequential GET (polling smoke).
//!
//! ## phi-core leverage (P6.0 pre-audit)
//!
//! Q1 **none** — zero new `use phi_core::…` in this file. Q2: the
//! wizard POST payload transits a `defaults_snapshot_override` field
//! wrapping 4 phi-core types (identical to P4's pattern; audited
//! there). The dashboard GET response stays phi-core-stripped (P5
//! pre-audit). Q3: no phi-core `Session` / `AgentEvent` / `Usage`
//! surfaces introduced.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use acceptance_common::TEST_SESSION_SECRET;
use domain::repository::Repository;
use serde_json::{json, Value};
use server::session::{sign_and_build_cookie, SessionKey};

fn wizard_body() -> Value {
    json!({
        "display_name": "Acme M3 Compose",
        "vision": "End-to-end M3 acceptance",
        "mission": "Prove page 06 + 07 compose",
        "consent_policy": "implicit",
        "audit_class_default": "logged",
        "authority_templates_enabled": ["a"],
        "default_model_provider": null,
        "ceo_display_name": "Alice",
        "ceo_channel_kind": "email",
        "ceo_channel_handle": "alice@acme.test",
        "token_budget": 2_500_000,
    })
}

fn client_for(agent_id: &str) -> reqwest::Client {
    let key = SessionKey::for_tests(TEST_SESSION_SECRET);
    let (jwt, _cookie) = sign_and_build_cookie(&key, agent_id).expect("sign");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("baby_phi_session={jwt}")).unwrap(),
    );
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .build()
        .unwrap()
}

#[tokio::test]
async fn wizard_to_dashboard_preserves_audit_chain_and_counts() {
    // ----- 1. Boot environment -----
    let admin = spawn_claimed(false).await;

    // ----- 2. POST /api/v0/orgs (page 06 happy path) -----
    let res = admin
        .authed_client
        .post(admin.url("/api/v0/orgs"))
        .json(&wizard_body())
        .send()
        .await
        .expect("POST /orgs");
    assert_eq!(res.status().as_u16(), 201, "wizard create should 201");
    let receipt: Value = res.json().await.unwrap();

    let org_id_str = receipt["org_id"].as_str().unwrap().to_string();
    let ceo_agent_id = receipt["ceo_agent_id"].as_str().unwrap().to_string();
    let audit_event_ids: Vec<String> = receipt["audit_event_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        audit_event_ids.len(),
        2,
        "expect OrganizationCreated + AuthorityTemplateAdopted"
    );

    // ----- 3. GET /api/v0/orgs/:id/dashboard (page 07) authed as CEO -----
    let ceo_client = client_for(&ceo_agent_id);
    let dash = ceo_client
        .get(admin.url(&format!("/api/v0/orgs/{org_id_str}/dashboard")))
        .send()
        .await
        .expect("GET dashboard");
    assert_eq!(dash.status().as_u16(), 200, "dashboard should 200 for CEO");
    let d1: Value = dash.json().await.unwrap();

    // Dashboard reports the freshly-created org's shape.
    assert_eq!(d1["org"]["id"].as_str(), Some(org_id_str.as_str()));
    assert_eq!(d1["agents_summary"]["total"].as_u64(), Some(3));
    assert_eq!(d1["agents_summary"]["human"].as_u64(), Some(1));
    assert_eq!(d1["agents_summary"]["llm"].as_u64(), Some(2));
    assert_eq!(d1["viewer"]["role"].as_str(), Some("admin"));
    assert_eq!(d1["token_budget"]["total"].as_u64(), Some(2_500_000));
    let adopted = d1["templates_adopted"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        adopted.iter().any(|v| v == "a"),
        "dashboard must surface adopted Template A"
    );

    // ----- 4. Verify the per-org audit chain -----
    let org_uuid = uuid::Uuid::parse_str(&org_id_str).unwrap();
    let org_id = domain::model::ids::OrgId::from_uuid(org_uuid);
    let store = admin.acc.store.as_ref();
    let events = store
        .list_recent_audit_events_for_org(org_id, 10)
        .await
        .expect("list_recent_audit_events_for_org");
    assert_eq!(
        events.len(),
        2,
        "expect exactly 2 events under org_scope; got {events:?}"
    );
    // Match events by event_type — both are emitted with the same
    // `input.now` timestamp (see `create_organization` in
    // `platform/orgs/create.rs`), so a positional sort on timestamp
    // alone is non-deterministic. The per-org hash chain is the
    // authoritative order: the genesis event has
    // `prev_event_hash = None`; every subsequent event points at the
    // previous one.
    let created_ev = events
        .iter()
        .find(|e| e.event_type.contains("organization.created"))
        .expect("OrganizationCreated event must exist");
    let adopted_ev = events
        .iter()
        .find(|e| e.event_type.contains("authority_template.adopted"))
        .expect("AuthorityTemplateAdopted event must exist");

    // Both carry the same org_scope (the per-org chain invariant).
    assert_eq!(
        created_ev.org_scope,
        Some(org_id),
        "OrganizationCreated must scope to the new org"
    );
    assert_eq!(
        adopted_ev.org_scope,
        Some(org_id),
        "AuthorityTemplateAdopted must scope to the new org"
    );
    // The genesis event in the org's chain has no predecessor.
    assert!(
        created_ev.prev_event_hash.is_none(),
        "OrganizationCreated is the first event in this org's chain; \
         prev_event_hash must be None; got {:?}",
        created_ev.prev_event_hash
    );
    // Chain-continuity: the adopted event's prev_event_hash must
    // equal blake3(created event's canonical bytes). This is the
    // M3 headline deliverable — audit events leave the root chain
    // and start a per-org chain at org creation.
    let created_hash = domain::audit::hash_event(created_ev);
    assert_eq!(
        adopted_ev.prev_event_hash,
        Some(created_hash),
        "AuthorityTemplateAdopted's prev_event_hash must equal hash of \
         OrganizationCreated — chain continuity on a per-org basis"
    );

    // ----- 5. Second dashboard GET — polling smoke -----
    let dash2 = ceo_client
        .get(admin.url(&format!("/api/v0/orgs/{org_id_str}/dashboard")))
        .send()
        .await
        .expect("GET dashboard (second)");
    assert_eq!(dash2.status().as_u16(), 200);
    let d2: Value = dash2.json().await.unwrap();
    // Identity on the stable counters (no writes between GETs).
    assert_eq!(d1["agents_summary"], d2["agents_summary"]);
    assert_eq!(d1["token_budget"], d2["token_budget"]);
    assert_eq!(d1["templates_adopted"], d2["templates_adopted"]);
}

#[tokio::test]
async fn wizard_to_dashboard_phi_core_wire_shape_contracts_stable() {
    // Compose test for the two phi-core-transit contracts:
    //   - Page 06 (show endpoint + receipt) carries the full
    //     `defaults_snapshot` with all 4 phi-core-wrapped fields.
    //   - Page 07 (dashboard endpoint) DOES NOT carry
    //     `defaults_snapshot` or any phi-core-wrapping key.
    let admin = spawn_claimed(false).await;
    let res = admin
        .authed_client
        .post(admin.url("/api/v0/orgs"))
        .json(&wizard_body())
        .send()
        .await
        .expect("POST /orgs");
    assert_eq!(res.status().as_u16(), 201);
    let receipt: Value = res.json().await.unwrap();
    let org_id_str = receipt["org_id"].as_str().unwrap().to_string();
    let ceo_agent_id = receipt["ceo_agent_id"].as_str().unwrap().to_string();

    // Show endpoint carries the snapshot.
    let show = admin
        .authed_client
        .get(admin.url(&format!("/api/v0/orgs/{org_id_str}")))
        .send()
        .await
        .expect("GET /orgs/:id");
    let sbody: Value = show.json().await.unwrap();
    assert!(
        sbody["organization"]["defaults_snapshot"].is_object(),
        "show endpoint must carry defaults_snapshot (Q2 transit)"
    );

    // Dashboard endpoint MUST NOT carry the snapshot.
    let ceo_client = client_for(&ceo_agent_id);
    let dash = ceo_client
        .get(admin.url(&format!("/api/v0/orgs/{org_id_str}/dashboard")))
        .send()
        .await
        .expect("GET /dashboard");
    let dbody: Value = dash.json().await.unwrap();
    assert_no_keys(
        &dbody,
        &[
            "defaults_snapshot",
            "execution_limits",
            "context_config",
            "retry_config",
            "default_agent_profile",
            "blueprint",
        ],
    );
}

fn assert_no_keys(v: &Value, forbidden: &[&str]) {
    match v {
        Value::Object(map) => {
            for (k, inner) in map {
                for f in forbidden {
                    assert_ne!(
                        k, f,
                        "cross-page wire payload contains `{f}` — the dashboard wire shape must \
                         stay phi-core-stripped per P5 pre-audit"
                    );
                }
                assert_no_keys(inner, forbidden);
            }
        }
        Value::Array(arr) => {
            for inner in arr {
                assert_no_keys(inner, forbidden);
            }
        }
        _ => {}
    }
}
