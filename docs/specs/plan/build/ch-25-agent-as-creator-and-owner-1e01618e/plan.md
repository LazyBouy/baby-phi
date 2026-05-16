<!-- Last verified: 2026-05-15 by Claude Code (CH-25-1e01618e v2 re-plan after gate-1 user-lock divergent F1.b (NEW Owns variant on Edge enum) from planner-recommended F1.a (reuse OWNED_BY/CREATED + relax Resource trait); preserves all other 5 locks (F2.a/F3.a/F4.a/F5.a/F6.b) aligned. Drafted by chunk-planner v13. Applies v13 discipline rigorously: R1 closed-set audit-prompt verification, R2 source-map PRODUCTION-vs-TEST-FIXTURE classification + enum-string verification, R3 struct-placement dependency-direction verification, R4 drift M*-DEFERRED-NN allocation, R5 cross-cycle divergence prominent gate-1 callout, R6 §3.E anticipated gate-2.5 candidates, R7 forward-scope-vs-concept-doc precedence detection. v2 re-plan dated 2026-05-15.) -->

# CH-25 — Agent-as-creator-and-owner of Organization and Project + R5 permissions-audit skill fix

**Cycle hex**: `1e01618e`
**Forward-scope row**: [`plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 lines 242-254
**Chunk slug**: `ch-25-agent-as-creator-and-owner`
**Folder**: `baby-phi/docs/specs/plan/build/ch-25-agent-as-creator-and-owner-1e01618e/`
**Estimated effort**: ~3.5 engineer-days (3.0 core + 0.5 R5 investigation)
**Plan revision**: **v2** (re-plan after gate-1 user-lock — F1.b NEW `Owns` variant on `Edge` enum, divergent from planner-recommended F1.a)

---

## Forks for orchestrator

> ⚠️ **CROSS-CYCLE DIVERGENCE PATTERN**: planner-recommendation has diverged from user-lock in **6 of last 8 cycles** (CH-15 `c3f46f17` F5.B / CH-17 `40c4d759` F5.B / CH-18 `c77937bc` F3.B / CH-20 `240616a4` F1.B / CH-24 `5778bb77` F1.B + F-D59.2.b + F-D59.3.b — 3 within-cycle divergences / **CH-25 `1e01618e` F1.b — NEW Owns variant on Edge enum**). Lone non-diverger: CH-19 `2c520ba7`. Cumulative cross-cycle divergent forks **8-of-10 (80%)**. User systematically prefers tighter / more-fragmented / more-defensive / **more-explicit-and-visible** options at gate-1 (CH-25 F1.b confirms preference for explicit/visible edge semantics over re-use-existing structural cleanliness). **Treat divergence as the modal outcome, not the exception** when reviewing forks below. The user-pattern is now load-bearing across **6 cycles** of evidence; planner v13's divergence-aware framing was warranted + remains the default discipline going forward.

This chunk surfaced **6 forks** at v1 plan-draft time. All 6 are **LOCKED** at gate-1; F1 diverged from planner-recommendation (now refreshed below for posterity per v9 surfacing-not-suppressing approach).

### F1 — Owner-edge shape **[LOCKED AT GATE-1 (USER-DIVERGENT): F1.b]**

> **Locked at gate-1 (USER-DIVERGENT): F1.b — NEW `Edge::Owns` variant**. User rationale: explicit/visible edge semantics over structural cleanliness of re-using existing OWNED_BY/CREATED. The closed-set invariant `EDGE_KIND_NAMES.len() == 71` flips to `== 72`; the variant is a 1-of-72 dedicated Agent→Org/Project ownership semantic distinct from the generic `OwnedBy` (which stays Memory-as-Resource focused) and from `Created` (which carries provenance only).

The drift's remediation §1 sketches "Add `Owns` edge variant (or extend `Created` with new from/to type pair)". The §3.D pre-flight check surfaces that the v0 ontology **already has** generic `Edge::OwnedBy` + `Edge::Created` variants at `domain/src/model/edges.rs:347-356` with typed constructors `Edge::new_owned_by` + `Edge::new_created` accepting `&impl Principal` / `&impl Resource` arguments (`edges.rs:616-633`). Concept doc `core-philosophy.md:24` says *"Agents own Resources"* (generic OWNED_BY); `:23` says *"Agent create Projects"* (generic CREATED). The closed `EDGE_KIND_NAMES.len() == 71` invariant (test at `edges.rs:658`) AND the closed `Action::CANONICAL.len() == 34` invariant must hold.

Crucially: the drift cites `66 variants` (stale — actual is 71 per the canonical-name array). Critical pre-flight: `principal_resource.rs:182-186` states *"OrgId, UserId, ProjectId are Principals only — they are never the target of an OWNED_BY edge in the v0 ontology. If a future milestone needs org-owned-by-org or similar, we add `impl Resource for OrgId` at that time and note the relaxation here."* CH-25 IS that milestone — but the user-lock F1.b chose NOT to relax the Resource trait; instead it adds an explicit `Owns` variant whose payload carries the typed Agent + Org/Project IDs directly.

**Forward-scope literal text vs concept-doc invariant**: forward-scope §2.5 says *"New `Owns` edge variant in `EdgeKind` enum (or extended `Created`)"*. User-locked F1.b honors the forward-scope row's literal-form. The 71-variant invariant flips to 72; the §3.D re-interpretation note is REMOVED from ADR-0060 §D60.1 (the literal forward-scope wording is taken at face value rather than re-interpreted).

| Option | Approach | Cost | Impact on closed sets |
|---|---|---|---|
| **F1.a** (planner-recommended — historical record) | Re-use existing generic `Edge::OwnedBy` + `Edge::Created` variants by adding `impl Resource for OrgId` + `impl Resource for ProjectId` in `principal_resource.rs`; emit edges via existing `new_owned_by` / `new_created` typed constructors at 3 emit sites. | LOWER (2 lines in `principal_resource.rs` + 3-4 emit sites). | `EDGE_KIND_NAMES.len() == 71` PRESERVED. `Action::CANONICAL.len() == 34` PRESERVED. |
| **F1.b** ✅ **USER-LOCKED** | Add a NEW `Edge::Owns { id: EdgeId, from: AgentId, to: OwnedResourceId }` variant alongside existing OwnedBy/Created. `OwnedResourceId` is a NEW domain-tier enum `Org(OrgId) \| Project(ProjectId)` (per chunk-planner v13 R3 struct-placement: domain-tier enum, since trait return signatures cite it). Bumps closed set to 72 variants. Requires `EDGE_KIND_NAMES` extension + `tests::edge_kind_names_is_exactly_seventy_two` rename (3 test-cardinality literal sites). | HIGHER (full variant addition + canonical-name array entry + `Edge::name` arm + 71→72 cardinality flip on 4 literal-count test sites + new typed constructor `Edge::new_owns`). | `EDGE_KIND_NAMES.len() == 72` (invariant flipped, NOT broken — invariant test renamed + updated). |
| **F1.c** | EXTEND `Edge::Created` only (drift's §1 sub-option). | LOWEST. | Same as F1.a for closed sets, but only one edge type per resource. |

**Strict-reading planner recommendation was: F1.a** — preserved 71-variant invariant via Resource trait relaxation. User-lock chose F1.b for explicit-edge-semantics: a dedicated Agent→Org/Project ownership edge with typed payload (no NodeId erasure), distinct from the generic OWNED_BY (Memory ownership). This option is **structurally more explicit + more defensive against accidental misuse** — a strong fit for the cross-cycle user-preference pattern. Per chunk-planner v13 divergence-aware framing, F1.b was flagged at plan-draft as a credible user-lock target; the user-lock confirms.

### F2 — Owner-grant rule actions **[LOCKED AT GATE-1: F2.a]**

> **Locked at gate-1: F2.a — `[Action::Allocate, Action::Transfer]`** (planner-recommended).

Forward-scope §2.5 says *"auto-issue `[admin, transfer]` for the owner-Agent over the child Org/Project"*. The §3.D pre-flight check at `action.rs:31-82` + `:250-285` (`Action::CANONICAL.len() == 34`) surfaces: **`Action::Admin` does NOT exist in the canonical 34-verb set**. Only `Action::Allocate` + `Action::Transfer` are in the `Authority` category (`action.rs:341-345`).

**Forward-scope literal text vs concept-doc invariant**: per chunk-planner v8/v9 §3.D, the concept-doc wins. The `[admin, transfer]` literal text is scoping-gloss; the actual implementation issues canonical verbs.

| Option | Action set | Rationale |
|---|---|---|
| **F2.a** ✅ **USER-LOCKED** | `[Action::Allocate, Action::Transfer]` | Matches the canonical Authority category. `Allocate` is what CEO grant carries today (`claim.rs:241` + `orgs/create.rs:181`); `Transfer` enables the owner to move ownership. Forward-scope's `[admin]` re-interprets to `[allocate]` because `Allocate` IS the org-control-plane verb. |
| **F2.b** | `[Action::Allocate]` only (no Transfer at v0) | Tighter. |
| **F2.c** | `[Action::Wildcard]` | Privileged escape hatch. |

**§3.D re-interpretation note (for ADR-0060 §D60.2 body)**: forward-scope's `[admin, transfer]` re-interpreted as canonical `[Action::Allocate, Action::Transfer]` (F2.a). `Action::Admin` is not in the canonical 34-verb set; `Allocate` is the org-control-plane Authority verb.

### F3 — Owner-grant rule firing location in Permission Check engine **[LOCKED AT GATE-1: F3.a]**

> **Locked at gate-1: F3.a — inside `step_2_resolve_grants`** (planner-recommended via orchestrator default).

The Permission Check engine at `domain/src/permissions/engine.rs:55-122` runs 6 steps + Step 2a. Step 2 (`step_2_resolve_grants`, `:199-218`) collects candidate grants from `ctx.agent_grants`, `ctx.project_grants`, `ctx.org_grants` (3 ScopeTier types — `Agent`, `Project`, `Organization`).

| Option | Where the rule fires | Cost | Test surface |
|---|---|---|---|
| **F3.a** ✅ **USER-LOCKED** | Inside `step_2_resolve_grants` body — after the 3 existing `collect()` calls, synthesise owner-grants from `ctx.agent_owned_orgs` / `ctx.agent_owned_projects` slices (new `CheckContext` fields). | Mod `step_2_resolve_grants` + extend `CheckContext` struct + extend manifest-builder / call-site loaders to populate the new fields. ~6 sites for `CheckContext` field-add cascade. | New synth-grant unit tests + new acceptance test in §7. |
| **F3.b** | NEW pre-Step `step_0a_owner_grant_synthesis`. | Larger surgery. | Pipeline-step ordering tests. |
| **F3.c** | Outside the engine: build owner-grants in the manifest-resolver / call-site layer. | Smallest engine change; biggest call-site cascade. | Engine tests unchanged; call-site tests proliferate. |

### F4 — Acceptance test file location **[LOCKED AT GATE-1: F4.a]**

> **Locked at gate-1: F4.a — NEW `acceptance_m5_3_owner_grant.rs`** (planner-recommended).

| Option | File | Pattern |
|---|---|---|
| **F4.a** ✅ **USER-LOCKED** | NEW `acceptance_m5_3_owner_grant.rs` at `server/tests/` | Fresh canonical home for M5.3 acceptance. Mirrors CH-24's `acceptance_m5_<topic>.rs` 5-file split precedent. |
| **F4.b** | EXTEND `acceptance_m5_agents.rs` with new test | Bigger file; same audience. |
| **F4.c** | NEW `acceptance_m5_3_philosophy.rs` (one file for all M5.3 carve-out tests) | Reserves the philosophy axis for CH-26 reuse. |

### F5 — R5 investigation routing **[LOCKED AT GATE-1: F5.a]**

> **Locked at gate-1: F5.a — NEW dedicated phase P-R5-INVESTIGATE** (planner-recommended via orchestrator default).

The CH-24 carry-forward (forward-scope §2.5 line 254) requires investigating the `permissions-audit` skill's window-filter logic.

| Option | Where | Cost | Risk |
|---|---|---|---|
| **F5.a** ✅ **USER-LOCKED** | NEW dedicated phase **P-R5-INVESTIGATE** between P0 and P1 (cycle's first meaningful phase). | +0.5 engineer-day. CH-25's own retro can run the fixed skill against its own window. | Adds a phase. |
| **F5.b** | Side-task at gate-3 prep | Smaller pipeline disruption. | If fix requires verification against CH-25 window, requires re-running the skill after fix. |
| **F5.c** | Retrospective-time per user-deferred-to-CH-25 routing | Closest to where the skill is consumed. | Won't surface root cause until retro. |

### F6 — Audit envelope **[LOCKED AT GATE-1: F6.b]**

> **Locked at gate-1: F6.b — 3 auditors (large)** (planner-recommended).

CH-25 is code-touching + new ADR + new Permission Check rule + R5 skill-fix + NEW Edge variant + new domain-tier enum. Per `audit-envelope-size` skill guidance + CH-24 F4.B 3-auditor precedent.

| Option | Auditors | Coverage |
|---|---|---|
| **F6.a** | 2 auditors (medium) | Default for 4–6 phase chunks. |
| **F6.b** ✅ **USER-LOCKED** | 3 auditors (large per CH-24 F4.B precedent) | Audit-A code correctness + edge-variant cascade; Audit-B docs fidelity + concept alignment; Audit-C cross-cutting (governance + ADR Pre-existing-behaviour preservation + R5 skill-fix verification + R2/R3 verification + 71→72 invariant-flip discipline). |

---

## §1 — Context & principle

**Why this chunk**: closes D-philosophy-01 (HIGH-A), the first of two M5.3 carve-out drifts. The drift captures a foundational gap surfaced by the post-CH-21 philosophy alignment audit (2026-04-28): the concept doc `core-philosophy.md:9,23,24` claims *"Agent owns Organization"*, *"Agent create Projects"*, *"Agents own Resources"*. Code today has `Agent.owning_org: Option<OrgId>` (the **inverse** direction — Org-owns-Agent) but NO edge or field expressing Agent→Org/Project ownership/creation provenance. The Permission Check engine has no auto-grant rule that gives the owner-Agent admin authority over its owned Org/Project. CH-25 closes the gap so M6 admin pages 1 (Orgs) and 3 (Projects) plan against a unified ownership model rather than encoding bespoke per-handler gates.

**Quality-over-speed restatement**: *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-25 application: the philosophy claims have been honored at the docs tier since 2026-04-28's audit; CH-25 brings the code into alignment with the doc as the **first** M5.3 carve-out work, before any M6 chunk encodes around the gap.

**Forward-scope reference**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 lines 242-254 (CH-25 row + R5 carry-forward).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim/close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/core-philosophy.md` | line 9 | *"Agent owns Organization"* | contradicted (no edge with Org as the to-end of an Agent-source ownership edge) | honored (via NEW `Edge::Owns { from: AgentId, to: OwnedResourceId::Org(_) }`) |
| `concepts/core-philosophy.md` | line 23 | *"Agent create Projects"* | contradicted (no CREATED edge emitted at apply_project_creation) | honored (via existing `Edge::new_created` typed constructor emission at apply_project_creation; OrgId/ProjectId stay Principal-only per F1.b — `Created` emit uses generic `NodeId` payload, no Resource-trait relaxation needed) |
| `concepts/core-philosophy.md` | line 24 | *"Agents own Resources"* | partially-honored (generic OWNED_BY exists for Memory) | honored (Owns is the Agent→Org/Project specific ownership edge; OWNED_BY stays for Memory + future generic ownership) |
| `concepts/core-philosophy.md` | line 31 | *"Every Resource must have a creator"* | partially-honored | honored (at least for Org/Project — provenance via existing `Created` edge with NodeId payload) |
| `concepts/core-philosophy.md` | line 32 | *"Every Resource ownership must be tracked to the creator - Provenance"* | contradicted (no creator→owner inference at Permission Check time) | honored (synth-owner-grant fires on `Owns` edges; creator-to-owner provenance via paired Owns + Created emission) |
| `concepts/agent.md` | §"Participation in Projects" → §"Agent spawns other Agents (Ownership)" (line 4 amendment) | *"Agent spawn other Agents (Ownership)"* | silent-in-code | honored |
| `concepts/permissions/04-grants.md` | §"Auto-issue rules" (file location TBD verify at P1) | concept supports template-driven + bootstrap-driven auto-issue (analogous patterns) | silent-in-code for owner-grant rule | honored |
| `concepts/permissions/01-resource-ontology.md` | §"Principals + Resources" | Org / Project as Principal-only union members | preserved as-is (per F1.b — Org/Project STAY Principal-only; F1.b does NOT relax the Resource trait) | preserved (Owns variant uses NEW `OwnedResourceId` enum, NOT Resource trait) |
| `concepts/phi-core-mapping.md` | (no overlap) | N/A | N/A | N/A |

**Coverage rule** — every concept-doc claim touched by CH-25's code surface is enumerated above; mid-flight discoveries trigger `AskUserQuestion` + table-row addition per §6 mid-flight rule.

**Permissions subtree hook** — `permissions/04-grants.md` + `permissions/01-resource-ontology.md` touched → MUST cite `permissions/README.md` as the entry invariants source at P3 docs update.

**phi-core-mapping hook** — Org/Project/Permission Check are baby-phi-native; no phi-core overlap → `phi-core-mapping.md` row N/A confirmed.

**F1.b shift (per v2)**: `permissions/01-resource-ontology.md` row was *"partially-honored → honored (relaxation documented)"* at v1 (under F1.a). At v2 (F1.b) the row is *"preserved as-is → preserved"* — Org/Project STAY Principal-only; the new `OwnedResourceId` enum is the resource-end carrier, NOT a Resource-trait relaxation. The concept doc's Principal-only invariant for Org/Project is PRESERVED.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | CH-25 surfaces are baby-phi-native: edge model, Permission Check engine, repository compound-tx, owner-grant synthesis. None of `phi_core::*` overlap. `EdgeKind` / `Edge` enum + `OwnedResourceId` are baby-phi-defined, not phi-core. | — | keep orthogonal |

**Expected import-count delta at chunk close**: **Δ +0** (baseline 57 preserved).

**Positive close-audit greps**:
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect **57** (unchanged from CH-24 close).

**Forbidden-duplication greps**:
- `grep -rnE "^struct AgentProfile|^struct Session|^struct AgentEvent" /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::"` — expect 0 (CH-25 introduces no phi-core-shaped struct).

### Artifact A — `CheckContext` field-add cascade (per F3.a — unchanged from v1)

If F3.a is locked: adding `agent_owned_orgs: &[OrgId]` + `agent_owned_projects: &[ProjectId]` fields to `CheckContext` triggers a cascade. Pre-flight invocation:

```bash
git grep -nE 'CheckContext\s*\{$|CheckContext\s*\{$|CheckContext::' /root/projects/phi/baby-phi/modules/crates/
```

**Raw matched-line count at plan-draft time** (run 2026-05-15):

```
modules/crates/domain/src/permissions/engine.rs:    (2227 PRODUCTION call construction)
modules/crates/domain/src/permissions/engine.rs:    (2276 PRODUCTION call construction)
modules/crates/domain/src/permissions/engine.rs:    (2327 TEST-FIXTURE in #[cfg(test)] fixture builder)
modules/crates/domain/src/permissions/manifest.rs: (CheckContext struct definition — production)
modules/crates/server/src/platform/sessions/launch.rs: (real launch-handler synth-manifest construction)
modules/crates/server/src/platform/sessions/preview.rs: (real preview-handler synth-manifest construction)
modules/crates/server/src/platform/agents/disable.rs OR system_agents/disable.rs (NEW callsite at P2)
```

**Per-file breakdown** (predicted edit-sites — PRODUCTION vs TEST-FIXTURE classification per R2):
- `domain/src/permissions/manifest.rs:NN` — **PRODUCTION** (struct definition). +2 fields. **1 edit**.
- `domain/src/permissions/engine.rs` — **PRODUCTION** at 2 call construction sites (per existing `tier: ScopeTier::Organization` precedent at lines 2229 + 2278 — both confirmed via earlier grep). **2 edits** for field-add.
- `domain/src/permissions/engine.rs:2329` — **TEST-FIXTURE** (inside `#[cfg(test)] mod tests`). **1 edit** for fixture update.
- `server/src/platform/sessions/launch.rs` — **PRODUCTION** synth-manifest construction (per CH-11 retro v8 reading-list discipline at `permissions/engine` chunks). **1 edit**.
- `server/src/platform/sessions/preview.rs` — **PRODUCTION** synth-manifest construction. **1 edit**.
- `server/src/platform/<owner-disable-callsite>.rs` — NEW at P2 (new callsite emitting owner-disable path). **1 new construction**.

**Aggregate prediction**: ~6 production edit-sites + ~1 test-fixture edit. Pause threshold: ≥ 9 sites (1.5× aggregate of 6).

**Pre-existing-behaviour note**: `CheckContext` is the canonical Permission Check input struct; CH-11 (cycle hex `d5428c43`) reading-list rule (planner v8 / CH-15 retro Row 5) mandates including `server::platform::sessions::{launch,preview}` body in §9 reading list — applied below.

### Artifact B — NEW `Edge::Owns` variant cascade (per F1.b — REWRITTEN for v2)

If F1.b is locked: NEW `Edge::Owns { id: EdgeId, from: AgentId, to: OwnedResourceId }` variant added to the `Edge` enum, where `OwnedResourceId` is a NEW domain-tier enum:

```rust
pub enum OwnedResourceId {
    Org(OrgId),
    Project(ProjectId),
}
```

**Per chunk-planner v13 R3 struct-placement dep-direction verification**: `OwnedResourceId` lives in `domain::model::ids` (alongside `OrgId`, `ProjectId`, `AgentId`) — domain tier, since `Edge` enum payload references it (`Edge` is in `domain::model::edges`). Trait return types may reference it freely. Dependency direction: `domain → store → server` PRESERVED — no violation.

**Pre-flight grep — wire-mapping + match-block cascade enumeration** (run 2026-05-15):

(1) Workspace-wide variant-name match-block enumeration:
```bash
grep -rnE "match.*\&?Edge\b|impl Display for Edge\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "/target/"
```
Result: **0 hits outside `Edge::name()` itself** at `domain/src/model/edges.rs:436` — the only exhaustive `match` over `Edge` variants is `Edge::name()`. Per chunk-planner v3 additive-enum bias-low principle: typical exhaustive-match cascade for additive variants is 0; F1.b matches that pattern with `Edge::name()` as the SOLE exhaustive site (1 enumerative addition required).

(2) Wire-mapping function check (per chunk-planner v10):
```bash
grep -rnE "fn (http_status_for|wire_code_for|to_wire)\s*\(\s*\w+:\s*&?Edge\b" /root/projects/phi/baby-phi/modules/crates/
```
Result: **0 hits**. The wire-mapping functions found in the workspace (`platform/templates/mod.rs:133,147` + `platform/system_agents/mod.rs:113,129` + `platform/sessions/mod.rs:172,209`) are over typed-error enums (`TemplateError`, `SystemAgentError`, `SessionError`), NOT over `Edge`. The `Edge` enum is not directly wire-serialized via http_status / wire_code functions — it's persisted via `EdgeId` + SurrealDB relation tables. **No wire-mapping cascade hits for F1.b.**

(3) Workspace-wide `Edge::OwnedBy|Edge::Created|Edge::AllocatedTo` reference enumeration:
```bash
grep -rnE "Edge::OwnedBy|Edge::Created|Edge::AllocatedTo" /root/projects/phi/baby-phi/modules/crates/ | grep -v "/target/"
```
Result: **10 hits, ALL inside `domain/src/model/edges.rs`** (lines 498, 499, 500 = `Edge::name()` arms; 617, 627, 643 = typed constructors; 699, 713, 727 = test-block re-extracts; 743 = test assertion). **Zero production callsites outside the home module.** Per chunk-planner v3 additive-enum bias-low principle: the existing variants OwnedBy/Created/AllocatedTo have zero production callsites in the workspace today; the same will be true for the new `Owns` variant until P1 wires the emit sites.

(4) `EDGE_KIND_NAMES` cardinality cascade:
```bash
grep -rnE "EDGE_KIND_NAMES|edge_kind_names_is_exactly_seventy_one|\[&str; 71\]" /root/projects/phi/baby-phi/modules/crates/ | grep -v "/target/"
```
Result (full enumeration):
- `domain/src/model/edges.rs:525` — `pub const EDGE_KIND_NAMES: [&str; 71]` (PRODUCTION cardinality literal) → flip to `[&str; 72]`.
- `domain/src/model/edges.rs:658` — `fn edge_kind_names_is_exactly_seventy_one()` (TEST test-name literal) → rename to `_seventy_two()`.
- `domain/src/model/edges.rs:662` — `assert_eq!(EDGE_KIND_NAMES.len(), 71);` (TEST assertion literal) → flip to 72.
- `domain/src/model/edges.rs:521-524` — code-comment cardinality narrative (`67 at M3 close, +2 at M4/P1, +2 at CH-23`) → APPEND `, +1 at CH-25 (Owns)` and update total to 72.
- `domain/src/model/mod.rs:84` — `assert_eq!(EDGE_KIND_NAMES.len(), 71);` (TEST assertion literal) → flip to 72.
- `domain/tests/m3_model_counts.rs:23` — comment `[&str; 71]` literal → flip to 72.
- `domain/tests/m3_model_counts.rs:26` — `assert_eq!(EDGE_KIND_NAMES.len(), 71);` (TEST assertion) → flip to 72.

**Per-file breakdown** (PRODUCTION vs TEST-FIXTURE classification per R2):
- `domain/src/model/edges.rs:Edge enum (~line 343-356 area)` — **PRODUCTION**: add new variant `Owns { id: EdgeId, from: AgentId, to: OwnedResourceId }` in the Governance — Ownership section (alongside OwnedBy/Created/AllocatedTo). **1 edit**.
- `domain/src/model/edges.rs:Edge::name() match arm at ~line 498` — **PRODUCTION**: add arm `Edge::Owns { .. } => "OWNS",`. **1 edit**.
- `domain/src/model/edges.rs:EDGE_KIND_NAMES array at line 525-597` — **PRODUCTION**: cardinality flip `71 → 72`; add new entry `"OWNS",` in the Governance—Ownership section of the array. **2 edits** (cardinality literal + new entry).
- `domain/src/model/edges.rs:typed constructor section after line 648` — **PRODUCTION**: add `Edge::new_owns(agent: &AgentId, owned: OwnedResourceId) -> Edge` typed constructor. **1 new fn**.
- `domain/src/model/edges.rs:521-524 narrative comment` — **PRODUCTION** (doc-comment): append `+1 at CH-25 (Owns)` + total 72. **1 edit**.
- `domain/src/model/edges.rs:658 test name + :662 assertion` — **TEST-FIXTURE**: rename test fn + update assertion. **2 edits**.
- `domain/src/model/mod.rs:84` — **TEST-FIXTURE** (inside doctest or `#[cfg(test)] mod tests`): assertion update. **1 edit**.
- `domain/tests/m3_model_counts.rs:23 + :26` — **TEST-FIXTURE** (integration test): comment + assertion update. **2 edits**.
- `domain/src/model/ids.rs` (or wherever `OrgId`/`ProjectId` are defined) — **PRODUCTION**: add NEW enum `OwnedResourceId { Org(OrgId), Project(ProjectId) }` with `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize` derives + `pub fn node_id(&self) -> NodeId` helper if needed by Edge payload. **1 new enum + ~1 impl** (~15 lines).

**Aggregate prediction (Edge variant + cardinality cascade)**: **~9 PRODUCTION edits + ~5 TEST-FIXTURE edits = ~14 total edits**, all within 3 files (`domain/src/model/edges.rs`, `domain/src/model/mod.rs`, `domain/src/model/ids.rs` or sibling, `domain/tests/m3_model_counts.rs`). **Pause threshold**: ≥ 21 total edit-sites (1.5× of 14) — escalate to user.

### Artifact C — NEW production emit-site cascade (per F1.b)

If F1.b is locked: emit `Edge::new_owns(creator_agent, OwnedResourceId::Org(org_id))` + `Edge::new_created` at apply_org_creation + apply_project_creation. Pre-flight invocation (re-uses Artifact B production-callsite enumeration above — confirmed 0 existing production sites for OwnedBy/Created):

**Predicted NEW production emit-sites** (per F1.b):
1. `server/src/bootstrap/claim.rs:~150-300` (claim flow) — emit `Edge::new_owns(ceo_agent, OwnedResourceId::Org(org_id))` ONLY IF claim creates an Org. P0 deliverable 5 re-verifies that bootstrap-claim does NOT currently create an Org — likely no edge emit here.
2. `server/src/platform/orgs/create.rs:~257-280` (apply_org_creation payload assembly) — emit `Edge::new_owns(ceo_agent, OwnedResourceId::Org(new_org_id))` + `Edge::new_created(&creator_agent_node_id, &org_node_id)` (Created uses NodeId payload — no Resource-trait relaxation needed since the NodeId path bypasses the trait). The compound tx needs a new field on `OrgCreationPayload` for the Owns + Created edges.
3. `server/src/platform/projects/create.rs:~360-470` (apply_project_creation Shape A + Shape B both-approve materialization) — emit `Edge::new_owns(creator_agent, OwnedResourceId::Project(project_id))` + `Edge::new_created(&creator_agent_node_id, &project_node_id)`. The compound tx needs a new field on `ProjectCreationPayload` for the Owns + Created edges.

**Aggregate prediction**: ~2-3 production emit-sites (depending on bootstrap-vs-org-create flow clarification at P0). Pause threshold: ≥ 5 sites (1.5× of 3).

### Artifact D — Action vocabulary verification (per F2 — unchanged from v1)

Per chunk-planner v8/v9 §3.D MANDATORY pre-flight check (CH-15/CH-17 2-cycle pattern):

```bash
git grep -nE "^\s*Admin,$|^\s*Allocate,$|^\s*Transfer,$" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs
```

Result at plan-draft (run 2026-05-15):
- `:57: Allocate,` — present (canonical).
- `:58: Transfer,` — present (canonical).
- `Admin` — **NOT FOUND**. Confirmed absent from the 34-verb closed set.

**Verbatim concept-doc 03 citation** (per chunk-planner v9 mandatory pre-flight): `Action::CANONICAL.len() == 34` invariant pinned at `action.rs:250-285`. The concept-doc 03 §"Standard Action Vocabulary" enumerates 34 verbs partitioned into 10 categories. The `Authority` category contains: `Delegate`, `Approve`, `Escalate`, `Allocate`, `Transfer`. No `Admin` verb. **Closed-set invariant `Action::CANONICAL.len() == 34` is NOT broken** — F2 is a forward-scope-vs-concept-doc re-interpretation, not a closed-set break. **Closes v9 pre-flight check requirement.**

### Artifact E — Trybuild fixture impact under F1.b (REPLACES Artifact C of v1)

Under F1.b (NEW `Owns` variant, Resource trait NOT relaxed):

```bash
ls /root/projects/phi/baby-phi/modules/crates/domain/tests/edge_type_safety/compile_fail/
```

Result at plan-draft: `created_rejects_grant_as_principal.rs`, `created_rejects_org_as_resource.rs`, `owned_by_rejects_consent_as_principal.rs`, `owned_by_rejects_node_id_as_principal.rs`, `owned_by_rejects_user_as_resource.rs`.

**Critical finding (REVISED for F1.b)**: `created_rejects_org_as_resource.rs` is PRESERVED under F1.b — Org stays Principal-only; `Edge::new_created(&org, ...)` would still fail to compile because OrgId is not Resource. **F1.b path KEEPS this fixture intact** (no deletion, no replacement). The fixture continues to enforce the v0 ontology invariant that Org is Principal-only.

**Aggregate prediction**: 0 trybuild fixture edits (under F1.b). The 5 existing compile_fail fixtures all stay green.

**Optional NEW fixture (P3 deliverable)**: add `domain/tests/edge_type_safety/compile_fail/owns_rejects_user_as_owned_resource.rs` — verifies that `Edge::new_owns(&agent, OwnedResourceId::Org(uid))` fails to compile when `uid` is `UserId` (not `OrgId` / `ProjectId`). This tests the typed-payload safety of the new constructor. **1 new fixture**.

---

## §3.B — K8s microservice readiness check

| Axis | What to check | This chunk's surface | New blocker? | Action |
|---|---|---|---|---|
| **A1** | In-process state (`DashMap`, `RwLock`, etc.) | Owner-grant synthesis is pure-function within Permission Check engine (Permission Check is already pure per `engine.rs:1-13`). No new state. | no | none |
| **A2** | IPC channel | None. | no | none |
| **A3** | Pod-local resource | None. | no | none |
| **A4** | Migration runner | **F1.b path: no migration needed**. The new `Edge::Owns` variant uses a NEW `owns` relation table in SurrealDB — verify at P0 whether the SurrealDB layer auto-creates relation tables on first emit, OR whether a migration is needed for explicit table declaration. If migration needed: file as P1 deliverable (next-free slot `0017_add_owns_relation.surql`). **Predicted: no migration** (SurrealDB relation tables are dynamically-created on first RELATE; existing `owned_by` + `created` + `allocated_to` are similarly implicit). P0 must verify this assumption. **If migration IS required, auto-approval blocker fires (per orchestrator gate-1 criteria) — escalate to user.** | conditional | re-verify at P0 |
| **A5** | Trait-shape requirement | `Repository` trait already supports compound-tx; adding new edge-fields to `OrgCreationPayload` / `ProjectCreationPayload` preserves trait-object dispatch. New return types `Vec<OrgId>` + `Vec<ProjectId>` are domain-tier (R3 verified). | no | none |
| **A6** | Cross-pod state sharing | Owner-grant synthesis reads from `CheckContext` (per-request); no cross-pod state. Edges persist via SurrealDB. | no | none |
| **A7** | Audit hash-chain symmetry | Owner-grant rule does NOT introduce a new audit-event class. The ownership-edge emission at apply_org/project_creation flows through existing compound-tx audit emission (no new writer). | no | none |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP)**:
- D33.1 (`SessionRegistry` trait) — not touched.
- D33.2 (`SurrealStore::open_remote`) — F1.b adds 1 new relation kind via RELATE statement (same operation class as existing edges); verify no new schema-required syntax at P0.
- D33.3 (SIGTERM graceful shutdown) — no new `tokio::spawn` tasks.
- D33.4 (`EventBus.shutdown` + `drain`) — no new EventBus emitters or listeners.

