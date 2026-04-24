<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0029 — Session persistence + SessionRecorder wrap

**Status: Accepted** (flipped at M5/P3 close, 2026-04-23).

Ratification history:
- **P1 close (2026-04-22)** — §D29.1 (the 3-way Session /
  LoopRecordNode / TurnNode wrap) ratified. Serde round-trip +
  compile-time coercion witnesses green.
- **P3 close (2026-04-23)** — §D29.2 (`BabyPhiSessionRecorder` wrap)
  + §D29.3 (reject `BasicAgent`) + §D29.4 (reject
  `save_session`/`load_session`) + §D29.5 (reject
  `include_streaming_events = true`) all ratified. Implementation
  in [`domain/src/session_recorder.rs`](../../../../../../modules/crates/domain/src/session_recorder.rs)
  with 3 tests covering the AgentStart→TurnStart→TurnEnd→AgentEnd
  trace + re-entry de-duplication + default-scope invariant. phi-core
  import count now at **19** (17 pre-P3 + `SessionRecorder` +
  `AgentEvent` in `session_recorder.rs`).

## Context

M5 introduces session persistence (closes C-M5-3). Three questions
had to be resolved up front:

1. **How are `phi_core::Session` / `LoopRecord` / `Turn` stored at the
   governance tier?** phi-core ships these as full-shape rust structs
   with their own serde discipline. Duplicating them in baby-phi
   would re-introduce the exact two-source-of-truth problem the
   phi-core reuse mandate (see [`phi/CLAUDE.md`](../../../../../../CLAUDE.md)
   §phi-core Leverage) forbids.

2. **Who drives materialisation of `Turn` structs from the
   `AgentEvent` stream?** phi-core already ships
   `phi_core::session::recorder::SessionRecorder`, which consumes an
   `AgentEvent` stream + materialises `Session` → `LoopRecord` →
   `Turn`. Re-implementing this in baby-phi would be the second
   two-source-of-truth failure in the same milestone.

3. **Where does baby-phi's governance data (owning_org,
   owning_project, governance_state, tokens_spent, started_at /
   ended_at) live relative to phi-core's session tree?**

## Decision

### D29.1 — 3-way node wrap at the governance tier (D2 path a)

Follow the M3 `OrganizationDefaultsSnapshot` wrap pattern byte-for-byte.
In `modules/crates/domain/src/model/nodes.rs`:

```rust
use phi_core::session::model::LoopRecord as PhiCoreLoopRecord;
use phi_core::session::model::Session as PhiCoreSession;
use phi_core::session::model::Turn as PhiCoreTurn;

pub struct Session {
    pub id: SessionId,
    pub inner: PhiCoreSession,
    pub owning_org: OrgId,
    pub owning_project: ProjectId,
    pub started_by: AgentId,
    pub governance_state: SessionGovernanceState,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tokens_spent: u64,
}

pub struct LoopRecordNode {
    pub id: LoopId,
    pub inner: PhiCoreLoopRecord,
    pub session_id: SessionId,
    pub loop_index: u32,
}

pub struct TurnNode {
    pub id: TurnNodeId,
    pub inner: PhiCoreTurn,
    pub loop_id: LoopId,
    pub turn_index: u32,
}

pub enum SessionGovernanceState { Running, Completed, Aborted, FailedLaunch }
```

**Nested `inner`, NOT `#[serde(flatten)]`.** An earlier draft of this ADR
called for `#[serde(flatten)]` on the `inner` field. At P1 implementation
this was rejected because phi-core's `Session` already carries
`session_id: String` / `agent_id: String` fields that collide with
baby-phi's `id: SessionId` / `started_by: AgentId` newtype UUIDs —
flattening would either duplicate data at the wire tier or silently
override one set with the other during deserialisation. The nested
form matches M3's `OrganizationDefaultsSnapshot` precedent (itself
nested, not flattened) + lines up with SurrealDB's `FLEXIBLE TYPE
object` storage column per-tier.

Three compile-time coercion witnesses pin the invariant that the
`inner` field holds phi-core's type (identical to the M4 pattern in
[`agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs)):

```rust
#[allow(dead_code)] fn _is_phi_core_session(_: &phi_core::session::model::Session) {}
#[allow(dead_code)] fn _is_phi_core_loop_record(_: &phi_core::session::model::LoopRecord) {}
#[allow(dead_code)] fn _is_phi_core_turn(_: &phi_core::session::model::Turn) {}
```

Storage layout: one SurrealDB table per tier (`session`,
`loop_record`, `turn`) with `FLEXIBLE TYPE object` for the phi-core
`inner` field + explicit columns for the governance extensions. The
nested tree phi-core's `Session.loops: Vec<LoopRecord>` expresses is
**flattened at the storage tier** (one table per level) so per-Turn
queries are cheap; the nested form is reconstructed only when the
full `SessionDetail` aggregate is requested.

