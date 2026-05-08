<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal gate-2 inline correction — added new `auth_request.revoked` row to the audit-event-sequence table for level-≥1 cascaded ARs; replaced "NOT emitted at CH-14 / deferred per FOLLOWUP-02" paragraph with the shipped behaviour: handler iterates `CascadeResult.cascaded_ars`, calls `revoke_ar` to build the event, persists state Approved→Revoked via `update_auth_request`, emits via the AuditEmitter; idempotency-guarded against already-closed-terminal ARs; D-CH14-FOLLOWUP-02 closed in-cycle.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-14 P3 chunk — operator playbook for the authority-chain walker + recursive revocation cascade. Pairs with `architecture/authority-chain.md` (design) and ADR-0053.) -->

# Authority chain operations runbook

> **Audience:** SREs and operators verifying or debugging the authority chain (provenance walker + multi-hop revocation cascade). Pair this page with [`authority-chain.md`](../architecture/authority-chain.md) (design) and [ADR-0053](../decisions/0053-system-genesis-authority-chain-revocation-cascade.md).

---

## Audit-event sequence on Template-revoke

When an operator runs `POST /api/v0/orgs/:org/authority-templates/:kind/revoke` (Template A/B/C/D), the cascade emits one summary audit event with an accurate multi-hop grant count:

| Event | Source | When | Payload highlights |
|---|---|---|---|
| `template.revoked` | `templates::revoke::revoke_template` | Once per call | `grant_count_revoked` = total grants revoked across **all levels** of the descend-tree. CH-14 flips this from single-hop to recursive — operators reading this number now see grandchildren. |
| `auth_request.revoked` | `templates::revoke::revoke_template` (per level-≥1 cascaded AR) | N − 1 per call (one per cascaded AR; 0 if cascade depth = 1) | Built by `domain::auth_requests::revoke_ar`; carries the cascaded AR's id + revocation reason + actor. Emission is BFS-stable per backend. The level-0 adoption AR is intentionally NOT in this stream — its revocation is covered by the `template.revoked` summary (existing pre-CH-14 behaviour preserved verbatim per ADR-0053 §D53.7). |
| `Grant.revoked_at` (row mutation) | `Repository::revoke_grants_by_descends_from_recursive` | One row mutation per cascaded grant | The `revoked_at` timestamp lands on every affected grant row; no separate per-grant audit event today. |

The `template.revoked` summary captures the cascade impact via `grant_count_revoked`. Per-cascaded-AR state-transitions to `Revoked` happen in lockstep with their `auth_request.revoked` emission (handler iterates `CascadeResult.cascaded_ars` and persists state via `Repository::update_auth_request`). Idempotency-guarded: cascaded ARs already in a closed-terminal state (e.g. previously cascaded) are skipped, so re-running a Template-revoke does not double-emit.

To verify the cascade impact post-revoke:

```surql
-- Count revoked grants under the adoption AR's descend-tree.
SELECT count() AS revoked_grant_count
  FROM grant
  WHERE revoked_at != NONE
    AND revoked_at >= $cascade_at;
```

Cross-reference the count against the `template.revoked` event's `grant_count_revoked` field — they must match.

---

## Forward-only-cascade — no recovery procedure

Per concept-doc 02 §"How Auth Request Approval Maps to a Grant" + concept-doc 08 §9.3, the cascade is **forward-only**. There is no rollback. If an operator revokes Template A on the wrong org:

1. **Past reads remain in audit logs** — every read that happened before the revoke is unaffected. The audit hash chain stays intact.
2. **No new reads are permitted** under the revoked grants — Permission Check Step 4's grant-resolution rejects revoked grants.
3. **Re-issuance is the recovery path** — the operator must re-adopt the template (which mints a fresh adoption AR + fires fresh grants under it). The previously-revoked grants stay revoked forever; the new grants live under a sibling subtree of the authority chain.

Re-adoption command:

```bash
phi org template adopt --org acme --kind a --reason "rollback of erroneous revoke"
```

The `template.adopted` audit event records the re-adoption; the new adoption AR's id is distinct from the previously-revoked one. Operators investigating "why are the same agents holding new grants?" can correlate via `Organization.audit_class_default` + the `template.adopted` + `template.revoked` audit-event timestamps.

**Idempotency:** the recursive cascade is idempotent over already-revoked grants — re-running it returns an empty `grants_revoked` list (verified by the `revoke_cascade_does_not_re_revoke_already_revoked_grants` acceptance test).

---

## Walker as a debugging tool

The walker is exposed via `Repository::walk_provenance_chain(grant)` in code; there is no CLI command at CH-14. To trace a specific grant's chain back to the bootstrap, operators may:

1. Read the grant id from a `Grant.id` SurrealDB row.
2. Iteratively read `Grant.descends_from -> AuthRequest`, then `AuthRequest.descends_from_grant -> Grant`, etc., until they reach an AR that satisfies `is_bootstrap_ar` (requestor `system:genesis` AND `provenance_template = uuid::Uuid::nil()`).

```surql
-- Step 1: leaf grant -> direct AR id.
SELECT descends_from FROM type::thing('grant', $grant_id);

-- Step 2: AR (e.g., from step 1's `descends_from`) -> requestor + parent.
SELECT requestor_kind, requestor_id, provenance_template, descends_from_grant
  FROM type::thing('auth_request', $ar_id);

-- Repeat until requestor_id == 'system:genesis' AND
-- provenance_template == '00000000-0000-0000-0000-000000000000'.
```

Today, every shipped grant's chain has depth **1** — admin grant directly under the bootstrap AR. Multi-hop chains will appear once adoption-AR-side wiring lands (see drift `D-CH14-FOLLOWUP-01`).

---

## Depth-cap diagnostic

If the walker (or recursive revoke) returns `RepositoryError::ProvenanceCycleDepthExceeded`, this indicates a **schema bug**: the authority tree should never produce a chain deeper than 32 hops in normal operation. Diagnostic steps:

1. Identify the leaf grant id (from the call site logs).
2. Manually walk the chain via SurrealQL (recipe above).
3. Look for a node that points back to itself or to an earlier ancestor — this is the cycle.
4. The cycle indicates either (a) a hand-edited row, (b) a bug in a future adoption-AR-side wiring chunk, or (c) corruption from an interrupted multi-write transaction. Treat as P1 incident; preserve the row data before any remediation.

The depth cap (32) is set at `domain::repository::MAX_PROVENANCE_DEPTH`. Typical chain depth in production is 1–4; the cap exists exclusively as a defensive guard.

---

## Cross-references

- [`authority-chain.md`](../architecture/authority-chain.md) — design page.
- [ADR-0053](../decisions/0053-system-genesis-authority-chain-revocation-cascade.md) — design decisions.
- [`concepts/permissions/08-worked-example.md`](../../../concepts/permissions/08-worked-example.md) §9.3 — forward-only cascade source-of-truth.