**Conclusion**: **K8s-neutral** (conditional on A4 verification at P0). CH-25 introduces no new K8s deployment hurdle PROVIDED the SurrealDB relation table for `owns` is auto-created on first RELATE.

---

## §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_3/architecture/agent-ownership-model.md` | **NEW** — must be created (M5.3 has no architecture/ dir yet). Documents: (a) the NEW `Owns` edge variant + `OwnedResourceId` enum payload semantics; (b) the owner-grant rule's firing location + synthesis semantics; (c) the 71→72 invariant flip rationale; (d) why F1.b (NEW variant) over F1.a (Resource trait relaxation). | (a) update in-chunk at P3 |
| **Operations** | `docs/specs/v0/implementation/m5_3/operations/agent-ownership-operations.md` | **NEW** — must be created. Documents: (a) how operators inspect Owns edges via CLI/HTTP; (b) the synthesised-owner-grant query mode in Permission Check decision logs; (c) no new audit-event class. | (a) update in-chunk at P3 |
| **User-guide** | `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` (AMEND per chunk-planner v8 amend-don't-add rule) | The "first-org" + "first-project" walkthroughs need a paragraph noting that the CEO/creator-Agent gains auto-owner-grant on the resulting Org/Project via the new Owns edge. | (a) update in-chunk at P3 as "CH-25 amendment — owner-grant rule (2026-05-15)" subsection |

**Amend-don't-add precedence rule applied**: per CH-17 retro Row 3, the user-guide tier uses an existing `m5/user-guide/first-session-walkthrough.md` amendment rather than fragmenting to a NEW `m5_3/user-guide/` file. Architecture + operations tiers ship as NEW files because M5.3 has no peer doc trees yet (it's a carve-out).

