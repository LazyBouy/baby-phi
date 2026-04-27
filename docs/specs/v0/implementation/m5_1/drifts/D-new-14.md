<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-14 — `system:genesis` axiomatic principal + authority-chain traversal missing (provenance stored but not walkable to bootstrap)

## Identification
- **ID**: D-new-14
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `authority-chain`, `bootstrap-axiom`, `provenance-traversal`
- **Blocks**: Revocation cascade (D-new-18) needs provenance walker; audit trail completeness
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"System Bootstrap Template"; [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Authority Chain"; [`concepts/permissions/README.md`](../../../concepts/permissions/README.md) §"Provenance"
- **Concept claim**: Every Grant points to an AuthRequest; every AuthRequest was approved by a named approver; the chain traces back to a hardcoded `SystemBootstrap` template approved by `system:genesis` axiomatic principal. Provenance is fully walkable.
- **Contradiction**: `TemplateKind::SystemBootstrap` variant exists at [`nodes.rs:507`](../../../../../../modules/crates/domain/src/model/nodes.rs#L507); no `system:genesis` principal is defined anywhere in code. Grant.`auth_request_id` stored but no walker traces chains. No test asserts every grant chains to bootstrap.
- **Classification**: `partially-honored` — bootstrap kind exists, but traversal + `system:genesis` axiom missing
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Full chain traversable; `system:genesis` is the root.
- **Reality (shipped state at current HEAD)**: Pointer fields stored; no traversal code; no genesis principal.
- **Root cause**: `cascading-upstream-deferral` (bootstrap chain is load-bearing for revocation cascade + audit; deferred without explicit flag).

## Where visible in code
- **File(s)**: [`nodes.rs:507`](../../../../../../modules/crates/domain/src/model/nodes.rs#L507) TemplateKind::SystemBootstrap; no traversal code.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "system:genesis\|walk_provenance\|AuthorityChain" modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Define `system:genesis` principal constant. Bootstrap template approval at system-init creates the root AR. Repo method: `walk_provenance_chain(grant) -> Vec<AuthRequest>` returning path to bootstrap. Acceptance test asserts every mintable grant chains to bootstrap. Cross-reference with D-new-18 (revocation cascade walks the same chain).
- **Implementation chunk this belongs to**: CH-14
- **Dependencies on other drifts**: none
- **Estimated effort**: 3 engineer-days.
- **Risk to concept alignment if deferred further**: HIGH — audit cannot prove grant provenance; revocation cascade impossible without walker.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: TemplateKind::SystemBootstrap doc mentions the axiom
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report + user verification — SystemBootstrap kind exists but no genesis + walker)
