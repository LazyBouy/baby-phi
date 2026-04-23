<!-- Last verified: 2026-04-23 by Claude Code -->

# M5 — Templates, System Agents, First Session

Ships admin pages 12 (authority template adoption), 13 (system
agents config), 14 (first session launch). First milestone to
materialise `Session` / `LoopRecord` / `Turn` node persistence
(wrapping phi-core's three types per ADR-0029). First milestone to
launch a `phi_core::agent_loop` session from the platform with full
governance coverage (Permission Check preview, parallelize gate,
worker-saturation gate, cancellation, terminate, recorder-driven
persistence). First milestone to extend Template firing from A only
(M4) to A / C / D (M5), with E on demand via AR + B always-fires-on-Shape-B-AR-approve.

First milestone to wire **reactive supervisor agents**:
`memory-extraction-agent` (s02) subscribes to `SessionEnded`;
`agent-catalog-agent` (s03) subscribes to 8 edge + agent lifecycle
events. Closes six M4 carryovers (C-M5-1 through C-M5-6) + pins a
new M6 carryover C-M6-1 (Memory node tier + contract +
ownership-by-multi-tag + permission-over-time retrieval per D6
resolution).

Plan archive: [`../../../plan/build/01710c13-m5-templates-system-agents-sessions.md`](../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).

## Phase status

| Phase | Status | Scope |
|---|---|---|
| P0 — Post-flight delta + ADRs 0029/0030/0031 + base-plan M6 C-M6-1 amendment + CI grep extension + docs tree seed | [EXISTS] ✓ M5/P0 | archive plan; 10-item audit; 3 ADRs Proposed; base plan §M6 carries new `#### Carryovers from M5` subsection; `check-phi-core-reuse.sh` denylist +3 tokens; `check-spec-drift.sh` regex broadened for lowercase `s` / `a` prefix ids; docs tree seeded |
| P1 — Migration 0005, node wraps, web primitives, CLI scaffold | [PLANNED M5/P1] | 3-way Session wrap + ShapeBPendingProject/AgentCatalogEntry/SystemAgentRuntimeStatus composites + 8 schema changes + 4 web primitives + `phi session` CLI stubs |
| P2 — Repository expansion | [PLANNED M5/P2] | 14 new repo methods; flip `count_active_sessions_for_agent` stub |
| P3 — Event bus extensions + recorder wrap + Template C/D pure-fns + 4 listener scaffolds | [PLANNED M5/P3] | 8 new `DomainEvent` variants; `BabyPhiSessionRecorder`; Template C + D fire pure-fns (50-case proptests); 4 listeners wired in `AppState` (2 full bodies + 2 stubs) |
| P4 — Page 14 vertical (First Session Launch) | [PLANNED M5/P4] | **M5's biggest phase (5 carryover closes)**. Closes C-M5-2 (UsesModel writer), C-M5-3 (Session persistence), C-M5-4 (AgentTool resolver), C-M5-5 (ModelConfig change + real 409), C-M5-6 (Shape B materialise) |
| P5 — Page 12 vertical (Authority Template Adoption) | [PLANNED M5/P5] | Approve / deny / adopt-inline / revoke-cascade |
| P6 — Page 13 vertical (System Agents Config) | [PLANNED M5/P6] | Tune / add / disable / archive; live `SystemAgentRuntimeStatus` |
| P7 — `phi session` CLI + `phi agent update --model-config-id` + web polish + page 11 "Recent sessions" retrofit | [PLANNED M5/P7] | 4 session subcommands tail-by-default + `--detach`; web pages 12/13/14 + page-11 retrofit |
| P8 — s02 + s03 + s05 listener bodies | [PLANNED M5/P8] | `MemoryExtractionListener` body (audit-only at M5; M6 materialises nodes per C-M6-1); `AgentCatalogListener` body; Template C + D listener bodies (already at P3; P8 confirms via s05 acceptance) |
| P9 — Seal: cross-page acceptance + e2e first-session + CI + runbook + 3-agent re-audit | [PLANNED M5/P9] | Target ≥99% composite |

## ADRs

| # | Title | Status |
|---|---|---|
| [0029](decisions/0029-session-persistence-and-recorder-wrap.md) | Session persistence + SessionRecorder wrap | Proposed (→ Accepted at P3 close) |
| [0030](decisions/0030-template-node-uniqueness.md) | Template-node uniqueness (one shared row per kind) | Proposed (→ Accepted at P1 close) |
| [0031](decisions/0031-session-cancellation-and-concurrency.md) | Session cancellation + concurrency bounds | Proposed (→ Accepted at P4 close) |

## phi-core leverage (per-phase)

Per the [leverage checklist](../m3/architecture/phi-core-leverage-checklist.md)'s
four-tier enforcement model. M5 is the phi-core-heaviest milestone
yet — adds 3 new node-tier wraps (`Session`, `LoopRecordNode`,
`TurnNode` all wrapping `phi_core::session::model::*`) + direct
imports of `agent_loop`, `SessionRecorder`, `AgentEvent`,
`AgentTool`, `ModelConfig` across the `sessions/` platform tree +
listeners. See [phi-core-reuse-map.md](architecture/phi-core-reuse-map.md)
for the durable per-page table (seeded at P0, filled per-phase).

**Baseline at M5/P0**: 14 `use phi_core::` lines (7 unique types)
carried over from M4/P8. **Target at M5/P9**: ~24 lines (10 unique
types).

## Testing posture (plan §5)

Target: M4 close 805 Rust + 68 Web = **873** → M5 close **~1040**
combined (+~150 Rust / +~20 Web). Per-phase close audit runs the
**4-aspect** check (code correctness + docs accuracy + phi-core
leverage + **archive-plan compliance**) with explicit % target;
confidence reported before each next phase opens.

## Discipline — new at M5

1. **4-aspect confidence at every phase close** (upgraded from M4's
   3-aspect). Archive-plan compliance walks the phase's deliverables
   against the archived plan + marks each ✅/⚠/✗. Any ✗ blocks close.
2. **Phase-boundary pause is mandatory** — pinned since M3.
3. **Base-plan carryover for M6** — `#### Carryovers from M5`
   subsection landed at P0 pinning C-M6-1 (Memory contract +
   multi-tag ownership + permission-over-time retrieval).

## Cross-references

- [Base build plan §M5](../../../plan/build/36d0c6c5-build-plan-v01.md) — upstream scope definition + M5 carryovers from M3/M4.
- [Base build plan §M6 §Carryovers from M5](../../../plan/build/36d0c6c5-build-plan-v01.md) — C-M6-1 carryover landed at M5/P0.
- [M4/P8 close architecture](../m4/architecture/phi-core-reuse-map.md) — prior reuse-map baseline.
- [phi-core leverage checklist](../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
- [M4 post-flight delta](architecture/m4-postflight-delta.md) — M5/P0's 10-item verification audit.