---

## §3.D — Forward-scope-vs-concept-doc precedence (per chunk-planner v8/v9)

**v2 update**: F1.b user-lock RESOLVES the F1 contradiction in favor of the forward-scope literal text (NEW `Owns` variant). F2 contradiction remains as v1 — `Action::Admin` is not in the canonical 34-verb set; F2.a re-interprets to canonical `[Allocate, Transfer]`.

1. **F1 contradiction (RESOLVED via F1.b user-lock)**: forward-scope §2.5 says *"New `Owns` edge variant"*. v1 §3.D framed this as a contradiction with `EDGE_KIND_NAMES.len() == 71`. **User-lock F1.b accepts the literal forward-scope wording — the 71-invariant flips to 72**, which is a controlled invariant evolution (not a break — the invariant test is renamed `_seventy_two` and the literal cardinality is updated). The §3.D re-interpretation note is REMOVED from ADR-0060 §D60.1; instead §D60.1 documents the explicit-edge-semantics rationale + the 71→72 cardinality evolution.

2. **F2 contradiction (UNCHANGED — re-interpreted)**: forward-scope §2.5 says *"auto-issue `[admin, transfer]`"*. Concept-doc invariant: `Action::CANONICAL.len() == 34`; `Action::Admin` does NOT exist; canonical Authority verbs are `Delegate`, `Approve`, `Escalate`, `Allocate`, `Transfer` (`action.rs:53-58` + `:341-345`). The concept-doc wins → forward-scope literal `[admin, transfer]` re-interprets to canonical `[Action::Allocate, Action::Transfer]` (F2.a). Re-interpretation lands in ADR-0060 §D60.2.

