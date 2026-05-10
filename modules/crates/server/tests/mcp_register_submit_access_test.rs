//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `register_mcp_server`.
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor), preserving the existing MCP-register happy-path.

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
async fn mcp_register_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "ch18-mcp-key", b"secret-material").await;

    let url = admin.url("/api/v0/platform/mcp-servers");
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "display_name": "ch18-mcp",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/x",
            "secret_ref": "ch18-mcp-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let body: Value = r.json().await.unwrap();
    assert!(body["mcp_server_id"].as_str().is_some());
}
