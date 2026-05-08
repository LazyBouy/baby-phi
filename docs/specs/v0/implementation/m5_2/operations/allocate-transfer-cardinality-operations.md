<!-- Last verified: 2026-05-07 by Claude Code (CH-08 P3 — operations playbook for allocate/transfer cardinality including apply_transfer_grant atomic-rollback guarantee + RepositoryError mapping) -->

# `allocate` / `transfer` cardinality operations runbook

> **Audience:** SREs and operators triaging allocate/transfer grant flows. Pair this page with [`allocate-transfer-cardinality.md`](../architecture/allocate-transfer-cardinality.md) (design) and [ADR-0052](../decisions/0052-allocate-transfer-cardinality-and-refinement.md).

---

## Error-code reference (`Repository::apply_transfer_grant`)

`apply_transfer_grant` returns the **existing** `RepositoryError` variants (no new variant added at CH-08 — the existing surface covers every cardinality-violation case cleanly). Each variant maps to a specific operator triage path.

| `RepositoryError` variant | When it fires | Operator triage |
|---|---|---|
| `NotFound` | `payload.sender_grant_id` does not resolve to a stored grant. | Check the AR's slot history for the grant id the approve-flow attempted to revoke. The id may be a stale reference from a long-running AR whose sender grant was revoked out-of-band. **Action:** re-emit the AR with a fresh sender-grant lookup. |
| `Conflict("sender grant {id} already revoked")` | Sender's `[transfer]`-eligible grant is already in `revoked_at != None` state — re-entry attempt or concurrent AR approval. | This is **expected re-entry safety** under the idempotency framing: a duplicate approve-side-effect harmlessly returns Conflict. **Action:** verify no actual transfer-flow regressed; check the audit log for an earlier successful `apply_transfer_grant` on the same `sender_grant_id`. If found → no-op (atomic re-entry). If not found → escalate (concurrent revocation by an unrelated path, which is a structural-invariant violation). |
| `Conflict("recipient grant {id} already exists")` (in-memory) / SurrealDB primary-key violation (Surreal) | Caller passed a non-fresh `recipient_grant.id`. Should never happen in production paths; only callers that mint grant IDs out-of-band could trigger this. | **Action:** treat as a caller bug. The recipient grant id MUST be freshly minted per call. |
| `Conflict("no OWNED_BY edge for sender's holder ...; transfer cardinality precondition broken")` | The probe for an existing `OWNED_BY` edge whose `out` end matches the sender's holder returns zero rows. Concept-doc 02 line 206 framing: *"rewrites the OWNED_BY edge"* — there must be an edge to rewrite. | **Action:** investigate why the sender's holder is not an OWNED_BY-edge endpoint. If the sender holds a `[transfer]`-eligible grant but no ownership edge → the grant was minted without going through the standard ownership flow (e.g., bootstrap residue). Surface the inconsistency to engineering as a new drift; do **not** manually seed an OWNED_BY edge as a workaround. |
| `InvalidArgument("recipient_grant.resource ({}) must match sender_grant.resource ({})")` | Caller passed a recipient grant for a different resource than the sender's existing grant. Cardinality precondition: the sender currently has authority over what they're transferring. | **Action:** treat as a caller bug. The compound-tx is scoped to a single resource; transferring multiple resources requires multiple `apply_transfer_grant` calls. |
| `InvalidArgument("recipient_grant.holder must match payload.new_owner")` | Defensive — the recipient of the grant must be the new owner per concept-doc 02 line 206 single-update-of-`OWNED_BY`-edge framing. | **Action:** treat as a caller bug. |
| `InvalidArgument("PrincipalRef::System(...) has no NodeId; apply_transfer_grant does not target system axioms")` | Caller passed `PrincipalRef::System(...)` for sender or new owner. System principals (`system:genesis`, etc.) are hardcoded axioms with no `NodeId` representation. | **Action:** treat as a caller bug. Transfer flows are between non-system principals; system axioms are not transferable. |

