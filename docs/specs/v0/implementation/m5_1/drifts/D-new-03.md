<!-- Last verified: 2026-04-28 by Claude Code -->

# D-new-03 — Selector grammar is a 4-variant enum; PEG tag-predicate DSL (tags contains/intersects/any_match/subset_of + AND/OR/NOT) is absent

## Identification
- **ID**: D-new-03
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `remediated`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `permission-engine`, `selector-grammar`, `concept-contradiction`
- **Blocks**: Multi-scope cascade (D-new-06); Memory tag-based retrieval (D-new-16); Frozen-tag enforcement (D-new-08); worked-example scenarios 4–6 cannot resolve correctly without tag-intersect/tag-any-match selectors.
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) §"PEG Grammar" + all predicate sections; [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Memory as a Resource Class" (tag-predicate selector model)
- **Concept claim (verbatim)**: Normative grammar defines predicates `tags contains T`, `tags intersects {T1,T2}`, `tags any_match pattern`, `tags subset_of set_ref`, `tags empty`, `tags non_empty` plus AND/OR/NOT logical composition.
- **Contradiction**: Shipped `Selector` enum has 4 variants — `Any` / `Exact(String)` / `Prefix(String)` / `KindTag(String)` at [`permissions/selector.rs:32-42`](../../../../../../modules/crates/domain/src/permissions/selector.rs#L32-L42). No PEG parser. No recursive descent. No tag-intersect / tag-any-match primitives. No AND/OR/NOT combinators.
- **Classification**: `contradicts-concept`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (concepts/09 + worked-example scenarios): Selectors implement full tag-predicate DSL with logical composition.
- **Reality (shipped state at current HEAD)**: Trivial enum. Matching logic does string equality + prefix match only. Cannot express "sessions tagged agent:X AND project:Y" as a single selector; cannot intersect tag sets; cannot AND/OR/NOT-compose predicates.
- **Root cause**: `concept-doc-not-consulted` — M1 permission engine shipped a simpler selector model; concept 09's PEG was written aspirationally without code follow-through.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/permissions/selector.rs:32-42`](../../../../../../modules/crates/domain/src/permissions/selector.rs#L32-L42) (enum decl); [`selector.rs`](../../../../../../modules/crates/domain/src/permissions/selector.rs) `matches()` method (simple string/tag eq)
- **Test evidence**: Selector unit tests exercise the 4 variants; no coverage for any concept-09 predicate.
- **Grep for regression**: `grep -nE "^    (Contains|Intersects|AnyMatch|SubsetOf|Empty|And|Or|Not)" modules/crates/domain/src/permissions/selector.rs` — expect 0 hits while drift open; ≥6 hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Design grammar surface (AST + parser). Add enum variants for each predicate kind. Implement `matches()` for each. Add combinator variants (And/Or/Not). Add PEG parser (likely pest crate). Update all existing grants to use the richer vocabulary where applicable. Big design + impl effort.
- **Implementation chunk this belongs to**: CH-06
- **Dependencies on other drifts**: D-new-06 (multi-scope cascade uses these selectors); D-new-07 (manifest validator checks selector syntax); D-new-08 (frozen-tag enforcement uses the grammar)
- **Estimated effort**: 5–7 engineer-days (grammar design, parser, matcher, test suite, grant migration).
- **Risk to concept alignment if deferred further**: HIGH — selector grammar is foundational to permission-matching; without it, multi-scope + memory + several worked-example scenarios silently fail.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none — not in M5 drift ledger)
- Code comments: none flagging the discrepancy
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report + user-verification of enum variants)
- 2026-04-24 — `classified` — Bucket A HIGH; PEG grammar absent vs concept-09 lines 58-102 normative PEG; 6 predicates + 3 combinators + parens missing (backfill)
- 2026-04-24 — `scoped` — assigned to CH-06 per [forward-scope §1 line 77-82](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (backfill)
- 2026-04-28 — `in-chunk-plan` — CH-06 plan approved ([`build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md`](../../../../plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md)); pest-PEG grammar + 6 predicates (`contains`/`intersects`/`any_match`/`subset_of`/`empty`/`non_empty`) + 3 combinators (`AND`/`OR`/`NOT`) + parens + precedence in scope; user-decided forks: pest library + unified chunk + keep+extend backwards-compat + semantic continuity for grants
- 2026-04-28 — `remediated` — CH-06 chunk-seal; pest grammar shipped at [`modules/crates/domain/src/permissions/grammar.pest`](../../../../../../modules/crates/domain/src/permissions/grammar.pest) with 13 productions matching concept-09 lines 58-102; `Selector` enum extended with `Bool(Box<BoolExpr>)` + `Pred(Predicate)`; `parse_selector` (strict) + `parse_selector_or_uri` (legacy fast-path) public; evaluator + `SetRefRegistry` trait + `NOOP_SET_REF_REGISTRY` `'static` singleton; engine.rs:286 threads registry through `effective_evaluate`; 4 worked-parse golden ASTs + 6 predicate match + 6 combinator + 5 parse-error code (P-001..P-005) + 256-case proptest pin the grammar; `check-phi-core-reuse.sh` exit 0 (zero phi-core imports). ADR-0036 §D36.1–D36.5 records the design decisions.
