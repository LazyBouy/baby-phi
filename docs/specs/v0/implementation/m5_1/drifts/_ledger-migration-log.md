<!-- Last verified: 2026-04-24 by Claude Code -->

# M5.1/P1 — Ledger → file migration log

Every discrepancy, inaccuracy, or mis-classification discovered while
migrating the 29 drift addendum entries from the plan archive
([`plan/build/01710c13-m5-templates-system-agents-sessions.md`](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md)
§Drift addenda) into per-file entries under
[`m5_1/drifts/`](.) is logged here. Purpose: preserve the original
plan-archive text as-is while capturing every deviation the per-file
re-articulation introduced, plus any meta-inaccuracies discovered along
the way.

## Entry 0 — The "24 drifts" meta-drift (planning phase)

- **Discovered**: 2026-04-24 during M5.1 planning (pre-P0).
- **Context**: Throughout the M5.1 plan-v1 drafting + AskUserQuestion
  rounds I repeatedly referred to "24 drifts". The correct count from
  the plan archive is **29** drift items (D1.1, D1.2, D1.3, D2.1, D2.2,
  D3.1–D3.4, D4.1–D4.6, D5.1–D5.3, D6.1–D6.5, D7.1–D7.6 = 3 + 2 + 4 + 6
  + 3 + 5 + 6 = 29).
- **Verification**: `grep -cE "^### D[0-9]+\.[0-9]+ " plan/build/01710c13-m5-templates-system-agents-sessions.md` returns 29.
- **Root cause**: The first Explore-agent's Bucket breakdown summed Bucket A (6) + Bucket B (7) + Bucket C (17) = 30 (with D4.1 double-counted between buckets). I carried a miscount forward without re-summing.
- **Consequence**: M5.1 plan-v2 says "all 24 drifts" in its principle text; that number is wrong everywhere it appears. Kept the plan file as-is (approved version) but noted the real count here. The M5.1/P5 seal audit will confirm 29.
- **Demonstrates**: The catalogue discipline is working — an inaccuracy
  the aggregate ledger format hid surfaced immediately when the index
  was forced to enumerate per-file rows.

## Entry 1 — D5.1 status flipped from `discovered` to `remediated` at catalogue-time

- **Plan-archive ledger text (P5 drift addendum, line 1303)** describes
  the deferral but doesn't note resolution because the text was written
  at P5 close — before P7 shipped the CLI + Web.
- **Catalogue update**: At M5.1/P1 the drift file's `Status:
  remediated` reflects current HEAD where
  [`cli/src/commands/template.rs`](../../../../../../modules/crates/cli/src/commands/template.rs)
  and `modules/web/app/(admin)/organizations/[id]/templates/` both
  exist and are exercised by P7 completion-regression.
- **Forward implication**: No implementation chunk needs to target
  D5.1. Severity downgraded from "MEDIUM (as deferral)" to "MEDIUM
  (historical)".

## Entry 2 — D6.2 status flipped from `discovered` to `remediated` at catalogue-time

- Same pattern as Entry 1: the P6 drift text was written at P6 close;
  P7 subsequently landed `cli/src/commands/system_agent.rs` +
  `modules/web/app/(admin)/organizations/[id]/system-agents/`. Catalogue
  `Status: remediated` reflects current HEAD.
- **Forward implication**: No implementation chunk needs to target
  D6.2. Severity downgraded.

## Entry 3 — D4.1 + D4.2 severity re-assessed from ledger framing

- **Plan-archive ledger framing**: D4.1 is labelled "advisory" (reads
  like a hygiene choice) and D4.2 is labelled "synthetic feeder"
  (reads like a temporary stub). Both ledger entries are technically
  accurate.
- **Catalogue framing**: Both re-classified as **Bucket A HIGH** with
  `contradicts-concept` classification. The concept docs
  ([`permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md)
  for D4.1, [`phi-core-mapping.md`](../../../concepts/phi-core-mapping.md)
  + [`permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md)
  for D4.2) name invariants the shipped code does not honor.
- **Forward implication**: Both go into top-priority implementation
  chunks in forward-scope §1.

## Entry 4 — D6.1 severity re-assessed from ledger framing

- **Plan-archive ledger framing**: Helper shipped, call sites deferred
  to P8; framed as natural-deferral.
- **Catalogue framing**: **Bucket A HIGH** with `contradicts-concept`
  classification against
  [`concepts/system-agents.md`](../../../concepts/system-agents.md)
  §"runtime status telemetry". At M5 close the runtime-status table is
  empty for every org → page 13 list endpoint returns empty tiles →
  concept's "live queue-depth + last-fired-at" surface is an empty
  promise.
- **Forward implication**: D6.1 belongs to M5.2/P8b's scope (natural
  close with AgentCatalogListener body). The severity re-assessment
  does NOT change sequencing; it changes the description of what
  "closing at P8" means — it's not hygiene, it's restoring
  concept-mandated observability.

## Entry 5 — D6.5 severity re-assessed from ledger framing

- **Plan-archive ledger framing**: "audit + gate shipped; durable
  state flip deferred to M6" — reads like a minor schema follow-up.
- **Catalogue framing**: **Bucket A HIGH** with `contradicts-concept`
  against
  [`concepts/agent.md`](../../../concepts/agent.md) §"agent lifecycle" +
  [`concepts/system-agents.md`](../../../concepts/system-agents.md)
  §"operator can disable — pauses trigger subscriber". The concept's
  three-state lifecycle (active / disabled / archived) has no
  persistence; R-ADMIN-13-W3/W4 wire contracts return 200 on paper but
  don't change durable state.
- **Forward implication**: D6.5 becomes its own implementation chunk
  (~1.5 days) or piggybacks on a broader agent-lifecycle chunk. Severity
  re-assessment surfaces this as a first-tier remediation candidate.

## Entry 6 — Doc-links regex + path-depth corrections at scaffold-time

- **Discovered**: Mid-P1 during CI-guard run.
- **What happened**: Initial drift files used `../../../../../modules/`
  (5 parents up) but the drift-files live at `docs/specs/v0/implementation/m5_1/drifts/`
  which requires 6 parents to reach `baby-phi/modules/`. Fixed via
  bulk sed. Separately, markdown link URLs containing `(admin)` tripped
  the doc-links regex (parens close the URL early); converted those
  to code-formatted paths (backticks, no link target).
- **Forward implication**: Future drift-file authors use 6-up relative
  paths and keep `(admin)`/`[id]` path segments in code-format to avoid
  regex collision.

## Entry 7 — Last-verified header position

- **Discovered**: Mid-P1 during first CI-guard run.
- **What happened**: Initial drift files had the title on line 1 and
  the `<!-- Last verified: YYYY-MM-DD by Claude Code -->` header on
  line 3. The check-doc-links.sh regex requires the header on **line 1**
  (matches existing M5 convention). Fixed via bulk awk swap.
- **Forward implication**: Template canonicalised at this header-on-line-1
  placement. See [`_schema.md`](_schema.md).
