<!-- Last verified: 2026-05-16 by Claude Code (CH-25 P-SEAL — D-philosophy-01 transitions discovered → remediated; open count 2 → 1; remediated count 0 → 1. Cycle hex `1e01618e`.) -->
<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 Drift catalogue — index

This directory tracks drifts surfaced **post-M5-close** that close at M5.3 (carve-out between CH-24 final M5 seal and M6 plan-open). It is the **single source of truth for M5.3 drifts**, parallel to (and discoverable alongside) [`../../m5_1/drifts/README.md`](../../m5_1/drifts/README.md) which tracks M5.1 + M5.2 drifts (60 entries spanning M5/P2 concept-audit discoveries + the M5 ledger).

**Status (as of 2026-05-16 / CH-25 P-SEAL):**
- Drifts open: **1** (D-philosophy-02 — CH-26 target).
- Drifts remediated: **1** (D-philosophy-01 — CH-25 ✓ cycle hex `1e01618e`).

## Severity + Bucket summary

| Severity | Bucket | Count | IDs |
|---|---|---|---|
| HIGH | A (load-bearing scope gap) | 2 | D-philosophy-01, D-philosophy-02 |

## Lifecycle states

Same lifecycle as the M5.1 + M5.2 catalogue per [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md):

`discovered → classified → scoped → in-chunk-plan → remediated`

## Drift table

| ID | Title | Severity | Bucket | Concept doc(s) | phi-core leverage | Status | Closes at | File |
|---|---|---|---|---|---|---|---|---|
| D-philosophy-01 | Agent-as-creator-and-owner of Org/Project not modeled (no OWNS edge, no creator field, no owner-grant rule) | **HIGH** | **A** | core-philosophy; agent; organization; project | N/A | **remediated** | **CH-25 ✓** | [D-philosophy-01.md](D-philosophy-01.md) |
| D-philosophy-02 | Org/Project not in resource ontology (no Composite::OrganizationObject / ProjectObject) | **HIGH** | **A** | core-philosophy; ontology; permissions/01 | N/A | discovered | CH-26 | [D-philosophy-02.md](D-philosophy-02.md) |

## Companion files

- [`_concept-audit-matrix.md`](_concept-audit-matrix.md) — claim-by-claim audit of [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) against current code (the binding source for M5.3 drifts).
- Drift bodies follow [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md) structure.

## Cross-reference

- Philosophy alignment audit + clarifications: [`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md).
- Forward-scope M5.3 carve-out: [`../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5.
- M5.3 announcement plan archive (verbatim): [`../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md`](../../../../plan/core-philosophy-check/525d2085-m5-3-announcement-plan.md).
