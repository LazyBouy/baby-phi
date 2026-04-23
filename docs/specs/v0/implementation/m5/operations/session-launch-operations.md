<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Page 14 first session launch

**Status**: [PLANNED M5/P4] — stub seeded at M5/P0; filled at P4
with incident playbooks + dashboard queries + metric names.

Scope at M5/P4:

- Launch / preview / terminate / list / tools handlers.
- `AppState::session_registry` DashMap.
- `BabyPhiSessionRecorder` persist hook.
- `[session] max_concurrent` config ceiling.
- `tokio_util::CancellationToken` per session.

## Incident playbooks (land at P4)

- **Stuck "running" session** — task crashed, scopeguard should
  have persisted `FailedLaunch`; verify via `session.governance_state`.
- **Worker saturation** — `session_registry.len() >= max_concurrent`
  surfaces as 503 `SESSION_WORKER_SATURATED`; tune via config.
- **Terminate-mid-turn not reflecting** — phi-core honours
  `CancellationToken` within ≤1 turn; if >1 turn elapses, check
  `cancel_token` wiring.
- **ModelRuntime unresolved at launch** — agent has no
  `model_config_id` OR the id doesn't resolve to an active
  runtime; page 09 profile editor (C-M5-5 path at M5/P4) is the
  fix location.
- **Permission Check failure at step N** — preview endpoint
  returns the 0–6 trace; operators replay with the trace.

## Cross-references

- [Session launch architecture](../architecture/session-launch.md).
- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).
