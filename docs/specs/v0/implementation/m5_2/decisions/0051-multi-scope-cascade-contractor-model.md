<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P4 — Status flipped Proposed → Accepted; chunk closes drifts D-new-06 + D-new-20). -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-07 P0 scaffold; ADR Proposed). -->

# ADR-0051 — Multi-scope cascade + contractor-model membership bound

**Status: Accepted**

**Date:** 2026-05-07
**Chunk:** CH-07
**Closes:**
- [`D-new-06`](../../m5_1/drifts/D-new-06.md) (HIGH) — `step_5_scope_resolution` ships only single-tier resolution (top-tier-then-tie-break); concept docs 04 + 06 specify a 2-tier cascade with `base_X` tie-breakers and an intersection fallback for outsiders. CH-07 closes the cascade-shape half.
- [`D-new-20`](../../m5_1/drifts/D-new-20.md) (MEDIUM) — `step_2a_ceiling` is membership-blind; concept doc 06 §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166 specifies that an agent's home-org ceiling does not reach into sessions of scopes the agent is not a member of. CH-07 closes the contractor-model membership-bound half.

---

## Context

(Body fills at P1 — context paragraph will pin concept-doc 04 §"Mechanism 2: Scope Resolution" lines 354–375, concept-doc 06 §"The Unified Resolution Rule" lines 28–63 + §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166, and concept-doc 08 §"Step 4: Multi-Scope resolution for joint-research" lines 192–222 + §"Step 7: Contractor scenario" lines 287–298 as the canonical specs. Will note that today's `step_5_scope_resolution` at `engine.rs:317–337` is single-tier and `step_2a_ceiling` at `engine.rs:236` is membership-blind — both gaps land here.)

## Forks

- F1 → F1.A (2-tier + tie-breaker, concept-doc verbatim) — user-locked at plan approval 2026-05-07.
- F2 → F2.A (multi-scope read from `session.tags`; no Session shape change) — user-locked at plan approval 2026-05-07.
- F3 → F3.A (Step-2a-style re-clamp + new `DeniedReason::IntersectionEmpty`) — user-locked at plan approval 2026-05-07.
- F4 → F4.A (`DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count }`) — user-locked at plan approval 2026-05-07.

---

## Decision

### D51.1 — Cascade tier framing

The cascade in `domain::permissions::engine::step_5_scope_resolution` implements the **2-main-tier + tie-breaker + intersection-fallback** shape from `concepts/permissions/04-manifest-and-resolution.md` §"Mechanism 2: Scope Resolution" lines 354–375 and `concepts/permissions/06-multi-scope-consent.md` §"The Unified Resolution Rule" lines 28–53. The two main tiers are **Project** (most-specific) and **Organization** (less-specific); within each tier the `count == 1 / count > 1 / count == 0` branch decides between (a) pick that scope's candidates, (b) tie-break across `count > 1` matches via `base_project` / `base_org` (deferred to M6+ per §D51.2 — using lexicographic-minimum `ProjectId` / `OrgId` as the deterministic placeholder), or (c) fall through to the next tier. When **both** tiers fall through with `count == 0`, the cascade enters the intersection fallback per §D51.5.

The forward-scope row's "5-tier" wording (Project → Org → base_project → base_org → intersection fallback — `forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 84–89) is **informal labeling** for the 5 distinct **outcomes** the cascade can produce (single-project pick, base-project tie-break, single-org pick, base-org tie-break, intersection result), not 5 distinct tiers in the control flow. Concept-doc 04 line 310 ("Ties within a tier are broken by the base-org rule") is the canonical anchor: `base_X` is a tie-breaker **inside** a tier, not its own tier. F1.A user-locked at plan-approval gate 2026-05-07 pins this reading; F1.B (literal 5-ordered-tiers) was rejected because it would let an outsider with a `base_project` match in a non-session-tagged project leak through, which contradicts concept-06 lines 60–62. **Single-scope back-compat**: when `ctx.session_org_tags` and `ctx.session_project_tags` are both empty (Shape A/D session — today's dominant production traffic), the cascade reduces to today's single-tier behaviour (sort by `ScopeTier`, keep top-tier, tie-break by `(issued_at, id_bytes)`) so M1 callsite-count test invariants stay green.

### D51.2 — `step_5_scope_resolution` signature

