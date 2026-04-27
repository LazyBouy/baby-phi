//! Graceful shutdown — implements [ADR-0031 §D31.5] (designed at M5/P4,
//! shipped at CH-K8S-PREP P-3 / ADR-0033).
//!
//! On SIGTERM/SIGINT, axum's `with_graceful_shutdown` stops accepting new
//! HTTP requests. Once the HTTP server returns, [`graceful_shutdown`]
//! cancels every live agent-loop task via the [`SessionRegistry`] and
//! waits for the spawned tasks to drain (each `spawn_agent_task` removes
//! its own registry entry when `agent_loop` returns post-cancellation).
//!
//! Sessions that fail to drain within `timeout` are reported via the
//! [`DrainTimeout`] error so operators / readiness gates can act
//! accordingly. M7b extends this with a hard-clear that flips orphans to
//! `governance_state = FailedLaunch` (per ADR-0031 §D31.4 panic-safety
//! pattern) — out of scope for the prep refactor.
//!
//! [ADR-0031 §D31.5]: ../../../../../docs/specs/v0/implementation/m5/decisions/0031-session-cancellation-and-concurrency.md

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::state::SessionRegistry;

/// Default duration the shutdown handler waits for agent-loop tasks to
/// drain before reporting `DrainTimeout`. Matches ADR-0031 §D31.2's
/// 30-second baseline.
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, thiserror::Error)]
#[error("shutdown drain timed out with {remaining} live session(s) still running")]
pub struct DrainTimeout {
    /// Number of registry entries still present when the timeout
    /// expired. M7b's panic-safety hard-clear (FailedLaunch flip)
    /// uses this count to decide how many orphans to reconcile.
    pub remaining: usize,
}

/// Cancel every live agent-loop task and wait for the registry to
/// drain (or for `timeout` to expire).
///
/// Returns `Ok(())` when every spawn task has cleaned up its registry
/// entry, or `Err(DrainTimeout { remaining })` when the deadline is
/// hit. The caller (e.g., `main.rs`) typically logs the outcome and
/// exits the process; M7b adds a hard-clear of any remaining entries
/// before exit.
pub async fn graceful_shutdown(
    registry: Arc<dyn SessionRegistry>,
    timeout: Duration,
) -> Result<(), DrainTimeout> {
    registry.cancel_all();
    let deadline = Instant::now() + timeout;
    let poll_interval = Duration::from_millis(50);

    loop {
        if registry.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(DrainTimeout {
                remaining: registry.len(),
            });
        }
        tokio::time::sleep(poll_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_session_registry;
    use domain::model::ids::SessionId;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn empty_registry_drains_immediately() {
        let registry = new_session_registry();
        graceful_shutdown(registry, Duration::from_secs(1))
            .await
            .expect("empty registry returns Ok immediately");
    }

    #[tokio::test]
    async fn cancel_all_fires_every_token_and_drain_succeeds_when_tasks_remove_themselves() {
        let registry = new_session_registry();

        // Seed three tokens. A real spawn_agent_task watches the
        // token, then removes the entry on agent_loop return — we
        // simulate that with a small tokio task per token.
        let mut tokens = Vec::new();
        for _ in 0..3 {
            let session_id = SessionId::new();
            let token = CancellationToken::new();
            registry.insert(session_id, token.clone());
            let registry_clone = Arc::clone(&registry);
            tokens.push(token.clone());
            tokio::spawn(async move {
                token.cancelled().await;
                registry_clone.remove(&session_id);
            });
        }
        assert_eq!(registry.len(), 3);

        graceful_shutdown(Arc::clone(&registry), Duration::from_secs(2))
            .await
            .expect("registry drains when tasks remove themselves on cancel");

        assert_eq!(registry.len(), 0, "drain leaves registry empty");
        for token in &tokens {
            assert!(token.is_cancelled(), "every seeded token was cancelled");
        }
    }

    #[tokio::test]
    async fn drain_timeout_reports_remaining_count_for_stuck_tasks() {
        let registry = new_session_registry();

        // Seed two tokens whose simulated tasks never remove their
        // registry entries (i.e., a stuck spawn_agent_task that
        // panicked before reaching the cleanup line).
        for _ in 0..2 {
            let session_id = SessionId::new();
            let token = CancellationToken::new();
            registry.insert(session_id, token);
        }

        let result = graceful_shutdown(Arc::clone(&registry), Duration::from_millis(100)).await;

        match result {
            Err(DrainTimeout { remaining }) => assert_eq!(
                remaining, 2,
                "drain timeout reports the count of stuck sessions"
            ),
            Ok(()) => panic!("expected DrainTimeout, got Ok"),
        }
    }
}
