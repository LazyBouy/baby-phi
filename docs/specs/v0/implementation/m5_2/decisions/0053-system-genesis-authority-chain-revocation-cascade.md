<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal gate-2 inline correction — §D53.7 wording tightened to match shipped code: per-cascaded-AR `auth_request.revoked` emission now lives at `templates/revoke.rs:99` iterating `CascadeResult.cascaded_ars`; cascade method return type changed from `Vec<GrantId>` to `domain::repository::CascadeResult { revoked_grants, cascaded_ars }` at `repository.rs:170` + both backend impls + 2 new acceptance tests; level-0 axis behaviour clarified as "summary event covers level-0; companion `auth_request.revoked` from `revoke_ar` continues to be discarded" — pre-CH-14 wording incorrectly implied the handler emitted level-0.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-14 P4 chunk-seal — Status flipped Proposed → Accepted; sub-decisions D53.1–D53.7 pinned by P0–P3 deliverables: typed const + helper at `domain::permissions::axioms::{SYSTEM_GENESIS_PRINCIPAL, system_genesis_principal, is_bootstrap_ar, system_bootstrap_template_id}`; `AuthRequest.descends_from_grant: Option<GrantId>` field at `model/nodes.rs:847`; `Repository::walk_provenance_chain` + `Repository::revoke_grants_by_descends_from_recursive` at `domain/src/repository.rs` with InMemoryRepository impl at `in_memory.rs` + SurrealStore impl at `store/src/repo_impl_m5.rs`; Template-revoke handler flipped at `server/src/platform/templates/revoke.rs:90` to recursive variant; migration 0014 `descends_from_grant option<string>` ON auth_request; D-new-14 + D-new-18 remediated; 5 matrix rows flipped letter-for-letter per CH-12 F-AUDB-1.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-14 P0 — ADR drafted as Proposed) -->

# ADR-0053 — `system:genesis` axiom + authority-chain walker + recursive revocation cascade

**Status: Accepted**

**Date:** 2026-05-08
**Chunk:** CH-14
**Closes:**
- [`D-new-14`](../../m5_1/drifts/D-new-14.md) (HIGH, A) — `system:genesis` axiomatic principal + authority-chain traversal missing. Concept doc 02 §"System Bootstrap Template" + concept doc 04 §"The Authority Chain" + concept doc README §"Provenance" specify a typed root principal + a walkable provenance tree. Today `"system:genesis"` is a magic-string at 11 sites with no compile-time guard; no walker exists. CH-14 closes the typed-root half via `domain::permissions::axioms::{SYSTEM_GENESIS_PRINCIPAL, system_genesis_principal, is_bootstrap_ar}` and the walker half via `Repository::walk_provenance_chain(grant) -> Vec<AuthRequest>` on both backends with depth-cap 32 + cycle detection.
- [`D-new-18`](../../m5_1/drifts/D-new-18.md) (HIGH, A) — Grant revocation cascade walks provenance forward-only; today it's single-hop. Concept doc 08 §9.3 specifies tree-wide forward-only cascade. CH-14 closes via `Repository::revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` BFS algorithm + Template-revoke handler flip; the existing single-hop method is preserved verbatim for back-compat with M2 `narrow_mcp_tenants` (which legitimately wants single-hop per AR).

---

## Context

(Body fills at P1–P3 — context paragraph will pin concept doc `permissions/02-auth-request.md` §"System Bootstrap Template" lines 449–487 as the canonical spec for the axiomatic root, concept doc `permissions/04-manifest-and-resolution.md` §"The Authority Chain" lines 510–547 as the canonical spec for tree-traversal, concept doc `permissions/08-worked-example.md` §9.3 lines 360–364 as the canonical spec for forward-only cascade, and concept doc `permissions/README.md` §"Provenance" lines 92–114 as the canonical spec for chain trust.)

## Forks

- F1 → F1.A (`pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis"` + `pub fn system_genesis_principal() -> PrincipalRef` helper in new `domain::permissions::axioms` module — magic-string consolidation without enum-variant migration) — user-locked at plan approval 2026-05-08.
- F2 → F2.A (`walk_provenance_chain(grant) -> Vec<AuthRequest>` root-to-leaf — matches forward-scope literal signature) — user-locked at plan approval 2026-05-08.
- F3 → F3.A (BFS in repository layer with depth cap 32; new `revoke_grants_by_descends_from_recursive(ar, at) -> Vec<GrantId>` trait method; existing single-hop preserved verbatim; new `AuthRequest.descends_from_grant: Option<GrantId>` field with `#[serde(default)]`; migration 0014 single-column-add nullable) — user-locked at plan approval 2026-05-08.
- F4 → F4.A (keep current claim-time minting; document divergence from `fires_on: system_init` here in §D53.6) — user-locked at plan approval 2026-05-08.
- F5 → F5.A (two-witness predicate `requestor == system_genesis_principal() && provenance_template == Some(SYSTEM_BOOTSTRAP_TEMPLATE_ID)`) — user-locked at plan approval 2026-05-08.