The function signature lifts from `pub fn step_5_scope_resolution(matches: HashMap<ReachKey, Vec<Candidate>>, _ctx: &CheckContext<'_>) -> Result<HashMap<ReachKey, ResolvedGrant>, Decision>` (CH-07 chunk-open) to `pub fn step_5_scope_resolution(matches: HashMap<ReachKey, Vec<Candidate>>, ctx: &CheckContext<'_>) -> Result<HashMap<ReachKey, ResolvedGrant>, Decision>` — the `_ctx` underscore prefix is removed because the function now consumes `ctx.session_org_tags` + `ctx.session_project_tags` (the new fields added by §D51.4) plus `ctx.project_grants` + `ctx.org_grants` (existing fields) to compute the reader's per-scope membership intersections per concept-doc 06 lines 34–46.

**Reader-membership derivation**: the cascade reads "the projects the reader is a member of" by enumerating `ctx.project_grants[*].holder` for `PrincipalRef::Project(p)`, and similarly for orgs via `ctx.org_grants`. This is consistent with how CH-06 and CH-11 already derive scope membership (no new `Agent` struct field required at this milestone — see also §D51.3 + D-CH07-FOLLOWUP-01 for the M6+ deferral of the proper `base_project: Vec<ProjectId>` / `base_org: Vec<OrgId>` fields).

**Tie-break placeholder for `count > 1`**: when the reader matches multiple session-tagged projects (or orgs), the cascade picks the **lexicographically-smallest `ProjectId`** (or `OrgId`) byte ordering as a deterministic stand-in for `Agent.base_project_among(...)` / `Agent.base_org_among(...)`. Each call site carries a `// FIXME(D-CH07-FOLLOWUP-01)` marker. Concept-doc 06 line 58 names the proper tie-breaker as `base_project` / `base_org` — stable membership pins on `Agent` — but the field shape lands at M6+ to keep CH-07's blast radius bounded (no new migration; no new graph-edge). The lexicographic placeholder is forward-compatible: when M6+ ships the real fields, every test asserting "lo wins" updates to assert against the explicit `base_*` configuration without changing the cascade's outer shape. **Plan §3 Artifact A**: callsite cascade was 2 sites (1 call + 1 definition) — actual edits 1 callsite + 1 definition body + 14 test fixtures (within the §3 Artifact D 8–14 prediction band).

### D51.3 — `step_2a_ceiling` signature

The function signature lifts from `pub fn step_2a_ceiling(candidates: Vec<Candidate>, ceiling_grants: &[Grant]) -> Vec<Candidate>` (CH-07 chunk-open + P1 close) to `pub fn step_2a_ceiling(candidates: Vec<Candidate>, ceiling_grants: &[Grant], session_org_tags: &[OrgId]) -> Vec<Candidate>` — the third parameter `session_org_tags: &[OrgId]` is the same slice already threaded through `CheckContext` for §D51.4 + §D51.5, reused here at Step 2a to gate ceiling clamping by the reader's actual scope-membership.

**Why a fresh positional parameter rather than threading `&CheckContext`**: `step_2a_ceiling` is a pure clamping primitive with zero dependence on the rest of the context (no `agent_grants` / `consents` / `catalogue` reads, no `set_ref_registry` evaluation). Passing only the slice it needs preserves the existing signature shape and keeps the function unit-testable with a 3-arg fixture. The caller at `engine.rs::check_inner` (line 81) reads `ctx.session_org_tags` and passes the slice in — one positional callsite update.

**Plan §3 Artifact B**: predicted callsite cascade was 2 sites (1 call + 1 definition). Actual edits at P2: 1 callsite + 1 definition body + 4 new unit tests. Within the §3 Artifact B ≤ 5 pause threshold; no pause-discipline triggers fired.

**Membership-bound semantics pinned at §D51.6**: the body of `step_2a_ceiling` reads `session_org_tags` and filters `ceiling_grants` whose `holder == PrincipalRef::Organization(o)` to only those where `session_org_tags.contains(&o)`. Non-org ceiling holders (Project / Agent) pass through unfiltered — concept 06 line 162's bound is framed around `base_organization`, not the lower tiers. Empty `session_org_tags` preserves M1 behaviour exactly (every ceiling clamps; back-compat for Shape A/D).

### D51.4 — Multi-scope session encoding

