<!-- Last verified: 2026-05-08 by Claude Code (CH-15 plan-approval gate: F1.A / F2.A / F3.A / F4.C / F5.B user-locked at orchestrator gate-1 escalation — all align with planner recommendation; F3.A confirms forward-scope re-interpretation that the closed 34-verb action vocabulary takes precedence over the row's literal `session.start` / `session.tool_invoke` / `session.read_memory` wording (ADR-0054 §D54.2 + §D54.8 carry the rationale); F1.A migration 0015 (additive grant backfill on existing Template A grants) accepted as part of F1.A scope; chunk approved for P0 launch.) -->
<!-- Last verified: 2026-05-08 by Claude Code (chunk-planner v7 — fresh draft cycle hex c3f46f17) -->

# CH-15 — Real permission-check gate at session launch · Plan

**Cycle hex**: `c3f46f17`
**Forward-scope row**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 147-151.
**Severity / effort**: ⚠HIGH · ~2 days · M5 critical path.
**Test baseline at chunk-open**: 1431 passed / 0 failed / 2 ignored (`cargo test --workspace -j 4`, verified 2026-05-08).
**phi-core import baseline at chunk-open**: 49 `use phi_core` statements across 68 files; `check-phi-core-reuse.sh` green.
**Migration baseline at chunk-open**: 14 (post-CH-14 `0014_authority_chain.surql`). Next free slot: **`0015`**.
**ADR number reservation**: highest existing ADR = 0053 (CH-14). **CH-15 reserves ADR-0054**. (Forward-scope row says ADR-0033 but 0033 is taken by CH-K8S-PREP — reservation collision documented in plan §5.)

---

## Forks for orchestrator

The chunk has **5 user-locked-decision forks**. Each names options + planner recommendation; the orchestrator surfaces these via `AskUserQuestion` at plan approval.

### F1 — Template A grant-extension strategy (HIGHEST priority)

The chunk requires every existing CEO/lead with a Template A grant to be authorised against `session_object` reaches at session launch. Three options:

- **(F1.A) Extend Template A pure-fn grant issuance + ship a one-time backfill migration `0015`** for legacy Template A grants. New grants minted post-CH-15 carry a second resource URI (`session-class:<project_id>` or a parallel grant on `session_object` instances tagged `project:<id>`); the migration walks every active `Grant` whose `descends_from = <Template-A adoption AR>` and inserts the paired session-scoped grant.
- **(F1.B) Implicit-allow-via-Template-A-presence** — at launch.rs Step 3 / 3.5, before running the engine, check whether the agent holds ANY Template A grant whose `descends_from` is the project's HAS_LEAD adoption AR; if yes, short-circuit-allow without engine consultation. No data migration; no concept-doc-aligned grant shape change.
- **(F1.C) Hybrid** — extend the pure-fn for new grants (no migration), document a documented **grace window** in launch.rs that implicit-allows lead-agents whose Template A grant predates CH-15 (queryable via `issued_at < cutoff`), close the grace window in M6 once a manual or migration-based backfill runs.

**Planner recommendation: F1.A.**
**Reasoning**:
- Concept doc 04 §"Permission Check (Runtime Reconciliation)" §"Worked Example" is unambiguous — the agent must hold a grant covering the manifest's `(action, fundamental)` reaches. F1.B circumvents that ("this principal's grant on a *different* resource is treated as covering this resource") and is a structural concept-contradiction that opens a new HIGH drift exactly as we are closing D4.1.
- F1.C ships drift in the form of a defined grace window with no automatic close — it reproduces D4.1's "advisory clause" pattern at lower volume. Concept fidelity-over-speed locked principle says no.
- F1.A respects the Authority Chain (concept doc 04 §"The Authority Chain" lines 511-547) — the new session-class grant `descends_from` the same Template A adoption AR, and CH-14's `walk_provenance_chain` traversal handles the additional row transparently.
- F1.A's K8s cost (one additive migration) is well-trodden — CH-12 / CH-14 each shipped one; the migration runner story is unchanged (CHK8S-D-05 leader-election is the only K8s blocker, untouched here).

**User must lock F1 before plan approval.** The choice cascades into F2/F4/F5.

### F2 — Manifest builder location

Forward-scope row literal: *"`domain::permissions::builders::build_session_launch_manifest(project_id, tools)`"*.

- **(F2.A) NEW `domain::permissions::builders` module** at `modules/crates/domain/src/permissions/builders/mod.rs` + `builders/session_launch.rs`. Hosts `build_session_launch_manifest`. Future builders (per-action invocation manifests for tools, memory-recall manifests, etc.) co-locate.
- **(F2.B) Sibling fn in `permissions::manifest::session_launch`** — colocated with the existing `Manifest::from_node` constructor.

**Planner recommendation: F2.A.**
**Reasoning**: matches the forward-scope literal verbatim (Q5/Q7 Q&A locked module names at forward-scope time); a `builders/` namespace gives a natural home for the additional builders M6+ will introduce (per-tool invocation manifests, runtime-replay manifests, etc.); keeps `manifest/mod.rs` focused on the engine input contract type rather than its construction sites. Cost: +1 module + +1 sub-file. Auto-approval-criteria-clean.

### F3 — Action enum variant placement (DESCOPED — see below)

Forward-scope row says *"actions `session.start` / `session.tool_invoke` / `session.read_memory`"*. **Critical re-reading**: the v0 permission vocabulary (`concepts/permissions/03-action-vocabulary.md` §"Standard Action Vocabulary") is a **closed 34-verb set**. `session.start` / `session.tool_invoke` / `session.read_memory` are NOT new vocabulary — they're (action, resource) pairs naming launch-time reaches.

The closest mapping per concept docs 03 + 07:
- `session.start` = `Action::Invoke` on `session_object` (the launching agent invokes a session lifecycle).
- `session.tool_invoke` = `Action::Invoke` on the tool's manifest at runtime — NOT a launch-time reach. (Per concept doc 04 §"Permission Check" — runtime tool invocations gate at the per-tool manifest, not the launch manifest.) **Out of scope for launch-time gating.**
- `session.read_memory` = `Action::Recall` on `memory_object` — also NOT a launch-time reach. (Per concept doc 07 §"recall_memory" — gated at the per-tool manifest at runtime.) **Out of scope for launch-time gating.**

**Resolution**: CH-15's launch manifest gates **only** the `session.start` semantics → `actions: [Action::Read, Action::Inspect, Action::List]` on `resource: ["session_object"]`. This matches Template A's `[Read, Inspect, List]` issuance shape exactly (per `templates/a.rs:114-118`), giving F1.A a clean grant-shape match. The "tool_invoke" + "read_memory" gates ship at M6+ when per-tool manifest gating lands at runtime (forward-scope CH-21 memory-extraction-listener-body + CH-22 agent-catalog-listener-body).

- **(F3.A) Reuse existing `[Read, Inspect, List]` action set on `session_object`** — matches Template A grant shape; no Action enum change required.
- **(F3.B) Add 3 new Action variants `session.start` / `session.tool_invoke` / `session.read_memory`** — would expand the closed 34-verb vocabulary to 37 verbs; concept doc 03 update required; widens the action × fundamental matrix from 9×10 to 9×11 (+27 cells); breaks `Action::CANONICAL.len() == 34` invariant test + `concept-doc § "Standard Action Vocabulary" line count` invariants. Concept-contradiction.

**Planner recommendation: F3.A.**
**Reasoning**: F3.B contradicts concept doc 03's locked closed-set principle. The forward-scope row's `session.start / session.tool_invoke / session.read_memory` wording is a **scoping gloss** describing the three logical reaches the launch handler eventually wants to gate, not a literal Action variant set. CH-15 honours `session.start` at the launch boundary; the other two are runtime-tool-invocation reaches that ship via the per-tool manifest path, not the launch manifest. **Critical for plan approval — this is a forward-scope re-interpretation that needs orchestrator/user confirmation before code lands.**

### F4 — Hard-deny flip semantics for legacy sessions

Conditional on F1.A/F1.B/F1.C choice. Question is: how do we handle the moment between deploy + backfill?

- **(F4.A) Hard-deny flips immediately at deploy, no grace window** — pre-CH-15 sessions launching mid-deploy fail. (If F1.A: legacy Template A holders fail until backfill migration runs; running migration in same deploy is idempotent + simultaneous, so the gap is sub-millisecond.)
- **(F4.B) Implicit-allow grace window via F1.C** — pre-CH-15 grants implicit-allow until grace closes.
- **(F4.C) One-time Template A backfill migration `0015` runs BEFORE the launch.rs flip** — orchestrated via deploy ordering: migration `0015` populates session-scoped grants atomically; same-deploy code change activates hard-deny only after migration runs. This is the standard SurrealDB embedded-mode behaviour per ADR-0033 §D33.2 (migrations run on `SurrealStore::open_*` before the server accepts requests).

**Planner recommendation: F4.C** (assumes F1.A locked).
**Reasoning**: F4.C is structurally the same as F1.A — at server boot, migration `0015` runs to completion before launch.rs serves any request, so the "gap" is zero. F4.A would only fail if a deployer ran the new code without the migration — that's prevented by the standard migration-runner discipline. F4.B's grace window is concept-aspirational drift.

### F5 — Engine deny audit-event emission path

When launch.rs flips to hard-deny, every step-1-to-6 deny needs an audit trail. Two options:

- **(F5.A) Reuse existing engine-deny audit pattern** — the existing `permission_check_decision` field on `LaunchReceipt` (line 144) already carries the `Decision`; the audit-emit at Step 6 (line 362-372) already emits `platform.session.started` on success. For deny, emit the symmetric `platform.session.launch_denied` audit event with the failed step + reason.
- **(F5.B) Add launch-specific `session.launch_denied` audit event** in `domain/src/audit/events/m5_2/session_launch.rs` per the m5_2 audit-events doc shape. Same on-the-wire shape as F5.A; differs only in code organisation.

**Planner recommendation: F5.B** (matches the m5_2 audit-events module organisation; CH-13 / CH-14 each added per-feature audit-event modules; F5.A would land the new event in m5/templates.rs which is wrong-milestone).

**Audit event shape proposal (canonical_bytes contributors marked `*`)**:
```
platform.session.launch_denied {
  session_id*: SessionId,        // pre-allocated at Step 3.5 even on deny path
  agent_id*: AgentId,
  project_id*: ProjectId,
  org_id*: OrgId,
  failed_step*: u8,              // 0..6 per FailedStep::as_metric_label
  reason_kind*: String,           // DeniedReason variant tag (snake_case)
  reason_detail: serde_json::Value,  // OPTIONAL — non-canonical (operator data)
  emitted_at*: DateTime<Utc>,
}
```
Audit class: `Alerted` (concept doc 04 invariant 5: "audit trail on every outcome"; deny is alert-worthy per concept doc 07 §"audit_class composition" — failed permission checks default to `alerted`).

---

## §1 — Context & principle

**Why this chunk**: Drift D4.1 (HIGH; cascading-upstream, security-enforcement) — at M5/P4 the launch handler advisory-logs every step-1-to-6 Permission-Check denial and proceeds to spawn the agent task regardless. Concept doc 04 §"Permission Check (Runtime Reconciliation)" + §"Key Invariants" line 310 ("there is no 'default allow'") is contradicted: any agent without grants on `session_object` can launch a session today. Closing D4.1 is the linchpoint for security-enforcement at M5; subsequent milestones build on "launch-succeeded means permission-gated" and accumulate erosion if D4.1 is deferred further.

**Quality-over-speed restatement**: *Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently.* CH-15 application: where the forward-scope literal text (`session.start / session.tool_invoke / session.read_memory` as actions) disagrees with concept doc 03's closed action vocabulary, **the concept doc wins**, the forward-scope wording is re-interpreted as scoping-gloss (see F3), and the chunk gates the launch boundary using canonical Action verbs that concept doc 03 already permits.

**Forward-scope reference**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 row 13 (CH-15 entry, lines 147-151).

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/permissions/04-manifest-and-resolution.md` | §"Formal Algorithm (Pseudocode)" line 209-305 — `Decision = Allowed / Denied / Pending`; Step 3 missing reach → `Denied`; Step 6 missing consent → `Pending` | All steps gate; no advisory layer | partially-honored (engine honors it; launch.rs bypasses steps 1–6) | honored (launch.rs hard-denies on every Denied) |
| `concepts/permissions/04-manifest-and-resolution.md` | §"Key Invariants" line 309-314 — invariants 1–4: every reach must match a grant; ceilings clamp, never add; scope resolution deterministic; consent evaluated last | Engine honors invariants but launch.rs ignores invariant-1's "no default allow" | partially-honored | honored at launch boundary |
| `concepts/permissions/04-manifest-and-resolution.md` | §"Permission Resolution Hierarchy" lines 333-389 — org → project → agent ceiling; scope resolution most-specific-first | Already honored by engine + CH-07 multi-scope cascade | honored (no change) | honored (no change) |
| `concepts/permissions/04-manifest-and-resolution.md` | §"Two Shapes: Tool Authority Manifest vs Grant" line 22-32 — manifest is descriptive of tool requirements; the synthetic launch manifest is a **launch-context** manifest expressing "to launch this session, the launching agent needs `[Read, Inspect, List]` on `session_object`" | Synthetic manifest exists at preview.rs:88 + launch.rs:217; advisory at launch | partially-honored (shape exists; gating absent) | honored (shape gates at launch via builder) |
| `concepts/permissions/03-action-vocabulary.md` | §"Standard Action Vocabulary" line 31-82 — closed 34-verb set + 1 wildcard; `Action::CANONICAL.len() == 34` invariant | honored (Action enum unchanged across CH-15) | honored | honored (F3.A locks no new variants) |
| `concepts/permissions/03-action-vocabulary.md` | §"Action × Fundamental Applicability Matrix" lines 27-37 — 9×10 cells; Read/Inspect/List apply to DataObject + Tag (the constituents of `session_object`) | honored | honored | honored (no matrix changes) |
| `concepts/permissions/07-templates-and-tools.md` | §"Template A — Project Lead Authority" — when `HAS_LEAD` fires, Agent X gets `[read, inspect, list]` on every session tagged `project:P` | partially-honored — Template A issues `[Read, Inspect, List]` on `project:<id>` (concept-doc-aligned shape via `templates/a.rs:114-118`) but the on-the-wire grant covers `project:<id>` resource only, not `session_object` instances tagged `project:P`. The concept doc is more granular than the implementation. CH-15 closes this gap (F1.A) | partially-honored | honored — Template A issuance extended to mint **two** grants per HAS_LEAD: (1) the existing `project:<id>` grant; (2) a NEW grant on `session_object` filtered by `tags contains project:<id>` (matches concept-doc verbatim wording). |
| `concepts/permissions/07-templates-and-tools.md` | §"Templates Are Pre-Authorized Allocations" + §"Authority Chain" — every Grant traces to an adoption AR | honored (CH-14 ADR-0053) | honored | honored — both new grants `descends_from` the same Template A adoption AR; `walk_provenance_chain` impl unchanged |
| `concepts/permissions/07-templates-and-tools.md` | §"Standard Organization Template" `session_object_grants` block lines 191-197 — org-tier baseline session grant `[read, list, inspect]` selector `tags contains org:{this_org}` | concept-aspirational (no org-tier session_object_grants issuance today) | concept-aspirational (forward-defensive note in §10; out-of-scope for CH-15 which targets the lead-grant path only) | concept-aspirational (M6+) |
| `concepts/permissions/README.md` | §entry-invariants source — pulled in because permissions/04 + permissions/07 are touched | honored | honored | honored |
| `concepts/phi-core-mapping.md` | (none — Permission Check is baby-phi-native; phi-core has no Permission concept) | N/A | N/A | N/A — explicitly cited |

**Mid-flight pause discipline**: any concept contradiction not in this table → `AskUserQuestion` before phase close.

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::agent_loop` | Runtime call site at `launch.rs:489` | direct-reuse | unchanged |
| `phi_core::types::context::AgentContext` | Built per launch via `super::provider::build_agent_context` | direct-reuse | unchanged |
| `phi_core::session::model::Session` / `LoopRecord` | Wrapped (M5/P4 nested-inner pattern) | wrap | unchanged |
| `phi_core::types::event::AgentEvent` | Streamed through mpsc into recorder | direct-reuse | unchanged |
| `phi_core::provider::model::ModelConfig` | Cloned into `AgentLoopConfig` per launch | direct-reuse | unchanged |

**Expected import-count delta at chunk close**: **0 new phi-core imports.** CH-15 introduces builder + audit-event modules in the `domain` crate; neither overlaps any phi-core type. Baseline 49 → target 49.

**Positive close-audit greps** (canonical pattern names per CH-08 retro v6):

```bash
# (a) phi-core baseline preserved.
grep -rn "use phi_core\|pub use phi_core" /root/projects/phi/baby-phi/modules/crates/ --include="*.rs" | wc -l
# Expect: 49 (unchanged from chunk-open baseline)
```

```bash
# (b) The new builder module exists with the exact forward-scope-literal name.
git grep -nE 'pub fn build_session_launch_manifest\b' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/builders/
# Expect: ≥ 1 hit (the function definition).
```

```bash
# (c) launch.rs no longer carries the advisory-only string.
git grep -nE 'advisory at M5; not blocking|advisory at M5\b' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# Expect: 0 hits at chunk close (was 2 hits at chunk-open: lines 198, 244, 545, 715 — see grep output below).
```

```bash
# (d) The hard-deny flip is in place — every Denied returns SessionError.
git grep -nE 'PermissionCheckFailed \{|fn map_decision_to_session_error|hard_deny_decision' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# Expect: ≥ 2 hits (the existing Step-0 site + new Step-1..6 hard-deny site).
```

**Forbidden-duplication greps**:

```bash
# (e) Action enum stays at 34 canonical + 1 wildcard.
git grep -nE 'Action::CANONICAL\.len\(\) == 34|Action::ALL\.len\(\) == 35' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs
# Expect: ≥ 2 hits (the existing test invariants — F3.A locks no new variants).
```

```bash
# (f) No new struct named SessionLaunchManifest (would duplicate Manifest).
grep -rn "^struct SessionLaunchManifest\|^pub struct SessionLaunchManifest" /root/projects/phi/baby-phi/modules/crates/
# Expect: 0 hits.
```

```bash
# (g) No back-door advisory-allow.
git grep -nE 'fn advisory_allow|implicit_allow_pre_ch15|grace_window|legacy_template_a_bypass' /root/projects/phi/baby-phi/modules/crates/
# Expect: 0 hits (F1.B and F1.C grace-window patterns rejected).
```

### §3 cascade-artifact discipline (CH-13 v4 + v6 + v7 caveats)

#### Artifact A — Template A `fire_grant_on_lead_assignment` callsite cascade (F1.A)

**(a) Invocation**:
```bash
git grep -nE 'fire_grant_on_lead_assignment\(|FireArgs \{$' /root/projects/phi/baby-phi/modules/crates/
```

**(b) Raw match count at plan-draft time**: 8 hits.

**(c) Per-file breakdown** (verified 2026-05-08):
- `modules/crates/domain/src/templates/a.rs:103` — pure-fn definition (1 — the production fn).
- `modules/crates/domain/src/templates/a.rs:179, 188, 200, 208, 217, 224, 233` — 7 unit-test invocations.
- `modules/crates/domain/src/events/listeners.rs:42, 305` — listener import + production call site.

**(d) Predicted edit sites for CH-15 (F1.A)**:
- **1 site** — extend `FireArgs` struct: add new field if the listener needs to pass project's session-object resource info OR change the return type from `Grant` to `Vec<Grant>` (CascadeResult-style typed-multi-value precedent per CH-14 retro v7). Recommended: change the return type to `Vec<Grant>` so the listener iterates `repo.create_grant(&g).await?` over all returned grants. Predicted ~4 listener-side cascade edits.
- **~3 listener-side edits** at `events/listeners.rs:305-330` — listener body iterates the `Vec<Grant>` return + persists each + emits one audit event per grant.
- **~7 unit-test sites** in `templates/a.rs` tests — the existing tests assert grant shape directly (`g.holder`, `g.action`, `g.resource.uri`, etc.). Convert to `let grants = ...; assert_eq!(grants.len(), 2); let g = &grants[0]; ...` (8 asserts → ~16 asserts; add 8 for the new second grant's invariants).
- **~1 audit-event-builder edit** at `audit/events/m4/templates.rs` — the listener emits `template.a.fire` per Grant; if 2 grants are minted per fire, emit 2 audit events (or extend the audit-event shape to carry `Vec<GrantId>`).

**Total cascade band**: ~12 deliberate edits across ~4 files. **Pause discipline trigger**: PAUSE if actual edit-site count > 18 (1.5× upper bound). The bias here is unclear (struct-field cascades historically bias HIGH; CascadeResult return-type changes historically bias LOW because the test fixture set is small) — auditors should run `git grep -nE 'fire_grant_on_lead_assignment\b' modules/crates/` post-implementation as the canonical re-count.

#### Artifact B — `permissions::builders::build_session_launch_manifest` callsite cascade

**(a) Invocation**:
```bash
git grep -nE 'build_session_launch_manifest\b|domain::permissions::builders' /root/projects/phi/baby-phi/modules/crates/
```

**(b) Raw match count at plan-draft time**: 0 hits (module doesn't exist yet — confirmed via `grep -lnE` returning empty).

**(c) Per-file breakdown after CH-15**:
- `modules/crates/domain/src/permissions/builders/mod.rs` — new file; exports `build_session_launch_manifest`.
- `modules/crates/domain/src/permissions/builders/session_launch.rs` — new file; pure-fn definition.
- `modules/crates/domain/src/permissions/mod.rs` — `pub mod builders; pub use builders::session_launch::build_session_launch_manifest;` (2 lines).
- `modules/crates/server/src/platform/sessions/launch.rs:594` (today's manifest construction site) — replace inline `Manifest { ... }` literal with `build_session_launch_manifest(input.project_id)` call. **1 site.**
- `modules/crates/server/src/platform/sessions/preview.rs:88` — replace inline `Manifest { ... }` with `build_session_launch_manifest(input.project_id)`. **1 site.** (Critical: keeping preview + launch on identical synthetic manifest shape preserves the M5 invariant that preview's Decision matches launch's Decision when grants are stable.)

**Total cascade band**: 2 callsites + 4 new files. **Pause discipline trigger**: PAUSE if a 3rd callsite emerges (means a new caller needs a session-launch manifest — out of CH-15 scope).

#### Artifact C — Decision/FailedStep `match` cascade (additive — no callsite delta predicted)

**(a) Invocation**:
```bash
git grep -nE 'match.*Decision\b' /root/projects/phi/baby-phi/modules/crates/
```

**(b) Raw match count**: 9 hits across 3 files (all engine-internal or test-internal).

**(c) Per-file breakdown** (verified 2026-05-08):
- `modules/crates/domain/src/permissions/engine.rs` — engine-internal match arms.
- `modules/crates/server/src/platform/sessions/launch.rs:226` (today's `if let Decision::Denied { ... } = ` advisory-arm) — converts to `match preview.decision { Decision::Allowed { .. } => ..., Decision::Pending { .. } => ..., Decision::Denied { failed_step, reason } => return Err(SessionError::PermissionCheckFailed { ... }) }`. **1 callsite edit (existing).**

**Per CH-12 retro v3 additive-enum cascade discipline**: CH-15 does NOT add a new `Decision` or `FailedStep` variant; it only changes one match arm's body. **0 new exhaustive-match callsites cascade.**

**(d) Predicted edit sites**: 1 (the launch.rs:226 advisory→hard-deny flip). PAUSE if any other Decision-match site needs editing.

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | `gate_session_launch_consent` already populates an in-process `HashSet<AuthRequestId>` (line 604, `template_gated`); CH-15's new launch-engine call reuses the same shape | no | none — no new pod-local state |
| **A2** | New IPC channel | none — CH-15 stays synchronous (engine call + repo call) | no | none |
| **A3** | New pod-local resource | none | no | none |
| **A4** | Migration runner / first-apply race | F1.A ships migration `0015` (Template A backfill — see F1 reasoning). Single additive UPDATE — same pattern as `0014_authority_chain.surql`. Standard CHK8S-D-05 leader-election applies; not aggravated by additive UPDATEs | no | cross-ref CHK8S-D-05; no new entry |
| **A5** | Trait-shape requirement | `domain::permissions::builders::build_session_launch_manifest` is a pure free-fn, no trait shape needed. `Repository::create_grant` (existing trait method) covers the Vec<Grant> persistence path via repeated calls | no | none |
| **A6** | Cross-pod state sharing | none — Permission Check is read-only against persisted `Grant` rows; cross-pod sharing happens via SurrealDB which already covers this | no | none |
| **A7** | Audit hash-chain symmetry | F5.B adds a NEW audit-event variant `platform.session.launch_denied`. The new variant flows through the existing single-writer `AuditEmitter` trait at `domain::audit::AuditEmitter::emit` — no new writer, no chain bypass | no | none |

**ADR-0033 conforming-criteria check**:
- D33.1 (`SessionRegistry` trait) — CH-15 calls `registry.insert` only on the success path (post-Step-6 hard-deny); deny path NEVER inserts a registry entry, preserving cap accounting. Trait-object dispatch unchanged.
- D33.2 (`SurrealStore::open_remote`) — migration `0015` adds an additive UPDATE; runs on `open_*` before request-serving begins; no change to the open-mode story.
- D33.3 (SIGTERM graceful shutdown) — CH-15 does not spawn new `tokio::spawn` tasks (the deny path RETURNS before `spawn_agent_task`); no SIGTERM-handler change needed.
- D33.4 (`EventBus.shutdown` + `drain`) — CH-15 emits one new audit event on the deny path; flows through `Arc<dyn AuditEmitter>` (already wired); no `EventBus` change.

**Conclusion paragraph**: **K8s-neutral.** No new blockers introduced; no new ledger entry needed. The chunk reuses CH-11's per-session ambient-context plumbing + CH-14's persisted authority-chain plumbing without expanding either.

## §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5/architecture/session-launch.md` | YES — `§"9-step launch flow"` line 40 advisory-mention + Step 3 description need rewording from "advisory" to "hard-deny on Decision::Denied" | update in-chunk (Phase P3) |
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/session-launch-permission-gate.md` (NEW) | YES — net-new architecture doc for the manifest-builder + audit-event flow + Template A grant-extension cascade | create in-chunk (Phase P3) |
| **Operations** | `docs/specs/v0/implementation/m5/operations/session-launch-operations.md` | YES — `§"Error-code reference"` line 28 needs new row for `403 PERMISSION_CHECK_FAILED_AT_STEP_<N>` (was: only Step 0 documented; CH-15 adds steps 1–6); §"D4.1 advisory mention" lines 64-65 deleted | update in-chunk (Phase P3) |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/session-launch-permission-gate-operations.md` (NEW) | YES — audit-event dictionary entry for `platform.session.launch_denied`; metrics histograms for Step-N denial counts | create in-chunk (Phase P3) |
| **User-guide** | `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` | YES — line 40-65 mentions "Step 0 only blocking step at M5"; needs deletion + replacement with "all steps gate; missing Template A grant returns 403" | update in-chunk (Phase P3) |
| **User-guide** | `docs/specs/v0/implementation/m5/user-guide/cli-reference-m5.md` | NO change to CLI surface | no-change |
| **User-guide** | `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` | YES if exists — add troubleshooting entry for "agent can launch preview but cannot launch session: missing Template A grant" | update in-chunk (Phase P3) |

**Doc-sync sweep at gate-2** (CH-14 retro Row 3): the orchestrator's gate-2 doc-sync sweep should grep for `D4.1` + `advisory at M5` + `not blocking` + `Step 0 only blocking` across `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md`. Any matches surviving past Phase P3 = TRIVIAL gate-2 patches before auditor dispatch.

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D4.1` | [`drifts/D4.1.md`](../../v0/implementation/m5_1/drifts/D4.1.md) | HIGH | discovered → classified → **remediated** | Primary; ADR-0054 ratifies. Lifecycle move + status update + remediation timestamp + linked ADR row added at chunk seal. |

**No new drifts predicted** at chunk-open. Mid-flight discoveries trigger `AskUserQuestion` per template §6 mid-flight discipline.

**Adjacent drift watch list (chunk-open carry-forward verification)**:
- `D4.2` (real agent_loop) — already remediated via CH-02; no action.
- `D-new-30` (Standard Org Template config concept-aspirational) — out-of-scope; org-tier session_object_grants ship at M6+.

## §5 — ADRs drafted

**ADR-0054 — Session-launch manifest, hard-deny flip, and Template A session-object grant extension** (Proposed → Accepted at Phase P4 seal).

- **Number**: 0054 (current high = 0053 from CH-14, verified via `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE '[0-9]{4}' | sort -n | tail -3` — returns 0051, 0052, 0053).
- **Title**: *Session-launch manifest, hard-deny flip, and Template A session-object grant extension.*
- **Drafted at phase**: P0 (scaffolding).
- **Decision summary**: Close D4.1 by (a) lifting the synthetic launch manifest into a typed `domain::permissions::builders::build_session_launch_manifest` pure-fn; (b) flipping launch.rs Steps 1–6 from advisory-log to hard-deny via `SessionError::PermissionCheckFailed` mapped to 403; (c) extending Template A's `fire_grant_on_lead_assignment` to mint a paired `session_object` grant per HAS_LEAD edge; (d) shipping migration `0015` to backfill the paired grant for legacy Template A holders; (e) emitting a new `platform.session.launch_denied` audit event on every step-1-to-6 deny.
- **Sub-decision IDs**: D54.1 (builder location — F2.A), D54.2 (Action vocabulary preserved — F3.A re-interpretation), D54.3 (Template A double-grant shape — F1.A), D54.4 (migration `0015` backfill — F4.C), D54.5 (audit-event placement — F5.B), D54.6 (hard-deny error-mapping — Step N → `PERMISSION_CHECK_FAILED_AT_STEP_<N>` 403), D54.7 (preview-launch parity — both call the builder), D54.8 (forward-scope row literal re-interpretation note for the planning ledger).
- **ADR file path**: `docs/specs/v0/implementation/m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md`.

**ADR-body checklist (per CH-13 + CH-14 + per-chunk-template v6/v7)**:

1. **§"Forks" header**:
   - F1: user-locked at plan approval to `<F1.A | F1.B | F1.C>` (planner recommendation: F1.A).
   - F2: locked to `F2.A` (planner recommendation).
   - F3: locked to `F3.A` (forward-scope re-interpretation; planner recommendation; user MUST confirm).
   - F4: locked to `<F4.A | F4.B | F4.C>` (planner recommendation: F4.C).
   - F5: locked to `<F5.A | F5.B>` (planner recommendation: F5.B).

2. **§"Cross-references"** (all 4 categories, milestone-prefixed per CH-08 retro v6):
   - **(a) Originating concept doc + section + line range**:
     - `concepts/permissions/04-manifest-and-resolution.md` §"Permission Check (Runtime Reconciliation)" lines 166-202 + §"Formal Algorithm (Pseudocode)" lines 209-330 + §"Key Invariants" lines 309-314.
     - `concepts/permissions/07-templates-and-tools.md` §"Template A — Project Lead Authority" lines 36-40 + §"Templates Are Pre-Authorized Allocations" lines 14-72.
     - `concepts/permissions/03-action-vocabulary.md` §"Standard Action Vocabulary" lines 31-82 (re-interpretation rationale for F3).
   - **(b) Closed drifts**:
     - `m5_1/drifts/D4.1.md` (primary, HIGH, transitioned to remediated).
   - **(c) Prior ADRs cited as precedent (milestone-prefixed)**:
     - `m5_2/decisions/0033-k8s-prep-refactors.md` §D33.1/§D33.2 (SessionRegistry trait + open_*).
     - `m5_2/decisions/0044-publish-time-manifest-validator.md` §D44.A-D44.D (manifest validator precedent).
     - `m5_2/decisions/0048-per-session-consent-gating.md` §D48.3 / §D48.5 / §D48.7 (per-session ambient-context plumbing reused by hard-deny path).
     - `m5_2/decisions/0050-audit-class-composition-strictest-wins.md` §D50.5 / §D50.6 (audit-class composition precedent for the new launch_denied event).
     - `m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md` §D53.3 / §D53.5 (provenance chain reused — both new grants `descends_from` the Template A adoption AR).
     - `m4/decisions/0028-domain-event-bus.md` (Template A listener wiring precedent for the dual-grant emission).
     - `m5/decisions/0029-session-persistence-and-recorder-wrap.md` (launch handler architecture).
     - `m5/decisions/0031-session-cancellation-and-concurrency.md` (launch handler concurrency model — informs ordering of hard-deny BEFORE registry.insert).
   - **(d) Forward-scope row**: `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 147-151 (CH-15 row) — including the *forward-scope-literal-re-interpretation* note (§D54.8 below).

3. **Pre-existing-behaviour preservation note** (CH-14 retro Row 10):
   - Document explicitly the **pre-CH-15 advisory-log behaviour**: "At M5/P4, `launch.rs:198-246` advisory-logs every step-1-to-6 Permission-Check denial via `tracing::info!(..., 'sessions::launch: Permission Check denied (advisory at M5; not blocking)')` and proceeds to `spawn_agent_task`. Only Step 0 (Catalogue) gates, returning 403 `PERMISSION_CHECK_FAILED_AT_STEP_0`. CH-15 (this ADR) flips Step 1–6 deny paths to hard-deny via the same `SessionError::PermissionCheckFailed { step, reason }` shape."
   - Document the **new behaviour**: "Post-CH-15, every `Decision::Denied { failed_step, reason }` from the launch-time engine call returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>` where `<N>` is `failed_step.as_metric_label()` (0..6). The advisory `tracing::info!` block at line 244 is removed; the corresponding 'advisory at M5; not blocking' string disappears from the codebase. Step 6 (Consent) deny remains routed through `gate_session_launch_consent` per CH-11 + ADR-0048 — only Steps 1–5 widen from advisory to hard-deny because Step 6 was already enforced (CH-11)."
   - Document the **Template A pre-existing behaviour preserved**: "Pre-CH-15 Template A grants minted `[Read, Inspect, List]` on `project:<id>` only (`templates/a.rs:114-122`). CH-15 extends `fire_grant_on_lead_assignment` to mint a SECOND grant on `session_object` filtered by `tags contains project:<id>`. The first grant is preserved verbatim — CH-15 does NOT remove the project-resource grant, only ADD a session-resource grant alongside. Migration `0015` walks every `Grant` whose `descends_from` matches a Template A adoption AR and inserts the paired session-resource grant; idempotent on re-run via `INSERT INTO ... WHERE NOT EXISTS`."

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| CH-02 | `phi_core::agent_loop` runtime-call wired at `launch.rs:489`; `MockProvider` deterministic | `grep -nE 'phi_core::agent_loop\b' modules/crates/server/src/platform/sessions/launch.rs` — expect ≥ 1 hit (line 489); `cargo test -p server --test acceptance_sessions_m5p4 -j 4` — expect green |
| CH-04 | typed `Action` enum with closed 34-verb vocabulary; `Action::CANONICAL.len() == 34` | `cargo test -p domain action --lib -j 4` — expect green; run the `all_contains_thirty_five_variants` + `canonical_contains_thirty_four_variants` named tests |
| CH-06 | selector grammar (`grammar.pest`) parses `tags contains project:<uuid>` predicates | `cargo test -p domain selector -j 4` — expect green; the new `session_object` grant uses `tags contains project:<id>` selector at fire time |
| CH-09 / CH-10 | Consent node lifecycle (Requested → Acknowledged → Revoked) | `cargo test -p domain consents -j 4` — expect green |
| CH-11 | per-session ambient-context populated in `CheckContext.current_session`; consent-gating at Step 6 already lands as hard-deny via `gate_session_launch_consent` | `git grep -nE 'gate_session_launch_consent\b' modules/crates/server/src/platform/sessions/launch.rs` — expect ≥ 2 hits (declaration + call site); `cargo test -p server --test acceptance_per_session_consent_gating -j 4` — expect green |
| CH-13 | audit-class composition `compose_audit_class_with_source` available; the new `launch_denied` audit event composes its `audit_class` via `Alerted` constant (no listener-side composition because launch is at the boundary, not a template fire) | `git grep -nE 'compose_audit_class_with_source' modules/crates/domain/src/permissions/audit_composition.rs` — expect ≥ 1 hit (the public fn); reused by listeners only — launch.rs uses `AuditClass::Alerted` directly |
| CH-14 | `Repository::walk_provenance_chain` available; the dual-grant pattern preserves provenance traversal because both grants `descends_from` the same AR | `git grep -nE 'fn walk_provenance_chain\b' modules/crates/domain/src/repository.rs` — expect ≥ 1 hit (the trait method) |

**Re-verification recipe** (run at chunk open + chunk seal):
```bash
cd /root/projects/phi/baby-phi
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace -j 4
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh
```

## §7 — Phases within the chunk

### Phase P0 — Scaffolding (ADR draft + module skeleton + plan-archive index entry)

**Goal**: land the ADR-0054 stub at `Proposed`, create the empty `permissions/builders/` module, register the cycle-folder in `_cycle-index.md`. No behaviour change.

**Deliverables**:
1. `docs/specs/v0/implementation/m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md` — ADR stub with §"Forks" / §"Cross-references" / §"Pre-existing behaviour" headers + Status `Proposed`.
2. `modules/crates/domain/src/permissions/builders/mod.rs` — empty module + `pub mod session_launch;`.
3. `modules/crates/domain/src/permissions/builders/session_launch.rs` — `pub fn build_session_launch_manifest(project_id: ProjectId) -> Manifest` stub returning today's launch.rs literal (preserves preview/launch wiring; no behaviour change yet).
4. `modules/crates/domain/src/permissions/mod.rs` — add `pub mod builders; pub use builders::session_launch::build_session_launch_manifest;` (2 lines).
5. `docs/specs/plan/build/_cycle-index.md` — add CH-15 row (status `in-flight`).

**Tests**: 1 new unit test `build_session_launch_manifest_matches_today_inline_literal` at `permissions/builders/session_launch.rs` mod-tests — round-trips field-by-field equality with a hand-built `Manifest { ... }` literal that reproduces `launch.rs:594-601` verbatim. (PartialEq pre-check note: `Manifest` does NOT derive `PartialEq` — see §3 below for derive plan; if test runs at P0 before derive lands, use field-by-field assertions OR derive `PartialEq` in P0.)

**Concept-alignment check**: row 4 (manifest is descriptive — shape preserved at P0).

**phi-core leverage check**: 0 new imports.

**User-facing doc updates**: none in P0.

**Confidence target**: 100% (scaffolding only).

**Pause discipline**: none — pure additive.

### Phase P1 — Template A double-grant + migration `0015` backfill (F1.A path)

**Goal**: extend `fire_grant_on_lead_assignment` to mint paired grants; ship migration `0015` to backfill legacy Template A holders. Listeners + audit-events updated. **Pre-CH-15 launch.rs behaviour preserved (still advisory) — the hard-deny flip lands at P2.**

**Deliverables**:
1. `modules/crates/domain/src/templates/a.rs` — change return type from `Grant` to `Vec<Grant>` (CascadeResult-style typed-multi-value precedent); both grants `descends_from` the same `adoption_auth_request_id`. Pure-fn discipline preserved (no I/O).
2. `modules/crates/domain/src/events/listeners.rs` — `TemplateAFireListener::handle` body iterates `let grants = fire_grant_on_lead_assignment(args); for g in grants { repo.create_grant(&g).await?; ... }` + emits one `template.a.fire` audit event per grant.
3. `modules/crates/store/migrations/0015_template_a_session_object_grant.surql` — additive UPDATE: `INSERT INTO grant SELECT ... FROM grant WHERE descends_from IN (SELECT id FROM auth_request WHERE template_kind = "A") AND resource.uri STARTS_WITH "project:" AND id NOT IN (SELECT id FROM grant WHERE resource.uri = "session_object" AND ...)` (idempotent skeleton — exact SurrealQL crafted at P1).
4. `modules/crates/domain/src/audit/events/m4/templates.rs` — adjust the `template.a.fire` audit-event builder to accept either single `GrantId` or `Vec<GrantId>` (TBD at impl time — depends on whether the listener emits 1-event-per-grant or 1-event-with-list).
5. ADR-0054 §D54.3 + §D54.4 body filled.

**Tests**: ~10-12 new + ~7 amended.
- `templates/a.rs::tests::fire_grant_returns_two_grants_per_call` (NEW).
- `templates/a.rs::tests::fire_grant_first_grant_unchanged_from_pre_ch15_shape` (NEW — locks back-compat).
- `templates/a.rs::tests::fire_grant_second_grant_targets_session_object` (NEW).
- `templates/a.rs::tests::fire_grant_second_grant_selector_filters_by_project_tag` (NEW).
- `templates/a.rs::tests::fire_grant_both_grants_descend_from_same_ar` (NEW).
- `templates/a.rs::tests::fire_grant_both_grants_have_distinct_ids` (NEW).
- 7 existing single-grant unit tests amended to assert on `grants[0]` (~7 amendments).
- `events/listeners.rs::tests::template_a_listener_persists_two_grants_per_lead_assignment` (NEW).
- `events/listeners.rs::tests::template_a_listener_emits_two_audit_events` (NEW).
- 1-2 acceptance tests amending `acceptance_authority_templates.rs` to confirm the dual-grant shape lands per HAS_LEAD.
- 1 migration test (in-memory + Surreal both) — `acceptance_template_a_session_grant_backfill.rs` (NEW): seed a legacy single-grant fixture pre-migration, run migration `0015`, assert paired grant exists.

**Concept-alignment check**: row 7 (Template A grant shape) flips from partially-honored to honored.

**phi-core leverage check**: 0 new imports.

**User-facing doc updates**: none in P1 (audit-event-dictionary update lands in P3 with the new launch_denied event).

**Confidence target**: ≥ 97%.

**Pause discipline**:
- PAUSE if migration `0015` needs schema change (additive UPDATE only — no `ADD FIELD` should be needed; if Surreal complains, falls into K8s axis A4 territory and needs `AskUserQuestion`).
- PAUSE if `fire_grant_on_lead_assignment` cascade exceeds Artifact A's predicted band (> 18 edits — see §3 Artifact A).

### Phase P2 — Hard-deny flip in launch.rs + builder wiring

**Goal**: replace the inline synthetic-manifest literal at `launch.rs:594-601` + `preview.rs:88-95` with `build_session_launch_manifest(project_id)` calls; flip the advisory-log block at `launch.rs:198-246` to hard-deny via `SessionError::PermissionCheckFailed`.

**Deliverables**:
1. `modules/crates/server/src/platform/sessions/launch.rs` — replace the advisory `if let Decision::Denied { ... }` block with a `match preview.decision { ... }` that returns `Err(SessionError::PermissionCheckFailed { step: failed_step.as_metric_label().parse().unwrap_or(0), reason: format!("{reason:?}") })` for ANY failed step. Remove the `tracing::info!(... "advisory at M5; not blocking")` line. Update inline comment block (lines 198-216) to cite ADR-0054 + remove D4.1 references.
2. `modules/crates/server/src/platform/sessions/preview.rs` — replace inline `Manifest { ... }` literal with `build_session_launch_manifest(input.project_id)` call. Preview-launch parity preserved.
3. `modules/crates/server/src/platform/sessions/launch.rs` — also replace the inline `Manifest { ... }` literal at `gate_session_launch_consent` (lines 594-601) with `build_session_launch_manifest(input.project_id)`. (Rationale: launch + consent-gate run on identical synthetic manifest — divergence here would re-open D4.1's "advisory layer" pattern at the consent boundary.)
4. `modules/crates/domain/src/permissions/manifest/mod.rs` — add `#[derive(PartialEq, Eq)]` to `Manifest` (sub-types already derive these; see §3 PartialEq pre-check). Required for `assert_eq!` test patterns at the builder.
5. `modules/crates/domain/src/audit/events/m5_2/session_launch.rs` (NEW) — `platform.session.launch_denied` audit-event builder per F5.B shape (§"Forks" F5).
6. `modules/crates/domain/src/audit/events/m5_2/mod.rs` — add `pub mod session_launch;` + re-export.
7. `modules/crates/server/src/platform/sessions/launch.rs` — emit the `launch_denied` audit event on the deny path BEFORE returning `Err`. Threaded through the existing `Arc<dyn AuditEmitter>`.
8. `modules/crates/server/src/platform/sessions/mod.rs` — verify `SessionError::PermissionCheckFailed` already maps to 403 via `http_status_for` (line 162); no new error variant needed.
9. ADR-0054 §D54.1 / §D54.5 / §D54.6 / §D54.7 body filled.

**Tests**: ~12-15 new.
- `acceptance_sessions_m5p4.rs::test_launch_denies_with_403_when_agent_holds_no_session_grants` (NEW; covers the F1.A user-locked path — agent has no Template A grant → step 3 deny → 403 + `PERMISSION_CHECK_FAILED_AT_STEP_3` body).
- `acceptance_sessions_m5p4.rs::test_launch_denies_with_403_for_each_failed_step` (NEW; parameterised across steps 0..6 — Step 0 Catalogue + Step 1 Expansion + Step 2 Resolution + Step 3 Match + Step 4 Constraint + Step 5 Scope + Step 6 Consent).
- `acceptance_sessions_m5p4.rs::test_launch_succeeds_with_template_a_session_grant_after_p1_extension` (NEW; happy path — Template A holder launches successfully because P1 extended the grant).
- `acceptance_sessions_m5p4.rs::test_launch_emits_launch_denied_audit_event_on_403` (NEW).
- `acceptance_sessions_m5p4.rs::test_launch_does_not_register_session_on_403` (NEW; SessionRegistry size unchanged after deny — D33.1 invariant).
- `acceptance_sessions_m5p4.rs::test_preview_decision_matches_launch_decision_on_grants_stable` (NEW; preview ↔ launch parity assertion).
- `permissions/builders/session_launch.rs::tests::builder_produces_action_set_matching_template_a_grant_action_set` (NEW).
- `permissions/builders/session_launch.rs::tests::builder_resource_is_session_object_composite` (NEW).
- `permissions/manifest/mod.rs::tests::manifest_partialeq_derive_compiles_and_round_trips` (NEW; pinned via partial_eq derive).
- 2-3 audit-event-builder unit tests in `audit/events/m5_2/session_launch.rs` (NEW).

**Concept-alignment check**: rows 1 + 2 (Decision gating + invariant 1) flip from partially-honored to honored. Verified via the new acceptance test suite.

**phi-core leverage check**: 0 new imports; `check-phi-core-reuse.sh` re-runs at phase close.

**User-facing doc updates**: none in P2 (deferred to P3 in a single sweep so doc-sync sweep at gate-2 is one-pass).

**Confidence target**: ≥ 97%.

**Pause discipline**:
- PAUSE if any acceptance test from `acceptance_per_session_consent_gating.rs` breaks — would mean CH-11's per-session gating got tangled by the new hard-deny path. Cross-cycle invariant: Step 6 Consent path is unchanged by CH-15 (already hard-denying since CH-11 / ADR-0048).
- PAUSE if `Manifest::PartialEq` derive surfaces a sub-type that doesn't derive PartialEq (per §3 pre-check, all Manifest fields are `Vec<Action>`, `Vec<String>`, `HashMap<String, serde_json::Value>` — all `PartialEq` via std). If `serde_json::Value::PartialEq` causes ordering issues, fall back to per-field assertion in tests.
- PAUSE if migration `0015` from P1 needs to land BEFORE P2's hard-deny flip but the deploy ordering would catch this — coordinate with F4 user-lock.

### Phase P3 — User-facing doc updates (4 architecture/operations/user-guide files + 2 NEW m5_2 docs)

**Goal**: every §3.C row is closed in-chunk; advisory references purged from m5/architecture/session-launch.md + m5/operations/session-launch-operations.md + m5/user-guide/first-session-walkthrough.md; new m5_2 architecture + operations docs created.

**Deliverables**:
1. `docs/specs/v0/implementation/m5/architecture/session-launch.md` — Step 3 description rewrite (advisory → hard-deny); add cross-ref to ADR-0054.
2. `docs/specs/v0/implementation/m5_2/architecture/session-launch-permission-gate.md` (NEW) — full architecture doc covering builder + audit-event + Template A double-grant + migration `0015`. Last-verified header date.
3. `docs/specs/v0/implementation/m5/operations/session-launch-operations.md` — error-code reference table updated; D4.1 advisory mention deleted.
4. `docs/specs/v0/implementation/m5_2/operations/session-launch-permission-gate-operations.md` (NEW) — `launch_denied` audit-event dictionary entry + per-step Prometheus histogram counters.
5. `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` — line 40 "advisory at M5" replaced with "hard-deny at every step; missing Template A grant returns 403"; troubleshooting tip added.
6. `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` (touched if exists; create if missing — confirm at P3 open).
7. `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` — rows 1 + 2 of permissions/04 block (lines 175-185) Status flip from `partially-honored` to `**honored**` per CH-12 retro v3 letter-for-letter rule. Row at line 180 (the explicit D4.1 row) flips from `contradicted` to `**honored**` with Covering-drift cell now `D4.1 ✓`. Row 7 of permissions/07 block (Template A) flips from `partially-honored` to `**honored**` if the user-lock honors F1.A. Verified-header line 1 description matches the body diff exactly per CH-11 retro v1 P4 paperwork checklist.

**Tests**: 0 new tests — doc-only deliverables. CI guards (check-doc-links + check-ops-doc-headers) re-run.

**Concept-alignment check**: rows 1 + 2 + 7 of §2 — final-status flip recorded in concept-audit-matrix.

**phi-core leverage check**: N/A.

**User-facing doc updates**: ALL §3.C rows satisfied in this phase.

**Confidence target**: ≥ 99% (doc-only work; CI guards gate).

**Pause discipline**: none expected.

### Phase P4 — Chunk seal (drift transition + ADR Accepted + paperwork)

**Goal**: D4.1 → remediated; ADR-0054 → Accepted; concept-audit-matrix verified-header line 1 lifted to mention the closed rows.

**Deliverables**:
1. `docs/specs/v0/implementation/m5_1/drifts/D4.1.md` — lifecycle history append `2026-05-?? — remediated — closed via CH-15 / ADR-0054 §D54.1+§D54.5+§D54.6`; status `discovered` → `remediated`.
2. ADR-0054 status `Proposed` → `Accepted`.
3. Cycle index `_cycle-index.md` — status `in-flight` → `final-audit` → `closed`.
4. Verified-header line 1 of `_concept-audit-matrix.md` carries the canonical CH-15 paperwork sentence per CH-12 retro F-AUDB-1 letter-for-letter rule (drafted at P3, confirmed at P4).
5. P4 paperwork checklist (CH-11 retro v1 + CH-12 retro v2 P4 addenda):
   - Every modified doc with verified-header line 1 has its description matching the body diff exactly.
   - Every concept-audit-matrix row touched copies its Status from §2 plan target verbatim.

**Tests**: re-run full workspace test + 4 CI guards (verification recipe §12).

**Confidence target**: ≥ 99%.

**Pause discipline**: PAUSE if any CI guard or test fails at re-run; investigate before sealing.

## §8 — Tests summary

**Expected total test count at chunk close**: predicted 1462–1480 (1431 baseline + 31–49 new = +31 lower / +49 upper). Asymmetric accept band per CH-11/CH-12 retro v2: ×1.0 lower (1462) / ×1.20 upper (1490). PAUSE → AskUserQuestion if outside `1462..1490`.

**Layer breakdown**:
- Unit tests: ~18-25 new (templates/a.rs builder + listener body + audit-event builder + manifest PartialEq + builders/session_launch.rs).
- Acceptance / integration tests: ~9-15 new (`acceptance_sessions_m5p4.rs` + `acceptance_authority_templates.rs` amendments + `acceptance_template_a_session_grant_backfill.rs` NEW).
- Migration tests: 1-2 new (in-memory + Surreal idempotency on `0015`).
- Property tests: 0 new (existing `template_a_fire_grant_shape_props` proptest amended to assert `grants.len() == 2`).

**Named test files**:
- NEW: `modules/crates/domain/src/permissions/builders/session_launch.rs` (mod tests).
- NEW: `modules/crates/domain/src/audit/events/m5_2/session_launch.rs` (mod tests).
- NEW: `modules/crates/server/tests/acceptance_template_a_session_grant_backfill.rs`.
- AMENDED: `modules/crates/domain/src/templates/a.rs` (existing tests; +6-8 new).
- AMENDED: `modules/crates/domain/src/events/listeners.rs` (+2 new).
- AMENDED: `modules/crates/server/tests/acceptance_sessions_m5p4.rs` (+8-10 new).
- AMENDED: `modules/crates/server/tests/acceptance_authority_templates.rs` (+2-3 new).
- AMENDED: `modules/crates/server/tests/acceptance_per_session_consent_gating.rs` (regression-only re-run; expect 0 new tests but re-verify all pass).

**Named expected-still-green tests**:
- `acceptance_per_session_consent_gating.rs` — Step 6 path unchanged.
- `acceptance_authority_chain.rs` — provenance walk unchanged.
- `acceptance_sessions_m5p4.rs` (existing 8 scenarios) — happy paths still pass after the hard-deny flip because P1 backfills Template A grants for CEOs / leads.
- `acceptance_bootstrap.rs` — bootstrap claim path unchanged.
- 38-cell manifest validator tests — no validator change.

**Ignored count expected**: 2 (unchanged from baseline).

## §9 — Pre-chunk gate

**Reading list (mandatory; verified by planner at draft time):**

1. Concept docs (verified verbatim 2026-05-08):
   - `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md` (lines 1-589 read; §"Permission Check (Runtime Reconciliation)" + §"Formal Algorithm" + §"Key Invariants" + §"Authority Chain" focused).
   - `docs/specs/v0/concepts/permissions/07-templates-and-tools.md` (lines 1-826 read; §"Standard Permission Templates" + §"Template A — Project Lead Authority" + §"Tool Authority Manifest Examples" + §"Authoring a Tool Manifest" focused).
   - `docs/specs/v0/concepts/permissions/03-action-vocabulary.md` (verified the closed-set principle for F3 re-interpretation).
   - `docs/specs/v0/concepts/permissions/README.md` (entry-invariants source per template §2 rule).
2. Drift files:
   - `m5_1/drifts/D4.1.md` (verified verbatim).
3. Prior-chunk plans:
   - CH-02 plan archive (CH-02 closes phi-core agent_loop wiring; relied on for spawn_agent_task path).
   - CH-04 plan archive (closed-set Action vocabulary).
   - CH-06 plan archive (selector grammar).
   - CH-09 / CH-10 / CH-11 plan archives (per-session consent gating).
   - CH-13 plan archive (audit-class composition).
   - CH-14 plan archive (authority chain — `walk_provenance_chain` reused here).
4. `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §5 row 13 (CH-15) + §7 Q&A binding decisions.
5. `baby-phi/CLAUDE.md` phi-core Leverage section + `phi-core/CLAUDE.md` summary.
6. `permissions/04-manifest-and-resolution.md` Step-N enforcement semantics (re-read at chunk-open).
7. `repository.rs` module-level docstring lines 19-48 (Repository trait contract block) — **NOT applicable** to CH-15 (no new tag-write Repository method introduced; the new audit-event emission is via the existing `Arc<dyn AuditEmitter>` not a Repository tag-write surface).

**Carry-forward invariants** (verified green at chunk open 2026-05-08):
- `cargo test --workspace -j 4` returns 1431/0/2 (verified).
- `scripts/check-phi-core-reuse.sh` green (verified).
- `scripts/check-doc-links.sh` green (verified).
- `scripts/check-ops-doc-headers.sh` green (verified).
- `scripts/check-spec-drift.sh` green (verified).
- `modules/` diff against chunk-open git HEAD = empty (verified — no preload edits).

**Pending decisions carried into this chunk**: F1, F2, F3, F4, F5 — surfaced in §"Forks for orchestrator" above. F1, F3 + F4 require user lock before plan approval.

**Drift-file `discovered → classified → scoped` transitions owed**: D4.1 must be `classified` at chunk open (verified — its current status is `discovered` per the lifecycle line 50; planner verified it's in `classified` per `Bucket: A — load-bearing scope gap` annotation at line 12; CH-15 transitions it to `remediated` at P4 seal).

## §10 — Close criteria

**Source of truth**: concept docs (`permissions/04` + `permissions/07` + `permissions/03`).

**4 aspects (each pass / fail)**:

- **Code aspect** — every phase's deliverables shipped; full workspace `cargo test -j 4` green; `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` green; `cargo fmt --all -- --check` green.

- **Docs aspect**:
  - *Governance tier*: D4.1 status `discovered` → `remediated`; ADR-0054 status `Proposed` → `Accepted`; concept-audit-matrix rows 1 + 2 + 7 + 180 (line) of permissions/04 + permissions/07 blocks Status flip per §3.C + §7 P3 (letter-for-letter copy of §2 target columns per CH-12 retro F-AUDB-1).
  - *User-facing tier*: every §3.C row updated in-chunk OR carrying explicit defer-decision (no defers planned).

- **phi-core leverage aspect** — `check-phi-core-reuse.sh` green; positive grep (a) returns 49 (unchanged); forbidden-duplication greps (e) + (f) + (g) return 0; new-builder-fn grep (b) returns ≥ 1; advisory-string grep (c) returns 0; hard-deny grep (d) returns ≥ 2.

- **Concept alignment aspect** — every §2 row's target-status at chunk-close achieved; rows 1 + 2 + 7 verified by acceptance tests (§7 P2 deliverables); no row remains `contradicted`.

**2 confidence % (named numerator/denominator)**:

- **Implementation confidence %** = `(claims-honored-by-tests-and-code) / (total-claims-in-scope)`. Target: **≥ 9/10 = 90%**. Concrete: 9/9 = 100% target if all §2 rows honored + every F-locked option implemented. The 1 remaining slot is reserved for any mid-flight discovery converted to a new drift.

- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)`. Target: **≥ 7/7 = 100%** across the 7 §3.C touched files (3 m5 + 2 m5_2-NEW + 1 concept-audit-matrix + 1 ADR).

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** Target: **≥ 9/10 = 90%**.

**P4 chunk-seal paperwork checklist** (per template §10 v2026-05-03 + v2026-05-04 addenda):
- For every modified doc with line-1 verified-header: re-run after body diff complete; description matches body exactly.
- For every concept-audit-matrix row touched: Status copied letter-for-letter from §2 plan target column. No collapse-to-binary on partially-honored rows.

## §11 — Post-chunk independent audit plan

**Agent count**: **2 agents** (medium chunk — 4 phases per template §11; envelope assessment via `audit-envelope-size` skill).

**Audit aspects (a-d)** distributed across agents:

### Audit Agent A — Code correctness + phi-core leverage

**Audit prompt** (≤ 600 words):

> You are the chunk-auditor for CH-15 (real permission-check gate at session launch). Read the chunk plan at `baby-phi/docs/specs/plan/build/ch-15-real-permission-check-gate-at-session-launch-c3f46f17/plan.md`. Audit the implementer's changes for code correctness + phi-core leverage. Your scope: §3 phi-core leverage map + §3.B K8s readiness + §3 Artifact A/B/C cascades + Phase P1 (Template A double-grant + migration `0015`) + Phase P2 (hard-deny flip + builder wiring + audit-event).
>
> **Files in scope**:
> - `modules/crates/domain/src/templates/a.rs` (Template A pure-fn return-type change to `Vec<Grant>`).
> - `modules/crates/domain/src/events/listeners.rs` (TemplateAFireListener body iterates over Vec<Grant>).
> - `modules/crates/store/migrations/0015_template_a_session_object_grant.surql` (additive UPDATE; verify idempotency).
> - `modules/crates/domain/src/permissions/builders/mod.rs` + `builders/session_launch.rs` (NEW).
> - `modules/crates/domain/src/permissions/manifest/mod.rs` (PartialEq derive).
> - `modules/crates/server/src/platform/sessions/launch.rs` (advisory→hard-deny flip; builder call site at gate_session_launch_consent + Step 3).
> - `modules/crates/server/src/platform/sessions/preview.rs` (builder call site).
> - `modules/crates/domain/src/audit/events/m5_2/session_launch.rs` (NEW; launch_denied event).
>
> **Greps to run**:
> - `grep -rn "use phi_core\|pub use phi_core" modules/crates/ --include="*.rs" | wc -l` — expect 49 (unchanged).
> - `git grep -nE 'advisory at M5; not blocking|advisory at M5\b' modules/crates/server/src/platform/sessions/launch.rs` — expect 0.
> - `git grep -nE 'pub fn build_session_launch_manifest\b' modules/crates/domain/src/permissions/builders/` — expect ≥ 1.
> - `git grep -nE 'fire_grant_on_lead_assignment\b' modules/crates/` — count cascade against §3 Artifact A's predicted 12 deliberate edits across 4 files; PAUSE if > 18.
> - `bash scripts/check-phi-core-reuse.sh` — green.
> - `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets` (mark `NOT-EXECUTED-IN-AUDIT` if sandbox-blocked — orchestrator closes at gate-4).
>
> **Pass criteria**: every claim above honored by code + greps; no forbidden duplications; phi-core baseline preserved at 49; the §3 Artifact A cascade band held; the §3 Artifact B builder is the sole construction site for the synthetic launch manifest (preview + launch + gate_session_launch_consent all call it); the launch_denied audit-event canonical_bytes shape matches §"Forks" F5.B.
>
> **Report format**: §A scope-walk; §B per-claim verdict (PASS / PARTIAL / FAIL with file:line citations); §C cascade re-count; §D phi-core grep results; §E open questions; §F overall verdict (GREEN / YELLOW / RED).

### Audit Agent B — Docs fidelity + concept alignment + paperwork

**Audit prompt** (≤ 600 words):

> You are the chunk-auditor for CH-15. Read the chunk plan at `baby-phi/docs/specs/plan/build/ch-15-real-permission-check-gate-at-session-launch-c3f46f17/plan.md`. Audit the implementer's changes for docs fidelity + concept alignment + chunk-seal paperwork. Your scope: §2 concept alignment walk + §3.C user-facing doc impact map + §4 drifts closed + §5 ADRs drafted + §10 close criteria + Phase P3 (doc updates) + Phase P4 (seal paperwork).
>
> **Files in scope**:
> - `docs/specs/v0/implementation/m5/architecture/session-launch.md` (advisory→hard-deny rewrite).
> - `docs/specs/v0/implementation/m5_2/architecture/session-launch-permission-gate.md` (NEW).
> - `docs/specs/v0/implementation/m5/operations/session-launch-operations.md` (error-code reference + D4.1 mention deletion).
> - `docs/specs/v0/implementation/m5_2/operations/session-launch-permission-gate-operations.md` (NEW).
> - `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` (advisory deletion).
> - `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` (if exists).
> - `docs/specs/v0/implementation/m5_1/drifts/D4.1.md` (lifecycle status update).
> - `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` (rows touched per §3.C).
> - `docs/specs/v0/implementation/m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md` (NEW; §"Forks" + §"Cross-references" + pre-existing-behaviour preservation).
>
> **Greps to run**:
> - `grep -rnE 'D4\.1|advisory at M5|not blocking|Step 0 only blocking' docs/specs/v0/implementation/m*/` — expect 0 hits in `architecture/` + `operations/` + `user-guide/` directories (the `drifts/D4.1.md` lifecycle row is the sole acceptable mention).
> - `bash scripts/check-doc-links.sh` — green.
> - `bash scripts/check-ops-doc-headers.sh` — green.
> - Verify ADR-0054 carries §"Forks" + §"Cross-references" (4 categories, milestone-prefixed) + pre-existing-behaviour preservation note per CH-13/CH-14 retro caveats.
>
> **Pass criteria**: every §3.C row's status (`update in-chunk` / `defer with reason`) is honored by the diff; D4.1 lifecycle history correctly transitioned; ADR-0054 §"Cross-references" cites all 4 categories with milestone prefixes for cross-milestone ADRs (per CH-08 retro v6); concept-audit-matrix Status flips letter-for-letter per CH-12 retro F-AUDB-1; verified-header line 1 description matches body diff per CH-11 retro v1.
>
> **Report format**: §A scope-walk; §B per-claim verdict (PASS / PARTIAL / FAIL with file:line citations); §C concept-audit-matrix row diff; §D ADR-0054 §"Cross-references" coverage; §E paperwork-checklist verdict; §F overall verdict (GREEN / YELLOW / RED).

**Audit pass criteria**:
- Any new drift discovered → its own drift file BEFORE chunk seal.
- Any audit-flagged concept contradiction → fixed in-chunk OR renegotiated with user OR converted to a new drift with explicit future-chunk assignment.
- Chunk seal blocked until BOTH agents return clean on all 4 aspects.

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace -j 4

# 3. Chunk-specific greps
git grep -nE 'use phi_core\|pub use phi_core' modules/crates/ --include="*.rs" | wc -l
# Expect: 49 (baseline preserved)

git grep -nE 'advisory at M5; not blocking|advisory at M5\b' modules/crates/server/src/platform/sessions/launch.rs
# Expect: 0 hits (all advisory strings removed)

git grep -nE 'pub fn build_session_launch_manifest\b' modules/crates/domain/src/permissions/builders/
# Expect: 1 hit (the function definition)

git grep -nE 'build_session_launch_manifest\(' modules/crates/server/src/platform/sessions/
# Expect: 3 hits (preview.rs:NN + launch.rs:NN + launch.rs gate_session_launch_consent)

git grep -nE 'platform\.session\.launch_denied' modules/crates/
# Expect: ≥ 2 hits (audit-event-builder definition + launch.rs emit site)

git grep -nE 'fire_grant_on_lead_assignment\b' modules/crates/
# Expect: ~10-12 hits (Artifact A cascade band — pure-fn def + listener + tests)

git grep -nE 'PERMISSION_CHECK_FAILED' modules/crates/server/
# Expect: ≥ 3 hits (was: 2 hits at chunk-open — adds the new step-1..6 site at launch.rs)

# 4. Drift-file status
grep -l "Status.*remediated" docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <baseline_count> + 1 (D4.1 transitions to remediated)

grep -nE "Accepted|Proposed" docs/specs/v0/implementation/m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md
# Expect: status `Accepted` after chunk seal

# 5. Migration sanity
ls modules/crates/store/migrations/0015_*.surql
# Expect: 1 file matching `0015_template_a_session_object_grant.surql`

# 6. Acceptance test focus run
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_sessions_m5p4 -j 4 -- --nocapture 2>&1 | tail -5
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_per_session_consent_gating -j 4 2>&1 | tail -5
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_template_a_session_grant_backfill -j 4 2>&1 | tail -5
/root/rust-env/cargo/bin/cargo test -p domain --lib -j 4 2>&1 | tail -5
```

---

## Appendix A — Verified greps + reads (planner did this 2026-05-08, cycle hex c3f46f17)

- `Action::CANONICAL.len() == 34` invariant test at `action.rs:439` — verified.
- `Manifest::is_empty` predicate at `manifest/mod.rs:99-101` — verified semantics (empty = `actions.is_empty() OR (resource ∪ transitive).is_empty()`).
- launch.rs advisory strings at lines 198-216 + line 244 + line 545 + line 715 — verified 4 sites.
- preview.rs synthetic manifest at lines 88-95 — verified.
- Template A pure-fn at `templates/a.rs:103-131` — verified.
- TemplateAFireListener body at `events/listeners.rs:264-360` — verified the `repo.create_grant(&grant).await?` call shape.
- migrations directory contents `0001_initial.surql` through `0014_authority_chain.surql` — verified next-free slot is `0015`.
- ADR directory `m5_2/decisions/` — verified next-free ADR is `0054`.
- `_concept-audit-matrix.md` rows for permissions/04 (lines 175-184) + permissions/07 (lines 218-225) — verified row count + Status values.
- `Composite::SessionObject` constituents `[DataObject, Tag]` at `composites.rs:168-172` — verified.
- Action × Fundamental matrix: Read/Inspect/List × DataObject = ✓; Read/Inspect/List × Tag = ✓ — verified via concept doc 03 lines 27-37 + the 306-cell exhaustive test at `action.rs:614-721`.

## Appendix B — Verified-header sentence drafted at P3 (CH-12 retro F-AUDB-1)

For `_concept-audit-matrix.md` line 1 update at chunk seal (drafted; actual line goes in P4):

```
<!-- Last verified: 2026-05-?? by Claude Code (CH-15 chunk-seal: in the `permissions/04-manifest-and-resolution.md` block, the "All steps hard-gate (not advisory)" row Status flipped letter-for-letter from `contradicted` to `**honored**` per CH-12 retro Row 1 P4 paperwork addendum (copy-paste of CH-15 plan §2 row 1 target); Code-evidence cell now cites `domain::permissions::builders::build_session_launch_manifest` + `server::platform::sessions::launch.rs` hard-deny match arm at line ~226 + acceptance test `acceptance_sessions_m5p4::test_launch_denies_with_403_for_each_failed_step` covering steps 0..6 + ADR-0054 §D54.1+§D54.5+§D54.6; Covering-drift cell flipped to `D4.1 ✓`. In the `permissions/07-templates-and-tools.md` block, the "Template A [read, inspect, list] on project" row Status flipped letter-for-letter from `partially-honored` to `**honored**` (Code-evidence cell cites `templates/a.rs::fire_grant_on_lead_assignment` returning `Vec<Grant>` + the paired session_object grant + migration `0015` backfill + ADR-0054 §D54.3+§D54.4); Covering-drift `D4.1 ✓`. No new rows added. -->
```
