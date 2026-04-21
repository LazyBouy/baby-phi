//! Acceptance tests for the M2/P7 Platform Defaults HTTP surface.
//!
//! Real axum app + real embedded SurrealDB + real HTTP over the
//! loopback interface. Covers the GET + PUT routes plus the
//! optimistic-concurrency contract (stale `if_version` → 409 with the
//! current version surfaced to the client).
//!
//! Scenarios (plan §P7 + C11 verification):
//!
//! 1. Unauth — GET returns 401.
//! 2. Fresh install — GET returns `persisted: false` + factory
//!    baseline; `defaults.version == 0`.
//! 3. First PUT (if_version=0) — creates the row, returns
//!    `new_version = 1`, emits `platform.defaults.updated` (Alerted).
//! 4. PUT with stale if_version — 409 with `current_version` in the
//!    error message body.
//! 5. PUT rejects zero max_turns (validation).

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use domain::audit::AuditClass;
use domain::Repository;
use serde_json::{json, Value};
use uuid::Uuid;

fn url(admin: &ClaimedAdmin) -> String {
    admin.url("/api/v0/platform/defaults")
}

/// Fetch the factory baseline from the server and mutate the two
/// fields each test cares about — max_turns + retention_days —
/// leaving every other phi-core-nested field exactly as phi-core's
/// `Default::default()` produces it. This keeps the test robust
/// against phi-core's nested-struct evolution.
async fn factory_defaults_with(admin: &ClaimedAdmin, max_turns: u64, retention_days: u64) -> Value {
    let r = admin.authed_client.get(url(admin)).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200, "fetch factory failed");
    let body: Value = r.json().await.unwrap();
    let mut defaults = body["factory"].clone();
    defaults["execution_limits"]["max_turns"] = json!(max_turns);
    defaults["default_retention_days"] = json!(retention_days);
    defaults
}

// ---- 1. Unauth -------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_get_returns_401() {
    let admin = spawn_claimed(false).await;
    let r = admin.acc.client().get(url(&admin)).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 401);
}

// ---- 2. Fresh install GET returns factory baseline -------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_install_get_returns_factory_with_persisted_false() {
    let admin = spawn_claimed(false).await;
    let r = admin.authed_client.get(url(&admin)).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["persisted"], false);
    assert_eq!(body["defaults"]["version"], 0);
    assert_eq!(body["defaults"]["singleton"], 1);
    // Factory sub-tree reflects phi-core defaults.
    assert_eq!(body["factory"]["execution_limits"]["max_turns"], 50);
    assert_eq!(body["factory"]["retry_config"]["max_retries"], 3);
    // Live defaults == factory on a fresh install.
    assert_eq!(
        body["defaults"]["execution_limits"]["max_turns"],
        body["factory"]["execution_limits"]["max_turns"]
    );
}

// ---- 3. First PUT (if_version=0) creates the row + emits audit -------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_put_creates_row_and_emits_alerted_audit_event() {
    let admin = spawn_claimed(false).await;

    let defaults = factory_defaults_with(&admin, 40, 45).await;
    let body = json!({
        "if_version": 0,
        "defaults": defaults,
    });
    let r = admin
        .authed_client
        .put(url(&admin))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let resp: Value = r.json().await.unwrap();
    assert_eq!(resp["new_version"], 1);
    let audit_event_id = resp["audit_event_id"].as_str().unwrap().to_string();

    // Verify the audit event.
    let event_id = audit_event_id.parse::<Uuid>().unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("updated audit event persisted");
    assert_eq!(audit.event_type, "platform.defaults.updated");
    assert_eq!(audit.audit_class, AuditClass::Alerted);
    assert_eq!(audit.diff["before"], serde_json::Value::Null);
    assert_eq!(audit.diff["after"]["version"], 1);
    assert_eq!(audit.diff["after"]["default_retention_days"], 45);
    assert_eq!(audit.diff["after"]["execution_limits"]["max_turns"], 40);

    // Second GET reflects the write.
    let r = admin.authed_client.get(url(&admin)).send().await.unwrap();
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["persisted"], true);
    assert_eq!(body["defaults"]["version"], 1);
    assert_eq!(body["defaults"]["default_retention_days"], 45);
}

// ---- 4. PUT with stale if_version returns 409 ------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_if_version_returns_409_with_current_version_hint() {
    let admin = spawn_claimed(false).await;

    // Seed row at version 1.
    let defaults_v1 = factory_defaults_with(&admin, 30, 30).await;
    let r = admin
        .authed_client
        .put(url(&admin))
        .json(&json!({
            "if_version": 0,
            "defaults": defaults_v1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);

    // Now submit with if_version=0 again — should 409.
    let defaults_stale = factory_defaults_with(&admin, 99, 99).await;
    let r = admin
        .authed_client
        .put(url(&admin))
        .json(&json!({
            "if_version": 0,
            "defaults": defaults_stale,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 409);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["code"], "PLATFORM_DEFAULTS_STALE_WRITE");
    assert!(
        err["message"]
            .as_str()
            .unwrap()
            .contains("current server-side version is 1"),
        "message should surface the current version: {err:?}"
    );

    // Row is unchanged.
    let r = admin.authed_client.get(url(&admin)).send().await.unwrap();
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["defaults"]["version"], 1);
    assert_eq!(body["defaults"]["execution_limits"]["max_turns"], 30);
}

// ---- 5. Validation rejects zero max_turns ---------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn zero_max_turns_fails_validation() {
    let admin = spawn_claimed(false).await;
    let defaults = factory_defaults_with(&admin, 0, 30).await;
    let r = admin
        .authed_client
        .put(url(&admin))
        .json(&json!({
            "if_version": 0,
            "defaults": defaults,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 400);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["code"], "VALIDATION_FAILED");
}
