<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-09 by Claude Code -->

# Organization

> Extracted from brainstorm.md Sections 3.10-3.11, refined 2026-04-09.
> See also: [project.md](project.md), [permissions.md](permissions.md) (org-level rules and the Multi-Scope Session Access rule for joint projects)

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

Organizations sit at the top of the permission **ceiling enforcement** chain — they cap what any project or agent within them can do:

```
Organization (highest ceiling)
    │ caps ↓
Project (capped by its owning orgs)
    │ caps ↓
Agent (capped by its project + org)
```

This is **Mechanism 1** in the permission model: top-down ceiling enforcement. An org's restrictions are upper bounds on everything beneath it; a project cannot grant what its owning org has forbidden.

There is also a separate **Mechanism 2** — scope resolution — that determines which scope's grants apply when a session has multiple `org:` or `project:` tags. Scope resolution cascades the *opposite* direction (most-specific-first): project membership wins over org membership, with `base_*` as the tie-breaker, falling back to intersection for outsiders.

> **Joint projects (one project, multiple owning orgs):** When an Organization shares ownership of a project with another org, the joint project's sessions carry both `org:` tags. Each lead reads under their own org's rules (the contractor model). See [permissions.md → Multi-Scope Session Access](permissions.md#multi-scope-session-access) for the full rule and worked examples.

See [permissions.md](permissions.md) for the complete capability-based permission model, including resource ontology, action vocabulary, Authority Templates, and the Consent Policy mechanism.

---

## Market (Future Concept — Placeholder)

> **Not yet designed.** A shared space where agents post Tasks and other agents bid. The poster evaluates bids and allocates work.

The Market is where Supply (agent capability) meets Demand (task requirements). Key ideas:
- Agents can post Tasks to the Market (not just sponsors)
- Market has rules (minimum rating to bid, maximum bid amount, etc.)
- Market history provides price discovery (what similar tasks have cost)
- Could be per-Organization or cross-Organization
