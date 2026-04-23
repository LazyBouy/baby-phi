<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 14 — First Session Launch architecture

**Status**: [PLANNED M5/P4] — stub seeded at M5/P0; full flow
diagram + cancellation sequences + launch-time compound-tx details
land at M5/P4 when the vertical ships.

Page 14 is M5's biggest single phase (5 carryover closes) and the
phi-core-heaviest surface. Covers the launch wizard (R1-R4, W1-W4,
N1-N4 per requirements/admin/14-first-session-launch.md).

## Scope

- `POST /api/v0/orgs/:org_id/projects/:project_id/sessions` — launch.
- `POST /api/v0/orgs/:org_id/projects/:project_id/sessions/preview` — Permission Check preview (steps 0–6).
- `GET  /api/v0/sessions/:id` — drill-down with wrapped phi-core types.
- `POST /api/v0/sessions/:id/terminate` — cancel + state transition.
- `GET  /api/v0/projects/:project_id/sessions` — list with stripped header shape.
- `GET  /api/v0/sessions/:id/tools` — resolved `AgentTool` set.

## Flow

Launch compound-tx (from the plan's §P4 §launch_session flow):

1. Validate agent + project + membership.
2. Resolve `ModelRuntime` via `agent.profile.model_config_id` (C-M5-5 gate).
3. Permission Check steps 0–6.
4. Per-agent parallelize gate (`count_active_sessions_for_agent < profile.blueprint.parallelize`).
5. Session-registry size gate (`< [session] max_concurrent`, default 16).
6. Compound tx: persist Session + first LoopRecord + `runs_session` edge + `uses_model` edge + `platform.session.started` audit + `DomainEvent::SessionStarted` post-commit.
7. `tokio::spawn` running `phi_core::agent_loop` with event channel feeding `BabyPhiSessionRecorder`.
8. Register `(session_id, cancel_token)` in `AppState::session_registry`.
9. Return `LaunchReceipt { session_id, first_loop_id, permission_check_trace }`.

## Carryovers closed at P4

- **C-M5-2** — `uses_model` edge written at launch (edge retyped FROM agent TO model_runtime in migration 0005).
- **C-M5-3** — Session / LoopRecord / Turn persistence via `BabyPhiSessionRecorder`.
- **C-M5-4** — `AgentTool` resolver + `GET /sessions/:id/tools`.
- **C-M5-5** — `ModelConfig` change + real 409 on active sessions.
- **C-M5-6** — Shape B materialise-after-both-approve (via sidecar read).

## Cross-references

- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).
- [Session persistence](session-persistence.md).
- [Shape B materialisation](shape-b-materialisation.md).
- [M5 plan §P4](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
