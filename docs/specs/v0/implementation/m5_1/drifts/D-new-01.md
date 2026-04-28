<!-- Last verified: 2026-04-28 by Claude Code -->

# D-new-01 — Identity node is an id-only scaffold; 4-field shape (self_description/lived/witnessed/embedding) not materialized

## Identification
- **ID**: D-new-01
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `ontology-gap`, `v0-commitment-violated`, `concept-contradiction`
- **Blocks**: M5.2/P8 memory-extraction listener (extractor is expected to emit Identity updates per agent.md §"Identity is updated reactively on session end, memory extraction, skill change, rating received")
- **Blocked-by**: none directly; conceptually blocked-by D4.2 (real agent_loop must produce transcripts before Identity can be extracted)

## Concept alignment
- **Concept doc(s)**: [`concepts/agent.md`](../../../concepts/agent.md) §"Identity (Emergent)" + §"Identity Node Content"; [`concepts/ontology.md`](../../../concepts/ontology.md) §"Node Types — Identity"
- **Concept claim (verbatim)**: *"The three-field model below (`self_description` + `lived` + `witnessed` + `embedding`) is the v0 commitment — implementations should code against it."*
- **Contradiction**: `Identity` node is an id-only scaffold per `nodes.rs:813-818` with doc comment *"[PLANNED M5] — full field set (self_description, lived, witnessed, embedding) lands when memory-extraction wires in."* Concept labels this field set as a **v0 commitment**.
- **Classification**: `contradicts-concept`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (base plan §M4/§M5 + agent.md): Identity node materialized with 4 fields for LLM Agents; updated reactively by memory-extraction listener + rating events.
- **Reality (shipped state at current HEAD)**: [`modules/crates/domain/src/model/nodes.rs:813-818`](../../../../../../modules/crates/domain/src/model/nodes.rs#L813-L818) `scaffold_node!(Identity, NodeId)` — struct carries only `id`. No `self_description`, `lived`, `witnessed`, `embedding` fields. No edges populating it. No update path.
- **Root cause**: `concept-doc-not-consulted` during M4/M5 planning — scaffold deferred Identity materialization without flagging it against the v0 commitment.

## Where visible in code
- **File(s)**: [`nodes.rs:813-818`](../../../../../../modules/crates/domain/src/model/nodes.rs#L813-L818) Identity scaffold; `nodes.rs:814` PLANNED M5 comment
- **Test evidence**: No Identity acceptance test exists; drift is invisible to current test suite.
- **Grep for regression**: `grep -A3 "scaffold_node!(.*Identity" modules/crates/domain/src/model/nodes.rs` — expect a scaffold line while drift open; post-remediation expect a full struct def with `self_description`, `lived`, `witnessed`, `embedding` fields.

## Remediation scope (estimate only)
- **Approach (sketch)**: Define `Identity` struct with 4 fields per concept. Migration 0006 adds `identity` table with FLEXIBLE TYPE object for `lived` + `witnessed` nested shapes + `Vec<f32>` embedding. Repo methods: `upsert_identity(agent_id, ...)`. Wire memory-extraction listener to update reactively on SessionEnded (bundle with M5.2/P8 or new chunk).
- **Implementation chunk this belongs to**: CH-16
- **Dependencies on other drifts**: D4.2 (real transcripts needed before Identity extraction makes sense); D6.1 (listener wiring path)
- **Estimated effort**: 3 engineer-days (struct + migration + repo + listener update path).
- **Risk to concept alignment if deferred further**: HIGH — v0 commitment violated past M5; compounds with Memory contract (C-M6-1) since Identity extraction is tightly coupled.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none — not in M5 drift ledger; this is a newly-discovered concept-code drift)
- Code comments: `nodes.rs:814` "PLANNED M5" note
- ADR references: none
- Other doc pointers: [`concepts/agent.md`](../../../concepts/agent.md) §"Identity"

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 1 report)
- 2026-04-24 — `classified` — Bucket A HIGH; v0 commitment violated; concept-`agent.md` §"Identity Node Content" labels 4-field shape as binding (backfill)
- 2026-04-24 — `scoped` — assigned to CH-16 per [forward-scope §1 line 156](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (backfill)
- 2026-04-28 — `in-chunk-plan` — CH-16 plan approved ([`build/2ae4fabe-ch-16-identity-node-materialization.md`](../../../../plan/build/2ae4fabe-ch-16-identity-node-materialization.md)); 4-field Identity struct + `LivedExperience` + `WitnessedExperience` + `RatingPoint` + `SkillRef` + `ExtractionScopeDistribution` in scope; migration 0009 with UNIQUE-on-`agent_id`; eager creation in `apply_agent_creation` for every LLM agent; `DomainEvent::IdentityUpdated` variant + 4 repo methods; user-decided forks: EAGER timing + EMPTY embedding + NEW IdentityUpdated event + LEAVE QUERYABLE on archive + BOTH guards + EMPTY initial self_description
- 2026-04-28 — `remediated` — CH-16 chunk-seal; 4-field Identity struct shipped at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) (replaces the `scaffold_node!` line); migration 0009 ships at [`modules/crates/store/migrations/0009_identity_node.surql`](../../../../../../modules/crates/store/migrations/0009_identity_node.surql) with `OVERWRITE` field redefinitions over the 0001 scaffold + UNIQUE-on-`agent_id` index; 4 repo methods on both `InMemoryRepository` + `SurrealRepository`; eager creation wired in `apply_agent_creation` (Llm-kind only); audit emitter `platform.identity.created` (Alerted) wired post-commit at `agents/create.rs` + `system_agents/add.rs`; `DomainEvent::IdentityUpdated` variant + `IdentityUpdateTrigger` enum shipped (no production emitter — deferred to CH-21); embedding population deferred to **M6-DEFERRED-03**. Tests: 8 struct unit + 9 in-memory CRUD + 4 SurrealDB integration + 2 migration round-trip + 3 IdentityUpdated event + 3 audit emitter = ~29 new tests. ADR-0038 §D38.1–D38.7 records the design decisions. Bonus drift fix surfaced during P1: CH-06's migration 0008 was registered in `EMBEDDED_MIGRATIONS` (had been left out of the ledger at CH-06 seal — see §"Mid-flight discovery" in the plan).
