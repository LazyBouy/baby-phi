<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P4 — Status flipped Proposed → Accepted; chunk closes drifts D-new-13 + D-new-29). -->
<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P2 — §D52.1/§D52.5/§D52.6 bodies filled; Repository::apply_transfer_grant compound-tx primitive shipped on InMemoryRepository + SurrealStore). -->

# ADR-0052 — `allocate` / `transfer` cardinality + `AllocateRefinement` typed constraint

**Status: Accepted**

**Date:** 2026-05-07
**Chunk:** CH-08
**Closes:**
- [`D-new-13`](../../m5_1/drifts/D-new-13.md) (HIGH, A) — `allocate` / `transfer` cardinality enforcement gap. Concept-doc 02 line 206 specifies *"on approval, rewrites the `OWNED_BY` edge and revokes any residual authority the sender held through ownership"* but at chunk-open nothing distinguishes allocate from transfer at grant-mint time. CH-08 closes the structural-enforcement half via `Repository::apply_transfer_grant` compound-tx primitive (forward-defensive; no production caller wired at this chunk per F5.A).
- [`D-new-29`](../../m5_1/drifts/D-new-29.md) (LOW, B) — `allocate` refinement encoding gap. Concept-doc 02 line 197 names `allocate: no_further_delegation` as the canonical refinement; concept-doc 03 line 54 frames refinements as constraints. CH-08 closes the typed-encoding half via `Grant.allocate_refinement: Option<AllocateRefinement>` field with `{ no_further_delegation: bool, max_depth: Option<u8> }` field set.

---

## Context

(Body fills at P1 — context paragraph will pin concept-doc 02 §"`allocate` Scope Semantics" lines 199–207 (cardinality table at lines 201–204; atomic-revocation language at line 206; refinement framing at line 197) and concept-doc 03 §"`allocate` as the Umbrella Action" lines 48–54 (umbrella + refinement-as-constraint framing) as the canonical specs. Will note that today's `Action::Allocate` and `Action::Transfer` exist at `permissions/action.rs:57-58` but `Action::Transfer` has zero runtime mint sites — the cardinality distinction is silent in code per drift D-new-13. Will also note that today's `Grant` carries no typed refinement field — `ToolAuthorityManifest.constraints: Vec<String>` at `nodes.rs:1008` is the closest extant constraint surface but is stringly-typed and lives on the manifest, not the grant — the refinement encoding gap per drift D-new-29.)

## Forks

- F1 → F1.A (`Repository::apply_transfer_grant` compound-tx primitive — mirrors ADR-0022/0023 precedent) — user-locked at plan approval 2026-05-07.
- F2 → F2.A (`{ no_further_delegation: bool, max_depth: Option<u8> }` minimal v0 field set) — user-locked at plan approval 2026-05-07.
- F3 → F3.A (`Grant.allocate_refinement: Option<AllocateRefinement>` with `#[serde(default)]` — mirrors CH-11 D48.1 + CH-13 D50.5 precedents) — user-locked at plan approval 2026-05-07.
- F4 → F4.A (zero migrations — direct consequence of F3.A) — user-locked at plan approval 2026-05-07.
- F5 → F5.A (forward-defensive primitive; no production caller wired — mirrors CH-12 frozen-tag-write precedent) — user-locked at plan approval 2026-05-07.

All five forks at planner-recommendation; Direct-approval criteria hold under the all-A path per plan §0.

---

## Decision

### D52.1 — Cardinality enforcement boundary (F1.A)

Cardinality enforcement for the `[transfer]` action lives at `Repository::apply_transfer_grant` — a new compound-tx Repository method shipped at `domain/src/repository.rs`. The trait method body atomically performs three writes: (1) rewrites the resource's `OWNED_BY` edge to point at `payload.new_owner`, (2) revokes the sender's matching grant (sets `revoked_at = payload.at`), (3) mints `payload.recipient_grant`. The method is modelled on the existing `apply_org_creation` precedent (ADR-0022/0023) and signals the structural-enforcement boundary at the layer the data crosses on a transfer — single transactional unit, single Repository call.

