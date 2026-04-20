<!-- Last verified: 2026-04-20 by Claude Code -->

# baby-phi — M1 implementation documentation

M1 is the **Permission Check spine** milestone. It lands:

- The full graph model from the v0 ontology: 9 fundamentals, 8 composites,
  37 node types, 66 edge types.
- The `Repository` trait expansion and its SurrealDB (RocksDB) implementation.
- A forward-only migration runner with a startup-gate fail-safe.
- At-rest envelope encryption (AES-GCM) for the secrets vault.
- The audit-event framework (base schema + class tiers + hash-chain seed).
- The Permission Check 6-step engine with property tests.
- The Auth Request 9-state machine with property tests.
- The System Bootstrap (s01) flow.
- `GET /api/v0/bootstrap/status` + `POST /api/v0/bootstrap/claim` HTTP
  endpoints with a signed session cookie.
- The CLI `baby-phi bootstrap {status,claim}` subcommands (CLI migrated to
  layered config).
- The `/bootstrap` SSR page in the Next.js web app.
- Acceptance tests and the M1 extensions to the CI workflows.

This page is the index. The archived plan lives at
[`../../../plan/build/015a217a-m1-permission-check-spine.md`](../../../plan/build/015a217a-m1-permission-check-spine.md);
the v0.1 build plan it sits under is at
[`../../../plan/build/36d0c6c5-build-plan-v01.md`](../../../plan/build/36d0c6c5-build-plan-v01.md).

## Status at the start of writing (P1 landed)

M1 is delivered phase-by-phase. This index is updated as each phase lands.

| Phase | Status |
|---|---|
| P1 Foundation (graph model, schema, migrations, audit, crypto) | ✓ done |
| P2 Repository trait expansion + type-safe ownership edges (ADR-0015) | ✓ done |
| P3 Permission Check engine | ✓ done |
| P4 Auth Request state machine | ✓ done |
| P5 System Bootstrap flow | ✓ done |
| P6 HTTP endpoints + session cookie | ✓ done |
| P7 CLI subcommands | ✓ done |
| P8 Web `/bootstrap` page | ✓ done |
| P9 Acceptance harness + final seal | ✓ done |

## Layout

```
m1/
├── architecture/     "how M1 is built"
├── user-guide/       "how to run / develop on M1"
├── operations/       "how to deploy, monitor, and secure M1"
└── decisions/        ADRs — load-bearing choices and their rationale
```

## architecture/

Pages with live links exist now; the rest are `[PLANNED M1/P<n>]` and will
be linked once the corresponding phase lands.

| Page | Purpose |
|---|---|
| [overview.md](architecture/overview.md) | M1 system map: what P1 added to the M0 skeleton + what's planned in P2–P9 |
| [graph-model.md](architecture/graph-model.md) | 9+8+37+66 with type ↔ concept refs |
| [schema-migrations.md](architecture/schema-migrations.md) | Forward-only runner, embedded migration list, startup gate |
| [audit-events.md](architecture/audit-events.md) | Base shape, classes, hash-chain seed |
| [at-rest-encryption.md](architecture/at-rest-encryption.md) | Envelope encryption for the secrets vault |
| [permission-check-engine.md](architecture/permission-check-engine.md) | 6-step (+2a) pipeline with ASCII diagram, module layout, metric wiring, proptest coverage |
| [auth-request-state-machine.md](architecture/auth-request-state-machine.md) | 9-state lifecycle + aggregation tables + transition API + retention window + proptest coverage |
| [bootstrap-flow.md](architecture/bootstrap-flow.md) | s01 atomic adoption flow + entity shape + rollback contract |
| [server-topology.md](architecture/server-topology.md) | Extends the M0 route table with `/api/v0/bootstrap/*` + the signed-cookie layer + the `baby_phi_bootstrap_claims_total` counter |
| [web-topology.md](architecture/web-topology.md) | Extends the M0 web map with the `/bootstrap` SSR page + Server Action + session-cookie plumbing |
| [storage-and-repository.md](architecture/storage-and-repository.md) | M0 extension — 36-method Repository surface + typed ownership-edge helpers |

