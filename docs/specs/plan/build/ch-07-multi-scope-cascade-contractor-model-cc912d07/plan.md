<!-- Last verified: 2026-05-07 by Claude Code (orchestrator plan-approval gate; user locked F1.A / F2.A / F3.A / F4.A via AskUserQuestion; F2 lock conditioned on orchestrator-explained multi-scope tag-resolution mechanism per concept 06 lines 14–53 + listeners.rs:698–712 verification; cycle hex `cc912d07`) -->
<!-- Last verified: 2026-05-07 by Claude Code (chunk-planner v4 iter-1, cycle hex `cc912d07`) -->

# CH-07 — Multi-scope cascade + contractor model

**Plan file token:** `cc912d07` (generated via `openssl rand -hex 4`).
**Plan archive path:** `baby-phi/docs/specs/plan/build/ch-07-multi-scope-cascade-contractor-model-cc912d07/plan.md`. Archived at chunk-open Step 0 (2026-05-07).
**Chunk ID:** CH-07 (forward-scope §1 lines 84–89).
**Severity:** ⚠HIGH.
**Expected effort:** ~3 engineer-days (forward-scope estimate).
**Chunks unblocked at close:** none critical (standalone Permission-engine quality improvement).

---

## Forks for orchestrator

Four open forks. **F1 + F3 are LOCKED-FORK candidates that require user adjudication before plan-approval; F2 + F4 carry planner-recommendations that are likely auto-approvable but surface-worthy.**

> **User-lock outcome (2026-05-07, via AskUserQuestion at plan-approval gate)**:
> - **F1 → F1.A** (2-tier + tie-breaker, concept-doc verbatim).
> - **F2 → F2.A** (read multi-scope from `session.tags`, no Session shape change). User asked how F2.A resolves multi-org/multi-project sessions; orchestrator confirmed: cascade parses `org:<uuid>` / `project:<uuid>` tag prefixes from `session.tags` (canonical encoding per concept 06 lines 14–53). Launch handler computes `session_org_tags` / `session_project_tags` slices into CheckContext. Test fixtures construct Shape B/C tag sets directly. Production-side Shape B/C session-creation path is CH-15's territory (forward-scope deliverable). Back-compat for Shape A/D today: single tag → cascade reduces to single-tier behavior.
> - **F3 → F3.A** (Step-2a-style re-clamp + new `DeniedReason::IntersectionEmpty`).
> - **F4 → F4.A** (`DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count }`).
>
> ADR-0051 §"Forks" header records these lock outcomes per CH-13 retro Row 1.

### F1 — Cascade-tier framing: "5-tier" linear chain vs "2-tier × tie-breaker" (LOCKED-FORK)

The forward-scope row reads "Project → Org → base_project → base_org → intersection fallback" (5 entries). The canonical concept docs disagree:

- **Concept 04 lines 357–367** lays out **2 main tiers** (Project, Org), each with an internal `count() == 1 / count() > 1 / count() == 0` branch where `base_project` / `base_org` is a **tie-breaker WITHIN the tier**, not its own tier. Plus an `intersection` fall-through.
- **Concept 06 lines 38–53** (the `resolve_scope` fn — "Unified Resolution Rule") matches concept 04 exactly: project-level match-count branch → org-level match-count branch → intersection.
- **Concept 04 line 310** explicitly states: *"Ties within a tier are broken by the base-org rule."* — base_X is a TIE-BREAKER, not a TIER.

**Options:**
- **F1.A (planner-recommended)** — Implement as **2 main tiers + intersection** matching concept 04 + 06 verbatim. Encode the inner branching as `match reader_project_matches.count() { 1 => …, n if n > 1 => base_project_tiebreak, 0 => fall_through }` — same shape for the org tier. Forward-scope row's "5-tier" wording is treated as informal labeling for "5 distinct outcomes" (single-project, base-project tiebreak, single-org, base-org tiebreak, intersection); the implementation honours concept 06's 2-tier control flow. ADR-0051 §"Cascade tier framing" pins the canonical reading and notes the forward-scope phrasing as discrepant labeling, not a directive.
- **F1.B** — Implement literally as 5 ordered tiers (Project → Org → BaseProject → BaseOrg → Intersection). This **does not match either concept doc** — concept 04 line 363 says "fall through" from the Project tier directly to the Org tier, NOT through a "BaseProject tier" intermediate. F1.B would silently reorder the cascade and let, say, a reader who is in `0` of the session's projects but is `base_project` of a non-session-tagged project get matched at the BaseProject-tier — which is nowhere supported by the doc.
- **F1.C** — Defer to user with a stronger formulation: re-read forward-scope with the user, surface the discrepancy, request a corrected forward-scope wording before opening the chunk.

**Planner recommendation: F1.A**, with explicit ADR-0051 §"Cascade tier framing" §D51.1 sub-decision pinning concept 06's `resolve_scope` pseudocode as the normative shape.

### F2 — Multi-scope session shape: extend `Session` to carry `Vec<OrgId>` / `Vec<ProjectId>` NOW, or rely on `Session.tags` only (LOCKED-FORK)

Today (verified at `nodes.rs:1234-1235`):
```rust
pub struct Session {
    ...
    pub owning_org: OrgId,            // singular
    pub owning_project: ProjectId,    // singular
    pub tags: Vec<String>,            // CH-06 instance + governance tags (line 1249)
    ...
}
```

The cascade needs to enumerate the session's owning scopes (Shape B = 1 project + 2 orgs; Shape C = 2+ projects + 1 org; Shape D = 0 projects + 1 org). Two implementation paths:

- **F2.A (planner-recommended)** — **Cascade reads scope-tags from `session.tags`**, parsed via prefix scan (`org:<uuid>` / `project:<uuid>`). No `Session` struct change. Rationale:
  1. Concept 06 lines 14–26 + 06 lines 30–53 (`resolve_scope`) read the session's scopes from `session.tags` directly ("`session_projects = session.tags.filter(Tag::Project)`").
  2. CH-06 already shipped instance-tag emission for sessions; today's session tags include `project:<id>` + `org:<id>` (verified at `events/listeners.rs:703-704`).
  3. Shape B/C/D session creation paths don't ship at this chunk (forward-scope CH-07 deliverable list doesn't mention session multi-scope creation; Shape E enforcement at `in_memory.rs:1350` already constrains ProjectShape).
  4. The `owning_org` / `owning_project` singular fields stay as the **primary scope** for Shape A/D (single-org single-project — the dominant path); the cascade fallback to `session.tags` activates only when the tag set carries 2+ org/project tags.
  5. **Zero migration**, zero data-shape change → smaller blast radius.
  6. The CheckContext extension (F2.A) carries `session_org_tags: Vec<OrgId>` + `session_project_tags: Vec<ProjectId>` parsed by the launch handler from `session.tags` before constructing `CheckContext`. Engine internals stay tag-list-shaped; storage stays singular-shaped.

- **F2.B** — Add `Vec<OrgId>` / `Vec<ProjectId>` directly to `Session`. Migration 0013. Cascade reads from those fields directly. Rejected because (a) no Shape B/C session-creation path is being written in CH-07, (b) the data is already encoded in `session.tags` per CH-06, (c) doubling the encoding invites drift, (d) CH-15 (real launch-time manifest validator) is the right home for any Session-shape change. **Adds a migration the planner tries to avoid.**

- **F2.C** — Defer cascade to a future chunk and ship only contractor-model Step 2a. Rejected — D-new-06 + D-new-20 are paired in the forward-scope row; halving the chunk would leave Multi-Scope half-shipped.

**Planner recommendation: F2.A** with explicit ADR-0051 §"Multi-scope session encoding" §D51.4 pinning "tags-as-source-of-truth" and the launch-handler parse contract.

### F3 — Intersection fallback semantic: synthetic `ResolvedGrant` vs new `Decision::Denied(IntersectionFallback)` shape (LOCKED-FORK)

Concept 04 line 369–371:
> *"Intersection fallback (outsider): apply the intersection of all the session's scope ceilings."*

Concept 06 line 51:
```
0 => ResolvedScope::Intersection(session_orgs),
```

Concept 06 line 62:
> *"Empty-intersection case: When the intersection contains no action the outsider's grants can exercise … the effect is **deny by default**. The outsider receives a `Denied` Permission Check result with `failed_step: 3 (no grant covers this reach)`."*

The doc tells us **what** intersection means semantically (apply the strictest of all session-scope ceilings to the candidate grants) but **does not** prescribe a concrete `ResolvedGrant` shape for the fallback. Two paths:

