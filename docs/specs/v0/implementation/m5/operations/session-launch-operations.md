<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — verified-header re-stamp per plan §3.C; body content unchanged from CH-17 close. CH-24 milestone-seal cycle — page 14 ops surface re-verified PASS against current HEAD; symptoms + remediations + CH-17 SSE-tail amendment + CH-02 real-loop amendment all still correct; cross-refs to ADR-0029/ADR-0031/ADR-0054/ADR-0055 intact. New cross-ref added at chunk-seal: m5-ops-runbook.md (aggregate page index) per F5.A lock. Cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-17 amendment: live SSE tail endpoint `GET /api/v0/sessions/:id/events` ships per ADR-0055 — operator-facing live transcript surface gated by `Action::Observe` on `session_object`; the wire returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>` on Decision::Denied (parallel to launch hard-deny) + emits the new `platform.session.live_stream_denied` (Alerted-class) audit event before returning Err; on success returns SSE with 30s keep-alive (`: keep-alive` text), broadcast buffer 64, lagged consumers receive a typed `event: lagged` SSE error then close. New 410 `SESSION_LIVE_STREAM_UNAVAILABLE` error for already-finalised sessions. New ops doc at `m5_2/operations/session-live-stream-operations.md` carries the full reconnect + lagged-receiver runbook.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-15 amendment: error-code reference table now reflects hard-deny at every step 0–6 per ADR-0054 §D54.6; Permission Check denial playbook updated to point at the m5_2 detailed runbook + Template A backfill migration; D4.1 advisory-mention removed.) -->
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
| 403 | `PERMISSION_CHECK_FAILED` | `PERMISSION_CHECK_FAILED_AT_STEP_<N>` (N = 0..6 per FailedStep variant) — every Decision::Denied at the engine returns 403 (CH-15 / [ADR-0054](../../m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md) closes drift D4.1). Wire message body embeds the step number + DeniedReason variant tag. | Seed Template A grants for the lead (paired `[Read, Inspect, List]` on `project:<id>` + `tags contains "project:<id>" AND #kind:session`) OR seed catalogue entry per [step-N detail in operations doc](../../m5_2/operations/session-launch-permission-gate-operations.md). |
| 403 | `TERMINATE_FORBIDDEN` | Caller not the session starter + not an org Human | Terminate from an authorised session |
| 404 | `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` / `SESSION_NOT_FOUND` / `MODEL_RUNTIME_NOT_FOUND` | Id unknown | Verify ids |
| 409 | `PARALLELIZE_CAP_REACHED` | Agent is running `profile.parallelize` sessions | Wait for an active session to end OR tune cap |
| 409 | `MODEL_RUNTIME_UNRESOLVED` | Agent profile has no `model_config_id` | Bind via `PATCH /agents/:id/profile` (C-M5-5) |
| 409 | `MODEL_RUNTIME_ARCHIVED` | Bound runtime row is archived | Re-bind to an active runtime |
| 409 | `AGENT_PROFILE_MISSING` | Agent has no profile row yet | Create profile via update-agent path |
| 409 | `SESSION_ALREADY_TERMINAL` | Session already Completed / Aborted / FailedLaunch | No action — terminal state is idempotent |
| 410 | `SESSION_LIVE_STREAM_UNAVAILABLE` | SSE handler called but the session has already finalised — broadcast Sender already removed from `SessionLiveStreamRegistry`. Emitted by `GET /api/v0/sessions/:id/events`. | Re-fetch the terminal `SessionDetail` via `GET /api/v0/sessions/:id` instead of tailing live. Race-prone only if terminate beat the SSE connect; correct response for a finished session. |
| 503 | `SESSION_WORKER_SATURATED` | Per-worker registry full | Tune `config.session.max_concurrent` |
| 500 | `RECORDER_FAILURE` / `COMPOUND_TX_FAILURE` / `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` / `SESSION_REPLAY_PANIC` | Internal failure | Check server logs + audit chain for replay |

### CH-17 amendment — live SSE tail endpoint

`GET /api/v0/sessions/:id/events` is a new SSE endpoint shipped at CH-17 (cycle hex `40c4d759`). It streams `phi_core::AgentEvent`s as `data:` JSON lines for the duration of the live session.

| HTTP | Code / Header | Meaning | Fix |
|---|---|---|---|
| 200 | `Content-Type: text/event-stream` | Live tail subscribed; events flow until session finalises (Sender removed from registry → BroadcastStream ends) or the consumer disconnects. 30s keep-alive ping (`: keep-alive`). | — |
| 403 | `PERMISSION_CHECK_FAILED_AT_STEP_<N>` | `Action::Observe` not granted on `session_object` for this actor; emits `platform.session.live_stream_denied` (Alerted) BEFORE returning. Most common reason on a CH-15-era DB: legacy Template A grants don't yet contain `"observe"` in their action arrays. | Run migration `0016_template_a_session_object_grant_add_observe.surql` (the migration runner applies it on next boot per ADR-0033 §D33.2 ledger pattern). For agents that need observability without Template A, mint an explicit grant via `POST /api/v0/grants` (M6+) or via an authority template that includes `observe`. |
| 404 | `SESSION_NOT_FOUND` | Session id unknown to the repository | Verify the session id is correct |
| 410 | `SESSION_LIVE_STREAM_UNAVAILABLE` | Session has finalised — broadcast registry has no Sender | Use `GET /api/v0/sessions/:id` for the terminal `SessionDetail` |
| SSE event `lagged` | `data: {"missed": <count>}` | Consumer fell behind the broadcast buffer (size 64). The stream closes immediately after this event. | Reconnect; consider a lower-latency consumer or a longer in-flight buffer in your client |

**Audit-event impact:** denials emit `platform.session.live_stream_denied` (parallel to `platform.session.launch_denied` at CH-15). Successes emit ZERO audit events (read-only stream). Builder at `domain/src/audit/events/m5_2/session_live_stream.rs::session_live_stream_denied`.

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
- **Permission Check denial** — CH-15 (drift D4.1 closure):
  every `Decision::Denied` at any step 0–6 returns 403
  `PERMISSION_CHECK_FAILED_AT_STEP_<N>`. The most common deny
  shape post-CH-15 is Step 2 `NoGrantsHeld` — the lead has no
  Template A grants on `session_object`. Fix by triggering
  `HasLeadEdgeCreated` (production wiring) OR running migration
  `0015_template_a_session_object_grant` to backfill legacy
  grants. The Permission Check preview endpoint returns the full
  0–6 trace; the deny path emits a
  `platform.session.launch_denied` audit event (Alerted) with
  `failed_step` + `reason_kind` for dashboard correlation.
  Detailed step-by-step playbook lives at
  [`m5_2/operations/session-launch-permission-gate-operations.md`](../../m5_2/operations/session-launch-permission-gate-operations.md).

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
