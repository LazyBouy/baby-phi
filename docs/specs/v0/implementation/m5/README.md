<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-SEAL — M5 milestone seal: all P0–P9 phases shipped ✓ via chunk equivalence (P0 = CH-K8S-PREP; P1–P7 = CH-NN milestones; P8a = CH-21; P8b = CH-22; P8c = CH-23; P8d + P9 = CH-24); CH-24 close summary appended; composite confidence ≥99%; milestone tag `v0.1-m5` staged for user execution. Cycle hex `5778bb77`.) -->

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

Plan archive: [`../../../plan/build/m5-templates-system-agents-sessions-01710c13.md`](../../../plan/build/m5-templates-system-agents-sessions-01710c13.md).

## Phase status

| Phase | Status | Scope | Closing chunk |
|---|---|---|---|
| P0 — Post-flight delta + ADRs 0029/0030/0031 + base-plan M6 C-M6-1 amendment + CI grep extension + docs tree seed | [EXISTS] ✓ | archive plan; 10-item audit; 3 ADRs Proposed; base plan §M6 carries new `#### Carryovers from M5` subsection; `check-phi-core-reuse.sh` denylist +3 tokens; `check-spec-drift.sh` regex broadened for lowercase `s` / `a` prefix ids; docs tree seeded | M5/P0 + CH-K8S-PREP |
| P1 — Migration 0005, node wraps, web primitives, CLI scaffold | [EXISTS] ✓ | 3-way Session wrap + ShapeBPendingProject/AgentCatalogEntry/SystemAgentRuntimeStatus composites + 8 schema changes + 4 web primitives + `phi session` CLI stubs | CH-15 |
| P2 — Repository expansion | [EXISTS] ✓ | 14 new repo methods; flip `count_active_sessions_for_agent` stub | CH-16 |
| P3 — Event bus extensions + recorder wrap + Template C/D pure-fns + 4 listener scaffolds | [EXISTS] ✓ | 8 new `DomainEvent` variants; `BabyPhiSessionRecorder`; Template C + D fire pure-fns (50-case proptests); 4 listeners wired in `AppState` (2 full bodies + 2 stubs) | CH-17 |
| P4 — Page 14 vertical (First Session Launch) | [EXISTS] ✓ | **M5's biggest phase (5 carryover closes)**. Closes C-M5-2 (UsesModel writer), C-M5-3 (Session persistence), C-M5-4 (AgentTool resolver), C-M5-5 (ModelConfig change + real 409), C-M5-6 (Shape B materialise) | CH-18 |
| P5 — Page 12 vertical (Authority Template Adoption) | [EXISTS] ✓ | Approve / deny / adopt-inline / revoke-cascade | CH-19 |
| P6 — Page 13 vertical (System Agents Config) | [EXISTS] ✓ | Tune / add / disable / archive; live `SystemAgentRuntimeStatus` | CH-20 |
| P7 — `phi session` CLI + `phi agent update --model-config-id` + web polish + page 11 "Recent sessions" retrofit | [EXISTS] ✓ | 4 session subcommands tail-by-default + `--detach`; web pages 12/13/14 + page-11 retrofit | CH-21 (s02), CH-22 (s03), CH-23 (CLI/web) |
| P8a — s02 (Memory Extraction Agent) listener body | [EXISTS] ✓ | `MemoryExtractionListener` body (audit-only at M5; M6 materialises nodes per C-M6-1) | CH-21 |
| P8b — s03 (Agent Catalog Agent) listener body | [EXISTS] ✓ | `AgentCatalogListener` body; 8 DomainEvent subscriptions | CH-22 |
| P8c — Template C + D listener bodies + system-flows s05 acceptance | [EXISTS] ✓ | confirms Template C + D firing pure-fns via s05 acceptance suite | CH-23 |
| P8d + P9 — Seal: carryover re-verification + per-page acceptance suite + e2e first-session + recent_sessions API-surface flip + runbook + troubleshooting + reuse-map refresh + 3-agent re-audit + milestone tag | [EXISTS] ✓ | Composite ≥99% confidence; CH-24 closed C-M5-1..C-M5-6 carryover re-verification + filed `D-CH24-recent-sessions-api-flip` (mid-cycle scope expansion, ADR-0059) | CH-24 |

## ADRs

