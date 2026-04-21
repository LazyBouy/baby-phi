<!-- Last verified: 2026-04-21 by Claude Code -->

# baby-phi — M2 implementation documentation

M2 is the **Platform Setup** milestone. It lands admin pages **02–05**:

- Page 02 — Model Providers (bound LLM runtimes + tenant-scoped access).
- Page 03 — MCP Servers (external-service registry + cascading tenant
  revocation).
- Page 04 — Credentials Vault (at-rest encrypted secrets + reveal audit).
- Page 05 — Platform Defaults (non-retroactive platform-wide baselines).

It is also the first milestone where handlers actually call the
Permission Check engine (M1's bootstrap bypassed it) and emit audit
events through a real `AuditEmitter`. It introduces the **Template E**
("self-interested auto-approve") pattern every M2 write uses and the
reusable `handler_support` shim that every M3+ handler will build on.

This page is the index. The archived plan lives at
[`../../../plan/build/a6005e06-m2-platform-setup.md`](../../../plan/build/a6005e06-m2-platform-setup.md);
the v0.1 build plan it sits under is at
[`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../plan/build/36d0c6c5-build-plan-v01.md).

## Status

M2 is delivered phase-by-phase. This index is updated as each phase lands.

| Phase | Status |
|---|---|
| P0 Residual phi-core-reuse cleanup (M1 → M2 handoff) | ✓ done |
| P1 Foundation (IDs + `composites_m2` + migration 0002 + docs tree + web shell + CLI scaffolding + phi-core-reuse lint) | ✓ done |
| P2 Repository expansion + Template E + constraint value-match | ✓ done |
| P3 `handler_support` + `SurrealAuditEmitter` + `spawn_claimed` fixture | ✓ done |
| P4 Page 04 vertical (Credentials Vault) | ✓ done |
| P4.5 Grant instance-URI fundamentals detour (G19 / D17 / C21) | ✓ done |
| P5 Page 02 vertical (Model Providers) | ✓ done |
| P6 Page 03 vertical (MCP Servers + cascading revocation) | ✓ done |
| P7 Page 05 vertical (Platform Defaults) | ✓ done |
| P8 Seal (CLI completion + cross-page acceptance + CI + runbook + re-audit) | ✓ done |

## Layout

```
m2/
├── architecture/     "how M2 is built"
├── user-guide/       "how an admin / developer uses M2 surfaces"
├── operations/       "how to deploy, monitor, and secure M2"
└── decisions/        ADRs — load-bearing choices and their rationale
```

## architecture/

Pages are live as their phase lands; the rest carry a
`[PLANNED M2/P<n>]` tag pointing at the phase that fleshes them out.

| Page | Purpose |
|---|---|
| [overview.md](architecture/overview.md) | M2 system map — what each phase adds on top of the M1 spine |
| [phi-core-reuse-map.md](architecture/phi-core-reuse-map.md) | Durable publication of the M2 plan's §1.5 — which phi-core types M2 imports or wraps |
| [platform-catalogue.md](architecture/platform-catalogue.md) | How M2 writes seed the `resources_catalogue` so Step 0 of Permission Check resolves |
| [template-e-auto-approve.md](architecture/template-e-auto-approve.md) | The "self-interested auto-approve" pattern used by every M2 write |
| [vault-encryption.md](architecture/vault-encryption.md) | Per-entry seal / reveal flow for page 04 |
| [cascading-revocation.md](architecture/cascading-revocation.md) | MCP tenant-narrowing semantics (page 03) |
| [handler-support.md](architecture/handler-support.md) | `AuthenticatedSession` + `check_permission` + `emit_audit` shim |
| [platform-defaults.md](architecture/platform-defaults.md) | Singleton row + non-retroactive inheritance invariant |
| [server-topology.md](architecture/server-topology.md) | Extends M1's route table with `/api/v0/platform/*` |

## user-guide/

| Page | Purpose |
|---|---|
| [platform-setup-walkthrough.md](user-guide/platform-setup-walkthrough.md) | End-to-end 4-page tour for a fresh platform admin |
| [model-providers-usage.md](user-guide/model-providers-usage.md) | Registering / archiving LLM runtimes |
| [mcp-servers-usage.md](user-guide/mcp-servers-usage.md) | Registering / patching / archiving MCP servers |
| [secrets-vault-usage.md](user-guide/secrets-vault-usage.md) | Add / rotate / reveal / reassign custody |
| [platform-defaults-usage.md](user-guide/platform-defaults-usage.md) | Editing + importing / exporting the singleton |
| [cli-reference-m2.md](user-guide/cli-reference-m2.md) | New subcommands + `baby-phi completion <shell>` |

## operations/

| Page | Purpose |
|---|---|
| [model-provider-operations.md](operations/model-provider-operations.md) | Adding a new provider, health-probe incidents, archival |
| [mcp-server-operations.md](operations/mcp-server-operations.md) | Tenant-narrowing playbook, cascade audit trail |
| [secrets-vault-operations.md](operations/secrets-vault-operations.md) | Rotation policy, reveal audit, master-key rotation stub |
| [platform-defaults-operations.md](operations/platform-defaults-operations.md) | Non-retroactive rule, YAML import/export safety, factory reset |
| [m2-additions-to-m1-ops.md](operations/m2-additions-to-m1-ops.md) | Deltas to schema-migrations / at-rest / audit-retention ops pages |

## decisions/

Each ADR follows the Status / Context / Decision / Consequences /
Alternatives pattern. Numbering continues from M1 (0008–0015).

| # | Decision | Status |
|---|---|---|
| [0016](decisions/0016-template-e-self-interested-auto-approve.md) | Template E ("self-interested auto-approve") as the canonical M2 admin-write pattern | Accepted |
| [0017](decisions/0017-vault-encryption-envelope.md) | Per-entry seal-with-fresh-nonce + one master key for M2's vault | Accepted |
| [0018](decisions/0018-handler-support-module.md) | `handler_support` as a first-class shim (extractor + Permission Check wrapper + audit emitter) | Accepted |
| [0019](decisions/0019-platform-defaults-non-retroactive.md) | Platform defaults are non-retroactive — edits never mutate existing orgs | Accepted |

## Conventions

Same as M1:

- Every page carries a `<!-- Last verified: YYYY-MM-DD by Claude Code -->`
  header on line 1.
- Feature references are status-tagged: `[EXISTS]`, `[PLANNED M<n>/P<n>]`,
  `[CONCEPTUAL]`.
- Code claims link to file + line, relative to the docs root (so
  `../../../../../../modules/…` from `m2/{architecture,user-guide,operations,decisions}/`).
- Rationale links to the archived plan or a concept doc rather than
  restating.
- ASCII diagrams only.
- Docs for a phase land in the same commit as that phase's code.
