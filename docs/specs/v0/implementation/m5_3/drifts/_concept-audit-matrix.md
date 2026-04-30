<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 Concept-audit matrix

Claim-by-claim audit of M5.3-scoped concept docs against current code. Built post-CH-21 from the philosophy alignment audit ([`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md)) with user-clarified scoring applied 2026-04-28.

The M5.1 + M5.2 concept-audit matrix at [`../../m5_1/drifts/_concept-audit-matrix.md`](../../m5_1/drifts/_concept-audit-matrix.md) tracks claims under `agent.md`, `organization.md`, `project.md`, `permissions/*`, etc. — those are NOT duplicated here. **This matrix tracks ONLY the new [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md)** (binding source promoted at the M5.3 announcement-plan seal).

## Status legend

- `honored` — concept claim implemented in code, with test evidence.
- `partially-honored` — claim partially expressed; specific carve-out documented.
- `silent-in-code` — claim not yet implemented; deferred via successor marker.
- `contradicted` — claim contradicted by code; covering drift filed.

## `concepts/core-philosophy.md`

| § | Claim | Status | Code evidence | Covering drift |
|---|---|---|---|---|
| Two Types of Agents | `AgentKind::{Human, Llm}` enum + role-validity rules | honored | `nodes.rs:225-228` + `nodes.rs:284-289` | — |
| Agent owns Organization | Agent creates Org/Project; transitive ownership through Org-owned children | contradicted | no `Owns` edge; no creator field on Organization / Project; bootstrap-claim doesn't emit ownership edge | **D-philosophy-01** (HIGH, CH-25) |
| Organization has Resources | `OWNED_BY` / `BELONGS_TO` edges + Org-scoped resource catalogue | honored | `edges.rs` + `composites_m4::ResourceBoundaries` | — |
| Two Types of Resources (Fundamental + Composite) | 9 Fundamental + 8 Composite variants with `constituents()` codified | honored | `fundamentals.rs` (9 variants) + `composites.rs:150-174` (`constituents()`) | — |
| Resources have actions | Open string vocabulary per `permissions/03-action-vocabulary.md`; inheritance via composite expansion | honored | `permissions/manifest/mod.rs:38` (`actions: Vec<String>`) + `composites.rs:150` | — |
| Org has Projects (Project = Resource Type) | Project is a Composite resource subject to Permission Check selectors | contradicted | `Composite` enum has 8 variants; no `ProjectObject` / `OrganizationObject` entries; Page-1+Page-3 handlers bypass Permission Check engine | **D-philosophy-02** (HIGH, CH-26) |
| Sub-orgs + Sub-projects | `HasSuborganization` + `HasSubproject` edges | honored | `edges.rs:244` + `edges.rs:275` | — |
| Project shared between Orgs | Shape-B project + `BelongsTo` N:N edge | honored | `edges.rs:289` + `nodes.rs:510-516` (`ProjectShape::B`) | — |
| Resource shared between Projects + cross-org via shared Project | `BelongsTo` N:N + Shape-B + `AllocatedTo` edges | honored | `edges.rs:289,334` + `permissions/06-multi-scope-consent.md` § Joint project | — |
| Org has Agents (Ownership) | `MEMBER_OF` edge + `Agent.owning_org` | honored | `edges.rs:136` + `nodes.rs:194` | — |
| Agent spawn other Agents | `DELEGATES_TO` edge + audit-event capture of creator-actor | honored (creator-foreign-key on node deliberately omitted per philosophy §4.4 clarification — access-control over creator-tracking) | `edges.rs:100` + `agents/create.rs` audit emit | — |
| Agent create Projects | `apply_project_creation` actor recorded in audit; CREATED edge emitted | partially-honored (post-CH-25 closure adds Agent→Project Owns edge) | `apply_project_creation` audit + post-CH-25 `Owns` edge | — (post-D-philosophy-01) |
| Agents own Resources | `OWNED_BY` edge + `Grant.holder` | honored | `edges.rs` + `nodes.rs:624-639` | — |
| Agents work on several Projects/Orgs | N:N `MEMBER_OF` + `HAS_AGENT` edges | honored | `edges.rs:136,254` | — |
| Resources can be Transferred + co-owned | `transfer` action + `AllocatedTo` edge | honored | `permissions/03-action-vocabulary.md` + `edges.rs:334` | — |
| Every Resource has a creator (CREATED edge) | `CREATED` edge variant + `Edge::new_created` constructor | honored | `edges.rs:329-333` + `edges.rs:597-601` | — |
| Permission Tuple (Subject, Action, Resource, Constraints, Provenance) | Grant carries 4-of-5 directly; Constraints sourced from Manifest at call-time (acceptable per philosophy §4.3 clarification — constraint provision lives on Manifest, not Grant) | honored-with-design-note | `nodes.rs:624-639` + `permissions/engine.rs` Step 4 | — |
| Capability semantics (Permission Check) | 6+2a-stage algorithm: catalogue → expand → resolve grants → ceiling → match → scope → constraints → consent | honored | `permissions/engine.rs:38-121` | — |
| Session shared ownership (Org + Project + Agent) | `Session.owning_org` + `owning_project` + `started_by` + frozen tags + Templates A-D auto-issue grants | honored | `nodes.rs:998-1018` + `permissions/05-memory-sessions.md:218-290` | — |
| Memory shared ownership (private vs Org/Project, inherited from Session) | Tag-based inheritance via `session:{id}` / `org:{id}` / `project:{id}` derived in CH-21 listener; binary `{private, public}` bucket on `extraction_scope_distribution`; 4-pool routing deferred to M6-DEFERRED-04 | partially-honored | CH-21 / ADR-0040 §D40.2 + `events/listeners.rs` `build_memory_tags` | — (M6-DEFERRED-04 owns the 4-pool upgrade) |

## Cross-reference

- M5.1 + M5.2 matrix (claims under `agent.md`, `organization.md`, `project.md`, `permissions/*`): [`../../m5_1/drifts/_concept-audit-matrix.md`](../../m5_1/drifts/_concept-audit-matrix.md).
- Drift schema: [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md).
- Philosophy alignment audit + user clarifications: [`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md).
- M5.3 announcement plan archive (verbatim): [`../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`](../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md).
