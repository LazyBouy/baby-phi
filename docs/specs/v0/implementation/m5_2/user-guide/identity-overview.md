<!-- Last verified: 2026-04-28 by Claude Code -->

# Identity overview — operator's guide

> **Audience:** Operators who want to understand what Identity means in baby-phi, why it's LLM-only, and what each of the four fields tells them.

---

## What is Identity?

Every LLM agent in baby-phi has a stored **Identity** row — its emergent self, built from what it has done and what it has supervised. Identity is not assigned; it accumulates over the agent's working life. Concept-`agent.md` puts it this way:

> Identity is NOT assigned — it **develops** from the interaction of Soul, Experience, and Skills.

Identity is:

- **Per LLM agent** — Human agents do **not** have Identity (their identity exists outside the system; they're participants, not subjects of identity tracking).
- **Materialized** — a stored row, not computed on demand. Reactively updated by triggering events (session end, memory extraction, skill change, rating received).
- **Queryable** — operators can ask "what is agent X's current identity?" and get back a structured answer.

---

## The four fields

| Field | What it captures | Example |
|---|---|---|
| `self_description` | Agent-authored natural-language bio (≤500 tokens). The agent (or a system-agent synthesiser) rewrites this on session end / skill change / rating event. | "I specialise in security audits, particularly for Rust workspaces with phi-core dependencies. Strong on dependency-graph analysis." |
| `lived` | Direct-doing metrics: sessions completed, ratings received, skills, top tags by frequency. | `{ sessions_completed: 47, sessions_successful: 44, ratings_window: [...], skills: ["rust", "auditing"], specializations: ["security"] }` |
| `witnessed` | Supervised-doing metrics: memories extracted from subordinates' work, subordinates observed, extraction-scope distribution (private vs public). | `{ memories_extracted: 12, subordinates_observed: [...], extraction_scope_distribution: { private: 8, public: 4 } }` |
| `embedding` | Vector derived from `self_description`. Powers similarity search ("who in this org has worked on something like this?"). | (Empty at v0.1 — embedding provider integration deferred to M6.) |

---

## What's empty at v0.1

CH-16 ships the storage substrate. Several content sources are deferred:

- **`self_description`** is empty until CH-21 (memory-extraction listener body) wires the synthesiser. Operators reading a fresh agent's Identity will see an empty string until the agent finishes its first session.
- **`embedding`** is empty until M6-DEFERRED-03 (embedding provider integration). The vector field carries the reserved column; no values flow through yet.
- **`lived` / `witnessed` counters** are zero until the matching update writers ship (CH-21 for `witnessed.memories_extracted`; M6+ for the rest).

At v0.1, every newly-created LLM agent has a fully-zeroed Identity row. The substrate is ready; the writers are sequenced.

---

## How Identity changes over time

Concept-`agent.md` § "Materialization" lists four reactive triggers:

1. **Session ended** — updates `lived.sessions_completed`, `lived.ratings_window` (if a rating arrived).
2. **Memory extracted** — updates `witnessed.memories_extracted`, `witnessed.subordinates_observed`, `witnessed.extraction_scope_distribution`.
3. **Skill changed** — updates `lived.skills`.
4. **Rating received** — updates `lived.ratings_window`.

Each trigger emits a `DomainEvent::IdentityUpdated { trigger, ... }` so listeners (e.g., embedding-refresh, dashboard updates) can react without polling. CH-21 lights the first emitter (memory-extracted); the others land at M6+.

---

## What happens when an LLM agent is archived?

Archive flips `Agent.active = false` but **leaves the Identity row queryable**. This is intentional (ADR-0038 §D38.6): hiring or evaluation queries that join "agents who used to work here" against past Identity content remain answerable. Operators who need GDPR-style erasure can call `delete_identity(agent_id)` explicitly.

---

## Why no Identity for Human Agents?

Concept-`human-agent.md` § "No Identity" is unambiguous:

> A human's identity exists outside the system — they are participants, not subjects of identity tracking.

Two guards enforce this:

1. **Defensive** at the repository — every `upsert_identity` call checks `Agent.kind` and rejects Human-kind callers with a typed error.
2. **Preventive** at the call site — `apply_agent_creation` skips Identity insertion entirely for Human-kind agents.

If you see `HumanAgentHasNoIdentity` from a handler or test fixture, the fix is to skip writing an Identity for that agent — the error is by design.

---

## Cross-References

- [`concepts/agent.md`](../../../concepts/agent.md) — normative spec (§"Identity (Emergent, Event-Driven)" + §"Identity Node Content")
- [`concepts/human-agent.md`](../../../concepts/human-agent.md) §"No Identity"
- ADR-0038 — design decisions
- ADR-0039 — Human-Agent guard
- [`m5_2/architecture/identity-node.md`](../architecture/identity-node.md) — design page
- [`m5_2/operations/identity-operations.md`](../operations/identity-operations.md) — runbook
