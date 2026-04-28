<!-- Last verified: 2026-04-28 by Claude Code -->

# Selector syntax guide

> **Audience:** Operators authoring grants and tool manifests with the tag-predicate DSL. Pair this page with [`selector-grammar.md`](../architecture/selector-grammar.md) (design) and [`selector-grammar-operations.md`](../operations/selector-grammar-operations.md) (runbook).

---

## What's a selector?

A selector is a string that decides whether a Permission Check grant covers a given target. For grants that target a single instance (`secret:anthropic-api-key`) or all members of a class (`#kind:session`), the M1 shapes still work. For richer rules — "any session in this project that's not archived", "memories tagged with my org's eng subtree" — use the tag-predicate DSL.

The grammar is normative in [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md). This page is the practical operator-facing reference.

---

## The six predicates

| Predicate | Reads | Example |
|---|---|---|
| `tags contains <tag>` | True iff `<tag>` is one of the target's tags | `tags contains org:acme` |
| `tags intersects { <tag>, <tag>, ... }` | True iff at least one listed tag is on the target | `tags intersects {session:s-1, session:s-2}` |
| `tags any_match "<glob>"` | True iff at least one target tag matches the glob (`*` = one segment, `**` = zero+ segments) | `tags any_match "org:acme/eng/**"` |
| `tags subset_of <set_ref>` | True iff every target tag is in the named set (registered runtime sets only) | `tags subset_of supervisors_tagging_scope(supervisor-7)` |
| `tags empty` | True iff the target has no tags | `tags empty` |
| `tags non_empty` | True iff the target has at least one tag | `tags non_empty` |

---

## Combinators and grouping

Three logical combinators with explicit precedence:

1. `NOT` — unary, right-associative, binds tightest
2. `AND` — binary, left-associative
3. `OR` — binary, left-associative, binds loosest

Use parentheses to override:

| Selector | Parses as |
|---|---|
| `a AND b OR c` | `(a AND b) OR c` |
| `a AND (b OR c)` | `a AND (b OR c)` |
| `NOT a AND b` | `(NOT a) AND b` |
| `NOT (a AND b)` | `NOT (a AND b)` |

Replace `a` / `b` / `c` with full predicates: `tags contains org:acme`, etc.

---

## Tag forms

Three accepted tag shapes:

| Shape | Example | Notes |
|---|---|---|
| **Reserved** | `#kind:session`, `#archived` | Prefixed with `#`. Either bare (`#archived`) or namespaced-pair (`#kind:value`). Reserved namespaces are read-only for tools per ADR-0037. |
| **Namespace** | `org:acme`, `org:acme/eng/web/lead` | `identifier:identifier(/identifier)*`. The slash-separated value is the typical org-chart subtree shape. |
| **Literal (string)** | `"some text tag"` | Double-quoted; used inside `intersects` for tags that don't fit reserved or namespace shapes. |

Identifiers must match `[a-zA-Z_][a-zA-Z0-9_-]*` — they start with a letter or underscore and may contain digits or hyphens after the first char. **`a:1` is not a valid namespace tag** (value `1` doesn't start with a letter); use `a:x1`, `a:one`, or quote it as a literal `"a:1"`.

---

## Worked examples (from concept-09 §"Worked Parses")

### 1. Project-scoped session selector

```
tags contains project:acme-website-redesign AND tags contains #kind:session
```

Matches any entity tagged with both `project:acme-website-redesign` and `#kind:session` — i.e., a session belonging to the website-redesign project.

### 2. Co-owned session via set intersection

```
tags intersects {session:s-9831, session:s-9832} AND tags contains org:acme
```

Matches entities tagged with either of the two specific sessions AND with `org:acme`. The `org:acme` requirement guards against cross-org sharing.

### 3. Memory subset of a supervisor's tagging scope

```
tags subset_of supervisors_tagging_scope(supervisor-7)
```

Resolves `supervisors_tagging_scope(supervisor-7)` to the set of tags supervisor-7 is permitted to apply (via their org chart + Authority Templates). The selector admits any entity whose entire tag set is contained in that scope.

> The runtime registry that resolves `supervisors_tagging_scope` lands at CH-15. Before then, every `subset_of` predicate evaluates to `false` (the safe default).

### 4. Glob over an org-chart subtree, excluding archived

```
tags any_match "org:acme/eng/**" AND NOT tags contains #archived
```

Matches entities with any tag under the `org:acme/eng/...` subtree (e.g. `org:acme/eng/web/lead`, `org:acme/eng/platform/team-3`) but excludes archived ones.

---

## Glob syntax (for `any_match`)

- `*` matches exactly one slash-separated segment
- `**` matches zero or more slash-separated segments
- Inside a single segment, only literal equality (no `[abc]` char classes, no partial wildcards — these are deferred per concept-09 §"Non-Normative Notes")

Examples:

| Pattern | Matches | Doesn't match |
|---|---|---|
| `org:acme/**` | `org:acme`, `org:acme/eng`, `org:acme/eng/web/lead` | `org:other/eng` |
| `org:acme/*/web` | `org:acme/eng/web` | `org:acme/web`, `org:acme/eng/web/sub` |

---

## Common mistakes

- **Numeric tag values.** `tags contains a:1` fails P-001 because `1` doesn't start with a letter. Use `a:one` or rename the value.
- **Forgetting parens with mixed AND/OR.** `a AND b OR c` is `(a AND b) OR c`. If you mean `a AND (b OR c)`, parenthesise.
- **`any_match` with no wildcard.** `tags any_match "session:s-9831"` fails P-004. Use `tags contains session:s-9831` instead.
- **Quoting reserved tags.** `tags contains "#kind:session"` parses the tag as a `Literal`, not `Reserved`. Drop the quotes: `tags contains #kind:session`.

---

## Cross-References

- [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md) — normative grammar
- ADR-0036 — design decisions
- [`m5_2/architecture/selector-grammar.md`](../architecture/selector-grammar.md) — design page
- [`m5_2/operations/selector-grammar-operations.md`](../operations/selector-grammar-operations.md) — runbook (parse error codes)
- [`m5/user-guide/troubleshooting.md`](../../m5/user-guide/troubleshooting.md) — selector parse-error troubleshooting
