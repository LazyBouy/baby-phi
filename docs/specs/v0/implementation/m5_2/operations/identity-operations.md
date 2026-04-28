<!-- Last verified: 2026-04-28 by Claude Code -->

# Identity operations runbook

> **Audience:** SREs and operators triaging Identity-related issues. Pair this page with [`identity-node.md`](../architecture/identity-node.md) (design) and [`identity-overview.md`](../user-guide/identity-overview.md) (operator-facing reference).

---

## Common symptoms + fixes

| Symptom | Likely cause | Fix |
|---|---|---|
| `HumanAgentHasNoIdentity { agent_id }` error from a handler | Caller tried to write an Identity for a Human-kind agent. | Concept-`human-agent.md` § "No Identity" mandates this. Fix the caller — Human-kind agents do not have system-computed Identity. If the caller is a test fixture, change `payload.identity` from `Some(...)` to `None`. |
| LLM-kind agent created but `get_identity(agent_id)` returns `None` | The agent was created via a path that bypasses `apply_agent_creation` (e.g., a test fixture using direct `create_agent` without the compound-tx orchestrator). | Use `apply_agent_creation` for production paths. For test fixtures that need an LLM agent without an Identity row, call `repo.delete_identity(agent_id)` after creation — but understand this contradicts concept-doc semantics. |
| `Repository::upsert_identity` returns `InvalidArgument: agent not found` | The agent row hasn't been written yet (referential integrity). | Ensure `create_agent` (or `apply_agent_creation`) commits before `upsert_identity` runs. |
| Identity row stays after agent archive | Expected behaviour per ADR-0038 §D38.6 LEAVE QUERYABLE policy. | Not a bug. To delete explicitly, call `repo.delete_identity(agent_id)`. |
| `embedding: Vec<f32>` is always empty | Expected at v0.1 per ADR-0038 §D38.3. | Embedding population is deferred to M6-DEFERRED-03 (Identity embedding provider integration). No-op until then. |
| Identity row's `self_description` is `""` even after sessions ran | Expected at v0.1. | The synthesiser ships at CH-21 (memory-extraction listener body). Until then `self_description` only changes via direct `upsert_identity` calls. |

---

## Migration 0009 ops note

CH-16 ships migration 0009 which `OVERWRITE`-style redefines fields on the pre-existing `identity` table (scaffolded at migration 0001). Pre-CH-16 Surrealdb databases may have rows from the scaffold (id-only); after 0009 applies, those rows have only `id` + `created_at: string` and will fail to deserialize as the new `Identity` shape because `agent_id` is missing.

CH-16 expects no real-world pre-0009 Identity rows (the scaffold was id-only and never had production writers). If the operator's database has stray Identity rows from manual experimentation, run:

```sql
DELETE FROM identity;
```

before applying 0009. The migration is otherwise additive — no destructive cascade.

---

## Audit-event reference

Two new audit event types land at CH-16:

- **`platform.identity.created`** (Alerted class) — emitted by `apply_agent_creation` after a successful commit for an LLM agent. Pairs 1:1 with the preceding `platform.agent.created` event in the same org's hash chain.
- **`platform.identity.updated`** (Logged class) — emitted by future reactive update writers (CH-21 memory-extraction listener body, M6+ skill/rating events). The diff payload includes a `trigger` label naming which of `session_ended` / `memory_extracted` / `skill_changed` / `rating_received` caused the update.

Operators investigating a "missing Identity row" report should:

1. Search the audit log for `platform.identity.created` events with the agent_id in the diff `after`.
2. If absent: the agent was created without going through `apply_agent_creation`, OR the audit emit failed post-commit (check for `AUDIT_EMIT_FAILED` errors in the same window).
3. If present but `get_identity` returns `None`: the row was deleted (operator-driven `delete_identity` call). Search the audit log for the corresponding deletion path.

---

## Orphan policy + manual cleanup

CH-16 LEAVES QUERYABLE on archive (ADR-0038 §D38.6). Operators who need GDPR-style erasure can run:

```rust
// In a one-shot script or admin handler:
repo.delete_identity(agent_id).await?;
```

There is no `IdentityDeleted` audit event at v0.1; an operator-driven cleanup leaves no audit trail beyond the manual log of the cleanup script. Future M7b GDPR work will likely add an audit event for this.

---

## Cross-References

- ADR-0038 — Identity materialization design
- ADR-0039 — Human-Agent guard
- [`identity-node.md`](../architecture/identity-node.md) — design page
- [`identity-overview.md`](../user-guide/identity-overview.md) — operator-facing reference
- [`m5/user-guide/troubleshooting.md`](../../m5/user-guide/troubleshooting.md) — `HumanAgentHasNoIdentity` operator action
