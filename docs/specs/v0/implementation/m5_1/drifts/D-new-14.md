<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal — Status flipped to remediated; closes via typed `domain::permissions::axioms::{SYSTEM_GENESIS_PRINCIPAL, system_genesis_principal, is_bootstrap_ar}` + `Repository::walk_provenance_chain(grant) -> Vec<AuthRequest>` on both backends; acceptance test `acceptance_authority_chain::every_grant_chains_to_bootstrap_after_claim` asserts walker reaches bootstrap; ADR-0053 §D53.1 / §D53.2 / §D53.3.) -->
<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-14 — `system:genesis` axiomatic principal + authority-chain traversal missing (provenance stored but not walkable to bootstrap)

## Identification
- **ID**: D-new-14
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
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
- 2026-05-08 — `in-chunk-plan` — CH-14 plan drafted at `docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md`; ADR-0053 Proposed; F1.A / F2.A / F3.A / F4.A / F5.A user-locked at plan approval — closes via typed `system_genesis_principal()` const + helper-fn + `is_bootstrap_ar(ar)` two-witness predicate + `walk_provenance_chain(grant) -> Vec<AuthRequest>` repo method.
- 2026-05-08 — `remediated` — CH-14 chunk-seal — typed const `SYSTEM_GENESIS_PRINCIPAL` + helper-fn `system_genesis_principal()` + two-witness predicate `is_bootstrap_ar(ar)` shipped at `modules/crates/domain/src/permissions/axioms.rs`; `Repository::walk_provenance_chain(grant) -> Vec<AuthRequest>` shipped on both backends (`in_memory.rs` + `store/src/repo_impl_m5.rs`) with depth cap 32 + `RepositoryError::ProvenanceCycleDepthExceeded`; `AuthRequest.descends_from_grant: Option<GrantId>` field-add with `#[serde(default)]` + migration 0014; bootstrap claim flipped to use `system_genesis_principal()` at `bootstrap/claim.rs:192,204`; 5 acceptance tests at `acceptance_authority_chain.rs` + 8 walker unit tests at `authority_chain_walker.rs` + 5 axiom unit tests in-mod-tests; ADR-0053 §D53.1 / §D53.2 / §D53.3 / §D53.5 Accepted.
