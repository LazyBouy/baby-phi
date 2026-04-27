<!-- Last verified: 2026-04-24 by Claude Code -->

# CH-02 — Real `agent_loop` + MockProvider wiring

**Plan file token:** `16fd9a3a` (generated via `openssl rand -hex 4`)
**Chunk ID:** CH-02 (see [forward-scope §1](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) and [§5](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md))
**Severity:** HIGH
**Expected effort:** ~6 engineer-days (forward-scope §5 was 5d; +1d for per-profile `mock_response` governance field + migration 0006 + repo layer — Q decided 2026-04-24)
**Chunks enabled after close:** CH-15, CH-16, CH-17, CH-21, CH-24

---

## §1 — Context & principle

### Why this chunk

M5/P4 shipped a 9-step launch flow at [`sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) whose execution step (Step 7, `spawn_replay_task` at line 380) fabricates a canonical 4-event sequence (`AgentStart → TurnStart → TurnEnd → AgentEnd`) rather than calling `phi_core::agent_loop()`. The `use phi_core::{agent_loop, agent_loop_continue}` imports exist only as compile-time witnesses pinned by `_keep_agent_loop_live` at line 98.

This is drift **D4.2** (tagged `leverage-violation`) — the single biggest concept-contradiction in the shipped M5 surface. Every downstream system-agent listener (memory extraction, catalog, Template C/D) is pretending to consume transcripts that aren't real. CH-02 replaces the synthetic feeder with a real `phi_core::agent_loop()` call driven by `MockProvider` (deterministic, no network). Real LLM providers defer to M7 per forward-scope §2.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* — M5.1 governing principle.

Applied here: CH-02 exercises phi-core's canonical execution primitive (`agent_loop`) at runtime, not merely as a compile-time witness. The drift stays open until `phi_core::agent_loop(...)` is on a hot execution path with live events streaming to a real recorder.

### Forward-scope reference

[Forward-scope §1 CH-02 block](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) + [§5 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md) | §"agent_loop free function" + §"Runtime-Only Types" | *"The phi-core public `agent_loop` free function is the canonical execution primitive. Runtime-only types (AgentLoopConfig, StreamConfig, StreamEvent) are ephemeral process state. Session recording materializes the event stream into persisted LoopRecord/Turn nodes."* | contradicted | honored |
| [`permissions/05-memory-sessions.md`](../../v0/concepts/permissions/05-memory-sessions.md) | §"session lifecycle" | *"Sessions run agents; transcripts are real execution output. Loops and Turns inherit their parent Session's tags for permission purposes."* | partially-honored | honored |
| [`agent.md`](../../v0/concepts/agent.md) | §"turn execution" | *"Sessions are always independent executions. Each runs its own `agent_loop()`."* | contradicted | honored |
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"memory extraction reads transcripts" | *"[Memory Extraction Agent] reads completed sessions, identifies candidate memories..."* (prerequisite: transcripts must be real) | partially-honored | honored (prerequisite satisfied; full honor at CH-21 body wiring) |

**Permissions subtree hook:** [`permissions/README.md`](../../v0/concepts/permissions/README.md) entry invariants are not directly touched by CH-02 (no new action vocabulary, no new selector shapes), but `permissions/05` session-lifecycle is cited above.

**phi-core-mapping hook:** every surface CH-02 touches is explicitly listed in §3 below with its `direct-reuse` classification per [`phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md).

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::agent_loop` (free fn) | imported; called only by compile-witness `_keep_agent_loop_live` | **direct-reuse** | invoke at runtime from spawned tokio task inside `sessions/launch.rs` |
| `phi_core::provider::mock::MockProvider` | not used | **direct-reuse** | import + instantiate via `MockProvider::text(...)` in new `sessions/provider.rs` |
| `phi_core::provider::traits::StreamProvider` | not used | **direct-reuse** (trait object) | `Arc<dyn StreamProvider>` return type of `provider_for` helper |
| `phi_core::types::context::AgentContext` | not used | **direct-reuse** (build inline; do NOT wrap) | construct via `build_agent_context` helper |
| `phi_core::agent_loop::AgentLoopConfig` | not used | **direct-reuse** (build inline; do NOT wrap) | construct with `provider_override: Some(...)` |
| `phi_core::types::event::AgentEvent` | already imported (listed in `session_recorder.rs`) | **direct-reuse** | no change |
| `phi_core::session::recorder::SessionRecorder` | wrapped by `BabyPhiSessionRecorder` | **wrap** (existing) | unchanged |
| `phi_core::types::message::AgentMessage` | already imported | **direct-reuse** | unchanged |
| `phi_core::agents::profile::AgentProfile` | wrapped by baby-phi `AgentProfile` via `blueprint` field | **wrap** (existing; extend) | add `mock_response: Option<String>` to baby-phi wrapper (NOT to the phi-core inner) |

**Expected import-count delta at chunk close:**
- `+5` new phi-core imports: `agent_loop` (runtime call, not just witness), `MockProvider`, `StreamProvider`, `AgentContext`, `AgentLoopConfig` across [`sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) and new [`sessions/provider.rs`](../../../../modules/crates/server/src/platform/sessions/provider.rs).

