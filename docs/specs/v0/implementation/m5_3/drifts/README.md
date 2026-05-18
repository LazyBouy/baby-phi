<!-- Last verified: 2026-05-18 by Claude Code (CH-27 P-SEAL — D-CH26-FOLLOWUP-01 transitions discovered → remediated at wire-tier + synth-grant-widening + acceptance-fixture-extension axes via ADR-0062 §D62.1+§D62.2+§D62.4; resolvers-wiring axis deferred to M6 via NEW D-CH27-FOLLOWUP-01 (M6-DEFERRED-RESOLVERS-WIRING). Open count 1 → 1 (D-CH26-FOLLOWUP-01 closed; D-CH27-FOLLOWUP-01 opened at discovered); remediated count 2 → 3. M5.3 carve-out closes with 3-chunk arc {CH-25, CH-26, CH-27}; M6 plan-open unblocks. Cycle hex `0edcaba9`.) -->
<!-- Last verified: 2026-05-17 by Claude Code (CH-26 P-SEAL — D-philosophy-02 transitions discovered → remediated at load-bearing semantic axis; NEW drift D-CH26-FOLLOWUP-01 filed at discovered (Bucket B, LOW, closing chunk CH-27 — M5 carve-out extension); open count 1 → 1 (same count: D-philosophy-02 closed, D-CH26-FOLLOWUP-01 opened); remediated count 1 → 2. Cycle hex `d1cb9e1f`.) -->
<!-- Last verified: 2026-05-16 by Claude Code (CH-25 P-SEAL — D-philosophy-01 transitions discovered → remediated; open count 2 → 1; remediated count 0 → 1. Cycle hex `1e01618e`.) -->
<!-- Last verified: 2026-04-28 by Claude Code -->

# M5.3 Drift catalogue — index

This directory tracks drifts surfaced **post-M5-close** that close at M5.3 (carve-out between CH-24 final M5 seal and M6 plan-open). It is the **single source of truth for M5.3 drifts**, parallel to (and discoverable alongside) [`../../m5_1/drifts/README.md`](../../m5_1/drifts/README.md) which tracks M5.1 + M5.2 drifts (60 entries spanning M5/P2 concept-audit discoveries + the M5 ledger).

**Status (as of 2026-05-18 / CH-27 P-SEAL):**
- Drifts open: **1** (D-CH27-FOLLOWUP-01 — M6-DEFERRED-RESOLVERS-WIRING; resolver actor-passthrough architectural design).
- Drifts remediated: **3** (D-philosophy-01 — CH-25 ✓ cycle hex `1e01618e`; D-philosophy-02 — CH-26 ✓ cycle hex `d1cb9e1f` at load-bearing semantic axis; D-CH26-FOLLOWUP-01 — CH-27 ✓ cycle hex `0edcaba9` at wire-tier + synth-grant + acceptance-fixture axes).
- **M5.3 carve-out closed**: 3-chunk arc {CH-25, CH-26, CH-27} complete. M6 plan-open unblocks.

## Severity + Bucket summary

| Severity | Bucket | Count | IDs |
|---|---|---|---|
| HIGH | A (load-bearing scope gap) | 2 | D-philosophy-01 (remediated), D-philosophy-02 (remediated) |
| LOW | B (advisory → blocking gate tightening + follow-on engine-scope widening) | 2 | D-CH26-FOLLOWUP-01 (remediated at CH-27), D-CH27-FOLLOWUP-01 (discovered, M6-DEFERRED) |

## Lifecycle states

Same lifecycle as the M5.1 + M5.2 catalogue per [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md):

`discovered → classified → scoped → in-chunk-plan → remediated`

## Drift table

| ID | Title | Severity | Bucket | Concept doc(s) | phi-core leverage | Status | Closes at | File |
|---|---|---|---|---|---|---|---|---|
| D-philosophy-01 | Agent-as-creator-and-owner of Org/Project not modeled (no OWNS edge, no creator field, no owner-grant rule) | **HIGH** | **A** | core-philosophy; agent; organization; project | N/A | **remediated** | **CH-25 ✓** | [D-philosophy-01.md](D-philosophy-01.md) |
| D-philosophy-02 | Org/Project not in resource ontology (no Composite::OrganizationObject / ProjectObject) | **HIGH** | **A** | core-philosophy; ontology; permissions/01 | N/A | **remediated** | **CH-26 ✓** | [D-philosophy-02.md](D-philosophy-02.md) |
| D-CH26-FOLLOWUP-01 | Advisory → blocking gate tightening + synth-grant widening + resolvers wiring + acceptance-fixture extension | LOW | B | core-philosophy; permissions/04 | N/A | **remediated** | **CH-27 ✓** (cycle hex `0edcaba9`; ADR-0062 §D62.1+§D62.2+§D62.4 closed 3-of-4 axes; resolvers-wiring axis deferred to M6 via D-CH27-FOLLOWUP-01) | [D-CH26-FOLLOWUP-01.md](D-CH26-FOLLOWUP-01.md) |
| D-CH27-FOLLOWUP-01 | `projects::resolvers::*` actor-passthrough wiring deferred to M6 (background-listener trait shape) | LOW | B | core-philosophy | N/A | discovered | **M6-DEFERRED-RESOLVERS-WIRING** (per ADR-0062 §D62.3 F3.a LOCKED) | [D-CH27-FOLLOWUP-01.md](D-CH27-FOLLOWUP-01.md) |

## Companion files

- [`_concept-audit-matrix.md`](_concept-audit-matrix.md) — claim-by-claim audit of [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) against current code (the binding source for M5.3 drifts).
- Drift bodies follow [`../../m5_1/drifts/_schema.md`](../../m5_1/drifts/_schema.md) structure.

## Cross-reference

- Philosophy alignment audit + clarifications: [`../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md`](../../../../plan/core-philosophy-check/2026-04-28-philosophy-alignment-audit.md).
- Forward-scope M5.3 carve-out: [`../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §2.5.
- M5.3 announcement plan archive (verbatim): [`../../../../plan/core-philosophy-check/m5-3-announcement-plan-525d2085.md`](../../../../plan/core-philosophy-check/m5-3-announcement-plan-525d2085.md).
