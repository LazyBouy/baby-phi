<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-08 — Frozen session-tag immutability not enforced (no grant prevents [modify] on structural tags)

## Identification
- **ID**: D-new-08
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `security-boundary`, `tag-immutability`, `exfiltration-prevention`
- **Blocks**: none directly; required for security guarantee on multi-scope session reads
- **Blocked-by**: D-new-03 (needs tag-predicate selectors to express "structural tag"); D-new-07 (validator should reject [modify] on reserved tags at publish time)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Frozen-at-creation tags (immutability)"
- **Concept claim**: No grant template issues `[modify]` on structural session tags; frozen-at-creation rule prevents tag mutation. Only lifecycle tags (`#archived`, `#active`) are mutable via system events.
- **Contradiction**: No code path rejects `[modify]` on session structural tags. Grant builders can include `[modify]`. Session struct has no tag-list field yet (session tags belong to phi-core Session inner wrap); retag attempts would succeed if a grant existed.
- **Classification**: `silent-in-code` → security invariant not enforced
- **phi-core leverage status**: `wrap` (session tags live on phi-core's Session)

## Plan vs. reality
- **Plan said**: Tag immutability is a security property enforced at the permission layer (no [modify] grant covers structural tags).
- **Reality (shipped state at current HEAD)**: No enforcement; concept claim is theoretical until grammar + validator + gate all land.
- **Root cause**: `cascading-upstream-deferral` (needs D-new-03 selectors + D-new-07 validator).

## Where visible in code
- **File(s)**: No enforcement site yet; would live at permission-check Step 4 or a new validator hook.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "structural_tag\|frozen_tag\|immutable_tag" modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Once D-new-03 + D-new-07 land, reserved-namespace validator (D-new-07) rejects `[modify]` on reserved/structural tag namespaces (`agent:*`, `project:*`, `org:*`, etc.) at publish time. Runtime gate at Step 4 rejects retag attempts on session structural tags.
- **Implementation chunk this belongs to**: CH-12
- **Dependencies on other drifts**: D-new-03, D-new-07
- **Estimated effort**: 1 engineer-day (after prereqs; mostly test coverage + edge-case handling).
- **Risk to concept alignment if deferred further**: HIGH — exfiltration vector exists in theory; retagging session to remove `org:X` circumvents multi-scope boundary.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 flagged twice; merged D-new-session-tag-mutability + D-new-frozen-tag-enforcement)
