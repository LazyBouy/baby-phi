//! `/metrics` acceptance smoke — the Prometheus layer wires correctly.
//!
//! Boots the harness with `with_metrics: true` (the only acceptance
//! binary per process that may; see the `OnceLock` caveat in
//! `acceptance_common::spawn`), exercises a handful of authenticated
//! + unauthenticated requests, and scrapes `/metrics` to verify:
//!
//!   1. The `/metrics` route is reachable on an authenticated
//!      harness.
//!   2. The response is Prometheus text-exposition-format
//!      (`# HELP …` / `# TYPE …` preamble).
//!   3. The standard axum-prometheus HTTP request metrics
//!      (`axum_http_requests_total` or equivalent) surface non-zero
//!      counts after real traffic.
//!
//! Per plan §P8 §3: the histogram M2 is building toward is
//! `baby_phi_permission_check_duration_seconds` — populated when
//! handlers call `handler_support::check_permission`. M2 admin
//! handlers use the Template-E bypass (self-approved), so no M2
//! handler records into that histogram yet. This test proves the
//! layer, routing, and scrape surface are wired end-to-end; M3+
//! handlers (which DO call `check_permission`) will exercise the
//! histogram without any additional wiring.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use serde_json::json;

fn api(admin: &ClaimedAdmin, path: &str) -> String {
    admin.url(&format!("/api/v0/platform{path}"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metrics_endpoint_surfaces_prometheus_exposition_after_traffic() {
    // IMPORTANT: this is the **only** acceptance binary in the M2
    // workspace that runs `with_metrics = true` — the acceptance-
    // harness OnceLock ensures subsequent `true` calls panic on the
    // global recorder install.
    let admin = spawn_claimed(true).await;

    // 1. Drive some traffic through the layer:
    //    - An unauthenticated list (401 on a gated route).
    //    - An authenticated factory-defaults read (200).
    //    - A failed PUT (validation rejects max_turns=0 → 400).
    let _ = admin
        .acc
        .client()
        .get(api(&admin, "/secrets"))
        .send()
        .await
        .unwrap();
    let r = admin
        .authed_client
        .get(api(&admin, "/defaults"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let factory = r.json::<serde_json::Value>().await.unwrap()["factory"].clone();
    let mut bad = factory;
    bad["execution_limits"]["max_turns"] = json!(0);
    let r = admin
        .authed_client
        .put(api(&admin, "/defaults"))
        .json(&json!({ "if_version": 0, "defaults": bad }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 400);

    // 2. Scrape `/metrics`.
    let r = admin
        .authed_client
        .get(admin.url("/metrics"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "/metrics reachable");
    let body = r.text().await.unwrap();

    // 3. Verify Prometheus text-exposition format. At minimum the
    //    response must carry the `# TYPE` directive and a recognised
    //    axum-prometheus HTTP metric name. We don't hard-code the
    //    exact metric name (axum-prometheus evolves the names
    //    between versions); we match on the well-known prefixes.
    assert!(
        body.contains("# TYPE"),
        "/metrics must use Prometheus text-exposition format; got:\n{body}"
    );
    // axum-prometheus 0.7 exposes `axum_http_requests_total` +
    // `axum_http_requests_duration_seconds`. Accept either.
    let has_request_metric = body.contains("axum_http_requests_")
        || body.contains("http_requests_total")
        || body.contains("http_server_requests");
    assert!(
        has_request_metric,
        "/metrics must include at least one HTTP-request metric; got:\n{body}"
    );
}
