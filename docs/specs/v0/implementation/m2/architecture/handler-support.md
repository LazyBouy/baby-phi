<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — `handler_support` shim

**Status: [EXISTS]** — shipped in M2/P3; every M2 page handler builds on it.

The reusable axum shim every M2+ handler builds on:

- `AuthenticatedSession` extractor (cookie → `AgentId` → 401).
- `check_permission(state, ctx, manifest)` (engine call + exhaustive
  `Decision → HTTP` mapping + metric).
- `emit_audit(state.audit, event)` (trait dispatch + 500 on failure).
- Shared `ApiError { code, message }` envelope.

ADR: [0018-handler-support-module.md](../decisions/0018-handler-support-module.md).

See also:
- [`../../m1/architecture/permission-check-engine.md`](../../m1/architecture/permission-check-engine.md)
  — the engine `check_permission` wraps.
