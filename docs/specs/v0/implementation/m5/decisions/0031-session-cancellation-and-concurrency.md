<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0031 — Session cancellation + concurrency bounds

**Status: Accepted** (flipped at M5/P4 close, 2026-04-23).

Ratification evidence:
- `AppState::session_registry` = `Arc<DashMap<SessionId, CancellationToken>>`
  shipped in [`server::state`](../../../../../../modules/crates/server/src/state.rs).
- `session_max_concurrent` (default 16) on `AppState` drives the
  D3 `SESSION_WORKER_SATURATED` 503 gate in
  [`sessions::launch::launch_session`](../../../../../../modules/crates/server/src/platform/sessions/launch.rs).
- `sessions::terminate` fires `token.cancel()` + removes the
  registry entry + emits `DomainEvent::SessionAborted`; acceptance
  test `terminate_twice_returns_already_terminal_on_second_call`
  pins the idempotent-second-call contract.
- `tokio-util` workspace dep added at
  `Cargo.toml:[workspace.dependencies]`; `dashmap` alongside.

## Context

M5 page 14 (First Session Launch) introduces three new operational
concerns that did not exist at M4 close:

1. **Concurrency ceiling** — a runaway agent loop (or an operator
   script spamming launches) could spawn unbounded `tokio::task`s.
   Without a cap, the server process may OOM or saturate the event
   bus.
2. **Cancellation** — page 14-W3 requires a terminate button. A
   running `phi_core::agent_loop(...)` call is cancellable via
   `tokio_util::sync::CancellationToken`, but baby-phi needs a way
   to look up the token by `SessionId` from the HTTP handler.
3. **Multi-worker shared state** — if baby-phi ever runs with >1
   axum worker process (M7b production hardening), the in-process
   registry must be replaceable with a shared store (Redis).

## Decision

### D31.1 — Single-process `DashMap<SessionId, CancellationToken>` at M5

`server/src/state.rs::AppState` gains:

```rust
pub session_registry: Arc<DashMap<SessionId, CancellationToken>>,
```

Populated at `session::launch` time (immediately before
`tokio::spawn`); cleared at session end (in the recorder's
`SessionEnded` branch) OR at terminate (by the terminate handler);
cleared on task panic via a `scopeguard`-style drop wrapper.

M5 ships single-process only. M7b revisits for Redis-backed shared
registry if multi-process deployment becomes load-bearing.

### D31.2 — `[session] max_concurrent = 16` default ceiling

`config/default.toml`:

```toml
[session]
max_concurrent = 16
```

Plumbed through `server::config::ServerConfig::session: SessionConfig`.
When `session_registry.len() >= max_concurrent`, `session::launch`
returns HTTP 503 `SESSION_WORKER_SATURATED` (**distinct from** W2's
per-agent 409 `PARALLELIZE_CAP_REACHED`).

Rationale for 16: a single-machine dev box with 8 cores + 16GB RAM
can comfortably run 16 concurrent agent loops against moderate
context windows without GC / memory pressure. Operators with larger
machines tune up via `config/<profile>.toml` override. The value is
conservative on purpose — escalating is easier than debugging OOMs.

**Confirm with user at P4 opening.** Plan decision D3 marks this as
assumed-default pending P4 confirmation.

### D31.3 — Terminate = `cancel_token.cancel()` + state transition

`session::terminate_session(session_id, reason, actor)`:

1. `let token = registry.get(&session_id).ok_or(SESSION_NOT_FOUND)?.clone();`
2. `token.cancel();` — phi-core's `agent_loop` honors this within
   ≤1 turn + returns `Err(AgentLoopError::Cancelled)`.
3. Recorder's `SessionEnded` branch (triggered by the cancelled
   `AgentEnd` event) persists `governance_state = Aborted`.
4. Emit `platform.session.terminated` audit + `DomainEvent::SessionAborted`.
5. Remove from registry. Return `TerminateReceipt { session_id,
   terminated_at }`.

Idempotency: calling terminate on an already-terminal session
returns `SESSION_ALREADY_TERMINAL` (409, not 404 — the session
existed).

### D31.4 — Panic safety

`tokio::spawn` wraps the `agent_loop` call in a `scopeguard` that
removes the `SessionId` from the registry + persists
`governance_state = FailedLaunch` (distinct from `Aborted`) + emits
`platform.session.failed_launch` audit + `DomainEvent::SessionAborted
{ reason: "task panicked" }` on drop.

This matches `phi-core/CLAUDE.md` §Hook ordering invariant — every
Before*/After* hook and lifecycle event pair completes even on
panic.

### D31.5 — Graceful shutdown: drain before exit

On server SIGTERM, the axum shutdown hook iterates
`session_registry` + calls `cancel()` on every token + waits (with
timeout 30s per D3 default) for tasks to drain. Any still-running
task at shutdown-timeout is aborted via `JoinHandle::abort()` and
logged + persisted as `FailedLaunch`.

M7b extends this to advertise the active session count via
`/healthz` so load balancers can route fresh traffic away while
drain completes.

## Consequences

**Positive**
- Bounded concurrency = predictable resource usage.
- Cancellation composes cleanly with phi-core's existing
  `CancellationToken` primitive (zero re-invention).
- Panic safety guarantees no stuck "running" sessions in SurrealDB
  after a task crash — the scopeguard always persists a terminal
  state.
- `SESSION_WORKER_SATURATED` (503) and `PARALLELIZE_CAP_REACHED`
  (409) are distinguishable error codes for operators — distinct
  root causes (platform saturation vs per-agent limit).

**Negative**
- Single-process registry is a hard scale ceiling. 16 × N workers
  is still O(N) — multi-process needs Redis. M7b deferred.
- DashMap lookups add ~100ns per launch + per terminate. Negligible
  at 16-concurrent scale; M7b's Redis path adds network RTT and
  changes the calculus.

**Neutral**
- `[session] max_concurrent` is a runtime config value, not a
  compile-time constant. Tuning does not require recompile.

## References

- [M5 plan archive §D3 + §G16 + §G17 + §P4](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [`phi-core::agent_loop` cancellation semantics](../../../../../../../phi-core/src/agent_loop/mod.rs).
- [`tokio_util::sync::CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html).
- [Session launch architecture](../architecture/session-launch.md) — parallelize gate + worker-saturation gate flow (seeded at P0, filled at P4).
- [ADR-0027](../../m4/decisions/0027-per-agent-execution-limits-override.md) — sibling resource-limits pattern.
