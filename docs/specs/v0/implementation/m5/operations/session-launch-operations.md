<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-02 amendment (2026-04-24): synthetic feeder replaced with real `agent_loop()` + MockProvider; "Terminate-mid-turn not reflecting" playbook updated; new "MockProvider deterministic output" caveat. See §"CH-02 amendment" below. -->

# Operations — Page 14 first session launch

**Status**: `[EXISTS]` as of M5/P4.

Scope:

- Launch / preview / terminate / show / list / tools handlers.
- `AppState::session_registry` DashMap.
- `BabyPhiSessionRecorder` persist hook.
- `[session] max_concurrent` config ceiling (default 16).
- `tokio_util::CancellationToken` per live session.

## Error-code reference

Every 4xx / 5xx response carries a stable `code` string. The
full mapping lives in
[`platform::sessions::wire_code_for`](../../../../../../modules/crates/server/src/platform/sessions/mod.rs).

| HTTP | Code | Meaning | Fix |
|---|---|---|---|
| 400 | `SESSION_INPUT_INVALID` | Request body failed shape validation | Re-submit with valid shape |
| 400 | `TERMINATE_REASON_REQUIRED` | `reason` field empty | Supply non-empty reason |
| 403 | `FORBIDDEN` | Viewer not session's starter + not org-member | Use a CEO / member session |
| 403 | `AGENT_NOT_MEMBER_OF_PROJECT` | Agent's `owning_org` mismatch | Create agent in the project's org |
| 403 | `PERMISSION_CHECK_FAILED` | Step 0 Catalogue miss (only gating step at M5 — see D4.1) | Seed catalogue entry for `session` under the owning org |
| 403 | `TERMINATE_FORBIDDEN` | Caller not the session starter + not an org Human | Terminate from an authorised session |
| 404 | `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` / `SESSION_NOT_FOUND` / `MODEL_RUNTIME_NOT_FOUND` | Id unknown | Verify ids |
| 409 | `PARALLELIZE_CAP_REACHED` | Agent is running `profile.parallelize` sessions | Wait for an active session to end OR tune cap |
| 409 | `MODEL_RUNTIME_UNRESOLVED` | Agent profile has no `model_config_id` | Bind via `PATCH /agents/:id/profile` (C-M5-5) |
| 409 | `MODEL_RUNTIME_ARCHIVED` | Bound runtime row is archived | Re-bind to an active runtime |
| 409 | `AGENT_PROFILE_MISSING` | Agent has no profile row yet | Create profile via update-agent path |
| 409 | `SESSION_ALREADY_TERMINAL` | Session already Completed / Aborted / FailedLaunch | No action — terminal state is idempotent |
| 503 | `SESSION_WORKER_SATURATED` | Per-worker registry full | Tune `config.session.max_concurrent` |
| 500 | `RECORDER_FAILURE` / `COMPOUND_TX_FAILURE` / `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` / `SESSION_REPLAY_PANIC` | Internal failure | Check server logs + audit chain for replay |

## Incident playbooks

- **Stuck "running" session** — the replay task's `finalise_and_persist`
  call failed. Check server logs for `sessions::launch recorder
  finalise failed`. Manual remediation: call
  `POST /sessions/:id/terminate` with a reason.
- **Worker saturation** (503 `SESSION_WORKER_SATURATED`) — the
  per-worker DashMap is full. Raise `[session] max_concurrent` in
  `config/default.toml` or the profile-specific override. The
  entries drain as sessions finalise.
- **Terminate-mid-turn not reflecting** — the M5 synthetic
  replay feeder completes in a few ms so terminate often races
  finalise. Either outcome is correct; test
  `terminate_twice_returns_already_terminal_on_second_call` pins
  this invariant. M7+ swap to `phi_core::agent_loop` will surface
  real mid-turn cancellation when the phi-core loop honours the
  token.
