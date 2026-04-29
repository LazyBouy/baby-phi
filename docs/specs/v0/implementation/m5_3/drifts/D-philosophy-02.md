<!-- Last verified: 2026-04-28 by Claude Code -->

# D-philosophy-02 — Org/Project not represented in resource ontology (no Composite::OrganizationObject / Composite::ProjectObject)

## Identification
- **ID**: D-philosophy-02
- **Phase of origin**: post-CH-21 philosophy alignment audit (2026-04-28); filed under M5.3 drift catalogue.
- **Discovery source**: `core-philosophy-audit`
- **Date discovered**: 2026-04-28
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `philosophy-gap`, `unified-resource-model`, `composite-ontology`, `permission-engine-coverage`
- **Blocks**: future M7 admin-page-1 (Orgs CRUD) + admin-page-3 (Projects CRUD) Permission-Check-routed handlers; "transfer Org ownership" capability; system-flow consistency.
- **Blocked-by**: D-philosophy-01 (the new owner-grant rule depends on the OWNS edge from CH-25).

## Concept alignment
- **Concept doc(s)**: [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) — claim *"Organization has Projects (A Resource Type)"*; user clarification 2026-04-28: *"In a unified view should not Governance containers be part of Resources. That way it can be maintained and scaled using same Permission rules. For example, how else would you answer the question who grants other agents permission to a project or organization?"* — captured in [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md) §4.2.
- **Contradiction**: `Composite` enum at [`modules/crates/domain/src/model/composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs) has 8 variants (`ExternalServiceObject`, `ModelRuntimeObject`, `ControlPlaneObject`, `MemoryObject`, `SessionObject`, `AuthRequestObject`, `InboxObject`, `OutboxObject`). Org and Project are NOT among them. Permission Check selectors cannot target `org:O` or `project:P` as resource types — only as tags. Page-1 (Orgs admin) + Page-3 (Projects admin) handlers route around the Permission Check engine entirely.
- **Classification**: `contradicts-concept` (unified-resource-model gap).
- **phi-core leverage status**: `N/A` — resource ontology is baby-phi-native.

## Plan vs. reality
- **Plan said**: Core Philosophy claim line *"Organization has Projects (A Resource Type)"* + user clarification on unified view.
- **Reality (shipped state at HEAD post-CH-21 seal)**:
  - `Composite` enum has 8 variants; no `OrganizationObject` / `ProjectObject` entries — see [`modules/crates/domain/src/model/composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs).
  - Page-1 + Page-3 admin handlers under [`modules/crates/server/src/platform/orgs/`](../../../../../../modules/crates/server/src/platform/orgs/) + [`modules/crates/server/src/platform/projects/`](../../../../../../modules/crates/server/src/platform/projects/) carry bespoke gating logic; they do NOT call into the Permission Check engine the way the working-agent grant path does.
  - `BelongsTo` edge supports multi-org Project ownership (Shape B), but membership checks on Org/Project don't flow through Permission Check selectors.
- **Root cause**: `concept-implementation-asymmetry`. M3+M4 modeled Org/Project as governance containers (with edges + tags + bespoke handlers), distinct from the resource ontology (Composites + Fundamentals). The unified-resource-model intent was implicit in concept-`permissions/01-resource-ontology.md` but never codified for Org/Project specifically. M5+M5.2 chunks built features on top without revisiting.

## Where visible in code
- **Files**:
  - `modules/crates/domain/src/model/composites.rs` (8 Composite variants; no Org/Project entries).
  - `modules/crates/server/src/platform/orgs/*.rs` (admin handlers don't call Permission Check engine).
  - `modules/crates/server/src/platform/projects/*.rs` (same).
- **Test evidence**: No acceptance test exercises *"agent invokes Permission Check with `target_resource: Composite::OrganizationObject` and `selector: org:X`"*. The capability doesn't exist.
- **Grep for regression**:
  - `grep -n "OrganizationObject\|ProjectObject" modules/crates/domain/src/model/composites.rs` — expect 0 hits while drift open; ≥ 2 hits post-remediation.
  - `grep -rn "permission_check\|engine::check" modules/crates/server/src/platform/{orgs,projects}/` — expect 0 hits while drift open; ≥ 3 hits post-remediation (handlers route through Permission Check).

## Remediation scope (estimate only)
- **Approach (sketch)**:
  1. Add `Composite::OrganizationObject` + `Composite::ProjectObject` variants with `constituents()` (Organization → IdentityPrincipal + DataObject + Tag; Project → DataObject + Tag).
  2. Migration to backfill instance-identity tags on existing Organization + Project rows (cf CH-06 + CH-16 migration patterns).
  3. Refactor admin-page-1 (Orgs) + admin-page-3 (Projects) handlers to call Permission Check engine (replacing bespoke gates).
  4. Acceptance test: Permission Check matches `org:O` / `project:P` as resource types in selectors; admin actions like `[invite, archive, transfer]` on Org/Project route through grants.
  5. New ADR (likely 0043) ratifying the unified resource model.
- **Implementation chunk**: **CH-26** (M5.3 second chunk).
- **Dependencies on other drifts**: builds on D-philosophy-01 (the new owner-grant rule from CH-25 depends on the OWNS edge, and the new Composites need the owner-grant rule to express *"who can `[admin]` Org X"*).
- **Estimated effort**: ~3-4 engineer-days inside CH-26.
- **Risk to concept alignment if deferred further**: HIGH — every M7 admin page that ships without unified resource semantics encodes a parallel authorization model that V1 will need to harmonize.

## Prior documentation locations
- [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md) §4.2
- [`plan/core-philosophy-check/core-philosophy.md`](../../../../plan/core-philosophy-check/core-philosophy.md) (philosophy brief, post-rename)
- [`plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`](../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md) (M5.3 announcement plan archived verbatim)
- [`plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 M5.3

## Lifecycle history
- 2026-04-28 — `discovered` — surfaced by post-CH-21 philosophy alignment audit; user-confirmed as load-bearing intent; filed as drift in same session under M5.3 catalogue.