### D29.2 — Wrap `phi_core::SessionRecorder`, don't re-implement

`modules/crates/domain/src/session_recorder.rs` defines:

```rust
pub struct BabyPhiSessionRecorder {
    inner: phi_core::session::recorder::SessionRecorder,
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    event_bus: Arc<dyn EventBus>,
}

impl BabyPhiSessionRecorder {
    pub async fn on_phi_core_event(&mut self, event: phi_core::types::event::AgentEvent) {
        // 1. Feed phi-core's recorder.
        self.inner.on_event(&event).await;
        // 2. Read the materialised Session/LoopRecord/Turn back.
        // 3. Persist to SurrealDB via repo.
        // 4. Emit governance events (SessionStarted on first AgentStart,
        //    SessionEnded on final TurnEnd or non-null rejection).
    }
}
```

Rationale: phi-core's recorder is the source of truth for
event→record materialisation. baby-phi is the **sink**. Composing the
two avoids double-materialisation (which was the risk if we wrote our
own recorder), avoids phi-core serde drift (phi-core's recorder
always produces shapes phi-core's structs expect), and keeps the
governance extension surface minimal.

### D29.3 — Run `agent_loop()` directly, not `BasicAgent::run()`

Session launch at `server/src/platform/sessions/launch.rs` calls the
phi-core free function `phi_core::agent_loop(prompts, ctx, cfg, tx,
cancel_token)` directly. It does **NOT** instantiate
`phi_core::agents::BasicAgent`.

Rationale: baby-phi's governance `Agent` node is the identity
record; phi-core's `BasicAgent` is a runtime stateful wrapper (see
[`phi-core/CLAUDE.md`](../../../../../../../phi-core/CLAUDE.md)). Using
the raw loop keeps baby-phi's Session node authoritative + avoids
coupling to phi-core's runtime agent trait (which may evolve
independently). ADR-0027's per-agent ExecutionLimits override + the
M4 `AgentProfile` wrap already carry everything the loop needs via
`AgentLoopConfig`.

### D29.4 — Reject `phi_core::save_session` / `load_session`

phi-core ships JSON-file persistence helpers
(`phi_core::session::save_session` / `load_session`). baby-phi does
**not** use them — persistence is SurrealDB via the
`BabyPhiSessionRecorder` sink. The helpers stay in phi-core for
phi-core's own test harness + external consumers; baby-phi's reuse
mandate walks Q3 and rejects them explicitly.

### D29.5 — Reject `include_streaming_events = true` at M5

`SessionRecorderConfig::include_streaming_events` controls whether
every `MessageUpdate` delta is stored. At M5 this stays `false` —
SurrealDB row volume would balloon without a replay UX that needs
the deltas. M7b revisits if a rich replay surface surfaces as a
requirement.

## Consequences

**Positive**
- Zero re-implementation of phi-core session / recorder primitives.
- Single wrap pattern across `OrganizationDefaultsSnapshot` (M3),
  `AgentProfile` (M1+M4), and now `Session` / `LoopRecord` / `Turn`
  (M5) — reviewers recognise the shape instantly.
- Per-Turn SurrealDB queries stay O(1) via the flattened storage
  layout.
- Session-end governance events (`SessionStarted`, `SessionEnded`,
  `SessionAborted`) emit from the recorder's synchronous event
  callback — no scheduler races.

**Negative**
- `SessionDetail` aggregate requires 3 table reads (session +
  loop_records + turns). Mitigated by a single compound SurrealQL
  query with SELECT INCLUDE at the repo layer.
- `BabyPhiSessionRecorder` must be `Send + Sync + 'static` to be
  shared across the Session worker pool. Mitigated by `Arc<Mutex<_>>`
  on the phi-core recorder (acceptable — one Mutex per active
  session, no cross-session contention).

**Neutral**
- phi-core recorder version bumps bring breaking changes to `Turn` /
  `LoopRecord` shape via serde. The compile-time coercion witnesses
  catch this at build time + the `#[serde(default)]` discipline on
  governance extension fields keeps backward-compat within baby-phi.

## References

- [M5 plan archive §Part 1.5 + §D2 + §P1 + §P3](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
- [M3 ADR-0020](../../m3/decisions/0020-organization-defaults-embedded.md) — sibling wrap pattern (`OrganizationDefaultsSnapshot` wraps four phi-core types at the governance-node tier).
- [M4 ADR-0027](../../m4/decisions/0027-per-agent-execution-limits-override.md) — layered override precedent.
- [`phi/CLAUDE.md`](../../../../../../CLAUDE.md) §phi-core Leverage.
- [`phi-core/CLAUDE.md`](../../../../../../../phi-core/CLAUDE.md) §Session persistence.
- [Session persistence architecture](../architecture/session-persistence.md) — detailed sequence diagrams (seeded at P0, filled at P1).
