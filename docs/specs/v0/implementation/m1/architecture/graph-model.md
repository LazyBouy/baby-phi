<!-- Last verified: 2026-04-20 by Claude Code -->

# Architecture — graph model

M1 lands the full v0 ontology as Rust types. Source of truth is
[`docs/specs/v0/concepts/ontology.md`](../../../concepts/ontology.md) +
[`docs/specs/v0/concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md).

Counts (all asserted by unit tests in
[`modules/crates/domain/src/model/mod.rs`](../../../../../../modules/crates/domain/src/model/mod.rs)):

| Piece | Count | Rust home |
|---|---|---|
| Fundamentals | **9** | [`model::fundamentals::Fundamental`](../../../../../../modules/crates/domain/src/model/fundamentals.rs) |
| Composites | **8** | [`model::composites::Composite`](../../../../../../modules/crates/domain/src/model/composites.rs) |
| Node kinds | **37** | [`model::nodes::NodeKind`](../../../../../../modules/crates/domain/src/model/nodes.rs) |
| Edge kinds | **66** | [`model::edges::EDGE_KIND_NAMES`](../../../../../../modules/crates/domain/src/model/edges.rs) |
| Auth Request states | **9** | [`model::nodes::AuthRequestState::ALL`](../../../../../../modules/crates/domain/src/model/nodes.rs) |

The pre-M1 build-plan headline "31 nodes + 56+ edges" was a stale
approximation. The concept doc has been updated to the accurate 37 + 66
counts as part of P1.

## How the types are organised

```
modules/crates/domain/src/model/
├── mod.rs            public re-exports + 9/8/37/66 count tests
├── ids.rs            13 strongly-typed Uuid newtypes
├── fundamentals.rs   Fundamental enum (9 variants) + ALL[] + as_str()
├── composites.rs     Composite enum (8 variants) + constituents() + kind_tag()
├── nodes.rs          NodeKind enum (37 variants) + 37 struct types
└── edges.rs          Edge tagged enum (66 variants) + EDGE_KIND_NAMES[]
```

### IDs ([`ids.rs`](../../../../../../modules/crates/domain/src/model/ids.rs))

Every entity gets its own `Uuid` newtype so the compiler catches accidental
crosses. The `id_newtype!` macro generates `new()`, `from_uuid()`,
`as_uuid()`, `Default`, and `Display` for each. IDs serialize as plain UUID
strings (`#[serde(transparent)]`), so wire format and DB storage stay
flat.

13 newtypes: `NodeId`, `EdgeId`, `OrgId`, `AgentId`, `UserId`, `ProjectId`,
`GrantId`, `AuthRequestId`, `TemplateId`, `ConsentId`, `SessionId`,
`MemoryId`, `AuditEventId`.

### Fundamentals ([`fundamentals.rs`](../../../../../../modules/crates/domain/src/model/fundamentals.rs))

9 variants, grouped by the concept doc's flavors (physical/operational, data
access, identity):

| Variant | String form | Concept source |
|---|---|---|
| `FilesystemObject` | `filesystem_object` | physical/operational |
| `ProcessExecObject` | `process_exec_object` | physical/operational |
| `NetworkEndpoint` | `network_endpoint` | physical/operational |
| `SecretCredential` | `secret_credential` | physical/operational |
| `EconomicResource` | `economic_resource` | physical/operational |
| `TimeComputeResource` | `time_compute_resource` | physical/operational |
| `DataObject` | `data_object` | data access |
| `Tag` | `tag` | data access + structural substrate |
| `IdentityPrincipal` | `identity_principal` | identity |

`Fundamental::ALL` is a `const [Fundamental; 9]` so callers can iterate every
variant without Clippy grumbling about exhaustive matches.

### Composites ([`composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs))

8 variants, each with:

- `as_str()` — canonical string form (e.g. `memory_object`).
- `kind_tag()` — the `#kind:{name}` identity tag every instance carries.
- `constituents() -> &'static [Fundamental]` — the fundamentals this
  composite expands to at Permission Check time. Every composite implicitly
  pulls in `Fundamental::Tag` (asserted by
  `every_composite_includes_tag_fundamental` test).