## user-guide/

| Page | Purpose |
|---|---|
| [first-bootstrap.md](user-guide/first-bootstrap.md) | End-to-end walkthrough — install command, credential delivery, claim flow, failure cases |
| [cli-usage.md](user-guide/cli-usage.md) | `baby-phi bootstrap {status,claim}` + `agent demo` reference |
| [web-usage.md](user-guide/web-usage.md) | `/bootstrap` page walkthrough with error-case table |
| [http-api-reference.md](user-guide/http-api-reference.md) | `/api/v0/bootstrap/*` request + response contract |
| [troubleshooting.md](user-guide/troubleshooting.md) | M1 error codes + exit-code ladder + recovery paths |

## operations/

| Page | Purpose |
|---|---|
| [schema-migrations-operations.md](operations/schema-migrations-operations.md) | Applying migrations, audit, broken-migration recovery |
| [at-rest-encryption-operations.md](operations/at-rest-encryption-operations.md) | Master-key handling, backup guidance, rotation stub (full at M7b) |
| [bootstrap-credential-lifecycle.md](operations/bootstrap-credential-lifecycle.md) | Generation → delivery → consumption, recovery paths |
| [audit-log-retention.md](operations/audit-log-retention.md) | Class-tier retention, hash-chain verification, M7b deferrals |

## decisions/

Each ADR follows the Status / Context / Decision / Consequences / Alternatives
pattern. Numbering continues from M0 (which holds 0001–0007).

| # | Decision | Status |
|---|---|---|
| [0008](decisions/0008-permission-check-as-pipeline.md) | Permission Check as an eight-stage typed pipeline | Accepted — 2026-04-20 (P3) |
| [0009](decisions/0009-surrealdb-schema-layout.md) | SurrealDB schema layout: one SCHEMAFULL table per node + typed RELATION per edge | Accepted — 2026-04-20 |
| [0010](decisions/0010-per-slot-aggregation.md) | Per-slot aggregation for Auth Requests | Accepted — 2026-04-20 (P4) |
| [0011](decisions/0011-bootstrap-credential-single-use.md) | Bootstrap credential: argon2id-hashed, stdout-delivered, single-use | Accepted — 2026-04-20 (P5) |
| [0012](decisions/0012-forward-only-migrations.md) | Forward-only embedded migrations with startup-gate fail-safe | Accepted — 2026-04-20 |
| [0013](decisions/0013-audit-events-class-and-chain.md) | Audit events: class tiers + per-org hash-chain | Accepted — 2026-04-20 |
| [0014](decisions/0014-at-rest-encryption-envelope.md) | At-rest encryption: AES-GCM envelope for the secrets vault | Accepted — 2026-04-20 |
| [0015](decisions/0015-type-safe-ownership-edges.md) | Type-safe ownership edges — sealed `Principal`/`Resource` marker traits + typed `Edge::new_*` constructors + typed repository helpers, closing Risk 1 from the P1 self-review | Accepted — 2026-04-20 (P2) |

## Conventions

- Every page carries a `<!-- Last verified: YYYY-MM-DD by Claude Code -->`
  header on line 1.
- Feature references are status-tagged: `[EXISTS]`, `[PLANNED M<n>]`, or
  `[CONCEPTUAL]`.
- Code claims link to file + line (e.g.
  [`modules/crates/domain/src/model/fundamentals.rs`](../../../../../modules/crates/domain/src/model/fundamentals.rs)),
  so docs stay discoverable as code evolves.
- Rationale-heavy claims link to the archived plan or to a concept doc
  rather than restating them.
- Diagrams are ASCII — diff-able, dependency-free.
- Docs for a phase land in the same commit as that phase's code (same rule
  as M0).
