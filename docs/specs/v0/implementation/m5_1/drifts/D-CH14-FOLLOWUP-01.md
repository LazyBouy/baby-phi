<!-- Last verified: 2026-05-08 by Claude Code (filed by CH-14 chunk-seal; cycle hex `5803bb94`) -->

# D-CH14-FOLLOWUP-01 — Adoption-AR builders do not set `descends_from_grant`

## Identification
- **ID**: D-CH14-FOLLOWUP-01
- **Phase of origin**: CH-14 chunk-seal (cycle hex `5803bb94`)
- **Discovery source**: `cycle-plan-deferral` (plan §3 Artifact B scope-control decision per F3.A user-lock)
- **Date discovered**: 2026-05-08
- **Status**: `discovered`
- **Bucket**: B — concept-doc fidelity gap with chain root + walker landed; AR-to-Grant link wiring deferred
- **Severity**: LOW
- **Tags**: `authority-chain`, `provenance-traversal`, `adoption-ar`, `forward-defensive`
- **Blocks**: nothing today (every shipped chain has depth ≤ 2 — bootstrap-AR → admin-grant; the walker terminates correctly without the AR-to-Grant link populated)
- **Blocked-by**: future M6+ chunk that introduces a new adoption flow OR explicitly wants chain-depth > 2

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"The Authority Chain" lines 510–547.
- **Concept claim**: The provenance tree forms `Bootstrap-AR ← Bootstrap-Grant → Adoption-AR ← Adoption-Grant → Fired-Grant`. Every AR's `descends_from_grant` is the grant under whose authority the AR was submitted; the bootstrap AR is the unique root with `descends_from_grant = None`.
- **Contradiction**: NONE today. CH-14 ships the typed `AuthRequest.descends_from_grant: Option<GrantId>` field with `#[serde(default)]` shielding (defaults to `None`); the bootstrap claim explicitly stays at `None` (correct — bootstrap is the chain root). Adoption-AR builders (`templates/{a,b,c,d,e}.rs` + `templates/adoption.rs`) all delegate to `templates::e::build_auto_approved_request` which currently sets `descends_from_grant: None` for every adoption AR. Today's data shape masks this gap because adoption ARs use `system:genesis` as approver, terminating the walker one hop early at the adoption AR's `is_bootstrap_ar`-style check (false at the adoption AR; the walker still terminates because `descends_from_grant = None` triggers the `None`-terminator branch).
- **Classification**: `partially-honored` (chain-root axis honored; AR-to-Grant link axis defaults to `None` everywhere — forward-defensive).
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (CH-14 plan §3 Artifact B): "Adoption-AR-side wiring is deferred to a successor chunk (`D-CH14-FOLLOWUP-01`) to keep CH-14 within ~5 days. The walker still works: at chunk close, every grant's chain reaches bootstrap via the existing `Grant.descends_from -> AR` field — the missing AR-to-Grant link only matters when the chain has > 1 AR hop (which today's data shape does not produce because adoption-ARs use `system:genesis` as approver, terminating the walk one hop early)."
- **Plan said** (CH-14 plan §1): "Forward-defensive plumbing only — no callsite outside Template-revoke is rewired."
- **Reality**: matches the plan exactly. The field exists everywhere; only the bootstrap-AR site populates it (with `None`, the chain-root value).

## Required follow-up
- **What needs to happen**: when a future M6+ chunk lands a new adoption flow OR explicitly wants to render a multi-hop authority chain (e.g., for an audit UI that draws the tree), the adoption-AR builders SHOULD plumb the firing grant's id into `descends_from_grant`.
  - **Pure-fn surface change**: `templates/adoption::build_adoption_request` + `templates/e::build_auto_approved_request` would gain a new `descends_from_grant: Option<GrantId>` parameter (defaults to `None`).
  - **Caller plumbing**: each adoption-AR call site would pass the firing grant id (typically the admin's `system:root` grant id at org-create time).
  - The 5 sites are at `templates/a.rs:44`, `templates/b.rs:24`, `templates/c.rs:33`, `templates/d.rs:33`, `templates/e.rs:114`.
- **Tests required**: walker test asserting a 2-hop chain renders correctly when the adoption AR's `descends_from_grant` is populated. The InMemoryRepository walker unit test `walker_climbs_two_hop_chain_root_to_leaf` already proves the walker handles this shape; the gap is just that no production callsite produces it.
- **Acceptance**: every adoption AR's `descends_from_grant` is populated with the firing grant's id; chain depth > 1 renders correctly at all walker callers.

## Closing chunk
- TBD — likely a future M6+ chunk that introduces a delegation-flow change OR an audit-UI tree rendering; not yet allocated in forward-scope.

## Lifecycle
- **2026-05-08 — `discovered`** — filed by CH-14 chunk-seal. CH-14 ships the typed field + walker + cascade for the chain-root axis; AR-to-Grant link wiring deferred to keep CH-14 within ~5 days per plan §3 Artifact B scope-control. Mirrors CH-13's `D-CH13-FOLLOWUP-01` pattern (chunk closes one axis of a multi-axis concept-doc claim; the other axis tracked here).

## Cross-references
- CH-14 plan: [`baby-phi/docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md`](../../../../plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md) §3 Artifact B + §F3.
- ADR-0053: [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](../../m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md) §D53.5 (scope-control decision documented).
- D-new-14: [`D-new-14.md`](D-new-14.md) (closed by CH-14 — chain-root axis).
- Sister patterns: [`D-CH13-FOLLOWUP-01.md`](D-CH13-FOLLOWUP-01.md), [`D-CH12-FOLLOWUP-01.md`](D-CH12-FOLLOWUP-01.md), [`D-CH11-FOLLOWUP-01.md`](D-CH11-FOLLOWUP-01.md), [`D-CH07-FOLLOWUP-01.md`](D-CH07-FOLLOWUP-01.md).
- Affected files: `templates/a.rs:44`, `templates/b.rs:24`, `templates/c.rs:33`, `templates/d.rs:33`, `templates/e.rs:114`, `templates/adoption.rs:69`.