| Variant | `constituents()` |
|---|---|
| `ExternalServiceObject` | NetworkEndpoint, SecretCredential, Tag |
| `ModelRuntimeObject` | NetworkEndpoint, SecretCredential, EconomicResource, Tag |
| `ControlPlaneObject` | DataObject, IdentityPrincipal, Tag |
| `MemoryObject` | DataObject, Tag |
| `SessionObject` | DataObject, Tag |
| `AuthRequestObject` | DataObject, Tag |
| `InboxObject` | DataObject, Tag |
| `OutboxObject` | DataObject, Tag |

### Nodes ([`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs))

`NodeKind` is the inventory enum (37 variants). Alongside it, 37 struct
types exist — one per kind. M1-critical structs carry the full field
shape; the rest are scaffolded as `struct { id }` with a `[PLANNED M<n>]`
comment pointing at the milestone where they'll be fleshed out.

Load-bearing (full-field) in M1:
`Agent`, `AgentProfile`, `User`, `Organization`, `Template`, `Grant`,
`AuthRequest` (+ `ResourceSlot`, `ApproverSlot`, `AuthRequestState`,
`ResourceSlotState`, `ApproverSlotState`), `Consent`,
`ToolAuthorityManifest`, `Channel`, `InboxObject`, `OutboxObject`,
`Memory`. See
[`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) for the
full shape of each.

Scaffolded (id-only, for later milestones): Identity (M5), Session/Loop/
Turn/MessageNode/EventNode (M5), ModelConfig/ToolDefinition/
ToolImplementation/McpServer/OpenApiSpec/SystemPrompt/EvaluationStrategy
(M2), Skill (M4), ExecutionLimits/CompactionPolicy/RetryPolicy/CachePolicy
(M4), Project/Task/Bid (M4), Rating (M5), AgentConfig (M2), PromptBlock
(M4).

### Edges ([`edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs))

66 variants as a tagged enum. Each variant's payload carries the edge's
`EdgeId` plus the IDs of its `from` and `to` nodes — typed to the concrete
pair (`from: AgentId, to: GrantId` on `AgentHoldsGrant`, for example).

Where the concept doc lists the same edge *name* with multiple type pairs,
each pair is a distinct variant — this is what makes the count reach 66:

- `CONNECTS_TO` (Agent→McpServer vs Agent→OpenApiSpec) → 2 variants
- `HOLDS_GRANT` (Agent/Project/Org → Grant; Agent listed in both
  Agent-Centric and Governance tables) → 4 variants
- `PROVIDES_TOOL` (McpServer→ToolDef vs OpenApiSpec→ToolDef) → 2 variants
- `OWNED_BY` (Agent→User specific case + generic Resource→Principal)
  → 2 variants
- `SUBMITTED_BY` (Bid→Agent vs AuthRequest→Principal) → 2 variants

`EDGE_KIND_NAMES: [&str; 66]` is the companion string array — tests use it
to assert the 66-count and distinctness.

## Ontology ↔ SurrealDB schema mapping

Every `NodeKind` variant has a corresponding SCHEMAFULL table in
[`0001_initial.surql`](../../../../../../modules/crates/store/migrations/0001_initial.surql);
every `Edge` variant has a `DEFINE TABLE ... TYPE RELATION` line with
concrete `FROM <src> TO <dst>` endpoints where both ends are single-typed.

Three edge variants (`OwnedBy`, `Created`, `AllocatedTo`) accept
`Resource`/`Principal` type unions, so their schema entries use
unconstrained `DEFINE TABLE ... TYPE RELATION` — the domain layer enforces
the union constraint in Rust.

## Testing

27 unit tests in
[`modules/crates/domain/src/`](../../../../../../modules/crates/domain/src/),
including:

- `fundamentals::tests::all_contains_exactly_nine`
- `composites::tests::all_contains_exactly_eight`
- `composites::tests::every_composite_includes_tag_fundamental`
- `nodes::tests::node_kind_all_is_exactly_37`
- `nodes::tests::auth_request_state_all_is_exactly_nine`
- `edges::tests::edge_kind_names_is_exactly_66`
- `model::tests::ontology_has_nine_fundamentals` (+ 3 siblings) — the
  cross-cutting invariants that the commitment ledger's C1 row points at.

Plus serde round-trip coverage on every enum so wire format stays stable as
the schema evolves.
