//! Smoke test for native TLS termination via `axum-server`.
//!
//! Generates a self-signed cert at test time with `rcgen`, writes it to a
//! temp dir, boots `axum_server::bind_rustls` on a random port, and asserts
//! that HTTPS requests succeed while plaintext HTTP requests fail with a
//! protocol error.

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time::Duration;

use axum_server::tls_rustls::RustlsConfig;
use domain::in_memory::InMemoryRepository;
use rcgen::generate_simple_self_signed;
use server::{build_router, AppState, SessionKey};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn native_tls_listener_serves_https_and_rejects_plaintext() {
    // Install rustls' ring-backed crypto provider. rustls 0.23 requires an
    // explicit default provider when multiple backends could match; we don't
    // care which one the test picks, only that one is picked before TLS code
    // runs. `install_default` is idempotent across multiple tests.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // 1. Generate a self-signed cert for localhost.
    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).expect("rcgen");
    let tmp = tempfile::tempdir().expect("tempdir");
    let cert_path = tmp.path().join("cert.pem");
    let key_path = tmp.path().join("key.pem");
    std::fs::write(&cert_path, cert.cert.pem()).unwrap();
    std::fs::write(&key_path, cert.key_pair.serialize_pem()).unwrap();

    // 2. Boot the server on a random port with TLS.
    let port = free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let rustls = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .expect("load self-signed cert");

    let app = build_router(AppState {
        repo: Arc::new(InMemoryRepository::new()),
        session: SessionKey::for_tests("test-secret-test-secret-test-secret-test-secret"),
    });
    let handle = axum_server::Handle::new();
    let server_handle = handle.clone();
    tokio::spawn(async move {
        axum_server::bind_rustls(addr, rustls)
            .handle(server_handle)
            .serve(app.into_make_service())
            .await
            .expect("serve");
    });

    // 3. Wait until the listener is ready.
    let mut ready_addr = None;
    for _ in 0..50 {
        if let Some(a) = handle.listening().await {
            ready_addr = Some(a);
            break;
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
    ready_addr.expect("listener came up");

    // 4. HTTPS request succeeds (accepting the self-signed cert).
    let https = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let res = https
        .get(format!("https://127.0.0.1:{port}/healthz/live"))
        .send()
        .await
        .expect("https GET");
    assert_eq!(res.status(), 200);

    // 5. A plaintext HTTP request on the same port must fail — hitting a
    //    TLS listener with HTTP bytes produces a protocol error, never a 200.
    let http = reqwest::Client::new();
    let plaintext = http
        .get(format!("http://127.0.0.1:{port}/healthz/live"))
        .timeout(Duration::from_secs(2))
        .send()
        .await;
    assert!(
        plaintext.is_err(),
        "plaintext HTTP must not succeed against a TLS listener"
    );

    handle.graceful_shutdown(Some(Duration::from_millis(100)));
}