`Session.tags: Vec<String>` (added in CH-06 migration 0008) is the **canonical multi-scope encoding** per `concepts/permissions/06-multi-scope-consent.md` lines 14–53 (`session_projects = session.tags.filter(Tag::Project)` and `session_orgs = session.tags.filter(Tag::Org)`). CH-07 does **not** extend the `Session` graph node with `Vec<OrgId>` / `Vec<ProjectId>` fields (F2.B was considered + rejected because (a) no Shape B/C session-creation path is shipped at this chunk; (b) the data is already encoded in `session.tags` per CH-06's instance + governance-tag emission at `events/listeners.rs:703-704`; (c) doubling the encoding invites drift; (d) CH-15 — real launch-time manifest validator — is the right home for any Session-shape change if one becomes necessary). **No migration.** F2.A user-locked at plan-approval gate 2026-05-07.

**Cascade reads from `session.tags` indirectly via `CheckContext` slices.** Two new fields land on `CheckContext<'a>`:
- `pub session_org_tags: &'a [OrgId]` — session's owning-org tags parsed from `org:<uuid>` prefixes.
- `pub session_project_tags: &'a [ProjectId]` — session's owning-project tags parsed from `project:<uuid>` prefixes.

The new helper `domain::permissions::parse_session_scope_tags(&[String]) -> (Vec<OrgId>, Vec<ProjectId>)` performs the prefix-scan + UUID parse, deduplicating + preserving first-occurrence order. Tags whose suffix is not a parseable UUID are silently skipped (matching `format!("org:{}", session.owning_org)` emission shape).

**Launch handler integration**: at `server::platform::sessions::launch.rs::gate_session_launch_consent`, the launch-time gate fires **before** the Session row is persisted. The handler synthesizes the would-be tag set from `LaunchInput` (`format!("org:{}", input.org_id)` + `format!("project:{}", input.project_id)`) and parses through `parse_session_scope_tags` before constructing `CheckContext`. For Shape A/D launches today this produces a 1-org + 1-project slice pair; the cascade still reduces to single-tier behaviour because reader-membership matches both candidate scopes (count == 1 at each tier). Multi-scope (Shape B/C) launch paths land at CH-15.

**Defensive default**: every CheckContext construction site that doesn't carry session-tag awareness yet (preview path, handler-support test fixtures, secret-reveal class-level reads) populates `session_org_tags: &[]` and `session_project_tags: &[]` — empty slices route through `cascade_single_scope` (the M1 single-tier behaviour). This preserves M1 callsite-count invariants outside the cascade-aware launch path.

### D51.5 — Intersection fallback semantic

When `reader_project_matches.count() == 0` AND `reader_org_matches.count() == 0` the reader is an "outsider" per concept-doc 06 lines 60–62 — they are not a member of any session-tagged scope. The cascade enters the **intersection fallback** at the new private helper `cascade_intersection_fallback` (engine.rs).

**Implementation**: a Step-2a-style ceiling re-clamp at Step 5. The helper:
1. Collects every `ctx.org_grants[*]` row whose `holder == PrincipalRef::Organization(o)` AND `o ∈ ctx.session_org_tags` — these are the session-org ceilings (concept-doc 06 line 60: "the intersection of all the session's scope ceilings"). Revoked grants are skipped (`g.revoked_at.is_some()` filter).
2. For each candidate in the reach's match set, applies `ceiling_admits` (the same predicate Step 2a uses — engine.rs `fn ceiling_admits`) against every session-org ceiling. Survivors are candidates whose fundamentals are a subset of at least one ceiling's fundamentals AND whose actions are admitted by the ceiling's action list (or `Action::Wildcard`).
3. Survivors → tie-break via `tie_break_within_tier` and return as a normal `ResolvedGrant`.
4. Empty survivor set OR empty session-ceiling set → `Decision::Denied { failed_step: FailedStep::Scope, reason: DeniedReason::IntersectionEmpty { fundamental, action, session_scope_count } }`.

**Why Step-2a-shape reuse**: F3.A reuses existing engine machinery (`ceiling_admits` + `tie_break_within_tier`) — zero new mental model required for reviewers. F3.B (synthetic `ResolvedGrant` with `Selector::Any` + phantom `descends_from`) was rejected because synthetic grants violate the "Grant always traces back to a real `descends_from` provenance" invariant (concept 02 §"System Bootstrap Template"); they would also break CH-13's `audit_class_source` resolution (the audit class would have no real source to attribute to). F3.C (always-Deny on intersection) was rejected because concept-doc 06 line 60 explicitly endorses the outsider-can-succeed path when their grants survive the ceiling clamp.

**FailedStep mapping**: `FailedStep::Scope` (existing variant) — concept-doc 06 line 62 uses the doc's abstract "failed_step: 3 (no grant covers this reach)" framing; our pipeline's `FailedStep` enum encodes which step short-circuited, and the intersection fallback fires inside `step_5_scope_resolution`, so `FailedStep::Scope` is the correct attribution. F3.A user-locked at plan-approval gate 2026-05-07.

### D51.6 — Contractor-model membership bound

Concept-doc 06 §"Subject-Side Reach Is Bounded by Scope Membership" line 162 verbatim: *"an agent's home org (`base_organization`) does not reach into sessions belonging to scopes the agent is not a member of."* This is the structural enforcement of the contractor-model security boundary. Drift D-new-20 documented the gap: today's `step_2a_ceiling` filters every candidate against the entire `ceiling_grants` slice uniformly, with no awareness of whether the reader is actually a member of the ceiling's owning scope. A contractor (e.g., concept-doc 08 §"Step 7" `contractor-x-9`, base_org=Gamma) operating inside a session tagged `[org:acme]` would have their Gamma ceiling clamp the Acme-issued candidate grants — a privacy violation: Gamma's ceiling is reaching into an Acme session.

**Implementation at `step_2a_ceiling`**: the new third parameter `session_org_tags: &[OrgId]` (per §D51.3) gates which `ceiling_grants` participate in clamping:

```rust
let applicable_ceilings: Vec<&Grant> = if session_org_tags.is_empty() {
    ceiling_grants.iter().collect()        // back-compat: all ceilings clamp
} else {
    ceiling_grants
        .iter()
        .filter(|g| match g.holder {
            PrincipalRef::Organization(o) => session_org_tags.contains(&o),
            _ => true,                      // non-org ceilings unaffected
        })
        .collect()
};
```

**Empty `session_org_tags`** = single-org / Shape A / Shape D path — every ceiling clamps uniformly, preserving M1 behaviour exactly. This back-compat path is what every CheckContext construction site outside the cascade-aware launch handler defaults to (per §D51.4 defensive-default — preview, handler_support test fixtures, secret-reveal class-level reads all populate `session_org_tags: &[]`).

**Non-empty `session_org_tags`** = multi-scope (Shape B/C) path — a ceiling whose `holder == PrincipalRef::Organization(o)` only clamps when `o ∈ session_org_tags`. Non-org ceilings (Project / Agent holders) are unaffected because concept 06 line 162 frames the bound around `base_organization`, not the project / agent tiers.

**Empty `applicable_ceilings`** = every ceiling was filtered out by the membership bound. The function returns the candidate set unchanged ("infinite ceiling" path), matching the original empty-input semantics. This is the contractor scenario: a contractor whose base_org ceiling is filtered out has the candidate set face no applicable ceiling at Step 2a; clamping happens later at `cascade_intersection_fallback` in Step 5 if needed (per §D51.5).

**Closes drift D-new-20** (MEDIUM). The contractor-model acceptance test at `multi_scope_cascade_acceptance.rs::contractor_x_9_in_acme_session_per_concept_08_step_7` (P3) reproduces concept-08 §"Step 7" lines 287–298 verbatim: `contractor-x-9` (base_org=Gamma) reading sessions in `acme-website-redesign` (session-tagged `[org:acme]`) gets a project-tier resolution that succeeds, with the Gamma base_org ceiling correctly excluded by the membership bound.

### D51.7 — DeniedReason additive variant

A new variant lands on `DeniedReason`:

```rust
DeniedReason::IntersectionEmpty {
    fundamental: Fundamental,
    action: Action,
    session_scope_count: u8,
},
```

The first two fields mirror `DeniedReason::NoMatchingGrant`'s shape — they pin the failed reach so audit logs can diagnose the denial without re-running the engine. The third field reports the number of distinct session-tagged scopes the cascade considered (`session_org_tags.len() + session_project_tags.len()`, saturating at `u8::MAX`). It's useful in audit + observability dashboards (e.g. "denial-rate by session-scope cardinality") **without leaking specific OrgIds** — a 1-byte counter is below any per-tenant secrecy boundary.

**Naming rationale**: F4.B (`OutsiderDenied`) was rejected because "outsider" is a doc-scoped framing for the reader's role at concept-doc 06 line 60, while the engine's audit + observability story benefits from a name that reflects the engine state ("intersection-fallback was empty"). F4.C (reuse `NoMatchingGrant`) was rejected because the additive-cost is small (3 lines per match arm) and a separate variant keeps the mental model clean for SREs scanning denial-distribution histograms.

**Match-site cascade (per CH-12 retro Row 2 / chunk-planner v3 additive-enum discipline)**: the new variant is **additive**. Two explicit-per-variant matches across the workspace updated with new arms (no `_ =>` catch-all per F4.A):
- `server::handler_support::permission::denial_to_api_error` — message-format arm.
- `server::platform::secrets::reveal::denial_reason_text` — message-format arm.

The match site at `server::platform::sessions::launch.rs::gate_session_launch_consent` matches inside `failed_step == FailedStep::Consent` only (consent-tagged variants); `IntersectionEmpty` fires under `FailedStep::Scope`, so it's filtered upstream and routes through that handler's existing `Decision::Denied { .. } | Decision::Allowed { .. } => Ok(None)` advisory branch — no edit required at that site.

**HTTP error mapping**: `server::handler_support::permission::denial_to_api_error` maps `FailedStep::Scope` to `ApiError(StatusCode::FORBIDDEN, "SCOPE_UNRESOLVABLE", message)`. The `IntersectionEmpty` denial inherits the `SCOPE_UNRESOLVABLE` code via the existing `match failed_step` table; the message string carries the new `IntersectionEmpty`-specific text. The web tier's `lib/api/errors.ts` hint table stays stable per D10. F4.A user-locked at plan-approval gate 2026-05-07.

---

## Cross-references

- **(a) Originating concept-doc + sections**:
  - [`permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Mechanism 2: Scope Resolution" lines 354–375; §"Key Invariants" line 310 ("Ties within a tier are broken by the base-org rule"); §"Refinement" lines 393–425 (`ResolvedGrant` shape — unchanged).
  - [`permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"The Hard Schema Constraint" lines 13–26; §"The Unified Resolution Rule" lines 28–63 (incl. `resolve_scope` pseudocode); §"Subject-Side Reach Is Bounded by Scope Membership" lines 161–166.
  - [`permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §"Step 4: Multi-Scope resolution for joint-research" lines 192–222 (Scenarios 4/5/6 — `lead-acme-1`, `lead-beta-1`, `lead-gamma-1`); §"Step 7: Contractor scenario" lines 287–298 (`contractor-x-9`); §"Summary: Who Can Read What" line 383 (`lead-gamma-1` joint-research column = ✗).
- **(b) Closed drifts**:
  - [`D-new-06`](../../m5_1/drifts/D-new-06.md) (HIGH) — multi-scope cascade gap.
  - [`D-new-20`](../../m5_1/drifts/D-new-20.md) (MEDIUM) — contractor-model membership-bound gap.
- **(c) Prior ADRs cited as precedent**:
  - [ADR-0036](./0036-selector-grammar-pest-peg.md) — selector grammar provides the tag-predicate machinery the cascade reads `session.tags` through.
  - [ADR-0048](./0048-per-session-consent-gating.md) — per-session consent gating (precedent for adding `&[OrgId]`-shaped slices to CheckContext).
  - [ADR-0050](./0050-audit-class-composition-strictest-wins.md) — audit-class composition (precedent for additive enum variants without migration).
  - [ADR-0033](./0033-k8s-prep-refactors.md) — CH-K8S-PREP conforming criteria (referenced for K8s-neutral verification at plan §3.B).
- **(d) Forward-scope row cross-reference**:
  - [`baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 84–89 (CH-07 row).

---

## Phase placement

- **P0** — Plan archive + ADR-0051 scaffold (this file, Proposed) + cycle-index row.
- **P1** — `step_5_scope_resolution` 2-tier cascade body + `DeniedReason::IntersectionEmpty` variant + CheckContext extension (`session_org_tags` / `session_project_tags` slices) + ADR §D51.1, §D51.2, §D51.4, §D51.5, §D51.7 bodies filled.
- **P2** — `step_2a_ceiling` membership-bound clamp + launch handler tag-parse wiring + ADR §D51.3, §D51.6 bodies filled.
- **P3** — Acceptance tests for concept-08 scenarios 4/5/6 + contractor scenario + new architecture + operations docs + concept-audit-matrix row updates + verified-header refreshes.
- **P4** — ADR Proposed → Accepted flip + drift remediation (D-new-06, D-new-20) + final CI guards.
