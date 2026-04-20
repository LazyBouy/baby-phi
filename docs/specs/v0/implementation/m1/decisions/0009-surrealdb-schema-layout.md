<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0009: SurrealDB schema layout — one SCHEMAFULL table per node, typed RELATION per edge

## Status

Accepted — 2026-04-20 (M1 / P1).

## Context

The v0 ontology defines 37 node types and 66 edge types
([graph-model.md](../architecture/graph-model.md)). SurrealDB 2.x offers
several ways to model a graph: fully schemaless record collections;
SCHEMAFULL tables with explicit `DEFINE FIELD`s; typed `RELATION` edges
with `FROM`/`TO` endpoints; or a mix. The decision affects: how much
validation the DB enforces vs the app layer; how easy schema migrations
are; and how queries read.

## Decision

1. **One SCHEMAFULL table per node kind.** Every variant of
   `NodeKind` (37 total) maps to a `DEFINE TABLE <name> SCHEMAFULL;` with
   at minimum a `created_at: datetime` field. Load-bearing M1 nodes
   (Agent, AgentProfile, Grant, AuthRequest, Template, Consent, User,
   Organization, Channel, InboxObject, OutboxObject, Memory,
   ToolAuthorityManifest) declare their full field shape; scaffolded
   nodes declare only `created_at` and later milestones add fields.
2. **One typed `TYPE RELATION` edge per `Edge` variant.** Every variant
   of the 66-item `Edge` enum maps to a `DEFINE TABLE <name> TYPE RELATION
   FROM <src> TO <dst>;`. Where one concept-doc edge *name* covers
   multiple source/target pairs (HOLDS_GRANT, CONNECTS_TO, PROVIDES_TOOL,
   OWNED_BY, SUBMITTED_BY), each pair gets its own typed relation; this
   is the same reason the Rust enum has 66 variants.
3. **Three relations use the type-union escape hatch.** `owned_by`,
   `created`, `allocated_to` accept any Resource → Principal combination,
   so they are declared `DEFINE TABLE ... TYPE RELATION` without FROM/TO —
   the domain layer enforces the union constraint in Rust.
4. **Utility tables** (`_migrations`, `bootstrap_credentials`,
   `secrets_vault`, `audit_events`, `resources_catalogue`) are all
   SCHEMAFULL with strict field types and targeted indexes.
5. **Indexes on query-critical fields only.** Every node table has SurrealDB's
   implicit `id` index. Beyond that: `agent.owning_org`, `grant.holder_*`,
   `grant.resource_uri`, `auth_request.state`, `audit_events (org_scope,
   timestamp)` composite (for hash-chain walks),
   `resources_catalogue (owning_org, resource_uri)` UNIQUE.

## Consequences

Positive:

- **DB-side validation** catches schema drift early. The `auth_request.state`
  ASSERT and `audit_events.audit_class` ASSERT reject out-of-domain values
  without requiring the app to enforce them.
- **Relations are traversable with SurrealQL's `->`/`<-` syntax** —
  Permission Check Step 2 (authority chain traversal) becomes a single
  query in M2+ rather than N round-trips.
- **Scaffolded tables exist from day one.** Later milestones add
  `DEFINE FIELD`s rather than `DEFINE TABLE`s, which is a shallower
  migration delta.

Negative:

- **More DDL up front.** `0001_initial.surql` is ~260 lines. Manageable,
  but reviewers need to read carefully.
- **Three edges lose typed endpoints.** `owned_by`/`created`/`allocated_to`
  accept anything at the DB layer; mistakes are instead caught at
  compile time in Rust via the sealed `Principal`/`Resource` marker
  traits + typed `Edge::new_*` constructors + typed repository helpers
  added in P2. **See [ADR-0015](0015-type-safe-ownership-edges.md) for
  the full compile-time mitigation.** Without that mitigation this
  trade-off would be MEDIUM severity by M3; with it, the class of bug
  is eliminated for every Rust caller (and covered by ≥6 `trybuild`
  compile-fail fixtures).
- **Schema changes require new migration files.** We can't just tweak
  fields in the initial migration — anything beyond P1 lands as
  `0002_...`, `0003_...`, etc. (See [ADR-0012](0012-forward-only-migrations.md).)

## Alternatives considered

- **Fully SCHEMALESS tables.** Rejected: trades DB-side validation for
  app-side, and makes migrations harder (no schema to migrate, just
  conventions).
- **One polymorphic `node` table with a `kind` discriminator.** Rejected:
  indexes would need `WHERE kind = ...` prefixes, and `DEFINE FIELD`
  can't be conditioned on discriminator values — fields that apply to
  only a subset of kinds would become `option<...>` for every row.
- **Skip typed RELATIONs; use a single `edge` table with `from`/`to`/`name`
  columns.** Rejected: loses endpoint-type enforcement, makes queries
  depend on string matches.
- **Defer SCHEMAFULL to M2+ (start SCHEMALESS).** Rejected: we're about to
  write the Permission Check engine in M1/P3 — the engine *needs* the
  fields it reads to exist reliably. Putting discipline in from day one
  is cheaper than retrofitting.

## References

- Schema: [`modules/crates/store/migrations/0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql)
- Type inventory: [graph-model.md](../architecture/graph-model.md)
- Migration runner: [ADR-0012](0012-forward-only-migrations.md)
