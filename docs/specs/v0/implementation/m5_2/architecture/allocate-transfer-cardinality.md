<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P3 — allocate/transfer cardinality + AllocateRefinement design + apply_transfer_grant compound-tx atomicity per ADR-0052) -->

# `allocate` / `transfer` cardinality + `AllocateRefinement` — design page

> **Status:** [EXISTS] as of CH-08 (M5.2). The typed refinement struct ships at [`modules/crates/domain/src/permissions/allocate_refinement.rs`](../../../../../../modules/crates/domain/src/permissions/allocate_refinement.rs); `Grant.allocate_refinement` denormalisation lands at [`model/nodes.rs:701`](../../../../../../modules/crates/domain/src/model/nodes.rs); the compound-tx primitive `Repository::apply_transfer_grant` lands at [`repository.rs:1397`](../../../../../../modules/crates/domain/src/repository.rs) with adapter impls at [`in_memory.rs:1868`](../../../../../../modules/crates/domain/src/in_memory.rs) and [`store/src/repo_impl.rs:2691`](../../../../../../modules/crates/store/src/repo_impl.rs). For the normative concept-doc references, read [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics" lines 197 (refinement framing) + 199–207 (cardinality table + atomic-revocation language) and [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"`allocate` as the Umbrella Action" lines 48–54.

---

## Overview

CH-08 closes two drifts surfaced in the M5.1 concept audit:

- **D-new-13 (HIGH-A)** — concept-doc 02 lines 199–207 specify a structural cardinality split between `allocate` (additive, sender retains full share — Rust `Arc::clone`) and `transfer` (exclusive, sender loses authority — Rust `let y = x;`); pre-CH-08 nothing distinguished the two at grant-mint time. Concept-doc 02 line 206 mandates *"on approval, rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership"* atomically.
- **D-new-29 (LOW-B)** — concept-doc 02 line 197 + concept-doc 03 line 54 specify that `allocate` is **umbrella** with refinements expressed as constraints; the canonical example is `allocate: no_further_delegation`. Pre-CH-08 `Grant` had no typed refinement field — refinements would have lived as untyped `Vec<String>` on `ToolAuthorityManifest.constraints`.

CH-08 ships:

1. A new compound-tx primitive `Repository::apply_transfer_grant` that atomically (1) rewrites the resource's `OWNED_BY` edge to the new owner, (2) revokes the sender's `[transfer]`-eligible grant, (3) mints the recipient's grant. Both adapters (`InMemoryRepository` + `SurrealStore`) implement the same atomic-or-rollback semantics.
2. A new typed `AllocateRefinement` struct with the F2.A-locked minimal field set `{ no_further_delegation: bool, max_depth: Option<u8> }`.
3. A new `Grant.allocate_refinement: Option<AllocateRefinement>` field with `#[serde(default)]` shielding (mirrors CH-11 `approval_mode` + CH-13 `audit_class` field-add precedents).

ADR-0052 records the design decisions (sub-decisions D52.1–D52.7); this page is the operator-facing description.

---

## Cardinality table (mirrors concept-doc 02 lines 199–207)

| Operation | Cardinality | Effect on sender | Rust analogue |
|-----------|-------------|------------------|---------------|
| `allocate` | **additive** | Retains full share | `Arc::clone(&x)` — multiple owners, reference-counted |
| `transfer` | **exclusive** | Loses all authority on the resource | `let y = x;` — move; `x` is no longer valid |

**Allocate path** (additive, unchanged at this chunk): a `[allocate]`-scoped grant is minted via the existing `Repository::create_grant` boundary. Sender's prior grant is **untouched** — concept-doc 02 lines 199–204 *"sender retains full share"* invariant. Multiple principals may hold `[allocate]` on the same resource concurrently; this is precisely how co-ownership is expressed in the graph.

**Transfer path** (exclusive, new at this chunk): a `[transfer]`-scoped grant is minted via the new `Repository::apply_transfer_grant` compound-tx primitive. The single transactional unit covers all three structural writes — sender revocation, OWNED_BY edge rewrite, recipient mint — so concept-doc 02 line 206's *"rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership"* is realisable as a single Repository call with atomic-or-rollback semantics.

The two paths are deliberately **distinct method surfaces** on Repository: `create_grant` is additive and idempotent; `apply_transfer_grant` is the only path that mutates the sender's authority. This makes the cardinality split structural (not advisory) — there is no way to mint a transfer-grant without going through the compound-tx primitive that revokes the sender atomically.

---

## Three-write atomic compound-tx (`apply_transfer_grant`)

The compound-tx atomically performs the three writes mandated by concept-doc 02 line 206:

```text
apply_transfer_grant(payload):

  // pre-flight (read-side) — rejection leaves zero partial state
  ensure sender_grant exists                  // RepositoryError::NotFound
  ensure sender_grant.revoked_at is None      // RepositoryError::Conflict
  ensure recipient_grant.resource == sender_grant.resource    // InvalidArgument
  ensure recipient_grant.holder == payload.new_owner          // InvalidArgument
  ensure new_owner is not a system principal                  // InvalidArgument
  ensure OWNED_BY edge exists for sender's holder             // Conflict

  // compound tx (atomic — all writes commit together OR none commit)
  begin
    (1) rewrite OWNED_BY edge → new_owner          // structural ownership move
    (2) sender_grant.revoked_at = payload.at       // residual-authority revocation
    (3) create recipient_grant                     // new authority mint
  commit
```

**Atomicity guarantee** (ADR-0052 §D52.5). Both adapters provide atomic-or-rollback semantics at the same trait-level contract:

- `InMemoryRepository`: the entire pre-flight + three-write block runs under a single `Mutex` write-lock (`self.lock()`), matching the existing `apply_org_creation` / `apply_bootstrap_claim` precedents (ADR-0022, ADR-0023). Pre-flight rejection returns before any mutation, so a failed pre-flight leaves zero partial state.
- `SurrealStore`: the three writes are wrapped in a single `BEGIN TRANSACTION ... COMMIT TRANSACTION` block. SurrealDB rolls back the entire transaction on any per-statement failure. Pre-flight runs read-only outside the transaction and surfaces validation errors before opening the write tx, matching the symmetry of the in-memory adapter.

**No production caller wired today** — F5.A forward-defensive primitive. `Action::Transfer` has zero runtime mint sites at chunk close (verified by `grep -rn "Action::Transfer" modules/crates/`); the first M6+ chunk introducing a real transfer-flow surface (e.g., resource hand-off UX) consumes the primitive in a single `apply_transfer_grant` callsite.

---

## `AllocateRefinement` field semantics

```rust
// modules/crates/domain/src/permissions/allocate_refinement.rs
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AllocateRefinement {
    #[serde(default)]
    pub no_further_delegation: bool,
    #[serde(default)]
    pub max_depth: Option<u8>,
}
```

Concept-doc 02 line 197 verbatim: *"Specific refinements can be expressed as constraints on the Grant. For example, `allocate: no_further_delegation` as a constraint removes the allocate-with-delegation sub-capability, producing a 'one-level' shareholder who can issue operational sub-grants but cannot create further shareholders."*

Field semantics (F2.A user-locked at plan approval — minimal closure):

- **`no_further_delegation: bool`** — when `true`, removes the allocate-with-delegation sub-capability (the holder can issue operational sub-grants but cannot create further `[allocate]` shareholders). Hard binary per concept doc.
- **`max_depth: Option<u8>`** — when `Some(n)`, limits the depth of the allocation chain. `Some(1)` operationalises concept-doc 02 line 197's *"one-level shareholder"* exactly; `None` = unbounded. Natural generalisation of `no_further_delegation`.

**Defaults**: `AllocateRefinement::default() = { no_further_delegation: false, max_depth: None }` — unbounded, matching pre-CH-08 behaviour.

`Grant.allocate_refinement: Option<AllocateRefinement>` is `None` for non-`[allocate]` grants AND for legacy/pre-CH-08 grants (the `#[serde(default)]` shielding decodes a missing field as `None`). When `Some(_)`, the contained refinement narrows the allocate sub-capabilities. This mirrors the CH-11 D48.1 (`Grant.approval_mode`) + CH-13 D50.5 (`Grant.audit_class`) field-add patterns.

**Forward-defensive at this chunk** — `AllocateRefinement` is **typed and serialised** but has **no engine-side enforcement** wired today. The Permission Check engine's `step_5_scope_resolution` / `step_2a_ceiling` do not consume `allocate_refinement` at chunk close; the first M6+ chunk wiring engine-side enforcement (e.g., refusing a sub-grant mint when the parent grant has `no_further_delegation: true`) consumes the typed field. CH-08 closes the **structural-typing** half of D-new-29; engine-enforcement closure is a follow-up.

---

## SurrealStore atomicity strategy (DELETE + RELATE inside `BEGIN`/`COMMIT`)

**Implementation detail discovered at P2** (worth recording here for future SurrealStore compound-tx authors). SurrealDB graph edges (`RELATE` rows) carry immutable `in` / `out` endpoints — there is no UPDATE-equivalent for rewriting an edge's resource ↔ owner pair. The compound-tx therefore performs:

```sql
BEGIN TRANSACTION;
  -- (1a) DELETE the existing OWNED_BY edge (capture its `in` resource end first via pre-flight read)
  DELETE type::thing('owned_by', $old_edge_id) RETURN NONE;
  -- (1b) RELATE a fresh OWNED_BY edge from the same resource → new_owner
  LET $f = type::thing('node', $resource_id);
  LET $t = type::thing('node', $new_owner_id);
  RELATE $f -> owned_by -> $t SET id = type::thing('owned_by', $new_edge_id) RETURN NONE;
  -- (2) Revoke sender's grant
  UPDATE type::thing('grant', $sender_id) SET revoked_at = <datetime> $at RETURN NONE;
  -- (3) Mint recipient's grant
  CREATE type::thing('grant', $recipient_id) CONTENT $recipient_body RETURN NONE;
COMMIT TRANSACTION;
```

The DELETE+RELATE pair is the structural rewrite per concept-doc 02 line 206. It is **inside** the same `BEGIN`/`COMMIT` block as the sender revocation + recipient mint, so partial-failure semantics apply uniformly: any failure rolls back all four statements together.

The pre-flight probe (read-side `SELECT record::id(id) AS edge_id, record::id(in) AS resource_id FROM owned_by WHERE record::id(out) = $owner_id LIMIT 1`) captures the existing edge id + resource end **outside** the transaction, so a missing OWNED_BY edge returns `RepositoryError::Conflict` cleanly without opening the write tx.

---

## Forward-defensive note

Both deliverables ship as forward-defensive primitives at CH-08:

- **`apply_transfer_grant`** has no production caller — `Action::Transfer` has zero runtime mint sites at chunk close. The structural-enforcement boundary closes D-new-13 (HIGH); the first M6+ chunk wiring a real transfer-flow surface (e.g., resource hand-off UX) consumes the primitive.
- **`Grant.allocate_refinement`** has no engine-side enforcement — the Permission Check engine does not consume the field at chunk close. The typed-field structural-boundary closes the typing-half of D-new-29 (LOW); the first M6+ chunk wiring engine-side enforcement (e.g., refusing further sub-grant mints when the parent grant has `no_further_delegation: true`) consumes the typed field.

This mirrors the CH-12 forward-defensive precedent: CH-12 shipped `validate_tag_write_on_session` + `frozen_tag_write_rejected` audit-event builder forward-defensively, with the first wiring caller landing in a future chunk. The structural primitive ships now even though no production flow exists yet because the cardinality split is **structural** — once a transfer-flow surface ships in M6+, it must consume `apply_transfer_grant` (there is no shortcut to mint a transfer-grant without sender revocation).

---

## Cross-references

- **ADR-0052** — [`m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md`](../decisions/0052-allocate-transfer-cardinality-and-refinement.md). Sub-decisions D52.1 (cardinality enforcement boundary), D52.2 (refinement field set), D52.3 (Grant denormalisation), D52.4 (zero-migration), D52.5 (compound-tx atomicity), D52.6 (Allocate-path additive invariant), D52.7 (`Default` impl on Grant).
- **Concept docs**:
  - [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics" lines 197 (refinement framing) + 199–207 (cardinality table + atomic-revocation language at line 206).
  - [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"`allocate` as the Umbrella Action" lines 48–54 (umbrella + refinement-as-constraint framing).
- **Drifts closed**:
  - [`m5_1/drifts/D-new-13.md`](../../m5_1/drifts/D-new-13.md) — HIGH-A — closed by F1.A + F5.A (`apply_transfer_grant`).
  - [`m5_1/drifts/D-new-29.md`](../../m5_1/drifts/D-new-29.md) — LOW-B — closed by F2.A + F3.A (`AllocateRefinement` + `Grant.allocate_refinement`).
- **Operations runbook**: [`allocate-transfer-cardinality-operations.md`](../operations/allocate-transfer-cardinality-operations.md).