**Auto-approval blocker**: per chunk-planner v8 §3.D mechanical procedure step 4, forward-scope-vs-concept-doc contradictions ALWAYS trigger user-escalation at gate-1. CH-25 was NOT a Direct-approval candidate; orchestrator routed through AskUserQuestion + ExitPlanMode at gate-1 — locks captured above.

---

## §3.E — Anticipated gate-2.5 candidates (new v13 section per CH-24 retro Row 6)

Per chunk-planner v13, planner enumerates surfaces likely to surface mid-flight discoveries during P-NEW-TESTS / P-DOCS authoring. **Updated for v2 (F1.b path)**:

| Candidate | Surface | If discovered at gate-2.5, route to |
|---|---|---|
| **C1** | Doc-comments at `server/src/bootstrap/claim.rs:158-300` (bootstrap-claim does NOT currently create an Org). If P2 reveals that the *"first org"* creation is downstream of claim, the forward-scope §2.5 deliverable *"Bootstrap-claim flow ... emit the edge"* needs re-routing. | Option-A close-in-chunk via NEW phase **P-FLIP-BOOTSTRAP**; OR Option-B file follow-up drift `D-CH25-FOLLOWUP-01` (allocation: **M6-DEFERRED-NN** at file-creation time per chunk-planner v13 R4). **Planner recommendation**: Option-A close-in-chunk. |
| **C2** | Placeholder `Vec::new()` / `Default::default()` returns in `CheckContext` field-load paths (the new `agent_owned_orgs` / `agent_owned_projects` slices). If the test-fixture loaders return `vec![]` permanently, CH-26 unifies-resource-model would inherit a placeholder. | Option-A close-in-chunk via P-NEW-TESTS adding 1 acceptance test that LOADS owned-Org/Project IDs from real `Owns` edges; OR Option-B defer to CH-26 with explicit `D-CH25-FOLLOWUP-02` filed (M6-DEFERRED-NN). **Planner recommendation**: Option-A. |
| **C3** | Wire-mapping doc-comments referencing edge-count cardinality across the workspace. Per chunk-planner v15 (CH-15 retro Row 1) widened doc-sync sweep, grep ALL `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` for stale phrases: `71 edge kinds`, `71 variants`, `EDGE_KIND_NAMES.len() == 71`, `seventy-one`. Any match → patch to 72 at P3 doc-sync. | Option-A close-in-chunk via P3 doc-sync sweep (mandatory at gate-2 inline doc-sync per CH-15 retro Row 1). |
| **C4** | `D-philosophy-01.md:27` cites "66 variants" — STALE NOW (actual is 71 pre-CH-25; will be 72 post-CH-25). | Option-A close-in-chunk at P-SEAL deliverable 2 (drift remediation update — append amendment noting 66→71→72 cardinality history). |
| **C5** | SurrealQL persistence path: if `owns` relation table requires explicit migration declaration (vs implicit creation), this is a new migration. | Option-A close-in-chunk via P1 deliverable 6 adding `store/migrations/0017_add_owns_relation.surql`; OR Option-B re-plan (architectural FAIL if migration cascades into ADR-0033 K8s seams). **Planner recommendation**: Option-A if migration is needed (additive migration is in-scope); orchestrator escalates if Option-B becomes necessary. |
| **C6** | ADR-0060 sub-decisions §D60.3 + §D60.4 marked Proposed at chunk-open. If P2 / P3 surfaces a different firing-location-conclusion than F3.a, the ADR-0060 body needs re-write. | Option-A re-plan via planner re-spawn (architectural FAIL flow). |

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-philosophy-01` | `../../v0/implementation/m5_3/drifts/D-philosophy-01.md` | HIGH (Bucket A) | `discovered → remediated` | First M5.3 carve-out closure. Lifecycle entry appended at P-SEAL with CH-25 ✓ marker + cycle hex `1e01618e`. **F1.b lifecycle note**: drift text `:27` cites "66 variants" — stale; P-SEAL deliverable 2 appends an amendment noting `66 → 71 (pre-CH-25 reality) → 72 (post-CH-25 NEW Owns variant)`. |

**M*-DEFERRED-NN allocation check (per chunk-planner v13 R4)**: D-philosophy-01 transitions to terminal `remediated` — no `M*-DEFERRED-NN` allocation needed. **The chunk does NOT carry any `TBD` markers.** D-philosophy-02 (CH-26's target) remains at `discovered` and already cites *"Closes at: CH-26"* (terminal allocation already explicit — no remediation required).

If P-R5-INVESTIGATE surfaces a NEW drift (e.g., a follow-up `D-CH25-FOLLOWUP-NN` filed for an out-of-scope skill aspect), OR if any gate-2.5 candidate triggers Option-B routing, the planner/implementer MUST populate the new drift file's `Impl chunk` field with an explicit `M6-DEFERRED-NN` allocation at file-creation time per chunk-planner v13 R4. NEVER write `TBD`.

---

## §5 — ADRs drafted

**ADR number assignment**: highest current ADR is **0059** (`0059-recent-sessions-api-surface-flip.md` at `m5_2/decisions/`). Next free: **ADR-0060**.

| Number | Title | Drafted-at-phase | Decision-summary | Expected flip-to-Accepted phase |
|---|---|---|---|---|
| **ADR-0060** | Agent-as-creator-and-owner edge model + owner-grant Permission Check rule (CH-25, M5.3 first carve-out) | P0 (draft Proposed) | Closes D-philosophy-01 by adding a NEW `Edge::Owns` variant on the `Edge` enum (Agent→Org/Project ownership with typed `OwnedResourceId` payload, distinct from generic OwnedBy); emits `Owns` + existing `Created` edges at org-create + project-create flows; synthesizes auto-owner-grant inside `step_2_resolve_grants` from `Owns` edges; bumps `EDGE_KIND_NAMES` invariant 71→72. | P-SEAL |

**ADR file path proposal**: `docs/specs/v0/implementation/m5_3/decisions/0060-agent-as-creator-and-owner.md` (new directory — M5.3's first ADR; mirrors M5.3's `drifts/` peer layout).

**Sub-decisions sketch** (Proposed at P0; will be ratified at P-SEAL):

- **§D60.1 — Owner-edge shape** *(REWRITTEN for v2 under F1.b user-lock)*: NEW `Edge::Owns` variant on the `Edge` enum (user-locked F1.b, DIVERGENT from planner-recommended F1.a reuse-existing-+-Resource-trait-relax path). The variant carries `from: AgentId, to: OwnedResourceId` where `OwnedResourceId` is a NEW domain-tier enum `Org(OrgId) | Project(ProjectId)`. Rationale (user-supplied): explicit-and-visible edge semantics over structural cleanliness — the Agent→Org/Project ownership relation is distinct enough (typed payload, dedicated semantics) to warrant its own variant rather than re-using generic OWNED_BY (which stays Memory-as-Resource focused). Closed-set evolution: `EDGE_KIND_NAMES.len()` flips 71 → 72; the invariant test renames `edge_kind_names_is_exactly_seventy_one` → `_seventy_two`. The narrative comment at `edges.rs:521-524` is updated to append `+1 at CH-25 (Owns)` with the total 72. Cascade per §3 Artifact B: ~9 production edits + ~5 test-fixture edits, all within 4 files. **Pre-existing-behaviour preservation note** (per chunk-planner v11 strict form): *"Pre-existing implementation preserved: typed `Edge::new_owned_by` + `Edge::new_created` constructors (shipped at ADR-0015 / M1 close); CH-25 adds a NEW `Edge::Owns` variant + `Edge::new_owns` constructor but does not change existing OwnedBy/Created/AllocatedTo semantics."*

- **§D60.2 — Owner-grant action set** *(F2.a locked)*: `[Action::Allocate, Action::Transfer]` (planner recommendation, user-aligned). Forward-scope row literal *"[admin, transfer]"* re-interpreted because `Action::Admin` is not in canonical 34-verb set. **Pre-existing-behaviour preservation note**: *"Pre-existing scaffold preserved: `Action::Wildcard` used by bootstrap-AR Grant + CEO `[Allocate]` grants (shipped at ADR-0014 / claim.rs:241 + orgs/create.rs:181 / M1+M3 close); CH-25 adds a synth-only auto-owner-grant carrying the new `[Allocate, Transfer]` action set; does not change pre-existing Action verbs."*

- **§D60.3 — Owner-grant firing location** *(F3.a locked)*: inside `step_2_resolve_grants` (planner recommendation, user-aligned). **Pre-existing-behaviour preservation note**: *"Pre-existing implementation preserved: 6-step + Step 2a Permission Check pipeline (shipped at ADR-0008 / M1 close, refined at CH-07 / ADR-0051 + CH-15 / ADR-0054); CH-25's F3.a path extends `step_2_resolve_grants` with synth-grant collection; preserves all 6 steps' ordering + does not change Step 1/3/4/5/6 bodies."*

- **§D60.4 — Acceptance scope** *(F4.a locked)*: NEW `acceptance_m5_3_owner_grant.rs` file. Acceptance scenario verifies: (a) bootstrap → claim CEO; (b) `apply_org_creation` emits `Owns` + `Created` edges; (c) CEO disables a child agent WITHOUT explicit prior `[disable]` grant on the child Agent; (d) operation succeeds because owner-grant rule auto-synthesises `[Allocate, Transfer]` which COVERS the `disable` Manifest reach. **Pre-existing-behaviour preservation note** (per chunk-planner v11 multi-milestone-pattern variation form): *"Pre-existing implementation preserved: `disable_system_agent` handler shipped at CH-01 / ADR-0034 §D34.4 (durable `active: false` flip BEFORE audit emit). CH-25 adds an acceptance test that exercises owner-grant auto-issue at the existing disable-handler; does not change the handler itself."*

- **§D60.5 — R5 permissions-audit skill fix preservation note** (per chunk-planner v11 never-shipped-yet variation form): *"Pre-existing absence preserved: the skill returned `0` for non-empty windows due to the `jq` predicate bug at line 31 of `.claude/skills/permissions-audit.md` (`select(...) | .version == 1` evaluates a boolean instead of filtering); CH-25 ratifies the fix as canonical convention — replace with single-predicate `select(.ts >= start AND .ts <= end AND .version == 1)` form. No prior behaviour changes (the skill never returned correct counts for any window; CH-24 retrospective's `0` count is the first observed manifestation)."*

- **§D60.6 — Edge-count cardinality evolution META (NEW per v2)**: documents the `EDGE_KIND_NAMES` invariant evolution `66 → 71 → 72` across milestones. M1 close: 67 variants. M3 close: 67. M4/P1: +2 (HasSubproject, HasConfig) → 69. CH-23: +2 (Manages, HasAgentSupervisor) → 71. CH-25: +1 (Owns) → 72. The drift `D-philosophy-01.md:27` cites 66 — stale; corrected at P-SEAL deliverable 2 with an amendment trail. Updates to: (a) `D-philosophy-01.md:27` (drift text amendment); (b) `edges.rs:521-524` (narrative comment cardinality history); (c) any architecture/operations doc citing edge-count cardinality.

**Cross-references** (per chunk-planner v6 ADR-body checklist):
- (a) **Originating concept-doc** + section + line range: `concepts/core-philosophy.md:9,23,24,31,32` + `concepts/agent.md:4` + `concepts/permissions/04-grants.md:<§"Auto-issue rules">` + `concepts/permissions/01-resource-ontology.md:<§"Principals + Resources">`.
- (b) **Closed drift(s) by ID**: `D-philosophy-01` (HIGH-A).
- (c) **Prior ADRs cited as precedent** (per chunk-planner v6 — milestone-prefixed paths for cross-milestone refs):
  - `m1/decisions/0015-type-safe-ownership-edges.md` (the typed-Edge-constructor + Principal/Resource sealing pattern CH-25 extends with a new variant + new typed constructor).
  - `m1/decisions/0008-permission-check-as-pipeline.md` (the 6-step pipeline CH-25's F3.a extends).
  - `m3/decisions/0022-org-creation-compound-transaction.md` (the compound-tx pattern CH-25 extends at apply_org_creation).
  - `m1/decisions/0034-human-agent-identity-guard.md` (CH-01's `disable_system_agent` ADR for the acceptance test's child-disable path).
  - `m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md` (CH-14 / system-genesis precedent for synth-grants from axiomatic principals — analogous to owner-grant synthesis).
- (d) **Forward-scope row**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 lines 242-254.

**Forks header (per chunk-planner v6 ADR-body checklist)**: *"Forks (CH-25 planner-recommendation: F1.a / F2.a / F3.a / F4.a / F5.a / F6.b; user-locked at plan approval to **F1.b / F2.a / F3.a / F4.a / F5.a / F6.b** — F1 DIVERGENT from planner-recommendation; divergence-aware framing applied to F1 + F2 + F4 per chunk-planner v13 80%-cumulative-divergence rule)"*.

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-08 / ADR-0052** | `Action::Transfer` exists in canonical 34-verb set; cardinality-table-aware compound-tx pattern at `apply_transfer_grant` | `grep -n "Action::Transfer" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs` — expect ≥ 1 hit |
| **CH-13 / ADR-0050** | Permission Check engine `step_2_resolve_grants` ScopeTier ordering (Agent → Project → Organization); audit-class composition strictest-wins | `grep -n "ScopeTier::Agent\b" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/engine.rs` — expect ≥ 3 hits |
| **CH-14 / ADR-0053** | `system_genesis_principal()` precedent for synth-Principal axioms; CH-25's owner-grant synthesis mirrors this for owner-Agent | `grep -n "system_genesis_principal" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/axioms.rs` — expect ≥ 1 hit |
| **CH-15 / ADR-0054** | `Vec<Grant>` typed multi-value return precedent for cascade-result methods | (informational — no direct dep) |
| **CH-23 / ADR-0046** | Compound-tx pattern for new edge emissions inside `apply_*_creation` flows; idempotent re-POST 200-vs-201 split for edge-creation handlers | `grep -n "create_manages_edge\|create_has_agent_supervisor_edge" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs` — expect ≥ 2 hits |
| **CH-24 / ADR-0059** | Test-count baseline 1537 (pre-CH-25 baseline); phi-core import baseline 57; 4 CI guards green | `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4 2>&1 \| tail -5` — expect `1537 passed` |
| **CH-01 / ADR-0034 §D34.4** | `disable_system_agent` durable-state-flip-before-audit-emit pattern; CH-25 acceptance test re-uses this handler | `grep -n "set_agent_active" /root/projects/phi/baby-phi/modules/crates/server/src/platform/system_agents/disable.rs` — expect ≥ 1 hit |
| **CH-23 (M3 model-counts)** | `EDGE_KIND_NAMES.len() == 71` invariant pinned at 4 sites (per Artifact B enumeration); CH-25 flips all 4 to 72 in lockstep | `grep -rcE "EDGE_KIND_NAMES.len\(\) == 71\|\[&str; 71\]" /root/projects/phi/baby-phi/modules/crates/` — expect 4 hits at chunk-open, 0 hits at chunk-close (all flipped to 72) |

Per per-chunk-planning-template §6 rule, this table runs AT CHUNK OPEN before any phase opens, and again at chunk seal. Any regression produces a new drift file + surfaces as an open question for user before the chunk proceeds.

---

## §7 — Phases within the chunk

**Phase count: 6** (P0 + P-R5-INVESTIGATE + P1 + P2 + P3 + P-SEAL).

### P0 — Scaffolding + pre-conditions re-verify

- **Goal**: lock plan + verify §6 carry-forward invariants + draft ADR-0060 Proposed + create the `m5_3/decisions/` directory.
- **Deliverables**:
  1. Re-run §6 re-verification commands; record results in implementation report.
  2. Create directory `docs/specs/v0/implementation/m5_3/decisions/`.
  3. Draft `m5_3/decisions/0060-agent-as-creator-and-owner.md` as Proposed (with sub-decisions §D60.1–§D60.6 per §5 sketch + Forks header + Cross-references). **Note v2 F1.b shift**: §D60.1 body documents the explicit-edge-semantics rationale + 71→72 cardinality evolution + the divergence from planner-recommended F1.a.
  4. Verify SurrealDB relation-table auto-creation assumption (A4 axis): grep `store/src/repo_impl.rs` + migration files for explicit `DEFINE TABLE owned_by`/`DEFINE TABLE created` statements. If found, plan corresponding `DEFINE TABLE owns` migration in P1. If implicit-creation pattern is in use, no migration needed.
  5. Confirm bootstrap-claim flow does NOT create an Org (re-verify via `grep` on `claim.rs` body) — informs whether C1 gate-2.5 candidate fires.
  6. Confirm zero existing production callsites of `Edge::OwnedBy`/`Edge::Created`/`Edge::AllocatedTo` (per Artifact B enumeration — already verified at plan-draft: only home-module + tests).
- **Tests**: none added; baseline 1537 preserved.
- **Concept-alignment check**: no §2 row transitions yet (ADR draft only).
- **phi-core leverage check**: re-run baseline grep — expect 57.
- **User-facing doc updates**: none.
- **Confidence target**: 100% (scaffolding phase).
- **Pause discipline**: HALT for AskUserQuestion if (a) §6 regression detected; (b) bootstrap-claim flow IS found to create an Org (C1 candidate fires → user-lock close-in-chunk vs file follow-up); (c) explicit `DEFINE TABLE` for relations is required, meaning A4 axis fires → migration needed; (d) any existing production callsite of OwnedBy/Created/AllocatedTo is discovered outside the home module (would require ripple-edits).

### P-R5-INVESTIGATE — permissions-audit skill jq predicate fix (per F5.a; ~0.5 ed)

- **Goal**: close R5 carry-forward investigation; fix the broken `jq` predicate; validate against CH-24's cycle window + CH-25's own window.
- **Deliverables**:
  1. Read `/root/projects/phi/.claude/skills/permissions-audit.md` end-to-end (root cause already identified at plan-draft: line 31 `select(.ts >= "$start_ts" and .ts <= "$end_ts") | .version == 1` evaluates `.version == 1` as a boolean expression PIPED through `select` output, producing `true`/`false` literals instead of filtering original entries).
  2. Apply 1-line fix to `permissions-audit.md:31`: replace `select(.ts >= "$start_ts" and .ts <= "$end_ts") | .version == 1` with `select(.ts >= "$start_ts" and .ts <= "$end_ts" and .version == 1)`. The `and` operator in jq is valid + folds the version filter into a single `select` predicate.
  3. Bump skill version `3 → 4` in front-matter.
  4. Append a brief "v4 — fixed at CH-25 (cycle hex `1e01618e`) per CH-24 retro R5 carry-forward" header note.
  5. **Validation**: run the fixed jq pipeline against `.claude/tool-use.log` with CH-24 window (`2026-05-11T12:48:36Z → 2026-05-11T19:36:05Z`) — expect non-zero count. Cross-check against `grep -c "2026-05-11" /root/projects/phi/.claude/tool-use.log` (expected ≥ 1290 per CH-24 retro spot-check).
- **Tests**: none added (skill validation is manual; verification command captured in §12).
- **Concept-alignment check**: N/A.
- **phi-core leverage check**: N/A.
- **User-facing doc updates**: none.
- **Confidence target**: 100% (single 1-line jq fix; root cause already known).
- **Pause discipline**: HALT for AskUserQuestion if (a) the fix returns 0 against CH-24 window despite the predicate correction; (b) the version-1 filter rejects ALL entries.

### P1 — NEW `Edge::Owns` variant + `OwnedResourceId` enum + emit-site wiring (per F1.b user-lock)

- **Goal**: add the NEW `Edge::Owns` variant; add the new `OwnedResourceId` domain-tier enum; flip the 71→72 cardinality invariant; add `Edge::new_owns` typed constructor; emit `Owns` + `Created` at apply_org_creation + apply_project_creation.
- **Deliverables**:
  1. `domain/src/model/ids.rs` (or sibling): add NEW enum `pub enum OwnedResourceId { Org(OrgId), Project(ProjectId) }` with `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]`. Add helper `pub fn node_id(&self) -> NodeId { match self { Self::Org(o) => o.node_id(), Self::Project(p) => p.node_id() } }` (or equivalent — depending on how Edge variant payload is structured).
  2. `domain/src/model/edges.rs`: add NEW variant `Owns { id: EdgeId, from: AgentId, to: OwnedResourceId }` in the Governance — Ownership section (between `AllocatedTo` and `IssuedGrant`, line ~362 area). Add `Edge::name()` arm `Edge::Owns { .. } => "OWNS",`.
  3. `domain/src/model/edges.rs:521-525`: update narrative comment to append `+1 at CH-25 (Owns)`; total 72.
  4. `domain/src/model/edges.rs:525`: flip cardinality `[&str; 71] → [&str; 72]`. Add `"OWNS",` entry in the Governance — Ownership section of the array (alongside `"OWNED_BY"`, `"CREATED"`, `"ALLOCATED_TO"`).
  5. `domain/src/model/edges.rs:After typed constructor section ~line 648`: add `pub fn new_owns(agent: &AgentId, owned: OwnedResourceId) -> Edge { Edge::Owns { id: EdgeId::new(), from: agent.clone(), to: owned } }` typed constructor. **Note**: `from: AgentId` is concrete type (not generic `Principal`) per F1.b's explicit-edge-semantics — the variant payload is typed directly to AgentId rather than NodeId.
  6. `domain/src/model/edges.rs:658,662`: rename test fn `edge_kind_names_is_exactly_seventy_one → _seventy_two`; flip assertion `71 → 72`.
  7. `domain/src/model/mod.rs:84`: flip assertion `71 → 72`.
  8. `domain/tests/m3_model_counts.rs:23,26`: flip comment + assertion `71 → 72`.
  9. `domain/src/repository.rs`: extend `OrgCreationPayload` with a new field `pub creator_agent: AgentId` — needed to emit `Edge::new_owns` + `Edge::new_created`. Cascade to all `OrgCreationPayload {...}` literal-construction sites (predicted ~3 sites: `orgs/create.rs:257`, `in_memory.rs` impl, `store/tests/` fixtures).
  10. `domain/src/repository.rs`: extend `ProjectCreationPayload` with NEW field `pub creator_agent: AgentId`. Cascade to ~4-5 construction sites including `projects/create.rs:~340-400` Shape A + Shape B materialize paths.
  11. `domain/src/in_memory.rs`: in `apply_org_creation` (line ~358 area) — emit `Edge::new_owns(&payload.ceo_agent, OwnedResourceId::Org(organization.id))` + `Edge::new_created(&payload.creator_agent_node, &organization_node)` (Created uses NodeId via existing typed constructor — works since `Created` payload is `NodeId, NodeId`). **Important — pre-existing semantics**: the `ceo_agent` is the OWNER (per concept-doc 9), but `creator_agent` may be a different Agent (platform-admin who provisioned the Org on behalf of the CEO). The owner-grant rule synthesises on `Owns` (not `Created`); the `Created` edge is provenance-only. **Surface this distinction explicitly in §D60.1 body.**
  12. `domain/src/in_memory.rs`: in `apply_project_creation` (line ~1758 area) — emit `Edge::new_owns(&lead_agent, OwnedResourceId::Project(project.id))` + `Edge::new_created(&payload.creator_agent_node, &project_node)`. **`lead_agent` is the OWNER per the deliverable wording *"Agent create Projects"* → creator = owner at Shape A**; Shape B both-approve case needs design clarity (owner = first-approver? co-owner? both?) — this is a sub-fork **F1.b.subtle** that may need user-lock OR can default to `lead_agent` as the OWNER for both shapes.
  13. `store/src/...apply_org_creation` SurrealDB compound-tx body: add `RELATE Agent -> owns -> Org` (typed via `owns` relation table) + `RELATE Agent -> created -> Org`. **Verify at P0 deliverable 4** whether `owns` table needs explicit DEFINE TABLE (migration) OR is implicit-created.
  14. Mirror at `store/src/...apply_project_creation`.
  15. (Optional, per Artifact E) Add `domain/tests/edge_type_safety/compile_fail/owns_rejects_user_as_owned_resource.rs` — verifies that `Edge::new_owns(&agent, OwnedResourceId::Org(uid))` fails to compile when `uid: UserId`.
- **Tests**: ~3-5 new unit tests + ~2 in-memory tests (edge-emit assertions); +1 trybuild fixture (optional); +1 acceptance test in P3.
- **Concept-alignment check**: rows 1, 2, 3, 4 (line 9, 23, 24, 31 of `core-philosophy.md`) flip `contradicted/partially-honored → honored`.
- **phi-core leverage check**: re-run baseline grep — expect 57 (Δ +0).
- **User-facing doc updates**: none yet (P3 handles all docs).
- **Confidence target**: ≥ 97% (1 sub-fork F1.b.subtle re Shape B creator).
- **Pause discipline**: HALT for AskUserQuestion if (a) F1.b.subtle surfaces (Shape B both-approve creator semantics); (b) Artifact B total cascade exceeds 21 sites (1.5× of 14); (c) SurrealDB `owns` relation requires explicit DEFINE TABLE migration → re-routes to A4 axis fire; (d) any existing test outside the planned cascade-site list fails post-edit (would indicate hidden exhaustive-match dependency).

### P2 — Owner-grant rule synthesis in Permission Check engine (per F3.a)

- **Goal**: synthesise auto-owner-grants inside `step_2_resolve_grants`; add `agent_owned_orgs` / `agent_owned_projects` fields to `CheckContext`; load these at call-sites from `Owns` edges.
- **Deliverables**:
  1. `domain/src/permissions/manifest.rs`: extend `CheckContext` struct with `pub agent_owned_orgs: &'a [OrgId]` + `pub agent_owned_projects: &'a [ProjectId]`. (Verify exact location of `CheckContext` definition at P0.)
  2. `domain/src/permissions/engine.rs:199-218` (`step_2_resolve_grants` body): after the existing 3 `collect()` calls, append a synth-grant generator loop:
     - For each `org_id` in `ctx.agent_owned_orgs`: synthesise a `Grant { holder: PrincipalRef::Agent(ctx.agent), action: vec![Action::Allocate, Action::Transfer], resource: ResourceRef { uri: format!("org:{}", org_id) }, fundamentals: <per Org-as-Composite expansion>, descends_from: None, delegable: true, issued_at: <ctx.now>, revoked_at: None, approval_mode: ApprovalMode::Implicit, audit_class: AuditClass::Silent, allocate_refinement: None }`. Push as Candidate with `tier: ScopeTier::Agent` (most-specific tier).
     - Same for `agent_owned_projects` with `uri: format!("project:{}", project_id)`.
  3. Cascade: `engine.rs` test-fixture builder at line ~2229/2278/2329 — extend with the new fields (TEST-FIXTURE classification per R2).
  4. Cascade: `server/src/platform/sessions/launch.rs` + `server/src/platform/sessions/preview.rs` synth-manifest construction — load `agent_owned_orgs/projects` from the new `Repository::list_agent_owned_orgs(agent_id)` + `list_agent_owned_projects(agent_id)` query methods (NEW Repository methods reading `Owns` edges).
  5. Add 2 NEW Repository methods: `async fn list_agent_owned_orgs(&self, agent: AgentId) -> RepositoryResult<Vec<OrgId>>` + `async fn list_agent_owned_projects(&self, agent: AgentId) -> RepositoryResult<Vec<ProjectId>>`. Both backends (InMemory + SurrealDB) implement by reading `Owns` edges where `from == agent` and pattern-matching on `to` variant (`OwnedResourceId::Org` / `OwnedResourceId::Project`). **Per R3 struct-placement dependency-direction**: return types `Vec<OrgId>` + `Vec<ProjectId>` are existing `domain::model::ids` types; no NEW struct introduced at this boundary (`OwnedResourceId` lives in `domain::model::ids` per P1 deliverable 1 — domain-tier, trait return types may freely reference it). Dep direction: `domain → store → server` PRESERVED.
  6. `domain/src/permissions/engine.rs`: add ≥ 3 unit tests for the synth-grant loop (covers: empty owned-orgs returns no synth grants; owned-org with `disable` Manifest reach succeeds; owned-project covers project-scoped reaches).
- **Tests**: ~6-8 new unit tests + ~2 integration tests; +1 acceptance test in P3.
- **Concept-alignment check**: rows 5, 6, 7 (core-philosophy line 32 provenance; agent.md ownership; permissions/04-grants auto-issue) flip `contradicted/silent-in-code → honored`.
- **phi-core leverage check**: re-run baseline grep — expect 57.
- **User-facing doc updates**: none yet (P3 handles all docs).
- **Confidence target**: ≥ 97%.
- **Pause discipline**: HALT for AskUserQuestion if (a) `CheckContext` cascade exceeds 9 sites; (b) Repository method-add forces a non-additive change at the trait layer; (c) the synth-grant fundamentals expansion produces a regression in existing Permission Check tests.

### P3 — User-facing docs + acceptance test (per F4.a user-lock)

- **Goal**: ship user-facing docs at M5.3 tier; ship 1 acceptance test exercising the full owner-grant scenario; flip §2 rows that need final closure; flip §3.C rows.
- **Deliverables**:
  1. Create `docs/specs/v0/implementation/m5_3/architecture/agent-ownership-model.md` (NEW; ~150-200 LOC). Sections: §1 What this is, §2 NEW `Owns` edge variant + `OwnedResourceId` payload semantics, §3 Owner-grant synthesis (§D60.3 narrative + diagram), §4 71→72 cardinality evolution rationale + why F1.b over F1.a, §5 Acceptance scenario walk, §6 K8s posture.
  2. Create `docs/specs/v0/implementation/m5_3/operations/agent-ownership-operations.md` (NEW; ~80-120 LOC). Sections: §1 Inspecting Owns edges via CLI/HTTP, §2 Reading synth-owner-grants in Permission Check decision logs, §3 No new audit-event class.
  3. Amend `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` (per chunk-planner v8 amend-don't-add rule + CH-17 precedent) — add a "CH-25 amendment — owner-grant rule (2026-05-15)" subsection noting that the CEO/creator-Agent gains auto-owner-grant via the new Owns edge.
  4. Update `docs/specs/v0/concepts/permissions/04-grants.md` verified-header + add a §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" subsection.
  5. Update `docs/specs/v0/concepts/permissions/01-resource-ontology.md` verified-header — note that under F1.b, Org/Project STAY Principal-only (the F1.b path does NOT relax the Resource trait); the Owns edge variant uses the NEW `OwnedResourceId` enum payload to carry typed Org/Project ownership-target IDs.
  6. Update `docs/specs/v0/implementation/m5_3/drifts/_concept-audit-matrix.md` — flip the D-philosophy-01-tracking rows to `honored` letter-for-letter from §2 target column (per chunk-planner v6 P4 addendum + CH-12 retro Row 1).
  7. Add 1 acceptance test: `server/tests/acceptance_m5_3_owner_grant.rs` (per F4.a). Scenario: bootstrap + claim CEO + org create (CEO becomes owner via Owns edge) + create child Intern Agent under Org-A + CEO disables child Intern Agent without explicit prior `[disable]` grant → operation succeeds via synth-owner-grant.
  8. Doc-sync sweep across `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` per CH-15 retro Row 1 widened sweep rule: grep for `71 edge kinds`, `71 variants`, `EDGE_KIND_NAMES.len() == 71`, `seventy-one` — patch any match to 72.
- **Tests**: +1 acceptance test (Artifact E optional trybuild fixture already counted in P1).
- **Concept-alignment check**: all §2 rows at `target = honored` should be at `honored`. Final verification at P-SEAL.
- **phi-core leverage check**: re-run baseline grep — expect 57.
- **User-facing doc updates**: all §3.C rows shipped (architecture + operations NEW; user-guide AMEND).
- **Confidence target**: ≥ 97%.
- **Pause discipline**: HALT for AskUserQuestion if (a) doc-link CI guard fails (broken cross-ref); (b) doc-sync sweep surfaces > 5 stale-cardinality references (would suggest broader pre-existing drift requiring its own follow-up).

### P-SEAL — Chunk-seal paperwork

- **Goal**: ratify ADR-0060 Accepted; flip drift D-philosophy-01 to remediated; flip cycle-index row to ready-for-audit; finalize all verified-headers; ship final state to be auditor-ready.
- **Deliverables**:
  1. Flip ADR-0060 status `Proposed → Accepted`; ratify §D60.1–§D60.6 with final sub-decision bodies; finalize Forks header per user-lock outcome (F1.b/F2.a/F3.a/F4.a/F5.a/F6.b — DIVERGENT on F1).
  2. Flip `D-philosophy-01.md` Status `discovered → remediated`; append lifecycle entry `2026-05-15 — remediated — CH-25 ✓ (cycle hex 1e01618e); NEW Edge::Owns variant emitted at apply_org/project_creation; owner-grant synthesis live in step_2_resolve_grants; EDGE_KIND_NAMES 71→72; acceptance test green.` **Also amend drift text `:27`** to correct the stale `66 variants` reference to `66 → 71 (pre-CH-25 reality) → 72 (post-CH-25 NEW Owns)`.
  3. Update `m5_3/drifts/README.md`: D-philosophy-01 Status cell `discovered → remediated`; remediated count `0 → 1`; open count `2 → 1`.
  4. Update `m5_3/drifts/_concept-audit-matrix.md` rows.
  5. Verified-header bumps for every touched doc.
  6. Append cycle-index row to `docs/specs/plan/build/_cycle-index.md`: chunk slug + cycle hex + status `ready-for-audit` + iteration count 1 (per chunk-implementer v5+ + CH-17 retro Row 4).
  7. Re-run §6 carry-forward invariants — re-verify all green, including the EDGE_KIND_NAMES 71→72 flip at all 4 sites enumerated in Artifact B.
- **Tests**: re-run full workspace test — expect [1545, 1555] band (baseline 1537 + 1 acceptance + ~6-8 unit + ~2 integration + 1 optional trybuild + edge cascade = ~14-18 new tests; ×1.0–×1.30 range → 1545 lower, 1555 upper).
- **Concept-alignment check**: all §2 rows at `honored`.
- **phi-core leverage check**: final baseline grep — expect 57.
- **User-facing doc updates**: all §3.C rows shipped + audited.
- **Confidence target**: ≥ 99% (seal phase).
- **Pause discipline**: HALT for AskUserQuestion if test-count outside [1545, 1555] band.

---

## §8 — Tests summary

**Expected total test count at chunk close**: **[1545, 1555]** (Artifact-B-edge-cascade + Artifact-A-CheckContext-cascade chunk per chunk-planner v3 + CH-17 retro Row 5 ×1.30 ceiling).

- Pre-CH-25 baseline: 1537 (from CH-24 cycle-audit, validated in `_cycle-index.md`).
- Deliverable-listed:
  - P1: ~5 unit tests (edge-emit assertions in `in_memory.rs` impl tests) + ~2 store-tier tests (SurrealDB compound-tx assertions with new `Owns` edge) + ~1 trybuild fixture (optional `owns_rejects_user_as_owned_resource.rs`).
  - P2: ~6-8 unit tests for synth-grant loop (engine.rs `#[cfg(test)] mod tests`) + ~2 integration tests for new Repository methods (`list_agent_owned_orgs/projects`).
  - P3: 1 acceptance test (`acceptance_m5_3_owner_grant::ceo_disables_child_agent_without_explicit_grant`).
  - P-R5-INVESTIGATE: 0 tests (manual validation only).