- **F3.A (planner-recommended)** — Intersection fallback is implemented as a **Step-2a-style ceiling re-clamp** at Step 5: when `reader_project_matches.count() == 0 && reader_org_matches.count() == 0`, the cascade collects the union of `org_grants` from the session's tagged orgs (read from the new `session_org_tags` channel) and re-runs the `step_2a_ceiling`-shape logic against the candidate set, treating each tagged-org's grants as a ceiling. The winner — if any survives — is returned as a normal `ResolvedGrant`. Empty-intersection → `Decision::Denied { failed_step: FailedStep::Scope, reason: DeniedReason::IntersectionEmpty }`. This **reuses existing Step-2a machinery** (zero new mental model), and concept 06 line 62 explicitly endorses Step-3-shape denial semantics ("Denied" with `failed_step: 3`). Note: the concept doc uses "failed_step: 3" but our actual encoding here is `FailedStep::Scope` (Step 5 territory) since our pipeline's `failed_step` enum encodes which step short-circuited; this is a fidelity-preserving translation since the doc's "3" refers to the step in its abstract ordering, not our enum.
- **F3.B** — Synthesise a fresh `ResolvedGrant { fundamentals: ∩, selector: Selector::Any, kind_refinement: None, grant: <synthetic phantom> }` to carry the intersection through Step 4 (constraints) + Step 6 (consent). Rejected: synthetic grants violate the "Grant always traces back to a real `descends_from` provenance" invariant (concept 02 § "System Bootstrap Template"); breaks audit; breaks revocation cascade. Phantom grants would cause CH-13's `audit_class_source` resolution to assign a meaningless source.
- **F3.C** — Always Deny on intersection (no actual matching). Rejected: concept 06 lines 60–62 explicitly say the outsider faces the intersection and CAN succeed if their grants exercise an action that all session-scope ceilings permit (concept 06 line 60 — "the intersection of all the session's scope ceilings"). F3.C would over-restrict.

**Planner recommendation: F3.A** + a new `DeniedReason::IntersectionEmpty` variant on the existing `DeniedReason` enum. ADR-0051 §"Intersection fallback semantic" §D51.5 pins the Step-2a-reuse approach.

### F4 — `DeniedReason` enum additive variant naming (advisory)

Per the additive-enum-cascade discipline (chunk-planner v3, CH-12 retro): a new `DeniedReason::IntersectionEmpty` variant is additive; downstream `match` sites use `_ =>` exhaustively (verified at `decision.rs` Display impl + `engine.rs` constraint_violation site — see §3 grep table). Predicted **0 additive callsite cascade**. Naming options:

- **F4.A (planner-recommended)** — `DeniedReason::IntersectionEmpty { fundamental: Fundamental, action: Action, session_scope_count: u8 }` — mirrors `NoMatchingGrant` field shape, plus a 1-byte counter that's useful in audit logs without leaking specific OrgIds.
- **F4.B** — `DeniedReason::OutsiderDenied { ... }` — anchors on concept 06 line 60's "outsider rule" terminology. Rejected: the enum is internally an engine-emit; the concept-doc framing is "intersection," and "outsider" is a doc-scoped framing for the reader's role, not the engine state.
- **F4.C** — Reuse `DeniedReason::NoMatchingGrant` — distinguishable only by a logged context. Rejected: concept 06 line 62 says "Denied result with failed_step: 3 (no grant covers this reach)," but the engine's audit + observability story benefits from a separate variant for intersection-fallback to keep the mental model clean.

**Planner recommendation: F4.A**.

---

## §1 — Context & principle

### Why this chunk

Concept doc **`permissions/04-manifest-and-resolution.md` §"Mechanism 2: Scope Resolution"** (lines 354–375) and **`permissions/06-multi-scope-consent.md` §"Unified Resolution Rule"** (lines 28–63) jointly specify a 2-tier (Project / Org) cascade with `base_X` tie-breakers and an intersection-fallback for outsiders. Today's `step_5_scope_resolution` (engine.rs:317–337) is a **single-tier** resolver — it sorts candidates by `ScopeTier` (Agent → Project → Org), keeps top-tier, and ties-breaks by `(issued_at, id_bytes)`. **It has zero awareness of:**

