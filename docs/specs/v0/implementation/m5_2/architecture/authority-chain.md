<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal gate-2 inline correction — design page synced with shipped per-cascaded-AR `auth_request.revoked` emission + AR-state-transition logic; §D53.4 signature now returns `CascadeResult { revoked_grants, cascaded_ars }`; §A7 K8s-axis row notes per-AR emission BFS-stable + cross-pod determinism preserved; "What this page does NOT cover" no longer lists per-cascaded-AR emission as deferred — D-CH14-FOLLOWUP-02 closed in-cycle.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-14 P3 — design page paired with ADR-0053 covering the typed `system:genesis` axiom + `is_bootstrap_ar` predicate, the leaf-to-root walker, the BFS-based recursive cascade with depth cap 32, and the claim-time-as-system-init divergence rationale.) -->

# Authority chain — design page

> **Status:** [EXISTS] as of CH-14 (M5.2). The typed axiom + predicate ship at [`modules/crates/domain/src/permissions/axioms.rs`](../../../../../../modules/crates/domain/src/permissions/axioms.rs); the walker + recursive cascade Repository methods land at [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) with two backend impls at [`in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs) and [`store/src/repo_impl_m5.rs`](../../../../../../modules/crates/store/src/repo_impl_m5.rs); the new `AuthRequest.descends_from_grant: Option<GrantId>` field rides on migration 0014. For the normative concept-doc reference, read [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"System Bootstrap Template" + [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"The Authority Chain".

---

## What this page covers

The permissions concept docs specify a tree-shaped authority model rooted at `system:genesis`. Every Grant must trace, via `descends_from` provenance, back to a hardcoded bootstrap Auth Request approved by the axiomatic `system:genesis` principal; revocation of any AR in the tree cascades forward — every descendant grant flips to `revoked_at`.

CH-14 lifts that model from concept doc into typed Rust. This page describes:

- The `system:genesis` typed const + helper-fn pair (D53.1).
- The `is_bootstrap_ar` two-witness termination predicate (D53.2).
- The leaf-to-root walker semantics (D53.3).
- The BFS-based recursive revocation cascade + depth cap (D53.4).
- The `AuthRequest.descends_from_grant: Option<GrantId>` field-add + migration 0014 (D53.5).
- The claim-time-as-system-init divergence rationale (D53.6).
- Cross-pod determinism + K8s axes A1–A7 conformance.

ADR-0053 records the design decisions (sub-decisions D53.1–D53.7); this page is the operator-facing description.

---

## `system:genesis` typed const + helper-fn (D53.1)

```rust
// modules/crates/domain/src/permissions/axioms.rs

pub const SYSTEM_GENESIS_PRINCIPAL: &str = "system:genesis";

pub fn system_genesis_principal() -> PrincipalRef {
    PrincipalRef::System(SYSTEM_GENESIS_PRINCIPAL.into())
}

pub fn system_bootstrap_template_id() -> TemplateId {
    TemplateId::from_uuid(uuid::Uuid::nil())
}
```

The const is the single source of truth for the magic string `"system:genesis"`. Production callsites at `bootstrap/claim.rs:192,204` (requestor + approver) and the test fixtures in `repository_test.rs` (`sample_auth_request` + bootstrap fixture) + `template_e_props.rs` use the helper-fn instead of the literal. Docstring + JSON-sample literals at `nodes.rs` + `allocate_refinement.rs` stay literal — they describe wire format, not Rust callsites.

`PrincipalRef` does NOT derive `PartialEq` (per the existing wire-format-stability invariant), so the two-witness predicate uses pattern matching rather than `==`.

---

## `is_bootstrap_ar` two-witness predicate (D53.2)

```rust
pub fn is_bootstrap_ar(ar: &AuthRequest) -> bool {
    let requestor_is_genesis = matches!(
        &ar.requestor,
        PrincipalRef::System(s) if s == SYSTEM_GENESIS_PRINCIPAL,
    );
    let provenance_is_bootstrap = ar.provenance_template == Some(system_bootstrap_template_id());
    requestor_is_genesis && provenance_is_bootstrap
}
```

Two witnesses must both match: requestor is `system:genesis` AND `provenance_template` is the all-zero UUID. Either witness alone is insufficient (a future code path that mints a non-bootstrap AR with `system:genesis` requestor — e.g., for an auditor-bot system-internal op — cannot alias the bootstrap; a future Template AR that happens to use the all-zero UUID cannot alias the bootstrap either). The two-witness AND-predicate is the structural safety net concept-doc 02 lines 478–486 implies. The walker, the recursive-cascade frontier-test, and the genesis-test fixture all share this predicate.

---

## Walker semantics (D53.3)

```rust
async fn walk_provenance_chain(
    &self,
    grant: GrantId,
) -> RepositoryResult<Vec<AuthRequest>>;
```

Returns the chain root-to-leaf: bootstrap AR at index 0, the grant's direct AR at the last index. Empty for legacy / pre-CH-14 grants whose `descends_from` is `None`. Returns `Err(RepositoryError::ProvenanceCycleDepthExceeded { depth_cap: 32 })` if the chain exceeds `MAX_PROVENANCE_DEPTH` hops (defensive cycle guard).

Algorithm (both backends):

1. Read `grant.descends_from`. If `None`, return empty.
2. For each level (capped at 32): fetch the AR. If `is_bootstrap_ar(ar) == true`, push and reverse → done. Else push, then climb via `ar.descends_from_grant -> Grant -> Grant.descends_from -> next AR`.
3. Any `None` terminator (chain ends without reaching the bootstrap, e.g., legacy AR whose `descends_from_grant` is `None`) returns the partial chain root-to-leaf.

Typical chain depth in production today is **1** — admin grant directly under the bootstrap AR. The walker is forward-defensive for future delegation-chain scenarios up to 32 hops.

---

## BFS-based recursive revocation cascade (D53.4)

```rust
async fn revoke_grants_by_descends_from_recursive(
    &self,
    ar: AuthRequestId,
    at: DateTime<Utc>,
) -> RepositoryResult<CascadeResult>;

pub struct CascadeResult {
    pub revoked_grants: Vec<GrantId>,
    pub cascaded_ars: Vec<AuthRequestId>, // level ≥ 1 only
}
```

Algorithm (both backends, breadth-first):

- **Level 0**: revoke every live grant whose `descends_from == ar` → `newly_revoked`.
- **Level N**: collect every AR whose `descends_from_grant` is in `newly_revoked` → next frontier (these go into `cascaded_ars`). Recurse: revoke every live grant whose `descends_from` is in that frontier.
- Terminate when no live descendants OR depth cap fires.

Returns `CascadeResult` carrying both the flat ordered list of every grant revoked across all levels AND the flat list of every level-≥1 cascaded AR (consumed by the handler for per-AR audit emission — see §A7).

The existing single-hop `revoke_grants_by_descends_from(ar, at)` is **preserved verbatim** for back-compat with M2 `narrow_mcp_tenants` (per ADR-0033 contract — the MCP-server tenant-narrowing flow legitimately wants single-hop semantics per AR; it cascades via its own per-AR loop). Template-revoke handler at `server/src/platform/templates/revoke.rs:90` flips from the single-hop variant to the recursive one and iterates `cascade.cascaded_ars` to (a) call `domain::auth_requests::revoke_ar(ar, at, ...)` per cascaded AR, (b) persist the AR-state-transition `Approved → Revoked` via `repo.update_auth_request`, and (c) emit one `auth_request.revoked` audit event per cascaded AR via the injected `AuditEmitter`. Idempotency-guarded: ARs already in a closed-terminal state are skipped.

The SurrealDB impl (in `repo_impl_m5.rs`) uses two SQL statements per level: `UPDATE grant SET revoked_at = $at WHERE descends_from INSIDE $frontier AND revoked_at IS NONE RETURN ...` for step (a), then `SELECT record::id(id) FROM auth_request WHERE descends_from_grant INSIDE $revoked` for step (b). Two queries per level for typical depths 1–4 = 2–8 queries per Template-revoke — fine for M5 admin-write throughput. The InMemory impl walks the same loop in-memory with `HashMap` lookups.

---

## `AuthRequest.descends_from_grant` field-add + migration 0014 (D53.5)

```rust
// modules/crates/domain/src/model/nodes.rs:818
pub struct AuthRequest {
    // ... pre-existing fields ...
    pub provenance_template: Option<TemplateId>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub descends_from_grant: Option<GrantId>,   // CH-14 / ADR-0053 D53.5
}
```

The `#[serde(default)]` shield decodes pre-CH-14 AR rows as `descends_from_grant = None` (mirrors the CH-08 D52.3 / CH-11 D48.1 / CH-13 D50.5 typed-field-cascade precedent). `None` is semantically valid for every pre-CH-14 AR — those ARs have no parent grant by definition, and the bootstrap node continues to satisfy that property post-CH-14 (the chain root has no parent grant per concept doc 02 lines 478–486).

The store-side column is added by **migration 0014**:

```surql
DEFINE FIELD OVERWRITE descends_from_grant ON auth_request TYPE option<string>;
```

Idempotent (`OVERWRITE`); nullable; un-indexed (per K8s-axis A4 conformance — see below). Migration count rises from **13** (post-CH-12 baseline) to **14**.

**Cascade scope at CH-14:** the field is plumbed through ~20 AR-construction sites with `descends_from_grant: None`. Adoption-AR-side wiring (Template A/B/C/D/E adoption-AR builders setting `Some(firing_grant_id)`) is **deferred** to a successor chunk per F3.A scope-control decision — see ADR-0053 §D53.5 + drift `D-CH14-FOLLOWUP-01`.

---

## Claim-time-as-system-init divergence (D53.6)

Concept doc 02 line 469 says the bootstrap AR `fires_on: system_init`. In code, the bootstrap AR + Grant + audit event are minted at **first claim** (`server/src/bootstrap/claim.rs:188-274`), not at `bootstrap-init` time (which only generates the credential).

This divergence is **accepted** as a simplification. The bootstrap AR is functionally axiomatic from claim-time onward; pre-claim there is no platform admin to render an audit, no migration to apply, no grants to descend from. Concept doc 02's `fires_on: system_init` reads as "fires once, at the entrypoint that establishes the platform" — claim is that entrypoint. The walker test asserts that *every grant chains to bootstrap* — this passes the moment claim succeeds.

Eager bootstrap-AR minting at `phi-server` startup (the F4.B alternative) would aggravate K8s-axis A4 (multi-pod startup race for first-write); out of scope for CH-14.

---

## Cross-pod determinism + K8s-axis conformance

| Axis | Surface | Outcome |
|---|---|---|
| **A1** | New in-process state | None — walker is a pure read; recursive revoke is sequenced repo writes (no `OnceCell`, no `RwLock`). |
| **A2** | New IPC channel | None. |
| **A3** | New pod-local resource | None. |
| **A4** | Migration / first-apply race | Migration 0014 is single-column-add nullable on existing table; CHK8S-D-05 unaggravated per CH-08 precedent. |
| **A5** | Trait-shape requirement | `walk_provenance_chain` + `revoke_grants_by_descends_from_recursive` are `&dyn Repository`-dispatchable; the SurrealDB impl uses idiomatic per-level SQL. |
| **A6** | Cross-pod state sharing | None new — every walk + revoke flows through `Repository` (already durable per ADR-0033 §D33.2). |
| **A7** | Audit hash-chain symmetry | Recursive revoke emits at the handler layer (`templates/revoke.rs`); the existing `template.revoked` summary event is preserved with an accurate multi-hop `grant_count_revoked`. The handler additionally emits one `auth_request.revoked` event per level-≥1 cascaded AR via `cascade.cascaded_ars`; emission is BFS-stable per backend so `canonical_bytes` is deterministic per cascade run. canonical_bytes for `auth_request.revoked` excludes `prev_event_hash` per existing chain semantics; cross-pod chain-replay determinism is preserved (the `prev_event_hash` chain is rebuilt by the emitter at write time, post-cascade). |

K8s-neutral. See CH-14 plan §3.B for the full readiness check.

---

## What this page does NOT cover

- **Adoption-AR-side wiring** — Template A/B/C/D/E adoption-AR builders setting `Some(firing_grant_id)` is deferred per ADR-0053 §D53.5 (drift `D-CH14-FOLLOWUP-01`). The walker still functions correctly at chunk close: every shipped chain shape terminates at the bootstrap via `Grant.descends_from -> AR`; the missing AR-to-Grant link only matters when chain depth > 2.
- **Eager bootstrap-AR minting at server startup** — F4.B alternative; aggravates K8s axis A4.

---

## Cross-references

- [ADR-0053](../decisions/0053-system-genesis-authority-chain-revocation-cascade.md) — the design decisions captured as sub-decisions D53.1–D53.7.
- [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"System Bootstrap Template" lines 449–487 — source of truth for the axiomatic root.
- [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"The Authority Chain" lines 510–547 — source of truth for tree-traversal.
- [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §9.3 lines 360–364 — source of truth for forward-only revocation cascade.
- [ADR-0052 — Allocate/Transfer cardinality](../decisions/0052-allocate-transfer-cardinality-and-refinement.md) §D52.3 — typed-field cascade precedent.
- [ADR-0050 — Audit-class composition](../decisions/0050-audit-class-composition-strictest-wins.md) §D50.5 — Grant denormalisation precedent.
- [ADR-0048 — Per-session consent gating](../decisions/0048-per-session-consent-gating.md) §D48.1 — `#[serde(default)]` shielding precedent.
- [`authority-chain-operations.md`](../operations/authority-chain-operations.md) — operator playbook.
- [Drift D-new-14](../../m5_1/drifts/D-new-14.md) — closed by CH-14.
- [Drift D-new-18](../../m5_1/drifts/D-new-18.md) — closed by CH-14.
