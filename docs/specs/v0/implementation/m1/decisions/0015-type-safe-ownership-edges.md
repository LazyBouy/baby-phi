<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0015: Type-safe ownership edges via sealed marker traits + typed repository helpers

## Status

Accepted — 2026-04-20 (M1 / P2). Added as part of the plan's r2 revision
after the P1 self-review surfaced Risk 1 ("untyped RELATIONs can accept
nonsense endpoint pairs").

## Context

The v0 ontology defines three ownership edges (`owned_by`, `created`,
`allocated_to`) whose endpoints are type unions:

- **Resource** — any node that can be owned: filesystem objects, memories,
  sessions, agents (dual-role), etc. In v0 effectively every node kind
  can be a Resource.
- **Principal** — any entity that can hold authority: Agent, User,
  Organization, Project. The `system:` axioms are represented as
  string-keyed `PrincipalRef::System(String)`, not an ID newtype.

ADR-0009 chose to represent these three edges as single SurrealDB
`DEFINE TABLE ... TYPE RELATION` tables (without FROM/TO type constraints)
because the alternative — partitioning into concrete-pair tables — would
balloon to roughly 4 × 37 = ~150 edge tables for `owned_by` alone.

That choice, however, means **SurrealDB will accept nonsense pairs**
(e.g. `AuditEvent -owned_by→ BootstrapCredential`) at the schema layer.
The Rust `Edge` enum also lost type safety here because the three variants
had to be uniformly typed as `NodeId → NodeId`. Risk 1 from the P1
self-review:

> Currently LOW; grows to MEDIUM once more code paths create these edges.
> A corrupted authority chain makes Permission Check Step 2 return wrong
> decisions. Blast radius: authorization granted where it shouldn't be.

P2 owes a mitigation.

## Decision

Put the type enforcement **in Rust, compile-time, via two coordinated
layers** — keep the schema untouched.

### Layer 1 — sealed marker traits on ID newtypes

A new module [`domain::model::principal_resource`](../../../../../../modules/crates/domain/src/model/principal_resource.rs)
defines two marker traits:

```rust
pub trait Principal: sealed::Sealed { fn node_id(&self) -> NodeId; }
pub trait Resource:  sealed::Sealed { fn node_id(&self) -> NodeId; }
```

Both extend a crate-private `sealed::Sealed` supertrait, so downstream
crates cannot add rogue impls. The module-local `seal!($ty)` macro
declares `impl Sealed for $ty` for every participating newtype.

Impls match the concept doc's type unions:

- `Principal` on `AgentId`, `UserId`, `OrgId`, `ProjectId`.
- `Resource` on `NodeId` (generic fallback) + `AgentId` (dual-role) +
  every concrete ownable node ID (Session, Memory, AuthRequest, Grant,
  Template, Consent, …).

Absence is as important as presence: `NodeId`, `GrantId`, `TemplateId`,
`ConsentId` are deliberately NOT `Principal` — the type system rejects
them if they ever appear where a principal is expected.

### Layer 2 — typed constructors + typed repository helpers

`Edge` gains three typed constructors:

```rust
impl Edge {
    pub fn new_owned_by<R: Resource, P: Principal>(resource: &R, principal: &P) -> Edge;
    pub fn new_created<P: Principal, R: Resource>(creator: &P, resource: &R) -> Edge;
    pub fn new_allocated_to<P1: Principal, P2: Principal>(from: &P1, to: &P2) -> Edge;
}
```

The `Repository` trait gets **raw** methods that take `NodeId` (so it
stays object-safe), and the `repository` module exports **free functions**
that wrap them with the marker-trait bounds:

```rust
pub async fn upsert_ownership<R: Resource + ?Sized, P: Principal + ?Sized>(
    repo: &(dyn Repository + '_),
    resource: &R,
    principal: &P,
    auth_request: Option<AuthRequestId>,
) -> RepositoryResult<EdgeId> {
    repo.upsert_ownership_raw(resource.node_id(), principal.node_id(), auth_request).await
}
```

Callers use the typed free functions; mistakes like passing a
`ConsentId` into a Principal slot fail to compile.

### Layer 3 — `trybuild` compile-fail fixtures

Six `compile_fail` programs under
[`modules/crates/domain/tests/edge_type_safety/compile_fail/`](../../../../../../modules/crates/domain/tests/edge_type_safety/compile_fail/)
prove the wrong-pair cases don't build:

| Fixture | Rejects |
|---|---|
| `owned_by_rejects_consent_as_principal.rs` | `ConsentId` in Principal slot |
| `owned_by_rejects_node_id_as_principal.rs` | `NodeId` in Principal slot |
| `owned_by_rejects_user_as_resource.rs` | `UserId` in Resource slot |
| `created_rejects_grant_as_principal.rs` | `GrantId` in Principal slot |
| `created_rejects_org_as_resource.rs` | `OrgId` in Resource slot |
| `allocated_to_rejects_memory_as_principal.rs` | `MemoryId` in either Principal slot |

The harness at `domain/tests/edge_type_safety.rs` runs `trybuild` and
also snapshots the exact compiler-error text. If Rust's error wording
changes, `TRYBUILD=overwrite cargo test -p domain --test edge_type_safety`
regenerates the snapshots and a reviewer commits the diff.

## Consequences

Positive:

- **Risk 1 closed for Rust callers.** The class of bug — pasting the
  wrong ID type into an ownership edge — is now a compile error. The
  self-review's "LOW today, MEDIUM by M3" projection is eliminated.
- **Schema untouched.** No ~150-table blowup, no DDL migration churn.
- **Zero runtime cost.** The marker traits are compiled away; the typed
  wrappers compile to the same calls the raw methods would make.
- **Escape hatch preserved.** The raw `upsert_*_raw` methods are still
  public — bulk importers or future code paths that genuinely need
  dynamic dispatch can use them, accepting the loss of compile-time
  safety with eyes open.
- **`trybuild` regressions are a hard CI gate.** If anyone ever adds an
  impl that relaxes the type discipline (e.g. `impl Principal for
  ConsentId`), the `compile_fail` fixtures stop failing and CI breaks.

Negative:

- **The `node_id()` forwarding is boilerplate.** Every Principal/Resource
  impl is a one-liner that wraps the inner UUID — 13 impls today. Not
  painful, but worth noting.
- **`trybuild` snapshots are compiler-version-dependent.** A Rust upgrade
  that changes error wording requires a one-time snapshot refresh.
  Documented in the harness docstring.
- **Non-Rust callers lose the safety.** Only affects theoretical future
  non-Rust adapters (none planned). Domain types exported via the HTTP
  API already serialize to JSON without crossing a Principal/Resource
  boundary, so this is currently moot.

## Alternatives considered

- **DB-side partitioning (~150 concrete-pair tables).** Rejected: the
  schema bloat is real, migrations become per-pair, and Permission Check
  queries would need to UNION across the partitions. Bad trade.
- **Runtime type checks in the repository.** Rejected: reports the
  mistake at request time instead of compile time — exactly the class of
  bug Risk 1 predicts. Compile-time enforcement is the right place.
- **Generic methods on `Repository` directly.** Rejected: breaks
  `Arc<dyn Repository>` object-safety, which the M0 `AppState` pattern
  depends on.
- **An extension trait (`RepositoryExt`) with generic methods.** Works
  but needs explicit `Pin<Box<dyn Future>>` return types to stay
  dyn-compatible, which turns into noise at every call site.
  Free-function wrappers give the same ergonomics with less ceremony.
- **Skip Risk 1 mitigation entirely.** Rejected per "thoroughness over
  speed" — the fix is a one-day effort in P2, and the class of bug
  compounds as M2+ adds more edge-creation paths.

## References

- Implementation: [`modules/crates/domain/src/model/principal_resource.rs`](../../../../../../modules/crates/domain/src/model/principal_resource.rs), [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) (`new_*` constructors), [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) (free-function wrappers).
- Compile-fail fixtures: [`modules/crates/domain/tests/edge_type_safety/compile_fail/`](../../../../../../modules/crates/domain/tests/edge_type_safety/compile_fail/).
- Architecture page: [storage-and-repository.md](../architecture/storage-and-repository.md).
- Cross-ref: [ADR-0009 schema layout](0009-surrealdb-schema-layout.md) amended in P2 to point at this ADR as the compile-time mitigation for the three untyped RELATIONs.
- Risk source: P1 self-review (risk 1), plan r2 revision (§Commitment-ledger row C15, §Decisions row D9).
