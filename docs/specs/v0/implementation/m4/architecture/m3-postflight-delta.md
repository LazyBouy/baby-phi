<!-- Last verified: 2026-04-22 by Claude Code -->

# M3 post-flight delta (M4/P0)

**Status: [EXISTS]** — audit run at M4/P0 open.

Confirms M3→M4 boundary state before P1 opens. Every item tagged
`still-valid` (state confirmed as expected by plan), `stale` (plan
assumption wrong; needs pre-P1 remediation), or `missing` (plan-
declared gap confirmed open).

## 10-item audit

| # | Item | State | Evidence | Fix location |
|---|---|---|---|---|
| 1 | Edge-ontology count at M3 close | still-valid | `domain/src/model/edges.rs:607` asserts `EDGE_KIND_NAMES.len() == 67` | P1 grows count only if new edges added (see item 2) |
| 2 | Project-edge surface (`HAS_SPONSOR` / `HAS_AGENT` / `HAS_LEAD` / `HAS_TASK` / `BELONGS_TO`) | still-valid — 5 of 5 exist | `domain/src/model/edges.rs:247-274` — all five variants present with correct `from: ProjectId` shape | No P1 work needed for these |
| 2b | `HAS_SUBPROJECT` + `HAS_CONFIG` edges | **missing** | Grep returns 0 hits in `edges.rs`; concept doc `project.md §Project Edges` lists both | P1 adds both variants; `EDGE_KIND_NAMES` bumps to **69** |
| 3 | `HasLead` edge — production writes | still-valid as expected gap | Only references: enum variant definition (`edges.rs:260`) + variant-name arm (`edges.rs:443`) + docstring in `templates/a.rs:22`. **Zero production writes** — variant was pre-wired at M3/P1 for Template A constructor | P3 compound tx `apply_project_creation` writes the edge |
| 4 | `AgentRole` discriminator | **missing** (confirmed expected gap) | Grep: only reference is a TODO comment at `server/src/platform/orgs/dashboard.rs:134` noting the field is needed at M4 | P1 adds `AgentRole` enum + `Agent.role: Option<AgentRole>` field |
| 5 | `Project` struct + `ProjectShape` + `ProjectStatus` | **missing** | `grep 'pub struct Project' modules/crates/domain/src` returns 0 hits; `list_projects_in_org` returns `Vec<ProjectId>` stub | P1 adds all three + migration 0004's `project` table |
| 6 | `agent_execution_limits` table / `AgentExecutionLimitsOverride` | **missing** | Absent from `store/migrations/*.surql`; absent from `domain/src/model/composites_m3.rs` | P1 adds composite (wraps `phi_core::ExecutionLimits`) + migration 0004 adds table |
| 7 | Domain event bus (`EventBus` trait, `DomainEvent`, `InProcessEventBus`, `TemplateAFireListener`) | **missing** (confirmed expected gap per user decision D-M4-4) | Grep across workspace returns 0 hits | P1 adds `domain/src/events/` + ADR-0028; P3 wires listener + `AppState.event_bus` |
| 8 | `parallelize` field on `AgentProfile` | still-valid | M3/P1 shipped; `nodes.rs::AgentProfile.parallelize: u32` is present. **Enforcement absent**: no create/edit validation, no session-start gate (M5). | P5 validates `1 <= parallelize <= org_cap` on create + edit; M5 gates at session-start |
| 9 | Dashboard carryover counters (`AgentsSummary`, `ProjectsSummary`) | still-valid as zero-value placeholders | `dashboard.rs` returns `agents_summary.llm` + `.human` only (no role split); `projects_summary.active == list_projects_in_org(...).len()` which is 0 (SurrealDB stub returns empty vec); `shape_a` / `shape_b` always 0 | P8 retroactive rewrite after P1/P2 data is wired |
| 10 | M3 carryovers in base plan §M4 | still-valid | `36d0c6c5-build-plan-v01.md` contains `#### Carryovers from M3 — must-pick-up at M4 detailed planning` with C-M4-1 through C-M4-6 intact | P0 ADDS M5-section (C-M5-3/4) + M8-section (C-M8-1) per user decision; existing M4-section carryovers addressed during P1–P8 |

## Summary

- **0 stale** — plan assumptions confirmed on every item.
- **4 missing** — all four are plan-declared gaps (items 2b, 4, 5, 6, 7). They are addressed in P1 per the plan.
- **6 still-valid** — state as expected.

Since **0 items are stale**, P1 opens directly (no P0.5 remediation phase required per the plan's P0 close criterion).

## New-scope items from user decisions at plan close

These were user-added at M4 planning close; P0 fixes the plan documents so they don't slip:

- Concept-doc amendment for 6-variant `AgentRole` — addressed by P0 deliverable #3.
- Base plan M5 / M8 carryover entries — addressed by P0 deliverable #4.

## Cross-references

- [M4 plan archive](../../../../plan/build/a634be65-m4-agents-and-projects.md) §P0 deliverables.
- [M3 plan archive](../../../../plan/build/563945fe-m3-organization-creation.md) §Carryovers.
- [phi-core leverage checklist](../../m3/architecture/phi-core-leverage-checklist.md) — four-tier enforcement model applied per M4 phase.
