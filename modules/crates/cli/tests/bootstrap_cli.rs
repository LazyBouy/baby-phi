//! End-to-end test for `baby-phi bootstrap {status,claim}`.
//!
//! Boots an axum server in-process against `InMemoryRepository`, spawns
//! the built `baby-phi` binary (path via `CARGO_BIN_EXE_baby-phi`), and
//! asserts exit code + stdout shape for every subcommand.

use std::net::{SocketAddr, TcpListener};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use domain::in_memory::InMemoryRepository;
use domain::Repository;
use server::bootstrap::hash_credential;
use server::{build_router, AppState, SessionKey};

const TEST_SECRET: &str = "test-secret-test-secret-test-secret-test-secret";

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.local_addr().unwrap().port()
}

/// Boot the axum router on a random local port and return the base URL.
/// Seeds the repo with the supplied credentials so tests can exercise
/// the claim flow.
async fn spawn_server(credentials: &[&str]) -> (String, tokio::task::JoinHandle<()>) {
    let repo = Arc::new(InMemoryRepository::new());
    for plain in credentials {
        let hash = hash_credential(plain).unwrap();
        repo.put_bootstrap_credential(hash).await.unwrap();
    }
    let app = build_router(AppState {
        repo,
        session: SessionKey::for_tests(TEST_SECRET),
    });
    let port = free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    // Small wait so the listener is actually accepting before we call
    // the CLI. `tokio::net::TcpListener::bind` returning Ok is
    // sufficient but the spawned `serve` starts accepting at its own
    // pace; a short yield is empirically enough.
    tokio::time::sleep(Duration::from_millis(50)).await;
    (format!("http://127.0.0.1:{port}"), handle)
}

fn cli_bin() -> String {
    env!("CARGO_BIN_EXE_baby-phi").to_string()
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(cli_bin())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn baby-phi");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (code, stdout, stderr)
}

// ---- status ----------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_reports_unclaimed_on_fresh_install() {
    let (base, _h) = spawn_server(&[]).await;
    let (code, stdout, stderr) = run_cli(&["--server-url", &base, "bootstrap", "status"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("NOT yet claimed"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("baby-phi bootstrap claim"),
        "expected guidance, got: {stdout}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_reports_claimed_after_claim() {
    let (base, _h) = spawn_server(&["bphi-bootstrap-cli-status-claimed"]).await;
    // First claim succeeds.
    let (code, _, stderr) = run_cli(&[
        "--server-url",
        &base,
        "bootstrap",
        "claim",
        "--credential",
        "bphi-bootstrap-cli-status-claimed",
        "--display-name",
        "Alex CLI",
        "--channel-kind",
        "slack",
        "--channel-handle",
        "@alex",
    ]);
    assert_eq!(code, 0, "claim failed: {stderr}");

    // Now status reports claimed with admin_agent_id.
    let (code, stdout, stderr) = run_cli(&["--server-url", &base, "bootstrap", "status"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("already claimed"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("admin_agent_id"),
        "missing admin_agent_id: {stdout}"
    );
}

// ---- claim — happy path ----------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_happy_path_prints_all_ids_and_exits_zero() {
    let plain = "bphi-bootstrap-cli-happy";
    let (base, _h) = spawn_server(&[plain]).await;
    let (code, stdout, stderr) = run_cli(&[
        "--server-url",
        &base,
        "bootstrap",
        "claim",
        "--credential",
        plain,
        "--display-name",
        "Alex Chen",
        "--channel-kind",
        "slack",
        "--channel-handle",
        "@alex",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    for key in [
        "human_agent_id",
        "inbox_id",
        "outbox_id",
        "grant_id",
        "bootstrap_auth_request_id",
        "audit_event_id",
    ] {
        assert!(stdout.contains(key), "missing {key} in stdout: {stdout}");
    }
    assert!(stdout.contains("Next step"), "missing next-step guidance");
}

// ---- claim — error paths ---------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_with_wrong_credential_exits_rejected_code() {
    let (base, _h) = spawn_server(&["bphi-bootstrap-real"]).await;
    let (code, _stdout, stderr) = run_cli(&[
        "--server-url",
        &base,
        "bootstrap",
        "claim",
        "--credential",
        "bphi-bootstrap-WRONG",
        "--display-name",
        "Alex",
        "--channel-kind",
        "slack",
        "--channel-handle",
        "@alex",
    ]);
    // EXIT_REJECTED = 2.
    assert_eq!(
        code, 2,
        "expected rejected exit, got {code}; stderr: {stderr}"
    );
    assert!(
        stderr.contains("BOOTSTRAP_INVALID"),
        "expected BOOTSTRAP_INVALID, got: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claim_after_admin_exists_exits_rejected_code() {
    let (base, _h) = spawn_server(&["bphi-bootstrap-1", "bphi-bootstrap-2"]).await;
    // First claim succeeds.
    let (code, _, stderr) = run_cli(&[
        "--server-url",
        &base,
        "bootstrap",
        "claim",
        "--credential",
        "bphi-bootstrap-1",
        "--display-name",
        "First",
        "--channel-kind",
        "slack",
        "--channel-handle",
        "@first",
    ]);
    assert_eq!(code, 0, "first claim failed: {stderr}");

    // Second claim with a different valid credential → 409 rejected.
    let (code, _, stderr) = run_cli(&[
        "--server-url",
        &base,
        "bootstrap",
        "claim",
        "--credential",
        "bphi-bootstrap-2",
        "--display-name",
        "Second",
        "--channel-kind",
        "email",
        "--channel-handle",
        "second@example.com",
    ]);
    assert_eq!(code, 2, "expected rejected exit, got {code}");
    assert!(
        stderr.contains("PLATFORM_ADMIN_CLAIMED"),
        "expected PLATFORM_ADMIN_CLAIMED, got: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn server_unreachable_exits_transport_code() {
    // Deliberately pick a port with nothing on it.
    let port = free_port();
    // `free_port` returns a port that was free; the listener was dropped
    // before this line runs, but some OSes hold the socket in TIME_WAIT —
    // offset by 1 to make the port confidently dead.
    let dead = port.wrapping_add(1).max(2);
    let (code, _stdout, stderr) = run_cli(&[
        "--server-url",
        &format!("http://127.0.0.1:{dead}"),
        "bootstrap",
        "status",
    ]);
    // EXIT_TRANSPORT = 1. On the off chance the port *is* bound to
    // something weird, accept any non-zero exit — the assertion that
    // matters is that we didn't exit 0.
    assert_ne!(
        code, 0,
        "expected failure exit, got stdout-only success; stderr: {stderr}"
    );
}

// ---- agent demo ------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_demo_without_config_exits_non_zero() {
    // Run from a temp dir so config.toml is definitely missing. The
    // command should fail but NOT panic or hang.
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cli_bin())
        .args(["agent", "demo"])
        .current_dir(tmp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn baby-phi");
    assert_ne!(out.status.code().unwrap_or(-1), 0);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("config.toml"),
        "expected config.toml in error output, got: {stderr}"
    );
}