- **Sum**: 14-18 new tests; lower-bound × 1.0 → 14, upper × 1.30 → 24. Reasonable mid-band: **[1545, 1555]**.

**MUST-SHIP** (per chunk-planner v8 / CH-17 retro Row 6):
- `server/tests/acceptance_m5_3_owner_grant.rs::ceo_disables_child_agent_without_explicit_grant` — MUST exist at chunk-seal.
- `domain/src/model/edges.rs::tests::edge_kind_names_is_exactly_seventy_two` (renamed from `_seventy_one`) — MUST exist + pass at chunk-seal.
- `domain/tests/m3_model_counts.rs` assertion `EDGE_KIND_NAMES.len() == 72` — MUST pass.
- `domain/src/model/edges.rs::tests` for new `Edge::new_owns` constructor (≥ 1 happy-path test) — MUST exist.

**MAY-COVER** (band-floor surrogate tests):
- Unit tests in `engine.rs` `#[cfg(test)] mod tests` for synth-grant edge cases.
- Unit tests in `in_memory.rs` for edge-emit assertions.
- Repository-trait unit tests for `list_agent_owned_orgs/projects` (in-memory + SurrealDB).
- Optional trybuild fixture `owns_rejects_user_as_owned_resource.rs`.

**Layer breakdown**:
- Unit: ~12-14
- Integration: ~2
- Acceptance: 1
- Trybuild (compile-fail): +1 optional (net change 0 if Artifact E NEW fixture adds OR +1 if it's a true net addition since under F1.b no fixture is deleted)
- e2e: 0

**Named test files**:
- NEW: `server/tests/acceptance_m5_3_owner_grant.rs`.
- OPTIONAL NEW: `domain/tests/edge_type_safety/compile_fail/owns_rejects_user_as_owned_resource.rs` (trybuild fixture).
- **NO DELETIONS under F1.b** (Artifact E: `created_rejects_org_as_resource.rs` STAYS — Org remains Principal-only under F1.b).
- EXTENDED: `domain/src/permissions/engine.rs` `#[cfg(test)] mod tests` (+~6-8 tests).
- EXTENDED: `domain/src/in_memory.rs` `#[cfg(test)] mod tests` (+~2 tests).
- EXTENDED: `domain/src/model/edges.rs` `#[cfg(test)] mod tests` (test rename + new constructor test + new variant assertion test).

**Named expected-still-green tests** (fragile, re-verified at chunk close):
- `edges::tests::edge_kind_names_is_exactly_seventy_two` — NEW invariant (renamed from `_seventy_one`); MUST pass post-edit.
- `action::tests::canonical_actions_are_exactly_thirty_four` (assuming similar test exists) — invariant CH-25 preserves (per F2).
- All existing Permission Check tests at `engine.rs:2229+` — synth-grant loop must not regress single-tier paths.
- All session-launch + session-preview tests — CheckContext field-add cascade must not regress.
- All `apply_org_creation` + `apply_project_creation` compound-tx tests — new edge emission must not break atomicity guarantees.
- **All 5 existing trybuild compile_fail fixtures** — MUST stay green (none deleted under F1.b).

---

## §9 — Pre-chunk gate

**Reading list (mandatory)**:
1. `docs/specs/v0/concepts/core-philosophy.md` (BINDING SPEC SOURCE, promoted 2026-04-28).
2. `docs/specs/v0/concepts/agent.md` (§"Agent spawns other Agents (Ownership)" amendment).
3. `docs/specs/v0/concepts/permissions/04-grants.md`.
4. `docs/specs/v0/concepts/permissions/01-resource-ontology.md`.
5. `docs/specs/v0/concepts/permissions/03-action-vocabulary.md` (verify 34-verb closed set; §3.D contradiction check).
6. `docs/specs/v0/implementation/m5_3/drifts/D-philosophy-01.md` (drift being closed).
7. `docs/specs/v0/implementation/m5_3/drifts/_concept-audit-matrix.md` (matrix to flip).
8. `docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` §2.5 lines 242-254.
9. `docs/specs/plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md` §4.1 (user clarification record).
10. `docs/specs/plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md` (M5.3 announcement plan archive).
11. `baby-phi/CLAUDE.md` phi-core Leverage section.
12. **Conditional (CH-11 retro v8)**: chunk touches `domain::permissions::engine` Step 2 body → reading list MUST include `server/src/platform/sessions/launch.rs` body + `server/src/platform/sessions/preview.rs` body. **Applied here.**
13. **Conditional (CH-12 retro v3 tag-write rule)**: not applicable — CH-25 does NOT introduce a tag-write Repository method.
14. **Conditional (v2 F1.b path)**: chunk introduces a new variant on a closed-set-protected enum (`Edge`); reading list MUST include `domain/src/model/edges.rs` body (especially the cardinality narrative at `:521-524` + the test at `:658-664`) + `domain/tests/m3_model_counts.rs` body. **Applied here.**
15. Prior ADRs cited as precedent: **ADR-0015** (`m1/decisions/0015-type-safe-ownership-edges.md`), **ADR-0008** (`m1/decisions/0008-permission-check-as-pipeline.md`), **ADR-0022** (`m3/decisions/0022-org-creation-compound-transaction.md`), **ADR-0034** (`m1/decisions/0034-...md`), **ADR-0053** (`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`).

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace` test count = **1537** (CH-24 baseline).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `scripts/check-spec-drift.sh` green.
- `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -j 4` green (0 warnings).
- `cargo fmt --all -- --check` green.
- `grep -rn "use phi_core" modules/crates/ | wc -l` = **57** (phi-core import baseline).
- `EDGE_KIND_NAMES.len() == 71` invariant green AT CHUNK OPEN (`cargo test --workspace edge_kind_names_is_exactly_seventy_one`) — flips to `_seventy_two` at chunk close.
- `Action::CANONICAL.len() == 34` invariant green.
- `modules/` diff against chunk-open git HEAD is empty (no preload edits).

**Pending decisions carried into this chunk**:
- All 6 forks user-locked at gate-1 (F1.b DIVERGENT; F2.a/F3.a/F4.a/F5.a/F6.b aligned).
- D-philosophy-01 will transition `discovered → remediated` at P-SEAL.
- One open sub-fork F1.b.subtle (Shape B both-approve creator semantics) may surface at P1 → user-lock if so.

**Chunk-ordering note (Q4)**: CH-25 is the first M5.3 carve-out chunk per forward-scope §2.5. Prerequisite CH-24 (M5 final seal) ✓ done. Independent of CH-23. User selected CH-25 to open next.

---

## §10 — Close criteria

**4 aspects (each graded pass/fail)**:

- **Code aspect**:
  - All P0+P-R5-INVESTIGATE+P1+P2+P3+P-SEAL deliverables shipped.
  - **NEW `Edge::Owns` variant** extant at `domain/src/model/edges.rs` (variant declaration + `Edge::name()` arm + `EDGE_KIND_NAMES` entry).
  - **NEW `OwnedResourceId` enum** extant at `domain/src/model/ids.rs` (or sibling) with Org/Project variants.
  - **NEW `Edge::new_owns` typed constructor** extant.
  - **71→72 cardinality flip** applied at all 4 enumerated sites (Artifact B per-file breakdown).
  - **Wire-mapping cascade fully addressed**: 0 wire-mapping functions found over `Edge` (per §3 Artifact B grep result); no enumerative additions required at wire-mapping layer.
  - **Permission Check `step_2_resolve_grants` extension shipped** (synth-grant loop reading `agent_owned_orgs/projects` from `CheckContext`).
  - **`apply_org_creation` + `apply_project_creation` emit `Edge::Owns` + `Edge::Created`** inside the same compound tx.
  - **`server/tests/acceptance_m5_3_owner_grant.rs` extant + passing**.
  - `cargo test --workspace` 1545-1555 range. RUSTFLAGS clippy 0 warnings. fmt --check green.
- **Docs aspect**:
  - *Governance tier*: ADR-0060 Accepted; D-philosophy-01 remediated; concept-audit-matrix rows flipped letter-for-letter from §2 target column; drift README index refreshed; verified-headers bumped.
  - *User-facing tier* (§3.C): `m5_3/architecture/agent-ownership-model.md` shipped (NEW); `m5_3/operations/agent-ownership-operations.md` shipped (NEW); `m5/user-guide/first-session-walkthrough.md` amended with CH-25 subsection; `permissions/04-grants.md` + `permissions/01-resource-ontology.md` updated with §-anchored subsections (the latter clarifies that under F1.b Org/Project STAY Principal-only).
  - *ADR-0060 §D60.1-§D60.6 sub-decisions all enumerated*; concept-doc `permissions/04-grants.md` updated with Owner-grant rule subsection; D-philosophy-01 status flipped to remediated + edge-count amendment §D60.6 META; R5 skill fix preservation note per chunk-planner v11 never-shipped-yet variation.
  - Doc-sync sweep (P3 deliverable 8) green: 0 stale `71 edge kinds` / `seventy-one` references in workspace docs.
- **phi-core leverage aspect**: baseline grep returns 57 (Δ +0 honored). `check-phi-core-reuse.sh` green. Forbidden-duplication greps return 0.
- **Concept alignment aspect**: every §2 row's target-status reached. D-philosophy-01 remediated. No new contradictions surfaced (or, if surfaced, captured as follow-up drifts with explicit `M*-DEFERRED-NN` allocation per chunk-planner v13 R4).

**2 confidence %**:

- **Implementation confidence %** target: **≥ 9.5/10 (95%)** — `(claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk)`. Example: *"22/23 = 96% (1 claim re Shape B both-approve creator semantics deferred to F1.b.subtle resolution at P1)."* If F1.b.subtle resolved cleanly, target lifts to **10/10**.
- **Documentation confidence %** target: **≥ 95%** — `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs) / (doc-pages-touched-in-chunk)`. Expected: 8/8 = 100%.

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-aspect-binary, concept-alignment-aspect-binary)**. Target: **≥ 95%**.

**P4 chunk-seal paperwork checklist** (per chunk-planner v3 + CH-11 retro + CH-12 retro + CH-17 retro Row 4):
- Every modified doc's verified-header description matches body diff exactly.
- Every `_concept-audit-matrix.md` row Status flipped letter-for-letter from plan §2 target column.
- `_cycle-index.md` row appended for cycle `1e01618e` per CH-17 retro Row 4 (Status `ready-for-audit`).
- **Cargo-clean discipline placement 1**: AFTER each `cargo test --workspace` invocation across the cycle (P1/P2/P3 chunk-implementer + Audit A/B/C + orchestrator gate-4 + retrospector permissions-audit script), invoker runs `cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` per CH-18 retro Row 1.

---

## §11 — Post-chunk independent audit plan

**Agent count**: **3 auditors** (large envelope; per F6.b user-lock). Per chunk-planner v8 `audit-envelope-size` skill + CH-24 F4.B precedent for code+ADR+ACL chunks. Phase count is 6; ADR-0060 has 6 sub-decisions (vs typical 4-5); concept-doc invariant flip (71→72) is a load-bearing cross-cutting change; R5 skill-fix is orthogonal — all motivate 3-auditor envelope.

**Audit aspects (a–e)**:
- (a) Code correctness — Audit A.
- (b) Docs fidelity vs concept docs — Audit B.
- (c) Concept alignment across every concept doc the chunk touched — Audit B.
- (d) phi-core leverage (imports, no forbidden duplications) — Audit A.
- (e) Cross-cutting (governance + ADR Pre-existing-behaviour preservation per chunk-planner v11 + R5 skill-fix verification + R2 source-map classification verification + R3 struct-placement verification + 71→72 cardinality-flip discipline) — Audit C.

### Audit A — Code correctness + phi-core leverage + edge-variant cascade (~600 words)

**Scope**: verify code shipped at P1 + P2 + P3 matches plan §7 deliverables + invariants hold under F1.b user-lock.

**Files to audit** (closed-set per R1 verification — closed-set members verified against shipping code at plan-draft):
1. `modules/crates/domain/src/model/edges.rs` — NEW `Owns { id, from: AgentId, to: OwnedResourceId }` variant present in Edge enum; `Edge::name()` arm returns `"OWNS"`; `EDGE_KIND_NAMES` cardinality 72 with `"OWNS"` entry; `Edge::new_owns` typed constructor present; test renamed `_seventy_two` + assertion 72.
2. `modules/crates/domain/src/model/ids.rs` (or sibling) — NEW `OwnedResourceId { Org(OrgId), Project(ProjectId) }` enum with required derives + `node_id()` helper.
3. `modules/crates/domain/src/model/mod.rs:84` — assertion `EDGE_KIND_NAMES.len() == 72`.
4. `modules/crates/domain/tests/m3_model_counts.rs:23,26` — comment + assertion updated to 72.
5. `modules/crates/domain/src/permissions/engine.rs` — `step_2_resolve_grants` body extended with synth-grant loop after line ~218; new unit tests pass.
6. `modules/crates/domain/src/permissions/manifest.rs` — `CheckContext` struct has `agent_owned_orgs` + `agent_owned_projects` fields.
7. `modules/crates/domain/src/repository.rs` — `list_agent_owned_orgs` + `list_agent_owned_projects` methods present; `OrgCreationPayload` + `ProjectCreationPayload` extended with `creator_agent`.
8. `modules/crates/domain/src/in_memory.rs` — `apply_org_creation` + `apply_project_creation` impls emit `Edge::new_owns` + `Edge::new_created`.
9. `modules/crates/store/src/...` — SurrealDB compound-tx bodies emit `RELATE owns` + `RELATE created`.
10. `modules/crates/server/src/platform/sessions/launch.rs` + `preview.rs` — CheckContext construction sites populate `agent_owned_orgs/projects` from new Repository methods.
11. `modules/crates/server/tests/acceptance_m5_3_owner_grant.rs` — present + green.

**Greps to run**:
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` → expect 57 (Δ +0).
- `grep -nE "Owns\b" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs` → expect ≥ 4 hits (variant declaration + Edge::name arm + EDGE_KIND_NAMES entry "OWNS" + new_owns constructor).
- `grep -nE "OwnedResourceId" /root/projects/phi/baby-phi/modules/crates/domain/src/` → expect ≥ 4 hits (enum declaration + Edge variant payload + Repository return-type pattern-matches + edges.rs uses).
- `grep -nE "step_2_resolve_grants|owner_grant|owner_resolve" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/engine.rs` → expect ≥ 2 hits (existing fn declaration + owner-grant synth-loop comment/marker).
- `grep -nE "Edge::new_owns" /root/projects/phi/baby-phi/modules/crates/` → expect ≥ 3 hits (1 constructor declaration + 2 production emit sites in apply_org_creation + apply_project_creation).
- `grep -rcE "EDGE_KIND_NAMES.len\(\) == 72\|\[&str; 72\]" /root/projects/phi/baby-phi/modules/crates/` → expect 4 hits (all 4 cardinality literal sites flipped).
- `grep -rcE "EDGE_KIND_NAMES.len\(\) == 71\|\[&str; 71\]" /root/projects/phi/baby-phi/modules/crates/` → expect 0 hits (none should remain post-flip).
- `grep -n "agent_owned_orgs\|agent_owned_projects" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/manifest.rs` → expect ≥ 2 hits.
- `grep -rn "Action::CANONICAL" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs | grep -c "\[Action; 34\]"` → expect 1 hit (invariant preserved).