All five forks at planner-recommendation (F1–F5 user-locked at plan approval to F1.A / F2.A / F3.A / F4.A / F5.A — all align with planner recommendation).

---

## Decision

### D53.1 — `system:genesis` typed const + helper-fn (F1.A)

A new module `domain::permissions::axioms` ships:

```rust
pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis";

pub fn system_genesis_principal() -> PrincipalRef {
    PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL.into())
}

pub fn system_bootstrap_template_id() -> TemplateId {
    TemplateId::from_uuid(uuid::Uuid::nil())
}
```

The const is the single source of truth for the magic string `"system:genesis"`. The helper-fn `system_genesis_principal()` is the canonical constructor. Production callsites at `bootstrap/claim.rs:192,203` flip from `PrincipalRef::System("system:genesis".into())` to `system_genesis_principal()`. Test fixtures at `repository_test.rs` (4 sites) + `template_e_props.rs` (1 site) flip identically. Docstring + JSON-sample literals at `nodes.rs:795,804` + `allocate_refinement.rs:65` stay literal — they describe wire format, not Rust callsites.

**Rejected alternatives:**
- F1.B (new `PrincipalRef::SystemAxiom { axiom_name: SystemAxiom }` variant) — wire-format migration too aggressive for a magic-string consolidation chunk.
- F1.C (private const + opaque fn only) — encapsulation breaks down because tests outside `domain` need a public path anyway.

### D53.2 — `is_bootstrap_ar` two-witness predicate (F5.A)

```rust
pub fn is_bootstrap_ar(ar: &AuthRequest) -> bool {
    matches!(&ar.requestor, PrincipalRef::System(s) if s == SYSTEM_GENESIS_PRINCIPAL)
        && ar.provenance_template == Some(system_bootstrap_template_id())
}
```

Both witnesses must match: requestor is `system:genesis` AND `provenance_template` is the all-zero UUID. Either witness alone is insufficient — a future code path that mints a non-bootstrap AR with `system:genesis` requestor (e.g., for auditor-bot system-internal ops) would alias the bootstrap if we matched on requestor only; a future Template AR with the all-zero UUID would alias too if we matched on `provenance_template` only. The two-witness AND-predicate is the structural safety net concept-doc 02 lines 478–486 implies.

**Rejected alternatives:**
- F5.B (requestor only) — see above.
- F5.C (depth-cap-only termination) — loses the "auditable termination at the axiom" property concept-doc 02 line 480 specifies.

### D53.3 — `walk_provenance_chain(grant)` repo method + ID-based BFS (F2.A)

`Repository::walk_provenance_chain(grant: GrantId) -> RepositoryResult<Vec<AuthRequest>>` returns the chain root-to-leaf: bootstrap AR at index 0, the grant's direct AR at the last index. Empty for legacy grants whose `descends_from` is `None`. Returns `Err(RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: 32 })` if the chain depth exceeds 32 hops (defensive guard against schema bugs / cycles).

The walker walks via `Grant.descends_from -> AR` (existing field) **and** the new `AuthRequest.descends_from_grant: Option<GrantId>` field (D53.5). Termination is by `is_bootstrap_ar(current_ar) == true` OR `current_ar.descends_from_grant == None` OR depth-cap.

**Rejected alternatives:**
- F2.B (`Vec<(AuthRequest, Option<Grant>)>` pairs) — doubles store-side queries; richer audit context not needed at this chunk.
- F2.C (typed `ProvenanceChain` wrapper) — premature shape; today every chain has at least 1 element.

### D53.4 — `revoke_grants_by_descends_from_recursive` BFS (F3.A)

`Repository::revoke_grants_by_descends_from_recursive(ar: AuthRequestId, at: DateTime<Utc>) -> RepositoryResult<Vec<GrantId>>` ships as a NEW trait method. Algorithm: BFS from `ar`. Level 0: find all live grants with `descends_from == ar` → revoke them. For each revoked grant, find all live ARs with `descends_from_grant == Some(grant.id)` → for each such AR, find its descendant grants → revoke them. Continue until no more live descendants OR depth cap 32 → `ProvenanceCycleDepthExceeded`.

