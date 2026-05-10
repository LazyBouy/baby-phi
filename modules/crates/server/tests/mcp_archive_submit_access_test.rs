//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `archive_mcp_server`.
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor), preserving the existing MCP-archive happy-path.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use serde_json::{json, Value};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_archive_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    let url = admin.url("/api/v0/platform/mcp-servers");

    // Register first (its own Submit check fires + passes).
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "display_name": "ch18-mcp-archive",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/x",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let body: Value = r.json().await.unwrap();
    let mcp_server_id = body["mcp_server_id"].as_str().unwrap().to_string();

    // Archive — the archive handler fires its OWN Submit check via
    // the Template-E AR it builds.
    let r = admin
        .authed_client
        .post(format!("{url}/{mcp_server_id}/archive"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    assert!(body["audit_event_id"].as_str().is_some());
}