**Pass criteria**: all 11 production deliverables present; 71→72 invariant evolution complete at all 4 sites; baseline test 1545-1555 range; clippy 0 warnings; fmt clean; trybuild compile-fail suite still green (all 5 existing fixtures + 1 optional new).

**R1 closed-set verification**: all 11 file targets verified at plan-draft via earlier greps — present in code OR explicitly NEW (P3 deliverable). No file named here is fictional.

### Audit B — Docs fidelity + concept alignment (~600 words)

**Scope**: verify docs shipped at P3 + P-SEAL match concept docs + ADR-0060 + drift D-philosophy-01.

**Files to audit**:
1. `docs/specs/v0/implementation/m5_3/decisions/0060-agent-as-creator-and-owner.md` — ADR Accepted; §D60.1–§D60.6 ratified; Forks header documents F1.b DIVERGENT lock; Cross-references all 4 categories (a)+(b)+(c)+(d).
2. `docs/specs/v0/implementation/m5_3/architecture/agent-ownership-model.md` — NEW; sections §1-§6 per P3 deliverable 1.
3. `docs/specs/v0/implementation/m5_3/operations/agent-ownership-operations.md` — NEW; sections §1-§3 per P3 deliverable 2.
4. `docs/specs/v0/implementation/m5/user-guide/first-session-walkthrough.md` — amended with CH-25 subsection.
5. `docs/specs/v0/concepts/permissions/04-grants.md` — §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" subsection added.
6. `docs/specs/v0/concepts/permissions/01-resource-ontology.md` — Org/Project STAY Principal-only clarification + NEW `OwnedResourceId` enum cross-ref.
7. `docs/specs/v0/implementation/m5_3/drifts/D-philosophy-01.md` — Status `remediated`; lifecycle entry appended; drift text `:27` cardinality reference amended (66 → 71 → 72).
8. `docs/specs/v0/implementation/m5_3/drifts/_concept-audit-matrix.md` — D-philosophy-01-tracking rows letter-for-letter flipped to plan §2 target.
9. `docs/specs/plan/build/_cycle-index.md` — CH-25 row appended (Status `ready-for-audit`).