| # | Title | Status |
|---|---|---|
| [0029](decisions/0029-session-persistence-and-recorder-wrap.md) | Session persistence + SessionRecorder wrap | Accepted (CH-17) |
| [0030](decisions/0030-template-node-uniqueness.md) | Template-node uniqueness (one shared row per kind) | Accepted (CH-15) |
| [0031](decisions/0031-session-cancellation-and-concurrency.md) | Session cancellation + concurrency bounds | Accepted (CH-18) |
| [0059](../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) | `recent_sessions` API-surface flip (CH-24 mid-cycle scope expansion) | Accepted (CH-24) |

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

## CH-24 close summary (M5 milestone seal — 2026-05-11)

CH-24 (cycle hex `5778bb77`) sealed M5 with a milestone-seal chunk:

- **Carryover re-verification (P-VERIFY)**: all 5 inherited M4 carryovers (C-M5-2 through C-M5-6) PASS against the real-loop output that CH-02 wired in (replacing the M5/P1-P7 MockProvider-stubbed transcripts that landed those carryovers initially). C-M5-1 was closed in M5/P1.
- **New acceptance suite (P-NEW-TESTS)**: 5 per-page acceptance files (`acceptance_m5_orgs.rs`, `acceptance_m5_projects.rs`, `acceptance_m5_agents.rs`, `acceptance_m5_sessions.rs`, `acceptance_m5_memory.rs`) + 1 cli-tier subprocess e2e (`e2e_first_session.rs`, at `cli/tests/` because `CARGO_BIN_EXE_phi` is package-scoped to the `cli` package). Net delta: 8 new tests at v2 close → 1537/0/2 workspace pass count.
- **Mid-cycle scope expansion (P-FLIP-RECENT-SESSIONS)**: P-NEW-TESTS authoring surfaced a `recent_sessions: Vec::new()` hardcode at `server/src/platform/projects/detail.rs:229+654` that M5/P4 had promised but never landed. User-locked at gate-2 to close in-chunk; 1 drift filed (`D-CH24-recent-sessions-api-flip`); 1 ADR ratified (ADR-0059 §D59.1–§D59.4); dedicated `Repository::list_recent_sessions_for_project(project_id, limit)` method added + new view-shape struct `RecentSessionEntry` replacing `RecentSessionStub`; tripwire assertion at `acceptance_m5_sessions.rs:224-230` flipped from "asserts empty" to "asserts contains session".
- **Docs (P-DOCS)**: ops runbook (`m5/operations/m5-ops-runbook.md`), troubleshooting end-to-end (`m5/user-guide/troubleshooting.md`), phi-core reuse-map refresh, drift file + concept-audit-matrix row + drift README index entry, ADR-0059, module doc-comment refresh at `detail.rs:33`.
- **Seal (P-SEAL)**: CI extensions in `.github/workflows/rust.yml` (5 server-tier acceptance + 1 cli-tier e2e); cycle-index row; forward-scope amendment for the 1-drift scope expansion; ADR-0059 status flipped Proposed → Accepted; milestone tag `v0.1-m5` staged for user execution.
- **Composite confidence**: ≥99% (impl 25/25 + docs 20/20 + code + phi-core Δ +0 + concept-alignment 7/7 honored).

## Cross-references

- [Base build plan §M5](../../../plan/build/build-plan-v01-36d0c6c5.md) — upstream scope definition + M5 carryovers from M3/M4.
- [Base build plan §M6 §Carryovers from M5](../../../plan/build/build-plan-v01-36d0c6c5.md) — C-M6-1 carryover landed at M5/P0.
- [M4/P8 close architecture](../m4/architecture/phi-core-reuse-map.md) — prior reuse-map baseline.
- [phi-core leverage checklist](../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model.
- [M4 post-flight delta](architecture/m4-postflight-delta.md) — M5/P0's 10-item verification audit.
- [CH-24 plan](../../../plan/build/ch-24-carryover-reverification-m5-seal-5778bb77/plan.md) — milestone-seal chunk plan archive.
- [ADR-0059](../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) — `recent_sessions` API-surface flip.
- [D-CH24-recent-sessions-api-flip](../m5_1/drifts/D-CH24-recent-sessions-api-flip.md) — drift remediated in CH-24 via mid-cycle scope expansion.
