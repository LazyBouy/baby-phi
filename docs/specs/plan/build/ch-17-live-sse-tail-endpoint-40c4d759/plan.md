<!-- Last verified: 2026-05-09 by Claude Code (CH-17 chunk-seal trivial-1L orchestrator inline patch on §3.C row 4 — user-guide tier shipped as `m5/user-guide/first-session-walkthrough.md` CH-17 amendment subsection rather than NEW `m5_2/user-guide/session-live-events-walkthrough.md` stub; deliberate avoid-fragmentation choice during paperwork phase; m5_2 user-guide tier deferred to M5_2-tag-close. Audit B iter 1 caught divergence as PARTIAL claim 11; no auditor re-spawn per CLAUDE.md trivial split.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-17 plan-approval gate close: all 6 forks + 1 sub-fork user-locked at gate-1 — F1.A / F2=64 / F3.B / F4=30s / F5.B (Action::Observe) / F6.B / F5.B.subfork.a (NEW migration 0016 backfilling Observe + extending Template A mint to [Read, Inspect, List, Observe]); chunk approved for P0 launch.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-17 chunk-plan v2 — user-locked-divergent-fork re-spawn at F5.B; iter-1 cited Action::Observe as a closed-set break, which was incorrect — Action::Observe IS canonical at action.rs:73,282 and concept-doc 03 line 22+44 makes Observability universal across every fundamental including session_object's data_object+tag constituents. Iter-2 reverts F5 to F5.B-locked path and surfaces a NEW sub-fork F5.B.subfork on legacy-grant back-compat; iter-1 §3.D claim that F5.B contradicts concept-doc is RETRACTED. Cycle hex `40c4d759`; chunk-planner v8 per CH-15 retro Row 5; iter-2 awaiting orchestrator + user F5.B.subfork lock + ExitPlanMode) -->
<!-- iter-1 (2026-05-09): drafted with F1.A / F2 / F3.B / F4 / F5.A / F6.B planner-recommendations; orchestrator escalated 6 forks. -->
<!-- iter-2 (2026-05-09): F5 user-locked to F5.B (Action::Observe); fact-base correction applied — Observe is already canonical (action.rs:282); F5.B.subfork on legacy-grant back-compat surfaced. -->

# CH-17 — Live SSE tail endpoint — chunk plan

**Forward-scope row** — [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §"Live SSE", lines 161–167.
**Cycle hex** — `40c4d759` (token derived from `openssl rand -hex 4` at draft time).
**Severity** — ⚠HIGH · ~1.0–1.3 days (iter-2 with F5.B-locked + F5.B.subfork(a) migration 0016).
**Drift closed** — D7.1.

---

## Forks for orchestrator (iter-2 — 1 NEW sub-fork; F1–F6 user-locked)

### Originally-forked decisions (locked at iter-1 plan-approval)

| ID | Decision | iter-1 options & recommendation | iter-2 user-lock |
|---|---|---|---|
| **F1** | `tokio::broadcast::Sender<AgentEvent>` placement | (a) trait-shaped per-session `SessionLiveStreamRegistry` (planner-recommended); (b) recorder-side-table; (c) global `OnceCell` (rejected). | **F1.A** ✓ planner-recommended. |
| **F2** | Channel buffer size | 16 / 64 / 256 / unbounded; recommended **64**. | **F2 = 64** ✓ planner-recommended. |
| **F3** | Lagging-receiver policy | (a) swallow + warn; (b) typed SSE error event then close (planner-recommended); (c) reconnect-signal. | **F3.B** ✓ planner-recommended. |
| **F4** | SSE keep-alive interval | 15 s / 30 s / 60 s; recommended **30 s**. | **F4 = 30 s** ✓ planner-recommended. |
| **F5** | **CRITICAL** — Permission gate on SSE endpoint | **(a)** reuse `[Read, Inspect, List]` (CH-15 launch parity — planner iter-1 recommendation). **(b)** use `Action::Observe` on `session_object` — **iter-1 REJECTED** under §3.D rule with stated rationale "would extend the closed 34-verb vocabulary". | **F5.B** ✗ user-locked iter-2 — **fact-base correction**: `Action::Observe` IS already in `Action::CANONICAL` at `domain/src/permissions/action.rs:282`; `as_str()` mapping at `:322` returns `"observe"`; concept-doc 03 line 22 lists `Observability \| observe, log, attest`; concept-doc 03 line 44 states *"Discovery, Authority, and Observability apply universally (every fundamental has list/inspect, delegate/allocate/transfer, and observe/log/attest)"*. F5.B does NOT break the closed-set invariant; semantically more precise — live event tailing IS an observability operation, not a generic Read. iter-1 §3.D claim retracted. |
| **F6** | `RecorderEvent` type definition | (a) new domain type wrapping `phi_core::AgentEvent`; (b) re-export `phi_core::types::event::AgentEvent` directly (planner-recommended). | **F6.B** ✓ planner-recommended. |

### NEW sub-fork (iter-2)

| ID | Decision | Options | Planner recommendation | Notes |
|---|---|---|---|---|
| **F5.B.subfork** | Back-compat for legacy Template A grants minted before CH-17 (existing grants carry `[Read, Inspect, List]` and lack `observe` — F5.B's `[Observe]` manifest fails Step 3 against them) | **(a)** New migration `0016_template_a_session_object_grant_add_observe.surql` parallel to CH-15's 0015 — walks every legacy `descends_from(Template A adoption AR)` grant whose `action` array does not yet contain `"observe"` and appends it. Also extend `domain/src/templates/a.rs::fire_grant_on_lead_assignment` to mint `[Read, Inspect, List, Observe]` going forward. **Test fixtures** in `domain/src/templates/a.rs` (15 callsites) + `domain/tests/template_a_firing_props.rs` (8 callsites) + `domain/src/events/listeners.rs:312` (production firing path) update from `[Read, Inspect, List]` to `[Read, Inspect, List, Observe]`. **(b)** Manifest-builder shape change: SSE handler builds manifest `[Observe, Read, Inspect, List]` (4-action manifest); engine's Step 3 requires every reach to match a grant, so the SSE path covers in 4 reaches — first 3 against the existing `[Read, Inspect, List]` grant, the 4th `Observe` reach falls back to a synthetic "observability is universal so any grant covering the resource implicitly grants observe" rule — **REJECT**: requires engine semantic change at Step 3 + breaks the explicit-action covering-grant model. **(c)** Hard-deny on legacy: SSE manifest `[Observe]` only; existing Template A grants don't cover SSE; operator must re-issue grants to use SSE. v0 has zero prod data so this is **operationally cheap today** but fragile — every persisted-fixture acceptance test (`acceptance_*.rs` files seeding Template A grants) breaks unless test fixtures are updated. | **(a)** — same migration shape as CH-15's 0015 (precedent); idempotent forward-only; `fire_grant_on_lead_assignment` extends from a 3-action set to a 4-action set (single-line callsite cascade per call site, ~24 sites, all in test fixtures + 1 production minter); preserves preview-launch parity at the manifest layer (preview/launch keep `[Read, Inspect, List]`; SSE alone uses `[Observe]`). Migration 0015 took ~150 lines of SurQL + a deterministic-seed acceptance test; 0016 will be **smaller** (~80 lines — purely an `UPDATE` with `array::push` rather than a CREATE-paired-grant). | **F5.B.subfork is the SOLE remaining locked-fork in iter-2.** Auto-approval criteria check (below) status flips to ESCALATE since (a) introduces a NEW migration (auto-approval criterion "no new migration" fails). |

### iter-2 Auto-approval criteria check (per orchestrator gate-1 rules)

- **Locked forks present?** F1–F6 already user-locked; F5.B.subfork is the new locked-fork → escalate to user.
- **Scope ratio vs forward-scope?** Forward-scope predicts ~1 day; iter-2 with F5.B.subfork(a) predicts ~1.0–1.3 days (migration 0016 + ~24 fire_grant_on_lead_assignment-test-fixture sites + 1 production minter line). **Within 1.0×–1.3× cap (≤ 1.5×)**.
- **phi-core leverage delta?** Unchanged from iter-1 — **+1** (`use phi_core::types::event::AgentEvent` in `events.rs`).
- **New K8s blocker class?** Unchanged from iter-1 — A1+A2+A6 light up; **CHK8S-D-10** filed.
- **Audit envelope size?** Iter-2 is **Small still** (1 auditor; 4 phases — P1 plumbing, P2 SSE handler + manifest, P3 CLI + docs, P-seal). Migration 0016 lands in P1 alongside the registry.
- **Confidence?** **9/10** on F5.B-locked path with F5.B.subfork(a). Risk vector: migration-runner-first-apply-race (axis A4 lights up). But ADR-0033 §D33.2 + the existing CH-15 migration 0015 precedent close this.
- **New migration?** **YES** — `0016_template_a_session_object_grant_add_observe.surql`. **This single criterion forces escalation to user** even though every other criterion holds.

**Verdict** → **Escalate to user** (F5.B.subfork lock + new migration).

---

## §1 — Context & principle

**Why this chunk.** Concept-doc 05 §"Standard Actions Applied to Sessions" line 242 names `read = "Retrieve a Session and its contents (Loops, Turns, Messages)"`; concept-doc 03 §"Action × Fundamental Applicability Matrix" lines 28–38 lists `Observability` as **universal across all 9 fundamentals** (line 44: *"Discovery, Authority, and Observability apply universally"*). Drift D7.1 (`m5_1/drifts/D7.1.md` lines 17–22) records that no `/events` SSE endpoint exists; CLI prints "(live tail deferred to M7)" at `cli/src/commands/session.rs:228`. Operators have no live-transcript surface. With CH-02 (real `phi_core::agent_loop()` runtime per ADR-0032) + CH-15 (real permission gate at session launch per ADR-0054) shipped, the prerequisite for a real SSE stream is in place: `BabyPhiSessionRecorder::on_phi_core_event` (`domain/src/session_recorder.rs:126`) is a single funnel through which every `AgentEvent` flows. CH-17 attaches a `tokio::broadcast::Sender<AgentEvent>` at that funnel, exposes a hard-deny-gated SSE handler at `GET /api/v0/sessions/:id/events` using **`Action::Observe`** on `session_object` (semantically precise observability gate per F5.B user-lock), and flips the CLI's deferred-tail print into a real stream consumer.

**Quality-over-speed restatement.** *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* For CH-17: the closed 34-verb action vocabulary (concept-doc 03 §"Standard Action Vocabulary"; `Action::CANONICAL.len() == 34` invariant at `domain/src/permissions/action.rs:439`) is **non-negotiable** — F5.B uses an action verb (`Observe`) that is already canonical (line 282), so no closed-set break occurs. Iter-1 incorrectly claimed F5.B would extend the vocabulary; iter-2 retracts that claim with the verified citations above.

**Forward-scope reference.** [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §"Live SSE", lines 161–167.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/permissions/05-memory-sessions.md` | §"Standard Actions Applied to Sessions", line 242 | `read (Data category) — Retrieve a Session and its contents (Loops, Turns, Messages)` | partially-honored — `GET /sessions/:id` returns a terminal `SessionDetail`; live-tail surface is absent (drift D7.1 line 20) | **honored** — `read` is the FULL-CONTENT action per concept doc; SSE stream surfaces in-flight `AgentEvent`s for an observability reader. Both action surfaces (`read` for fetch, `observe` for live-stream) are exercised by their respective endpoints |
| `concepts/permissions/03-action-vocabulary.md` | §"Standard Action Vocabulary" line 22 (Observability row) | `Observability \| observe, log, attest` | honored | **honored unchanged** — CH-17 exercises an already-canonical verb |
| `concepts/permissions/03-action-vocabulary.md` | §"Action × Fundamental Applicability Matrix" line 44 | `Discovery, Authority, and Observability apply universally (every fundamental has list/inspect, delegate/allocate/transfer, and observe/log/attest)` | honored — applicability matrix at `domain/src/permissions/action.rs:172,189,202` encodes universal-across-fundamentals for Discovery/Authority/Observability | **honored unchanged** — `Action::Observe.applies_to_composite(Composite::SessionObject) == true` (composite-inheritance: SessionObject = DataObject + Tag, both fundamentals admit Observability per row 28+35); the SSE manifest's `[Observe]` resolves at Step 3 against `session_object` |
| `concepts/permissions/05-memory-sessions.md` | §"Default Grants Issued to Every Agent", lines 253–268 (Default Grant 1) | `Default Grant 1: action: [read, list, inspect, append]; resource.selector: tags contains agent:<self>` | **NOT-IMPLEMENTED-IN-V0** — every `create_agent` callsite passes `default_grants: vec![]` (verified: `agents/create.rs:232` + 4 acceptance-test callsites). Concept-doc Default Grants are aspirational scaffolding; runtime grants come from Template A/B/C/D/E firings exclusively in v0. | **out-of-scope-for-chunk** — Default Grant 1 not implemented today; CH-17 does not change that. Acceptance tests use Template A grants. **Future M6+ chunk** ships Default Grant 1 issuance at agent creation; that chunk SHOULD include `observe` in the action set per F5.B. ADR-0055 §D55.5 captures this as a forward-defensive note. |
| `concepts/permissions/05-memory-sessions.md` | §"Authority Templates" lines 294–433 (A/B/C/D/E) | Each template issues a `[read, inspect, list]` grant on `session_object/project:<id>` (Template A — line 309), `agent:<subordinate>` (Template C), etc. | honored at launch — `fire_grant_on_lead_assignment` mints two grants per fire with `[Read, Inspect, List]` (lines 139–143 + 175–179) | **honored at SSE — extended** — F5.B.subfork(a) extends Template A's mint to `[Read, Inspect, List, Observe]`. Migration 0016 backfills legacy Template A grants. Templates B/C/D/E are M6+ scope (not yet implemented) so they will mint with the extended set when they ship. |
| `concepts/permissions/04-manifest-and-resolution.md` | §"All steps hard-gate (not advisory)" — invariant flipped to `honored` at CH-15 (per matrix line 181) | Every `Decision::Denied` → 403; advisory layer retired | honored at launch (CH-15 / ADR-0054 §D54.6) | **honored at SSE** — same hard-deny semantics applied at the SSE entrypoint |
| `concepts/permissions/README.md` | (entry invariants — required when permissions/01–09 cited) | Closed action vocabulary + closed fundamental kinds + closed selector grammar | honored | **honored unchanged** — F5.B uses `Action::Observe` which is already canonical |
| `concepts/phi-core-mapping.md` | §"Sessions / events" row | `phi_core::types::event::AgentEvent` is the canonical agent-loop event type; baby-phi consumes via the recorder | honored | **honored** — CH-17 adds the SSE wire-serialisation site (`+1` direct-reuse import in `server/src/platform/sessions/events.rs`) |

**Coverage check.** Every concept doc whose claims the chunk's code touches is in the table. iter-2 added 2 new rows (concept-doc 03 line 22 + line 44) confirming `Action::Observe`'s pre-existing canonical status — these are the verifications iter-1 missed.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::types::event::AgentEvent` | imported in `domain/src/session_recorder.rs:42` (recorder funnel), `server/src/platform/sessions/launch.rs:92` (mpsc routing) | direct-reuse | **+1 new import** in `server/src/platform/sessions/events.rs` (SSE handler serialises `AgentEvent` JSON onto the wire). |
| `phi_core::session::recorder::SessionRecorder` | composed via `BabyPhiSessionRecorder.inner` (`session_recorder.rs:92`) | direct-reuse | unchanged — broadcast Sender is added next to `inner`, not a parallel recorder |
| `phi_core::agent_loop` | runtime-exercised in `launch.rs:539` via `tokio::join!(agent_fut, drain_fut)` | direct-reuse | unchanged — the new broadcast tap sits between phi-core's mpsc producer and the existing recorder consumer |

**Expected import-count delta at chunk close** — **+1 phi-core import** (`use phi_core::types::event::AgentEvent;` in `server/src/platform/sessions/events.rs`). Net workspace baseline goes from **49 → 50**.

**iter-2 note: F5.B vs F5.A delta on phi-core leverage = 0.** F5.B changes the manifest-builder shape (described below in §3 cascade-artifact discipline) but does not change phi-core import surface.

**Positive close-audit greps:**
```bash
grep -rn "use phi_core::types::event::AgentEvent" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect ≥ 4 (was 3: session_recorder.rs:42 + launch.rs:92 + comment session.rs:28; CH-17 adds events.rs handler import)

grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect ≥ 50 (canonical no-`::` form per CH-15 retro Row 3; baseline 49 + 1 new)
```

**Forbidden-duplication greps (iter-2: `Action::Observe` removed from forbidden list — Observe is canonical):**
```bash
grep -rn "^pub struct RecorderEvent\b" /root/projects/phi/baby-phi/modules/crates/
# expect 0 hits — F6 user-locks at re-export of phi_core::AgentEvent; no new type

grep -rn "^pub enum SsEvent\b\|^pub enum SseEvent\b\|^pub struct SseRecorderEvent\b" /root/projects/phi/baby-phi/modules/crates/
# expect 0 hits — same anti-duplication invariant

grep -rnE "^\s*Action::(SessionRead|ReadEvents|Tail|Stream)\b" /root/projects/phi/baby-phi/modules/crates/
# expect 0 hits — closed 34-verb invariant honored (note: Action::Observe IS canonical, removed from forbidden list per iter-2 fact-base correction)
```

Per [`baby-phi/CLAUDE.md`](../../../../CLAUDE.md) §"phi-core Leverage" rules 1 + 4. `scripts/check-phi-core-reuse.sh` MUST stay green at chunk close.

### §3 cascade-artifact discipline (CH-13/14 retro v3 — full per-file breakdown; iter-2 expanded for F5.B)

CH-17's primary cascades are now **3** under F5.B:

**Artifact A — `BabyPhiSessionRecorder::new` callsites (registry plumbing — unchanged from iter-1):**
```bash
git -C /root/projects/phi/baby-phi grep -nE 'BabyPhiSessionRecorder::new\(' modules/crates/
```
Raw match-count at draft-time: **4** (verified at git HEAD `f42393a` / 2026-05-09 via the iter-2 grep above).

Per-file breakdown (verified):
- `modules/crates/server/src/platform/sessions/launch.rs:500` (production launch path — 1 site)
- `modules/crates/domain/src/session_recorder.rs:439` (test fixture)
- `modules/crates/domain/src/session_recorder.rs:559` (test fixture)
- `modules/crates/domain/src/session_recorder.rs:595` (test fixture)

Predicted aggregate edit-band: **4 sites**. Per-file fix is small (each call adds an `Option<broadcast::Sender<AgentEvent>>` argument or — preferred — a builder method `.with_broadcast(sender)`). Pause-discipline trigger: **PAUSE if actual cascade > 1.5× predicted (≥ 6 sites)** — would indicate a missed downstream that needs scope review.

**Artifact B — `recorder.on_phi_core_event` callsites:**
```bash
git -C /root/projects/phi/baby-phi grep -nE '\.on_phi_core_event\(' modules/crates/
```
Raw match-count at draft-time: **3+** (multi-call test fixtures).

**Cascade impact**: ZERO functional callsite edits — the broadcast Send happens INSIDE `on_phi_core_event` (the funnel). Every existing caller continues to work; the broadcast tap is internal. This is the **single-funnel architectural bet** that makes CH-17 a 1-day chunk.

**Artifact C — `fire_grant_on_lead_assignment` action-set cascade (NEW in iter-2 — F5.B.subfork(a) cascade):**

This is the F5.B-specific cascade. F5.B-subfork(a) extends Template A's mint from `[Read, Inspect, List]` to `[Read, Inspect, List, Observe]`. The literal-struct cascade (per CH-13 retro v3 — paste invocation + raw count + per-file breakdown):

```bash
git -C /root/projects/phi/baby-phi grep -nE 'fire_grant_on_lead_assignment\(' modules/crates/
```
Raw match-count at draft-time: **24** (verified at git HEAD `f42393a` / 2026-05-09).

Per-file breakdown:
- `modules/crates/domain/src/templates/a.rs:128` (production minter — 1 site, the one that gets the `Action::Observe` push)
- `modules/crates/domain/src/templates/a.rs:241,252,269,280,288,300,308,309,319,333,362,373,403,413` (14 test-fixture call sites in `templates::a::tests`)
- `modules/crates/domain/src/events/listeners.rs:312` (production firing path — 1 site)
- `modules/crates/domain/tests/template_a_firing_props.rs:68,82,97,105,112,120,128,135,136` (8 prop-test call sites)

Per-call-site edit:
- The **24 callsites** of `fire_grant_on_lead_assignment(args)` themselves do NOT change — the function signature stays `pub fn fire_grant_on_lead_assignment(args: FireArgs) -> Vec<Grant>`. Only the BODY of the function changes (a single 1-line edit at `templates/a.rs:139–143` and another at `:175–179`, appending `Action::Observe` to the action vec).
- **The cascade is therefore in the ASSERTIONS of the 24 test sites**, not the call signature. Test assertions like `assert_eq!(grants[0].action, vec![Action::Read, Action::Inspect, Action::List])` need to become `vec![Action::Read, Action::Inspect, Action::List, Action::Observe]`. Grep for action-equality assertions in test fixtures:

```bash
git -C /root/projects/phi/baby-phi grep -nE 'action.*Action::Read.*Action::Inspect.*Action::List' modules/crates/domain/src/templates/a.rs modules/crates/domain/tests/template_a_firing_props.rs
```
Predicted: ≤ 6 such assertions (most test sites assert on length/holder/resource shape, not the action set verbatim — the verbose action-equality assertion appears in a small subset of tests). Verify at P1 plan-revision. **Pause-discipline trigger: PAUSE if actual assertion-edit count > 12 sites (predicted upper bound 6 × 2 = 12).**

**Artifact D — `Action::Read | Action::Inspect | Action::List` usage in launch builder (iter-2: UNCHANGED for SSE; CH-17 does NOT touch launch builder):**
```bash
git -C /root/projects/phi/baby-phi grep -nE 'Action::(Read|Inspect|List)' modules/crates/domain/src/permissions/builders/session_launch.rs
```
Raw match-count at draft-time: **4** (lines 56 body + 76 test + 79 test docstring + comments).

**Cascade impact**: ZERO — F5.B preserves preview-launch parity via `build_session_launch_manifest` UNCHANGED (still `[Read, Inspect, List]`). CH-17 introduces a NEW sibling builder for SSE: `build_session_observe_manifest(project_id) -> Manifest` returning `[Action::Observe]` on `session_object`. The two builders are siblings; neither modifies the other. ADR-0055 §D55.5 captures this divergence.

### §3 additive-enum cascade discipline check (CH-12 retro)

CH-17 adds **zero new enum variants** in iter-2. iter-1 introduced `SessionError::SessionLiveStreamUnavailable` — that stays. F5.B does NOT add a new `Action` variant (Observe pre-exists). `SessionError::SessionLiveStreamUnavailable` cascade analysis from iter-1 stands: 2 amendments at `wire_code_for` + `http_status_for` in `server/src/platform/sessions/mod.rs:157–225`. No other callsites.

### §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | `InProcessSessionLiveStreamRegistry { inner: DashMap<SessionId, broadcast::Sender<AgentEvent>> }` is pod-local by definition | **YES** | Trait-shape NOW: `pub trait SessionLiveStreamRegistry: Send + Sync` (mirrors `SessionRegistry` per ADR-0033 §D33.1). M7+ Redis-pubsub swap ships as a new impl, not a refactor. **File CHK8S-D-10**. |
| **A2** | New IPC channel | `tokio::sync::broadcast::channel::<AgentEvent>(64)` is a pod-local IPC channel | **YES** (architectural — it IS the substrate) | Document explicitly. Cross-pod fan-out requires Redis pub/sub; covered by CHK8S-D-10. |
| **A3** | New pod-local resource (file, socket, sub-process) | None — broadcast is in-memory only | NO | — |
| **A4** | Migration runner / first-apply race | **Migration 0016** lands per F5.B.subfork(a). iter-2 lights up A4 (was: NO at iter-1; flipped to YES at iter-2). | **YES** | Migration follows ADR-0033 §D33.2 ledger pattern + CH-15's 0015 shape; no new K8s blocker class — A4 is a pre-existing "migration applies under multi-pod start" axis already mitigated by the migration-runner pattern. iter-2 does NOT add a new ledger entry for A4 (the existing ledger CHK8S-D-3 already covers migration-first-apply). |
| **A5** | Trait-shape requirement | `SessionLiveStreamRegistry` MUST be a trait so future impl can be `RedisPubSubLiveStreamRegistry` (Sender→XADD; SSE handler→XREAD blocking) | **YES** | Ship trait now (single-impl-trait pattern; `Arc<dyn SessionLiveStreamRegistry>`). |
| **A6** | Cross-pod state sharing | An SSE client connecting to pod A sees only events from sessions hosted on pod A. Pod-B session events do NOT reach pod-A SSE clients without Redis pub/sub fan-out. | **YES** (single-pod-only assumption) | Document the assumption in ADR-0055 §D55.7 + the architecture page §"K8s readiness" + CHK8S-D-10 ledger entry. |
| **A7** | Audit hash-chain symmetry | SSE handler writes ZERO audit events on success path (read-only stream). On 403 it emits a NEW `platform.session.live_stream_denied` audit event (Alerted-class, parallel to CH-15's `platform.session.launch_denied`) — single-writer guarantee preserved. | NO (matches existing single-writer pattern) | Document audit-event in ops doc + cite CH-15 ADR-0054 §D54.5 precedent. |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — chunk does NOT touch the registry. Adds a SIBLING `SessionLiveStreamRegistry` trait. Trait-object dispatch preserved.
- D33.2 (`SurrealStore::open_remote`) — iter-2 chunk DOES add a storage migration (0016). The migration runner already honours D33.2's `_migrations` ledger pattern; 0016 follows the 0015 forward-only / idempotent shape. NA for the trait/registry surface; covered for the migration surface.
- D33.3 (SIGTERM graceful shutdown) — chunk adds NO new `tokio::spawn` tasks (the SSE stream consumes from the broadcast; axum's SSE response future is per-connection and ends when the client disconnects). **No SIGTERM handler change needed**.
- D33.4 (`EventBus.shutdown` + `drain`) — chunk does NOT touch `EventBus`; the broadcast channel is orthogonal (per-session, not cross-cutting governance).

**Conclusion paragraph.** **K8s-negative** — CH-17 introduces 1 new K8s-blocker class (single-pod broadcast fan-out), filed as **CHK8S-D-10 — Redis-pub/sub-backed `SessionLiveStreamRegistry` impl for cross-pod live-event fan-out**. iter-2 lights up A4 as well but reuses the existing migration-runner mitigation; no new ledger entry beyond CHK8S-D-10.

**CHK8S-D-10 ledger entry draft (filed at CH-17 plan-approval):**
- ID: `CHK8S-D-10`
- Title: `Redis-pub/sub-backed SessionLiveStreamRegistry impl for cross-pod live-event fan-out`
- Severity: **HIGH** (operator UX degraded under multi-pod deploy: SSE consumers connected to pod A miss session events produced on pod B)
- Origin: CH-17 P-1 (trait-shape `SessionLiveStreamRegistry`)
- Successor target: M7b "externalize SessionLiveStreamRegistry"
- Cross-ref: ADR-0055 §D55.7; ADR-0033 §D33.1 precedent; readiness doc §B1, §B2

### §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/session-live-events.md` (NEW) | YES — new design page describing `SessionLiveStreamRegistry` trait + `tokio::broadcast` semantics + SSE 9-step gate flow + K8s axis A1/A2/A6 conformance + **F5.B `Action::Observe` rationale** + migration 0016 reference | (a) update in-chunk (P3 deliverable) |
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/session-launch-permission-gate.md` (existing) | YES — CH-15's design page references `build_session_launch_manifest` as the launch-time gate; CH-17 adds a paragraph noting **the SSE gate uses `build_session_observe_manifest` (sibling builder, `[Observe]`-only) — NOT the launch builder** + cross-link to the new live-events page | (a) update in-chunk (P3 deliverable) |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/session-live-events-operations.md` (NEW) | YES — new ops page: `platform.session.live_stream_denied` audit-event row (parallel to launch_denied), reconnect playbook, `Lagged` event semantics, error-code reference (`SESSION_LIVE_STREAM_UNAVAILABLE` 410, `PERMISSION_CHECK_FAILED_AT_STEP_<N>` 403), tracing logs + **migration 0016 runbook entry (when running fresh DB vs upgrading from CH-15-era DB)** | (a) update in-chunk (P3 deliverable) |
| **User-guide** | ~~`docs/specs/v0/implementation/m5_2/user-guide/session-live-events-walkthrough.md` (NEW)~~ — **AMENDED at chunk-seal Trivial-1L orchestrator patch (Audit B iter 1 PARTIAL claim 11)**: content folded into `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` CH-17 amendment subsection rather than a new m5_2 stub, because the existing m5 walkthrough is the discoverable single-walkthrough surface for users (avoiding 2-file fragmentation). **`m5_2/user-guide/` directory remains intentionally unused at CH-17 — defer to M5_2-tag-close cleanup if the tier needs to materialize.** Final shipped artifact: `m5/user-guide/first-session-walkthrough.md` "CH-17 amendment — live SSE tail (2026-05-09)" subsection covering `phi session launch` (live tail visible by default) + `--no-tail` flag (detach-equivalent), `curl … /events` direct-curl example, reconnect after `Lagged`, AgentEvent variant glossary, 403-debug runbook citing `Action::Observe` on `session_object`. | (a) update in-chunk (P3 deliverable) — SHIPPED via m5 amendment, not m5_2 NEW |
| **User-guide** | `docs/specs/v0/implementation/m5_2/user-guide/cli-reference-mN.md` (existing if present; otherwise the m5/user-guide CLI reference) — verify at draft time | YES — `phi session launch` flag table updated to remove the "(live tail deferred to M7)" caveat; new flag `--no-tail` documented (preserves prior detach-equivalent behaviour) | (a) update in-chunk (P3 deliverable) |
| **Operations** | `m5_1/architecture/session-recorder.md` (verify existence at draft time) | If exists, YES — recorder body now publishes to broadcast tap, doc page must add the `tokio::broadcast::Sender<AgentEvent>` field + the publishing semantics | (a) update in-chunk if exists; (b) defer to follow-up if recorder doc is governance-tier-only |
| **Operations / migrations** (NEW iter-2) | `docs/specs/v0/implementation/m5_2/operations/migrations.md` (verify existence; create if absent) | YES — append migration 0016's purpose + idempotency note + run-order vs 0015 | (a) update in-chunk (P3 deliverable) |

**Doc-sync sweep at gate-2** (per repo CLAUDE.md widened sweep, CH-15 retro Row 1): grep ALL `m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` for the canonical stale-narrative phrase set: `live tail deferred`, `deferred to M7`, `(live tail deferred`, `SSE deferred`, `D7.1`, `is NOT emitted`, `not emitted at CH-NN`. Patch any matches BEFORE dispatching auditors.

### §3.D — Forward-scope-vs-concept-doc precedence (iter-2: REVISED — only F6 contradiction remains)

**iter-1 listed 2 contradictions; iter-2 retracts the F5 contradiction.**

**Contradiction 1 (RETRACTED in iter-2)** — iter-1 incorrectly claimed F5.B (`Action::Observe`) would extend the closed 34-verb vocabulary. **Verified false**: `Action::Observe` IS in `Action::CANONICAL` at `domain/src/permissions/action.rs:282`; `as_str()` mapping at `:322` returns `"observe"`; concept-doc 03 line 22 lists Observability's three verbs (`observe, log, attest`); concept-doc 03 line 44 states `Discovery, Authority, and Observability apply universally`. F5.B exercises an already-honored vocabulary. **No §3.D action needed for F5**; the SSE gate using `Action::Observe` is concept-aligned, not concept-divergent.

**Contradiction 2 (UNCHANGED — re-interpretation stands)** — Forward-scope row says "streams `RecorderEvent`". **No `RecorderEvent` type exists in the codebase**:

```bash
git -C /root/projects/phi/baby-phi grep -nE 'pub (struct|enum|type) RecorderEvent' modules/crates/
# returns: (no output)
```

`RecorderEvent` is also not present in concept-doc 05 or any other concept doc. **Re-interpretation**: per F6 (planner-recommended (b) — re-export `phi_core::types::event::AgentEvent`), the forward-scope literal `RecorderEvent` is scoping-gloss for "the events the recorder funnels". Implementation surface: `phi_core::types::event::AgentEvent`. Documented in **ADR-0055 §D55.6**.

**§3.D escalation status**: F6 alone triggered §3.D; F5 no longer does. iter-2 escalates ONLY because of F5.B.subfork (new migration). User-lock at gate 1 confirms:
- F6 → re-export `AgentEvent` (no new type)
- F5.B → use `Action::Observe` on `session_object` (already canonical, semantically precise)
- F5.B.subfork(a) → migration 0016 + extended `fire_grant_on_lead_assignment`

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D7.1` | [`m5_1/drifts/D7.1.md`](../../../v0/implementation/m5_1/drifts/D7.1.md) | HIGH | `discovered` (current) → `remediated` at chunk seal | Concept claim "operators observe via live event stream" — closed via SSE handler + CLI default-tail flip. Drift file's "where visible in code" pointer at `cli/src/commands/session.rs:228` returns 0 hits at chunk seal (line 32 of D7.1 says expect 1 hit while drift open; 0 post-remediation). |

**No new drifts discovered at iter-2 plan time.** If P3 surfaces a new K8s blocker beyond CHK8S-D-10 (e.g., audit-event hash-chain regression on the `live_stream_denied` event), it's added to this table per template §4 mid-flight rule.

**iter-2 §4 amendment** — concept-audit-matrix row for `permissions/05` will update from "honored unchanged" to "extended at SSE": Template A grants now mint `[Read, Inspect, List, Observe]` post-CH-17 (was `[Read, Inspect, List]` pre-CH-17). The pre-existing-behaviour preservation note in ADR-0055 §D55.8 cites this explicitly.

---

## §5 — ADRs drafted

**ADR-0055 — SSE broadcast fan-out + keep-alive + per-session live-stream registry + observe-action gate.**

- ADR number assignment: highest existing ADR is **ADR-0054** (CH-15); next free is **ADR-0055**.
- File path: `docs/specs/v0/implementation/m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md`.
- Drafted-at-phase: P0 (Proposed), flipped to Accepted at chunk seal (P-seal).
- Decision summary: F1.A (per-session trait-shaped registry) + F2 (buffer 64) + F3.B (`Lagged` → typed SSE error event + close) + F4 (30 s keep-alive) + **F5.B (use `Action::Observe` on `session_object` for the SSE gate — sibling builder `build_session_observe_manifest` returns `[Observe]`)** + F5.B.subfork(a) (migration 0016 + extended `fire_grant_on_lead_assignment` mints `[Read, Inspect, List, Observe]`) + F6.B (re-export `phi_core::types::event::AgentEvent` as the wire format). Sub-decisions D55.1 through D55.9 cover each fork plus the K8s readiness paragraph. (iter-2 adds D55.9 for migration 0016.)

**ADR-body checklist v3 (per CH-13/14 retro):**

1. **§"Forks" header with explicit user-lock outcome.** Drafted as: `Forks (F1.A / F2 (buffer=64) / F3.B / F4 (30s) / F5.B (Action::Observe) / F5.B.subfork(a) (migration 0016) / F6.B — F1–F6 user-locked at iter-1 plan approval; F5 re-locked at iter-2 to F5.B with corrected fact-base; F5.B.subfork(a) user-locked at iter-2 plan approval)`. Locked-state captured.

2. **§"Cross-references" with all 4 categories.**
   (a) **Originating concept docs**:
   - `concepts/permissions/05-memory-sessions.md` §"Standard Actions Applied to Sessions" line 242 (the canonical `read` semantics — out-of-scope-for-SSE since SSE uses `observe`); §"Default Grants Issued to Every Agent" lines 253–290 (Default Grants 1+2 — out-of-scope-in-v0); §"Authority Templates" lines 294–433 (A/B/C/D/E grant shape — Template A extended in iter-2).
   - `concepts/permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" line 22 (Observability row — `observe, log, attest`); §"Action × Fundamental Applicability Matrix" line 44 (Observability universal across fundamentals — the cited rationale for F5.B).
   - `concepts/permissions/04-manifest-and-resolution.md` §"All steps hard-gate" (CH-15 invariant, reused at SSE).
   - `concepts/phi-core-mapping.md` §"Sessions / events" row.

   (b) **Closed drifts by ID**: `D7.1` (HIGH).

   (c) **Prior ADRs cited as precedent (milestone-prefixed paths per CH-08 retro Row 1):**
   - `m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md` — CH-15 manifest-builder + hard-deny precedent. **iter-2 extension cite**: F5.B introduces a SIBLING manifest-builder `build_session_observe_manifest` for SSE; preview-launch parity preserved (preview/launch keep `[Read, Inspect, List]`).
   - `m5_2/decisions/0048-per-session-consent-gating.md` — CH-11 launch-flow consent boundary (clarifies CH-17 stays consent-free).
   - `m5_2/decisions/0033-k8s-prep-refactors.md` — CH-K8S-PREP §D33.1 trait-shaping precedent for `SessionLiveStreamRegistry` (mirrors `SessionRegistry`); §D33.2 migration-runner ledger pattern (basis for migration 0016 idempotency).
   - `m4/decisions/0028-domain-event-bus.md` — CH-K8S-PREP / CH-04 governance event-bus design (clarifies CH-17's broadcast is orthogonal to `EventBus`).
   - `m5/decisions/0029-session-persistence-and-recorder-wrap.md` — CH-M5/P3 recorder-wrap design (where the broadcast tap attaches).
   - `m5/decisions/0031-session-cancellation-and-concurrency.md` — `SessionRegistry` precedent + 503 saturation gate (`SessionLiveStreamRegistry` mirrors this shape).
   - `m5_2/decisions/0032-real-agent-loop-with-mock-provider.md` — CH-02 the real `agent_loop` runtime that produces the events being streamed.

   (d) **Forward-scope row** (mandatory per CH-13 retro Row 1): [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §"Live SSE", lines 161–167.

3. **Pre-existing-behaviour preservation note** (per CH-14 retro Row 10): documented in ADR-0055 §D55.8:
   > *"Pre-CH-17 behaviour preserved: the CLI's `phi session launch` print at `cli/src/commands/session.rs:228` ('(live tail deferred to M7 — `phi session show --id <id>` inspects terminal state)') is retired. The `--detach` flag's wire shape is preserved verbatim — `--detach` continues to skip the live tail and return the JSON receipt only. Pre-CH-17, every launch was effectively `--detach`. CH-17 makes `--detach` semantic for the first time. The `permission_check.decision` field on the launch receipt is preserved; CH-17 adds zero fields to the receipt schema. **Iter-2-specific preservation note**: `Action::Observe` was already canonical at `domain/src/permissions/action.rs:73,282,322` pre-CH-17 (introduced at CH-04 / ADR-0043 when the 34-verb enum landed). CH-17 does NOT introduce the `Observe` variant; it merely exercises the pre-existing canonical verb on `session_object` for the first time. Migration 0016 walks legacy Template A grants and appends `\"observe\"` to their action arrays — preserving every other field (holder, descends_from, delegable, issued_at, approval_mode, audit_class) verbatim. The pre-CH-15 single-grant-per-fire shape is unchanged; the pre-CH-15 paired-grant migration 0015 is unchanged; 0016 is purely additive."*

   Pre-CH-17 absent surface: `GET /api/v0/sessions/:id/events` returned 404 ROUTE_NOT_FOUND. CH-17 is the first writer.

**Sub-decisions (iter-2 — D55.1 through D55.9):**
- **D55.1** — F1.A trait-shaped per-session `SessionLiveStreamRegistry` (mirrors ADR-0033 §D33.1 `SessionRegistry`).
- **D55.2** — F2 buffer = 64 (config-driven via `[session_live_stream] buffer = 64`).
- **D55.3** — F3.B Lagged → typed SSE error event then close.
- **D55.4** — F4 keep-alive interval = 30 s.
- **D55.5** — **F5.B `Action::Observe` on `session_object` for the SSE gate (iter-2 user-lock).** Sibling builder `build_session_observe_manifest(project_id) -> Manifest` returns `[Observe]` on `session_object`. Pre-existing canonical status of `Action::Observe` cited explicitly: `domain/src/permissions/action.rs:73` (variant), `:282` (CANONICAL array), `:322` (`as_str()` returns `"observe"`); concept-doc 03 line 22 (Observability row) + line 44 (universal applicability). **NOT a new verb. NOT a closed-set break.** Iter-1's claim to the contrary is retracted in this ADR. Semantically more precise than `[Read, Inspect, List]` because live-event tailing IS an observability operation per concept-doc nomenclature.
- **D55.6** — F6.B re-export `phi_core::types::event::AgentEvent`; §3.D re-interpretation rationale for forward-scope literal `RecorderEvent` (scoping-gloss — names "the events the recorder funnels").
- **D55.7** — K8s readiness paragraph + CHK8S-D-10 ledger reference.
- **D55.8** — Pre-existing-behaviour preservation note (above).
- **D55.9** (iter-2 NEW) — **F5.B.subfork(a) migration 0016 + extended `fire_grant_on_lead_assignment` mint.** Migration `0016_template_a_session_object_grant_add_observe.surql` walks every legacy Template A grant (provenance: `descends_from` an AR with `kinds CONTAINS '#template:a'`; live: `revoked_at = NONE`) and appends `"observe"` to the `action` array if not already present (idempotent). Forward-only per ADR-0012. Production minter at `domain/src/templates/a.rs:128 fire_grant_on_lead_assignment` extends BOTH grants (`project_grant` + `session_grant`) to mint `[Read, Inspect, List, Observe]` going forward. ~24 test-fixture call sites in `templates::a::tests` + `template_a_firing_props.rs` + `events/listeners.rs:312` are updated where assertions verify the action set verbatim (predicted ≤ 12 assertion edits per Artifact C cascade analysis). **Authority Chain preservation (ADR-0053 / CH-14)**: 0016 leaves `descends_from` untouched; provenance walks transparent over the modified action array.

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-02** (real `agent_loop`) | `phi_core::agent_loop` runtime call site at `launch.rs:539` produces `AgentEvent`s through the mpsc → recorder funnel | `git -C /root/projects/phi/baby-phi grep -nE 'phi_core::agent_loop\b' modules/crates/server/src/platform/sessions/launch.rs` — expect ≥ 1 hit |
| **CH-04** (Action enum) | `Action::Observe` is canonical (action.rs:73,282,322); `Action::CANONICAL.len() == 34` | `cargo test -p domain permissions::action::tests::canonical_contains_thirty_four_variants 2>&1 \| tail -5` — expect 1 PASS |
| **CH-K8S-PREP** (ADR-0033) | `SessionRegistry` trait-shape preserved; `Arc<dyn SessionRegistry>` dispatch through `AppState` | `git -C /root/projects/phi/baby-phi grep -nE 'pub trait SessionRegistry' modules/crates/server/src/state.rs` — expect 1 hit |
| **CH-15** (real permission gate) | `build_session_launch_manifest(project_id)` UNCHANGED at `[Read, Inspect, List]` on `session_object`; preview/launch parity invariant via the typed builder | `cargo test -p domain --test session_launch_manifest_test 2>&1 \| tail -5` (or named tests `builder_produces_action_set_matching_template_a_grant_action_set` + `builder_resource_is_session_object_composite`); expect green AFTER iter-2's Template A test-fixture updates (the matching test asserts the launch builder's `[Read, Inspect, List]`, NOT the Template A mint's `[Read, Inspect, List, Observe]` — they're different surfaces) |
| **CH-15** (hard-deny invariant) | Every `Decision::Denied` from `check()` → 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>`; advisory-layer retired | `git -C /root/projects/phi/baby-phi grep -nE 'PERMISSION_CHECK_FAILED_AT_STEP' modules/crates/` — expect ≥ 2 hits (server error code + acceptance test) |
| **CH-15** (Template A migration 0015) | Migration 0015 idempotent + ledger-aware | `cargo test -p store migration_0015 2>&1 \| tail -5` — expect green; iter-2's migration 0016 mirrors its shape |
| **CH-11** (per-session consent) | Launch-time consent gate at Step 3.5; SSE endpoint stays consent-free per F5.B reasoning (consent is launch-time, not stream-time) | manual review: ADR-0055 §D55.5 cites why SSE skips consent |
| **M5/P3** (recorder wrap) | `BabyPhiSessionRecorder::on_phi_core_event` is the single funnel; broadcast tap attaches here | `git -C /root/projects/phi/baby-phi grep -nE 'pub async fn on_phi_core_event' modules/crates/domain/src/session_recorder.rs` — expect 1 hit (line 126) |
| **CH-22** (agent-catalog listener — `handler_count_is_five_at_m5` test) | Bus handler count remains 5 after CH-17; broadcast Sender is NOT a bus listener | `cargo test -p server --test state_test handler_count_is_five_at_m5 2>&1 \| tail -5` — expect green |
| **CH-14** (Authority Chain preservation) | `walk_provenance_chain` traverses Template A grants transparently; migration 0016 leaves `descends_from` untouched | `cargo test -p domain authority_chain 2>&1 \| tail -5` — expect green |

This table runs at chunk OPEN (P0 gate) and again at chunk SEAL (P-seal). Any regression produces a new drift file + surfaces as an open question for user before the chunk proceeds.

---

## §7 — Phases within the chunk

CH-17 is structured as **3 phases** + a seal phase. Iter-2 keeps the same phase boundaries; F5.B.subfork(a) deliverables (migration 0016 + extended Template A mint) land in **P1** alongside the registry plumbing.

### **P0 — Plan-mode gate / draft + ExitPlanMode**

- **Goal**: validate readings, surface forks, draft this plan, drive user-lock on F1–F6 + §3.D re-interpretations + iter-2 F5.B.subfork; then `ExitPlanMode`.
- **Deliverables**: this plan file (cycle-folder layout under `<slug>-<8hex>/`); ADR-0055 stub (Proposed); CHK8S-D-10 ledger entry pre-drafted in §3.B.
- **Tests**: NA (plan-mode).
- **Concept-alignment check**: §2 table populated.
- **phi-core leverage check**: §3 table populated.
- **User-facing doc updates**: NONE (P3 owns the doc deliverables).
- **Confidence target**: 100%.
- **Pause discipline**: ExitPlanMode is the gate. Auto-approval blocked iter-1 by 6 forks + §3.D contradictions; iter-2 by F5.B.subfork lock + new migration.

### **P1 — Trait-shaped `SessionLiveStreamRegistry` + recorder broadcast tap + Template A `Observe` extension + migration 0016 (~3–4 hours)**

- **Goal**: ship the `SessionLiveStreamRegistry` trait + `InProcessSessionLiveStreamRegistry` impl in `server/src/state.rs`; ship the `tokio::broadcast::Sender<AgentEvent>` field on `BabyPhiSessionRecorder`; extend `domain/src/templates/a.rs::fire_grant_on_lead_assignment` to mint `[Read, Inspect, List, Observe]`; ship migration `0016_template_a_session_object_grant_add_observe.surql`.
- **Deliverables**:
  1. `server/src/state.rs` — new trait `SessionLiveStreamRegistry: Send + Sync` with methods `insert(SessionId, broadcast::Sender<AgentEvent>)`, `get(&SessionId) -> Option<broadcast::Sender<AgentEvent>>` (clones Sender for `subscribe()`), `remove(&SessionId)`, `len()`. New struct `InProcessSessionLiveStreamRegistry { inner: DashMap<SessionId, broadcast::Sender<AgentEvent>> }`. New helper `pub fn new_session_live_stream_registry() -> Arc<dyn SessionLiveStreamRegistry>`. New `AppState` field: `pub session_live_stream_registry: Arc<dyn SessionLiveStreamRegistry>`.
  2. `domain/src/session_recorder.rs` — new field `BabyPhiSessionRecorder::broadcast_tx: Option<tokio::sync::broadcast::Sender<phi_core::types::event::AgentEvent>>`. New method `pub fn with_broadcast(self, tx: broadcast::Sender<AgentEvent>) -> Self` (builder pattern). Inside `on_phi_core_event`, after the `rec.on_event(event)` call but BEFORE the lifecycle-event-emit narrow scope: `if let Some(tx) = &self.broadcast_tx { let _ = tx.send(event.clone()); }`.
  3. `server/src/platform/sessions/launch.rs` — `spawn_agent_task` pre-allocates `let (tx, _rx) = broadcast::channel::<AgentEvent>(64); registry_live.insert(ctx.session_id, tx.clone());` BEFORE the `tokio::spawn`; the recorder is constructed with `.with_broadcast(tx)`. After the join + finalise + registry.remove, ALSO `registry_live.remove(&ctx.session_id);`.
  4. **(iter-2 NEW)** `domain/src/templates/a.rs:139–143` (project_grant action vec) + `:175–179` (session_grant action vec) — extend BOTH from `[Read, Inspect, List]` to `[Read, Inspect, List, Observe]`. Update assertion sites in `templates::a::tests` + `events/listeners.rs:312` + `tests/template_a_firing_props.rs` per Artifact C cascade analysis (predicted ≤ 12 assertion edits).
  5. **(iter-2 NEW)** `store/migrations/0016_template_a_session_object_grant_add_observe.surql` — forward-only idempotent SurQL migration that walks live Template A grants (provenance: `descends_from(AR with kinds CONTAINS '#template:a')`; `revoked_at = NONE`) and pushes `"observe"` onto the `action` array if not already present. Mirrors 0015's body shape (FOR-loop over filtered grants → UPDATE with array::push). ~80 lines.
  6. **(iter-2 NEW)** `store/tests/migration_0016_test.rs` (NEW) — deterministic-seed test: seed a Template A grant with `["read", "inspect", "list"]`; run migration 0016; assert action becomes `["read", "inspect", "list", "observe"]`; re-run migration; assert action UNCHANGED (idempotency). Mirrors `migration_0015_test.rs`'s shape.
- **Tests**: 4 unit tests in `state.rs` (round-trip insert/get/remove/len through trait object) + 2 unit tests in `session_recorder.rs` (`broadcast_tap_emits_events_when_sender_attached`, `broadcast_tap_is_zero_overhead_when_sender_absent`) + **(iter-2 NEW)** 1 prop-test extension in `domain/tests/template_a_firing_props.rs` (`mints_observe_action_in_both_grants`) + 2 migration tests in `store/tests/migration_0016_test.rs` (forward apply + idempotency). Existing tests `wrap_persists_session_loop_turn_and_emits_lifecycle_events` + `wrap_emits_session_started_exactly_once_even_on_re_entry` + `scope_default_is_ephemeral` + Template A property tests MUST still pass (after assertion updates per Artifact C).
- **Concept-alignment check**: §2 row 5 (Authority Templates) transitions from honored-at-launch to honored-at-SSE-extended.
- **phi-core leverage check**: §3 row 1 transitioned: `phi_core::types::event::AgentEvent` import-count up 1.
- **User-facing doc updates**: NONE (P3 owns it).
- **Confidence target**: ≥ 96% (iter-2 confidence slightly lower than iter-1's 97% due to migration 0016 risk).
- **Pause discipline**: PAUSE if `BabyPhiSessionRecorder` ends up with a NON-Optional `broadcast::Sender` field. PAUSE if Template A action-cascade is > 12 assertion sites (per Artifact C). PAUSE if migration 0016 ledger interaction conflicts with 0015's compound-tx semantics.

### **P2 — SSE handler + 403 gate + audit event + observe-manifest builder (~3–4 hours)**

- **Goal**: ship `GET /api/v0/sessions/:id/events` axum handler + the platform-layer fn at `server/src/platform/sessions/events.rs`; **(iter-2 CHANGED)** ship `domain/src/permissions/builders/session_observe.rs` (NEW) — sibling builder `build_session_observe_manifest(project_id) -> Manifest` returning `[Observe]` on `session_object`. SSE handler imports + uses this builder (NOT `build_session_launch_manifest`).
- **Deliverables**:
  1. **(iter-2 NEW)** `domain/src/permissions/builders/session_observe.rs` (NEW file, ~50 lines) — sibling to `session_launch.rs`. Body: `pub fn build_session_observe_manifest(_project_id: ProjectId) -> Manifest { Manifest { actions: vec![Action::Observe], resource: vec!["session_object".to_string()], transitive: vec![], constraints: vec![], constraint_requirements: HashMap::new(), kinds: vec![] } }`. Module docstring cites concept-doc 03 line 22 + line 44 (Observability universal). 4 unit tests (mirrors `session_launch.rs` test shape: actions == `[Observe]`; resource == `session_object`; no constraints; non-empty).
  2. **(iter-2 NEW)** `domain/src/permissions/builders/mod.rs` — `pub use session_observe::build_session_observe_manifest;` (mirrors line 16).
  3. **(iter-2 NEW)** `domain/src/permissions/mod.rs` — `pub use builders::session_observe::build_session_observe_manifest;` (mirrors line 56).
  4. `server/Cargo.toml` — new dep `tokio-stream = { version = "0.1", features = ["sync"] }`. (axum 0.7 SSE support is already feature-flagged.)
  5. `domain/src/audit/events/m5_2/session_live_stream.rs` (NEW) — builder `pub fn session_live_stream_denied(actor, session_id, agent_id, project_id, org_id, step, reason_kind, error_summary, now) -> AuditEvent` mirroring `session_launch_denied` shape. Audit-class: **Alerted**.
  6. `server/src/platform/sessions/events.rs` (NEW) — `pub async fn open_live_stream(...)`. Steps:
     - Step A: fetch session row → 404 SESSION_NOT_FOUND if absent.
     - Step B: gather actor's grants — same projection as launch.
     - Step C: build manifest via **(iter-2 CHANGED)** `build_session_observe_manifest(session.owning_project)` (NOT `build_session_launch_manifest`). Build CheckContext + call `check(&ctx, &manifest, &NoopMetrics)`.
     - Step D: on `Decision::Denied`, emit `platform.session.live_stream_denied` BEFORE returning Err. Return `SessionError::PermissionCheckFailed { step, reason }`. NO consent gate.
     - Step E: on `Decision::Allowed`, fetch the broadcast Sender from `live_stream_registry.get(&session_id)`. If `None`, return `SessionError::SessionLiveStreamUnavailable(session_id)` (HTTP 410 GONE).
     - Step F: subscribe via `let rx = sender.subscribe()`. Build a `tokio_stream::wrappers::BroadcastStream::new(rx)` mapped through a `match` arm. Return the stream.
  7. `server/src/handlers/sessions_events.rs` (NEW) — axum handler. Returns `Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30)).text(": keep-alive"))`.
  8. `server/src/router.rs` — add `.route("/sessions/:id/events", get(handlers::sessions_events::events))`.
  9. `server/src/main.rs` (or boot helpers) — instantiate `new_session_live_stream_registry()` at boot.
  10. `server/src/platform/sessions/mod.rs` — extend `SessionError` + `wire_code_for` + `http_status_for` with `SessionLiveStreamUnavailable(SessionId)` → `SESSION_LIVE_STREAM_UNAVAILABLE` / 410 GONE.
- **Tests**: 6 platform-layer/handler tests + 4 builder-unit tests for `session_observe.rs` = **10 tests** for P2 (iter-1 had 6; iter-2 adds 4 builder tests):
  - **(iter-2 CHANGED)** `live_stream_returns_403_when_agent_holds_no_observe_grant` — actor with `[Read, Inspect, List]`-only grant denied at SSE (legacy-grant simulation BEFORE migration 0016 applies).
  - `live_stream_returns_200_when_agent_holds_observe_grant` (post-migration legitimate path; actor with `[Read, Inspect, List, Observe]`).
  - `live_stream_returns_403_emits_live_stream_denied_audit`.
  - `live_stream_returns_410_when_session_already_finalised`.
  - `live_stream_returns_404_when_session_not_found`.
  - `live_stream_subscriber_receives_AgentEvent_after_launch`.
  - `live_stream_emits_lagged_event_then_closes_when_consumer_falls_behind`.
  - **(iter-2 NEW)** 4 unit tests in `session_observe.rs`: `builder_produces_action_set_with_observe_only`, `builder_resource_is_session_object_composite`, `builder_carries_no_constraints_or_kind_filters`, `builder_produces_non_empty_manifest`.
- **Concept-alignment check**: §2 table rows 1, 2, 3, 6, 7 transition from honored-at-launch / partially-honored to honored-at-SSE.
- **phi-core leverage check**: §3 row 1 — `+1` import in `events.rs`.
- **User-facing doc updates**: NONE (P3 owns it).
- **Confidence target**: ≥ 97%.
- **Pause discipline**: PAUSE if `BroadcastStream` import requires a `tokio-stream` major-version mismatch. PAUSE if the SSE response body size exceeds axum 0.7's default body limit on a long-lived stream.

### **P3 — CLI default-tail flip + new `--no-tail` flag + doc-tier updates (~2 hours)**

- **Goal**: flip CLI's deferred-tail print into a real SSE consumer; ship the user-facing doc tier updates from §3.C; flip drift D7.1 to `remediated`.
- **Deliverables**:
  1. `cli/Cargo.toml` — add `eventsource-stream = "0.2"` OR hand-roll a 30-line parser (planner recommends hand-roll).
  2. `cli/src/commands/session.rs` — replace the `(live tail deferred to M7 — …)` print at line 228 with a real SSE consumer; new flag `#[arg(long = "no-tail")] no_tail: bool`.
  3. `cli/src/commands/session.rs` — module-level docstring updates: remove the "deferred to M7" sentence.
  4. `m5_2/architecture/session-live-events.md` (NEW) per §3.C row 1. **(iter-2 CHANGED)** body cites `Action::Observe` rationale + sibling-builder design + concept-doc 03 line 44.
  5. `m5_2/architecture/session-launch-permission-gate.md` — add a paragraph noting the SIBLING builder design (launch uses `[Read, Inspect, List]`; SSE uses `[Observe]`; both reuse the same `check()` engine surface).
  6. `m5_2/operations/session-live-events-operations.md` (NEW) per §3.C row 3 + migration 0016 runbook entry.
  7. `m5_2/user-guide/session-live-events-walkthrough.md` (NEW) per §3.C row 4 + 403-debug runbook citing the `Action::Observe` requirement.
  8. `m5_2/user-guide/cli-reference-mN.md` — `phi session launch` flag table updates.
  9. `_concept-audit-matrix.md` — extend the `concepts/permissions/05-memory-sessions.md` block with the new row + extend `concepts/permissions/03-action-vocabulary.md` block with a row noting Observability's first runtime exercise on `session_object`. Status: `honored` (unchanged for 03; new row for 05).
  10. `m5_1/drifts/D7.1.md` — append lifecycle entry.
  11. **(iter-2 NEW)** `m5_2/operations/migrations.md` (verify existence; create if absent) — append migration 0016's purpose + idempotency note + run-order vs 0015.
- **Tests**: 3 CLI / acceptance tests:
  - `cli_session_launch_default_tails_events_via_sse` (golden-stream test against a mock server fixture).
  - `cli_session_launch_no_tail_skips_stream_consumption` (flag plumbing).
  - `acceptance_sessions_m5p4::launch_then_tail_round_trip` (full HTTP-level acceptance: launch session → subscribe to `/events` → observe at least 1 `AgentEvent` → SSE stream ends cleanly when session finalises).
- **Concept-alignment check**: §2 final pass — every row's target Status achieved.
- **phi-core leverage check**: net `+1` import confirmed.
- **User-facing doc updates**: ALL §3.C rows shipped (6 files iter-2: 4 NEW + 2 UPDATED, +1 migrations.md vs iter-1's 5 files).
- **Confidence target**: ≥ 99%.
- **Pause discipline**: PAUSE if the gate-2 doc-sync sweep finds a `m*/architecture|operations|user-guide/*.md` file with `live tail deferred to M7` or `D7.1` wording NOT in §3.C — patch in-cycle per CH-15 retro Row 1 widened sweep.

### **P-seal — Chunk seal + paperwork**

- ADR-0055 flipped Proposed → Accepted.
- D7.1 status `discovered` → `remediated`.
- CHK8S-D-10 ledger entry committed.
- `_concept-audit-matrix.md` rows added per P3 deliverable 9.
- All §10 close-criteria measures recorded.

---

## §8 — Tests summary

**Expected total test count at chunk close** — **1462 baseline + ~22 new = ~1484** (iter-2 increased from iter-1's ~15 due to 4 new builder unit tests + 2 migration-0016 tests + 1 prop-test extension). Apply asymmetric ×1.0–×1.20 buffer band per CH-11 + CH-12 retros: lower bound `1484` / upper bound `1484 + 0.20×22 ≈ 1488`. **Plan §8 chunk-close prediction band: [1484, 1488]**. Outside this band → AskUserQuestion.

Layer breakdown:
- **Unit** (P1 + P2 builder + audit-event builder): 4 (state.rs trait round-trip) + 2 (recorder broadcast tap) + 1 (audit-event builder shape test for `live_stream_denied`) + **4 (session_observe.rs builder iter-2)** + **1 (Template A prop-test extension iter-2)** + **2 (migration 0016 forward + idempotency iter-2)** = **14 unit**.
- **Integration / platform** (P2 events.rs + handler): 7 = **7 integration** (iter-2 added 1 vs iter-1's 6 — `live_stream_returns_403_when_agent_holds_no_observe_grant` separated from the post-migration legitimate path).
- **Acceptance** (P3 round-trip + CLI): 2 (CLI default-tail + CLI no-tail) + 1 (acceptance launch+tail round-trip) = **3 acceptance**.

Sum: 14 + 7 + 3 = **24**, lower-band 24, upper-band ~29 (24 × 1.20). Predicted close: **+22–24**.

(Plan-author note: refining to a tighter band — the 2 unit tests for the recorder broadcast tap may merge into 1 if the parameterised-fixture pattern fits; band calibrated around 22.)

Named test files:
- `modules/crates/server/src/state.rs` — extend existing `mod tests` with `in_process_session_live_stream_registry_round_trips_through_trait_object`.
- `modules/crates/domain/src/session_recorder.rs` — extend existing `mod tests` with broadcast-tap tests.
- `modules/crates/domain/src/permissions/builders/session_observe.rs` (NEW iter-2) — 4 builder unit tests.
- `modules/crates/domain/tests/template_a_firing_props.rs` — extend with `mints_observe_action_in_both_grants` (iter-2).
- `modules/crates/store/tests/migration_0016_test.rs` (NEW iter-2) — 2 migration tests.
- `modules/crates/server/tests/sse_live_stream_test.rs` (NEW) — the 7 integration tests.
- `modules/crates/cli/tests/session_live_tail_test.rs` (NEW or extend existing) — CLI tests.
- `modules/crates/server/tests/acceptance_sessions_m5p4.rs` (existing) — append `launch_then_tail_round_trip`.

Named expected-still-green tests:
- `state::tests::handler_count_is_five_at_m5` (CH-22 invariant — broadcast registry is NOT a bus listener).
- `state::tests::in_process_session_registry_round_trips_through_trait_object` (CH-K8S-PREP §D33.1 invariant).
- `permissions::builders::session_launch::tests::*` (4 tests — CH-15 manifest invariants UNCHANGED at `[Read, Inspect, List]`).
- `permissions::action::tests::canonical_contains_thirty_four_variants` + `canonical_is_all_minus_wildcard` (CH-04 closed-set invariants — UNCHANGED).
- `acceptance_sessions_m5p4::launch_denies_with_403_when_agent_holds_no_session_grants` (CH-15 hard-deny invariant — verifies launch path UNCHANGED at `[Read, Inspect, List]`).
- `session_recorder::tests::wrap_persists_session_loop_turn_and_emits_lifecycle_events` (M5/P3 invariant).
- `templates::a::tests::*` (15 tests — most unchanged; ≤ 6 assertion-shape edits per Artifact C).

---

## §9 — Pre-chunk gate

**Reading list (mandatory):**
1. `concepts/permissions/05-memory-sessions.md` (full read of §"Standard Actions Applied to Sessions" + §"Default Grants Issued to Every Agent" + §"Authority Templates").
2. `concepts/permissions/03-action-vocabulary.md` (the closed 34-verb set + applicability matrix). **(iter-2 EMPHASIS)** lines 22 (Observability row) + 44 (universal applicability) — these are the verifications iter-1 missed.
3. `concepts/permissions/04-manifest-and-resolution.md` (manifest semantics + the All-Steps-Hard-Gate invariant).
4. `concepts/permissions/README.md` (entry invariants — required when permissions/01–09 cited).
5. `concepts/phi-core-mapping.md` §"Sessions / events" row.
6. `m5_1/drifts/D7.1.md` (full).
7. `m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md` (CH-15 manifest builder + hard-deny precedent).
8. `m5_2/decisions/0048-per-session-consent-gating.md` (consent-launch-only invariant).
9. `m5_2/decisions/0033-k8s-prep-refactors.md` (trait-shaping precedent for `SessionLiveStreamRegistry` + migration-runner ledger pattern for 0016).
10. `m5/decisions/0029-session-persistence-and-recorder-wrap.md` (recorder-wrap design).
11. `m5_2/decisions/0032-real-agent-loop-with-mock-provider.md` (CH-02).
12. `forward-scope/remaining-scope-post-m5-p7-22035b2a.md` §"Live SSE", lines 161–167 + §7 Q&A.
13. `baby-phi/CLAUDE.md` §"phi-core Leverage" (rules 1–5).
14. `m7b/architecture/k8s-microservices-readiness.md` + `m7b/architecture/deferred-from-ch-k8s-prep.md` (current ledger; CHK8S-D-10 will append).
15. **Conditional (CH-11 retro)**: `server::platform::sessions::launch.rs` body + `server::platform::sessions::preview.rs` body (manifest-shape preconditions).
16. **(iter-2 NEW)** `domain/src/permissions/action.rs` lines 73 (Observability variant), 282 (CANONICAL array containing Observe), 322 (`as_str() => "observe"`) — the source-citations for F5.B's pre-existing canonical status.
17. **(iter-2 NEW)** `store/migrations/0015_template_a_session_object_grant.surql` (the migration shape 0016 mirrors).
18. **(iter-2 NEW)** `domain/src/templates/a.rs` lines 128–193 (`fire_grant_on_lead_assignment` body — the cascade source for F5.B.subfork(a)).
19. **Conditional (per `chunk-planner` v8 tag-write Repository contract reading-list rule)**: NOT triggered — CH-17 ships zero tag-write methods.

**Carry-forward invariants** (verified green at chunk open):
- `cargo test --workspace` test count = **1462** (verified at CH-15 cycle-audit gate-4 / 2026-05-08; current git HEAD `f42393a`).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `scripts/check-spec-drift.sh` green.
- `modules/` diff against the chunk-open git HEAD is empty (no preload edits).
- `Action::CANONICAL.len() == 34` (closed-set invariant — CH-17 preserves; F5.B.locked path uses an existing variant).
- `state::tests::handler_count_is_five_at_m5` green (CH-22 invariant).

**Pending decisions carried into this chunk:**
- F1–F6 user-locked at iter-1 (F5 → F5.B at iter-2 with corrected fact-base).
- F5.B.subfork(a) — pending user-lock at iter-2 plan approval.
- D7.1 transition `discovered → remediated` at chunk seal (P-seal).

---

## §10 — Close criteria

**4 aspects (each pass / fail):**

- **Code aspect**: All P1+P2+P3 deliverables shipped; `cargo test --workspace` passes (target ~1484–1488 tests); `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` green; `cargo fmt --all -- --check` green. **(iter-2 NEW)** migration 0016 applies cleanly + idempotently; `fire_grant_on_lead_assignment` mints 4-action grants.
- **Docs aspect**: Two scopes:
  - *Governance tier*: ADR-0055 flipped Proposed → Accepted (iter-2 includes D55.5 + D55.9 sub-decisions); D7.1 status flipped to `remediated`; `_concept-audit-matrix.md` extended with the new live-stream row + concept-03-line-44-Observability-on-session-object reference; CHK8S-D-10 ledger entry added.
  - *User-facing tier*: All 6 §3.C rows shipped (iter-2 added migrations.md row).
- **phi-core leverage aspect**: `+1` import delta (workspace baseline 49 → 50); all forbidden-duplication greps return 0; `scripts/check-phi-core-reuse.sh` green.
- **Concept alignment aspect**: every §2 row's target-status at chunk-close achieved; none remain `contradicted`. `Action::CANONICAL.len() == 34` invariant preserved (Observe was already there).

**2 confidence % (each with named numerator/denominator):**

- **Implementation confidence target**: `≥ 9/11 claims-honored / claims-in-scope = 82%` (iter-2 lower bound). Initial draft target: **11/11 = 100%**.
  - Claims-in-scope (11) — iter-2 expanded: (1) D7.1 closed; (2) **`[Observe]` on `session_object` for SSE gate** (iter-2 changed from iter-1's `[Read, Inspect, List]`); (3) closed 34-verb invariant preserved; (4) per-session trait-shaped registry shipped; (5) broadcast tap in recorder (Option-shaped); (6) 30s keep-alive; (7) `Lagged` → typed SSE error event; (8) CLI default-tail flipped; (9) audit-event `live_stream_denied` parallel to `launch_denied`; (10) CHK8S-D-10 ledger entry filed; (11) **(iter-2 NEW)** migration 0016 + extended `fire_grant_on_lead_assignment` mint + assertion-cascade closed.
- **Documentation confidence target**: `≥ 8/9 = 89%` (iter-2 lower bound; added migrations.md row).
  - Doc pages touched: ADR-0055 + D7.1 lifecycle + concept-audit-matrix rows (×2) + 6 §3.C user-facing tier files + CHK8S-D-10 ledger.

**Composite = min(impl%, doc%, code-aspect, phi-core-leverage-aspect, concept-alignment-aspect).** Target composite: **≥ 99%** at full pass; ≥ 82% as the lower-bound trip-wire. Below 82% → close blocked.

**P4 chunk-seal paperwork checklist**: every modified verified-header reflects the body diff exactly; `_concept-audit-matrix.md` rows Status copy-pasted letter-for-letter from §2 target; new tag-write Repository contract reading-list conditional NA (CH-17 ships zero tag-write methods).

---

## §11 — Post-chunk independent audit plan

**Agent count**: **1 auditor** — iter-2 confirms Small envelope (4 phases including seal; per audit-envelope-size skill: ≤ 4 phases → 1 agent). Migration 0016's complexity is well-bounded (mirrors 0015 precedent).

**Audit aspects (a–d) — iter-2 adjusted for F5.B**:
- (a) **Code correctness** — diff review + grep verification of (1) `BabyPhiSessionRecorder.broadcast_tx` Optional shape; (2) **(iter-2 CHANGED)** SSE handler uses `build_session_observe_manifest` (NOT `build_session_launch_manifest`); manifest is `[Observe]` on `session_object`; (3) `fire_grant_on_lead_assignment` mints `[Read, Inspect, List, Observe]`; (4) migration 0016 idempotent via deterministic-seed test.
- (b) **Docs fidelity vs concept docs** — verify `[Observe]` shape on session_object matches concept-doc 03 line 22 + line 44 (Observability universal). Verify ADR-0055 §D55.5 cites the pre-existing canonical status of `Action::Observe`.
- (c) **Concept alignment** — verify §3.D re-interpretations are documented in ADR-0055 §D55.6; verify closed 34-verb invariant unchanged (`Action::CANONICAL.len() == 34`); **(iter-2 NEW)** verify Observe is NOT a new variant (`grep -nE '^\\s*Observe,$' modules/crates/domain/src/permissions/action.rs` returns 1 hit at line 73 — pre-existing).
- (d) **phi-core leverage** — verify net `+1` import; verify zero forbidden duplications (no `RecorderEvent` type, no new `Action::*` variants).

**Audit agent prompt (single agent, scoped):**

> *You are auditing CH-17 — Live SSE tail endpoint, cycle hex `40c4d759`, iter-2 (F5 user-locked-divergent at F5.B; F5.B.subfork(a) user-locked). Plan: `baby-phi/docs/specs/plan/build/ch-17-live-sse-tail-endpoint-40c4d759/plan.md`. ADR: `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md`. Drift: `baby-phi/docs/specs/v0/implementation/m5_1/drifts/D7.1.md`. Migration: `baby-phi/modules/crates/store/migrations/0016_template_a_session_object_grant_add_observe.surql`.*
>
> *Audit aspects (a)–(d) below. Spawn fresh subagent context — do NOT reuse the implementer's notes.*
>
> *(a) **Code correctness.** (1) Verify `BabyPhiSessionRecorder.broadcast_tx: Option<broadcast::Sender<AgentEvent>>` exists at `domain/src/session_recorder.rs`; the publish call is inside `on_phi_core_event` after `rec.on_event(event)`; the field is None-by-default. (2) Verify `server/src/platform/sessions/events.rs::open_live_stream` calls `build_session_observe_manifest(session.owning_project)` (NOT `build_session_launch_manifest`); the manifest is `[Observe]` on `session_object` (single-action manifest, no `Read`/`Inspect`/`List`). (3) Verify `fire_grant_on_lead_assignment` at `domain/src/templates/a.rs:128` mints `[Read, Inspect, List, Observe]` for both `project_grant` (line 139–143) and `session_grant` (line 175–179). (4) Verify migration 0016 idempotency via `cargo test -p store migration_0016` — expect 2 PASS. (5) Verify the 403 path emits `platform.session.live_stream_denied` BEFORE returning Err. (6) Verify the SSE handler returns `Sse::new(stream).keep_alive(...interval(30s))`. (7) Verify `tokio-stream` is added to `server/Cargo.toml`. (8) Run `cargo test -p server --test sse_live_stream_test` — expect 7 PASS.*
>
> *(b) **Docs fidelity vs concept docs.** Read `concepts/permissions/03-action-vocabulary.md` lines 22 + 44. Verify ADR-0055 §D55.5 cites these exact line numbers as the rationale for F5.B. Verify `Action::Observe` is at `domain/src/permissions/action.rs:73` (variant), `:282` (CANONICAL array), `:322` (`as_str() => "observe"`) — pre-existing per CH-04. Verify the §3.D re-interpretation rationale in ADR-0055 §D55.6 explicitly addresses the missing "session lifecycle — live events" concept-doc section + the absent `RecorderEvent` type.*
>
> *(c) **Concept alignment.** Run `cargo test -p domain permissions::action::tests::canonical_contains_thirty_four_variants 2>&1 | tail -5` — expect 1 PASS (Action::CANONICAL.len() == 34 preserved). Verify `_concept-audit-matrix.md` block `permissions/05` has the new "Live session-event stream" row with Status `honored` (letter-for-letter) and Covering-drift `D7.1 ✓`; verify `permissions/03` block has the new Observability-on-session_object row.*
>
> *(d) **phi-core leverage.** Run `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect ≥ 50. Verify net `+1` is `phi_core::types::event::AgentEvent` in `server/src/platform/sessions/events.rs`. Run `git -C /root/projects/phi/baby-phi grep -nE '^pub (struct|enum|type) RecorderEvent' modules/crates/` — expect 0 hits.*
>
> *Report format: §A summary verdict (PASS/PARTIAL/FAIL); §B per-aspect findings (a–d each with verdict); §C any drift discovered; §D K8s-axis re-verification (axes A1+A2+A4+A6 — A4 lit by migration 0016); §E counter-examples; §F final verdict.*
>
> *Hard limits: ≤ 600 words. Cite file:line for every claim. Do NOT execute `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` or the 4 `bash scripts/check-*.sh` guards — mark these as `NOT-EXECUTED-IN-AUDIT` (sub-agent sandbox-blocked); orchestrator closes them at gate 4.*

**Audit pass criteria:**
- Any new drift discovered → its own drift file BEFORE chunk seals.
- Any audit-flagged concept contradiction → fixed in-chunk, renegotiated with user (ADR), or new drift file with explicit future-chunk assignment.
- Chunk seal blocked until audit returns clean on (a)–(d) + all audit-discovered drifts explicitly scoped.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (orchestrator gate-4 MUST-RUN list — sub-agent auditors mark these NOT-EXECUTED-IN-AUDIT).
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (-j 4 cap per feedback_cargo_jobs_cap.md).
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace

# 3. Chunk-specific tests (iter-2 expanded).
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test sse_live_stream_test
/root/rust-env/cargo/bin/cargo test -j 4 -p domain session_recorder::tests::broadcast_tap
/root/rust-env/cargo/bin/cargo test -j 4 -p domain permissions::builders::session_observe::tests
/root/rust-env/cargo/bin/cargo test -j 4 -p domain --test template_a_firing_props mints_observe_action_in_both_grants
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migration_0016_test
/root/rust-env/cargo/bin/cargo test -j 4 -p server state::tests::in_process_session_live_stream_registry_round_trips_through_trait_object
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_sessions_m5p4 launch_then_tail_round_trip

# 4. phi-core leverage greps (canonical no-`::` form).
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 50.

grep -rn "use phi_core::types::event::AgentEvent" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: ≥ 4.

# 5. Forbidden-duplication greps.
git -C /root/projects/phi/baby-phi grep -nE '^pub (struct|enum|type) RecorderEvent' modules/crates/ | wc -l
# Expect: 0.

# 6. Closed 34-verb invariant (iter-2 — Observe is canonical, NOT in forbidden list).
git -C /root/projects/phi/baby-phi grep -nE 'Action::(SessionRead|ReadEvents|Tail|Stream)' modules/crates/domain/src/permissions/builders/ | wc -l
# Expect: 0 (closed 34-verb invariant; SSE manifest uses Observe which IS canonical).

/root/rust-env/cargo/bin/cargo test -j 4 -p domain permissions::action::tests::canonical_contains_thirty_four_variants
# Expect: 1 PASS.

# 7. F5.B verification — Action::Observe is canonical (pre-existing).
grep -nE '^\s*Observe,$' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs
# Expect: ≥ 1 hit at line 73 (variant declaration, pre-existing per CH-04).

grep -nE 'Action::Observe' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs | wc -l
# Expect: ≥ 5 hits (pre-existing per CH-04; lines 160, 241, 282, 322, 351).

# 8. F5.B SSE manifest builder — sibling builder shipped.
grep -nE 'pub fn build_session_observe_manifest' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/builders/session_observe.rs
# Expect: 1 hit.

grep -nE 'Action::Observe' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/builders/session_observe.rs | wc -l
# Expect: ≥ 1 (the manifest body).

# 9. F5.B.subfork(a) — Template A mint extended to 4-action.
grep -nE 'Action::Read.*Action::Inspect.*Action::List.*Action::Observe' /root/projects/phi/baby-phi/modules/crates/domain/src/templates/a.rs | wc -l
# Expect: ≥ 2 (project_grant + session_grant action vecs).

# 10. F5.B.subfork(a) — migration 0016 shipped.
ls /root/projects/phi/baby-phi/modules/crates/store/migrations/0016_template_a_session_object_grant_add_observe.surql
# Expect: file exists.

# 11. Drift D7.1 closure regression.
git -C /root/projects/phi/baby-phi grep -nE 'live tail deferred to M7|deferred to M7' modules/crates/cli/src/commands/session.rs | wc -l
# Expect: 0.

# 12. K8s readiness (axes A1+A2+A6 single-pod-only; A4 lit by migration 0016).
grep -nE 'CHK8S-D-10' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md | wc -l
# Expect: ≥ 2 hits.

# 13. Drift-file status.
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count> + 1 (D7.1 transitions in this chunk).

# 14. Doc-sync sweep (gate-2 widened sweep — CH-15 retro Row 1).
grep -rE 'live tail deferred|deferred to M7|D7\.1|live tail not yet' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m*/architecture/*.md /root/projects/phi/baby-phi/docs/specs/v0/implementation/m*/operations/*.md /root/projects/phi/baby-phi/docs/specs/v0/implementation/m*/user-guide/*.md 2>/dev/null | wc -l
# Expect: 0 (or only references inside ADR-0055 / D7.1 / matrix Update Note as historical record).
```

---

## Plan-author closing notes

- **Pre-archive line-number re-verification** (chunk-planner v5): every cited file:line in this plan has been re-greppted at git HEAD `f42393a` immediately before write. Citations stand. iter-2 NEW citations: `action.rs:73,282,322` (Observe canonical); `templates/a.rs:128,139–143,175–179` (mint extension); migrations directory contents.
- **Chunk-planner version**: v8 (iter-2 applied per chunk-planner v3 re-spawn-on-user-locked-divergent-fork discipline + chunk-planner v8 §3.D detection rule with corrected fact-base).
- **Cycle hex**: `40c4d759`.
- **Test count baseline**: 1462 (verified at CH-15 cycle-audit gate-4, 2026-05-08, ignored=2).
- **K8s posture**: K8s-negative — 1 new blocker class (CHK8S-D-10). A4 lit by migration 0016 but covered by existing migration-runner mitigation.
- **Auto-approval verdict**: ESCALATE TO USER (F5.B.subfork lock + new migration 0016).
- **iter-2 deltas vs iter-1** (summary):
  - F5 lock flipped iter-1's F5.A → iter-2's F5.B (+ subfork).
  - §2: added 2 rows for concept-doc 03 lines 22 + 44 (Observe canonical).
  - §3: added Artifact C cascade analysis (`fire_grant_on_lead_assignment` 24 sites).
  - §3.B: A4 axis flipped to YES (migration 0016).
  - §3.D: contradiction 1 RETRACTED.
  - §5: ADR sub-decisions D55.5 + D55.9 added; D55.5 carries the F5.B fact-base correction.
  - §7 P1: added migration 0016 + Template A mint extension + assertion cascade as deliverables.
  - §7 P2: SSE handler uses `build_session_observe_manifest` (NEW sibling builder).
  - §8: test-count target up from ~15 to ~22.
  - §9: reading list +3 (action.rs lines, migration 0015, templates/a.rs).
  - §10: claims-in-scope expanded from 10 to 11.
  - §11: audit prompts updated for F5.B verification.
  - §12: 14 verification commands (iter-1 had 10).