**Concept-doc alignment greps**:
- `grep -n "Agent owns Organization\|Agent create Projects\|Agents own Resources" /root/projects/phi/baby-phi/docs/specs/v0/concepts/core-philosophy.md` → expect 3 hits (claims line 9, 23, 24).
- Verify each claim has a corresponding §2 row marked `honored` at chunk close.
- `grep -rn "71 variants\|seventy-one\|EDGE_KIND_NAMES.len() == 71" /root/projects/phi/baby-phi/docs/specs/v0/` → expect 0 hits post-doc-sync-sweep (P3 deliverable 8).

**Verified-header sanity**:
- `head -2 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/decisions/0060-agent-as-creator-and-owner.md` → first line is `<!-- Last verified: 2026-05-15 by Claude Code ... -->`.
- Same for all 7+ touched docs.

**Doc-link CI guard**: `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` → green (no broken cross-refs in ADR-0060's prior-ADR citations).

**Pass criteria**: all 9 doc files match plan; concept-audit-matrix rows letter-for-letter aligned; ADR-0060 cross-references all 4 categories present; doc-link CI green; ops-doc-headers CI green; F1.b user-lock divergence documented in ADR Forks header.

### Audit C — Cross-cutting (governance + Pre-existing-behaviour notes + R5 fix + R2/R3 + 71→72 verification) (~600 words)

**Scope**: cross-cutting verification of governance compliance + ADR Pre-existing-behaviour preservation rule (per chunk-planner v11) + R5 skill-fix verification + R2 source-map classification + R3 struct-placement dep-direction + 71→72 cardinality-flip discipline.

**Specific claims to verify**:

1. **ADR-0060 §D60.1 Pre-existing-behaviour preservation note** present + identifies (i) what was the case before this chunk (typed Edge constructors at ADR-0015 / M1 close), (ii) whether this chunk changes it (NO — adds new variant + constructor, does not modify existing OwnedBy/Created/AllocatedTo semantics), (iii) where the historical evidence lives.
2. **ADR-0060 §D60.2 Pre-existing-behaviour preservation note** present + identifies bootstrap-AR Wildcard usage pre-CH-25 + CEO Allocate usage pre-CH-25; CH-25 adds new synth-only auto-owner-grant.
3. **ADR-0060 §D60.3 Pre-existing-behaviour preservation note** present + identifies 6-step Permission Check pipeline pre-CH-25 + ADR-0008 + ADR-0051 + ADR-0054 precedents; CH-25 extends step_2_resolve_grants only.
4. **ADR-0060 §D60.4 Pre-existing-behaviour preservation note** uses multi-milestone-pattern variation (per chunk-planner v11) — `disable_system_agent` handler shipped at CH-01 / ADR-0034 §D34.4; CH-25 adds an acceptance test.
5. **ADR-0060 §D60.5 Pre-existing-behaviour preservation note** uses never-shipped-yet variation (per chunk-planner v11) — `permissions-audit` skill has never returned correct counts; CH-25 fixes the predicate.
6. **ADR-0060 §D60.6 META** documents the `EDGE_KIND_NAMES` cardinality evolution `66 → 71 → 72`; amendment to drift `D-philosophy-01.md:27` applied.
7. **R5 permissions-audit skill fix validation**: re-run the fixed jq pipeline against `.claude/tool-use.log` with CH-24 window — Audit C reports non-zero count. Cross-check against `grep -c "2026-05-11" /root/projects/phi/.claude/tool-use.log`.
8. **R2 source-map classification (per chunk-planner v13)**: verify §3 Artifact A's cited `engine.rs:2229` + `:2278` are PRODUCTION call construction sites; verify `engine.rs:2329` is TEST-FIXTURE. Verify §3 Artifact B's cited PRODUCTION-vs-TEST-FIXTURE labels for edges.rs/mod.rs/m3_model_counts.rs are correct.
9. **R3 struct-placement dep-direction verification (per chunk-planner v13)**: verify NEW `OwnedResourceId` enum lives in `domain::model::ids` (or sibling domain-tier location), NOT in `store::` or `server::` — domain-tier placement enables trait-return-type reference. Verify Repository trait return types `Vec<OrgId>` + `Vec<ProjectId>` reference domain-tier types. Verify `Edge::Owns` variant payload `from: AgentId, to: OwnedResourceId` references only domain-tier types.
10. **R4 drift M*-DEFERRED-NN allocation check (per chunk-planner v13)**: verify D-philosophy-01 transitions to terminal `remediated` (no allocation needed); verify any NEW follow-up drifts filed at P-R5-INVESTIGATE or P1/P2 cite explicit `M6-DEFERRED-NN` (never `TBD`).
11. **CI guards (per orchestrator gate-4 MUST-RUN)**: Audit C marks `bash scripts/check-*.sh` claims as `NOT-EXECUTED-IN-AUDIT` (sandbox-blocked); orchestrator closes at gate-4.
12. **R5 skill version bump**: `head -10 /root/projects/phi/.claude/skills/permissions-audit.md | grep -n "version: 4"` → expect 1 hit (skill bumped 3 → 4).
13. **71→72 cardinality-flip discipline**: all 4 enumerated cardinality literal sites flipped (`edges.rs:525` array decl + `:658` test name + `:662` assertion + `mod.rs:84` + `m3_model_counts.rs:23,26`). Test rename `_seventy_one → _seventy_two` applied. No straggler `71` literal references remain in `domain/` workspace tier.
14. **Philosophy alignment end-to-end**: concept-doc `core-philosophy.md` claims *"Agent owns Organization"* + *"Agent create Projects"* honored via NEW Owns + existing Created edge emissions; R5 skill fix verification.

**Pass criteria**: all 14 cross-cutting claims green; ADR-0060 Pre-existing-behaviour notes complete (5/5 sub-decisions + 1 META §D60.6); R5 fix validated empirically; R2/R3 verifications green; 71→72 flip complete at all enumerated sites; no `TBD` markers remain in any drift the chunk touched.

---

## §12 — Verification section (end-to-end recipe)

```bash
# 1. CI guards (run from absolute path per granular Bash discipline)
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (cap cargo jobs at 4 per feedback_cargo_jobs_cap)
/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets -j 4
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace -j 4

# 3. Chunk-specific invariants — phi-core leverage
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (Δ +0)

# 4. NEW Edge::Owns variant cascade
grep -nE "Owns\b" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs
# Expect: ≥ 4 hits (variant declaration + Edge::name arm + EDGE_KIND_NAMES "OWNS" entry + new_owns constructor)

grep -nE "OwnedResourceId" /root/projects/phi/baby-phi/modules/crates/domain/src/model/ids.rs
# Expect: ≥ 1 hit (enum declaration)

grep -nE "Edge::new_owns" /root/projects/phi/baby-phi/modules/crates/
# Expect: ≥ 3 hits (1 constructor decl + 2 production emit sites)

# 5. 71→72 cardinality-flip verification (per Artifact B 4-site enumeration)
grep -rcE "EDGE_KIND_NAMES.len\(\) == 72|\[&str; 72\]" /root/projects/phi/baby-phi/modules/crates/
# Expect: ≥ 4 hits across the 4 enumerated cardinality literal sites

grep -rcE "EDGE_KIND_NAMES.len\(\) == 71|\[&str; 71\]" /root/projects/phi/baby-phi/modules/crates/
# Expect: 0 hits (all flipped)

# 6. Permission Check engine extension
grep -n "step_2_resolve_grants\|owner_grant\|owner_resolve" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/engine.rs
# Expect: ≥ 2 hits (existing fn + new owner-grant synth-loop marker)

# 7. Action vocabulary invariant
grep -c "Action::CANONICAL" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/action.rs
# Expect: ≥ 2 (invariant preserved)

# 8. ADR + acceptance test extant
grep -c "Status: Accepted" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/decisions/0060*.md
# Expect: 1 (ADR Accepted at chunk-seal)

ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/decisions/
# Expect: NEW directory + ADR-0060 file present

ls /root/projects/phi/baby-phi/modules/crates/server/tests/acceptance_m5_3_owner_grant.rs
# Expect: file exists

# 9. Drift-file status
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-*.md | wc -l
# Expect: 1 (D-philosophy-01 remediated)

# 10. Acceptance test focused run
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p server --test acceptance_m5_3_owner_grant -j 4
# Expect: 1 acceptance test passes

# 11. R5 skill-fix validation
head -10 /root/projects/phi/.claude/skills/permissions-audit.md | grep "version: 4"
# Expect: 1 hit (skill bumped 3 → 4)

# Run the fixed jq pipeline against CH-24 cycle window
cat /root/projects/phi/.claude/tool-use.log /root/projects/phi/.claude/tool-use.log.* 2>/dev/null | jq -c 'select(.ts >= "2026-05-11T12:48:36Z" and .ts <= "2026-05-11T19:36:05Z" and .version == 1)' | wc -l
# Expect: ≥ 1290 (CH-24 retro spot-check baseline; non-zero validates fix)

# 12. Doc-sync sweep (P3 deliverable 8)
grep -rn "71 variants\|seventy-one\|EDGE_KIND_NAMES.len() == 71" /root/projects/phi/baby-phi/docs/specs/v0/
# Expect: 0 hits (all stale-cardinality references patched to 72)

# 13. Cleanup (per CH-18 retro Row 1 USER DIRECTIVE — placement-1 immediate-post-test)
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml
```

---

## Plan-revision history

- **v1** (2026-05-15, chunk-planner v13 iter-1) — initial draft; planner-recommended F1.a / F2.a / F3.a / F4.a / F5.a / F6.b.
- **v2** (2026-05-15, chunk-planner v13 iter-2) — re-plan after gate-1 user-lock divergent F1.b (NEW `Edge::Owns` variant on Edge enum) from planner-recommended F1.a (reuse OWNED_BY/CREATED + relax Resource trait). Preserves all other 5 locks (F2.a / F3.a / F4.a / F5.a / F6.b — aligned with planner-recommendation). Cross-cycle divergence count refreshed 7-of-9 → **8-of-10 (80%)**. F1.b cascade footprint mapped via Artifact B re-write: ~9 production edits + ~5 test-fixture edits across 4 files; zero workspace-wide wire-mapping cascade (no `match Edge` exhaustive blocks outside `Edge::name()` itself). §3.E gate-2.5 candidates refreshed (C3 doc-sync sweep + C4 D-philosophy-01 cardinality amendment + C5 SurrealDB owns relation migration). ADR-0060 §D60.1 rewritten + §D60.6 added (cardinality-evolution META). §6 new row for 71→72 invariant evolution. §10 close criteria expanded with edge-variant cascade aspect.
