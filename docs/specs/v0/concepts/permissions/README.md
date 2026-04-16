<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — this is the index; see the section map below -->

# Permissions Model (Capability-Based) — Index

> The permissions spec is split across nine files. This `README.md` is the landing page. Start here; the eight content files below hold the actual spec.
>
> See also: [agent.md](../agent.md), [organization.md](../organization.md) (resolution hierarchy), [project.md](../project.md) (project-level rules), [ontology.md](../ontology.md) (nodes + edges)

## What lives where

| File | What it covers |
|------|----------------|
| [README.md](README.md) | You are here. High-level framing: Core Insight, Canonical Shape, The Five Components, plus the section map. |
| [01-resource-ontology.md](01-resource-ontology.md) | Two-tier resource ontology (9 fundamentals + 6 composites). Resource Ownership (Creation / Transfer / Shared Allocation). Composite Identity Tags (`#kind:` + `{kind}:{id}`). |
| [02-auth-request.md](02-auth-request.md) | Auth Request Lifecycle — the composite that mediates Grant creation. Schema, per-resource slot model, state machine, per-state access matrix, routing, multi-approver dynamics, `allocate` scope semantics, retention, System Bootstrap Template. |
| [03-action-vocabulary.md](03-action-vocabulary.md) | Standard Action Vocabulary (10 categories), `allocate` + `transfer` as Authority-category actions, Constraints, Selector vs Constraints distinction, Per-Resource-Class Reference. |
| [04-manifest-and-resolution.md](04-manifest-and-resolution.md) | Two Shapes: Tool Authority Manifest vs Grant. Permission Check (runtime reconciliation). Permission Resolution Hierarchy (ceilings + scope resolution). Authority Chain. Grant as a Graph Node. |
| [05-memory-sessions.md](05-memory-sessions.md) | Memory as a Resource Class. Sessions as a Tagged Resource. Authority Templates (A–E definitions). Specific Considerations for Sessions. |
| [06-multi-scope-consent.md](06-multi-scope-consent.md) | Multi-Scope Session Access (Shapes A–E, cascade rule, Co-Ownership × Multi-Scope interaction). Consent Policy (Implicit / One-Time / Per-Session) + Consent Lifecycle. |
| [07-templates-and-tools.md](07-templates-and-tools.md) | Standard Permission Templates (A–E with full YAML). `audit_class` composition rule. Tool Authority Manifest Examples (14 tools). Authoring a Tool Manifest: A Guide for Tool Creators. |
| [08-worked-example.md](08-worked-example.md) | Worked End-to-End Use Case (3 orgs, 12 projects, 14 agents). phi-core Extension Points. Open Questions. |
| [09-selector-grammar.md](09-selector-grammar.md) | Normative PEG grammar for the tag-predicate DSL used in `resource.selector` and `tag_predicate` constraints. Reference artefact for parser implementers; not in the main reading order. |

## Reading order

