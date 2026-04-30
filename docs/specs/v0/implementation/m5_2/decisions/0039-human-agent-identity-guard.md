<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0039 — Human Agent Identity guard: defensive (Repository) + preventive (call-site)

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-16
**Closes:** D-new-23 (LOW) — Human Agents have no guard preventing Identity assignment

---

## Context

[`concepts/human-agent.md`](../../../concepts/human-agent.md) lines 16–17 (§ "No Identity") commits:

> No system-computed Identity: Unlike LLM Agents, Human Agents do **not** have a system-computed Identity node. A human's identity exists outside the system — they are participants, not subjects of identity tracking.

CH-01 reviewed this drift but deferred enforcement to CH-16 because Identity had no writers at M5 (D-new-01 still open). With ADR-0038 landing eager Identity creation in `apply_agent_creation`, the guard becomes load-bearing at the same surface.

## Decision

### D39.1 — BOTH guards (defensive at repo + preventive at call site)

Defense in depth (Fork 5 BOTH):

**Defensive guard** at [`Repository::upsert_identity`](../../../../../../modules/crates/domain/src/repository.rs):

- Reads `Agent.kind` for `identity.agent_id`.
- Returns `RepositoryError::HumanAgentHasNoIdentity { agent_id }` if `kind == AgentKind::Human`.
- Fails closed for every caller: production handlers, test fixtures, REPL probes, future scripts.
- Implemented at both layers — `InMemoryRepository::upsert_identity` checks the in-memory `Agent` map; `SurrealRepository::upsert_identity` does a `SELECT kind FROM type::thing('agent', $aid)` + Rust-side check before the SurrealQL write.

**Preventive guard** at [`Repository::apply_agent_creation`](../../../../../../modules/crates/domain/src/repository.rs):

- Pre-flight `match (payload.agent.kind, &payload.identity)`:
  - `(Human, Some(_))` → `Err(HumanAgentHasNoIdentity)`
  - `(Llm, None)` → `Err(InvalidArgument)` (server orchestrator must build via `Identity::default_for_llm`)
  - `(Human, None)` → OK (Identity insertion skipped)
  - `(Llm, Some(_))` → OK (Identity insertion proceeds)
- Skips Identity-row insertion entirely for Human kind; the operator's create call returns success without an Identity ever being written.

### D39.2 — Server orchestrator builds `Identity::default_for_llm` only for LLM agents

`server/src/platform/agents/create.rs` and `server/src/platform/system_agents/add.rs`:

```rust
let identity = if matches!(input.kind, AgentKind::Llm) {
    Some(domain::model::nodes::Identity::default_for_llm(agent_id, input.now))
} else {
    None
};
```

`bootstrap/claim.rs` (CEO is Human-kind by design) leaves `identity: None` implicitly via the orchestrator pattern — verified by inspection.

### D39.3 — 3 unit tests pin the guard pair

| Test | Layer | Asserts |
|---|---|---|
| `upsert_identity_rejects_human_agent_with_typed_error` (in-memory) | defensive | `RepositoryError::HumanAgentHasNoIdentity { agent_id }` |
| `upsert_identity_rejects_human_agent_at_surreal_layer` (Surreal) | defensive | same; SurrealQL kind-read fires |
| `list_identities_for_org_excludes_human_agents` (in-memory) | invariant | listing for an org with one LLM + one Human returns 1 entry |

### D39.4 — No backfill migration for pre-existing Human Identity rows

D-new-01 means Identity has zero rows pre-CH-16; no scaffold migration ever wrote one. After CH-16, only LLM-kind agents can persist Identity rows. No backfill is needed.

### D39.5 — Concept-doc semantics unchanged

`concepts/human-agent.md` § "No Identity" stays at "external to system" framing. The guard is enforcement of a concept invariant, not a concept change. Concept-doc verified-header bumped at CH-16 seal to capture the runtime-enforcement reference.

---

## Consequences

**Positive:**
- D-new-23 closed at M5.2; concept-`human-agent.md` § "No Identity" flips from `silent-in-code` to `honored`.
- Defense-in-depth pair survives future direct-repo usage (test fixtures, REPL).
- Fails closed: a misuse path produces a typed error, not a silent partial commit.

**Negative:**
- One extra SurrealDB read per `upsert_identity` call (the kind-check before the upsert). The kind-check is a single LIMIT-1 select on a primary key, ~sub-ms; not a hot path.

**Mitigations:**
- The kind-check fast-path on the in-memory side is a HashMap lookup; same on the SurrealDB side it's an indexed primary-key read.

---

## Cross-References

- [`concepts/human-agent.md`](../../../concepts/human-agent.md) § "No Identity"
- [drift D-new-23](../../m5_1/drifts/D-new-23.md)
- ADR-0034 (CH-01) — `AgentKind::Human/Llm` discriminator that this guard reads
- ADR-0038 (sibling) — Identity materialization that this guard protects
- CH-16 plan archive: [`baby-phi/docs/specs/plan/build/ch-16-identity-node-materialization-2ae4fabe.md`](../../../../plan/build/ch-16-identity-node-materialization-2ae4fabe.md)
