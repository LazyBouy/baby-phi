<!-- Last verified: 2026-04-28 by Claude Code -->

# Selector grammar — design page

> **Status:** [EXISTS] as of CH-06 (M5.2). The PEG-shaped tag-predicate DSL ships in [`modules/crates/domain/src/permissions/`](../../../../../../modules/crates/domain/src/permissions/) (`selector.rs` + `grammar.pest`). For the normative grammar, read [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md).

---

## What this page covers

The Permission Check engine evaluates a target entity (URI + tag set) against grant selectors at Step 3 of the formal algorithm. Until CH-06 the selector surface was a 4-variant enum with a hand-rolled URI parser; CH-06 lights up the full PEG tag-predicate DSL from concept-09 alongside the legacy variants. This page describes:

- The AST shape and how it relates to concept-09 productions.
- The two parser entry points (`parse_selector` strict; `parse_selector_or_uri` total).
- How the evaluator interprets each predicate, including glob matching and `subset_of`.
- The `SetRefRegistry` trait and its M5.2 noop / CH-15 future-production wiring.
- Engine integration: where Step 3 calls into the evaluator.

ADR-0036 records the design decisions and rationale; this page is the operator-facing description.

---

## AST shape

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

pub enum BoolExpr {
    Or(Vec<BoolExpr>),
    And(Vec<BoolExpr>),
    Not(Box<BoolExpr>),
    Pred(Predicate),
}

pub enum Predicate {
    Contains(Tag),
    Intersects(Vec<Tag>),
    AnyMatch(String /* glob pattern */),
    SubsetOf(SetRef),
    Empty,
    NonEmpty,
}

pub enum Tag {
    Reserved(String, Option<String>),  // #ns or #ns:val
    Namespace(String, Vec<String>),    // ns:val[/val…]
    Literal(String),                   // "..."
}

pub struct SetRef { pub name: String, pub args: Vec<String> }
```

**Mapping to concept-09 productions** (1:1):

| Concept-09 production | AST type |
|---|---|
| `Selector` | top-level `Selector` |
| `OrExpr` | `BoolExpr::Or(parts)` (collapsed to inner when single-element) |
| `AndExpr` | `BoolExpr::And(parts)` (same) |
| `NotExpr` | `BoolExpr::Not(inner)` |
| `Predicate` (paren or tag) | `BoolExpr::Pred(...)` (paren returns inner OrExpr) |
| `TagPredicate` + `ContainsOp` | `Predicate` variants |
| `Tag` (Reserved/Namespace/Literal) | `Tag` variants |
| `TagSet` | `Vec<Tag>` inside `Predicate::Intersects` |
| `TagGlob` | `String` inside `Predicate::AnyMatch` |
| `SetRef` | `SetRef` struct |

Worked-parse examples 1–4 from concept-09 reproduce as golden ASTs in [`selector.rs::tests`](../../../../../../modules/crates/domain/src/permissions/selector.rs).

---

## Parser entry points

### `parse_selector(input: &str) -> Result<Selector, SelectorParseError>`

Strict full-grammar parse. Returns one of:

- `Ok(Selector::Bool(...))` — composed expression
- `Ok(Selector::Pred(...))` — single predicate (top-level grammar simplification)
- `Err(SelectorParseError)` — one of `P-001` … `P-005` codes (see operations doc)

Used by manifest validators, future Grant body fields, and any code that knows it's reading a selector expression.

### `parse_selector_or_uri(input: &str) -> Selector`

Total parse — never errors. Decision tree:

1. `*` → `Selector::Any`
2. `#kind:<name>` → `Selector::KindTag(name)`
3. `<prefix>**` (suffix) → `Selector::Prefix(prefix)`
4. Else: try `parse_selector(input)`; on grammar parse error, return `Selector::Exact(input)`.

The 9 M1 grant-mint call sites (`secrets/add`, `mcp_servers/register`, …) write opaque instance URIs through `Selector::parse(uri)` (a thin wrapper over `parse_selector_or_uri`); each falls through to `Exact` because the URI strings don't begin with `tags …`. New-grammar grants written into `ResourceRef.uri` parse cleanly through the grammar branch.

---

## Evaluator

```rust
impl Selector {
    pub fn evaluate(
        &self,
        target_uri: &str,
        target_tags: &[String],
        registry: &dyn SetRefRegistry,
    ) -> bool;
}
```

