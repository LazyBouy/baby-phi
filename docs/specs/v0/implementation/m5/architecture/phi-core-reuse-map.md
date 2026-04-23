<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — M5 phi-core reuse map

**Status**: [PLANNED M5/P9] — stub seeded at M5/P0 with predicted
import counts from the plan's Part 1.5; per-phase close audits
populate the actuals + positive close-audit greps. Appends to the
M4 reuse map at
[`../../m4/architecture/phi-core-reuse-map.md`](../../m4/architecture/phi-core-reuse-map.md).

M5 is the phi-core-heaviest milestone yet. Adds **3 new node-tier
wraps** (`Session`, `LoopRecordNode`, `TurnNode` all wrapping
`phi_core::session::model::*`) + **direct imports of**
`agent_loop`, `SessionRecorder`, `AgentEvent`, `AgentTool`,
`ModelConfig` across the `sessions/` platform tree + listeners.

Legend:
- **✅ direct** — `use phi_core::X` followed by usage of the type as-is.
- **🔌 wrap** — baby-phi composite holds a `phi_core::X` field.
- **♻ inherit** — surface reads a wrap/direct established at an
  earlier milestone; no new phi-core coupling introduced.
- **🏗 build-native** — pure baby-phi governance primitive, no
  phi-core counterpart exists.

## Predicted totals at M5 close (per plan §Part 1.5)

- **M4 close baseline**: 14 `use phi_core::` lines / 7 unique types.
- **M5 close target**: **~24 lines / 10 unique types** (+`Session`,
  `LoopRecord`, `Turn`, `AgentEvent`, `SessionRecorder`,
  `AgentTool` new; `AgentProfile`, `ExecutionLimits`,
  `ModelConfig`, `ThinkingLevel` carry over from M4).

Actual counts land here at each phase close.

## M5 aggregate table — what each page touches

| Page | Phase | Q1 predicted | Q2 transitive | Q3 rejections |
|---|---|---|---|---|
| Session persistence (foundation) | P1 | 3 (`Session`, `LoopRecord`, `Turn`) | `Session.inner` / `LoopRecordNode.inner` / `TurnNode.inner` via serde flatten | `save_session` / `load_session` JSON helpers |
| Session recorder wrap | P3 | 2 (`SessionRecorder`, `AgentEvent`) | `BabyPhiSessionRecorder.on_phi_core_event` consumes `AgentEvent` | `include_streaming_events = true` (stays `false` at M5) |
| Page 14 — First Session Launch | P4 | **4** (`agent_loop`, `agent_loop_continue`, `AgentTool`, `ModelConfig`) | `GET /sessions/:id` carries wrapped phi-core types; `GET /sessions/:id/tools` carries `AgentTool` summaries | `BasicAgent::run()` — M5 uses the raw `agent_loop()` free fn |
| s02 memory-extraction | P8 | 1 (`agent_loop`) | Reuses the same phi-core runtime primitive as session launch | (none new) |
| Page 12 — Authority Templates | P5 | 0 | 0 at wire tier | `phi_core::AgentEvent` orthogonal (not a governance trigger) |
| Page 13 — System Agents | P6 | 1 (`AgentProfile` for profile_ref validation — M4 pattern) | 0 at wire tier | `trigger` enum is governance (NOT `AgentEvent`) |
| s03 agent-catalog | P8 | 0 | 0 | Pure governance-plane |

## Compile-time coercion witnesses

3 new witnesses land in `domain/src/model/nodes.rs::tests` at P1
(applied to `Session.inner` / `LoopRecordNode.inner` /
`TurnNode.inner`). A rename in phi-core breaks the baby-phi build
immediately — the M3 / M4 discipline.

```rust
#[allow(dead_code)] fn _is_phi_core_session(_: &phi_core::session::model::Session) {}
#[allow(dead_code)] fn _is_phi_core_loop_record(_: &phi_core::session::model::LoopRecord) {}
#[allow(dead_code)] fn _is_phi_core_turn(_: &phi_core::session::model::Turn) {}
```

## Three phi-core surfaces M5 MIGHT miss-leverage if not pinned

Per plan §Part 1.5 §Three phi-core surfaces:

1. **`phi_core::BasicAgent`** — M5 launches via `agent_loop()` free
   function, NOT `BasicAgent::run()`. Pinned in ADR-0029 §D29.3.
2. **`SessionRecorderConfig::include_streaming_events = true`** —
   tempting for replay UX; don't. Default `false` at M5; revisit at
   M7b. Pinned in ADR-0029 §D29.5.
3. **`phi_core::save_session` / `load_session`** — JSON file
   helpers. Rejected; baby-phi uses SurrealDB via
   `BabyPhiSessionRecorder`. Pinned in ADR-0029 §D29.4 + plan
   §Part 1.5 §Q3.

## Enforcement at M5 close

- `scripts/check-phi-core-reuse.sh` extended at P0 — denylist adds
  `Session`, `Turn`, `AgentTool` (with wrap-layer exception at
  `modules/crates/domain/src/model/nodes.rs` for `Session`).
- `scripts/check-spec-drift.sh` regex broadened for lowercase `s`
  / `a` prefix ids.
- Positive close-audit greps per phase pinned in plan §Part 1.5 +
  §Part 4 per-phase subsections.

## Cross-references

- [M5 plan archive §Part 1.5](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [M4 phi-core reuse map](../../m4/architecture/phi-core-reuse-map.md) — cumulative platform + M4 table.
- [M3 phi-core leverage checklist](../../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
- [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md).
- [ADR-0030](../decisions/0030-template-node-uniqueness.md).
- [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).