- **Learning the model the first time:** README → 01 → 02 → 03 → 04 → 05 → 06 → 07 → 08.
- **Authoring a new tool:** jump to [07 § Authoring a Tool Manifest](07-templates-and-tools.md#authoring-a-tool-manifest-a-guide-for-tool-creators).
- **Setting up an org:** jump to [07 § Standard Permission Templates](07-templates-and-tools.md#standard-permission-templates).
- **Looking up a specific rule:** use the Index above to find the right file; use the file-local TOC (Ctrl-F `## `) within each file.

---

# Permissions Model (Capability-Based)

> Extracted from brainstorm.md Section 4 + permission_model.md. Refined 2026-04-15 with two-tier ontology (fundamentals + composites), `tag` fundamental, `#kind:` identity tags, publish-time manifest validation, Standard Permission Templates, Tool Authority Manifest Examples Catalog, Manifest Authoring Guide, Resource Ownership / Transfer Concepts built with Auth Request composit and an end-to-end worked use case.
> See also: [agent.md](agent.md), [organization.md](organization.md) (resolution hierarchy), [project.md](project.md) (project-level rules)

---

## Core Insight

**Permissions are not about tools — they're about actions on resources with constraints.**

This mirrors capability-based security and cloud IAM: authority is tied to a specific object and operation, not ambient possession of a broad tool. Tools are merely *implementations* of actions on resources — which is why Tool Authority Manifests (defined in [04-manifest-and-resolution.md](04-manifest-and-resolution.md)) declare *what actions the tool performs on what resources*, making the five-tuple the common currency on both the tool-description side and the grant side. The Core Insight and the Tool Manifest machinery are the same idea seen from the tool-author's vs the permission-issuer's perspective.

## Canonical Shape

```
Permission = ⟨ subject, action, resource, constraints, provenance ⟩
```

This 5-tuple is the shape of a **Grant** — a capability HELD by a specific principal. Every component answers a distinct question:

| Component | Question | Discussed In |
|-----------|----------|--------------|
| **subject** | WHO holds the capability? | The Five Components → Subject |
| **action** | WHAT operation is permitted? | Standard Action Vocabulary |
| **resource** | ON WHAT is the action performed? | Resource Ontology |
| **constraints** | UNDER WHAT CONDITIONS does the permission apply? | Constraints |
| **provenance** | WHO granted this and HOW? | The Five Components → Provenance |

> **Important distinction:** The 5-tuple describes a **Grant** (what an agent IS allowed to do). It is **not** the same as a **Tool Authority Manifest** (what a tool REQUIRES from its caller). The two are reconciled at runtime by a Permission Check. See the three sections below for each shape and a worked example showing how they interact.

---

## The Five Components

### Subject — WHO

The subject is the **principal** on whom the authority is conferred. In baby-phi, the subject is **implicit in the graph edge** that connects the principal to the Grant node — it is not stored as a property on the Grant node itself.

```
Agent ──HOLDS_GRANT──▶ Permission     -- the source of the edge IS the subject
Project ──HOLDS_GRANT──▶ Permission   -- project-scoped grant
Organization ──HOLDS_GRANT──▶ Permission  -- org-level ceiling
```

**Examples of subjects:**

| Subject | Meaning |
|---------|---------|
| `agent:claude-coder-7` | A specific agent instance |
| `project:website-redesign` | A project (propagates to all member agents) |
| `org:acme-corp` | An organization (topmost ceiling for everything inside) |
| `role:lead@website-redesign` | Anyone holding the lead role within a project |
| `system` | The platform itself (used for non-revocable bootstrap permissions) |

> **Why subject is implicit in the edge:** A Grant node is reusable — the same capability shape can be granted to multiple subjects. Storing subject as a property would force duplication of the Grant node per subject. Modeling subject as the edge source allows one Permission to be referenced by many `HOLDS_GRANT` edges.

### Provenance — WHO GRANTED IT and HOW

Provenance is the **audit trail** of authority. It records who created the grant and how.

**Examples of provenance:**

| Provenance | Meaning |
|------------|---------|
| `system` | Granted by the platform at bootstrap (e.g., default permissions for a System Agent) |
| `agent:human-sarah@2026-04-01` | Granted by a human sponsor at a specific time |
| `agent:admin-bot` | Granted by an administrative agent's explicit action |
| `config:org-default.toml#network-policy` | Declared in a config file at org setup time |
| `inherited_from:org:acme-corp` | Propagated down from an org-level grant |
| `delegated_from:agent:supervisor-3` | Passed down via a delegation chain |
| `contract:bid-4581` | Granted as part of accepting a contract bid (auto-revokes on contract end) |

**What provenance enables:**

1. **Auditing** — "Who decided this agent could read the production database?" is answerable by looking at provenance.
2. **Revocation cascades** — If a parent grant is revoked, all `inherited_from:` and `delegated_from:` grants that descend from it should also be revoked.
3. **Trust assessment** — A permission with provenance `system` is more trusted than one with provenance `agent:random-bot`. This matters for permission-to-grant-permissions decisions.
4. **Time-bounded grants** — Provenance like `contract:bid-4581` tells the system when to auto-revoke (when the contract ends).

---

