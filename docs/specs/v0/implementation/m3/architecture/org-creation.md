<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Org Creation (page 06)

**Status: [EXISTS]** — shipped at M3/P4.

## End-to-end flow

```
 Wizard (web)         Server                         SurrealDB
 ──────────           ──────                         ─────────
  Step 1..8           POST /api/v0/orgs              1 compound tx
       │                    │                             │
       │  DraftState  ───▶  │                             │
       │                    │  validate                   │
       │                    │  snapshot defaults ─────────┤ (read-only)
       │                    │  build CEO + system agents  │
       │                    │  (clone phi_core::AgentProfile)
       │                    │                             │
       │                    │  apply_org_creation ────────┤ BEGIN TX
       │                    │                             │   CREATE org / agents /
       │                    │                             │   profiles / channel /
       │                    │                             │   inbox / outbox /
       │                    │                             │   grant / token_budget_pool
       │                    │                             │   RELATE edges ×12
       │                    │                             │   CREATE catalogue ×N
       │                    │                             │ COMMIT TX
       │                    │                             │
       │                    │  emit_audit_batch ──────────┤ write audit events
       │                    │    OrganizationCreated      │ (per-org hash chain)
       │                    │    AuthorityTemplateAdopted │   — Alerted, Some(org_id)
       │  receipt JSON ◀─── │                             │
       │                    │                             │
       │  redirect /orgs/:id                              │
```

## Code map

- **Business logic**: [`modules/crates/server/src/platform/orgs/`](../../../../../../modules/crates/server/src/platform/orgs/)
  - `mod.rs` — `OrgError`, `CreatedOrg`.
  - `create.rs` — orchestrator (the *one* new `use phi_core::...`
    import in M3).
  - `list.rs`, `show.rs` — read paths.
- **HTTP handlers**: [`modules/crates/server/src/handlers/orgs.rs`](../../../../../../modules/crates/server/src/handlers/orgs.rs)
- **Router**: [`modules/crates/server/src/router.rs`](../../../../../../modules/crates/server/src/router.rs)
- **Compound tx**:
  [`Repository::apply_org_creation`](../../../../../../modules/crates/domain/src/repository.rs)
  trait method + both impls.
- **CLI**: [`modules/crates/cli/src/commands/org.rs`](../../../../../../modules/crates/cli/src/commands/org.rs)
  + 3 reference-layout fixtures at [`modules/crates/cli/fixtures/reference_layouts/`](../../../../../../modules/crates/cli/fixtures/reference_layouts/).
- **Web wizard**:
  `modules/web/app/(admin)/organizations/` (Next.js route-group
  parentheses confuse relative links; navigate manually) — page 06
  lives at `new/page.tsx` (8-step wizard).

## phi-core leverage (Q1/Q2/Q3)

Per the [leverage-checklist](phi-core-leverage-checklist.md):

**Q1 — direct imports.** One new line across all of P4:
`use phi_core::agents::profile::AgentProfile` in [`create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs)
— used to clone the snapshot baseline and override `name` +
`system_prompt` per system-agent role. The CLI tier and web tier have
zero phi-core imports (phi-core is a Rust crate; the CLI + web use
opaque `Record<string, unknown>` / `serde_json::Value` for phi-core
fields).

**Q2 — transitive payload.**
- `phi_core::AgentProfile` transits via
  `Organization.defaults_snapshot.default_agent_profile` and via
  each system agent's `AgentProfile.blueprint` (×2).
- `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig`
  transit via `OrganizationDefaultsSnapshot` (one copy per org;
  ADR-0023 inherit-from-snapshot).
- `phi_core::ModelConfig` **does not transit** at P4 — the wire
  payload carries `default_model_provider: ModelProviderId` (an id);
  the full `ModelRuntime.config: phi_core::ModelConfig` stays in its
  M2/P6 row and is referenced by id only.

**Q3 — rejections walked in [plan §P4](../../../../plan/build/563945fe-m3-organization-creation.md):**
- `phi_core::Agent` / `BasicAgent` — not applicable (governance
  plane; M5 session launch invokes them).
- `phi_core::Session` / `LoopRecord` / `Turn` — not applicable
  (D11/Q6 defers to M5).
- `phi_core::config::AgentConfig` schema — not applicable
  (governance writes, not external-config parsing).
- `phi_core::tools` / `mcp` / `openapi` / `StreamProvider` — not
  applicable (no runtime invocation).

## ADR-0023 invariant (enforcement)

System agents inherit execution context from
`Organization.defaults_snapshot`; no per-agent `ExecutionLimits` /
`RetryPolicy` / `CachePolicy` / `CompactionPolicy` nodes are created.
Verified by:

1. [`apply_org_creation_tx_test::adr_0023_invariant_no_per_agent_policy_nodes_materialised`](../../../../../../modules/crates/store/tests/apply_org_creation_tx_test.rs)
   — asserts 0 rows on each of the 4 per-agent policy tables
   post-tx.
2. [`acceptance_orgs_create::create_respects_adr_0023_no_per_agent_policy_nodes`](../../../../../../modules/crates/server/tests/acceptance_orgs_create.rs)
   — same invariant through the HTTP path.

## Template node persistence deferral

Template graph nodes (`Template { id, name, kind, created_at }`)
are **not** persisted at M3/P4 — the `template.name` UNIQUE INDEX
would collide when two orgs adopt the same kind. The adoption AR's
resource URI (`org:<id>/template:<kind>`) carries the kind info the
dashboard needs. Template nodes land in M5 when trigger-fire requires
a shared pattern node; at that point we either rename to
`template:<kind>:org:<id>` or convert the UNIQUE INDEX to `(kind)`
with one shared row per kind.

## References

- [M3 plan §D6 / §D7 / §P4](../../../../plan/build/563945fe-m3-organization-creation.md)
- [ADR-0022 — compound transaction](../decisions/0022-org-creation-compound-transaction.md)
- [ADR-0023 — inherit-from-snapshot](../decisions/0023-system-agents-inherit-from-org-snapshot.md)
- [Per-org audit chain architecture](per-org-audit-chain.md)
- [Requirements: admin page 06](../../../requirements/admin/06-org-creation-wizard.md)
