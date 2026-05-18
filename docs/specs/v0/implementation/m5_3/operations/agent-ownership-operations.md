<!-- Last verified: 2026-05-18 by Claude Code (CH-27 / ADR-0062 P-DOCS — synth-grant action-vec table row + operator-troubleshooting bullet updated to reflect 4-verb scope `[Allocate, Transfer, Observe, Inspect]` per ADR-0062 §D62.2. Cycle hex `0edcaba9`.) -->
<!-- Last verified: 2026-05-16 by Claude Code (CH-25 / ADR-0060 P3 — operations page paired with the agent-ownership-model architecture page; documents how operators inspect Owns edges + read synth-owner-grants in Permission Check decision logs.) -->

# Agent ownership — operations page

> **Status:** [EXISTS] as of CH-25 (M5.3). For the architectural design read [`agent-ownership-model.md`](../architecture/agent-ownership-model.md); for the originating decision read [ADR-0060](../decisions/0060-agent-as-creator-and-owner.md).

---

## §1 — Inspecting `Owns` edges

`Owns` edges live in the SurrealDB `owns` relation table (migration `0017_add_owns_relation.surql`). Each row maps `(from: AgentId) → (to: Org or Project)` with an `EdgeId` primary key.

### CLI

The CLI exposes `Repository.list_agent_owned_orgs` + `list_agent_owned_projects` via a future ops sub-command (M6+ scope; not yet wired at M5.3 close). For now, operators query directly via SurrealQL:

```surql
-- Orgs owned by agent <agent_id>
SELECT * FROM owns WHERE in = type::thing('agents', '<agent_id>')
  AND record::tb(out) = 'organizations';

-- Projects owned by agent <agent_id>
SELECT * FROM owns WHERE in = type::thing('agents', '<agent_id>')
  AND record::tb(out) = 'projects';
```

The `record::tb()` filter pattern-matches on `OwnedResourceId` variant (Org → `organizations` table, Project → `projects` table). The in-memory backend uses an equivalent slice-filter against the in-memory `owns_edges` vector.

### HTTP

No dedicated REST endpoint at M5.3 close. The chunk's acceptance test [`server/tests/acceptance_m5_3_owner_grant.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_3_owner_grant.rs) demonstrates programmatic access via `Repository.list_agent_owned_orgs` after wizard-driven org creation. M6+ admin pages 1 (Orgs) + 3 (Projects) will surface owner relationships in dashboard payloads.

### When edges are emitted

The `Owns` edge is emitted inside two compound transactions:

| Compound tx | Emit point | `from` end | `to` end |
|---|---|---|---|
| `apply_org_creation` | After the org row + CEO agent are inserted | `payload.ceo_agent.id` | `OwnedResourceId::Org(org_id)` |
| `apply_project_creation` (Shape A) | After the project row + member agents are inserted | `payload.lead_agent_id` | `OwnedResourceId::Project(project_id)` |
| `apply_project_creation` (Shape B materialise) | After the both-approve materialisation | `payload.lead_agent_id` (lead = creator per Decision-3) | `OwnedResourceId::Project(project_id)` |

