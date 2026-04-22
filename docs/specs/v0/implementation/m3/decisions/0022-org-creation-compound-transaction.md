<!-- Last verified: 2026-04-22 by Claude Code -->

# ADR-0022 — Org Creation compound transaction

**Status: [ACCEPTED M3/P4]** — shipped at P4. Invariant re-verified
by `modules/crates/server/tests/acceptance_orgs_create.rs` (13
scenarios) + `modules/crates/store/tests/apply_org_creation_tx_test.rs`
(4 scenarios including ADR-0023 zero-rows assertion).

## Context

R-ADMIN-06-W4 requires org creation to be **atomic** — a partial
failure mid-provisioning must leave zero persisted state, and the
operator may retry with a fresh request. The wizard builds a payload
spanning ~10 node types + 12 edges:

1. `Organization` (1)
2. CEO `Agent` (1; kind Human, owning_org = new org)
3. CEO `Channel` (1)
4. CEO `InboxObject` + `OutboxObject` (2)
5. CEO `Grant` (1; `[allocate]` on `org:<id>` with fundamentals
   `{IdentityPrincipal, Tag}`)
6. Two system `Agent`s (kind Llm, owning_org = new org)
7. Two `AgentProfile` nodes, each wrapping a
   `phi_core::agents::profile::AgentProfile` blueprint
8. `TokenBudgetPool` (1)
9. N × adoption `AuthRequest` (one per enabled Template in A/B/C/D)
10. Edge set: `HasCeo`, `HasMember`×3, `MemberOf`×3, `HasInbox`,
    `HasOutbox`, `HasChannel`, `HasProfile`×2 (12 edges)
11. Catalogue seeds: the org's control-plane URI + per-adoption
    template URIs

Running these 20+ writes as separate repository calls would leave
orphan nodes on any mid-flight error — breaking compliance invariants
and the wizard UX.

## Decision

The `Repository::apply_org_creation(payload: OrgCreationPayload) ->
RepositoryResult<OrgCreationReceipt>` trait method commits every write
in **one atomic transaction**. The SurrealDB impl wraps the sequence
in `BEGIN TRANSACTION;` … `COMMIT TRANSACTION;`. The in-memory fake
validates up-front and then applies under a single write-lock
(matches the `apply_bootstrap_claim` precedent from M1).

**Audit events are composed and emitted OUTSIDE the tx** via
`handler_support::audit::emit_audit_batch`. A successful commit is
durable before the first audit event is written; audit-emit failure
after commit surfaces as a 500 `AUDIT_EMIT_FAILED` but does not
attempt to rewind the repository write. This is the same discipline
M1/M2 pages use (audit events are write-once + durably queued;
chain-repair happens out-of-band if a persisted row ends up without a
chain entry).

## Consequences

**Positive:**
- R-ADMIN-06-W4 satisfied: partial-failure scenarios leave zero
  orphan state (verified by the rollback test at
  `apply_org_creation_tx_test::duplicate_org_id_is_conflict_with_no_partial_state`).
- Complements ADR-0023's inherit-from-snapshot pattern: the compound
  tx is already large; adding per-agent `ExecutionLimits` /
  `RetryPolicy` / `CompactionPolicy` / `CachePolicy` nodes would make
  it larger and harder to audit. Keeping those in the snapshot keeps
  the tx bounded.
- Keeps the repo surface narrow: callers bundle inputs; repo is a
  single entry point; wizard orchestration is in
  `server::platform::orgs::create`.

**Negative:**
- The SurrealQL `BEGIN … COMMIT` envelope is long (~50 lines of
  statements + per-edge `LET` bindings). `RELATE type::thing(...) ->
  edge -> type::thing(...)` isn't accepted directly in SurrealDB 2.x,
  so each edge binds its `FROM` + `TO` through intermediate `LET`s.
  The pattern is the same as `upsert_ownership_raw`'s — not a new
  cost.
- Retries on repository-level transient errors aren't automatic; the
  wizard UI must surface the 500 and let the operator re-submit (the
  payload is idempotent except for the server-minted ids — each
  re-submit is a fresh org).

**Neutral:**
- The Template graph node is NOT persisted at M3/P4 (see
  `modules/crates/server/src/platform/orgs/create.rs:6.` comment):
  the UNIQUE INDEX on `template.name` would collide when two orgs
  adopt the same kind. Adoption ARs carry the template info via their
  resource URI prefix (`org:<id>/template:<kind>`); the dashboard
  resolves kind from the AR directly. Template-node persistence
  lands in M5 when trigger-fire actually needs a shared pattern node.

## phi-core leverage

Per [leverage-checklist](../architecture/phi-core-leverage-checklist.md)
Q1/Q2/Q3, the compound tx's phi-core exposure is:

- **Q1 direct**: `server/src/platform/orgs/create.rs` adds **one**
  `use phi_core::agents::profile::AgentProfile` — the per-system-agent
  blueprint clone.
- **Q2 transitive**: `OrgCreationPayload.organization.defaults_snapshot`
  wraps `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig` /
  `AgentProfile`; each system agent's `AgentProfile.blueprint` wraps
  `phi_core::AgentProfile`. All transit through SurrealDB
  `FLEXIBLE TYPE object` columns unchanged.
- **Q3**: ADR-0023 pins the inherit-from-snapshot decision; no
  per-agent `ExecutionLimits` / `RetryPolicy` / `CachePolicy` /
  `CompactionPolicy` nodes are created. Integration test asserts
  0 rows on those tables post-creation.

## References

- [M3 plan §D6 / §P3 / §P4](../../../../plan/build/563945fe-m3-organization-creation.md)
- [ADR-0023 — inherit-from-snapshot](0023-system-agents-inherit-from-org-snapshot.md)
- [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) — `OrgCreationPayload` + `OrgCreationReceipt` + trait method.
- [`modules/crates/store/src/repo_impl.rs::apply_org_creation`](../../../../../../modules/crates/store/src/repo_impl.rs) — SurrealQL body.
- [`modules/crates/server/src/platform/orgs/create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) — orchestrator.
