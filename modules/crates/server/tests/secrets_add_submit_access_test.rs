//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `add_secret`.
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor), preserving the existing vault Add happy-path.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn secret_add_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    let url = admin.url("/api/v0/platform/secrets");

    let material = b"sk-test-12345";
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "slug": "ch18-submit-access",
            "material_b64": BASE64_NOPAD.encode(material),
            "sensitive": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["slug"], "ch18-submit-access");
}