Per ADR-0060 §D60.1 + Decision-3 user-lock at P1: `lead_agent_id` IS the owner at both Shape A and Shape B materialise paths (the AR-submitter chose the lead at Shape B; that's the owner).

---

## §2 — Reading synth-owner-grants in Permission Check decision logs

The Permission Check engine's synth-owner-grant rule (per ADR-0060 §D60.3) fires inside [`step_2_resolve_grants`](../../../../../../modules/crates/domain/src/permissions/engine.rs) AFTER the 3 existing `collect()` calls for the typed grant tiers. The synth-grant is a real `Grant` value in the candidate pool — downstream Step 3 (Match), Step 4 (Constraint), Step 5 (Scope), Step 6 (Consent) all treat it identically to any persisted grant.

### Distinguishing synth vs persisted grants

Synth-owner-grants carry these distinctive fields:

| Field | Synth-owner-grant value | Persisted-grant typical value |
|---|---|---|
| `id` | freshly-generated `GrantId::new()` per check | stable id from `grants` table |
| `descends_from` | `None` (no AR provenance) | `Some(GrantId)` for template-issued grants |
| `audit_class` | `AuditClass::Silent` | `AuditClass::Logged` (or per-template) |
| `approval_mode` | `ApprovalMode::Implicit` | per-template (e.g., `PerSession` for Template C) |
| `delegable` | `true` (owners can delegate) | per-template |
| `action` | `[Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]` (4-verb universal-applicability per ADR-0062 §D62.2, widened at CH-27) | any |
| `resource.uri` | `org:<uuid>` or `project:<uuid>` | varies |

In decision logs, synth-owner-grants surface as the winning grant when:

1. The manifest reach requests any of `[Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]` (post-CH-27 / ADR-0062 §D62.2 widened scope) over the owned-Org/Project URI.
2. NO explicit grant covers the same reach.
3. The agent is the owner per `Edge::Owns`.

Operator troubleshooting tip: if a Permission Check unexpectedly **passes** for an agent that shouldn't have authority, the first check is whether the agent owns the target Org/Project via an `Owns` edge. Run the SurrealQL query in §1 to verify.

### Silent audit class rationale

Owner-grant synthesis is a **structural inference** (the engine inferred the grant from the Owns edge), not a state-changing template adoption. The synth-grant resolution itself is `AuditClass::Silent` because:

- The Owns edge emission at `apply_org/project_creation` ALREADY emits an audit event (the org/project creation's compound-tx audit row carries the provenance).
- The underlying operation that the synth-grant authorised (e.g., a child-agent disable) emits ITS OWN audit event via the operation's normal audit-emission path.
- Auditing the synth-grant resolution separately would produce a duplicate event without information gain.

This mirrors the M1-baseline pattern for implicit class-level grants (per ADR-0008 + ADR-0014).

---

## §3 — No new audit-event class

CH-25 does NOT introduce a new audit-event class. The chunk's emission surfaces:

| Surface | Audit class | Emitted via |
|---|---|---|
| `Edge::Owns` creation at `apply_org_creation` | covered by the existing `org.created` audit event | the compound-tx audit emission (unchanged) |
| `Edge::Owns` creation at `apply_project_creation` | covered by the existing `project.created` audit event | the compound-tx audit emission (unchanged) |
| Synth-grant resolution in Permission Check | `AuditClass::Silent` | the engine's existing `step_5_compose` audit-class composition (no new emit) |

This is an intentional design choice — the chunk introduces a new candidate-grant source (synth-grants from Owns edges) without changing the audit-event vocabulary. Concept-doc invariant `permissions/05-audit-vocabulary.md` event-set is preserved.

---

## §4 — Migration `0017_add_owns_relation.surql`

The SurrealDB backend requires explicit `DEFINE TABLE owns TYPE RELATION` declaration (per A4 axis pre-flight verification at P0). The migration is **additive** (no schema change on existing rows; new relation table only) and **idempotent re-run safe** (the `IF NOT EXISTS` clause prevents double-declaration).

```surql
DEFINE TABLE owns TYPE RELATION FROM agents TO organizations | projects;
```

The `FROM agents TO organizations | projects` clause encodes the `OwnedResourceId` enum's closed 2-variant shape at the schema level — the SurrealDB layer rejects RELATE statements with wrong-typed endpoints.

Operators do NOT need to manually run this migration — the migration runner (per ADR-0012) auto-applies at server startup. The migration test `store/tests/migrations_test.rs` asserts the 0017 migration is present + applied.

---

## §5 — Cross-references

- **Architecture page**: [`agent-ownership-model.md`](../architecture/agent-ownership-model.md).
- **ADR-0060**: [`0060-agent-as-creator-and-owner.md`](../decisions/0060-agent-as-creator-and-owner.md).
- **Drift D-philosophy-01**: [`../drifts/D-philosophy-01.md`](../drifts/D-philosophy-01.md).
- **Acceptance test**: [`server/tests/acceptance_m5_3_owner_grant.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_3_owner_grant.rs).
