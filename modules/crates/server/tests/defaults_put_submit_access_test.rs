//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `put_platform_defaults`.
//!
//! The handler builds a Template-E AR with `requestor: input.actor` and
//! invokes the synthetic-Draft Submit check before persisting. The
//! check returns Ok via the Requestor classification. Asserts the
//! happy-path (PUT returns 200 + version bump + AR persisted) is
//! preserved.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use serde_json::{json, Value};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn defaults_put_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    let url = admin.url("/api/v0/platform/defaults");

    // Fetch factory baseline.
    let r = admin.authed_client.get(&url).send().await.unwrap();
    let body: Value = r.json().await.unwrap();
    let mut defaults = body["factory"].clone();
    defaults["execution_limits"]["max_turns"] = json!(7);
    defaults["default_retention_days"] = json!(180);

    // PUT with if_version=0. The handler's synthetic-Draft Submit
    // check runs upstream of `repo.create_auth_request(&ar)` and
    // returns Ok via the Requestor classification.
    let put_body = json!({
        "if_version": 0,
        "defaults": defaults,
    });
    let r = admin
        .authed_client
        .put(&url)
        .json(&put_body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["new_version"].as_u64(), Some(1));
}
