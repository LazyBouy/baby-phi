<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal — Status flipped to remediated; closes via `Repository::revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` BFS algorithm with depth cap 32 + Template-revoke handler flip from single-hop to recursive; acceptance test `acceptance_authority_chain::revoke_cascades_to_grandchildren` asserts grandchild revocation; existing single-hop `revoke_grants_by_descends_from` preserved verbatim for M2 `narrow_mcp_tenants` per ADR-0033 contract; ADR-0053 §D53.4.) -->
<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-18 — Grant revocation cascade (walks grants by `provenance` to forward-only revoke descendants) not implemented

## Identification
- **ID**: D-new-18
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `revocation-cascade`, `provenance-walk`
- **Blocks**: Template-revoke handler correctness (page 12 revoke-cascade test exists but may not verify full cascade semantics if walker missing)
- **Blocked-by**: D-new-14 (provenance walker)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/README.md`](../../../concepts/permissions/README.md) §"Provenance"; [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §"Ad-hoc AR and revocation cascade"
- **Concept claim**: Revocation of an AR cascade-revokes every Grant descended from it via `provenance: auth_request:req-X`. Cascade walks the tree forward-only (past reads stand).
- **Contradiction**: Need to verify: Repository has `revoke_grants_by_descends_from(ar_id, at)` method used by Template revoke handler. But does it walk the full tree (AR → Grant → child AR → child Grant → ...) or only one hop?
- **Classification**: `partially-honored` (probably single-hop; needs audit)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Full tree walk, forward-only revocation.
- **Reality (shipped state at current HEAD)**: Needs verification — [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) has `revoke_grants_by_descends_from`; Template revoke handler calls it. But multi-hop tree walking to grandchildren may not be exercised.
- **Root cause**: `underspecified-plan` — single-hop is the common case; concept's full-tree walk not explicitly tested.

## Where visible in code
- **File(s)**: [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) `revoke_grants_by_descends_from`; [`server/src/platform/templates/revoke.rs`](../../../../../../modules/crates/server/src/platform/templates/revoke.rs) caller
- **Test evidence**: `acceptance_authority_templates::revoke_cascade_count_grants_revoked` exists — need to check if multi-hop case is tested.
- **Grep for regression**: review whether tree-walking recursion is present; otherwise add.

## Remediation scope (estimate only)
- **Approach (sketch)**: Audit current revoke logic (may be fine for single-hop case). Add multi-hop tree-walk test. If implementation is single-hop only, extend to walk the full tree (grants whose provenance is an AR issued by an already-revoked grant's AR).
- **Implementation chunk this belongs to**: CH-14
- **Dependencies on other drifts**: D-new-14 (walker infrastructure)
- **Estimated effort**: 1–2 engineer-days (depending on audit outcome).
- **Risk to concept alignment if deferred further**: HIGH if multi-hop is broken (revoked-child grants leak); LOW if single-hop turns out to be the only case baby-phi supports at M5.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: Repository `revoke_grants_by_descends_from` doc
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
- 2026-05-08 — `in-chunk-plan` — CH-14 plan drafted at `docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md`; ADR-0053 Proposed; F3.A user-locked — closes via `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` BFS algorithm + Template-revoke handler flip + acceptance test asserting grandchild-grant revocation (existing single-hop preserved verbatim for M2 `narrow_mcp_tenants` per ADR-0033 contract).
- 2026-05-08 — `remediated` — CH-14 chunk-seal — `Repository::revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` BFS algorithm shipped on both backends (`in_memory.rs` + `store/src/repo_impl_m5.rs`) with depth cap 32 + `RepositoryError::ProvenanceCycleDepthExceeded`; Template-revoke handler at `server/src/platform/templates/revoke.rs:90` flipped from `revoke_grants_by_descends_from` (single-hop) to `revoke_grants_by_descends_from_recursive` (recursive); single-hop sibling preserved verbatim for M2 `narrow_mcp_tenants` per ADR-0033; 5 acceptance tests at `acceptance_authority_chain.rs` (cascade + idempotency + depth-cap) + 8 unit tests at `authority_chain_walker.rs`; ADR-0053 §D53.4 Accepted.