**Positive close-audit greps** (must return ≥ 1 each):
```bash
grep -rn "phi_core::agent_loop(" modules/crates/server/src/platform/sessions/
grep -rn "MockProvider::text" modules/crates/server/
grep -rn "AgentLoopConfig" modules/crates/server/src/platform/sessions/
grep -rn "mock_response" modules/crates/domain/src/model/nodes.rs
```

**Forbidden-duplication greps** (must return 0 each):
```bash
grep -rn "spawn_replay_task" modules/crates/                # function is replaced
grep -rn "^pub struct AgentProfile\b" modules/crates/       # no parallel redeclaration
grep -rn "^pub struct AgentContext\b" modules/crates/       # no parallel redeclaration
grep -rn "^pub struct AgentLoopConfig\b" modules/crates/    # no parallel redeclaration
bash scripts/check-phi-core-reuse.sh                        # exit 0
```

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D4.2` | [`../../v0/implementation/m5_1/drifts/D4.2.md`](../../v0/implementation/m5_1/drifts/D4.2.md) | HIGH | `in-chunk-plan → remediated` | Synthetic feeder replaced with real `phi_core::agent_loop()`; `leverage-violation` tag removed at seal |

---

## §5 — ADRs drafted

ADR numbers claimed at plan-drafting time per the chunk-lifecycle Q6 rule. Command used:

```bash
ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null \
  | xargs -I{} basename {} .md \
  | grep -oE "^[0-9]{4}" | sort -u | tail -5
# result: 0029, 0030, 0031 → next free = 0032
```

| # | Title | Drafted-at-phase | Decision summary | Flip to Accepted at |
|---|---|---|---|---|
| **ADR-0032** | MockProvider at M5 driven by `AgentProfile.mock_response`; real providers deferred to M7 | Step 2 (pre-P1) | `provider_for(runtime, profile)` returns `MockProvider::text(profile.mock_response.unwrap_or("Acknowledged."))` at M5; real provider dispatch via `ProviderRegistry` defers to M7 | Chunk seal (P5) |

ADR file path: [`../../v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md`](../../v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md)

---

## §6 — Prior-chunk regression re-verification

CH-02 is the first chunk opened under the per-chunk-plan discipline (no prior chunk uses this template). Re-verifies the **M5/P7 shipped state** that CH-02 builds on, not a prior chunk's invariants.

| Upstream | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| M5/P7 | `cargo test --workspace -- --test-threads=1` = 966 passed | `/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1 2>&1 \| grep -E "^test result:" \| awk -F'[;: ]+' '{s+=$4} END {print s}'` |
| M5/P7 | All 4 CI guards green | `bash scripts/check-doc-links.sh && bash scripts/check-ops-doc-headers.sh && bash scripts/check-phi-core-reuse.sh && bash scripts/check-spec-drift.sh` |
| M5.1 | M5.1 catalogue sealed, 60 drift files, 2 remediated | `ls docs/specs/v0/implementation/m5_1/drifts/D*.md \| wc -l` = 60 |
| M5/P4 | Launch flow 9-step structure intact at `launch.rs::launch_session` | `grep -c "Step [0-9]" modules/crates/server/src/platform/sessions/launch.rs` ≥ 9 |
| M5/P4 | Compound-tx writes Session + LoopRecordNode atomically | Inspect Step 6 of `launch_session` (unchanged by CH-02) |

These run at chunk-open AND at chunk-seal.

---

## §7 — Phases within the chunk

### P1 — AgentProfile `mock_response` field + migration 0006 (~1d)

**Goal.** Extend baby-phi's `AgentProfile` wrapper with a new governance field `mock_response: Option<String>`, add the backing schema column, and wire the repo layer.

**Deliverables.**
1. [`modules/crates/domain/src/model/nodes.rs`](../../../../modules/crates/domain/src/model/nodes.rs) around line 296 — add `pub mock_response: Option<String>` with `#[serde(default)]`.
2. [`modules/crates/store/migrations/0006_agent_profile_mock_response.surql`](../../../../modules/crates/store/migrations/) — new migration: `DEFINE FIELD mock_response ON TABLE agent_profile TYPE option<string>;`.
3. [`modules/crates/store/src/repo_impl.rs`](../../../../modules/crates/store/src/repo_impl.rs) — update `get_agent_profile_for_agent`, `put_agent_profile`, any SELECT strings for agent_profile; wire field round-trip.
4. Unit tests: AgentProfile serde round-trip with/without `mock_response`; repo persists + reads back.