Recursive interpretation:

- `Any` → `true`
- `Exact(s)` → `target_uri == s`
- `Prefix(p)` → `target_uri.starts_with(p)`
- `KindTag(k)` → `target_tags.contains("#kind:{k}")`
- `Pred(p)` → `evaluate_predicate(p, target_tags, registry)`
- `Bool(expr)` → `evaluate_bool(expr, target_tags, registry)`

`evaluate_predicate`:

- `Contains(t)` — `target_tags.contains(t.to_canonical())`
- `Intersects(ts)` — `target_tags ∩ {ts.canonical()} ≠ ∅`
- `AnyMatch(g)` — `∃ t ∈ target_tags. glob_match(g, t)` (slash-segmented globs; `*` = one segment, `**` = zero or more)
- `SubsetOf(sref)` — `target_tags ⊆ registry.resolve(sref.name, sref.args)`; `None` (unknown set) → `false`
- `Empty` — `target_tags.is_empty()`
- `NonEmpty` — `!target_tags.is_empty()`

`evaluate_bool` is short-circuit `Or` / `And` / `Not` over inner expressions.

A backwards-compat `Selector::matches(target_uri, target_tags)` shim exists; it wraps `evaluate` with [`NoopSetRefRegistry`](../../../../../../modules/crates/domain/src/permissions/selector.rs).

---

## `SetRefRegistry` trait

```rust
pub trait SetRefRegistry: Send + Sync {
    fn resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>>;
}

pub struct NoopSetRefRegistry;
impl SetRefRegistry for NoopSetRefRegistry { /* always None */ }

pub static NOOP_SET_REF_REGISTRY: NoopSetRefRegistry = NoopSetRefRegistry;
```

Field on [`CheckContext`](../../../../../../modules/crates/domain/src/permissions/manifest.rs):

```rust
pub set_ref_registry: &'a dyn SetRefRegistry,
```

Every M5.2 construction site borrows `&NOOP_SET_REF_REGISTRY` (12 sites in `domain` + `server`). CH-15 will introduce a production registry implementation that resolves `supervisors_tagging_scope(supervisor-id)` and similar named scopes against runtime data; the `CheckContext` field swaps without touching any other engine code.

Per ADR-0036 §D36.3 + ADR-0033 (CH-K8S-PREP) §D33: the trait is shipped from day one so a future remote-backed registry (Redis cache, RPC) drops in without engine changes.

---

## Engine integration (Step 3)

[`engine.rs`](../../../../../../modules/crates/domain/src/permissions/engine.rs) `step_3_match_reaches` filters effective candidates to those whose selector matches the call target:

```rust
.filter(|c| c.resolved.effective_evaluate(
    &ctx.call.target_uri,
    &ctx.call.target_tags,
    ctx.set_ref_registry,
))
```

`ResolvedGrant::effective_evaluate` ANDs the explicit selector with the implicit `#kind:` refinement (concept-04 §"Refinement"), threading the registry through both:

```rust
pub fn effective_evaluate(
    &self,
    target_uri: &str,
    target_tags: &[String],
    registry: &dyn SetRefRegistry,
) -> bool {
    let base = self.selector.evaluate(target_uri, target_tags, registry);
    match &self.kind_refinement {
        Some(r) => base && r.evaluate(target_uri, target_tags, registry),
        None => base,
    }
}
```

`effective_matches(target_uri, target_tags)` is a backwards-compat shim that calls `effective_evaluate` with the noop registry.

---

## Out-of-scope (per concept-09 §"Non-Normative Notes")

Explicitly deferred:

- Bracket char-classes inside globs (`org:[ab]cme`).
- Time predicates (`tags contains created_after:...`).
- Numeric comparisons.
- String-content matching on tag values.
- Cross-instance joins.

Future extensions are additive (new productions; no semantic change to existing ones).

---

## Cross-References

- [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) — normative spec
- ADR-0036 — design decisions
- ADR-0037 — instance identity tag rollout (sibling)
- [`m5_2/operations/selector-grammar-operations.md`](../operations/selector-grammar-operations.md) — runbook
- [`m5_2/user-guide/selector-syntax-guide.md`](../user-guide/selector-syntax-guide.md) — operator syntax reference