- **ModelRuntime unresolved at launch** — agent has no
  `model_config_id` OR the id doesn't resolve to an active
  runtime. Fix via `PATCH /agents/:id/profile` with a
  `model_config_id` field naming an active `model_runtime` row.
  Note: C-M5-5 blocks the change with 409
  `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` if the agent has live
  sessions — terminate those first.
- **Permission Check denial** — at M5, Step 0 (Catalogue) is the
  only blocking step; steps 1–6 are advisory. If a launch
  unexpectedly returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_0`,
  seed the resources_catalogue with an entry for `session` scoped
  to the owning org. The Permission Check preview endpoint
  returns the full 0–6 trace.

## Metrics (M7 observability extensions)

At M5/P4 the launch chain does not emit dedicated metrics — phi's
`axum-prometheus` middleware counts HTTP call rates by route +
status. M7b adds:

- `phi_sessions_live{org_id}` gauge (from `session_registry.len()`).
- `phi_sessions_launch_total{outcome}` counter.
- `phi_sessions_replay_duration_seconds` histogram.
- `phi_sessions_terminate_total{reason_class}` counter.

## CH-02 amendment — real `agent_loop()` + MockProvider (2026-04-24)

The "M5 synthetic replay feeder completes in a few ms so terminate often races finalise" caveat under "Terminate-mid-turn not reflecting" is **stale** post-CH-02. The actual flow is now:

- `phi_core::agent_loop()` runs inside the spawned task.
- `MockProvider` returns a deterministic response (default `"Acknowledged."`, or the agent profile's `mock_response` override) but the loop still cycles through the real `AgentStart → TurnStart → MessageUpdate → TurnEnd → AgentEnd` event sequence with phi-core's runtime tick.
- Cancellation tokens are honoured by phi-core's loop — terminate-mid-turn now produces a real `AgentEnd { rejection: Some("cancelled") }` event, which the recorder maps to `governance_state = Aborted`.

### Updated playbook — Terminate-mid-turn not reflecting

If a terminate races to completion you can confirm via `GET /api/v0/sessions/:id` — `governance_state` will be `Aborted` (mid-turn cancel) or `Completed` (finalise won the race). Both are correct. Pre-CH-02 the synthetic feeder always won the race; post-CH-02 the outcome depends on where in the loop the cancel arrives.

### New failure modes from MockProvider path

| Symptom | Cause | Fix |
|---|---|---|
| All sessions return identical text `"Acknowledged."` | Default `MockProvider::text()` response — operator hasn't set `mock_response` on the agent profile | Pin a custom response via `PATCH /api/v0/agents/:id/profile` with `mock_response: "<your text>"` |
| Acceptance test asserting specific message text fails post-upgrade | Pre-CH-02 tests asserted the synthetic 4-event sequence's canned strings | Update the assertion to the MockProvider output (default `"Acknowledged."` or the profile's `mock_response`) |
| Session row's `tokens_spent` is 0 | MockProvider doesn't accumulate token usage — at M5 token accounting is a no-op | Real provider integration (M7+) re-enables token-spend assertions |
| `phi_core::agent_loop` panics propagate to `SESSION_REPLAY_PANIC` 500 | Same path as M5/P4; CH-02 didn't widen this surface | Check server logs; loop panics indicate a bug in phi-core itself or in the recorder's event handling |

### Real LLM providers

Real providers (Anthropic / OpenAI / etc.) are deferred to M7. M5/CH-02 ships only the `MockProvider` standin. ADR-0032 documents the deferral path.

## Cross-references

- [Session launch architecture](../architecture/session-launch.md).
- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).
- [ADR-0032](../../m5_2/decisions/0032-mock-provider-at-m5.md) — MockProvider at M5.
- [M5 plan §P4](../../../../plan/build/m5-templates-system-agents-sessions-01710c13.md).
- [CH-02 plan](../../../../plan/build/ch-02-real-agent-loop-wiring-16fd9a3a.md).
