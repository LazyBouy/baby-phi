<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P3 — multi-scope cascade design + contractor-bound + intersection-fallback typed-Rust translation per ADR-0051) -->

# Multi-scope cascade + contractor-model membership bound — design page

> **Status:** [EXISTS] as of CH-07 (M5.2). The 2-tier-plus-tie-breaker-plus-intersection cascade ships at [`modules/crates/domain/src/permissions/engine.rs::step_5_scope_resolution`](../../../../../../modules/crates/domain/src/permissions/engine.rs); the membership-bounded ceiling clamp ships at the same file's [`step_2a_ceiling`](../../../../../../modules/crates/domain/src/permissions/engine.rs); the [`DeniedReason::IntersectionEmpty`](../../../../../../modules/crates/domain/src/permissions/decision.rs) variant routes the empty-fallback case. For the normative concept-doc reference, read [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Mechanism 2: Scope Resolution" lines 354–375 + §"Key Invariants" line 310 and [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"The Unified Resolution Rule" lines 28–63 + §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166.

---

## Overview

The Permission Check engine resolves a winning grant per `(fundamental, action)` reach via a cascade. Pre-CH-07 the cascade was **single-tier**: sort candidates by `ScopeTier::{Agent, Project, Organization}`, keep the top tier, tie-break by `(issued_at, id_bytes)`. That implementation was correct for Shape A / Shape D sessions (single project + single org / 0 project + single org) but **silent on multi-scope (Shape B / Shape C)** sessions and silent on the **contractor-model security boundary** (a non-member org's ceiling reaching cross-scope).

CH-07 closes both gaps, lifting concept-doc 04 + 06 + 08 invariants into typed Rust. The chunk closes two drifts:

- [`D-new-06`](../../m5_1/drifts/D-new-06.md) (HIGH) — multi-scope cascade body.
- [`D-new-20`](../../m5_1/drifts/D-new-20.md) (MEDIUM) — contractor-model membership bound at `step_2a_ceiling`.

Both are flipped to `remediated` at CH-07 P4. ADR-0051 [`m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`](../decisions/0051-multi-scope-cascade-contractor-model.md) records the design decisions (sub-decisions §D51.1–§D51.7); this page is the design-facing description.

---

## 2-tier cascade structure

The cascade canonicalised in concept-doc 04 lines 354–375 + concept-doc 06 lines 28–63 has **two main tiers** (Project, Organization), each with an internal `count == 0 / count == 1 / count > 1` branch, plus an **intersection fallback** for the outsider case (reader is in 0 of the session's tagged scopes at both tiers).

The forward-scope row's "5-tier" labeling (`Project → Org → base_project → base_org → intersection`) is informal labeling for "5 distinct outcomes," not 5 distinct tiers in the control flow. ADR-0051 §D51.1 pins concept-06's `resolve_scope` pseudocode as the normative shape; `base_project` / `base_org` are **tie-breakers WITHIN a tier** per concept-04 line 310 ("Ties within a tier are broken by the base-org rule"), not separate tiers.

```text
                       step_5_scope_resolution
                              │
                              ▼
              ┌───────────────────────────────┐
              │  multi_scope = !ctx           │
              │      .session_org_tags        │
              │      .is_empty() ||           │
              │   !ctx.session_project_tags   │
              │      .is_empty()              │
              └───────┬───────────────────┬───┘
                      │ false             │ true
                      ▼                   ▼
        ┌────────────────────┐    ┌──────────────────────┐
        │ cascade_single_    │    │ cascade_multi_scope  │
        │ scope (M1 / Shape  │    │ (CH-07 / Shape B/C)  │
        │ A/D back-compat)   │    │                      │
        │                    │    │  ┌────────────────┐  │
        │ sort by tier;      │    │  │ Project tier   │  │
        │ keep top-tier;     │    │  │ count==1 ───▶  │  │
        │ tie-break          │    │  │   pick that    │  │
        │ (issued_at,        │    │  │   project's    │  │
        │  id_bytes)         │    │  │   candidate(s) │  │
        └────────────────────┘    │  │ count>1  ───▶  │  │
                                  │  │   lex-min      │  │
                                  │  │   placeholder  │  │
                                  │  │ count==0 ───▶  │  │
                                  │  │   fall-through │  │
                                  │  └────────┬───────┘  │
                                  │           │          │
                                  │  ┌────────▼───────┐  │
                                  │  │ Org tier       │  │
                                  │  │ (same 3-branch │  │
                                  │  │  shape)        │  │
                                  │  └────────┬───────┘  │
                                  │           │ both     │
                                  │           │ count==0 │
                                  │           ▼          │
                                  │  ┌────────────────┐  │
                                  │  │ Intersection   │  │
                                  │  │ fallback —     │  │
                                  │  │ Step-2a-style  │  │
                                  │  │ ceiling        │  │
                                  │  │ re-clamp using │  │
                                  │  │ session-tagged │  │
                                  │  │ orgs as        │  │
                                  │  │ ceilings       │  │
                                  │  │   ∅ ───▶ Denied│  │
                                  │  │ IntersectionEmpty│ │
                                  │  └────────────────┘  │
                                  └──────────────────────┘
```

The single-scope branch (`cascade_single_scope`) preserves the M1 / pre-CH-07 cascade exactly so empty `session_org_tags` and `session_project_tags` slices give back the legacy behaviour byte-for-byte; this is the defensive default that keeps M1 callsite-count test invariants green for every non-cascade-aware construction site.

---

## Membership derivation

The cascade computes the reader's per-scope membership **without a dedicated `Agent.member_of_projects` / `member_of_orgs` field**. Concept-doc 06 lines 38–43 state the membership semantically:

> *"`reader_project_matches = session_project_tags ∩ {projects the reader is a member of}`"*

In typed Rust, the reader's project / org membership is **derived from their grant set** — a grant in `ctx.project_grants` whose `holder == PrincipalRef::Project(p)` is the structural witness that the reader is a member of project `p` (the grant only attaches to the reader because they hold it through project membership). The cascade walks `ctx.project_grants` + `ctx.org_grants` to collect the projects/orgs the reader actually has presence in, then intersects with `ctx.session_project_tags` / `ctx.session_org_tags`.

This is a **deviation from the concept-doc's abstract framing**: the concept doc speaks of "the reader is a member of P" as a free-standing predicate, while the implementation derives membership from grant presence. The deviation was noted at P1; ADR-0051 §D51.2 records that no `Agent.member_of_*` field was added, and the chunk-implementer's report flagged this. The justification is concept-doc 02 (System Bootstrap Template) — every grant has a `descends_from` chain that traces back to a real bootstrap event; presence of a project-tier grant is therefore equivalent to project membership at the storage layer.

A future M6+ chunk may add explicit `Agent.member_of_*` fields if the workspace gains a non-grant-mediated membership concept (e.g. inherited group membership without a grant); CH-07 ships against the grant-derived shape per ADR-0051 §D51.2.

---

## `base_X` tie-breaker placeholder (deferred to M6+)

Concept-doc 04 line 310 says *"Ties within a tier are broken by the base-org rule."* Concept-doc 06 lines 38–53's `resolve_scope` pseudocode names `Agent.base_project` (singular, on the agent struct) for the project-tier tie-breaker and `Agent.base_org` for the org-tier tie-breaker.

Today's `domain::model::Agent` carries no `base_project: Vec<ProjectId>` or `base_org: Vec<OrgId>` field; adding them would be a Repository-shape migration that the planner explicitly deferred (forward-scope-row + plan §1 Quality-over-speed). CH-07 ships a **deterministic lexicographic-min placeholder**: when `count > 1` at either tier, the cascade picks the lexicographically-smallest matched ProjectId (resp. OrgId) as the deterministic winner.

Both placeholder branches carry an inline FIXME tag per ADR-0051 §D51.2 / §D51.3:

```rust
// FIXME(D-CH07-FOLLOWUP-01): tie-breaker deferred to M6 — using
// lexicographic ordering as deterministic placeholder per ADR-0051
// §D51.2/§D51.3.
```

The follow-up drift `D-CH07-FOLLOWUP-01` (created at P4, severity `LOW`, deferral target `M6+`) tracks the proper `Agent.base_project` / `Agent.base_org` field additions. The deterministic placeholder ensures cascade-output stability across runs even today: lexicographic-min is a total order on UUIDs, so test fixtures + production runs converge on the same winner.

---

## Intersection fallback (F3.A — Step-2a-style re-clamp)

When both tiers fall through (`count == 0` at project AND `count == 0` at org), concept-doc 06 lines 60–62 frames the case as *"the outsider faces the intersection of all the session's scope ceilings"*:

> *"Empty-intersection case: When the intersection contains no action the outsider's grants can exercise … the effect is **deny by default**. The outsider receives a `Denied` Permission Check result with `failed_step: 3 (no grant covers this reach)`."*

The implementation reuses the Step-2a ceiling-clamp machinery (`ceiling_admits` per `engine.rs:299`):

1. Collect the union of `org_grants` whose `holder == PrincipalRef::Organization(o)` for `o ∈ ctx.session_org_tags`. These are the session's scope ceilings.
2. If the collected ceiling set is empty (no grant in `org_grants` matches a session-tagged org), the fallback is structurally undefined — the engine returns `Decision::Denied { failed_step: FailedStep::Scope, reason: DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count } }` directly.
3. Otherwise, run the Step-2a ceiling-clamp shape: keep candidates whose `ResolvedGrant` is admitted by at least one of the collected session-org ceilings.
4. If any candidate survives → tie-break via `tie_break_within_tier` and return as `ResolvedGrant`.
5. If every candidate is clamped out → `IntersectionEmpty` denial.

**Translation note (concept-doc → typed Rust):** concept-doc 06 line 62 says `failed_step: 3`. Our `FailedStep` enum encodes which engine step short-circuited; `FailedStep::Scope` is the variant that fires from Step 5 territory. The doc's "3" refers to the abstract step ordering in concept-doc text, not our enum's positional index — this is a fidelity-preserving translation. ADR-0051 §D51.5 + §D51.7 record the mapping.

The new `DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count }` variant is **additive** (per CH-12 retro Row 2 / chunk-planner v3 additive-enum discipline). `decision.rs`'s explicit-arm `match` sites (Display impl + metric-label) gain one arm each; downstream HTTP error mapping in the `server` crate continues via `From` conversions and is unchanged. The `session_scope_count: u8` field is the count of session-tagged scopes the cascade considered (orgs + projects), useful in audit logs without leaking specific OrgIds / ProjectIds.

---

## Contractor-bound (concept-doc 06 line 162 verbatim)

Concept-doc 06 line 162 verbatim:

> *"an agent's home org (`base_organization`) does not reach into sessions belonging to scopes the agent is not a member of."*

This is the **subject-side reach bound** — the security-critical invariant that prevents a contractor's home-org ceiling from clamping their candidates when the contractor is operating inside another org's scope. Without this bound, a contractor with a strict home-org ceiling (e.g. Gamma's `Connect`-only ceiling) operating inside an Acme project would have all their `Read` candidates filtered to empty by the Gamma ceiling, breaking concept-doc 08 §"Step 7" Scenario 7 ("the contractor operates entirely under Acme's rules for the duration of the contract").

CH-07 lifts the invariant into `step_2a_ceiling` (engine.rs:256–297) via a **membership filter** on the ceiling-grants slice:

```text
                       step_2a_ceiling
                              │
                              ▼
                ┌───────────────────────────┐
                │  session_org_tags empty?  │
                └────────┬─────────┬────────┘
                  yes    │         │  no
                         ▼         ▼
                ┌─────────────┐  ┌────────────────────────┐
                │ M1 / Shape  │  │ Filter ceiling_grants  │
                │ A/D back-   │  │ to those whose         │
                │ compat:     │  │ `holder ==             │
                │ all         │  │  PrincipalRef::        │
                │ ceilings    │  │  Organization(o)`      │
                │ apply       │  │ AND                    │
                │ uniformly.  │  │ `session_org_tags      │
                │             │  │  .contains(&o)`.       │
                │             │  │ (Project / Agent       │
                │             │  │  holders pass through  │
                │             │  │  unchanged.)           │
                └──────┬──────┘  └────────┬───────────────┘
                       │                  │
                       └────┬─────────────┘
                            ▼
              ┌────────────────────────────┐
              │ applicable_ceilings empty? │
              └────────┬───────────────────┘
                       │
            yes ───────┴────── no
            ▼                  ▼
    ┌────────────────┐  ┌────────────────────────┐
    │ "infinite      │  │ Standard ceiling_admits│
    │ ceiling"       │  │ filter — keep candidate│
    │ — pass         │  │ iff at least one       │
    │ candidates     │  │ applicable ceiling     │
    │ through        │  │ admits it.             │
    │ unchanged.     │  │                        │
    │                │  │ This is the M1 /       │
    │                │  │ pre-CH-07 body, now    │
    │                │  │ membership-bounded.    │
    └────────────────┘  └────────────────────────┘
```

The membership bound applies **only to Organization-tier ceilings**; Project-tier and Agent-tier ceilings pass through unchanged because concept-doc 06 line 162 frames the bound around `base_organization`, not project / agent tiers. ADR-0051 §D51.6 records the scope of the bound.

**Empty-applicable-ceilings sub-case (back-compat hardening):** when every ceiling is filtered out by the membership bound, `step_2a_ceiling` returns the candidate set unchanged (the "infinite ceiling" path). This matches concept-doc 06 lines 161–166's framing — a contractor with no applicable session-org ceiling has no constraint at Step 2a; clamping comes from Step 5 instead (the cascade's own resolution).

---

## Cross-references

- **ADR**: [`m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`](../decisions/0051-multi-scope-cascade-contractor-model.md) — sub-decisions §D51.1 (cascade tier framing), §D51.2 (`step_5_scope_resolution` signature), §D51.3 (`step_2a_ceiling` signature), §D51.4 (multi-scope session encoding via `Session.tags`), §D51.5 (intersection-fallback semantic), §D51.6 (contractor-model membership bound), §D51.7 (`DeniedReason::IntersectionEmpty` additive variant).
- **Concept docs**:
  - [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Mechanism 2: Scope Resolution" lines 354–375 + §"Key Invariants" line 310 (cascade structure + `base_X` tie-breaker).
  - [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"The Unified Resolution Rule" lines 28–63 (`resolve_scope` pseudocode) + §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166 (contractor-bound).
  - [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §"Step 4: Multi-Scope resolution for joint-research" lines 192–222 (Scenarios 4/5/6) + §"Step 7: Contractor scenario" lines 287–298 (Scenario 7).
- **Drifts**:
  - [`m5_1/drifts/D-new-06.md`](../../m5_1/drifts/D-new-06.md) (HIGH) — multi-scope cascade body. Closed at CH-07.
  - [`m5_1/drifts/D-new-20.md`](../../m5_1/drifts/D-new-20.md) (MEDIUM) — contractor-bound. Closed at CH-07.
  - `D-CH07-FOLLOWUP-01` (LOW) — `Agent.base_project` / `Agent.base_org` field additions deferred to M6+; drift file created at CH-07 P4 chunk-seal under `m5_1/drifts/D-CH07-FOLLOWUP-01.md`.
- **Operations playbook**: [`multi-scope-cascade-operations.md`](../operations/multi-scope-cascade-operations.md) — error-code dictionary for `IntersectionEmpty`, audit-event mapping, troubleshooting tree.
- **Acceptance tests**: [`modules/crates/domain/tests/multi_scope_cascade_acceptance.rs`](../../../../../../modules/crates/domain/tests/multi_scope_cascade_acceptance.rs) — 6 tests covering Scenarios 4 / 5 / 6 / 7 + Shape A baseline + Shape D system-session.
