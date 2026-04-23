<!-- Last verified: 2026-04-23 by Claude Code -->

# Session persistence — 3-wrap pattern

**Status**: [EXISTS] since M5/P1 for the node-tier wraps (Session /
LoopRecordNode / TurnNode) + migration 0005 storage tier.
[PLANNED M5/P3] for `BabyPhiSessionRecorder` + [PLANNED M5/P2] for
the 14 new repo methods.

Documents the three-way Session / LoopRecordNode / TurnNode wrap
pattern M5 introduces to persist phi-core's session tree at the
baby-phi governance tier. Pinned by
[ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md)
(wrap portion Accepted at P1, recorder portion Proposed until P3).

## Wrap pattern — shipped at M5/P1

Three node types in
[`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs)
wrap the corresponding phi-core types via a nested `inner` field.
Governance fields (owning_org, owning_project, started_by,
governance_state, started_at, ended_at, tokens_spent) sit alongside.
Pattern mirrors M3's `OrganizationDefaultsSnapshot` wrap byte-for-byte
(4 nested phi-core types + governance fields) + M4's
`AgentProfile.blueprint` wrap.

```rust
use phi_core::session::model::LoopRecord as PhiCoreLoopRecord;
use phi_core::session::model::Session as PhiCoreSession;
use phi_core::session::model::Turn as PhiCoreTurn;

pub struct Session {
    pub id: SessionId,
    pub inner: PhiCoreSession,           // wrap
    pub owning_org: OrgId,               // governance
    pub owning_project: ProjectId,       // governance
    pub started_by: AgentId,             // governance
    pub governance_state: SessionGovernanceState,  // governance
    pub started_at: DateTime<Utc>,       // governance
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>, // governance
    #[serde(default)]
    pub tokens_spent: u64,               // governance (M5/P3 recorder maintains)
}

pub struct LoopRecordNode {
    pub id: LoopId,
    pub inner: PhiCoreLoopRecord,        // wrap
    pub session_id: SessionId,           // governance
    pub loop_index: u32,                 // governance
}

pub struct TurnNode {
    pub id: TurnNodeId,
    pub inner: PhiCoreTurn,              // wrap
    pub loop_id: LoopId,                 // governance
    pub turn_index: u32,                 // governance
}
```

The wraps are renamed `LoopRecordNode` / `TurnNode` (not `LoopRecord`
/ `Turn`) to keep them outside the
[`check-phi-core-reuse.sh`](../../../../../../scripts/check-phi-core-reuse.sh)
`FORBIDDEN` denylist for `LoopRecord` / `Turn`. The `Session` wrap
keeps the name `Session` (phi-core's `Session` is aliased as
`PhiCoreSession` at import time to disambiguate) — the denylist
carries a special-case `WRAP_FORBIDDEN` entry permitting `Session`
declaration **only** in `domain/src/model/nodes.rs`.

## Why nested, not flattened

An earlier ADR-0029 draft called for `#[serde(flatten)]` on the
`inner` field. At P1 implementation this was rejected:
- phi-core's `Session` carries `session_id: String` / `agent_id:
  String` fields that would collide with baby-phi's `id: SessionId`
  / `started_by: AgentId` newtype UUIDs on flatten — either
  duplicating data or silently overriding one set during
  deserialisation.
- SurrealDB `FLEXIBLE TYPE object` columns on the storage tier map
  naturally to a nested JSON document for the `inner` field.
- Nested matches the M3 `OrganizationDefaultsSnapshot` wrap + the
  M4 `AgentProfile.blueprint` wrap — uniform idiom across the three
  milestones.

## Compile-time coercion witnesses (M5/P1)

Three witness fns in
[`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs)'s
test module pin the invariant that `inner` carries phi-core's type —
a rename or shape change in phi-core breaks the baby-phi build
immediately:

```rust
#[allow(dead_code)] fn _is_phi_core_session(_: &PhiCoreSession) {}
#[allow(dead_code)] fn _is_phi_core_loop_record(_: &PhiCoreLoopRecord) {}
#[allow(dead_code)] fn _is_phi_core_turn(_: &PhiCoreTurn) {}
```

Identical discipline to M4's `update.rs` witnesses for `AgentProfile`
/ `ExecutionLimits` / `ThinkingLevel`.

## Storage layout — migration 0005

One SurrealDB table per tier. `session` and `turn` already existed
as id-only scaffolds in migration 0001 (with just `created_at:
string`); M5/P1's migration 0005 layers the M5 governance columns
on top via `DEFINE FIELD` (no `DEFINE TABLE` — SurrealDB rejects
re-declaration). `loop_record` is a fresh table (the 0001 `loop`
scaffold stays as a zombie; M7b cleanup may drop it).

| Table | Origin | M5/P1 fields added |
|---|---|---|
| `session` | 0001 scaffold | `inner (FLEXIBLE object)`, `owning_org`, `owning_project`, `started_by`, `governance_state`, `started_at`, `ended_at?`, `tokens_spent` + 2 indexes (by project, by agent) |
| `loop_record` | 0005 (new) | `inner (FLEXIBLE object)`, `session_id`, `loop_index` + 1 index (by session) |
| `turn` | 0001 scaffold | `inner (FLEXIBLE object)`, `loop_id`, `turn_index` + 1 index (by loop) |

Governance column `governance_state` carries an ASSERT clause
matching the 4-variant `SessionGovernanceState` enum (`running`,
`completed`, `aborted`, `failed_launch`). Per-Turn queries stay
O(1) via the flattened storage layout; the nested
`phi_core::Session.loops: Vec<LoopRecord>` tree is reconstructed at
the repo layer only on full drill-down.

Production writers at M5/P2 + P4 must also populate the
pre-existing `created_at: string` column (0001 invariant) alongside
`started_at`; the repo layer typically sets both to the same
wall-clock value.

## Runs-session edge retype

0001 scaffolded `runs_session` as a RELATION `agent → session` with
zero production writers. At M5/P1 migration 0005 REMOVEs that
table + DEFINEs it fresh as `session → project` — the semantically
correct direction for page 11's "Recent sessions" panel query.
Forward-only retype pattern, same as the `uses_model` flip in the
same migration.

## SessionRecorder wrap (planned for M5/P3)

`BabyPhiSessionRecorder` composes `phi_core::session::recorder::SessionRecorder`
+ adds a SurrealDB persist hook. phi-core's recorder materialises the
Session/LoopRecord/Turn tree from the `AgentEvent` stream; baby-phi
is the sink. See
[ADR-0029 §D29.2](../decisions/0029-session-persistence-and-recorder-wrap.md)
for the full composition contract (remains Proposed until M5/P3).

## Cross-references

- [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md) — session persistence + recorder wrap.
- [Session launch architecture](session-launch.md) — page 14 flow at M5/P4.
- [phi-core reuse map](phi-core-reuse-map.md) — per-phase import count predictions + actuals.
- [`phi-core/CLAUDE.md` §Session persistence](../../../../../../../phi-core/CLAUDE.md).
