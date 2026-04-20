//! Shared harness for acceptance tests.
//!
//! Each test gets a fresh `SurrealStore` in its own tempdir (real
//! RocksDB, not the in-memory fake) and a real `baby-phi-server` axum
//! app bound to a loopback port. That's the "E2E" contract — we
//! exercise the exact same code paths the production binary runs,
//! against the exact same storage backend.
//!
//! Why this file isn't compiled as its own `tests/mod.rs`: Cargo treats
//! every `.rs` under `tests/` as a separate test binary, but
//! `tests/<subdir>/mod.rs` is treated as an auxiliary module shared by
//! sibling `acceptance_*.rs` files via `mod acceptance_common;`. This
//! pattern is documented in the Cargo book under "Integration tests".

use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::Router;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::PrometheusMetricLayer;
use server::bootstrap::{generate_bootstrap_credential, GeneratedCredential};
use server::{build_router, AppState, SessionKey};
use store::SurrealStore;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub const TEST_SESSION_SECRET: &str = "acceptance-secret-acceptance-secret-acceptance";

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    l.local_addr().unwrap().port()
}

/// A running acceptance server. Drop the harness to shut everything down
/// (the tempdir is deleted when `_tmp` goes out of scope).
pub struct Acceptance {
    pub base_url: String,
    pub store: Arc<SurrealStore>,
    pub _tmp: TempDir,
    pub _join: JoinHandle<()>,
}

impl Acceptance {
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            // The happy-path claim response sets a cookie; don't let
            // reqwest auto-follow it into any redirects.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("build reqwest client")
    }
}

/// Boot a fresh server against an embedded SurrealDB in a tempdir.
/// `with_metrics = true` wraps the router in the Prometheus layer + the
/// `/metrics` route. Only the first caller in a given process may pass
/// `true` — subsequent `true` calls panic on the global-recorder
/// install. This is enforced with a `OnceLock` so tests cooperate.
pub async fn spawn(with_metrics: bool) -> Acceptance {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Unique namespace per test so parallel runs never collide inside
    // the shared SurrealDB process state.
    let namespace = format!("acc-{}", Uuid::new_v4());
    let store = SurrealStore::open_embedded(tmp.path(), &namespace, "v0")
        .await
        .expect("open embedded SurrealStore");

    let store = Arc::new(store);
    let state = AppState {
        repo: store.clone(),
        session: SessionKey::for_tests(TEST_SESSION_SECRET),
    };

    let app: Router = if with_metrics {
        install_prometheus_layer(build_router(state))
    } else {
        build_router(state)
    };

    let port = free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let join = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let base_url = format!("http://127.0.0.1:{port}");
    wait_until_serving(&base_url).await;

    Acceptance {
        base_url,
        store,
        _tmp: tmp,
        _join: join,
    }
}

/// Deterministic readiness probe: poll `/healthz/live` until the server
/// returns 200 OK. The axum task is spawned on the current runtime so
/// the bind-to-accept transition needs a reactor tick — this poll
/// guarantees we observe that transition before any test traffic,
/// without the fixed-sleep guesswork.
async fn wait_until_serving(base_url: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .build()
        .expect("build reqwest client for readiness probe");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let probe = format!("{base_url}/healthz/live");
    loop {
        if let Ok(res) = client.get(&probe).send().await {
            if res.status().is_success() {
                return;
            }
        }
        if std::time::Instant::now() >= deadline {
            panic!("acceptance harness: server did not start accepting on {probe} within 5 s",);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Install the Prometheus layer + `/metrics` route exactly once per
/// process. `axum-prometheus` installs a global recorder — calling
/// `pair()` twice panics on the `set_global_recorder` path.
fn install_prometheus_layer(router: Router) -> Router {
    static HANDLE: OnceLock<(PrometheusMetricLayer<'static>, PrometheusHandle)> = OnceLock::new();
    let (layer, handle) = HANDLE
        .get_or_init(|| {
            let (l, h) = PrometheusMetricLayer::pair();
            (l, h)
        })
        .clone();
    router
        .route(
            "/metrics",
            axum::routing::get(move || async move { handle.render() }),
        )
        .layer(layer)
}

/// Mint a fresh bootstrap credential against the harness store. Returns
/// the plaintext that the claim flow expects.
pub async fn mint_credential(acc: &Acceptance) -> String {
    let GeneratedCredential { plaintext, .. } = generate_bootstrap_credential(acc.store.as_ref())
        .await
        .expect("mint credential");
    plaintext
}

pub fn claim_body(
    credential: &str,
    display: &str,
    channel_kind: &str,
    channel_handle: &str,
) -> serde_json::Value {
    serde_json::json!({
        "bootstrap_credential": credential,
        "display_name": display,
        "channel": { "kind": channel_kind, "handle": channel_handle }
    })
}
