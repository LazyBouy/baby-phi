//! `spawn_claimed()` — end-to-end harness fixture that stands up the
//! acceptance server AND drives the bootstrap-claim flow, then hands
//! the caller a [`ClaimedAdmin`] bundle with a pre-cookied reqwest
//! client ready to call M2+ endpoints.
//!
//! Every M2/P4+ page acceptance test starts here: secret ops, model
//! provider registration, MCP server patches, platform-defaults
//! writes. Centralising the claim dance avoids copy/pasting 30 lines
//! of set-up into each test file.

// Cargo compiles each file under tests/ as a separate test binary and
// flags code not reached from that specific binary as dead. The
// functions here are exercised by acceptance_bootstrap.rs's sibling
// tests (starting at M2/P4); suppress the lint during P3 while the
// first real consumer is still being built.
#![allow(dead_code)]

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::time::Duration;

use super::{claim_body, mint_credential, spawn, Acceptance};

/// Everything a page-test needs after a successful bootstrap claim:
/// the acceptance server, the admin's `agent_id`, the signed-cookie
/// value, and a reqwest client that automatically sends the cookie
/// on every request.
pub struct ClaimedAdmin {
    pub acc: Acceptance,
    /// The human admin's agent_id (UUID string).
    pub agent_id: String,
    /// The raw `baby_phi_session` cookie value (the JWT). Useful for
    /// tests that want to sign a forged cookie variant.
    pub session_cookie: String,
    /// reqwest client preconfigured with `Cookie: baby_phi_session=<jwt>`
    /// on every request.
    pub authed_client: reqwest::Client,
}

impl ClaimedAdmin {
    /// Convenience — absolute URL for a path on the acceptance server.
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.acc.base_url, path)
    }
}

/// Boot a fresh acceptance server, mint a bootstrap credential, POST
/// `/api/v0/bootstrap/claim` with sensible defaults, and capture the
/// resulting session cookie into a preconfigured reqwest client.
///
/// `with_metrics` controls whether the Prometheus layer + `/metrics`
/// route are installed — only one caller per process may pass `true`
/// (see the `OnceLock` note in `acceptance_common::spawn`).
pub async fn spawn_claimed(with_metrics: bool) -> ClaimedAdmin {
    let acc = spawn(with_metrics).await;
    let credential = mint_credential(&acc).await;

    // Use the non-cookied client to run the claim — the response
    // carries the `Set-Cookie` header we capture below.
    let bootstrap_client = acc.client();
    let res = bootstrap_client
        .post(format!("{}/api/v0/bootstrap/claim", acc.base_url))
        .json(&claim_body(
            &credential,
            "Acceptance Admin",
            "web",
            "https://example.com/admin",
        ))
        .send()
        .await
        .expect("post claim");
    assert_eq!(
        res.status().as_u16(),
        201,
        "claim must return 201; body was {:?}",
        res.text().await
    );

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("claim response must set session cookie")
        .to_str()
        .expect("set-cookie value is ASCII")
        .to_string();
    let session_cookie = set_cookie
        .split("baby_phi_session=")
        .nth(1)
        .expect("baby_phi_session= prefix present")
        .split(';')
        .next()
        .expect("cookie value present")
        .to_string();

    let body: serde_json::Value = res.json().await.expect("decode claim body");
    let agent_id = body["human_agent_id"]
        .as_str()
        .expect("human_agent_id in claim response")
        .to_string();

    // Pre-cookied client — every request automatically carries the
    // session. reqwest doesn't expose a "set a default cookie" knob,
    // so we install a default `Cookie` header.
    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("baby_phi_session={session_cookie}"))
            .expect("cookie value is a valid header"),
    );
    let authed_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(default_headers)
        .build()
        .expect("build authed reqwest client");

    ClaimedAdmin {
        acc,
        agent_id,
        session_cookie,
        authed_client,
    }
}
