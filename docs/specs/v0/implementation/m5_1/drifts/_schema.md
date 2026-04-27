<!-- Last verified: 2026-04-24 by Claude Code -->

# Drift-file schema (M5.1 canonical template)

Every drift file under [`docs/specs/v0/implementation/m5_1/drifts/`](.) is a copy of the template
below with all fields filled. Fields are mandatory unless marked optional. An
empty or unjustified field blocks the file from counting toward catalogue
coverage at the M5.1/P5 seal audit.

The template is consumed by:
- **M5.1/P1** — migrating the 29 existing drift ledger entries from
  [`plan/build/01710c13-m5-templates-system-agents-sessions.md`](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md)
  §Drift addenda into one file per drift.
- **M5.1/P2** — minting new `D-new-NN` drift files for every concept/code
  contradiction discovered by the concept-vs-implementation audit.
- **Every subsequent implementation chunk** — when a mid-chunk pause surfaces a
  new drift, the drift is born as a file in this shape.

## Template

```markdown
# D<ID> — <one-line title>

<!-- Last verified: YYYY-MM-DD by Claude Code -->

## Identification
- **ID**: D<ID>
- **Phase of origin**: M5/P<n> (existing 29) OR `concept-audit` (discovered at M5.1/P2) OR `chunk/<chunk-name>` (discovered mid-chunk post-M5.1)
- **Discovery source**: `plan-archive-ledger` | `concept-code-audit` | `user-report` | `mid-chunk-pause`
- **Date discovered**: YYYY-MM-DD
- **Status**: `discovered` | `classified` | `scoped` | `in-chunk-plan` | `remediated` | `renegotiated` | `accepted-as-is`
- **Bucket**: `A — load-bearing scope gap` | `B — underspecified shape choice` | `C — convention/pattern decision`
- **Severity**: `HIGH` | `MEDIUM` | `LOW`
- **Tags**: (optional, comma-separated) e.g. `leverage-violation`, `security-enforcement`, `persistence`, `concept-silent`, `concept-aspirational`, `cascading-upstream`
- **Blocks**: <list of other drift IDs> OR `none`
- **Blocked-by**: <list of other drift IDs> OR `none`

## Concept alignment
- **Concept doc(s)**: path + §anchor (exact — e.g. [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"enforcement at launch")
- **Concept claim (verbatim quote or close paraphrase with line cite)**: "..."
- **Contradiction**: <one-sentence summary of the gap between concept claim and shipped reality>
- **Classification**: `contradicts-concept` | `concept-silent-plan-filled-gap` | `concept-out-of-date` | `concept-aspirational`
- **phi-core leverage status** (if drift touches a phi-core type): `direct-reuse` | `wrap` | `inherit-from-snapshot` | `reject-build-native` | `leverage-violation` | `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (verbatim quote with plan-archive line-range cite, e.g. "lines 372–380"): "..."
- **Reality (shipped state at current HEAD)**: <1–2 sentences with file:line cites>
- **Root cause** (pick ONE): `cascading-upstream-deferral` | `underspecified-plan` | `implementer-unilateral-choice` | `reality-feedback-contradicted-estimate` | `concept-doc-not-consulted` | `reviewer-gap-at-phase-close`

## Where visible in code
- **File(s)**: absolute or workspace-relative paths with line ranges
- **Test evidence**: test name(s), pass/fail/skipped status; "no test exists" if absent
- **Grep for regression**: exact `grep` / `ripgrep` pattern that catches the drift reappearing after close

## Remediation scope (estimate only — detailed plan lives in the per-chunk plan later)
- **Approach (sketch)**: <broad strokes only>
- **Implementation chunk this belongs to**: <chunk name from forward-scope inventory> (filled in at M5.1/P3 or later)
- **Dependencies on other drifts**: <IDs + reason>
- **Estimated effort**: X engineer-days (broad range, to be refined at per-chunk planning)
- **Risk to concept alignment if deferred further**: <one sentence>

## Prior documentation locations (pre-M5.1)
- Plan archive lines: A–B
- Code comments: file:line (if any)
- ADR references: `none` or `ADR-NNNN`
- Other doc pointers: <none | path + §>

## Lifecycle history
- YYYY-MM-DD — `<state transition>` — <brief note>
```

## Usage rules

1. **Mandatory fields.** Every field above is mandatory unless explicitly marked
   "optional". "N/A" is valid but must be justified in a trailing parenthetical.
2. **Citations must resolve.** Every path, line range, and grep pattern must
   resolve against current HEAD at the moment the file is written. A broken cite
   blocks P5 seal.
3. **Verbatim quotes.** Concept claims and plan quotes are verbatim or close
   paraphrase; if paraphrased, add `(paraphrase)`.
4. **Status transitions are logged.** The *Lifecycle history* block appends a
   dated line on every state change. No silent transitions.
5. **No speculative severity.** `HIGH` is reserved for Bucket-A drifts that
   cascade to other drifts or violate a concept-doc enforcement claim.
   `MEDIUM` for Bucket-B shape choices with concept-doc implications.
   `LOW` for Bucket-C conventions.
6. **phi-core leverage status is non-optional** when the drift touches any
   phi-core-overlapping code surface; see
   [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) for
   the reuse boundary.
