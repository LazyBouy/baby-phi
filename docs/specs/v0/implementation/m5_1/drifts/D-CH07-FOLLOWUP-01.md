<!-- Last verified: 2026-05-07 by Claude Code (filed by CH-07 chunk-seal; cycle hex `cc912d07`) -->

# D-CH07-FOLLOWUP-01 — Agent.base_project + Agent.base_org Vec field additions deferred (cascade tie-breaker placeholder)

## Identification
- **ID**: D-CH07-FOLLOWUP-01
- **Phase of origin**: CH-07 chunk-seal (cycle hex `cc912d07`)
- **Discovery source**: `cycle-plan-deferral` (planner §7 P1 deliverable 5 + ADR-0051 §D51.2 + §D51.3 explicitly defer the proper `base_project` / `base_org` Agent fields to M6+; CH-07 ships a deterministic lexicographic-min placeholder for the `count > 1` tie-break case)
- **Date discovered**: 2026-05-07
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap with cascade-shape landed; tie-breaker semantics deferred
- **Severity**: LOW
- **Tags**: `permission-engine`, `cascade`, `tie-breaker`, `agent-shape`, `m6-deferral`
- **Blocks**: nothing at v0 — the lexicographic-min placeholder produces a deterministic Decision in the `count > 1` case; the gap only matters when an operator explicitly designates an Agent's home project / home org for tie-break purposes (no such designation surface exists today)
- **Blocked-by**: future M6+ chunk (TBD; not yet allocated in forward-scope) when long-lived multi-org / multi-project agents land and a stable home-scope pin becomes load-bearing

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"The Unified Resolution Rule" line 58 (the `resolve_scope` pseudocode names `base_project` / `base_org` as the proper tie-breaker when `count > 1`); [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Key Invariants" line 310 ("Ties within a tier are broken by the base-org rule").
- **Concept claim**: When the cascade's project tier (or org tier) matches more than one of the session's tagged scopes for a given reader, the tie is broken by the reader's `Agent.base_project` (or `Agent.base_org`) — a stable membership pin capturing the agent's home scope. The home scope wins ties.
- **Contradiction**: Today's `cascade_multi_scope` at [`modules/crates/domain/src/permissions/engine.rs:435+`](../../../../../../modules/crates/domain/src/permissions/engine.rs) implements the `count > 1` branch by sorting matched `ProjectId` / `OrgId` byte values and picking the lexicographically-smallest. This is a deterministic ordering — the same Decision is produced across runs — but it does not honor the concept-doc semantics for "the home scope wins ties." The `Agent` struct does not carry `base_project: Vec<ProjectId>` or `base_org: Vec<OrgId>` fields; CH-07 explicitly defers adding them to M6+ to keep the chunk's blast radius bounded (no new migration; no new graph edge).
- **Classification**: `partially-honored` (cascade outer shape honored by CH-07; tie-breaker semantics use a deterministic placeholder)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (CH-07 plan §7 P1 deliverable 5 + ADR-0051 §D51.2 / §D51.3): "base_project / base_org M6+ deferral: ADR-0051 §D51.2 + §D51.3 explicitly defer the Agent.base_project / Agent.base_org Vec field additions to a future M6+ chunk. Today, when `count > 1` at either tier, the implementation picks the lexicographically-smallest tagged scope as a deterministic fallback and emits a doc-comment `// FIXME(D-CH07-FOLLOWUP-01): tie-breaker deferred to M6 — using lexicographic ordering as deterministic placeholder per ADR-0051 §D51.2`."
- **Reality**: matches the plan exactly. CH-07 ships the cascade outer shape (project-tier match-count branch → org-tier match-count branch → intersection fallback) plus the membership-bounded ceiling clamp; the `count > 1` branches in both `cascade_multi_scope` halves use lexicographic-min ordering as a deterministic placeholder. Two `// FIXME(D-CH07-FOLLOWUP-01)` markers carry the deferral note inline at the project-tier branch (engine.rs:510) and the org-tier branch (engine.rs:582).

## Required follow-up
- **What needs to happen**: when a future M6+ chunk lands "Long-lived multi-org / multi-project Agents" (or equivalent — TBD title at the time of allocation), the `Agent` struct gains `base_project: Vec<ProjectId>` and `base_org: Vec<OrgId>` fields capturing the agent's home scopes. The `cascade_multi_scope` `count > 1` branches (engine.rs:510 + engine.rs:582) replace the lexicographic-min selection with a `base_project_among(reader_project_matches)` / `base_org_among(reader_org_matches)` lookup that returns the agent's designated home scope when one of the matches qualifies. Concept-doc 06 line 58's `resolve_scope` pseudocode is the canonical reference. Migration may be required to backfill `base_project` / `base_org` on existing Agent rows (likely default to the org/project of `Agent.owning_org` / first creating project for back-compat).
- **Tests required**: acceptance scenarios verifying that when a reader matches multiple session-tagged scopes, the cascade picks the reader's `base_project` / `base_org` (not the lexicographically-smallest scope). Existing CH-07 tests (`cascade_project_tier_multiple_matches_picks_lexicographic_min`, `cascade_org_tier_multiple_matches_picks_lexicographic_min`) get rewritten to assert against an explicit `base_*` configuration without changing the cascade's outer shape.
- **Acceptance**: every multi-match cascade outcome honors the agent's `base_project` / `base_org` pin per concept-doc 06 line 58; the lexicographic-min placeholder is removed.

## Closing chunk
- TBD — likely M6+ "Long-lived multi-org / multi-project Agents" follow-up; not yet allocated in forward-scope.

## Lifecycle
- **2026-05-07 — `discovered`** — filed by CH-07 chunk-seal. CH-07 ships the cascade outer shape + contractor-model membership bound for the 2-tier branching; Agent home-scope tie-breaker fields deferred to M6+. Mirrors CH-11's `D-CH11-FOLLOWUP-01` + CH-12's `D-CH12-FOLLOWUP-01` + CH-13's `D-CH13-FOLLOWUP-01` patterns (chunk closes one axis of a multi-axis concept-doc claim; the other axis tracked here).

## Cross-references
- CH-07 plan: [`baby-phi/docs/specs/plan/build/ch-07-multi-scope-cascade-contractor-model-cc912d07/plan.md`](../../../../plan/build/ch-07-multi-scope-cascade-contractor-model-cc912d07/plan.md) §7 P1 deliverable 5 + §10 close criteria.
- ADR-0051: [`m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`](../../m5_2/decisions/0051-multi-scope-cascade-contractor-model.md) §D51.2 (cascade tie-break placeholder) + §D51.3 (signature defers Agent shape change).
- D-new-06: [`D-new-06.md`](D-new-06.md) (closed by CH-07 — cascade outer shape).
- D-new-20: [`D-new-20.md`](D-new-20.md) (closed by CH-07 — contractor-model membership bound).
- Sister patterns: [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md) (CH-11's analogous follow-up for `Project.deadline_at`); [`D-CH12-FOLLOWUP-01.md`](D-CH12-FOLLOWUP-01.md) (CH-12's analogous follow-up for session-tag emission); [`D-CH13-FOLLOWUP-01.md`](D-CH13-FOLLOWUP-01.md) (CH-13's analogous follow-up for platform-admin Grant audit_class composition).
- Concept docs: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) line 58 (`resolve_scope` tie-breaker); [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) line 310 (Key Invariants — base-org rule).
- Code FIXME markers: `engine.rs:510` (project-tier `count > 1` branch in `cascade_multi_scope`); `engine.rs:582` (org-tier `count > 1` branch in `cascade_multi_scope`).