**Tests.** +2 unit tests (serde + repo round-trip). Workspace baseline 966 → ~968.

**Concept-alignment check.** No concept-doc claim transitions in P1 (infrastructure layer).

**phi-core leverage check.** `check-phi-core-reuse.sh` green; `mock_response` is a governance field on the wrapper, not a phi-core type duplicate.

**Confidence target.** ≥ 97%.

**Pause discipline.** None anticipated — isolated schema + repo work.

---

### P2 — `provider_for` helper + `AgentContext` builder (~1d)

**Goal.** Introduce the two helpers `agent_loop` will consume.

**Deliverables.**
1. [`modules/crates/server/src/platform/sessions/provider.rs`](../../../../modules/crates/server/src/platform/sessions/provider.rs) — **new file** — `pub fn provider_for(runtime: &ModelProvider, profile: &AgentProfile) -> Arc<dyn StreamProvider>`. Body: `Arc::new(MockProvider::text(profile.mock_response.clone().unwrap_or_else(|| "Acknowledged.".to_string())))`. Doc-comment: "M7 swaps body for `ProviderRegistry` dispatch per ADR-0032."
2. [`modules/crates/server/src/platform/sessions/mod.rs`](../../../../modules/crates/server/src/platform/sessions/mod.rs) — add `pub(crate) mod provider;` and re-export the helper.
3. `build_agent_context(ctx: &SessionLaunchContext, prompt: &str, profile: &AgentProfile) -> AgentContext` — helper (placed either in `provider.rs` or a new `sessions/context.rs`). Populates `system_prompt` from `profile.blueprint.system_prompt`; sets `agent_id`/`session_id`/`loop_id` strings from ctx; `messages = vec![]` (prompt goes in via agent_loop's `prompts` param).
4. Unit tests: `provider_for` default path (None → "Acknowledged."), override path (Some → value); trait-object coercion compiles; `build_agent_context` id propagation + system-prompt passthrough.

**Tests.** +3 unit tests. Baseline ~968 → ~971.

**Concept-alignment check.** phi-core-mapping claim (§agent_loop free-function) partially flipped on this phase: builder helpers exist but not yet called at runtime.

**phi-core leverage check.** Greps `MockProvider::text`, `StreamProvider`, `AgentContext` ≥ 1 each; no duplicate struct definitions.

**Confidence target.** ≥ 97%.

**Pause discipline.** If `AgentContext` field set changes upstream in phi-core between P1 and P2 (unlikely but possible), pause for `AskUserQuestion`.

---

### P3 — spawn replacement + event drain (~1.5d)

**Goal.** Replace `spawn_replay_task` body with real `agent_loop` invocation.

**Deliverables.**
1. [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) lines 380-451 — rewrite body to:
   ```rust
   let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<phi_core::types::event::AgentEvent>();
   let mut ctx = build_agent_context(&launch_ctx, &prompt, &profile);
   let cfg = AgentLoopConfig {
       model_config,
       provider_override: Some(provider_for(&runtime, &profile)),
       ..Default::default()
   };
   let prompts = vec![AgentMessage::Llm(LlmMessage::new(Message::user(prompt)))];
   let cancel = cancel_token.clone();
   let agent_task = tokio::spawn(async move {
       phi_core::agent_loop(prompts, &mut ctx, &cfg, tx, cancel).await
   });
   while let Some(evt) = rx.recv().await {
       recorder.on_phi_core_event(evt).await;
   }
   let _ = agent_task.await;
   if let Err(e) = recorder.finalise_and_persist().await { tracing::error!(...); }
   registry.remove(&launch_ctx.session_id);
   ```
2. Rename the function from `spawn_replay_task` → `spawn_agent_task` (signals the semantic shift).
3. Update the dead-code witness at line 89-98: comment now "runtime-exercised via MockProvider at M5; pins against phi-core rename."
4. Update call-site at launch_session around line 342 to use new name.

**Tests.** No new unit tests at this phase; P4 covers behavioral verification via acceptance.

**Concept-alignment check.** All 4 concept-doc claims target status reached (`honored`). Matrix flip happens at P5 seal.

**phi-core leverage check.** Full set of positive greps (phi_core::agent_loop(, MockProvider, AgentLoopConfig) ≥ 1 each; `spawn_replay_task` → 0 hits.

**Confidence target.** ≥ 97%.

**Pause discipline.** If the real `agent_loop` emits events the BabyPhiSessionRecorder doesn't handle, pause — this would be a new drift.

---

### P4 — Rewrite acceptance tests in place (~1d)

**Goal.** Flip 3 of 4 tests in `acceptance_sessions_m5p4.rs` from synthetic-shape assertions to real-loop assertions.

**Deliverables.** Per the plan's test-migration table:

| Test | New assertion |
|---|---|
| `test_happy_path_records_1_loop_1_turn` | 1 loop + ≥1 turn; first turn `triggered_by = User`; first user message role = user; prompt text contained in input messages; final status = `Completed` |
| `list_sessions_in_project_strips_phi_core_inner` | Replace 150ms sleep with `wait_for_session_finalised` call; same header-strip assertion |
| `terminate_twice_returns_already_terminal_on_second_call` | Unchanged race-tolerance; second 409 guaranteed |
| `get_session_tools_returns_empty_list_at_m5` | Unchanged (MockProvider wires no tools) |

**Tests.** ~971 → ~971 (rewrites, not additions). Verify `npm run test` equivalent not broken — backend only.

**Concept-alignment check.** No new transitions; tests confirm P3 behavior.

**phi-core leverage check.** No new greps; P3's greps still green.

**Confidence target.** ≥ 97%.

**Pause discipline.** If the new MockProvider-driven turn shape surfaces an unforeseen difference (e.g., `TurnEnd.message` wraps the mock response differently than the synthetic `Message::user(prompt)`), pause and surface.

---

### P5 — ADR-0032 Accepted + drift lifecycle + close audit (~1d)

**Goal.** Flip the terminal governance state.

**Deliverables.**
1. ADR-0032 [`0032-mock-provider-at-m5.md`](../../v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md) — Status `Proposed` → `Accepted`. Fill conforming-real-provider criteria (see §11).
2. D4.2 drift file — `Status` → `remediated`; lifecycle entry `2026-04-24 — remediated — via CH-02 (plan 16fd9a3a-ch-02-real-agent-loop-wiring.md); synthetic feeder replaced with phi_core::agent_loop + MockProvider`; `leverage-violation` tag removed; grep-for-regression updated.
3. `_concept-audit-matrix.md` — flip `agent_loop direct-reuse` row from `contradicted` → `honored`; update Code-evidence column.
4. `drifts/README.md` index — refresh D4.2 Status column.
5. `phi-core-mapping.md` — bump `Last verified` header.
6. Spawn 2 audit agents (see §11).

**Tests.** No new tests. Focus on lifecycle integrity + audit pass.

**Concept-alignment check.** All 4 matrix rows at `honored`.

**phi-core leverage check.** Final green sweep.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** Audit findings → surface to user before seal.

---

## §8 — Tests summary

- **Expected total test count at chunk close:** 966 (baseline) + 5 new unit tests (2 P1 + 3 P2) − 0 (rewrites count the same) = **~971** serialised passing.
- **Layer breakdown:**
  - Unit (domain serde + repo round-trip + provider_for + context builder): +5
  - Acceptance (server/tests/acceptance_sessions_m5p4.rs): 3 rewritten, 1 unchanged = 4 still passing
  - Integration: unchanged
- **New test files:** none (all new tests land in existing test modules).
- **Expected-still-green fragile tests:** all current `acceptance_sessions_*.rs` tests (especially `acceptance_sessions_m5p6` and `acceptance_sessions_m5p7` which exercise list + show endpoints that consume the same launched sessions).

---

## §9 — Pre-chunk gate

**Reading list (drafter reads before chunk-open ritual completes):**
1. Every concept doc in §2 (phi-core-mapping, permissions/05, agent.md, system-agents.md).
2. [`../../v0/implementation/m5_1/drifts/D4.2.md`](../../v0/implementation/m5_1/drifts/D4.2.md).
3. [`../../v0/implementation/m5_1/process/per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md).
4. [`../../v0/implementation/m5_1/process/chunk-lifecycle-checklist.md`](../../v0/implementation/m5_1/process/chunk-lifecycle-checklist.md).
5. [`../../v0/implementation/m5_1/process/drift-lifecycle.md`](../../v0/implementation/m5_1/process/drift-lifecycle.md).
6. [`../forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §1 CH-02 + §7 Q&A.
7. [`../../../../CLAUDE.md`](../../../../CLAUDE.md) §phi-core Leverage rules 1–5.
8. Existing ADRs 0029 / 0030 / 0031 in [`../../v0/implementation/m5/decisions/`](../../v0/implementation/m5/decisions/) for style reference.

**Carry-forward invariants** (verified green at chunk-open):
- `cargo test --workspace -- --test-threads=1` = 966.
- 4 CI guards green.
- `git diff --stat HEAD -- modules/` empty.
- M5.1 drift count = 60.

**Chunk-ordering note.** CH-02 is user-selected as the first chunk opened after M5.1 seal (Q4 — user-decided per chunk-open). The forward-scope §4 dependency graph identifies CH-02 as the critical-path unblock for CH-15/16/17/21/24.

---

## §10 — Close criteria

**4 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — cargo test --workspace --test-threads=1 green (~971 passed); clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect** — D4.2 lifecycle entries present chronologically; matrix row flipped; README index current; ADR-0032 Accepted; phi-core-mapping verified-header bumped.
- **phi-core leverage aspect** — import-count delta = +5; all positive greps ≥ 1; all forbidden-duplication greps = 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — all 4 §2 rows at target-status `honored`.

**Two confidence % (named numerator/denominator):**
- **Implementation confidence** = `claims-verified-honored-by-tests-and-code-inspection / claims-in-scope-for-chunk` = **4/4 = 100%** target.
- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity / doc-pages-touched-in-chunk` = target **7/7 = 100%**.

Touched doc pages (denominator): ADR-0032 · D4.2 drift · `_concept-audit-matrix.md` · `drifts/README.md` · `phi-core-mapping.md` header · `acceptance_sessions_m5p4.rs` comment doc · migration 0006 doc-header.

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P5 seal phase specifically. Composite below target blocks close. No aspect-averaging, no rounding up.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 5 phases = medium chunk → **2 agents** (per P4 template rule).

**Audit aspects (a–d):**
- (a) Code correctness
- (b) Docs fidelity vs concept docs
- (c) Concept alignment across all 4 concept docs in §2
- (d) phi-core leverage (imports, no forbidden duplications, compile-time witness intact)

**Auditor constraint.** Fresh `Explore` subagents. Neither may be the implementer.

### Audit Agent A — Code + phi-core leverage

> **Prompt** (locked at Step 2; fired at P5 seal):
> You are performing an independent code + phi-core leverage audit of CH-02 in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code.
>
> Verify these claims against the current HEAD code state. For each claim report PASS or FAIL with 1-line evidence:
>
> 1. `phi_core::agent_loop(` is called at runtime from a spawned tokio task in `modules/crates/server/src/platform/sessions/launch.rs` (not only as compile-witness).
> 2. The caller task drains an `mpsc::UnboundedReceiver<phi_core::types::event::AgentEvent>` and calls `recorder.on_phi_core_event(evt).await` on each item.
> 3. The `CancellationToken` passed to `agent_loop` is the same token registered in `SessionRegistry::insert`.
> 4. `provider_for(runtime, profile)` at `sessions/provider.rs` returns `MockProvider::text(profile.mock_response.unwrap_or_else(|| "Acknowledged.".to_string()))` — verify both branches (None default + Some override).
> 5. `_keep_agent_loop_live` witness fn is preserved with updated comment.
> 6. `scripts/check-phi-core-reuse.sh` returns exit 0.
> 7. `grep -rn "spawn_replay_task" modules/crates/` returns 0 hits.
> 8. baby-phi's `AgentProfile` wrapper at `modules/crates/domain/src/model/nodes.rs` around line 296 has a new `mock_response: Option<String>` field with `#[serde(default)]`, and this field is NOT duplicated on phi-core's inner `blueprint` struct.
>
> Report each as PASS/FAIL. ≤ 500 words. Read-only.

### Audit Agent B — Docs fidelity + D4.2 lifecycle

> **Prompt** (locked at Step 2; fired at P5 seal):
> You are performing an independent docs audit of CH-02. You did NOT write these docs.
>
> For each claim report PASS or FAIL:
>
> 1. `docs/specs/v0/implementation/m5_1/drifts/D4.2.md` — Status = `remediated`; lifecycle history block contains 5 entries in chronological order: discovered → classified → scoped → in-chunk-plan → remediated; `leverage-violation` tag removed; `Last verified: YYYY-MM-DD` header bumped.
> 2. `_concept-audit-matrix.md` — the row for phi-core-mapping §"agent_loop free function" has status flipped from `contradicted` to `honored` + code-evidence column cites `launch.rs` with line numbers.
> 3. `drifts/README.md` — Status column for D4.2 shows `remediated`.
> 4. `docs/specs/v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md` — Status = `Accepted`; body contains (a) the `provider_for` signature; (b) the `AgentProfile.mock_response` field placement rationale; (c) the 4 conforming-real-provider criteria for M7.
> 5. `docs/specs/v0/concepts/phi-core-mapping.md` — `Last verified` header bumped; no semantic edit (claim was always correct).
> 6. Acceptance test file `acceptance_sessions_m5p4.rs` — comment/doc block at the top or relevant test(s) reflects that CH-02 replaced the synthetic feeder (no lingering "synthetic replay" wording on the rewritten tests).
>
> Report each as PASS/FAIL. ≤ 500 words. Read-only.

**Seal-blocking rule.** Both audits must report PASS on every check, OR any FAIL must be either (a) fixed in-chunk before seal, (b) reframed via user-approved ADR, or (c) converted to a new drift file with explicit future-chunk assignment before seal.

---

## §12 — Verification section

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1

# 3. CH-02-specific positive greps
grep -c "phi_core::agent_loop(" modules/crates/server/src/platform/sessions/launch.rs     # ≥ 1
grep -rn "MockProvider::text" modules/crates/server/                                       # ≥ 1 recursive
grep -rn "AgentLoopConfig" modules/crates/server/src/platform/sessions/                    # ≥ 1
grep -rn "mock_response" modules/crates/domain/src/model/nodes.rs                          # ≥ 1
ls modules/crates/store/migrations/0006_*.surql | wc -l                                    # 1

# 4. CH-02-specific negative greps
grep -rn "spawn_replay_task" modules/crates/                                               # 0 hits

# 5. Drift-file status
grep -c "^- \*\*Status\*\*: \`remediated\`" docs/specs/v0/implementation/m5_1/drifts/D4.2.md   # 1
grep -lE '^- \*\*Status\*\*: `remediated`' docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: 3 (D5.1 + D6.2 pre-M5.1 + D4.2 via CH-02)

# 6. ADR status
grep -c "^\- \*\*Status\*\*: Accepted" docs/specs/v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md   # 1
```

---

## Notes on M5.1/P3 Q&A binding

This plan honors all 7 planning decisions from [forward-scope §7](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md):

- **Q1** (storage-backend) — untouched by CH-02; CH-03 owns.
- **Q2** (selector PEG split) — untouched; CH-06 owns.
- **Q3** (consent triad sequencing) — untouched; CH-09/10/11 own.
- **Q4** (chunk ordering) — user-selected CH-02 as first chunk; honored.
- **Q5** (M5 scope — HIGH-all-M5) — CH-02 is HIGH, closes before M5 tag; honored.
- **Q6** (ADR numbering at draft time) — ADR-0032 claimed at §5 via the `ls … | grep -oE "^[0-9]{4}" | sort -u | tail -5` pattern; honored.
- **Q7** (uniform ExitPlanMode ritual) — this plan was approved via ExitPlanMode; honored.
