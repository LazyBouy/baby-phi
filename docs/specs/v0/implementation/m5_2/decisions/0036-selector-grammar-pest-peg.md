<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0036 — Selector grammar adopts pest PEG; preserve enum-based legacy URI shapes via grammar fast-path

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-06
**Closes:** D-new-03 (HIGH) — selector grammar PEG + tag-predicate DSL

---

## Context

Concept doc [`permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) §"PEG Grammar" (lines 58–102) defines a normative tag-predicate DSL with 13 productions, six predicates (`contains` / `intersects` / `any_match` / `subset_of` / `empty` / `non_empty`), three combinators (`AND` / `OR` / `NOT`), parenthesised grouping, and explicit operator precedence (NOT > AND > OR). The shipped state at chunk-open was a 4-variant `Selector` enum (`Any` / `Exact` / `Prefix` / `KindTag`) with a hand-rolled URI parser — none of the predicates, combinators, or grouping were expressible.

Drift D-new-03 elevated this to HIGH because:

- Worked-example scenarios 4–6 in concept-09 cannot resolve correctly without `tags intersects` and `tags any_match` predicates.
- Multi-scope cascade (CH-07), frozen-tag enforcement (CH-12), and Memory tag-based retrieval (CH-15 / M6-DEFERRED-01) all depend on selector matching being able to express tag-set composition.
- The 9 grant-mint call sites (`secrets/add`, `mcp_servers/register`, `model_providers/register`, `orgs/create`, `secrets/reveal` test fixtures, `projects/create`, `mcp_servers/{patch_tenants,archive}`, `model_providers/archive`, `defaults/put`) write opaque instance-URI strings into `ResourceRef.uri: String`. Any data-migration approach to a new grammar would have to rewrite those rows.

## Decision

### D36.1 — Parser library: pest

Adopt the `pest` crate (v2) for the selector grammar parser. The grammar lives at [`modules/crates/domain/src/permissions/grammar.pest`](../../../../../../modules/crates/domain/src/permissions/grammar.pest) and is compiled at build time via `pest_derive::Parser` — a 1:1 transcription of concept-09's 13 productions into pest's syntax. PEG ordered choice (`|` in pest) handles the otherwise-ambiguous overlap between reserved tags (`#kind:session`) and namespace tags (`session:s-9831`) deterministically, matching concept-09 line 106 ("PEG's commit-on-first-match semantics is exactly the right tool here").

Alternatives considered:

- **`nom`** — combinator-based, no grammar file. Rejected because the grammar text in concept-09 is the spec; we want operators reading the parser to verify it production-by-production against the doc, not reconstruct it from combinator chains.
- **`lalrpop`** — LALR(1) parser generator. Rejected because the grammar is intentionally PEG-shaped (ordered-choice for reserved-tag disambiguation); LALR(1) requires factored rewrites that drift from the concept doc.
- **Hand-rolled recursive descent** — reviewed but rejected as the parser would still need to be hand-aligned with concept-09 every time the grammar evolves; pest's `.pest` file is the alignment.

### D36.2 — `Selector` becomes an extended enum (keep + extend)

Rather than rewrite `Selector` as a struct wrapping a new AST type, the existing enum is extended with two new variants:

```rust
pub enum Selector {
    // Legacy (M1) — preserved verbatim for the 9 grant-mint call sites.
    Any,
    Exact(String),
    Prefix(String),
    KindTag(String),
    // New (CH-06) — full PEG DSL.
    Bool(Box<BoolExpr>),
    Pred(Predicate),
}
```

`BoolExpr`, `Predicate`, `Tag`, and `SetRef` are new public types in [`selector.rs`](../../../../../../modules/crates/domain/src/permissions/selector.rs). The 8 existing unit tests continue to operate on the legacy variants unchanged; the M1 wire format for grant URIs remains valid.

Rationale: keep+extend preserves all 9 grant-mint call sites (forks 5-A in plan §1) without code changes, lets the `parse_selector_or_uri` fast-path keep returning the legacy variants for legacy shapes, and avoids the need to migrate data. The new variants encode richer-grammar selectors.

### D36.3 — `SetRefRegistry` shipped as a trait; production registry deferred to CH-15

`SetRefRegistry` is a trait with one method `resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>>`. The `NoopSetRefRegistry` (and `'static` `NOOP_SET_REF_REGISTRY` singleton) is the only implementation in M5.2; all 13 [`CheckContext`](../../../../../../modules/crates/domain/src/permissions/manifest/mod.rs) construction sites borrow it. Under the noop registry, every `subset_of` predicate evaluates to `false` — the safe default.

CH-15 will land the production registry that resolves `supervisors_tagging_scope(supervisor-7)` and similar named scopes against runtime data. The trait shape is fixed in M5.2 so the swap is local to `CheckContext.set_ref_registry`.

### D36.4 — Out-of-scope per concept-09 §"Non-Normative Notes"

Explicitly out of scope for v0:

- Bracket char-classes inside globs (`org:[ab]cme`).
- Time predicates (`tags contains created_after:2026-01-01`).
- Numeric comparisons (`token_count > 1000`).
- String-content matching on tag values (regex, substring).
- Cross-instance joins (`tags contains #kind:session AND its session.duration > 1h`).

These are deferred per concept-09's own non-normative list. Future extensions are additive: add new productions; do not change the meaning of existing ones.

### D36.5 — Existing-grant policy: semantic continuity

`ResourceRef.uri: String` is unchanged. Every M1 grant-mint call site continues to write opaque URIs. The new `parse_selector_or_uri` fast-path:

1. `*` → `Selector::Any`
2. `#kind:<name>` → `Selector::KindTag(name)`
3. `<prefix>**` → `Selector::Prefix(prefix)`
4. Otherwise: try `parse_selector(input)` (full grammar); on grammar parse error, fall back to `Selector::Exact(input)`.

This preserves M1 semantics for every legacy URI while letting future grants land richer-grammar selectors in the same `uri: String` field. Zero data migration.

---

## Consequences

**Positive:**
- Selector grammar matches concept-09 production-by-production.
- 9 grant-mint call sites untouched.
- 8 M1-baseline unit tests continue to pass.
- Parser surface ready for CH-07 (multi-scope cascade), CH-12 (frozen tag enforcement), CH-15 (memory tag retrieval).

**Negative:**
- `Selector` now has 6 variants (4 legacy + Bool + Pred); pattern-match arms expand by 2 wherever Selector is matched.
- `SetRefRegistry` is a `&dyn` field on `CheckContext`, adding one borrow lifetime parameter at every construction site (mitigated by the `'static` `NOOP_SET_REF_REGISTRY`).

**Mitigations:**
- The legacy variants are kept first in the enum; new code can default-match `_ => …` for them in most non-evaluator contexts.
- Documentation pages (m5_2 architecture + operations + user-guide) explain the dual-shape encoding for operators.

---

## Cross-References

- [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) §"PEG Grammar"
- [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Formal Algorithm Step 3"
- [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Memory as Resource Class" + §"Supervisor Extraction"
- [drift D-new-03](../../m5_1/drifts/D-new-03.md)
- ADR-0034 (agent durable lifecycle — pattern: keep+extend over rewrite)
- CH-06 plan archive: [`baby-phi/docs/specs/plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md`](../../../../plan/build/ch-06-selector-grammar-peg-and-instance-tags-acd383e2.md)
