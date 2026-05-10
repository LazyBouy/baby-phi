//! CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
//! Submit-gate happy-path test for `patch_mcp_tenants`.
//!
//! Asserts the synthetic-Draft Submit check passes (admin == AR
//! requestor), preserving the existing MCP-patch happy-path. The
//! patch handler builds a Template-E AR for the mutation; the
//! synthetic-Draft Submit check runs upstream.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use serde_json::{json, Value};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_patch_tenants_passes_synthetic_draft_submit_check() {
    let admin = spawn_claimed(false).await;
    let url = admin.url("/api/v0/platform/mcp-servers");

    // Register first with `Only([uuid])`, then patch to `Only([different])`
    // — this exercises the create_auth_request path inside patch_tenants.
    let initial_org = uuid::Uuid::new_v4().to_string();
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "display_name": "ch18-patch-mcp",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/x",
            "tenants_allowed": { "mode": "only", "orgs": [initial_org] }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let mcp_server_id = r.json::<Value>().await.unwrap()["mcp_server_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Patch — narrow to a different tenant set (still non-empty Only)
    // → triggers AR creation + Submit check.
    let new_org = uuid::Uuid::new_v4().to_string();
    let r = admin
        .authed_client
        .patch(format!("{url}/{mcp_server_id}/tenants"))
        .json(&json!({ "tenants_allowed": { "mode": "only", "orgs": [new_org] } }))
        .send()
        .await
        .unwrap();
    // Per current code path the patch returns 200; the synthetic-Draft
    // Submit check passes via the Requestor classification.
    assert_eq!(r.status().as_u16(), 200);
}
