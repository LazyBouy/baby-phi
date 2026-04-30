<!-- Last verified: 2026-04-27 by Claude Code -->

# CH-01 — Agent durable lifecycle (`active` + `archived_at`)

**Plan file token:** `2aa37c80` (generated via `openssl rand -hex 4`)
**Chunk ID:** CH-01 (see [forward-scope §1 CH-01 block](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) and [§5 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md))
**Severity:** HIGH
**Expected effort:** ~2.25 engineer-days (incl. P0 process-doc ratification)
**Chunks enabled after close:** CH-22 (AgentCatalogListener body reads `Agent.active`)
**New ritual codified by this chunk:** K8s microservice readiness check (§3.B) — binding for all subsequent chunk plans

---

## §1 — Context & principle

### Why this chunk

M4/P0 added the `agent.role` column (migration 0004) but stopped short of the durable lifecycle fields the concept docs mandate: `active: bool` and `archived_at: Option<DateTime>`. Today the system-agent disable / archive handlers ([`platform/system_agents/disable.rs`](modules/crates/server/src/platform/system_agents/disable.rs) and [`archive.rs`](modules/crates/server/src/platform/system_agents/archive.rs)) emit audit events but **do not flip any persisted state** — there is no column to flip. AgentCatalogListener at [`domain/src/events/listeners.rs:497`](modules/crates/domain/src/events/listeners.rs#L497) is a P3 stub today; the M5.2/P8 body that lights up the `SystemAgentRuntimeStatus` tile cannot read durable disable/archive state because the columns don't exist. This is drift **D6.5** (HIGH).

CH-01 also ratifies two adjacent drifts:
- **D-new-22** (role immutability) — verified already enforced at [`platform/agents/update.rs:133-141`](modules/crates/server/src/platform/agents/update.rs#L133); CH-01 closes this drift by adding the explicit acceptance test that pins the rule against future regression.
- **D-new-23** (Human Agent has no system-computed Identity) — partial close. The actual guard is moot at M5 because Identity has no writers (D-new-01 deferred to CH-16). CH-01 records a dated lifecycle entry confirming the scope review and forwards the closure to CH-16.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* — M5.1 governing principle.

Applied here: `agent.md` §"Roles" treats role as immutable post-creation; `system-agents.md` §"Operator can disable" treats disable as a durable state-flip, not just an audit ping. Both must be reflected in code + tests, not in inference.

### Forward-scope reference

[Forward-scope §1 CH-01 block](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) + [§5 CH-01 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md).

**Note:** forward-scope §1 CH-01 says "migration 0006" — this was written before CH-02 shipped migration 0006 (`agent_profile_mock_response`). CH-01 lands as **migration 0007**. ADR is **ADR-0034** (next free above CH-K8S-PREP's ADR-0033).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`agent.md`](../../v0/concepts/agent.md) | §"Agent Roles" | *"Role is immutable post-creation; role transitions go through separate flows."* | honored (enforced at `update.rs:133-141`, but no acceptance test pins it) | honored (acceptance test added) |
| [`agent.md`](../../v0/concepts/agent.md) | §"Lifecycle" / Roles intro | *"Agents have an active vs disabled vs archived lifecycle. Disable pauses participation; archive is a terminal soft-delete."* | contradicted (no `active`, no `archived_at` column on `agent`) | honored (columns added, persisted, read by listeners + handlers) |
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"Operator can disable — pauses trigger subscriber" | *"Disabling a system agent pauses its subscription to the event bus until re-enabled."* | partially-honored (handler emits audit, AgentCatalogListener has no way to read disabled state) | honored (handler flips `active = false`; CH-22 will read `active` at body-wiring time) |
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"Archive flow" | *"Archive marks the agent as terminal — non-standard system agents only; standard agents (memory-extraction, agent-catalog) cannot be archived."* | partially-honored (handler rejects standard agents but doesn't write `archived_at`) | honored (handler writes `archived_at = now`) |
| [`human-agent.md`](../../v0/concepts/human-agent.md) | §"No system-computed Identity" | *"Human Agents do **not** have a system-computed Identity node."* | silent-in-code (no Identity writers exist at M5; guard would have nothing to guard against) | silent-in-code (deferred to CH-16; lifecycle entry on D-new-23 confirms scope review at CH-01 time) |

**Permissions subtree hook:** none. CH-01 does not touch `permissions/01`–`permissions/09`. No selectors, no actions, no manifest changes.

**phi-core-mapping hook:** baby-phi's `Agent` struct in `domain/src/model/nodes.rs` is a baby-phi-only governance node (NOT a wrap of a phi-core type) per [`phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md) "Orthogonal surfaces — `domain::Agent` is governance metadata, not phi-core's runtime agent identity." Schema additions therefore do not interact with phi-core types. No row in §3 below.

---

## §3 — phi-core leverage map

CH-01 touches no phi-core type's structure or imports. All work lives in baby-phi's `domain` (struct + repo trait), `store` (migration + repo impl), and `server` (handler wiring + acceptance test) crates. The orthogonality claim is grounded — phi-core has no struct counterpart to baby-phi's `Agent` governance node — but the existing project docs are imprecise about this. P0 closes those doc gaps alongside the K8s readiness ratification.

**Connection-point map** (where baby-phi's `domain::Agent` flows into phi-core types — ID-only, no struct-level reuse):

| phi-core type | Shape | Current handling in baby-phi | Classification | Action in CH-01 |
|---|---|---|---|---|
| `phi_core::Agent` ([`phi-core/src/agents/agent.rs:60`](../../../../../phi-core/src/agents/agent.rs#L60)) | TRAIT (runtime interface: prompting / state / control) | Not implemented by `domain::Agent`; satisfied at runtime by phi-core's `BasicAgent` | **Orthogonal** — runtime interface vs governance node | Confirm orthogonality; document in P0 |
| `phi_core::agents::profile::AgentProfile` | STRUCT (execution blueprint: `system_prompt`, `thinking_level`, `temperature`, etc.) | Wrapped by `domain::AgentProfile.blueprint` (shipped CH-02) | **Wrap (existing)** | Untouched |
| `phi_core::types::context::AgentContext` | STRUCT (in-memory loop accumulator with `agent_id: Option<String>`) | Constructed at [`sessions/provider.rs:61-68 build_agent_context`](modules/crates/server/src/platform/sessions/provider.rs#L61) from `ctx.started_by.to_string()` (where `started_by: domain::AgentId`) | **Direct-reuse (existing)** — receives ID string only | Untouched |
| `phi_core::agents::BasicAgent.last_active_at: Option<DateTime<Utc>>` | Runtime in-memory rotation tracker (not persisted, no serde) | Not used | **Orthogonal** — runtime rotation vs persisted governance | Confirm `domain::Agent.active` does NOT shadow this; they serve different purposes |

**Why `domain::Agent` is orthogonal (not a phi-core wrap candidate):**

- phi-core's `Agent` is a **trait**, not a struct — there's no field set to wrap or inherit.
- phi-core's `AgentProfile` is the execution blueprint (no governance fields: no `role`, `owning_org`, `kind`, `active`, `archived_at`).
- The new `active: bool` and `archived_at: Option<DateTime<Utc>>` fields encode **platform permission policy** (who is allowed to launch sessions, who appears in catalogs, who's archived). They are not runtime execution control.
- Connection is ID-only: `domain::Agent.id.to_string()` → `AgentContext.agent_id`. No struct reuse possible.

**Expected import-count delta at chunk close:** **0**. CH-01 adds no phi-core imports and removes none.

**Positive close-audit greps** (must return ≥ 1 each — confirms structural extension landed):
```bash
grep -n "pub active: bool" modules/crates/domain/src/model/nodes.rs
grep -n "pub archived_at: Option<DateTime<Utc>>" modules/crates/domain/src/model/nodes.rs
grep -n "set_agent_active" modules/crates/domain/src/repository.rs
grep -n "set_agent_archived_at" modules/crates/domain/src/repository.rs
ls modules/crates/store/migrations/0007_*.surql
```

**Forbidden-duplication greps** (must return 0 each):
```bash
grep -rn "^pub struct Agent\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"     # 0 hits
grep -rn "^pub enum AgentRole\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"   # 0 hits
bash scripts/check-phi-core-reuse.sh                                                      # exit 0
```

---

## §3.B — K8s microservice readiness check

**Rule (binding from this chunk forward):** Every chunk plan evaluates whether its changes introduce new K8s-deployment hurdles. The rule is codified at:
- [`m5_1/process/per-chunk-planning-template.md`](../../v0/implementation/m5_1/process/per-chunk-planning-template.md) §3.B (template content — every chunk fills in)
- [`forward-scope/22035b2a-...md`](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §7 Q8 (binding planning decision)
- [`m7b/architecture/k8s-microservices-readiness.md`](../../v0/implementation/m7b/architecture/k8s-microservices-readiness.md) §11 (strategic doc cross-link)
- [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) (any new blocker creates a CHK8S-D-XX entry here before chunk seal)

P0 below ratifies these doc changes for the first time.

**The 7-axis evaluation table for CH-01:**

| K8s-deployability axis | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|
| **A1** In-process state (`DashMap`, `RwLock`, `AtomicBool`, mutex, `OnceCell`, etc.) | None added; CH-01 only writes durable SurrealDB columns | No | — |
| **A2** IPC channel (mpsc, broadcast, oneshot, watch) | None added | No | — |
| **A3** Pod-local resource (file handle, listener, sub-process, lock file, on-disk cache) | None added | No | — |
| **A4** Migration runner / first-apply race | Migration 0007 added; runner unchanged. Already-known issue: **CHK8S-D-05** (migration runner leader-election lock missing for multi-replica startup). Adding a single column-add migration does not aggravate the race posture | No new blocker | Cross-ref existing CHK8S-D-05 |
| **A5** Trait-shape requirement (does any new surface need to be trait-objects-friendly for future broker/Redis/remote swap?) | New repo methods (`set_agent_active`, `set_agent_archived_at`) land on existing `Repository` trait — already trait-shaped per CH-K8S-PREP P-2 conforming criteria | No | — |
| **A6** Cross-pod state sharing (does this introduce data that must be visible cross-pod?) | `Agent.active` and `archived_at` are durable SurrealDB columns — already cross-pod-visible via `SurrealStore::open_remote` (CH-K8S-PREP P-2). AgentCatalogListener (CH-22) will read via `repo.get_agent`, not from in-process cache | No | — |
| **A7** Audit hash-chain symmetry (does the chunk introduce a new audit writer that breaks single-writer guarantee, or sidesteps the existing emitter?) | No new audit writers; existing system-agent disable/archive handlers continue using the existing `AuditEmitter` impl | No | — |

**Conclusion:** CH-01 is **K8s-neutral**. No new blockers introduced; no new entries in `deferred-from-ch-k8s-prep.md` required.

**Conforming criteria for ADR-0033 (CH-K8S-PREP) still satisfied:**
- D33.1 (`SessionRegistry` trait) — untouched.
- D33.2 (`SurrealStore::open_remote`) — repo methods inherit the trait-shaped `Repository` impl; remote-DB compatibility unchanged.
- D33.3 (SIGTERM graceful shutdown) — no new spawn tasks added.
- D33.4 (`EventBus.shutdown` + `drain`) — no new event emitters added.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D6.5` | [`../../v0/implementation/m5_1/drifts/D6.5.md`](../../v0/implementation/m5_1/drifts/D6.5.md) | HIGH | `scoped → in-chunk-plan → remediated` | Migration 0007 adds columns; struct extended with `#[serde(default)]`; 2 repo methods; 2 handlers wired |
| `D-new-22` | [`../../v0/implementation/m5_1/drifts/D-new-22.md`](../../v0/implementation/m5_1/drifts/D-new-22.md) | MEDIUM | `scoped → in-chunk-plan → remediated` | Code already enforces (`update.rs:133-141`); CH-01 adds acceptance test that pins the rule + flips drift to `remediated` |
| `D-new-23` | [`../../v0/implementation/m5_1/drifts/D-new-23.md`](../../v0/implementation/m5_1/drifts/D-new-23.md) | LOW | stays at `scoped` | Partial closure only — guard cannot be added until D-new-01 (Identity writers) lands at CH-16. CH-01 appends a dated lifecycle entry: *"2026-04-27 — reviewed at CH-01 plan time; scope confirmed deferred-to-CH-16 because Identity has no writers at M5; no code action this chunk"*. Drift owner remains CH-16. |

Lifecycle transitions happen at chunk seal (P5). No drift transitions before seal.

---

## §5 — ADRs drafted

ADR number assigned at plan-drafting time per Q6 rule. Command used:
```bash
ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null \
  | xargs -I{} basename {} .md \
  | grep -oE "^[0-9]{4}" | sort -u | tail -5
# result: 0030, 0031, 0032, 0033 → next free = 0034
```

| # | Title | Drafted-at-phase | Decision summary | Flip to Accepted at |
|---|---|---|---|---|
| **ADR-0034** | Agent durable lifecycle state + governance-vs-runtime boundary | Step 2 (pre-P1) | Six sub-decisions: D34.1 `active: bool DEFAULT true` column on `agent` table; D34.2 `archived_at: option<datetime>` column; D34.3 two repo methods (`set_agent_active`, `set_agent_archived_at`); D34.4 system-agent disable/archive handlers wired to flip durable state; D34.5 conforming criteria for CH-22 (catalog body MUST read `Agent.active` from repo, not from audit log); **D34.6 governance-vs-runtime boundary — `domain::Agent` is a governance-only struct; baby-phi never instantiates `phi_core::Agent` (trait) or `BasicAgent` (impl) because baby-phi is per-request stateless; the only connection from `domain::Agent` to phi-core runtime is ID-string propagation at `sessions/provider.rs::build_agent_context`. Review trigger: any future milestone introducing long-lived in-memory chat agents would re-evaluate trait usage.** | Chunk seal (P5) |

ADR file path: [`../../v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md`](../../v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| M5/P7 + CH-02 + CH-K8S-PREP baseline | `cargo test --workspace -- --test-threads=1` = 985 passed | `/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1 2>&1 \| grep -E "^test result:" \| awk -F'[;: ]+' '{s+=$4} END {print s}'` |
| CH-02 | Migration 0006 (`agent_profile_mock_response`) is the highest-applied migration; new migration must register as 0007 | `ls modules/crates/store/migrations/*.surql \| sort \| tail -1` returns `0006_agent_profile_mock_response.surql` |
| CH-K8S-PREP | `SurrealStore::open_embedded` + `open_remote` both still work; migration runner unchanged | `/root/rust-env/cargo/bin/cargo test -p store --tests` green |
| CH-K8S-PREP | `SessionRegistry` trait surface intact; AppState wiring untouched | `grep -n "trait SessionRegistry" modules/crates/server/src/state.rs` returns ≥ 1 |
| M4/P0 | `agent.role` column rejects non-canonical values; `#[serde(default)]` handles pre-M4 rows | `/root/rust-env/cargo/bin/cargo test -p store --test migrations_0004_test` green |
| M5/P6 | System-agent disable/archive handlers reject standard system agents (memory-extraction, agent-catalog) and require `confirm: true` | `acceptance_system_agents.rs` tests still green |
| M5/P3 | AgentCatalogListener stub at `listeners.rs:497-543` is a no-op debug! body subscribed to 8 variants | `grep -n "AgentCatalogListener" modules/crates/domain/src/events/listeners.rs` returns ≥ 1 |
| All chunks | 4 CI guards green | `bash scripts/check-doc-links.sh && bash scripts/check-ops-doc-headers.sh && bash scripts/check-phi-core-reuse.sh && bash scripts/check-spec-drift.sh` |

These run at chunk-open (Step 2) AND chunk-seal (Step 8).

---

## §7 — Phases within the chunk

### P0 — Process-doc ratification: K8s readiness rule (~0.25d)

**Goal.** Codify the K8s microservice readiness check as a binding rule for this chunk and all subsequent chunks. P0 ships **only documentation changes**; no code.

**Deliverables.**
1. **[`docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md`](docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md)** — extend §3 with a new sub-section **§3.B "K8s microservice readiness check"**. Content:
   - 7-axis table template (axes A1–A7 as defined in this CH-01 plan's §3.B above).
   - Rule statement: any new K8s blocker introduced by the chunk must file a `CHK8S-D-XX` entry in [`deferred-from-ch-k8s-prep.md`](docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md) before chunk seal.
   - Conforming-criteria check against ADR-0033 D33.1–D33.4 (every chunk verifies it doesn't break those four trait-surface contracts).
   - Bump the file's `Last verified` header.
2. **[`docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md)** §7 — append **Q8 — K8s microservice readiness rule**:
   - Decided: every chunk plan evaluates the 7 K8s-deployability axes via §3.B; any new blocker creates a `CHK8S-D-XX` entry in the m7b ledger.
   - Scope impact: per-chunk-planning-template gains §3.B; CH-01 is the first chunk to apply.
   - Rationale: prevent silent K8s-deployment-debt accumulation between now and M7b plan-open. Pre-position trait surfaces incrementally instead of in a big-bang at M7b.
   - Pre-Q8 chunks (CH-02, CH-K8S-PREP) are grandfathered (CH-K8S-PREP itself originated the 7-axis evaluation).
   - Bump file's `Last verified` header.
3. **[`docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md`](docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md)** — append a new closing §11 titled **"Per-chunk readiness check rule (CH-01+)"** with cross-links to the per-chunk-planning-template's §3.B + forward-scope §7 Q8 + the deferred-items ledger. Bump `Last verified` header.
4. **[`docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md`](docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md)** — extend the "Adding new entries" guidance with: *"From CH-01 onward, the per-chunk-planning-template §3.B K8s-readiness check is the canonical source of new entries. Each entry's provenance field cites the originating chunk."* Bump `Last verified` header.
5. CH-01's own §3.B above stands as the worked example for future drafters.

**Doc-gap closures surfaced during plan-time phi-core leverage check** (independent of K8s rule, fixed in same P0 since both are process-doc work):

6. **[`baby-phi/CLAUDE.md`](CLAUDE.md)** §"Orthogonal surfaces that are NOT phi-core duplicates" — append a new bullet: *"`domain::Agent` (governance node — identity, role, org membership, lifecycle) vs `phi_core::Agent` trait (runtime interface) and `phi_core::BasicAgent` (runtime in-memory state) — wholly orthogonal; baby-phi tracks principals, phi-core executes them. Connection at `sessions/provider.rs::build_agent_context`: ID-only delegation."* Bump file's `Last verified` (none today, but set/update as appropriate).
7. **[`docs/specs/v0/concepts/phi-core-mapping.md`](docs/specs/v0/concepts/phi-core-mapping.md)** §"agents/" classification table — clarify two ambiguous rows:
   - Add `phi_core::Agent` (trait) → **Runtime-only** (the stateless execution interface; not a node).
   - Refine the `BasicAgent` row to note: **Runtime-only impl of `phi_core::Agent` trait**; baby-phi does not persist `BasicAgent` state. baby-phi's `domain::Agent` is the governance counterpart (orthogonal — see §"Connection point" below).
   - Add a short §"Connection point" or §"Agent governance vs runtime separation" sub-section pointing to `sessions/provider.rs::build_agent_context` as the ID-only delegation site.
   - Bump `Last verified` header.

**Concept re-evaluation marker surfaced during CH-01 plan review** (out-of-scope for CH-01 implementation; in-scope for repo-level capture so the question is recoverable at a future milestone):

8. **[`docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §3** — append a new entry **`M6+-OPEN-01 — AgentProfile cardinality re-evaluation (1:1 → N:1 template-sharing)`**. Distinguished from the existing `M6-DEFERRED-*` / `M7-DEFERRED-*` / `M7b-DEFERRED-*` markers because those represent scope items that *will* happen; this entry represents an *open question* that may or may not be pursued. Entry contents:
   - **Status**: *open question — may or may not be pursued; not a committed deferred-scope item*.
   - **Origin**: surfaced during CH-01 chunk-open by user during plan review (2026-04-27). Investigation confirmed the current 1:1 cardinality is concept-mandated by [`ontology.md`](../../v0/concepts/ontology.md) line 92 + [`agent.md`](../../v0/concepts/agent.md) §"Soul" (profile-as-genetics) + schema UNIQUE constraint at migration 0001:131.
   - **Question to evaluate**: should baby-phi adopt N:1 template-sharing (multiple agents share one profile row) instead of the current 1:1 model? Argument FOR: profiles are templates ("intern coder", "research assistant"), template-sharing fits standard infrastructure patterns (Kubernetes ConfigMaps, Helm values), and ephemeral `AgentContext` makes profile sharing structurally feasible without disrupting runtime semantics. Argument AGAINST: concept-mandated profile-as-genetics + per-agent governance fields (`parallelize`, `model_config_id`, `mock_response`) + audit clarity + org-level isolation.
   - **Required if pursued**:
     - Concept-doc amendment to `agent.md` §"Soul" + `ontology.md` cardinality line.
     - New ADR documenting the redesign rationale + migration plan.
     - Schema migration: drop UNIQUE constraint; introduce `uses_profile` 1:N edge; relocate per-agent governance fields to either Agent-side columns or `agent_profile_override` table.
     - Refactor of `apply_agent_creation`, `upsert_agent_profile`, `get_agent_profile_for_agent`, `in_memory.rs` validation, `repo_impl.rs` upsert tx.
     - Decision about per-agent overrides placement.
     - Data migration to re-key existing per-agent profile rows.
   - **Scope-defer rationale**: redesign requires concept-doc amendment first; baby-phi's M5 scope is to align code with current concepts (M5.1 governing principle: "concepts = source of truth, code aligns to them"). Concept re-evaluation is a separate workflow that should not be conflated with drift remediation.
   - **Target**: M6 plan-open (when "Memory contract + Memory operations" lands and the data model is being revisited anyway), or as a standalone concept re-evaluation chunk before then if the user prioritises.
   - **Provenance**: CH-01 plan [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](./ch-01-agent-durable-lifecycle-2aa37c80.md) §P1 conceptual context (R1 sub-section).

**Tests.** No tests; pure docs work.

**Concept-alignment check.** N/A — process docs are not concept docs.

**phi-core leverage check.** N/A — no code.

**K8s readiness check.** P0 itself introduces no code. The rule it codifies is documented in §3.B above and concludes K8s-neutral for CH-01.

**Confidence target.** ≥ 99% (process ratification — must be unambiguous).

**Pause discipline.** If the user has additional locations to update beyond the four listed (e.g., `chunk-lifecycle-checklist.md` or `drift-lifecycle.md`), pause via `AskUserQuestion`.

---

### P1 — Migration 0007 + Agent struct extension (~0.5d)

#### Conceptual context: why the new fields belong on `domain::Agent` and not on a phi-core type

This sub-section is purely explanatory — it walks through the layered architecture so a reader unfamiliar with the boundary can verify by inspection that no phi-core leverage is being missed. The technical evidence is in §3 above; this is the plain-language version.

**The two-layer model.** baby-phi and phi-core are two software layers with different responsibilities:

- **phi-core is the engine.** Given fuel (prompts), a manual (system prompt), and a key (model config), it runs and emits exhaust (events). It does not know who owns the car, when it was registered, whether the driver has a licence, or whether the car is impounded.
- **baby-phi is the registration authority + insurance + audit log.** It tracks who owns each car, who's licensed to drive it, whether the car is currently registered or archived, and records every trip in a durable log.

When a trip happens (an HTTP request says "launch a session for agent X"), baby-phi checks all the paperwork first, then hands phi-core a manual + a key and watches the dashboard while it runs.

**Three different "Agent" things — three different jobs.**

| Type | Layer | What it represents | Persisted? | Used by baby-phi today? |
|---|---|---|---|---|
| `domain::Agent` | baby-phi (`domain/src/model/nodes.rs`) | A *principal* in the system: identity, role, org membership, lifecycle | Yes — SurrealDB row in `agent` table | Yes (governance gating) |
| `phi_core::Agent` (trait) | phi-core (`src/agents/agent.rs`) | A *runtime contract*: "anything that can be prompted and continued" | No — trait, has no state | **No** |
| `phi_core::BasicAgent` (struct) | phi-core (`src/agents/basic_agent.rs`) | A concrete in-memory implementation of the trait, for callers who want a stateful long-lived chat-style wrapper | No — in-memory only | **No** |
| `phi_core::agents::profile::AgentProfile` | phi-core (`src/agents/profile.rs`) | An execution *blueprint*: system prompt, model, temperature, tools | No (caller persists if needed) | Yes — wrapped as `domain::AgentProfile.blueprint` (CH-02) |
| `phi_core::types::context::AgentContext` | phi-core (`src/types/context.rs`) | The in-memory accumulator for one loop run: messages, agent_id (string), session_id, loop_id | No — ephemeral | Yes — built by `sessions/provider.rs::build_agent_context` |
| `phi_core::agent_loop` (free fn) | phi-core (`src/agent_loop/mod.rs`) | The canonical execution primitive: drives one loop run end-to-end | N/A | Yes — invoked at `sessions/launch.rs::spawn_agent_task` (CH-02) |

**How the principal (`domain::Agent`) connects to `AgentProfile`, `Session`, and `AgentContext`.**

The principal is the identity anchor; everything else hangs off of it through three distinct relationships:

***(R1) `domain::Agent` ↔ `domain::AgentProfile` — 1-to-1 by concept design (profile-as-genetics).***

```
domain::Agent {                 domain::AgentProfile {
  id: AgentId         ─────FK───── agent_id: AgentId       (UNIQUE INDEX)
  kind, role, ...                  parallelize, model_config_id,
  active, archived_at              mock_response, created_at,
}                                  blueprint: phi_core::AgentProfile  ← phi-core wrap (CH-02)
                                }
```

**The 1:1 cardinality is concept-mandated**, not an implementation default — the evidence:

- [`ontology.md`](docs/specs/v0/concepts/ontology.md) §"Edges" line 92 lists the relationship as **`Agent | HAS_PROFILE | AgentProfile | 1:1 | Blueprint identity`** explicitly.
- [`agent.md`](docs/specs/v0/concepts/agent.md) §"Soul" (lines 157–169): *"The Soul is the agent's **genetics** — defined at creation, never mutated... If you need to change an agent's fundamental nature, you create a new agent."* The profile is part of the agent's genetics in the concept model — owned, not borrowed from a shared pool.
- Schema enforcement: [`migrations/0001_initial.surql`](modules/crates/store/migrations/0001_initial.surql) line 131 — `DEFINE INDEX agent_profile_agent_id ON agent_profile FIELDS agent_id UNIQUE;`.
- Code enforcement: [`domain::AgentProfile`](modules/crates/domain/src/model/nodes.rs#L296) carries non-optional `agent_id: AgentId`; [`apply_agent_creation`](modules/crates/domain/src/repository.rs#L287) validates `profile.agent_id == agent.id`; [`upsert_agent_profile`](modules/crates/store/src/repo_impl.rs#L411) explicitly preserves the 1:1 invariant by deleting any conflicting prior row in the same transaction.
- Drift catalogue: [D4.4](docs/specs/v0/implementation/m5_1/drifts/D4.4.md) confirms the 1:1 invariant; no drift currently questions it.

**Why baby-phi chose this over a template-sharing model.** Profile-as-genetics aligns with three governance properties baby-phi wants:

1. **Per-agent governance fields naturally belong on the profile row.** `parallelize` (max concurrent loops) is per-agent (agents have different capacities); `model_config_id` is per-agent (each agent is assigned its own approved model from the org's model roster); `mock_response` is per-agent test configuration. Sharing one profile row across multiple agents would require relocating these fields to a per-agent override layer, adding indirection.
2. **Audit + provenance are simpler.** When an agent's profile changes, the audit log records *the agent's* profile changing — not "profile X changed, which is now seen differently by 5 agents". 1:1 keeps the change-source unambiguous.
3. **Org-level isolation.** Each agent's profile lives in the owning org's roster — orgs control their agents' configurations independently without cross-org coordination.

**Template-sharing already exists at the code-definition level.** [`system-agents.md`](docs/specs/v0/concepts/system-agents.md) §"Templates" — system agents (memory-extraction, agent-catalog) are instantiated FROM code-level template definitions (`profile_ref: system-memory-extraction`), but each org gets its own freshly-instantiated profile *row*. The "template" is shared as code; the "instance" (the row) is unique per agent. So template-sharing exists; it just operates at instantiation time, not at row-share time.

**The `agent.md` §"Parallelized Sessions" line "5 agents sharing a profile have 5 × parallelize total concurrent sessions"** is logical sharing (5 agents with similar profiles holding the same `parallelize` value) — not row sharing. The very next line clarifies *"the `parallelize` value is set on the AgentProfile in the owning org's agent roster"* (each agent has its own roster entry).

**Could baby-phi adopt true row-sharing template-pattern in a future chunk?** Yes, technically — phi-core itself is agnostic (its `AgentProfile` is a serialisable struct with no `agent_id` field; phi-core wouldn't break). The redesign would require:
- Concept-doc amendment to `agent.md` §"Soul" + `ontology.md` cardinality line.
- A new ADR documenting the redesign rationale + migration plan.
- Schema migration: drop the UNIQUE constraint; introduce a `uses_profile: FROM agent TO agent_profile` edge with 1:N cardinality; relocate per-agent governance fields to either an Agent-side column or a separate `agent_profile_override` table.
- Refactor of `apply_agent_creation`, `upsert_agent_profile`, `get_agent_profile_for_agent`, validation logic in `in_memory.rs` and `repo_impl.rs`.
- Decision about per-agent overrides (where do `parallelize`, `model_config_id`, `mock_response` live in a shared-profile world?).

**This redesign is out-of-scope for CH-01.** CH-01 ships only the two governance lifecycle fields (`active`, `archived_at`); the 1:1 ↔ N-to-1 redesign is its own concept-amendment + chunk-plan candidate. **If the user wants to pursue this**, the right next move is: (a) draft a forward-scope §3 entry as a candidate future chunk (M6+ scope), and (b) at a minimum, file a drift/scope marker in the M5.1 drift catalogue tagging it as a known-design-question. Neither lands inside CH-01; CH-01 references the 1:1 invariant as-is.

**Lifecycle linkage with CH-01 fields:** when `agent.archived_at = Some(t)`, the agent's profile row remains in the database (audit trail preserved) but session-launch is gated upstream by the `active`/`archived_at` checks at step 1 of the launch flow. The agent's `active = false` flag similarly gates without deleting the profile row. The 1:1 invariant is unchanged.

***(R2) `domain::Agent` ↔ `Session` — 1-to-many run history ("sessions THIS agent has run").***

```
domain::Agent ─────┐
                   │ STARTED_BY (1-to-many)
                   ▼
              Session {
                id: SessionId,
                started_by: AgentId,        ← back-reference to principal
                project_id: ProjectId,
                phi_core_session_id: String, ← UUID propagated to phi_core::Session
                status: SessionStatus,
                ...
              }
```

Each `Session` carries `started_by: AgentId` — the agent who launched it. One agent can launch many sessions over time. The `phi_core_session_id` (a string UUID) flows down to `phi_core::session::Session.id` and `AgentContext.session_id` for end-to-end traceability across emitted events. **Lifecycle linkage with CH-01:** an archived agent (`archived_at = Some`) cannot launch new sessions (gating logic enforced at session-launch step 1; explicitly planned for CH-15 enforcement, with the data source landing here in CH-01). Past sessions remain queryable for audit regardless of archive state.

***(R3) `domain::Agent` ↔ `phi_core::AgentContext` — ephemeral built-from ("the runtime snapshot for ONE loop run").***

```
At session launch — sessions/provider.rs::build_agent_context:

  domain::Agent       domain::AgentProfile.blueprint       launched Session + LoopRecord
       │                       │                                     │
       │ id.to_string()        │ system_prompt.clone()               │ id.to_string()
       ▼                       ▼                                     ▼
  phi_core::types::context::AgentContext {
      system_prompt: ...,        ← from AgentProfile.blueprint.system_prompt
      agent_id:    Some(...),    ← from domain::Agent.id (R1+R3 ID propagation)
      session_id:  Some(...),    ← from launched Session.id
      loop_id:     ...,          ← from launched LoopRecord.id
      messages:    vec![],       ← starts empty; phi-core's loop appends as the run proceeds
      ..AgentContext::default()
  }
```

`AgentContext` is **not persisted** — it's an in-memory accumulator scoped to one `agent_loop()` invocation. It's constructed at launch by combining state from the principal (`Agent.id`), the blueprint (`AgentProfile.blueprint.system_prompt`), and the just-created Session/LoopRecord (their UUIDs). After the loop ends, `AgentContext` is dropped; the persisted artefacts are the `Turn` rows recorded by `BabyPhiSessionRecorder` (which read from phi-core's `AgentEvent` stream).

**Summary table — what connects to the principal, how, and where it lives:**

| Type | Relationship to `domain::Agent` | Persisted? | Direction of FK / wrap |
|---|---|---|---|
| `domain::AgentProfile` | **R1** — 1-to-1 ownership | Yes (DB row) | Profile carries `agent_id` FK back to Agent |
| `phi_core::AgentProfile` (the inner) | 1-to-1 (via wrap field of R1) | Persisted indirectly inside `domain::AgentProfile` | `domain::AgentProfile.blueprint` is the wrap field; phi-core sees only the inner struct at runtime |
| `Session` (baby-phi side) | **R2** — 1-to-many run history | Yes (DB row) | Session carries `started_by: AgentId` |
| `phi_core::session::Session` (runtime side) | Wrapped by `BabyPhiSessionRecorder` | Recorder persists `Turn` rows | Receives `agent_id` (string) from Agent at session launch |
| `domain::LoopRecord` | 1-to-many under each Session | Yes (DB row) | Session 1→N LoopRecord; LoopRecord carries `session_id` FK |
| `domain::Turn` | 1-to-many under each LoopRecord | Yes (DB row) | LoopRecord 1→N Turn; Turn materialised from phi-core's `TurnStart`/`TurnEnd` events |
| `phi_core::AgentContext` | **R3** — ephemeral built-from | No (in-memory only) | Constructed at launch from `Agent.id` + `AgentProfile.blueprint` + launched `Session`/`LoopRecord` |
| `domain::ModelConfig` | Referenced by `AgentProfile.model_config_id` | Yes (DB row) | Profile holds the FK; resolved at session launch into `phi_core::provider::model::ModelConfig` for `AgentLoopConfig` |

**The pattern:** the principal is the persistent identity anchor; AgentProfile (R1) defines HOW the agent runs; Session+LoopRecord+Turn (R2 chain) record WHAT happened during runs; AgentContext (R3) is the ephemeral runtime projection that exists only during one run. All of them reference the agent through ID propagation, but only AgentProfile is a structural "owned-by" relationship — Session is "initiated-by" and AgentContext is "built-from-a-snapshot-of."

---

**Why does baby-phi NOT use the `Agent` trait or `BasicAgent`?** Because phi-core's `agent_loop()` is a free function — it does not require the caller to implement or instantiate the `Agent` trait. The trait + `BasicAgent` exist for callers who want to wrap an agent in a stateful long-lived in-memory object (e.g., a CLI chat REPL where state lives in process memory across user inputs). **baby-phi's architecture is per-request stateless**: every session is a fresh `agent_loop` invocation; nothing lives in process memory between requests; all state is persisted to SurrealDB via `BabyPhiSessionRecorder`. So we don't need the wrapper.

**Where do `AgentProfile`, model configs, and the rest connect to `domain::Agent`?** Through the session-launch flow at runtime:

```
HTTP POST /sessions { agent_id, prompt, ... }
                │
                ▼
[Step 1] domain::Agent loaded: repo.get_agent(agent_id) → domain::Agent
         ─── governance checks: agent.active? agent.archived_at None? agent.role allows launch?
         ─── (CH-01 adds active + archived_at checks at this step — gating phi-core invocation)
                │
                ▼
[Step 2] domain::AgentProfile loaded: repo.get_agent_profile_for_agent(agent_id) → domain::AgentProfile
         ─── governance checks: parallelize cap? model_config_id approved?
         ─── inside: profile.blueprint = phi_core::agents::profile::AgentProfile (the wrap from CH-02)
                │
                ▼
[Step 3] domain::ModelConfig resolved: repo.get_model_config(profile.model_config_id)
         ─── derives phi_core::provider::model::ModelConfig (the runtime config)
                │
                ▼
[Step 4] sessions/provider.rs::build_agent_context(launch_ctx, profile) builds the runtime input:
         ─── AgentContext {
         ─────  system_prompt: profile.blueprint.system_prompt.clone(),  ← phi-core blueprint field
         ─────  agent_id: Some(launch_ctx.started_by.to_string()),       ← domain::AgentId, ID-only
         ─────  session_id: Some(launch_ctx.phi_core_session_id.clone()),
         ─────  loop_id: launch_ctx.first_loop_id.map(|id| id.to_string()),
         ─────  ..AgentContext::default()
         ───  }
                │
                ▼
[Step 5] AgentLoopConfig assembled with model_config + provider override (MockProvider at M5)
                │
                ▼
[Step 6] phi-core takes over: tokio::spawn(async move { phi_core::agent_loop(prompts, &mut ctx, &cfg, tx, cancel).await })
         ─── phi-core sees: prompts, AgentContext, AgentLoopConfig, event sender, cancel token.
         ─── phi-core does NOT see: domain::Agent. It only sees agent.id as a string for traceability.
                │
                ▼
[Step 7] phi-core emits AgentEvents through tx; BabyPhiSessionRecorder drains them into Turn rows.
```

**The four explicit connection points** between `domain::Agent` and phi-core types:

- **(a) Permission gating** at Step 1 — `domain::Agent.active`, `archived_at`, `role` checked **before** phi-core is invoked. The CH-01 fields land here. This is governance saying "should we even start the engine?"
- **(b) ID propagation** at Step 4 — `domain::Agent.id.to_string()` becomes `AgentContext.agent_id`. Used by phi-core only for tagging emitted events with the originating agent ID. ID-only delegation; no struct reuse.
- **(c) Blueprint flow** at Step 4 — `domain::AgentProfile.blueprint.system_prompt` (phi-core wrap field) becomes `AgentContext.system_prompt`. The wrap is where phi-core's `AgentProfile` struct is reused.
- **(d) Model flow** at Step 3+5 — `domain::ModelConfig` resolves to `phi_core::provider::model::ModelConfig` for `AgentLoopConfig`. Another phi-core reuse point.

**Why no phi-core leverage opportunity for the CH-01 fields?**

The new fields (`active`, `archived_at`) are **gating fields** — their job is to allow or deny the path from governance → phi-core invocation (step 1 above). phi-core never reads them; it doesn't even know they exist. There's no phi-core type to wrap because phi-core's design intentionally does not carry permission or lifecycle metadata — engines don't track licences. If we put `active` on phi-core's `AgentProfile`, we'd be conflating two layers and pulling permission policy down into the engine.

**When would `phi_core::Agent` (the trait) become a leverage candidate?**

Only if a future feature introduces *long-lived in-memory chat agents* — e.g., an interactive REPL where state lives in process memory across HTTP requests instead of being persisted to SurrealDB on every turn. In that scenario we'd want `prompt_messages_with_sender` + `continue_loop_with_sender` as the public API of the in-memory wrapper. **No such feature is in the current roadmap** (M1–M7b), so the trait stays unused. ADR-0034 records this as a deliberate architectural choice and includes a review trigger if "live in-memory agents" ever lands.

#### End of conceptual context — P1 deliverables follow.

**Goal.** Add the durable lifecycle columns to the `agent` table and extend the domain struct with back-compat-safe serde defaults.

**Deliverables.**
1. **New migration** `modules/crates/store/migrations/0007_agent_active_archived.surql` — applies idempotently to existing `agent` table:
   ```sql
   -- 0007_agent_active_archived.surql
   -- CH-01 / ADR-0034 — Agent durable lifecycle.
   -- DEFAULT true so pre-CH-01 rows materialise active.
   -- archived_at is a string (RFC3339) per existing project convention
   -- (see grant.revoked_at, consent.revoked_at in 0001_initial.surql).
   DEFINE FIELD active ON agent TYPE bool DEFAULT true;
   DEFINE FIELD archived_at ON agent TYPE option<string>;
   ```
2. **Migration registry update** in [`modules/crates/store/src/migrations.rs`](modules/crates/store/src/migrations.rs) — add an `EmbeddedMigration { version: 7, slug: "agent_active_archived", surql: include_str!(...) }` entry following the existing pattern.
3. **Domain struct** at [`modules/crates/domain/src/model/nodes.rs:187`](modules/crates/domain/src/model/nodes.rs#L187) — extend `Agent`:
   ```rust
   pub struct Agent {
       pub id: AgentId,
       pub kind: AgentKind,
       pub display_name: String,
       pub owning_org: Option<OrgId>,
       #[serde(default)]
       pub role: Option<AgentRole>,
       pub created_at: DateTime<Utc>,
       /// CH-01: durable disable flag. Defaults to true for pre-CH-01 rows.
       /// Read by AgentCatalogListener (CH-22) and system-agent disable handler.
       #[serde(default = "default_agent_active")]
       pub active: bool,
       /// CH-01: archive timestamp. None = not archived. Set by system-agent
       /// archive handler. Pre-CH-01 rows deserialise as None.
       #[serde(default)]
       pub archived_at: Option<DateTime<Utc>>,
   }

   fn default_agent_active() -> bool { true }
   ```
4. **Update existing Agent constructors** that don't go through serde (e.g., `Agent::new(...)` if present) to set `active: true, archived_at: None`. Audit `nodes.rs` test fixtures + any test helper.
5. **Migration acceptance test** `modules/crates/store/tests/migrations_0007_test.rs` — new file, mirrors `migrations_0004_test.rs` shape:
   - `agent_active_defaults_true_for_inserted_rows`
   - `agent_archived_at_accepts_none_and_rfc3339_string`
   - `pre_ch01_rows_deserialise_with_active_true_and_archived_at_none`

**Tests.** +3 unit/integration tests at the migration layer + a `domain::nodes` serde round-trip test asserting both new fields. Workspace baseline 985 → ~989.

**Concept-alignment check.** `agent.md §Lifecycle` row transitions from `contradicted` to `honored` at the schema layer (P3 completes the runtime honor). Verified by reading the migration + struct.

**phi-core leverage check.** No phi-core surface touched. `check-phi-core-reuse.sh` green.

**Confidence target.** ≥ 97%.

**Pause discipline.** None anticipated.

---

### P2 — Repo methods (`set_agent_active` + `set_agent_archived_at`) (~0.5d)

**Goal.** Add the two trait methods + Surreal impl that flip the new columns.

**Deliverables.**
1. **Repository trait** at [`modules/crates/domain/src/repository.rs`](modules/crates/domain/src/repository.rs) (sibling to `create_agent`, `upsert_agent`, `get_agent`):
   ```rust
   /// CH-01 / ADR-0034 — flip the durable `active` flag on an existing agent.
   /// Returns `RepositoryError::AgentNotFound` if the agent doesn't exist.
   async fn set_agent_active(
       &self,
       agent_id: &AgentId,
       active: bool,
   ) -> RepositoryResult<()>;

   /// CH-01 / ADR-0034 — set `archived_at` to a timestamp (or clear it).
   /// `Some(t)` archives at time t; `None` un-archives.
   async fn set_agent_archived_at(
       &self,
       agent_id: &AgentId,
       archived_at: Option<DateTime<Utc>>,
   ) -> RepositoryResult<()>;
   ```
2. **Surreal impl** at [`modules/crates/store/src/repo_impl.rs`](modules/crates/store/src/repo_impl.rs) — both methods use `UPDATE type::thing('agent', $id) SET active = $active RETURN NONE` (or `SET archived_at = $archived_at`); error if the UPDATE returns 0 affected rows (use the existing pattern for "agent not found").
3. **Repo unit tests** in [`modules/crates/store/src/repo_impl.rs`](modules/crates/store/src/repo_impl.rs) `mod tests` (or a new `tests/repo_agent_lifecycle_test.rs` integration test):
   - `set_agent_active_flips_column_and_round_trips_through_get_agent`
   - `set_agent_archived_at_writes_rfc3339_and_round_trips`
   - `set_agent_active_returns_not_found_for_missing_agent`
   - `set_agent_archived_at_with_none_clears_column`

**Tests.** +4 repo tests. Baseline ~989 → ~993.

**Concept-alignment check.** No transitions; P2 is repo-layer plumbing.

**phi-core leverage check.** No new phi-core imports. `check-phi-core-reuse.sh` green.

**Confidence target.** ≥ 97%.

**Pause discipline.** None anticipated.

---

### P3 — Wire system-agent disable + archive handlers to flip durable state (~0.5d)

**Goal.** The two existing handlers ([`disable.rs`](modules/crates/server/src/platform/system_agents/disable.rs) and [`archive.rs`](modules/crates/server/src/platform/system_agents/archive.rs)) currently emit audit events but do not persist state. Wire them to call the P2 repo methods so disable/archive becomes durable.

**Deliverables.**
1. **`platform/system_agents/disable.rs`** — after current validation + before audit emit, call `repo.set_agent_active(&agent_id, false).await?`. If the call fails, surface the error via the existing `AgentError` variants. Order matters: write durable state FIRST so that if audit emit fails the durable state is still consistent (audit is replayable; durable state is not).
2. **`platform/system_agents/archive.rs`** — same shape: call `repo.set_agent_archived_at(&agent_id, Some(Utc::now())).await?` before audit emit. Standard-agent rejection (memory-extraction, agent-catalog) stays untouched (it fires earlier in the handler).
3. **Update existing acceptance tests** in [`modules/crates/server/tests/acceptance_system_agents.rs`](modules/crates/server/tests/acceptance_system_agents.rs) — the disable + archive happy-path tests must now assert that the `Agent` row's `active` / `archived_at` columns reflect the call (use `repo.get_agent` round-trip). Verify by extending existing tests rather than adding new ones unless the existing tests don't cover the right path.
4. **Audit-event payload** — confirm the existing audit events ([`platform::audit_events`](modules/crates/server/src/platform/system_agents/audit_events.rs)) carry enough info that operators can correlate with the durable flip (no schema change expected; just a code-review check).

**Tests.** No new tests — modify ~2 existing tests to assert durable-state flip. Baseline ~993 stays ~993.

**Concept-alignment check.** `system-agents.md §"Operator can disable"` row flips from `partially-honored` to `honored` at runtime. `system-agents.md §"Archive flow"` row flips from `partially-honored` to `honored`. Verified by the modified acceptance tests.

**phi-core leverage check.** No new phi-core imports.

**Confidence target.** ≥ 97%.

**Pause discipline.** If the modified handler breaks any other acceptance test (e.g., a test asserting that disable was a no-op for the agent row), pause via `AskUserQuestion` — that would indicate a hidden invariant we missed.

---

### P4 — Role-immutability acceptance test + concept-doc refresh (~0.25d)

**Goal.** Pin D-new-22's existing handler-layer enforcement with an explicit acceptance test, and update the relevant concept doc verified-headers.

**Deliverables.**
1. **Role-immutability acceptance test** in [`modules/crates/server/tests/acceptance_agents_profile.rs`](modules/crates/server/tests/acceptance_agents_profile.rs) — new test `update_rejects_role_change_with_immutable_field_changed`. Creates an agent with role `Member`, fires a PATCH request attempting to set role to `Admin`, asserts HTTP 400 + error code `ImmutableFieldChanged("role")`. Mirrors any existing "ImmutableField" test shape if present.
2. **Concept-doc verified-header bump** at:
   - [`docs/specs/v0/concepts/agent.md`](docs/specs/v0/concepts/agent.md) — bump `Last verified` header to today's date.
   - [`docs/specs/v0/concepts/system-agents.md`](docs/specs/v0/concepts/system-agents.md) — bump header.
   - [`docs/specs/v0/concepts/human-agent.md`](docs/specs/v0/concepts/human-agent.md) — bump header (re-read confirms the §"No Identity" claim is still accurate; nothing to refresh in body).
3. **`agent.md` body refresh** — if §"Lifecycle" or §"Roles" doesn't already explicitly mention `active` + `archived_at` columns as durable state, add a 1-paragraph note (does not change the claim; clarifies the persistence form). Skip if already explicit.

**Tests.** +1 acceptance test. Baseline ~993 → ~994.

**Concept-alignment check.** `agent.md §"Roles"` row stays at `honored` and is now ratified by an acceptance test (not just inspection of `update.rs`).

**phi-core leverage check.** No new phi-core imports.

**Confidence target.** ≥ 97%.

**Pause discipline.** If `update.rs` is found to have a hole (e.g., role can be changed via a different route or via `apply_agent_creation` re-issue), pause and surface — that's a new HIGH drift outside CH-01's plan.

---

### P5 — ADR-0034 Accepted + drift lifecycle + audit + seal (~0.25d)

**Goal.** Flip terminal governance state.

**Deliverables.**
1. **ADR-0034** [`docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md`](docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md) — Status `Proposed` → `Accepted`. Body includes:
   - Context (D6.5 + downstream CH-22 dependency + the user's plan-time challenge re: phi-core integration boundary, which surfaced the doc-gap closures shipped in P0).
   - Decision sub-decisions:
     - **D34.1** `active: bool DEFAULT true` column on `agent` table (migration 0007).
     - **D34.2** `archived_at: option<datetime>` column.
     - **D34.3** repo methods `set_agent_active` + `set_agent_archived_at`.
     - **D34.4** system-agent disable/archive handlers flip durable state before audit emit.
     - **D34.5** conforming criteria for CH-22: *"AgentCatalogListener body MUST consult `Agent.active` (via `repo.get_agent`) when computing `SystemAgentRuntimeStatus.is_paused`. The audit log is NOT the source of truth for current state."*
     - **D34.6** governance-vs-runtime boundary — record (in permanent ADR form) why `domain::Agent` does not wrap or implement `phi_core::Agent` (trait) or `BasicAgent` (impl). Reasoning: baby-phi's architecture is per-request stateless (every session = a fresh `agent_loop` invocation; nothing lives in process memory between requests; all state lives in SurrealDB). The `Agent` trait + `BasicAgent` exist for callers who want a stateful long-lived in-memory wrapper. baby-phi has no such caller today. Connection between `domain::Agent` and phi-core is ID-only at `sessions/provider.rs::build_agent_context` — `domain::AgentId.to_string()` flows into `AgentContext.agent_id: Option<String>`. **Review trigger:** any future milestone introducing long-lived in-memory chat agents (e.g., an interactive REPL with state living in process memory across HTTP requests) would re-evaluate phi-core trait adoption at that point.
   - Ratification evidence (table of code locations + test names).
   - Consequences (positive: durable state replaces inferred state; durable governance/runtime separation prevents accidental conflation; negative: pre-CH-01 rows need migration sanity check on first read; neutral: schema grows by 2 columns).
   - Review trigger (CH-22 plan-open + any future "live in-memory agent" feature).
2. **D6.5 drift file** [`drifts/D6.5.md`](docs/specs/v0/implementation/m5_1/drifts/D6.5.md) — Status `scoped` → `remediated`; lifecycle entry:
   `2026-04-27 — remediated — via CH-01 (plan ch-01-agent-durable-lifecycle-2aa37c80.md); migration 0007 added active + archived_at, system-agent disable/archive handlers wired to flip durable state, ADR-0034 Accepted`. `Last verified` bumped.
3. **D-new-22 drift file** [`drifts/D-new-22.md`](docs/specs/v0/implementation/m5_1/drifts/D-new-22.md) — Status `scoped` → `remediated`; lifecycle entry mentions both the existing `update.rs:133-141` enforcement and the new acceptance test as the closure ratification. `Last verified` bumped.
4. **D-new-23 drift file** [`drifts/D-new-23.md`](docs/specs/v0/implementation/m5_1/drifts/D-new-23.md) — Status stays `scoped`. Append lifecycle entry:
   `2026-04-27 — review at CH-01 — confirmed scope deferred-to-CH-16 because Identity has no writers at M5; no code action this chunk; CH-16 owns final closure`. `Last verified` bumped.
5. **`_concept-audit-matrix.md`** — flip rows for `agent.md §Lifecycle`, `system-agents.md §Operator-can-disable`, `system-agents.md §Archive-flow` from `contradicted`/`partially-honored` to `honored`. Update Code-evidence column with `migrations/0007_*.surql` and the wired handler line numbers.
6. **`drifts/README.md`** — refresh D6.5, D-new-22 Status columns to `remediated`; D-new-23 stays `scoped`.
7. **Spawn 2 audit agents** (see §11). Block seal until both report PASS.

**Tests.** No new tests. P5 is governance only.

**Concept-alignment check.** All §2 rows at target-status.

**phi-core leverage check.** Final green sweep — `check-phi-core-reuse.sh` exit 0; forbidden-duplication greps all 0; positive greps all ≥ 1.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** Audit findings → surface to user before seal.

---

## §8 — Tests summary

- **Expected total test count at chunk close:** 985 (post-CH-K8S-PREP baseline) + 8 new tests = **~993** serialised passing.
  - +0 P0 (process docs only)
  - +3 P1 (migration 0007 tests + serde round-trip)
  - +4 P2 (repo round-trip tests)
  - +0 P3 (modify existing acceptance tests in place)
  - +1 P4 (role-immutability acceptance test)
- **Layer breakdown:**
  - Unit / integration (store crate migration + repo impl): +7
  - Acceptance (server/tests): +1, modified ~2
  - e2e: 0
- **New test files:**
  - `modules/crates/store/tests/migrations_0007_test.rs`
  - Possibly `modules/crates/store/tests/repo_agent_lifecycle_test.rs` (or co-located inline `#[cfg(test)] mod tests` in `repo_impl.rs`; final placement decided at P2 implementation time).
- **Expected-still-green fragile tests:**
  - `acceptance_system_agents.rs` — disable / archive happy-path tests (modified at P3 to assert durable flip; must still pass alongside the non-modified rejection-path tests).
  - `migrations_0004_test.rs` — ensures pre-existing `role` column behaviour intact (no regression from adding 0007).
  - `acceptance_agents_profile.rs` — entire suite (the new P4 test should not interfere with existing tests).

---

## §9 — Pre-chunk gate

**Reading list (drafter reads before chunk-open ritual completes):**
1. Concept docs: [`agent.md`](docs/specs/v0/concepts/agent.md), [`system-agents.md`](docs/specs/v0/concepts/system-agents.md), [`human-agent.md`](docs/specs/v0/concepts/human-agent.md).
2. Drift files: [`D6.5.md`](docs/specs/v0/implementation/m5_1/drifts/D6.5.md), [`D-new-22.md`](docs/specs/v0/implementation/m5_1/drifts/D-new-22.md), [`D-new-23.md`](docs/specs/v0/implementation/m5_1/drifts/D-new-23.md).
3. Process: [`per-chunk-planning-template.md`](docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md), [`chunk-lifecycle-checklist.md`](docs/specs/v0/implementation/m5_1/process/chunk-lifecycle-checklist.md), [`drift-lifecycle.md`](docs/specs/v0/implementation/m5_1/process/drift-lifecycle.md).
4. Forward-scope: [`22035b2a-remaining-scope-post-m5-p7.md`](docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §1 CH-01 + §7 Q&A.
5. [`baby-phi/CLAUDE.md`](CLAUDE.md) §phi-core Leverage rules 1–5 + §"Orthogonal surfaces".
6. Sibling chunk plan: [`ch-02-real-agent-loop-wiring-16fd9a3a.md`](docs/specs/plan/build/ch-02-real-agent-loop-wiring-16fd9a3a.md) for style + structure reference.
7. Existing migration patterns: [`0001_initial.surql`](modules/crates/store/migrations/0001_initial.surql), [`0004_agents_projects.surql`](modules/crates/store/migrations/0004_agents_projects.surql), [`0006_agent_profile_mock_response.surql`](modules/crates/store/migrations/0006_agent_profile_mock_response.surql).
8. Existing handlers: [`platform/system_agents/disable.rs`](modules/crates/server/src/platform/system_agents/disable.rs), [`archive.rs`](modules/crates/server/src/platform/system_agents/archive.rs), [`platform/agents/update.rs`](modules/crates/server/src/platform/agents/update.rs).
9. K8s-readiness inputs (for §3.B + P0 ratification): [`m7b/architecture/k8s-microservices-readiness.md`](docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md), [`deferred-from-ch-k8s-prep.md`](docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md), [`m5_2/decisions/0033-k8s-prep-refactors.md`](docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) (D33.1–D33.4 conforming criteria).

**Carry-forward invariants** (verified green at chunk-open):
- `cargo test --workspace -- --test-threads=1` = 985 (post-CH-K8S-PREP baseline).
- 4 CI guards green.
- `git diff --stat HEAD -- modules/` empty.
- Highest applied migration is 0006.

**Pending decisions carried into this chunk:**
- Forward-scope Q5 (M5 scope): CH-01 closes a HIGH drift (D6.5) → must close before M5 tag. D-new-22 (MEDIUM) closes opportunistically since enforcement is already shipped. D-new-23 (LOW) stays scoped to CH-16.
- Forward-scope Q4 (chunk ordering): user selected CH-01 as the next chunk after CH-K8S-PREP seal. CH-01 enables CH-22 per the dependency graph.

**Chunk-ordering note.** No predecessor in §6 has hard dependencies on CH-01's pre-state; the carry-forward invariants are the only gating items.

---

## §10 — Close criteria

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — `/root/rust-env/cargo/bin/cargo test --workspace -- --test-threads=1` green at ~993; clippy green under `RUSTFLAGS="-Dwarnings"`; `cargo fmt --all -- --check` green; migration 0007 idempotent (re-running the runner does not error).
- **Docs aspect** — D6.5 + D-new-22 Status flipped to `remediated`; D-new-23 lifecycle entry appended (Status stays `scoped`); `_concept-audit-matrix.md` rows for `agent.md §Lifecycle` + `system-agents.md §disable` + `system-agents.md §archive` flipped to `honored`; `drifts/README.md` index current; ADR-0034 Accepted; `agent.md` / `system-agents.md` / `human-agent.md` verified-headers bumped.
- **phi-core leverage aspect** — import-count delta = **0**; all positive greps (§3) ≥ 1; all forbidden-duplication greps (§3) = 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row at target-status; no row remains `contradicted`. `human-agent.md §No Identity` stays `silent-in-code` (target = `silent-in-code` per scope-defer to CH-16) — this is honored as-stated.
- **K8s readiness aspect** *(new — codified by P0)* — §3.B 7-axis table populated with conclusions; no new CHK8S-D-XX entries needed for CH-01 (K8s-neutral); `per-chunk-planning-template.md` §3.B section landed; forward-scope §7 Q8 added; m7b-readiness §11 cross-link added; deferred-items ledger guidance updated.

**Two confidence % (named numerator/denominator):**
- **Implementation confidence** = `claims-verified-honored-by-tests-and-code-inspection / claims-in-scope-for-chunk` = target **5/5 = 100%**. The 5 claims:
  1. agent.md §Roles role-immutability rejected at handler (acceptance test pinned)
  2. agent.md §Lifecycle `active` column persists durably
  3. agent.md §Lifecycle `archived_at` column persists durably
  4. system-agents.md §Disable handler flips `active = false`
  5. system-agents.md §Archive handler writes `archived_at`
- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity / doc-pages-touched-in-chunk` = target **14/14 = 100%**.

Touched doc pages (denominator):
1. ADR-0034 (`m5_2/decisions/0034-agent-durable-lifecycle.md`)
2. D6.5 drift
3. D-new-22 drift
4. D-new-23 drift (lifecycle append)
5. `_concept-audit-matrix.md`
6. `drifts/README.md`
7. `agent.md` header
8. `system-agents.md` header
9. `per-chunk-planning-template.md` (P0 §3.B addition)
10. `forward-scope/22035b2a-...md` §7 (P0 Q8 append)
11. `m7b/architecture/k8s-microservices-readiness.md` (P0 §11 append)
12. `m7b/architecture/deferred-from-ch-k8s-prep.md` (P0 guidance update)
13. `baby-phi/CLAUDE.md` (P0 §"Orthogonal surfaces" append for `domain::Agent`)
14. `phi-core-mapping.md` (P0 agents/ table refinement + §"Connection point" addition)

(`human-agent.md` is also bumped but does not count as a doc-confidence subject because no claim transitions there.)

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-readiness-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P5 seal phase specifically. Composite below target blocks close. No aspect-averaging, no rounding up.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 6 phases (P0–P5) = medium chunk → **2 agents** (per per-chunk-planning-template.md guardrail: ≤3 phases = 1 agent, 4–6 = 2 agents, 7+ = 3 agents).

**Audit aspects (a–e):**
- (a) Code correctness (P1 + P2 + P3 land cleanly; tests pass; migration idempotent).
- (b) Docs fidelity vs concept docs (ADR-0034 ratification; drift lifecycle entries correct; matrix flipped).
- (c) Concept alignment (`agent.md`, `system-agents.md`, `human-agent.md` claims honored or scope-deferred as stated).
- (d) phi-core leverage (import-count delta = 0; no struct duplication; check-phi-core-reuse.sh green).
- (e) **K8s readiness rule landed** (P0 process-doc additions present + this plan's §3.B populated; no new CHK8S-D-XX entries needed since CH-01 is K8s-neutral).

**Auditor constraint.** Fresh `Explore` subagents. Neither may be the implementer.

### Audit Agent A — Code + phi-core leverage

> **Prompt** (locked at Step 2; fired at P5 seal):
> You are performing an independent code + phi-core leverage audit of CH-01 in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code.
>
> Verify these claims against the current HEAD code state. For each claim report PASS or FAIL with 1-line evidence:
>
> 1. `modules/crates/store/migrations/0007_agent_active_archived.surql` exists and contains `DEFINE FIELD active ON agent TYPE bool DEFAULT true` and `DEFINE FIELD archived_at ON agent TYPE option<string>` (or `option<datetime>` — note actual choice).
> 2. `modules/crates/store/src/migrations.rs` registers version 7 in `EMBEDDED_MIGRATIONS` with slug `agent_active_archived`.
> 3. `modules/crates/domain/src/model/nodes.rs` `pub struct Agent { ... }` declares `pub active: bool` with `#[serde(default = "default_agent_active")]` (or similar default-true mechanism) and `pub archived_at: Option<DateTime<Utc>>` with `#[serde(default)]`.
> 4. `modules/crates/domain/src/repository.rs` declares `async fn set_agent_active` and `async fn set_agent_archived_at` on the `Repository` trait.
> 5. `modules/crates/store/src/repo_impl.rs` implements both methods using `UPDATE type::thing('agent', $id) SET ...` patterns and returns `RepositoryError::AgentNotFound` (or equivalent) when the UPDATE affects 0 rows.
> 6. `modules/crates/server/src/platform/system_agents/disable.rs` calls `repo.set_agent_active(_, false).await?` BEFORE the audit emit.
> 7. `modules/crates/server/src/platform/system_agents/archive.rs` calls `repo.set_agent_archived_at(_, Some(Utc::now())).await?` BEFORE the audit emit.
> 8. `bash scripts/check-phi-core-reuse.sh` returns exit 0.
> 9. `grep -rn "^pub struct Agent\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"` returns 0 hits.
> 10. `cargo test --workspace -- --test-threads=1` test count is ~993 (≥ 985 baseline + 8 new tests; report actual).
> 11. Migration 0007 acceptance tests in `modules/crates/store/tests/migrations_0007_test.rs` pass: default-true, RFC3339 round-trip, pre-CH-01 row deserialise.
> 12. New acceptance test in `modules/crates/server/tests/acceptance_agents_profile.rs` named `update_rejects_role_change_with_immutable_field_changed` (or similar) passes and asserts HTTP 400 + error code `ImmutableFieldChanged("role")`.
> 13. Connection-point file `modules/crates/server/src/platform/sessions/provider.rs` `build_agent_context` is unchanged by CH-01 (no new fields wired into `AgentContext`); the function still passes `ctx.started_by.to_string()` as `agent_id` only.
>
> Report each as PASS/FAIL with 1-line evidence. ≤ 700 words. Read-only.

### Audit Agent B — Docs fidelity + drift lifecycle

> **Prompt** (locked at Step 2; fired at P5 seal):
> You are performing an independent docs audit of CH-01. You did NOT write these docs.
>
> For each claim report PASS or FAIL:
>
> 1. `docs/specs/v0/implementation/m5_1/drifts/D6.5.md` — Status = `remediated`; lifecycle history block has a chronological chain ending with `2026-04-27 — remediated — via CH-01 ...`; `Last verified` header bumped to today.
> 2. `docs/specs/v0/implementation/m5_1/drifts/D-new-22.md` — Status = `remediated`; lifecycle entry mentions both the existing `update.rs:133-141` enforcement and the new acceptance test as the closure ratification.
> 3. `docs/specs/v0/implementation/m5_1/drifts/D-new-23.md` — Status = `scoped` (UNCHANGED); lifecycle entry appended dated 2026-04-27 referencing CH-01 review + scope-defer to CH-16; `Last verified` bumped.
> 4. `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md` — rows for `agent.md §Lifecycle`, `system-agents.md §disable`, `system-agents.md §archive` flipped to `honored`; Code-evidence column updated to cite migration 0007 + handler line numbers.
> 5. `docs/specs/v0/implementation/m5_1/drifts/README.md` — D6.5 + D-new-22 Status columns show `remediated`; D-new-23 stays `scoped`.
> 6. `docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md` — Status = `Accepted`; body has Context / Decision (with sub-decisions D34.1–D34.5) / Consequences / Conforming-criteria-for-CH-22 / Ratification evidence / Review-trigger / References sections.
> 7. `docs/specs/v0/concepts/agent.md` and `system-agents.md` — `Last verified` headers bumped to today; if §"Lifecycle" / §"Operator can disable" body needed clarification re durable persistence, the change is present and accurate.
> 8. `docs/specs/v0/concepts/human-agent.md` — `Last verified` header bumped (no body change required).
> 9. **(P0 K8s readiness rule landed):** `docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md` contains a new sub-section `§3.B — K8s microservice readiness check` with the 7-axis table template (axes A1 in-process state · A2 IPC · A3 pod-local resource · A4 migration runner / first-apply race · A5 trait-shape requirement · A6 cross-pod state · A7 audit hash-chain symmetry) plus the rule that any new blocker creates a `CHK8S-D-XX` entry in the m7b ledger.
> 10. `docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §7 contains a new `Q8 — K8s microservice readiness rule` block whose decision binds future chunks to apply §3.B.
> 11. `docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md` has a new closing §11 titled "Per-chunk readiness check rule (CH-01+)" with cross-links to the per-chunk template + forward-scope Q8 + the deferred-items ledger.
> 12. `docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` "Adding new entries" guidance section explicitly names the per-chunk-template §3.B as the canonical entry-source from CH-01 onward; provenance field guidance updated to cite originating chunks.
> 13. All four P0 docs above had their `Last verified` headers bumped to today's date.
> 14. No new CHK8S-D-XX entries appear in the deferred-items ledger as a result of CH-01 (CH-01 is declared K8s-neutral in §3.B).
> 15. **(P0 doc-gap closures):** `baby-phi/CLAUDE.md` §"Orthogonal surfaces that are NOT phi-core duplicates" contains a new bullet for `domain::Agent` vs `phi_core::Agent` (trait) + `BasicAgent` (runtime impl), naming the connection point at `sessions/provider.rs::build_agent_context` as ID-only delegation.
> 16. `docs/specs/v0/concepts/phi-core-mapping.md` agents/ table: `phi_core::Agent` (trait) row added with classification "Runtime-only"; `BasicAgent` row clarified as "Runtime-only impl of `phi_core::Agent` trait, not persisted by baby-phi"; a §"Connection point" or §"Agent governance vs runtime separation" sub-section names `provider.rs::build_agent_context` as the ID-only delegation site; `Last verified` header bumped.
> 17. **(Concept re-evaluation marker)** `docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §3 contains a new `M6+-OPEN-01 — AgentProfile cardinality re-evaluation (1:1 → N:1 template-sharing)` entry. The entry MUST include: a `Status: open question` line (distinguishing it from committed `DEFERRED-*` markers); origin/provenance citing CH-01 plan-review 2026-04-27; question being evaluated; argument-for + argument-against summary; required-if-pursued list (concept amendment, ADR, schema migration, code refactor); target milestone (M6 plan-open or standalone concept-reeval chunk).
>
> Report each as PASS/FAIL with 1-line evidence. ≤ 800 words. Read-only.

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
# Expect: 985 (CH-K8S-PREP baseline) + ~8 new tests ≈ 993

# 3. CH-01-specific positive greps
ls modules/crates/store/migrations/0007_*.surql                                            # 1
grep -n "version: 7" modules/crates/store/src/migrations.rs                                # ≥ 1
grep -n "pub active: bool" modules/crates/domain/src/model/nodes.rs                        # 1
grep -n "pub archived_at: Option<DateTime<Utc>>" modules/crates/domain/src/model/nodes.rs  # 1
grep -n "set_agent_active" modules/crates/domain/src/repository.rs                         # ≥ 1
grep -n "set_agent_archived_at" modules/crates/domain/src/repository.rs                    # ≥ 1
grep -n "set_agent_active" modules/crates/server/src/platform/system_agents/disable.rs     # ≥ 1
grep -n "set_agent_archived_at" modules/crates/server/src/platform/system_agents/archive.rs # ≥ 1

# 4. CH-01-specific negative greps (no struct duplication)
grep -rn "^pub struct Agent\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"       # 0
grep -rn "^pub enum AgentRole\b" modules/crates/ | grep -v "domain/src/model/nodes.rs"     # 0

# 5. Migration acceptance test
/root/rust-env/cargo/bin/cargo test -p store --test migrations_0007_test                   # all pass

# 6. Role-immutability acceptance test
/root/rust-env/cargo/bin/cargo test -p server --test acceptance_agents_profile -- update_rejects_role_change   # 1 pass

# 7. Drift-file status
grep -c "^- \*\*Status\*\*: \`remediated\`" docs/specs/v0/implementation/m5_1/drifts/D6.5.md      # 1
grep -c "^- \*\*Status\*\*: \`remediated\`" docs/specs/v0/implementation/m5_1/drifts/D-new-22.md  # 1
grep -c "^- \*\*Status\*\*: \`scoped\`" docs/specs/v0/implementation/m5_1/drifts/D-new-23.md      # 1 (UNCHANGED)

# 8. ADR status
grep -c "^\*\*Status: Accepted\*\*" docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md   # 1

# 9. Concept-audit matrix
grep -c "honored" docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md          # baseline + 3

# 10. P0 K8s readiness rule landed
grep -c "§3.B\|K8s microservice readiness check" docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md   # ≥ 1
grep -c "Q8 — K8s microservice readiness rule\|Q8 . K8s microservice readiness rule" docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md   # ≥ 1
grep -c "Per-chunk readiness check rule" docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md   # ≥ 1
grep -c "per-chunk-template §3.B\|per-chunk-planning-template §3.B" docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md   # ≥ 1

# 11. CH-01 introduced no new K8s blockers (ledger entry count unchanged)
grep -c "^### CHK8S-D-" docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md   # 8 (unchanged from CH-K8S-PREP)

# 12. P0 doc-gap closures landed
grep -c "domain::Agent" CLAUDE.md                                                                # ≥ 1 (new bullet under §Orthogonal surfaces)
grep -c "phi_core::Agent\|Runtime-only" docs/specs/v0/concepts/phi-core-mapping.md               # ≥ 2 (trait classification + connection-point note)
grep -c "build_agent_context\|Connection point" docs/specs/v0/concepts/phi-core-mapping.md       # ≥ 1 (connection-point cross-link)

# 13. Concept re-evaluation marker captured in forward-scope §3
grep -c "M6+-OPEN-01\|AgentProfile cardinality re-evaluation" docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md   # ≥ 1
grep -c "Status: open question\|Status.*open question" docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md           # ≥ 1 (distinguishes from DEFERRED markers)
```

---

## What this plan does NOT do

- **No general-agent (non-system) disable / archive endpoints.** Concept docs only mandate the lifecycle for system agents at M5; non-system agent lifecycle is out-of-scope for CH-01 and not currently in any chunk.
- **No Identity-creation guard for Human Agents.** Deferred to CH-16 (which lands the Identity writer surface). D-new-23 stays `scoped` with a CH-01 lifecycle entry confirming the review.
- **No CH-22 listener body wiring.** Reading `Agent.active` from the AgentCatalogListener is CH-22's job; CH-01 only ships the column + repo + handler-flip surface. Conforming criteria for CH-22 codified in ADR-0034.
- **No phi-core changes.** Agent is a baby-phi-only governance node per `phi-core-mapping.md`'s "orthogonal surfaces" list.
- **No new permission-engine work.** CH-01 does not touch grants, manifests, selectors, or actions.
- **No retroactive §3.B back-fill on CH-02 / CH-K8S-PREP plans.** Those chunks pre-date the rule and are grandfathered (CH-K8S-PREP itself originated the 7-axis evaluation in [ADR-0033](docs/specs/v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md)).
- **No `chunk-lifecycle-checklist.md` rewrite.** Step 2 of the existing checklist already says "verify §3 greps, walk leverage maps". §3.B inherits this Step 2 walk implicitly. If the user wants Step 2 explicitly extended, that's a P0 deliverable add — pause for `AskUserQuestion`.

---

## Notes on M5.1/P3 Q&A binding

This plan honors all 7 planning decisions from [forward-scope §7](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md):

- **Q1** (storage-backend) — untouched by CH-01; CH-03 owns.
- **Q2** (selector PEG split) — untouched; CH-06 owns.
- **Q3** (consent triad sequencing) — untouched; CH-09/10/11 own.
- **Q4** (chunk ordering) — user-selected CH-01 as the next chunk after CH-K8S-PREP seal; honored.
- **Q5** (M5 scope — HIGH-all-M5 + MEDIUM/LOW case-by-case) — CH-01 closes one HIGH (D6.5) and one MEDIUM (D-new-22) opportunistically since the latter's enforcement is already shipped. One LOW (D-new-23) stays `scoped` to CH-16. Honored.
- **Q6** (ADR numbering at draft time) — ADR-0034 claimed via the documented `ls … | grep -oE "^[0-9]{4}" | sort -u | tail -5` pattern; honored.
- **Q7** (uniform ExitPlanMode ritual) — this plan is being approved via ExitPlanMode; honored.
- **Q8** (K8s microservice readiness rule, **new — codified by P0 of this chunk**) — every chunk plan §3.B evaluates the 7 K8s-deployability axes; new blockers create CHK8S-D-XX entries in the m7b ledger; ADR-0033 D33.1–D33.4 conforming criteria checked. Origin: CH-K8S-PREP's 7-axis evaluation generalised. CH-01 is the first chunk to apply (and ratifies the rule into the per-chunk-planning-template + forward-scope §7 + m7b strategic doc + ledger guidance).