Returns the **flat ordered list of all revoked grants across all levels**.

The existing `revoke_grants_by_descends_from(ar, at)` is **preserved verbatim** (single-hop semantics). M2 `narrow_mcp_tenants` continues to call the single-hop variant per AR (it cascades via its own per-AR loop, and explicitly wants single-hop semantics per AR per ADR-0033). Template-revoke handler flips from single-hop to recursive (P3).

**Rejected alternatives:**
- F3.B (single SurrealDB recursive query) — graph-traversal syntax differs across backends; in-memory adapter would still need its own iterative impl.
- F3.C (DFS recursion in domain layer + per-grant `revoke_grant` calls) — N round-trips; transaction-atomicity weakens.

### D53.5 — `AuthRequest.descends_from_grant: Option<GrantId>` field-add (F3.A consequence)

A new `pub descends_from_grant: Option<GrantId>` field ships on `AuthRequest` with `#[serde(default)]` shielding (mirrors the CH-08 D52.3 / CH-11 D48.1 / CH-13 D50.5 typed-field-cascade precedent). Default value `None` is semantically valid for every pre-CH-14 AR — those ARs have no parent grant by definition. Migration 0014 adds `descends_from_grant option<string>` column on the SurrealDB `auth_request` table (idempotent `DEFINE FIELD OVERWRITE`).

**Cascade scope at CH-14:** the field is plumbed through ~28 AR-construction sites with `descends_from_grant: None`. Adoption-AR-side wiring (Template A/B/C/D/E adoption-AR builders setting `Some(firing_grant_id)`) is **deferred** to a successor chunk per F3.A scope-control decision — adoption builders are pure-fns that take no grant context today, and plumbing the firing-grant id through them aggravates the cascade beyond CH-14's scope. The walker still functions correctly at chunk close: every shipped chain shape terminates at the bootstrap via the existing `Grant.descends_from -> AR` field; the missing AR-to-Grant link only matters when the chain has > 1 AR hop, which today's data shape does not produce because adoption-ARs use `system:genesis` as approver, terminating the walk one hop early.

### D53.6 — Claim-time-as-system-init divergence (F4.A)

Concept doc 02 line 469 says the bootstrap AR `fires_on: system_init`. Reality: the bootstrap AR + Grant + audit event are minted at **first claim** (`server/src/bootstrap/claim.rs:188-274`), not at `bootstrap-init` (which only generates the credential at `init.rs:34-47`).

This divergence is **accepted** as a simplification: the bootstrap AR is functionally axiomatic from claim-time onward; pre-claim there is no platform admin to render an audit, no migration to apply, no grants to descend from. Concept doc 02's `fires_on: system_init` reads as "fires once, at the entrypoint that establishes the platform" — claim is that entrypoint. The walker test asserts that *every grant chains to bootstrap* — this passes the moment claim succeeds.

**Rejected alternatives:**
- F4.B (eager bootstrap-AR minting at `phi-server` startup, idempotent skip-if-exists) — aggravates K8s-axis A4 (multi-pod startup race for first-write); out of scope for CH-14.
- F4.C (mint at `bootstrap-init` time) — inverts the current "credential-first, claim-later" two-step; orphan-bootstrap-AR risk if claim never happens.

### D53.7 — Audit-event emission semantics on cascade

When Template-revoke triggers the recursive cascade, audit-event emission preserves the existing single-writer pattern:

1. **One** `template.revoked` summary event from the Template-revoke handler (carries `grant_count_revoked` aggregating all levels). Existing emission preserved. This summary event is the canonical audit record covering the level-0 adoption AR's revocation.
2. **N − 1** additional `auth_request.revoked` events — one per **level-≥1** cascaded AR. The `revoke_ar` domain helper at `auth_requests/revocation.rs:57` builds each event; the Template-revoke handler iterates `CascadeResult.cascaded_ars` and emits per-AR. The level-0 adoption AR is intentionally NOT in `cascaded_ars` and its companion `auth_request.revoked` event from the existing pre-CH-14 `let (next, _auth_ar_audit_event) = revoke_ar(&ar, ...)` call continues to be discarded — the `template.revoked` summary event covers the level-0 axis without a paired per-AR event (existing behaviour preserved verbatim; revisited only if a future chunk requires symmetric per-AR emission for the level-0 path).
3. **Implicit:** N `grant.revoked_at` flips on the affected `Grant` rows. No separate `grant.revoked` audit event exists today; the row mutation IS the audit signal at the storage layer.

