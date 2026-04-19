# baby-phi v0 — Concept Documents

> Each file expands one concept from [brainstorm.md](../brainstorm.md).
> Status: All concepts are `CONCEPTUAL` — ideas, not specs.

## Index

| Concept | File | Description |
|---------|------|-------------|
| **Agent** | [agent.md](agent.md) | Agent anatomy: Soul (immutable), Power (tools/skills), Experience, Identity (emergent). Contract vs Worker modes. |
| **Human Agent** | [human-agent.md](human-agent.md) | Human participants as Agents without models. Channel routing (Slack, email, web UI). |
| **Ontology** | [ontology.md](ontology.md) | Core data model: all node types, edge types, value objects, schema registry. |
| **Token Economy** | [token-economy.md](token-economy.md) | Token currency flow, bidding, Worth (reputation), Value (market price), Meaning. |
| **Project** | [project.md](project.md) | Project, Task, Bid, Rating — the work hierarchy. |
| **Organization** | [organization.md](organization.md) | Organization structure, Market (future placeholder). |
| **Permissions** | [permissions.md](permissions/README.md) | Capability-based permission model: resource ontology, action vocabulary, constraints, tool manifests, resolution hierarchy. |
| **Coordination** | [coordination.md](coordination.md) | Multi-agent patterns: blackboard, event-driven, messaging, orchestrator. Future scenarios from phi-core. |
| **phi-core Mapping** | [phi-core-mapping.md](phi-core-mapping.md) | Complete mapping of all phi-core types to the ontology (nodes, value objects, runtime-only). |
| **System Agents** | [system-agents.md](system-agents.md) | Standard System Agents included in every org by default: `memory-extraction-agent` and `agent-catalog-agent`. Plus future additions. |

## Reference Catalogues

Ten worked organization layouts and five worked project layouts. Illustrative, not normative — each demonstrates a valid configuration under the concept spec.

| Catalogue | Folder | Contents |
|-----------|--------|----------|
| **Organization Layouts** | [../organizations/](../organizations/README.md) | 10 org examples spanning: minimal startup, mid product team, strict consultancy, regulated enterprise, nested research lab, joint venture (2 sides), marketplace gig, education org, platform infrastructure. |
| **Project Layouts** | [../projects/](../projects/README.md) | 5 project examples spanning: flat single project, deeply-nested sub-projects, Shape B joint project, market-bid flow, long-duration compliance audit. |
| **Requirements** | [../requirements/](../requirements/README.md) | Derived from the concept via the fresh-install admin journey. 15 admin pages + 5 agent-self-service pages + 6 system flows + 4 NFRs + traceability matrix. |

## How to Use

- **brainstorm.md** remains the single-document overview with all concepts together
- **Concept files** allow independent expansion of each area without the brainstorm growing unbounded
- Cross-references link related concepts
- When a concept solidifies, it graduates from `CONCEPTUAL` to a proper spec