The `Allocate` path is **untouched** at this chunk per §D52.6 — the existing `create_grant` boundary remains additive by construction; sender's existing grant survives a sibling Allocate-grant mint on the same resource (concept-doc 02 lines 199–204 cardinality table preserved).

**Rejected alternatives:**

- **F1.B — Inline cardinality branch in `AuthRequest` slot-aggregation flow (`auth_requests/transitions.rs::transition_slot`).** `transitions.rs` is **pure** — every fn takes `&AuthRequest` and returns `Result<AuthRequest, _>` (verified at `transitions.rs:1-9`). Adding repository side-effects breaks the proptest invariant + concept doc 02's *"AuthRequest is the slot-machine, grant-mint is downstream of approval"* framing.
- **F1.C — Engine-side runtime invariant at `step_5_scope_resolution`.** Detection, not enforcement — the broken state is already in storage. Concept-doc 02 line 206's Rust-analogue framing (`let y = x;` move semantics) is a write-time guarantee, not a read-time check.

**Plan §3 Artifact B prediction verified at P2 close**: 0 production callsites (forward-defensive — F5.A) + 2 trait-impl blocks (`InMemoryRepository::apply_transfer_grant` at `domain/src/in_memory.rs`; `SurrealStore::apply_transfer_grant` at `store/src/repo_impl.rs`). No 3rd Repository impl was discovered at P2 (pause-discipline trigger from plan §3 Artifact B did not fire).

Verified at P2 close: `domain/src/repository.rs` (trait + `TransferGrantPayload` struct); `domain/src/in_memory.rs` (InMemory impl); `store/src/repo_impl.rs` (SurrealStore impl).

### D52.2 — `AllocateRefinement` field set (F2.A)

The `AllocateRefinement` struct ships at `domain/src/permissions/allocate_refinement.rs` with the minimal closed v0 field set:

```rust
pub struct AllocateRefinement {
    #[serde(default)]
    pub no_further_delegation: bool,
    #[serde(default)]
    pub max_depth: Option<u8>,
}
```

`no_further_delegation` is the canonical refinement named at concept-doc 02 line 197 *"`allocate: no_further_delegation` as a constraint removes the allocate-with-delegation sub-capability, producing a 'one-level' shareholder who can issue operational sub-grants but cannot create further shareholders"*. `max_depth: Option<u8>` is the natural generalisation from the same concept-doc's *"one-level shareholder"* framing — `Some(1)` operationalises *"one-level"* exactly; `Some(n)` generalises to n-level chains; `None` = unbounded. Default value `{ no_further_delegation: false, max_depth: None }` matches pre-CH-08 behaviour (unbounded allocation authority).

Both fields carry `#[serde(default)]` shielding so future struct extensions (e.g., adding a third field) deserialise cleanly across stored grant rows.

Rejected alternatives:
- **F2.B (expanded set)** — `restrict_to_resources: Vec<ResourceRef>`, `restrict_to_scopes: Vec<...>`, etc. Concept docs name no such fields; speculative; v0 should ship minimal. Future fields land via drift-list expansion when concrete use-cases arrive.
- **F2.C (migration-only empty-struct)** — defeats the chunk's purpose since D-new-29 explicitly names `no_further_delegation`.

Verified at `domain/src/permissions/allocate_refinement.rs` (P1, this chunk).

### D52.3 — `Grant.allocate_refinement` denormalisation (F3.A precedent: D48.1 + D50.5)

`Grant` extends with the new field at `domain/src/model/nodes.rs:649-704`:

```rust
#[serde(default)]
pub allocate_refinement: Option<crate::permissions::AllocateRefinement>,
```

`None` for non-`[allocate]` grants AND for legacy/pre-CH-08 grants (serde-default-shielded). When `Some(_)`, the contained `AllocateRefinement` narrows the allocate sub-capabilities (`no_further_delegation`, `max_depth`) per concept-doc 02 line 197 + concept-doc 03 line 54.

