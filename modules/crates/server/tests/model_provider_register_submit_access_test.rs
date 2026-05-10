//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `register_provider`.
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor), preserving the existing model-provider-register happy-path.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use serde_json::{json, Value};

async fn seed_vault_secret(admin: &ClaimedAdmin, slug: &str, material: &[u8]) {
    let url = admin.url("/api/v0/platform/secrets");
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "slug": slug,
            "material_b64": BASE64_NOPAD.encode(material),
            "sensitive": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn model_provider_register_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "ch18-prov-key", b"sk-ant-test").await;

    let url = admin.url("/api/v0/platform/model-providers");
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "config": {
                "id": "claude-sonnet-4-20250514",
                "name": "Test Model",
                "api": "anthropic_messages",
                "provider": "anthropic",
                "base_url": "https://api.anthropic.com",
                "reasoning": false,
                "context_window": 200_000,
                "max_tokens": 8192,
            },
            "secret_ref": "ch18-prov-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let body: Value = r.json().await.unwrap();
    assert!(body["provider_id"].as_str().is_some());
}