1. The session's multi-scope tag set (concept 06 Shape B/C sessions tagged with multiple `org:` or `project:` tags).
2. The reader's membership across the session's tagged scopes.
3. The `base_project` / `base_org` tie-breaker for multi-match readers.
4. The intersection fallback for outsiders (readers who are in 0 of the session's tagged scopes).

`step_2a_ceiling` (engine.rs:236) is also membership-blind: today it filters every candidate against the entire `ceiling_grants` slice uniformly — there is no "ceiling only applies if reader is a member of that ceiling's scope" check. Per concept 06 lines 161–166 ("Subject-Side Reach Is Bounded by Scope Membership") and concept 08 line 289 ("the reader's base_org is irrelevant when a project-level resolution succeeds"), a contractor's home-org ceiling **must not** clamp a candidate when the contractor is operating inside another scope. This is the `D-new-20` contractor-model gap.

CH-07 closes both drifts: `step_5_scope_resolution` becomes the full 2-tier-plus-tie-breaker-plus-intersection cascade; `step_2a_ceiling` gains a membership-bounded clamp. Acceptance tests reproduce concept 08 worked-example scenarios 4 (`lead-acme-1` → org:acme), 5 (`lead-beta-1` → org:beta-corp), 6 (`lead-gamma-1` → intersection-Denied), and the contractor scenario (`contractor-x-9` operating under acme rules — concept 08 lines 289–298).

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* Applied: concept 06's `resolve_scope` pseudocode is the normative spec. The Rust implementation must be reviewable against it line-by-line; the four worked scenarios in concept 08 (4, 5, 6, contractor) are golden test fixtures, not illustrative prose. The forward-scope row's "5-tier" labeling is informal — the canonical reading is concept 04's 2-tier-plus-tiebreaker-plus-fallback shape (locked via F1.A).

### Forward-scope reference

[§1 CH-07 row](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 84–89. No §4 critical-path entry (CH-07 unblocks nothing critical).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (paraphrase / key line numbers) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/README.md`](../../../v0/concepts/permissions/README.md) | (entry invariants) | Permissions subtree entry — scope-resolution claim mapped to §04 + §06 cross-ref | honored | honored (re-verified) |
| [`permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Mechanism 2: Scope Resolution" lines 354–375 | 2-tier cascade (project / org), `base_X` tie-breaker, intersection fallback for outsiders | partially-honored (single-tier resolver only) | honored |
| [`permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Key Invariants" line 310 | "Ties within a tier are broken by the base-org rule" | silent-in-code | honored (encoded explicitly in `step_5_scope_resolution`) |
| [`permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Refinement" lines 393–425 (`ResolvedGrant`) | `ResolvedGrant` carries fundamentals + effective_selector — unchanged shape, cascade reads it | honored | honored (no new fields needed for cascade) |
| [`permissions/06-multi-scope-consent.md`](../../../v0/concepts/permissions/06-multi-scope-consent.md) | §"The Hard Schema Constraint" lines 13–26 | Shape A/B/C/D allowed; Shape E forbidden. Multi-scope sessions tagged with multiple `org:` / `project:` tags | honored | honored (cascade reads from `session.tags`, no new shape work) |
| [`permissions/06-multi-scope-consent.md`](../../../v0/concepts/permissions/06-multi-scope-consent.md) | §"The Unified Resolution Rule" lines 28–63 (incl. `resolve_scope` pseudocode) | 2-tier cascade with project-tier first, org-tier fallback, base-X tiebreak, intersection-fallback | partially-honored (cascade weak) | honored |
| [`permissions/06-multi-scope-consent.md`](../../../v0/concepts/permissions/06-multi-scope-consent.md) | §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166 | Reader's base_org ceiling does NOT reach into sessions of scopes they aren't a member of (contractor-model security boundary) | silent-in-code | honored |
| [`permissions/08-worked-example.md`](../../../v0/concepts/permissions/08-worked-example.md) | §"Step 4: Multi-Scope resolution for joint-research" lines 192–222 | Scenario 4 (lead-acme-1 → org:acme), Scenario 5 (lead-beta-1 → org:beta-corp), Scenario 6 (lead-gamma-1 → intersection-Denied) | partially-honored (no acceptance coverage) | honored (3 acceptance tests) |
| [`permissions/08-worked-example.md`](../../../v0/concepts/permissions/08-worked-example.md) | §"Step 7: Contractor scenario" lines 287–298 | `contractor-x-9` (base_org=Gamma) reads sessions in `acme-website-redesign`, gets project-level resolution, base_org irrelevant | silent-in-code | honored (1 acceptance test) |
| [`permissions/08-worked-example.md`](../../../v0/concepts/permissions/08-worked-example.md) | §"Summary: Who Can Read What" line 383 | `lead-gamma-1` joint-research column = ✗ (intersection fallback) | partially-honored | honored (matrix row test) |
| [`phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) | (no overlap) | The cascade is a baby-phi internal scope-resolution concern; phi-core has no `Session.tags` / scope-resolution surface. | N/A — no phi-core overlap | N/A |

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | The cascade is a domain-level Permission-Check concern. `domain::permissions::*` carries 0 `phi_core::` imports today (verified: `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ → 0 hits`). phi-core has no scope-resolution / session-tag concept (per `concepts/phi-core-mapping.md` — sessions in phi-core are an execution-trace primitive; baby-phi's `Session` wraps `phi_core::session::Session` for governance-state extension). | N/A — no overlap | keep orthogonal |

**Expected import-count delta at chunk close**: **0** phi-core imports added in `modules/crates/domain/src/permissions/` (engine.rs + manifest/mod.rs + decision.rs).

**Positive close-audit greps:**
```bash
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l
# expect 0 (matches chunk-open baseline)

grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/ | wc -l
# expect 10 (matches chunk-open baseline of 10 — verified 2026-05-07)
```

**Forbidden-duplication greps:**
```bash
grep -rnE "^struct ResolvedGrant\b|^pub struct ResolvedGrant\b" /root/projects/phi/baby-phi/modules/crates/ | grep -v "domain/src/permissions/expansion.rs"
# expect 0 — single canonical definition stays at expansion.rs:56
```

### §3 cascade artifacts (per chunk-planner v3 — CH-13 retro Row 2 discipline)

CH-07 touches `Candidate`, `ResolvedGrant`, `ReachKey`, `step_2a_ceiling` signature, `step_5_scope_resolution` signature, and `CheckContext`. Below: the 3-artifact treatment for every load-bearing struct/enum/function.

#### Artifact A — `step_5_scope_resolution` callsite cascade

(a) Invocation:
```bash
git grep -nE 'step_5_scope_resolution' /root/projects/phi/baby-phi/modules/crates/
```
(b) Raw match count: **2** matches.
(c) Per-file breakdown:
- `modules/crates/domain/src/permissions/engine.rs:96` — call site in `check_inner`
- `modules/crates/domain/src/permissions/engine.rs:317` — definition

**Predicted cascade if signature changes (e.g., +`session_org_tags` parameter via CheckContext):** signature change rides through `_ctx: &CheckContext<'_>` (already a parameter) — no new positional parameter at the public signature. Estimated edit sites: **1 callsite + 1 definition body + tests**. Pause if actual cascade > 5 sites.

#### Artifact B — `step_2a_ceiling` callsite cascade

(a) Invocation:
```bash
git grep -nE 'step_2a_ceiling' /root/projects/phi/baby-phi/modules/crates/
```
(b) Raw match count: **2** matches.
(c) Per-file breakdown:
- `modules/crates/domain/src/permissions/engine.rs:81` — call site in `check_inner`
- `modules/crates/domain/src/permissions/engine.rs:236` — definition

**Predicted cascade if signature changes** (e.g., +`session_org_tags: &[OrgId]` parameter for membership-bounded clamping): callsite + definition + tests. Plan: extend the function with an additional `session_org_tags: &[OrgId]` parameter rather than changing the existing signature in-place. **2 edit sites + tests.** Pause if actual cascade > 5 sites.

#### Artifact C — `Candidate` struct cascade

(a) Invocation:
```bash
git grep -nE '\bCandidate\b' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/
```
(b) Raw match count: **20** matches.
(c) Per-file breakdown:
- `engine.rs:191-218` — definition + `step_2_resolve_grants` (the constructor)
- `engine.rs:236-249` — `step_2a_ceiling` consumer
- `engine.rs:277-307` — `step_3_match_reaches` consumer
- `engine.rs:317-337` — `step_5_scope_resolution` consumer
- `engine.rs:342-360` — `tie_break_within_tier` consumer

**Predicted cascade**: NO new field on `Candidate`. The cascade reads membership-information from `CheckContext` (which carries `session_org_tags` / `session_project_tags`), not from `Candidate` itself. Candidate's `tier: ScopeTier` is sufficient to identify which scope a candidate came from. **0 struct-field edits.** Pause if actual cascade > 5 sites (would indicate a structural rethink).

#### Artifact D — `CheckContext` struct cascade (the BIGGEST blast radius)

(a) Invocation:
```bash
git grep -nE 'CheckContext' /root/projects/phi/baby-phi/modules/crates/
```
(b) Raw match count: **45** matches across the workspace.
(c) Per-file breakdown (high-impact construction sites):

```
modules/crates/domain/src/permissions/engine.rs                 13 matches (definition refs + tests)
modules/crates/domain/src/permissions/manifest/mod.rs            6 matches (definition + impls)
modules/crates/domain/tests/...                                  ~6 matches (acceptance tests)
modules/crates/server/src/platform/sessions/launch.rs            ~4 matches (production construction)
modules/crates/server/src/platform/sessions/preview.rs           ~4 matches (preview construction)
modules/crates/server/src/platform/agents/handler_support.rs     ~5 matches (agent-tool construction)
modules/crates/domain/src/in_memory.rs                           ~4 matches (test fixtures)
modules/crates/store/tests/                                      ~3 matches
```

To get exact per-file numbers, the chunk-planner verified by running:
```bash
git grep -lnE 'CheckContext' /root/projects/phi/baby-phi/modules/crates/
```
Match files: at least 8 distinct files carry `CheckContext` references.

**Predicted cascade if CheckContext gains 2 new lifetime-bound fields** (`session_org_tags: &'a [OrgId]`, `session_project_tags: &'a [ProjectId]`): every construction site must populate the new fields. Construction sites identified:

1. `engine.rs::Fixture::ctx` (1 in-engine test fixture) — 1 edit
2. `manifest/mod.rs` (definition only — 1 edit to add fields with doc-comments)
3. `server::platform::sessions::launch.rs` — 1+ construction site (production launch)
4. `server::platform::sessions::preview.rs` — 1 construction site (preview)
5. `server::platform::agents::handler_support.rs` — 1+ test fixture site
6. `domain::tests::*` — 1+ acceptance tests (likely several files)
7. CH-11 P-CH11 reading-list conditional (preview-path manifest-resource bug per chunk-template v2026-05-03): **all preview/launch construction sites must populate the new fields with empty slices when the session has no multi-scope tags**, OR populate them from the session-tag parse.

**Predicted cascade size**: **8–14 edits across ~6 files.** Pause if actual cascade > 21 sites (1.5× upper bound) — would indicate construction-site sprawl beyond the predicted touch surface.

**Defensive default**: introduce both new fields with `#[serde(default)]`-shielded slices that default to empty (`&[]`). Construction sites that don't carry session-tag awareness yet (preview, handler_support test fixtures) keep working — empty slices mean "no multi-scope sessions, fall through to existing single-org/project semantics," matching today's behaviour exactly for Shape A/D sessions. **This preserves M1 callsite-count test invariants outside of the cascade-aware launch path.**

#### Artifact E — `DeniedReason` additive variant cascade (per chunk-planner v3 additive-enum discipline)

(a) Invocation for `match` over DeniedReason:
```bash
git grep -nE 'match.*DeniedReason' /root/projects/phi/baby-phi/modules/crates/
```
(b) Raw match count: **2** matches.
(c) Per-file breakdown:
- `modules/crates/domain/src/permissions/decision.rs` — Display impl (uses explicit per-variant arms — needs +1 arm for `IntersectionEmpty`)
- `modules/crates/domain/src/permissions/decision.rs` — `DeniedReason::as_metric_label` (uses explicit per-variant arms — needs +1 arm)

**Predicted cascade**: `DeniedReason` has explicit-per-variant matches (NOT `_ =>` catch-all) for Display + metric-label. Adding `IntersectionEmpty` requires **2 explicit arms.** Other consumers (HTTP error mapping in `server` crate) likely use `From<E>` → HTTP via `Display`, predicted **0** additional edits per the additive-enum discipline. The planner will run a final `git grep -nE 'DeniedReason::'` at implementer-handoff to confirm.

#### Artifact F — `FailedStep` enum (no change predicted)

CH-07's intersection-empty case maps to `FailedStep::Scope` (existing variant — engine.rs returns it from Step 5). **0 enum changes.** Verified at decision.rs (existing variant in use).

---

## §3.B — K8s microservice readiness check

**Source-of-truth: [`m7b/architecture/k8s-microservices-readiness.md`](../../v0/implementation/m7b/architecture/k8s-microservices-readiness.md). Tactical ledger: [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md).**

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`/`RwLock`/`AtomicBool`/`Mutex`/`OnceCell`) | Pure-fn additions to `engine.rs`. Verified: `grep -nE "DashMap\|RwLock\|AtomicBool\|Mutex\|OnceCell\|RefCell" engine.rs` returns only the existing `Mutex<Vec<...>>` inside the `metrics_recorded_on_every_decision` test fixture (line 1096). No new in-process state. | **no** | — |
| **A2** | New IPC channel (`mpsc`/`broadcast`/`oneshot`/`watch`/`Notify`) | None — engine is pure-fn, no async, no channels. | **no** | — |
| **A3** | New pod-local resource (file handle, listener, sub-process, lock file) | None — pure logic. | **no** | — |
| **A4** | Migration runner / first-apply race | **No new migration.** Cascade reads from existing `Session.tags: Vec<String>` (added at CH-06 migration 0008); `step_2a_ceiling` consumes existing `CheckContext.org_grants` slice. No SurrealDB schema change. | **no** | — |
| **A5** | Trait-shape requirement (broker/Redis/remote-DB swap friendliness) | Engine functions are free-fn pure-fn. Already trait-object-friendly via `&dyn CatalogueLookup` + `&dyn SetRefRegistry` (existing). No new traits needed; cascade reads from `&[OrgId]` slices. | **no** | — |
| **A6** | Cross-pod state sharing (data must be visible across pods?) | None — all cascade inputs are derived from durable `Session` row (SurrealDB-persisted) + durable `Grant` rows. No pod-local data introduced. The launch handler computes `session_org_tags` / `session_project_tags` per-request from the durable session row. Stateless. | **no** | — |
| **A7** | Audit hash-chain symmetry — does the chunk add a new audit writer? | **No new audit-event emitter.** Cascade emits no audit events; engine returns Decision; Decision propagation up through callsites uses existing audit paths (set in CH-13 ADR-0050 §D50.1). The new `IntersectionEmpty` denied-reason flows through the same `Decision::Denied` → audit-event-mapping path as every other denied reason. | **no** | — |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — not touched. ✓
- D33.2 (`SurrealStore::open_remote`) — no new storage operations. ✓
- D33.3 (SIGTERM graceful shutdown) — no new `tokio::spawn`. ✓
- D33.4 (`EventBus.shutdown` + `drain`) — no new `EventBus` emitters/listeners. ✓

**Conclusion**: **K8s-neutral**. CH-07 introduces no new K8s-deployment hurdles. Pure-fn extensions of an existing pure-fn pipeline.

---

## §3.C — User-facing documentation impact map

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m5_2/architecture/<feature>.md` — design, data flow | New file: `architecture/multi-scope-cascade.md` (cascade design + tier/tie-breaker/intersection encoding + contractor-bound diagram + ADR-0051 cross-ref) | (a) **update in-chunk** — P3 deliverable |
| **Operations** | `docs/specs/v0/implementation/m5_2/operations/<feature>-operations.md` — error codes, audit-event dictionary, playbooks | New file: `operations/multi-scope-cascade-operations.md` documenting `DeniedReason::IntersectionEmpty` (new error code), the contractor-bound denial path, troubleshooting tree for "why was my outsider denied?" / "why didn't my base_org ceiling apply?" | (a) **update in-chunk** — P3 deliverable |
| **User-guide** | `docs/specs/v0/implementation/m5_2/user-guide/<feature>-walkthrough.md` — operator tours, CLI, error codes | No new operator-visible CLI command. The cascade is engine-internal; no UI-visible behaviour shifts beyond Decision-shape semantics already documented at concept-doc 06 + 08. **Defer with reason**: no new operator-affordance lands at CH-07. Successor chunk: M6+ admin-side multi-scope dashboard or CH-15 (real launch-time enforcement) — whichever first surfaces a user-visible affordance. | (b) **defer** with successor reference: CH-15 (real launch-time hard-gate) OR M6 admin dashboard, whichever first ships an operator-visible Multi-Scope affordance |

The 2 in-chunk doc updates are P3 deliverables alongside concept-audit-matrix updates + verified-header bumps. The user-guide deferral is **explicit** with successor reference per CH-22 codification.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| D-new-06 | [`../../v0/implementation/m5_1/drifts/D-new-06.md`](../../v0/implementation/m5_1/drifts/D-new-06.md) | HIGH | discovered → **remediated** | Full 2-tier cascade (project → org) + base_X tie-breakers + intersection fallback shipped at `step_5_scope_resolution`; ADR-0051 (CH-07) drafted + flipped Accepted at P4. |
| D-new-20 | [`../../v0/implementation/m5_1/drifts/D-new-20.md`](../../v0/implementation/m5_1/drifts/D-new-20.md) | MEDIUM | discovered → **remediated** | Contractor-model membership bound shipped at `step_2a_ceiling` (gated by `session_org_tags` slice on CheckContext); ADR-0051 §D51.6 pins the bound semantics. |

No new drifts anticipated. If implementation surfaces a new drift mid-flight (e.g., a launch-handler-shape misalignment that was invisible at plan time), it will be added here per the §6 mid-flight pause rule.

---

## §5 — ADRs drafted

### ADR-0051 — Multi-scope cascade + contractor-model membership bound

- **Path**: `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`
- **Status at plan draft**: Proposed
- **Drafted at phase**: P1 (Proposed); flipped to **Accepted** at P4 (chunk-seal)
- **Closes**: D-new-06 (HIGH) + D-new-20 (MEDIUM)

**Decision summary** (one line): The 2-tier cascade (project → org) with base_project / base_org tie-breakers and Step-2a-style intersection fallback for outsiders, plus a membership-bounded ceiling clamp encoding the contractor-model security boundary, lands at `domain::permissions::engine::{step_5_scope_resolution, step_2a_ceiling}` with both functions consuming new `session_org_tags` / `session_project_tags` slices threaded through `CheckContext`.

**Sub-decisions to be pinned in ADR-0051 body** (drafted at P1, finalized at P4):
- **D51.1** — Cascade tier framing: 2 main tiers + tie-breaker + intersection fallback (concept 04 + 06 verbatim). Forward-scope row's "5-tier" labeling acknowledged as informal naming for "5 distinct outcomes," not 5 distinct tiers in the control flow.
- **D51.2** — `step_5_scope_resolution` signature: `pub fn step_5_scope_resolution(matches: HashMap<ReachKey, Vec<Candidate>>, ctx: &CheckContext<'_>) -> Result<HashMap<ReachKey, ResolvedGrant>, Decision>` — the `_ctx` underscore prefix removed; the function consumes `ctx.session_org_tags` + `ctx.session_project_tags` to determine reader's per-scope membership.
- **D51.3** — `step_2a_ceiling` signature: `pub fn step_2a_ceiling(candidates: Vec<Candidate>, ceiling_grants: &[Grant], session_org_tags: &[OrgId]) -> Vec<Candidate>` — gains the third parameter to bound ceilings to the agent's actual membership in session-tagged scopes.
- **D51.4** — Multi-scope session encoding: tags-as-source-of-truth via `Session.tags` (no `Vec<OrgId>` shape change to Session). Launch handler parses `org:<uuid>` / `project:<uuid>` tag prefixes into the new CheckContext slices. **No migration.** (F2.A locked.)
- **D51.5** — Intersection fallback semantic: Step-2a-style ceiling re-clamp at Step 5; empty intersection → `Decision::Denied { failed_step: FailedStep::Scope, reason: DeniedReason::IntersectionEmpty }`. (F3.A locked.)
- **D51.6** — Contractor-model membership bound: at `step_2a_ceiling`, a ceiling grant only clamps a candidate when the ceiling's owning scope appears in `session_org_tags` (membership check). Concept 06 line 162 verbatim: "an agent's home org (`base_organization`) does not reach into sessions belonging to scopes the agent is not a member of."
- **D51.7** — `DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count }` additive variant; `_ =>` catch-all NOT used at decision.rs Display + metric-label sites; `From<DeniedReason>` HTTP error mapping in `server` crate continues to use Display.

**ADR Forks header** (per CH-13 retro Row 1, chunk-planner v3 ADR-body checklist):
```
Forks (all planner-recommended at chunk-open; user-locked at plan approval to F1.A / F2.A / F3.A / F4.A)
```
OR (if user diverges at plan-review):
```
Forks (F1 user-locked to F1.B at plan approval — diverges from planner recommendation F1.A; F2/F3/F4 at planner-recommendation A-options)
```
The actual lock-state will be filled in by P1 implementer when the chunk-planner returns the user-decision.

**ADR Cross-references header** (per CH-13 retro Row 1 ADR-body checklist):
- (a) **Originating concept-doc + sections**: `permissions/04-manifest-and-resolution.md` §"Mechanism 2: Scope Resolution" lines 354–375; `permissions/06-multi-scope-consent.md` §"The Unified Resolution Rule" lines 28–63 + §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166; `permissions/08-worked-example.md` §"Step 4" lines 192–222 + §"Step 7" lines 287–298.
- (b) **Closed drifts**: D-new-06 (HIGH), D-new-20 (MEDIUM).
- (c) **Prior ADRs cited as precedent**: ADR-0036 (selector grammar — provides the tag-predicate machinery the cascade reads `session.tags` through), ADR-0048 (per-session consent — precedent for adding `&[OrgId]`-shaped slices to CheckContext), ADR-0050 (audit-class composition — precedent for additive enum variants without migration), ADR-0033 (CH-K8S-PREP conforming criteria — referenced for K8s-neutral verification).
- (d) **Forward-scope row cross-reference**: `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 84–89 (CH-07 row).

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| CH-06 | `Session.tags` carries `org:<uuid>` / `project:<uuid>` prefixes per CH-06 instance + governance tag emission. | `grep -nE 'format!\("org:\|format!\("project:' /root/projects/phi/baby-phi/modules/crates/domain/src/events/listeners.rs` — expect ≥ 2 hits at lines 703–704 (governance tag-build for memory). |
| CH-06 | Selector grammar predicates (PEG) parse cleanly; cascade does not need to evaluate `tags contains` predicates against session.tags directly (cascade walks tag prefix scan, not selector eval). | `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` (CH-06 doc invariants); `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain selector` — expect green. |
| CH-09 | `Consent` node + consent index unchanged by cascade. | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain consent` — expect 0 failures. |
| CH-10 | `ConsentState` 6-state machine + sweeper unchanged. | Same as above. |
| CH-11 | Per-Session consent gating + `Grant.approval_mode` + `CheckContext.current_session` unchanged. Cascade runs Step 5 BEFORE Step 6, so consent gating evaluates against the resolved winner — cascade must produce a winner that is structurally compatible. Existing 17 Step-6 tests at engine.rs:1156–1449 must stay green. | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain step_6` — expect ≥ 17 passing tests. |
| CH-12 | Frozen-tag enforcement + `validate_tag_write_on_session` unchanged (cascade is a read-path concern; tag-write is orthogonal). | `grep -rn 'validate_tag_write_on_session' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/` — expect ≥ 1 hit at the validator module. |
| CH-13 | `Grant.audit_class` + `compose_audit_class` unchanged. The cascade winner inherits its audit_class from the grant; no new composition path. | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain audit_composition` — expect green. |
| CH-K8S-PREP (ADR-0033) | `&dyn CatalogueLookup` + `&dyn SetRefRegistry` trait-shape preserved on CheckContext. Cascade adds 2 new slice fields; no new trait. | `grep -nE 'pub set_ref_registry|pub catalogue' /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/manifest/mod.rs` — expect existing trait-object lines unchanged. |
| (workspace baseline) | All 1379 existing tests pass; clippy under `-Dwarnings` green; fmt green. | `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace 2>&1 \| grep -E "test result:" \| awk '{p+=$4} END {print p}'` — expect 1379 baseline at chunk-open + delta at chunk-close. |

This table runs AT CHUNK OPEN before P1 begins, and again at chunk seal (P4). Any regression produces a new drift file + AskUserQuestion before the chunk proceeds.

---

## §7 — Phases within the chunk

CH-07 splits into **5 phases** (P0 setup, P1–P3 implementation, P4 seal):

### P0 — Cycle archive + ADR scaffolding

- **Goal**: Archive plan, scaffold ADR-0051 stub (Proposed), confirm baselines green.
- **Deliverables**:
  1. Plan already archived at `baby-phi/docs/specs/plan/build/ch-07-multi-scope-cascade-contractor-model-cc912d07/plan.md` (this file).
  2. `baby-phi/docs/specs/v0/implementation/m5_2/decisions/0051-multi-scope-cascade-contractor-model.md` — minimal scaffold (Proposed status; Forks header empty; sub-decision headers stubbed). Body fills at P1 + finalises at P4.
  3. Cycle index updated at `baby-phi/docs/specs/plan/build/_cycle-index.md`.
- **Tests**: none (paperwork-only).
- **Concept-alignment check**: row "scope resolution cascade" remains `partially-honored` (no code change yet).
- **phi-core leverage check**: row 1 unchanged (0 imports baseline).
- **User-facing doc updates**: none in P0; deferred to P3.
- **Confidence target**: 100% (scaffolding).
- **Pause discipline**: none.

### P1 — `step_5_scope_resolution` 2-tier cascade body + `IntersectionEmpty` denied-reason variant

- **Goal**: Ship the full cascade in `step_5_scope_resolution`. Add `DeniedReason::IntersectionEmpty` variant.
- **Deliverables**:
  1. `domain::permissions::engine::step_5_scope_resolution` rewritten:
     - Reads `ctx.session_org_tags: &[OrgId]` + `ctx.session_project_tags: &[ProjectId]` from CheckContext.
     - For each `(reach_key, candidates)` pair:
       - **Project-tier resolution**: `reader_project_matches = session_project_tags ∩ {tier == ScopeTier::Project candidates' source-project}`. Branch on count:
         - `count == 1`: pick that project's candidate(s); tie-break within using existing `tie_break_within_tier`.
         - `count > 1`: pick `base_project` (currently no `base_project` field on Agent; FORK F2 leaves `base_project` resolution at "use the project the candidate's grant attributes to" — see "open question for ADR-0051 §D51.2 — base_project field is forward-deferred; M6+ tracking via D-CH07-FOLLOWUP-01").
         - `count == 0`: fall through to org-tier.
       - **Org-tier resolution**: similar 3-branch, with `base_org` tie-breaker (same forward-defer note as `base_project`).
       - **Intersection fallback**: when both tiers fall through (`count == 0` at both), invoke a Step-2a-style re-clamp against the union of `org_grants` from session-tagged orgs. If non-empty: tie-break + return winner. If empty: `Decision::Denied { failed_step: FailedStep::Scope, reason: DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count } }`.
  2. `domain::permissions::decision::DeniedReason` — add `IntersectionEmpty { fundamental: Fundamental, action: Action, session_scope_count: u8 }` variant.
  3. `decision.rs` Display impl + metric-label arm — 2 explicit arms added (no `_ =>` catch-all per F4.A).
  4. `domain::permissions::manifest::CheckContext` extension: add `pub session_org_tags: &'a [OrgId]` + `pub session_project_tags: &'a [ProjectId]` fields with doc-comments cross-referencing concept-doc 06 lines 30–53.
  5. **base_project / base_org M6+ deferral**: ADR-0051 §D51.2 + §D51.3 explicitly defer the Agent.base_project / Agent.base_org Vec field additions to a future M6+ chunk. Today, when `count > 1` at either tier, the implementation **picks the lexicographically-smallest tagged scope** as a deterministic fallback and emits a doc-comment `// FIXME(D-CH07-FOLLOWUP-01): tie-breaker deferred to M6 — using lexicographic ordering as deterministic placeholder per ADR-0051 §D51.2`. New drift file `D-CH07-FOLLOWUP-01.md` is created at P4 with status `discovered` and deferral target M6+.
- **Tests** (added):
  - `cascade_project_tier_single_match_picks_project_scope` — Shape A baseline, reader is in 1 of session's projects.
  - `cascade_project_tier_zero_match_falls_through_to_org` — reader in 0 of session's projects, has org match.
  - `cascade_project_tier_multiple_matches_picks_lexicographic_min` — placeholder for base_project tie-breaker (FIXME-tagged).
  - `cascade_org_tier_single_match_picks_org_scope` — Scenario 4 (lead-acme-1 → org:acme).
  - `cascade_org_tier_other_org_match` — Scenario 5 (lead-beta-1 → org:beta-corp).
  - `cascade_org_tier_multiple_matches_picks_lexicographic_min` — placeholder for base_org tie-breaker.
  - `cascade_intersection_fallback_outsider_denied` — Scenario 6 (lead-gamma-1 → IntersectionEmpty).
  - `cascade_intersection_fallback_outsider_with_universal_grant_allows` — outsider with a wildcard grant that survives the intersection clamp → Allowed.
  - `denied_reason_intersection_empty_display_is_stable` — Display string format pinning.
  - `denied_reason_intersection_empty_metric_label_is_stable` — metric label pinning.
- **Concept-alignment check**: rows for concept 04 §"Mechanism 2" + concept 06 §"Unified Resolution Rule" + concept 08 §"Step 4" flip from `partially-honored` → `honored`.
- **phi-core leverage check**: row 1 unchanged (0 imports added).
- **User-facing doc updates**: none in P1 (deferred to P3).
- **Confidence target**: ≥ 97%.
- **Pause discipline**: PAUSE if cascade callsite cascade > 5 sites (Artifact A) or CheckContext construction-site cascade > 21 sites (Artifact D 1.5×). Also PAUSE on any Step-6 consent-gating test regression — they are CH-11's invariants.

### P2 — `step_2a_ceiling` membership-bounded clamp

- **Goal**: Ship the contractor-model security bound at Step 2a.
- **Deliverables**:
  1. `step_2a_ceiling` signature extended: `pub fn step_2a_ceiling(candidates: Vec<Candidate>, ceiling_grants: &[Grant], session_org_tags: &[OrgId]) -> Vec<Candidate>`.
  2. Body: a ceiling grant only clamps a candidate when the ceiling's owning-scope (derived from `Grant.holder` per `PrincipalRef::Organization(_)`) appears in `session_org_tags`. **Empty `session_org_tags` slice** = "single-org / Shape A / Shape D" path → preserves M1 behaviour exactly (every ceiling clamps, matching today). **Non-empty** = membership-bounded clamping kicks in.
  3. Callsite at `engine.rs:81` updated to pass `ctx.session_org_tags`.
- **Tests** (added):
  - `step_2a_membership_bound_excludes_non_member_org_ceiling` — Scenario 7 contractor: contractor's base_org=Gamma ceiling does NOT clamp candidates when session is tagged `[org:acme]` only.
  - `step_2a_membership_bound_keeps_non_member_org_ceiling_when_session_tags_empty` — back-compat: empty `session_org_tags` → today's uniform clamping behaviour.
  - `step_2a_membership_bound_with_multi_org_session_clamps_only_member_orgs` — Shape B: session tagged `[org:acme, org:beta-corp]`; reader's Acme + Gamma ceilings → Acme clamps, Gamma doesn't.
  - `contractor_model_acceptance_test_per_concept_08_step_7` — End-to-end test reproducing concept 08 §"Step 7" lines 287–298 (`contractor-x-9` reading sessions in `acme-website-redesign`, base_org=Gamma irrelevant). Uses real CheckContext + StaticCatalogue + ConsentIndex.
- **Concept-alignment check**: rows for concept 06 §"Subject-Side Reach Is Bounded" + concept 08 §"Step 7" flip from `silent-in-code` → `honored`.
- **phi-core leverage check**: unchanged (still 0 imports).
- **User-facing doc updates**: none in P2.
- **Confidence target**: ≥ 97%.
- **Pause discipline**: PAUSE if `step_2a_ceiling` callsite cascade > 5 sites (Artifact B). PAUSE if any of CH-11's 17 Step-6 tests regress (re-verify via `cargo test step_6`).

### P3 — Acceptance tests + user-facing doc updates + concept-audit-matrix

- **Goal**: Comprehensive acceptance coverage matching concept 08 worked-example scenarios + ship the in-chunk user-facing docs.
- **Deliverables**:
  1. **Acceptance test file**: `modules/crates/domain/tests/multi_scope_cascade_acceptance.rs` (new file). Reproduces:
     - Scenario 4: `lead-acme-1` reading joint-research session → org:acme resolution → Allowed (with Acme grant).
     - Scenario 5: `lead-beta-1` reading joint-research session → org:beta-corp resolution → Allowed (with Beta grant + one_time consent placeholder).
     - Scenario 6: `lead-gamma-1` reading joint-research session → IntersectionEmpty Denied.
     - Scenario 7: `contractor-x-9` reading `acme-website-redesign` session, base_org=Gamma, session has only `org:acme` tag → project-tier resolution succeeds, Allowed.
     - Shape A baseline: single-project single-org session, reader is owner → project-tier picks. (Regression-protection.)
     - Shape D system session: 0 projects, 1 org, system reader → org-tier picks.
  2. **Architecture doc**: `baby-phi/docs/specs/v0/implementation/m5_2/architecture/multi-scope-cascade.md` (new file).
     - 2-tier cascade diagram + base_X tie-breaker note + intersection-fallback semantic.
     - Contractor-bound diagram (concept 06 line 162 verbatim quote).
     - Cross-refs to ADR-0051, concept 04 + 06 + 08, drift files D-new-06 + D-new-20.
     - Header: `<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P3) -->`.
  3. **Operations doc**: `baby-phi/docs/specs/v0/implementation/m5_2/operations/multi-scope-cascade-operations.md` (new file).
     - Error-code dictionary entry: `DeniedReason::IntersectionEmpty` (when fired, what to log, how to triage).
     - Audit-event mapping (no new events; all flow through existing `Decision::Denied` audit path).
     - Troubleshooting tree: "Outsider unexpectedly denied" / "Contractor's base_org ceiling unexpectedly applied/not applied".
     - Header: `<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P3) -->`.
  4. **Concept-audit-matrix update** — `m5_1/drifts/_concept-audit-matrix.md`:
     - Row 178 `5-tier scope cascade` (currently `partially-honored`) → status flipped to `honored`. Evidence cell updated with `engine.rs::step_5_scope_resolution` reference + ADR-0051 link + cross-ref to D-new-06 ✓.
     - Row 202 `Unified resolve_scope cascade` (currently `partially-honored`) → `honored`.
     - Row 208 `Contractor model` (currently `silent-in-code`) → `honored`. Evidence cell: `engine.rs::step_2a_ceiling` membership bound + ADR-0051 §D51.6.
     - Row 226 `Shape A/B/C resolution per worked examples` → `honored`.
     - Row 227 `Contractor scenario` → `honored`.
     - **Discipline**: status values copy-pasted letter-for-letter from §2 above per CH-12 retro Row 1 + chunk-template P4 paperwork addendum.
  5. **Concept doc verified-header bumps** (no body changes — alignment-only):
     - `permissions/04-manifest-and-resolution.md`: prepend new verified-header `<!-- 2026-05-07 — CH-07 amendment: §"Mechanism 2: Scope Resolution" lines 354–375 lifted into typed Rust at domain::permissions::engine::step_5_scope_resolution (full 2-tier cascade body) per ADR-0051. Doc body unchanged. -->`.
     - `permissions/06-multi-scope-consent.md`: prepend new verified-header noting `§"Unified Resolution Rule" lines 28–63 + §"Subject-Side Reach Is Bounded" lines 161–166 lifted into typed Rust at domain::permissions::engine::{step_5_scope_resolution, step_2a_ceiling}` per ADR-0051.
     - `permissions/08-worked-example.md`: prepend new verified-header noting `§"Step 4" lines 192–222 + §"Step 7" lines 287–298 covered by acceptance tests per ADR-0051`.
- **Tests** (added):
  - 6 acceptance tests in the new test file (one per scenario above).
- **Concept-alignment check**: every row in §2 above flips to its target status.
- **phi-core leverage check**: unchanged (still 0 imports).
- **User-facing doc updates**: §3.C Tier 1 (Architecture) + Tier 2 (Operations) shipped; Tier 3 (User-guide) deferred per §3.C row 3.
- **Confidence target**: ≥ 97%.
- **Pause discipline**: PAUSE if any concept-audit-matrix row's target status doesn't apply letter-for-letter. PAUSE on AskUserQuestion if a doc-update reveals a mid-flight concept contradiction.

### P4 — Chunk-seal paperwork

- **Goal**: Flip all paperwork to closed state; verify the close-criteria composite.
- **Deliverables**:
  1. ADR-0051 status flipped from `Proposed` to `Accepted` with the actual user-locked Forks header line (filled per F1/F2/F3/F4 lock outcome).
  2. Drift files:
     - `D-new-06.md` — Lifecycle history appended: `2026-05-07 — remediated — CH-07 cycle cc912d07 — full 2-tier cascade + intersection fallback shipped at engine.rs::step_5_scope_resolution per ADR-0051`.
     - `D-new-20.md` — Lifecycle history appended: `2026-05-07 — remediated — CH-07 cycle cc912d07 — membership-bounded clamp shipped at engine.rs::step_2a_ceiling per ADR-0051 §D51.6`.
     - `D-CH07-FOLLOWUP-01.md` — NEW LOW drift created. Captures the M6+ deferral of `Agent.base_project: Vec<ProjectId>` + `Agent.base_org: Vec<OrgId>` field additions (per ADR-0051 §D51.2/D51.3). Status `discovered`, severity `LOW`, deferral target `M6+`. Cross-ref to ADR-0051 §D51.2.
     - Drifts README index — D-new-06 row Status → `**remediated**` + chunk → `CH-07 ✓`; D-new-20 row Status → `**remediated**` + chunk → `CH-07 ✓`; D-CH07-FOLLOWUP-01 added as new row.
  3. **P4 verified-header / matrix audit checklist** (per chunk-template v2026-05-03 + v2026-05-04):
     - For every modified doc: re-read line 1 verified-header + body diff, confirm header description matches diff.
     - For every concept-audit-matrix row touched: confirm Status value is letter-for-letter from §2 target (not paraphrased / interpretation).
  4. Cycle index `_cycle-index.md` updated with cycle close-row.
- **Tests** (re-run): full workspace `cargo test` → expect 1379 + ~16 = ~1395 passing (see §8).
- **Concept-alignment check**: every §2 target-status confirmed.
- **phi-core leverage check**: `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l` → 0 (unchanged from baseline).
- **User-facing doc updates**: P4 paperwork checklist run on the 2 new docs from P3.
- **Confidence target**: ≥ 99%.
- **Pause discipline**: PAUSE on any verified-header / matrix-Status mismatch caught at the P4 checklist.

---

## §8 — Tests summary

### Expected total test count at chunk close

**Baseline at chunk-open**: **1379 passed / 0 failed / 2 ignored** (verified via `cargo test --workspace 2>&1 | grep -E "test result:" | awk '{p+=$4; f+=$6; i+=$8} END {print "passed=" p " failed=" f " ignored=" i}'`).

**Per-deliverable test count breakdown:**

| Phase | Test source | Count |
|---|---|---|
| P1 | Cascade unit tests (engine.rs `mod tests`) — 10 tests listed in P1 | **10** |
| P2 | Step 2a membership bound unit tests (engine.rs `mod tests`) — 4 tests listed in P2 | **4** |
| P3 | Acceptance tests (`tests/multi_scope_cascade_acceptance.rs`) — 6 tests | **6** |
| **Sum** | | **20 new tests** |

**Asymmetric ×1.0–×1.20 buffer band** (per chunk-template v2026-05-04 + CH-12 retro):
- Lower bound: 20 × 1.0 = **20** new tests (1379 + 20 = **1399 total**)
- Upper bound: 20 × 1.20 = **24** new tests (1379 + 24 = **1403 total**)

**Implementer-accept band**: actual passing test count at chunk-close should land in **[1399, 1403]**. Outside this band → AskUserQuestion before chunk seal.

### Layer breakdown

- **Unit (engine.rs `mod tests`)**: 14 (10 from P1 + 4 from P2)
- **Acceptance (`tests/multi_scope_cascade_acceptance.rs`)**: 6 (P3)
- **Integration / e2e**: 0 (no HTTP / DB-roundtrip tests; cascade is pure-fn)
- **Property**: 0 (no proptest in this chunk; the determinism property is covered by the explicit lexicographic-min tie-break tests)

### Named test files

- **Modified**: `modules/crates/domain/src/permissions/engine.rs` (the `#[cfg(test)] mod tests` block — 14 new tests).
- **New**: `modules/crates/domain/tests/multi_scope_cascade_acceptance.rs` — 6 acceptance tests.

### Named expected-still-green tests

- **CH-11 Step-6 tests** (17 tests at engine.rs:1156–1449) — all must stay green, since cascade runs BEFORE Step 6.
- **Existing engine pipeline tests** at engine.rs:752–1116 — `step_0_*`, `step_1_*`, `step_2_*`, `step_3_*`, `engine_allows_*`, `engine_denies_*`, `agent_tier_beats_project_and_org_in_scope_cascade`, `metrics_recorded_on_every_decision`. All must stay green.
- **CH-06 selector grammar tests** — unchanged surface, but the cascade may exercise `session.tags` parsing — expect green.
- **CH-13 audit-class composition tests** — cascade winner inherits `audit_class` from grant; expect green.

---

## §9 — Pre-chunk gate

### Reading list (mandatory — completed during plan-draft)

1. **Concept docs** cited in §2:
   - `permissions/README.md` — entry invariants. ✓
   - `permissions/04-manifest-and-resolution.md` — full read of §"Mechanism 2: Scope Resolution" + §"Key Invariants" (lines 305–375 in particular). ✓
   - `permissions/06-multi-scope-consent.md` — full doc read; emphasis on §"Hard Schema Constraint" + §"Unified Resolution Rule" + §"Subject-Side Reach Is Bounded by Scope Membership". ✓
   - `permissions/08-worked-example.md` — full doc read; emphasis on §"Step 4: Multi-Scope resolution" + §"Step 7: Contractor scenario". ✓
   - `phi-core-mapping.md` — confirmed no overlap. ✓
2. **Drift files** cited in §4:
   - `D-new-06.md` — full read. ✓
   - `D-new-20.md` — full read. ✓
3. **Prior-chunk plan** for CH-06 (the only listed prerequisite per forward-scope):
   - `baby-phi/docs/specs/plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md` (lines 1–80 read; selector grammar + instance-tag emission shipped, confirmed). ✓
4. **Forward-scope §5 + §7**: read CH-07 row at lines 84–89. ✓
5. **`baby-phi/CLAUDE.md` phi-core Leverage section**: read; cascade adds 0 imports, no overlap. ✓
6. **Conditional (CH-11 retrospective rule)**: chunk touches `domain::permissions::engine` Step 5 + Step 2a — required reading list extension:
   - `server/src/platform/sessions/launch.rs` — read for synthetic-manifest construction sites + CheckContext construction. The launch path is where the new `session_org_tags` / `session_project_tags` slices will be populated from `session.tags` parse. Verified the construction-site cascade artifact (Artifact D §3) covers this.
   - `server/src/platform/sessions/preview.rs` — read for the preview-path manifest-resource shape (CH-11 D4.1 was a preview-path bug). Plan: pass empty `&[]` slices from preview construction sites — Shape A/D-only behaviour preserved. ✓
7. **CLAUDE.md granular Bash discipline**: read; §12 verification recipe uses granular forms (one logical operation per Bash invocation). ✓

### Carry-forward invariants (verified green at chunk open)

- ✓ `cargo test --workspace` test count = **1379 passed / 0 failed / 2 ignored** (verified).
- ✓ `bash scripts/check-phi-core-reuse.sh` green (per CH-13 close — re-verify at P4).
- ✓ `bash scripts/check-doc-links.sh` green (per CH-13 close).
- ✓ `bash scripts/check-ops-doc-headers.sh` green (per CH-13 close).
- ✓ `bash scripts/check-spec-drift.sh` green (per CH-13 close).
- ✓ `modules/` diff against `git HEAD` is empty (verified — `git status` returned clean at plan-draft time).

### Pending decisions carried into this chunk

- **F1 / F2 / F3 / F4 locks** (see top "Forks for orchestrator" section) — orchestrator escalates to user before plan-approval.
- **D-new-06 + D-new-20** transition `discovered → remediated` at P4.
- **D-CH07-FOLLOWUP-01** new drift opens `discovered` at P4 (M6+ deferral of `Agent.base_project` + `Agent.base_org` Vec field additions for proper tie-breaking).

### Chunk-ordering note

Per Q4: user picked CH-07 to open after CH-13 close. CH-06 is the only prerequisite per forward-scope row line 87, and is sealed (verified at `baby-phi/docs/specs/plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md`).

---

## §10 — Close criteria

Composite 4-aspect + 2 confidence-% ritual.

### 4 aspects (each pass / fail)

- **Code aspect** — pass when:
  - All P0–P4 deliverables shipped per §7.
  - `/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace` → 1399–1403 passing (per §8 asymmetric band).
  - `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets` green.
  - `/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check` green.

- **Docs aspect** — pass when:
  - **Governance tier**: D-new-06 + D-new-20 status `remediated`; D-CH07-FOLLOWUP-01 created `discovered`; ADR-0051 `Accepted`; concept-audit-matrix rows 178/202/208/226/227 status updated letter-for-letter from §2 (per CH-12 retro Row 1).
  - **User-facing tier**: §3.C Tier 1 (Architecture) + Tier 2 (Operations) docs shipped at `m5_2/architecture/multi-scope-cascade.md` + `m5_2/operations/multi-scope-cascade-operations.md` with verified-header + cross-refs. Tier 3 (User-guide) deferred per §3.C with successor reference (CH-15 OR M6 admin dashboard).
  - **P4 paperwork checklist** run: every modified verified-header description matches body diff; every matrix row Status value letter-for-letter from §2.

- **phi-core leverage aspect** — pass when:
  - `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l` → 0 (unchanged baseline).
  - `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` green.

- **Concept alignment aspect** — pass when:
  - Every §2 row's target-status achieved.
  - Concept doc 04 + 06 + 08 verified-header bumps applied with no body drift.

### 2 confidence percentages

- **Implementation confidence target**: **≥ 9/10** (`claims-honored / total-claims-in-scope-for-chunk`). Numerator: each row of §2 + each ADR §D51.X sub-decision + each P1/P2 test + each P3 acceptance test + each user-facing doc tier. Denominator: 11 §2 rows + 7 ADR sub-decisions + 14 unit tests + 6 acceptance tests + 2 user-facing docs + 5 governance paperwork artifacts = **45 claims**. Target: **≥ 41 of 45 honored = 91%**.
- **Documentation confidence target**: **≥ 8/8 = 100%** for doc pages where independent reader can cross-check against code + concept-doc + ADRs. Pages: ADR-0051, multi-scope-cascade.md (architecture), multi-scope-cascade-operations.md (operations), concept-audit-matrix.md update, drift D-new-06.md update, drift D-new-20.md update, drift D-CH07-FOLLOWUP-01.md (new), cycle-index.md update.

### Composite

`min(impl%, doc%, code-pass, phi-core-pass, concept-pass)` ≥ **91%** to seal.

### Locked forks resolution

The 4 forks at the top of this plan are escalated to the user via the orchestrator. **Plan does not auto-approve until F1, F2, F3 are user-locked**; F4 is advisory. Each user-locked fork's outcome is recorded in ADR-0051 §"Forks" header per chunk-planner v3 ADR-body checklist (CH-13 retro Row 1).

### Repository tag-write contract conditional (per chunk-planner v3 — CH-12 retro)

CH-07 introduces no new tag-write Repository method. The cascade is read-side; it consumes `session.tags` rather than mutating them. **The Repository trait docstring contract bullet (CH-12 ADR-0049 §D49.5/§D49.7) does NOT apply at this chunk.** No reading-list entry from `repository.rs:19-48` is required.

---

## §11 — Post-chunk independent audit plan

### Agent count

CH-07 has **5 phases** → **medium chunk** → **2 audit agents** per audit-envelope-size skill + chunk-template §11 guardrail (4–6 phases = 2 agents).

### Audit agents

#### Audit A — Code correctness + phi-core leverage (single agent)

**Scope**:
- (a) Code correctness: cascade body, contractor-bound, Decision shape changes.
- (d) phi-core leverage: zero new imports in `permissions/`, no forbidden duplications.

**Files audited**:
- `modules/crates/domain/src/permissions/engine.rs` (full file)
- `modules/crates/domain/src/permissions/decision.rs` (Display + metric-label + new variant)
- `modules/crates/domain/src/permissions/manifest/mod.rs` (CheckContext extension)
- `modules/crates/domain/tests/multi_scope_cascade_acceptance.rs`
- ADR-0051 body for code-claims accuracy (file paths, signatures, sub-decision invariants pinned by tests)

**Greps to run**:
```bash
git grep -nE 'step_5_scope_resolution|step_2a_ceiling' /root/projects/phi/baby-phi/modules/crates/
git grep -nE 'IntersectionEmpty' /root/projects/phi/baby-phi/modules/crates/
git grep -nE 'session_org_tags|session_project_tags' /root/projects/phi/baby-phi/modules/crates/
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/
```

**Pass criteria**:
- Cascade body matches concept 06 lines 30–53 line-by-line (with documented translation to typed Rust).
- Contractor-bound is structurally enforced at `step_2a_ceiling` per ADR-0051 §D51.6.
- Concept 04 + 06 + 08 quoted line numbers match latest doc state.
- 0 phi-core imports in `permissions/`.
- All 20 new tests pass; CH-11's 17 Step-6 tests still green.
- Workspace test count in [1399, 1403] band (§8).

**Audit prompt** (≤ 600 words):
```
You are an independent code-correctness + phi-core-leverage auditor for CH-07 (multi-scope cascade + contractor model). Read the cycle plan at <plan-path> §3, §3.C, §7, §11 first. Then read engine.rs, decision.rs, manifest/mod.rs, the new acceptance test file, and ADR-0051. Verify:

1. step_5_scope_resolution body implements concept 06 lines 30–53 verbatim shape (project-tier match-count branch → org-tier match-count branch → intersection fallback). Cite the matching code lines for each pseudocode block.
2. step_2a_ceiling body bounds clamping by `session_org_tags` membership. Concept 06 line 162 quote: 'an agent's home org (base_organization) does not reach into sessions belonging to scopes the agent is not a member of.' — confirm the implementation enforces this via membership check on `Grant.holder == PrincipalRef::Organization(org)` AND `session_org_tags.contains(&org)`.
3. DeniedReason::IntersectionEmpty has explicit Display + metric-label arms (no `_ =>` catch-all per F4.A).
4. CheckContext field additions are `&'a [OrgId]` / `&'a [ProjectId]`, defaulting to empty slices for legacy callers.
5. Run `git grep -n "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/` — expect 0 hits.
6. Run `git grep -nE "^pub struct ResolvedGrant\b"` — expect exactly 1 hit at `expansion.rs:56`.
7. Spot-check 3 of the 20 new test bodies; confirm they exercise the claim-line under test.
8. The MUST-RUN cargo + clippy + 4 CI guards lines: mark NOT-EXECUTED-IN-AUDIT (sandbox-blocked); orchestrator runs at final cycle re-audit gate.

Format: per-claim PASS/FAIL with file:line citation. Final verdict: GREEN / ARCHITECTURAL-FAIL / TACTICAL-FAIL / TRIVIAL-FAIL. Output a markdown audit log. Word budget ≤ 1500 words for the report body. Write to <audit-A-iter-N-path>.
```

#### Audit B — Docs fidelity + concept alignment (single agent)

**Scope**:
- (b) Docs fidelity vs concept docs.
- (c) Concept alignment across every concept doc the chunk touched.
- Drift-file lifecycle correctness.
- Concept-audit-matrix Status values letter-for-letter (per CH-12 retro Row 1).

**Files audited**:
- `permissions/04-manifest-and-resolution.md`, `permissions/06-multi-scope-consent.md`, `permissions/08-worked-example.md` (verified-header amendments + body unchanged confirmation).
- `m5_2/architecture/multi-scope-cascade.md` (new file).
- `m5_2/operations/multi-scope-cascade-operations.md` (new file).
- `m5_1/drifts/_concept-audit-matrix.md` rows 178/202/208/226/227.
- `m5_1/drifts/D-new-06.md`, `D-new-20.md`, `D-CH07-FOLLOWUP-01.md`.
- `m5_2/decisions/0051-multi-scope-cascade-contractor-model.md` (Forks header + Cross-references checklist per CH-13 retro Row 1).
- `_cycle-index.md` row.

**Pass criteria**:
- Every concept-doc verified-header bump matches its body diff (no overpromise).
- Every concept-audit-matrix row Status value is **letter-for-letter** from §2 target (no paraphrase).
- ADR-0051 `Forks` header line correctly captures user-lock outcome (planner-rec OR diverged).
- ADR-0051 Cross-references covers all 4 categories (a/b/c/d) per CH-13 retro Row 1.
- Drift README index updated with CH-07 ✓ rows.
- D-CH07-FOLLOWUP-01 file present, status `discovered`, severity LOW, deferral target M6+.

**Audit prompt** (≤ 600 words):
```
You are an independent docs-fidelity + concept-alignment auditor for CH-07. Read the cycle plan at <plan-path> §2, §3.C, §4, §5, §10 first. Then verify:

1. Concept doc verified-header amendments (04, 06, 08): each new line accurately describes its lift-into-typed-Rust scope. No overpromise. Re-read the body diff to confirm body unchanged from chunk-open state. Run `git diff <chunk-open-sha> -- docs/specs/v0/concepts/permissions/{04,06,08}*.md` to verify body deltas (should be header-only).
2. New architecture doc `m5_2/architecture/multi-scope-cascade.md`: cross-refs ADR-0051 + concept 04/06/08 + drifts D-new-06 + D-new-20. Header verified-header present.
3. New operations doc `m5_2/operations/multi-scope-cascade-operations.md`: error-code dictionary entry for IntersectionEmpty, audit-event mapping, troubleshooting tree. Header verified-header present.
4. Concept-audit-matrix rows 178/202/208/226/227: Status values letter-for-letter from plan §2 target column (per CH-12 retro Row 1). Re-paste each row + plan §2 target side-by-side.
5. ADR-0051: Forks header line follows CH-13 retro Row 1 v3 ADR-body-checklist format (lock-state explicit, not just planner-recommendation). Cross-references contain ALL FOUR categories: (a) concept-docs, (b) drifts, (c) prior ADRs, (d) forward-scope row.
6. Drift README index: D-new-06 + D-new-20 rows show **remediated** + CH-07 ✓; D-CH07-FOLLOWUP-01 row added with LOW severity + M6+ target.
7. The MUST-RUN cargo + clippy + 4 CI guards lines: mark NOT-EXECUTED-IN-AUDIT (sandbox-blocked); orchestrator handles at final cycle re-audit gate.

Format: per-claim PASS/FAIL with file:line citation. Final verdict: GREEN / ARCHITECTURAL-FAIL / TACTICAL-FAIL / TRIVIAL-FAIL (split Trivial-1L vs Trivial-multi per CLAUDE.md). Output a markdown audit log. Word budget ≤ 1500 words for the report body. Write to <audit-B-iter-N-path>.
```

### Audit pass criteria

- Any new drift discovered → drift file created BEFORE chunk seals.
- Any concept contradiction → fixed in-chunk OR renegotiated with user OR converted to drift with future-chunk assignment.
- Chunk seal blocked until both audits return GREEN.
- Per CLAUDE.md Trivial FAIL split: Trivial-1L (≤ 1-line orchestrator inline patch on verified-header / changelog row / index entry) → orchestrator verifies in `cycle-audit.md`. Trivial-multi → re-spawn auditor at iter N+1.
- Iteration cap: ≥ 3 iterations on the same finding → STOP, escalate to user.

---

## §12 — Verification section (end-to-end recipe)

Concrete commands a reviewer can run to replay the chunk-close verification. **All commands use granular Bash discipline (per CLAUDE.md + `permissions/granular-bash-discipline-ab19399b.md`): one logical operation per Bash invocation; absolute paths; cargo capped at `-j 4`; cargo binary at `/root/rust-env/cargo/bin/cargo`.**

```bash
# 1. CI guards (each its own Bash invocation per granular discipline)
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (each its own Bash invocation)
/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all -- --check

RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets

/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace

# 3. Chunk-specific test counts
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace 2>&1 | grep -E "test result:" | awk '{p+=$4; f+=$6; i+=$8} END {print "passed=" p " failed=" f " ignored=" i}'
# Expect: passed in [1399, 1403]; failed=0; ignored=2

# 4. Cascade-specific test names — verify they ran + passed
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain cascade

/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain step_2a_membership_bound

/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --test multi_scope_cascade_acceptance

# 5. CH-11 + CH-13 regression checks
/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain step_6

/root/rust-env/cargo/bin/cargo test -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml -p domain audit_composition

# 6. phi-core leverage check (zero new imports in permissions/)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/

# Expect: 0 hits

# 7. Forbidden-duplication check
grep -rnE "^pub struct ResolvedGrant\b" /root/projects/phi/baby-phi/modules/crates/

# Expect: 1 hit at expansion.rs:56

# 8. Drift-file status
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: previous count + 2 (D-new-06 + D-new-20 transitioned)

# 9. New drift file present
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-CH07-FOLLOWUP-01.md
# Expect: file exists

# 10. ADR-0051 file present + Accepted
grep -nE "^\*\*Status:" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0051-multi-scope-cascade-contractor-model.md
# Expect: "**Status: Accepted**"

# 11. Architecture + Operations docs present with verified-header
head -1 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/architecture/multi-scope-cascade.md

head -1 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/operations/multi-scope-cascade-operations.md

# Expect: both prefix with `<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P3) -->`

# 12. Concept-audit-matrix row Status check (manual review by orchestrator at final re-audit)
grep -nE "5-tier scope cascade|Unified resolve_scope cascade|Contractor model|Shape A/B/C resolution per worked examples|Contractor scenario" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md
```

---

## Appendix — Cross-references

- **Forward-scope row**: `baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 84–89.
- **Per-chunk template**: `baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md`.
- **Granular Bash discipline source-of-truth**: `baby-phi/docs/specs/permissions/granular-bash-discipline-ab19399b.md`.
- **Multi-agent pipeline meta-plan**: `baby-phi/docs/specs/agentic-workflow/multi-agent-chunk-pipeline-0853574c.md`.
- **K8s readiness rationale**: `baby-phi/docs/specs/v0/implementation/m7b/architecture/k8s-microservices-readiness.md`.
- **Concept docs (canonical)**: `permissions/04-manifest-and-resolution.md`, `permissions/06-multi-scope-consent.md`, `permissions/08-worked-example.md`.
- **Prior CH plans of relevance**: CH-06 (`ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md`), CH-11 (per-session consent — CheckContext precedent), CH-13 (ADR-0050 — additive-enum precedent).