The mapping reuses `NotFound`, `Conflict`, and `InvalidArgument` rather than introducing a new `RepositoryError::TransferGrantConflict` variant — the existing semantics fit cleanly without contract bloat.

---

## SRE playbook — "what if `apply_transfer_grant` fails halfway?"

**Atomic-rollback guarantee** (ADR-0052 §D52.5). Both adapters provide atomic-or-rollback semantics on the three-write compound tx (rewrite `OWNED_BY` + revoke sender + mint recipient):

- **`InMemoryRepository`**: the entire pre-flight + three-write block runs under a single `Mutex` write-lock; pre-flight rejection returns before any mutation, so a failed call leaves zero partial state.
- **`SurrealStore`**: the three writes are wrapped in a single `BEGIN TRANSACTION ... COMMIT TRANSACTION` block. SurrealDB rolls back the entire transaction on any per-statement failure.

**Operator action when a transfer compound-tx returns an error:**

1. **No manual cleanup.** The atomic-rollback guarantee means there are no half-states to repair — either all three writes committed (success path) or none of them did (rollback path).
2. **Re-emit the AR.** Once the underlying error condition is addressed (e.g., sender's grant gets re-issued, OWNED_BY edge is repaired by the upstream flow), re-emit the AR carrying `Action::Transfer`. The compound-tx is idempotent under the re-entry safety variant: a re-call after a successful first call returns `Conflict("sender grant {id} already revoked")` cleanly without further mutation.
3. **Cross-pod / multi-replica K8s deployments.** SurrealDB's `BEGIN`/`COMMIT` blocks are executed server-side under the SurrealStore single-writer model (CHK8S-D-08); cross-pod determinism is preserved (no in-process state involved). The compound-tx works identically under `open_embedded` (single-process) and `open_remote` (replicated cluster) per ADR-0033 D33.2.

**Diagnostic SurrealDB query** to verify atomicity post-incident:

```sql
-- Confirm the three-write atomic outcome on a single resource. After a successful
-- apply_transfer_grant, expect: (1) one OWNED_BY edge with out = new_owner;
-- (2) the sender's grant with revoked_at set; (3) the recipient's fresh grant.
SELECT
  (SELECT id, out FROM owned_by WHERE record::id(in) = $resource_id LIMIT 1) AS owned_by_edge,
  (SELECT id, revoked_at FROM grant WHERE record::id(id) = $sender_grant_id) AS sender,
  (SELECT id, holder, action FROM grant WHERE record::id(id) = $recipient_grant_id) AS recipient
FROM ONLY 1;
```

If any of the three rows is in an unexpected state (e.g., OWNED_BY edge still points at the sender, sender grant has `revoked_at = NONE`, or recipient grant is missing), the rollback contract is broken — escalate as a structural-invariant violation; do **not** attempt manual repair.

---

## Audit-event reference

**No new audit events at CH-08.** The compound-tx primitive is forward-defensive and has no production caller wired today (`Action::Transfer` has zero runtime mint sites). The first M6+ chunk wiring a real transfer-flow surface (e.g., resource hand-off UX) defines the audit-event integration — likely a `transfer.grant.minted` event paired with a `transfer.grant.sender_revoked` event (or a single composite event mirroring the compound-tx structure).

**CH-13 audit-class composition continues to apply** (see [`audit-class-composition-operations.md`](audit-class-composition-operations.md)). The recipient grant minted by `apply_transfer_grant` carries an `audit_class` field — when the M6+ caller assembles the `recipient_grant`, it composes `audit_class` per the strictest-wins rule (org default × adoption AR × override). Operators querying audit events for the recipient grant should expect the standard `audit_class_source` diff field on whatever audit event the M6+ caller emits.

**Audit log integrity for transfer flows.** Per concept-doc 02 line 206 *"past actions stand in the audit log, but future actions on the resource require new authority"* — the sender's pre-revocation actions remain in the audit log under the sender's identity; the recipient's post-mint actions are logged under the recipient's identity. The compound-tx itself does not back-fill or rewrite any historical audit event — it only mutates current authority state.

---

## `AllocateRefinement` diagnostic

**How to inspect a Grant's refinement state** (SurrealDB query):

```sql
SELECT
  id,
  holder,
  action,
  resource.uri AS resource_uri,
  allocate_refinement
FROM grant
WHERE allocate_refinement IS NOT NONE;
```

**Interpreting the field:**

| `allocate_refinement` value | Operational meaning |
|---|---|
| `NONE` (or omitted) | Pre-CH-08 grant **OR** non-`[allocate]`-scope grant **OR** unbounded `[allocate]` grant. Indistinguishable from the storage row alone — disambiguation requires checking `action`. |
| `{ no_further_delegation: false, max_depth: NONE }` | Unbounded `[allocate]` grant — equivalent to `NONE` semantics; explicit `Some(default)` representation only happens if a caller explicitly constructs it. |
| `{ no_further_delegation: true, max_depth: NONE }` | "One-level shareholder" per concept-doc 02 line 197 — holder can issue operational sub-grants but **cannot** create further `[allocate]` shareholders. |
| `{ no_further_delegation: false, max_depth: Some(1) }` | "One-level shareholder" via `max_depth` — equivalent to `no_further_delegation: true` in v0 semantics; `max_depth = Some(1)` operationalises *"one-level shareholder"* per concept-doc 02 line 197 verbatim. |
| `{ no_further_delegation: true, max_depth: Some(N>0) }` | **Inconsistent** — the strictest constraint applies (`no_further_delegation: true` wins → no further sub-grants regardless of `max_depth`). Surface as a caller-bug warning if observed in production. |
| `{ no_further_delegation: false, max_depth: Some(N) }` for `N > 1` | Bounded multi-level allocation. v0 ships the typed field; engine-side enforcement is forward-defensive — see *Forward-defensive note* in the [architecture page](../architecture/allocate-transfer-cardinality.md). |

**Forward-defensive at this chunk.** `Grant.allocate_refinement` is **typed and serialised** but the Permission Check engine does not consume it at chunk close. The first M6+ chunk wiring engine-side enforcement (e.g., refusing further `[allocate]` sub-grant mints when a parent grant has `no_further_delegation: true`) consumes the typed field. Until then, the field is operator-visible (audit-log inspection) but does not constrain runtime authority resolution.

**Audit-log interpretation of refinement state.** When an audit event records a `Grant` (or grant_id reference), the operator can cross-reference `allocate_refinement` via the SurrealDB query above. A grant minted with `Some(AllocateRefinement { no_further_delegation: true, max_depth: Some(1) })` should be interpreted in audit logs as a **shareholder grant explicitly scoped to one allocation level** — operators auditing delegation chains can confirm any sub-grant minted from this parent is structurally invalid (once engine-enforcement ships in M6+).

---

## Cross-references

- **ADR-0052** — [`m5_2/decisions/0052-allocate-transfer-cardinality-and-refinement.md`](../decisions/0052-allocate-transfer-cardinality-and-refinement.md). Sub-decisions D52.1 (cardinality enforcement boundary), D52.5 (compound-tx atomicity guarantee), D52.6 (Allocate-path additive invariant).
- **Architecture page**: [`allocate-transfer-cardinality.md`](../architecture/allocate-transfer-cardinality.md) — design-level companion.
- **Concept docs**:
  - [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"`allocate` Scope Semantics" lines 197 + 199–207.
  - [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"`allocate` as the Umbrella Action" lines 48–54.
- **Drifts closed**:
  - [`m5_1/drifts/D-new-13.md`](../../m5_1/drifts/D-new-13.md) — HIGH-A.
  - [`m5_1/drifts/D-new-29.md`](../../m5_1/drifts/D-new-29.md) — LOW-B.
- **Related runbooks**: [`audit-class-composition-operations.md`](audit-class-composition-operations.md) — strictest-wins composition that applies to the recipient grant.
