<!-- Last verified: 2026-05-19 by Claude Code (CH-28 P-DOCS deliverable 4 — §Edges count bumped 66 → 74 to reflect post-CH-28 EDGE_KIND_NAMES cardinality (+2 from F1.c hybrid Blueprint table: AGENT_PROFILE_USES_BLUEPRINT + AGENT_USES_BLUEPRINT_OVERRIDE; rename HAS_PROFILE → USES_PROFILE keeps the count flat; CH-23 +2 MANAGES + HAS_AGENT_SUPERVISOR + CH-25 +1 OWNS not yet reflected here pre-CH-28 — CH-28 brings the count current). §Nodes load-bearing list extended with NEW Blueprint struct + BlueprintId newtype per ADR-0063 §D63.1 (sibling to AgentProfile in nodes.rs; carries `id`, `agent_id` Option<AgentId>, `parallelize`, `model_config_id`, `mock_response`, `created_at`). §AgentProfile-wraps-phi-core paragraph amended: per-agent override fields (`parallelize`, `model_config_id`, `mock_response`) MOVE OFF the `AgentProfile` literal entirely (post-CH-28) and live on the NEW per-agent Blueprint override row via AGENT_USES_BLUEPRINT_OVERRIDE; AgentProfile struct now carries only `id`, `agent_id`, `blueprint` (phi-core wrap), `created_at`. Cross-ref added to m6/architecture/agent-profile-cardinality.md. Testing list updated: 66 → 74 invariant. Cycle hex `0412eb06`. --><!-- Last verified: 2026-04-28 by Claude Code (CH-06: 14 composite/node struct types gain `pub tags: Vec<String>` (#[serde(default)]) per ADR-0037; emission wired at 10+ creation handlers; Memory.tags pre-existed. CH-16: Identity node materialized as 4-field struct (`self_description` / `lived` / `witnessed` / `embedding`) per ADR-0038; `scaffold_node!` line replaced with full struct + LivedExperience + WitnessedExperience + supporting types; migration 0009 adds `identity` table fields with UNIQUE-on-`agent_id` index. CH-21: adds `DomainEvent::MemoryExtracted` variant to the events enum (no schema change — Memory + Identity tables pre-existed); 11 DomainEvent variants total. See ADR-0041.) -->

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
| Node kinds | **38** (37 + Blueprint added at CH-28) | [`model::nodes::NodeKind`](../../../../../../modules/crates/domain/src/model/nodes.rs) |
| Edge kinds | **74** (post-CH-28; 66 at M1 + 2 M4/P1 + 2 CH-23 + 1 CH-25 + 2 CH-28 F1.c hybrid Blueprint = 73 net … see edge-history below) | [`model::edges::EDGE_KIND_NAMES`](../../../../../../modules/crates/domain/src/model/edges.rs) |
| Auth Request states | **9** | [`model::nodes::AuthRequestState::ALL`](../../../../../../modules/crates/domain/src/model/nodes.rs) |

The pre-M1 build-plan headline "31 nodes + 56+ edges" was a stale
approximation. The concept doc has been updated through M5 + CH-28
accumulations to the accurate post-CH-28 cardinalities (38 nodes + 74
edges). See [`m6/architecture/agent-profile-cardinality.md`](../../m6/architecture/agent-profile-cardinality.md)
for the CH-28-specific changes (NEW Blueprint node + NEW edge variants).

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

**Load-bearing additions at CH-28 (per ADR-0063 §D63.1):** `Blueprint`
(NEW struct + `BlueprintId` newtype, sibling to `AgentProfile` in
`nodes.rs`) carries the per-row override fields that pre-CH-28 lived on
the `AgentProfile` literal — `parallelize`, `model_config_id`,
`mock_response`. The `agent_id: Option<AgentId>` discriminator
distinguishes template-rows (`None`) from per-agent override-rows
(`Some(id)`); a UNIQUE-WHERE index enforces "at most one override row per
agent". See [`m6/architecture/agent-profile-cardinality.md`](../../m6/architecture/agent-profile-cardinality.md)
for the full design.

Scaffolded (id-only, for later milestones): ~~Identity (M5)~~ now
materialized at CH-16 / M5.2 — see ADR-0038 + concept-`agent.md`
§"Identity Node Content"; the 4-field struct (`self_description` /
`lived: LivedExperience` / `witnessed: WitnessedExperience` /
`embedding: Vec<f32>`) lives in [`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs)
behind a `HAS_IDENTITY` edge from each LLM agent. UNIQUE-on-`agent_id`
per migration 0009. Session/Loop/Turn/MessageNode/EventNode (M5),
ModelConfig/ToolDefinition/ToolImplementation/McpServer/OpenApiSpec/
SystemPrompt/EvaluationStrategy (M2), Skill (M4), ExecutionLimits/
CompactionPolicy/RetryPolicy/CachePolicy (M4), Project/Task/Bid (M4),
Rating (M5), AgentConfig (M2), PromptBlock (M4).

**`Grant.fundamentals`** (added M2/P4.5 — G19 / D17). The `Grant` node
carries an explicit `fundamentals: Vec<Fundamental>` field alongside
`resource.uri`. When non-empty, the engine's
[`resolve_grant`](../../../../../../modules/crates/domain/src/permissions/expansion.rs)
uses this list verbatim (Case D). When empty (the `#[serde(default)]`
for pre-P4.5 rows), the engine falls back to the legacy URI-derivation
path that shipped in M1: fundamental name → {that fundamental};
composite name → constituent fundamentals; `system:root` → every
fundamental; opaque URI → empty set (grant can't match). **Empty
preserves M1 semantics**, so every grant persisted before P4.5
continues to resolve identically. Handlers that issue instance-URI
grants (e.g. the vault's `secret:<slug>` grant) populate the field
explicitly so the engine binds the grant to the right class without a
URI-scheme convention.

**AgentProfile wraps phi-core (with post-CH-28 override-field relocation).** Baby-phi's `AgentProfile` node
[wraps](../../../../../../modules/crates/domain/src/model/nodes.rs)
[`phi_core::agents::profile::AgentProfile`](../../../../../../../phi-core/src/agents/profile.rs)
as a `blueprint` field. Pre-CH-28, baby-phi added the per-row override
fields (`parallelize`, `model_config_id`, `mock_response`) directly to
the `AgentProfile` struct alongside `id`, `agent_id`, `created_at`.
**Post-CH-28 (ADR-0063 §D63.1)**, the override fields MOVE OFF the
`AgentProfile` literal entirely and live on a NEW per-agent `Blueprint`
override row reached via `AGENT_USES_BLUEPRINT_OVERRIDE`; the
`AgentProfile` struct now carries only `id`, `agent_id`, `blueprint`
(phi-core wrap), `created_at`. Every field phi-core's blueprint already
models (`system_prompt`, `thinking_level`, `temperature`, `skills`,
`workspace`, etc.) continues to live on `blueprint` as the single source
of truth. This matches
[`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md)
which classifies phi-core's `AgentProfile` as a **Node** mapping to
phi's `AgentProfile`. M6's "Agent Profile Editor" page (CH-37 a05) edits
both the governance fields (now on the per-agent `Blueprint` override
row) and the template `blueprint` wrap directly. See
[`m6/architecture/agent-profile-cardinality.md`](../../m6/architecture/agent-profile-cardinality.md)
for the full design.

**ToolDefinition vs `phi_core::AgentTool`.** The scaffolded
`ToolDefinition` node type (fleshed out in M2) is a **policy / audit
metadata** surface — the tool's permission grants, cost bounds, audit
class — whereas
[`phi_core::types::tool::AgentTool`](../../../../../../../phi-core/src/types/tool.rs)
is a **runtime trait** (`name`, `parameters_schema`, `execute(params,
ctx) -> ToolResult`). They meet at execution time: M4's session-launch
wraps each `phi_core::AgentTool` behind a permission-check that
consults the matching `ToolDefinition`. Distinct concerns; no
reimplementation.

### Edges ([`edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs))

74 variants as a tagged enum (post-CH-28 cardinality). Each variant's payload carries the edge's
`EdgeId` plus the IDs of its `from` and `to` nodes — typed to the concrete
pair (`from: AgentId, to: GrantId` on `AgentHoldsGrant`, for example).

Where the concept doc lists the same edge *name* with multiple type pairs,
each pair is a distinct variant — this is what pushes the count above the
plain concept-doc enumeration:

- `CONNECTS_TO` (Agent→McpServer vs Agent→OpenApiSpec) → 2 variants
- `HOLDS_GRANT` (Agent/Project/Org → Grant; Agent listed in both
  Agent-Centric and Governance tables) → 4 variants
- `PROVIDES_TOOL` (McpServer→ToolDef vs OpenApiSpec→ToolDef) → 2 variants
- `OWNED_BY` (Agent→User specific case + generic Resource→Principal)
  → 2 variants
- `SUBMITTED_BY` (Bid→Agent vs AuthRequest→Principal) → 2 variants

**CH-28 additions (per ADR-0063 §D63.10):**

- `AGENT_PROFILE_USES_BLUEPRINT` (AgentProfile → Blueprint, template pointer) → 1 variant
- `AGENT_USES_BLUEPRINT_OVERRIDE` (Agent → Blueprint, per-agent override pointer) → 1 variant

**CH-28 rename (per ADR-0063 §D63.2; no count change):** `HAS_PROFILE` → `USES_PROFILE` (the variant body is unchanged; the verb rename matches the post-CH-28 N:1 template-sharing semantic).

`EDGE_KIND_NAMES: [&str; 74]` is the companion string array — tests use it
to assert the 74-count and distinctness.

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
- `nodes::tests::node_kind_all_is_exactly_38` (was 37 pre-CH-28; bumps to 38 with the NEW Blueprint node per ADR-0063 §D63.1)
- `nodes::tests::auth_request_state_all_is_exactly_nine`
- `edges::tests::edge_kind_names_is_exactly_74` (was 66 at M1; cumulative bumps M4/P1 +2 + CH-23 +2 + CH-25 +1 + CH-28 +2 = 74 per ADR-0063 §D63.10)
- `model::tests::ontology_has_nine_fundamentals` (+ 3 siblings) — the
  cross-cutting invariants that the commitment ledger's C1 row points at.

Plus serde round-trip coverage on every enum so wire format stays stable as
the schema evolves.