This mirrors the CH-11 / ADR-0048 §D48.1 precedent of adding `Grant.approval_mode: ApprovalMode` (verified at `nodes.rs:675`) and the CH-13 / ADR-0050 §D50.5 precedent of adding `Grant.audit_class: AuditClass` (verified at `nodes.rs:693`). Both prior fields ship with `#[serde(default)]` shielding; pre-CH-08 grants decode as `allocate_refinement: None`. Storage round-trip is symmetric: the SurrealDB `GrantRow` translator at `store/src/repo_impl.rs::GrantRow` rides the field under the existing FLEXIBLE TYPE object column with the same `#[serde(default)]` shielding pattern (no schema migration — see D52.4).

Rejected alternatives:
- **F3.B (reserved-key in a new `Grant.constraint_context: HashMap<String, serde_json::Value>` map)** — adds a new field of a different shape; less typed; would require a parallel typed accessor anyway; loses static type-checking of refinement semantics.
- **F3.C (new `Constraint` enum with variant `AllocateRefinement(AllocateRefinement)`)** — no `Constraint` enum exists today (`Vec<String>` on `ToolAuthorityManifest.constraints` at `nodes.rs:1008` is the closest); inventing one as part of CH-08 inflates scope to ~3× forward-scope.

Verified at `domain/src/model/nodes.rs` + `store/src/repo_impl.rs` (P1, this chunk).

### D52.4 — Zero-migration property (F4.A)

CH-08 ships zero migrations. F3.A adds an `Option<AllocateRefinement>` field with `#[serde(default)]`; SurrealDB schemaless persistence absorbs the new field, and serde-default shielding covers the round-trip for pre-CH-08 stored grants without a schema change. The `GrantRow` row-translator at `store/src/repo_impl.rs::GrantRow` adds the same `#[serde(default)] allocate_refinement: Option<...>` field; pre-CH-08 stored grant rows lack the column entirely, deserialise as `None`, and round-trip cleanly back to `Grant.allocate_refinement: None` via `GrantRow::into_domain`.

Pattern matches the two existing precedents on the same struct:
- CH-11 / ADR-0048 §D48.1 added `Grant.approval_mode: ApprovalMode` with no migration.
- CH-13 / ADR-0050 §D50.5 added `Grant.audit_class: AuditClass` with no migration.

Migration count remains at chunk-open baseline (0013_per_session_consent_gating.surql) at chunk close.

F4.B (migration 0014) was rejected because it only applies under F3.B / F3.C — the user-locked path is F3.A.

Verified at chunk close: `ls modules/crates/store/migrations/` shows no `0014_*.surql`.

### D52.5 — `apply_transfer_grant` compound-tx atomicity guarantee (F5.A)

`Repository::apply_transfer_grant` atomically performs three writes:

1. **Rewrite `OWNED_BY` edge for the resource** to point at `payload.new_owner`.
2. **Revoke the sender's matching `[transfer]`-eligible grant** (sets `revoked_at = payload.at`).
3. **Mint `payload.recipient_grant`**.

All three writes commit atomically OR roll back together. Concept-doc 02 line 206 verbatim: *"on approval, rewrites the OWNED_BY edge and revokes any residual authority the sender held through ownership"*.

**`TransferGrantPayload` shape** (at `domain/src/repository.rs`):

```rust
pub struct TransferGrantPayload {
    pub sender_grant_id: GrantId,
    pub new_owner: PrincipalRef,
    pub recipient_grant: Grant,
    pub at: DateTime<Utc>,
}
```

