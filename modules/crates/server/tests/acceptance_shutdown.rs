//! Graceful-shutdown integration test (CH-K8S-PREP P-3 / ADR-0033 /
//! ADR-0031 §D31.5).
//!
//! Exercises [`server::shutdown::graceful_shutdown`] against the live
//! `Acceptance.session_registry` — the same `Arc<dyn SessionRegistry>`
//! the running phi-server holds in its `AppState`. Proves the
//! cancellation + drain path that K8s pod-termination relies on.
//!
//! Why we don't drive a real launch + real SIGTERM here:
//! - At M5 `MockProvider::text("Acknowledged.")` finishes a turn in
//!   sub-millisecond time; by the time the test could send a signal,
//!   the agent_loop has already completed and the registry entry has
//!   been removed naturally. There is no observable "mid-flight"
//!   window from a same-process test.
//! - SIGTERM to the test process kills the harness (and the runner).
//!
//! The acceptance suite for the M7b production-hardening milestone
//! adds a subprocess-spawning fixture that exercises real SIGTERM
//! end-to-end. CH-K8S-PREP only ships the trait + drain function; the
//! full SIGTERM-with-real-launch test is M7b scope.

mod acceptance_common;

use std::sync::Arc;
use std::time::Duration;

use acceptance_common::spawn;
use domain::model::ids::SessionId;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn graceful_shutdown_against_live_app_state_drains_simulated_sessions() {
    let acc = spawn(false).await;
    let registry = acc.session_registry.clone();

    // Inject 2 synthetic "stuck-task" entries simulating in-flight
    // agent-loop tasks. Each spawns a watcher that mirrors the real
    // `spawn_agent_task`: when the cancel-token fires, the watcher
    // removes its registry entry — same cleanup contract.
    let mut tokens = Vec::new();
    for _ in 0..2 {
        let session_id = SessionId::new();
        let token = CancellationToken::new();
        registry.insert(session_id, token.clone());
        tokens.push(token.clone());
        let registry_clone = Arc::clone(&registry);
        tokio::spawn(async move {
            token.cancelled().await;
            registry_clone.remove(&session_id);
        });
    }
    assert_eq!(registry.len(), 2, "two simulated sessions registered");

    server::shutdown::graceful_shutdown(Arc::clone(&registry), Duration::from_secs(2))
        .await
        .expect("live registry drains within timeout");

    assert_eq!(
        registry.len(),
        0,
        "graceful_shutdown drains every entry before returning Ok"
    );
    for (i, token) in tokens.iter().enumerate() {
        assert!(
            token.is_cancelled(),
            "token #{i} was cancelled by graceful_shutdown"
        );
    }
}