The cascade-method return type is `domain::repository::CascadeResult { revoked_grants: Vec<GrantId>, cascaded_ars: Vec<AuthRequestId> }` — `revoked_grants` feeds the summary event's `grant_count_revoked`; `cascaded_ars` feeds the per-AR emission loop in the handler. Cascade method stays in the Repository layer (audit-emitter-free); handler owns audit emission per CH-08 / CH-13 single-writer precedent.

`canonical_bytes` for `auth_request.revoked` excludes `prev_event_hash` per existing chain semantics. Cross-pod determinism preserved (BFS insertion order is stable per backend; both InMemoryRepository and SurrealStore produce the same `cascaded_ars` ordering for any fixed input subtree).

---

## Cross-references

- **Concept docs:** `permissions/02-auth-request.md` §"System Bootstrap Template" lines 449–487; `permissions/04-manifest-and-resolution.md` §"The Authority Chain" lines 510–547; `permissions/08-worked-example.md` §9.3 lines 360–364; `permissions/README.md` §"Provenance" lines 92–114.
- **Closed drifts:** `D-new-14`, `D-new-18`.
- **Prior ADRs cited as precedent (milestone-prefixed paths per CH-08 retro Row 1):**
  - [`m3/decisions/0022-org-creation-compound-transaction.md`](../../m3/decisions/0022-org-creation-compound-transaction.md) — compound-tx primitive precedent. Recursive revoke is a multi-write transaction over the Grant + AR tables; the BFS sub-tx pattern mirrors the org-creation primitive's atomicity envelope.
  - [`m5_2/decisions/0033-k8s-prep-refactors.md`](./0033-k8s-prep-refactors.md) §D33.2 (`SurrealStore::open_remote`) — relevant for trait-shape A5 conformance; the new `walk_provenance_chain` + `revoke_grants_by_descends_from_recursive` methods are `&dyn Repository`-dispatchable.
  - [`m5_2/decisions/0050-audit-class-composition-strictest-wins.md`](./0050-audit-class-composition-strictest-wins.md) (CH-13) §D50.5 — Grant denormalisation + audit-event-source pattern; precedent for §D53.5 typed-field cascade.
  - [`m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md`](./0052-allocate-transfer-cardinality-and-refinement.md) (CH-08) §D52.3 — typed-field cascade pattern; precedent for §D53.5 `Option<GrantId>` field-add cascade.
  - [`m5_2/decisions/0048-per-session-consent-gating.md`](./0048-per-session-consent-gating.md) (CH-11) §D48.1 — `Grant.approval_mode` denormalisation with `#[serde(default)]`; precedent for §D53.5 serde back-compat.
  - [`m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`](./0051-multi-scope-cascade-contractor-model.md) (CH-07) §D51.1 — cascade algorithm inside engine; the BFS in §D53.4 is structurally analogous to CH-07's cascade resolution but in the revoke-direction.
- **Forward-scope row:** [`baby-phi/docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 139–143.

---

## Consequences

**Positive:**
- Magic-string `"system:genesis"` collapses to a single typed source.
- Walker codifies "every grant chains to bootstrap" as an executable assertion.
- Multi-hop revocation cascade closes the silent-in-code gap concept doc 08 §9.3 specifies.
- Adoption-AR-side wiring stays defer-friendly via the typed `Option<GrantId>` field shape (forward-defensive plumbing for chain depth > 2).

**Negative:**
- `AuthRequest` carries one extra optional field forever (≈8 bytes serialized when populated, 0 when None).
- The deferred adoption-AR-side wiring leaves the walker structurally over-built relative to today's chain depth (typically 1 — bootstrap AR only). This is acceptable per CH-13 D50.5 forward-defensive precedent; the field exists from CH-14 onward so successor chunks need not migrate.

**Neutral:**
- `revoke_grants_by_descends_from` (single-hop) lives on as a sibling to the recursive variant. M2 `narrow_mcp_tenants` continues calling single-hop per AR per ADR-0033 contract.

---

## Validation

P4 chunk-seal verification:
1. `RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets -j 4` — green.
2. `cargo test --workspace -j 4` — 1426–1430 passed / 0 failed / 2 ignored.
3. `bash scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` — all 4 green.
4. Acceptance test `acceptance_authority_chain::every_grant_chains_to_bootstrap_after_claim` — passes.
5. Acceptance test `acceptance_authority_chain::revoke_cascades_to_grandchildren` — passes with 3-hop chain.
6. Drift D-new-14 + D-new-18 → `remediated`.
7. Audit-matrix rows (lines 44, 135, 157, 181, 231) → `honored`.
