<!-- Status: CONCEPTUAL -->

# Organization

> Extracted from brainstorm.md Sections 3.10-3.11.
> See also: [project.md](project.md), [permissions.md](permissions.md) (org-level rules)

---

## Organization (Node Type)

An Organization is a **social structure** that contains agents and projects.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `org_id` | String | Unique identifier |
| `name` | String | Organization name |
| `vision` | Option<String> | Long-term aspiration |
| `mission` | Option<String> | How the vision is pursued |
| `created_at` | DateTime | When the organization was created |

### Organization Edges

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Organization | `HAS_BOARD` | Agent | 1:N | role: sponsor/stakeholder |
| Organization | `HAS_CEO` | Agent | 1:1 | — |
| Organization | `HAS_PROJECT` | Project | 1:N | — |
| Organization | `HAS_MEMBER` | Agent | 1:N | role, joined_at |
| Organization | `HAS_SUBORGANIZATION` | Organization | 1:N | — |

### Agent Membership

| From | Edge | To | Cardinality | Properties |
|------|------|----|-------------|------------|
| Agent | `MEMBER_OF` | Organization | N:N | role, is_primary: bool |

Every agent has exactly one `is_primary: true` membership. May have additional secondary memberships.

---

## Permission Hierarchy

Organizations sit at the top of the permission resolution chain:

```
Organization config (highest authority)
    │ overrides ↓
Project config
    │ overrides ↓
Agent config (most specific)
```

See [permissions.md](permissions.md) for full details.

---

## Market (Future Concept — Placeholder)

> **Not yet designed.** A shared space where agents post Tasks and other agents bid. The poster evaluates bids and allocates work.

The Market is where Supply (agent capability) meets Demand (task requirements). Key ideas:
- Agents can post Tasks to the Market (not just sponsors)
- Market has rules (minimum rating to bid, maximum bid amount, etc.)
- Market history provides price discovery (what similar tasks have cost)
- Could be per-Organization or cross-Organization
