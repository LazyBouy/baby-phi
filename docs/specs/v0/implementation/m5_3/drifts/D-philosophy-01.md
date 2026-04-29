<!-- Last verified: 2026-04-28 by Claude Code -->

# D-philosophy-01 — Agent-as-creator-and-owner of Organization and Project not modeled

## Identification
- **ID**: D-philosophy-01
- **Phase of origin**: post-CH-21 philosophy alignment audit (2026-04-28); filed under M5.3 drift catalogue.
- **Discovery source**: `core-philosophy-audit`
- **Date discovered**: 2026-04-28
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `philosophy-gap`, `ownership-model`, `provenance-chain`, `permission-engine`
- **Blocks**: D-philosophy-02 (the unified-resource-model gap presupposes the OWNS edge); future M7 admin-page-1 (Orgs CRUD) + admin-page-3 (Projects CRUD) refactor; token-economy bid-revenue routing.
- **Blocked-by**: CH-24 M5 final seal (the M5.3 carve-out opens after M5 close).

## Concept alignment
- **Concept doc(s)**: [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) (promoted from `plan/core-philosophy-check/core-philosophy.md` at the M5.3 announcement-plan seal) — claims "Agent owns Organization" and "Agent create Projects".
- **Concept claim (verbatim, user clarification 2026-04-28)**: *"An Organization as well as a Project needs to be created by an agent to maintain provenance chain. Organization also can create agents and projects who would be owned by the organization (in turn will be owned by the agent owning the organization indirectly). But agents owned by an organization may work for other orgs."* — captured in [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md) §4.1.
- **Contradiction**: Concept asserts Agent → CREATES/OWNS → Org/Project. Code has `Agent.owning_org: Option<OrgId>` (Org-owns-Agent direction) but no edge or field expressing the inverse direction. `MEMBER_OF` and `HAS_CEO` edges exist but don't carry ownership semantics. Bootstrap-claim flow at `server/src/bootstrap/claim.rs:158` creates first User+CEO+Org without an explicit ownership edge.
- **Classification**: `contradicts-concept` (foundational ownership-model gap).
- **phi-core leverage status**: `N/A` — ownership semantics are baby-phi-native governance.

## Plan vs. reality
- **Plan said**: Core Philosophy claim list (line 4 of [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md)): *"Agent owns Organization"*.
- **Reality (shipped state at HEAD post-CH-21 seal)**:
  - Edge enum at [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) (66 variants) has no `Owns` or analogous Agent→Org/Project edge.
  - `Organization` struct at [`modules/crates/domain/src/model/nodes.rs:262-334`](../../../../../../modules/crates/domain/src/model/nodes.rs) has no `created_by_agent` / `owner_agent` field.
  - `Project` struct at [`modules/crates/domain/src/model/nodes.rs:434-463`](../../../../../../modules/crates/domain/src/model/nodes.rs) has no creator/owner field.
  - Permission Check engine at [`modules/crates/domain/src/permissions/engine.rs`](../../../../../../modules/crates/domain/src/permissions/engine.rs) has no owner-grant rule (no automatic admin authority for the owner-Agent over child Org/Project).
- **Root cause**: `concept-implementation-asymmetry`. Bootstrap was modeled on User-as-platform-identity + Agent-as-org-identity without explicit Agent-creates-Org provenance modeling. Each subsequent admin handler (CH-01 disable/archive, CH-22 catalog refresh) extended the model without revisiting the gap.

## Where visible in code
- **Files**:
  - `modules/crates/domain/src/model/edges.rs` (no `Owns` variant; 66 total).
  - `modules/crates/domain/src/model/nodes.rs:262-334` (`Organization` lacks owner field).
  - `modules/crates/domain/src/model/nodes.rs:434-463` (`Project` lacks owner field).
  - `modules/crates/server/src/bootstrap/claim.rs:158` (no ownership edge emitted at first-org creation).
- **Test evidence**: No acceptance test exercises *"Agent X is the owner of Org Y → therefore X has admin authority over child Agent Z"*. The capability simply doesn't exist.
- **Grep for regression**:
  - `grep -n "Owns\b\|owner_agent\|created_by_agent" modules/crates/domain/src/model/{nodes,edges}.rs` — expect 0 hits while drift open; ≥ 3 hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**:
  1. Add `Owns` edge variant (or extend `Created` with new from/to type pair) to `EdgeKind` enum.
  2. Bootstrap-claim flow + `apply_project_creation` emit the edge inside the same compound tx.
  3. Permission Check gains an "owner-grant" rule (analogous to Template A/C/D auto-issue): the owner-Agent of an Org/Project gets `[admin, transfer]` on the container.
  4. Acceptance test: owner-Agent can disable a child Agent without an explicit grant.
- **Implementation chunk**: **CH-25** (M5.3 first chunk). New ADR (likely 0042).
- **Dependencies on other drifts**: D-philosophy-02 builds on this (the unified-resource-model gap presupposes the OWNS edge so Permission Check selectors can match).
- **Estimated effort**: ~3 engineer-days inside CH-25.
- **Risk to concept alignment if deferred further**: HIGH — every M7 admin page that lands without this gap closed encodes the bespoke-handler pattern, increasing V1 refactor cost.

## Prior documentation locations
- [`plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md) §4.1
- [`plan/core-philosophy-check/core-philosophy.md`](../../../../plan/core-philosophy-check/core-philosophy.md) (philosophy brief, post-rename)
- [`plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`](../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md) (M5.3 announcement plan archived verbatim)
- [`plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 M5.3

## Lifecycle history
- 2026-04-28 — `discovered` — surfaced by post-CH-21 philosophy alignment audit; user-confirmed as load-bearing intent; filed as drift in same session under M5.3 catalogue.