The payload deliberately omits separate `recipient` / `resource` fields — both are derivable from `recipient_grant.holder` and `recipient_grant.resource` respectively, so the call sees a single source of truth (the recipient's grant) for who/what is receiving authority.

**`InMemoryRepository` impl** (at `domain/src/in_memory.rs`): pre-flight validates every invariant (sender-grant exists; sender-grant not already revoked; recipient-grant id is fresh; recipient-grant resource matches sender-grant resource; recipient-grant holder matches `new_owner`; an OWNED_BY edge currently points the resource at sender-holder), then mutates under the existing `Mutex`-guarded state lock (`lock()?` covers atomicity in-memory — same pattern as `apply_org_creation` / `apply_bootstrap_claim`).

**`SurrealStore` impl** (at `store/src/repo_impl.rs`): pre-flight validation runs as separate read queries; then a single SurrealQL `BEGIN TRANSACTION ... COMMIT TRANSACTION` block wraps the three writes. SurrealDB graph-edge `in` / `out` fields are immutable on RELATE rows, so the OWNED_BY-edge rewrite is implemented as `DELETE` of the existing edge + fresh `RELATE` with a new edge id — both inside the same BEGIN/COMMIT block (atomically committed or rolled back). Mirrors existing compound-tx patterns at `repo_impl.rs::create_manages_edge` (~line 2769) and `repo_impl.rs::create_has_agent_supervisor_edge` (~line 2901).

**Atomicity verified at P2 close** via integration tests:

- `domain/tests/transfer_grant_atomicity_test.rs::transfer_revokes_sender_and_mints_recipient` — happy path (InMemory): post-state has sender's grant revoked + new OWNED_BY edge + recipient's grant minted.
- `domain/tests/transfer_grant_atomicity_test.rs::transfer_rolls_back_on_sender_grant_missing` — non-existent sender_grant_id → `RepositoryError::NotFound`; recipient's grant NOT minted.
- `domain/tests/transfer_grant_atomicity_test.rs::transfer_rolls_back_on_sender_grant_already_revoked` — re-entry safety: pre-revoked sender → `RepositoryError::Conflict`; recipient's grant NOT minted; sender's `revoked_at` stays at original timestamp.
- `store/tests/transfer_grant_surreal_test.rs::transfer_grant_atomic_round_trip_surreal` — happy path (SurrealStore in-memory mode): three writes commit atomically; `owned_by` edge count remains at 1 after rewrite.
- `store/tests/transfer_grant_surreal_test.rs::transfer_rolls_back_on_already_revoked_surreal` — atomic rollback under SurrealDB BEGIN/COMMIT: recipient's grant NOT minted; OWNED_BY edge unchanged.

**Forward-defensive (F5.A user-locked):** No production caller wires this method today. `Action::Transfer` has zero runtime mint sites — verified at plan-draft via `grep -rn "Action::Transfer" modules/crates/`. The first M6+ chunk introducing a real transfer-flow surface (e.g., a resource hand-off UX) consumes this primitive at a single callsite. Mirrors the CH-12 forward-defensive precedent (`validate_tag_write_on_session` + `frozen_tag_write_rejected` audit-event builder shipped without runtime caller).

**Rejected alternatives:**

- **F5.B — Wire AR approve flow at slot-close.** Inflates chunk to ~3 engineer-days + introduces a new listener type (AR does not currently scope by `action: Vec<Action>` — actions are on Grant, not AR; routing the slot-aggregation by transfer-action requires a new pipeline stage). Defer to M6+ when a real transfer-flow surface is shipped.
- **F5.C — Defer the entire primitive to M6+.** Leaves D-new-13 (HIGH-A security-boundary drift) open against concept-doc 02 line 206 mandate; rejected per the chunk's quality-over-speed restatement (§1).

**Cardinality precondition error mapping** (verified at P2 close — no new `RepositoryError` variant was needed):

| Failure mode | `RepositoryError` variant |
|---|---|
| `sender_grant_id` does not exist | `NotFound` |
| Sender's grant is already revoked | `Conflict { reason: "already revoked" }` |
| Recipient grant id duplicates an existing grant | `Conflict { reason: "already exists" }` |
| `recipient_grant.resource` ≠ `sender_grant.resource` | `InvalidArgument` |
| `recipient_grant.holder` ≠ `new_owner` | `InvalidArgument` |
| `PrincipalRef::System(_)` end (no NodeId backing) | `InvalidArgument` |
| No existing OWNED_BY edge for sender (precondition broken) | `Conflict { reason: "no OWNED_BY edge ... cardinality precondition broken" }` |

Existing `Conflict(String)` + `InvalidArgument(String)` + `NotFound` variants cleanly cover the cardinality-violation surface; the optional `TransferGrantConflict { reason }` variant flagged at plan §7 P2 deliverable 4 was **not added** (existing variants suffice).

### D52.6 — `Allocate`-path unchanged invariant

The chunk is **purely additive on the Allocate path**. `Repository::create_grant` (the v0 grant-mint boundary at `domain/src/repository.rs`) remains additive by construction — sender's existing grant survives a sibling Allocate-grant mint on the same resource. Concept-doc 02 lines 199–204 cardinality table preserved verbatim:

| Scope | Cardinality | Semantic | Rust analogue |
|---|---|---|---|
| `[allocate]` | **additive** | Sender retains full share; recipient gets parallel grant | `Arc::clone` |
| `[transfer]` | **exclusive** | Sender loses authority; recipient gets the only grant | `let y = x;` (move) |

The CH-08 chunk does **not** introduce any branch, check, or new code on the Allocate path. The new `apply_transfer_grant` compound-tx primitive is gated on the `[transfer]` cardinality path explicitly — it does not run on Allocate-grant mints (which continue to flow through `create_grant` unchanged).

**Regression-risk pin verified at P2 close**: `domain/tests/transfer_grant_atomicity_test.rs::allocate_path_remains_additive` — seeds a sender's `[transfer]`-eligible grant + mints a sibling Allocate-grant on the same resource via `create_grant`; asserts (a) sender's grant remains unrevoked (`revoked_at = None`), (b) allocate-grant exists alongside, (c) `list_grants_for_principal` returns each grant under its respective holder. The test pins the additive invariant explicitly so future refactors that accidentally alter the Allocate path break the test rather than silently shift cardinality.

### D52.7 — `Default` impl on Grant for cascade neutralisation

**Decision: strategy (b) — explicit `allocate_refinement: None` at every callsite.**

The implementer evaluated strategy (a) — deriving `Default` on `Grant` to enable `..Default::default()` shorthand at every `Grant { ... }` literal-construction site — and rejected it because two of `Grant`'s field types do not implement `Default` and adding `Default` to them would change non-trivial semantics:

- `holder: PrincipalRef` — enum at `nodes.rs:791` with no obvious zero-variant. `Default` would have to pick a canonical variant (e.g., `System(String::new())`), which is semantically misleading for a permission-model identity type.
- `resource: ResourceRef` — struct at `nodes.rs:803` carrying a single `uri: String` field. A `Default` would yield `ResourceRef { uri: String::new() }`, which mocks a real resource — fragile in tests that assert on URIs.

Other Grant fields would have worked under (a): `id: GrantId` has `Default` via the `id_newtype!` macro at `model/ids.rs:35`; `action: Vec<Action>` defaults to empty; `fundamentals: Vec<Fundamental>` defaults to empty; `descends_from: Option<AuthRequestId>` defaults to `None`; `delegable: bool` defaults to `false`; `issued_at: DateTime<Utc>` has chrono's `Default` (UNIX epoch); `revoked_at: Option<...>` defaults to `None`; `approval_mode: ApprovalMode` has `Default` via `#[default] Implicit` at `nodes.rs:743`; `audit_class: AuditClass` rides under `#[serde(default = "Grant::default_audit_class")]` so it doesn't need a struct-level `Default`.

Strategy (b) — explicit-field cascade — was applied at **27 callsites across 21 files** (see plan §3 Artifact A). Within the [13–25] band's 1.5× buffer (38 sites); pause discipline NOT triggered. Strategy (b) also matches the two prior precedents:
- CH-11 added `approval_mode` via per-callsite explicit-field cascade (verified by inspection of `engine.rs:1027` test helper).
- CH-13 added `audit_class` via the same per-callsite cascade pattern.

Adopting strategy (b) preserves the existing semantics of `PrincipalRef` and `ResourceRef` (no `Default` impl) and keeps the cascade pattern symmetric with the two field-add precedents. Each callsite carries `allocate_refinement: None` in deterministic line-trailing position after `audit_class` to keep diffs reviewable.

Rationale for the field-shielding convention: cascade-cost minimisation per CH-12 retro Row 2 + chunk-planner v3 additive-field discipline. The `#[serde(default)]` attribute on the field shields against future drift — a Grant deserialised from a row that omits the field still decodes cleanly.

Verified at P1 close: `cargo build --workspace` succeeds; cascade count = 27 (within band).

---

## Cross-references

- **(a) Originating concept-doc + sections**:
  - [`permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics" lines 179–207 (cardinality table at lines 201–204; atomic-revocation language at line 206; refinement framing at line 197).
  - [`permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"`allocate` as the Umbrella Action" lines 48–54 (umbrella + refinement-as-constraint framing).
- **(b) Closed drifts**:
  - [`D-new-13`](../../m5_1/drifts/D-new-13.md) (HIGH, A) — allocate/transfer cardinality enforcement gap.
  - [`D-new-29`](../../m5_1/drifts/D-new-29.md) (LOW, B) — typed `AllocateRefinement` encoding gap.
- **(c) Prior ADRs cited as precedent**:
  - [ADR-0022](../../m3/decisions/0022-org-creation-compound-transaction.md) — compound-tx pattern for org creation (F1.A precedent for `apply_transfer_grant` compound-tx shape).
  - [ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — inherit-from-snapshot (F1.A relevance: Repository compound-tx as the structural-enforcement boundary).
  - [ADR-0028](../../m4/decisions/0028-domain-event-bus.md) — audit-event emission via `AuditEmitter` (A7 conformance pattern if `apply_transfer_grant` emits an audit event).
  - [ADR-0033](./0033-k8s-prep-refactors.md) — CH-K8S-PREP conforming criteria (referenced for K8s-neutral verification at plan §3.B).
  - [ADR-0043](./0043-typed-action-vocabulary.md) — typed `Action` enum (CH-04 prerequisite — `Allocate` + `Transfer` variants live at `action.rs:57-58`).
  - [ADR-0048](./0048-per-session-consent-gating.md) §D48.1 — `Grant.approval_mode` field-add precedent (F3.A's structural twin: additive Grant field with `#[serde(default)]` shielding, zero migration).
  - [ADR-0050](./0050-audit-class-composition-strictest-wins.md) §D50.5 — `Grant.audit_class` field-add precedent (F3.A's structural twin: same shielding pattern as D48.1).
- **(d) Forward-scope row cross-reference**:
  - [`baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) lines 91–96 (CH-08 row).

---

## Phase placement

- **P0** — Plan archive + ADR-0052 scaffold (this file, Proposed) + cycle-index row.
- **P1** — `AllocateRefinement` struct + `Grant.allocate_refinement` field with `#[serde(default)]` shielding + `Grant` Default-derive decision + ADR §D52.2, §D52.3, §D52.4, §D52.7 bodies filled.
- **P2** — `Repository::apply_transfer_grant` compound-tx primitive on `InMemoryRepository` + `SurrealStore` + atomicity unit + integration tests + ADR §D52.1, §D52.5, §D52.6 bodies filled.
- **P3** — Architecture page (`m5_2/architecture/allocate-transfer-cardinality.md`) + operations page (`m5_2/operations/allocate-transfer-cardinality-operations.md`) + concept-audit-matrix row updates + verified-header refreshes on `permissions/02` + `permissions/03`.
- **P4** — ADR Proposed → Accepted flip + drift remediation (D-new-13, D-new-29) + final CI guards.
