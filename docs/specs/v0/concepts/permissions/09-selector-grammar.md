<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Part of the permissions spec — see README.md for the full map -->

# Selector Grammar (PEG)

> **Purpose.** This file is the **normative grammar** of the tag-predicate DSL used in `resource.selector` fields on Grants and `tag_predicate` constraints on tool manifests. The prose examples elsewhere in the permissions spec are *instances* of this grammar; this file defines what is and isn't a legal selector.
>
> The grammar is presented as a **PEG (Parsing Expression Grammar)** rather than BNF or EBNF. PEG was chosen because (a) its ordered-choice operator (`/`) handles the otherwise-ambiguous overlap between reserved tags (`#kind:session`) and namespace tags (`session:s-9831`) deterministically, and (b) it maps 1:1 to a recursive-descent parser, which is the implementation shape an interpreter most naturally takes.

---

## Atoms

The grammar's terminals.

- **Identifier** — `[a-zA-Z_][a-zA-Z0-9_-]*`. Used for principal IDs, tag values, namespace names, set-reference names.
- **Reserved-prefix tag** — any tag whose first character is `#`. Two forms: `#identifier` (e.g. `#public`, `#sensitive`) and `#identifier:identifier` (e.g. `#kind:session`).
- **Namespace tag** — `identifier:value` where `value` may itself be a `/`-separated path (e.g. `org:acme/eng/web/lead`).
- **String literal** — double-quoted, no interpolation. Used for glob patterns and rare literal-tag cases.
- **Whitespace** — spaces and tabs between tokens; ignored by the parser. Newlines are allowed but conventionally not used (selectors are typically single-line).

---

## Primary Predicates

A predicate evaluates against the target entity's tag set and returns a boolean.

| Predicate | Meaning |
|-----------|---------|
| `tags contains <tag>` | Membership: true iff `<tag>` is in the target's tag set. |
| `tags intersects { <tag>, <tag>, ... }` | Set intersection: true iff at least one listed tag is in the target's tag set. |
| `tags any_match <glob>` | Glob match: true iff at least one tag in the target matches the glob pattern. Supports `*` (single-segment wildcard) and `**` (multi-segment wildcard). Glob is a string literal. |
| `tags subset_of <set-ref>` | Parameterised: true iff every tag in the target is also in the named set, where `<set-ref>` is a function-style reference resolved by the runtime (e.g. `supervisors_tagging_scope(supervisor-7)` returns the set of tags that supervisor-7 is permitted to apply). |
| `tags empty` | True iff the target has no tags at all. |
| `tags non_empty` | True iff the target has at least one tag. |

---

## Logical Composition

Predicates compose via `AND`, `OR`, `NOT` with parentheses for explicit grouping.

**Operator precedence (highest to lowest):**

1. `NOT` (unary, right-associative)
2. `AND` (binary, left-associative)
3. `OR` (binary, left-associative)

`a AND b OR c` parses as `(a AND b) OR c`. Use parentheses if you mean otherwise.

`NOT a AND b` parses as `(NOT a) AND b`. `NOT` binds tighter than `AND`.

---

## The PEG Grammar

```peg
# Top-level selector
Selector        <- _ OrExpr _

# Logical composition (lowest precedence first)
OrExpr          <- AndExpr (_ "OR" _ AndExpr)*
AndExpr         <- NotExpr (_ "AND" _ NotExpr)*
NotExpr         <- "NOT" _ Predicate
                 / Predicate

# A predicate is either a parenthesised sub-selector or a tag predicate
Predicate       <- "(" _ OrExpr _ ")"
                 / TagPredicate

# Tag-predicate operators
TagPredicate    <- "tags" _ ContainsOp
ContainsOp      <- "contains" _ Tag
                 / "intersects" _ TagSet
                 / "any_match" _ TagGlob
                 / "subset_of" _ SetRef
                 / "empty"
                 / "non_empty"

# Tag forms (ordered: reserved-prefix wins over namespace, namespace over literal)
Tag             <- ReservedTag
                 / NamespaceTag
                 / LiteralTag
ReservedTag     <- "#" Identifier ":" Identifier
                 / "#" Identifier
NamespaceTag    <- Identifier ":" TagValue
TagValue        <- Identifier ("/" Identifier)*
LiteralTag      <- StringLiteral

# Sets and globs
TagSet          <- "{" _ Tag (_ "," _ Tag)* _ "}"
TagGlob         <- StringLiteral             # must contain "*" or "**" — semantic check, not syntactic
SetRef          <- Identifier "(" _ Identifier (_ "," _ Identifier)* _ ")"

# Lexical
Identifier      <- [a-zA-Z_] [a-zA-Z0-9_-]*
StringLiteral   <- "\"" StringChar* "\""
StringChar      <- !["\\] .
                 / "\\" .
_               <- [ \t\n\r]*
```

**Notes on the grammar:**

- The ordered choice in `Tag` (`ReservedTag / NamespaceTag / LiteralTag`) is what makes `#kind:session` parse as a `ReservedTag` rather than as a `NamespaceTag` whose namespace happens to start with `#`. PEG's commit-on-first-match semantics is exactly the right tool here.
- `_` is greedy whitespace; PEG's `*` quantifier already commits, so no backtracking concerns.
- The grammar is **context-free**. Validation rules that depend on context (e.g., "the tag glob must contain `*` or `**`", "the set-ref must resolve to a registered runtime function") are **semantic checks** layered on top by the interpreter, not part of the grammar.

---

## Worked Parses

### Example 1 — A simple project-scoped session selector

Input: `tags contains project:acme-website-redesign AND tags contains #kind:session`

Parse tree:

```
Selector
└── OrExpr
    └── AndExpr
        ├── NotExpr → Predicate → TagPredicate
        │              ContainsOp("contains", Tag→NamespaceTag("project", "acme-website-redesign"))
        └── NotExpr → Predicate → TagPredicate
                       ContainsOp("contains", Tag→ReservedTag("#kind", "session"))
```

This selector matches any entity whose tag set includes both `project:acme-website-redesign` *and* `#kind:session` — i.e., a session belonging to the website-redesign project.

### Example 2 — A co-owned-session selector with set intersection

Input: `tags intersects {session:s-9831, session:s-9832} AND tags contains org:acme`

Parse tree:

```
Selector
└── OrExpr
    └── AndExpr
        ├── NotExpr → Predicate → TagPredicate
        │              ContainsOp("intersects",
        │                TagSet[
        │                  Tag→NamespaceTag("session", "s-9831"),
        │                  Tag→NamespaceTag("session", "s-9832")])
        └── NotExpr → Predicate → TagPredicate
                       ContainsOp("contains", Tag→NamespaceTag("org", "acme"))
```

Matches entities tagged with either of the two specific sessions AND with `org:acme` (the latter eliminates the cross-org case if either session were re-shared elsewhere).

### Example 3 — Memory selector with a parameterised set reference

Input: `tags subset_of supervisors_tagging_scope(supervisor-7)`

Parse tree:

```
Selector
└── OrExpr
    └── AndExpr
        └── NotExpr → Predicate → TagPredicate
                       ContainsOp("subset_of",
                         SetRef("supervisors_tagging_scope", ["supervisor-7"]))
```

The runtime resolves `supervisors_tagging_scope(supervisor-7)` to the set of tags supervisor-7 is permitted to apply (typically derived from their org chart position and Authority Template grants). The selector then admits any entity whose entire tag set is contained in that resolved set.

### Example 4 — A glob over an org-chart subtree

Input: `tags any_match "org:acme/eng/**" AND NOT tags contains #archived`

Parse tree:

```
Selector
└── OrExpr
    └── AndExpr
        ├── NotExpr → Predicate → TagPredicate
        │              ContainsOp("any_match", TagGlob("org:acme/eng/**"))
        └── NotExpr("NOT" → Predicate → TagPredicate
                       ContainsOp("contains", Tag→ReservedTag("#archived")))
```

Matches entities with any tag under the `org:acme/eng/...` subtree (e.g., `org:acme/eng/web/lead`, `org:acme/eng/platform/team-3`) but excludes archived ones. Note the `NOT` binds tighter than `AND`.

---

## Reserved Namespace Enforcement

The parser **accepts** reserved tags (`#kind:*`, `{kind}:*`, `delegated_from:*`, `derived_from:*`) inside selectors — that is exactly where they are read, since selectors are queries against the target's tag set, and reserved tags are part of every composite instance's tag set.

The **publish-time manifest validator** is what rejects reserved tags in tool manifests' `actions: [modify]` declarations. A tool that declares the ability to *write* to a reserved namespace is rejected at publish; a tool that declares the ability to *read* (i.e., select for) reserved tags is fine. See [01-resource-ontology.md § Instance Identity Tags (`{kind}:{id}`)](01-resource-ontology.md#instance-identity-tags-kindid) for the reserved-namespace catalog and the read-vs-write asymmetry.

This division of labour means the selector grammar above is **deliberately permissive** about which tag values appear in selectors — the validator handles the security-relevant restrictions on what tags a tool may *create*.

---

## Non-Normative Notes

Things intentionally **not** in the grammar at v0:

- **Glob semantics beyond `*` and `**`.** Bracketed character classes (`org:[ab]cme`), negation inside globs (`!`), and other extensions are not supported in v0. If they become necessary, they will be added by extending `TagGlob`'s semantic-validation pass, not the syntactic grammar.
- **Time predicates** (`tags contains created_after:2026-01-01`). Time is currently expressed via tag namespaces with ISO-8601 string values; richer time arithmetic is deferred.
- **Numeric comparisons** (`token_count > 1000`). Selectors are tag-only; numeric constraints belong on the `constraints` field of the Grant, not the `selector` field.
- **String-content matching on tag values.** The grammar matches whole tag values exactly (after slash-segmentation for `TagValue`); regex or substring matches on values would require an extension and are deferred.
- **Cross-instance joins** (`tags contains #kind:session AND its session.duration > 1h`). The selector evaluates against a single target's tags only; relational joins across instances are out of scope for selectors and live in the broader graph-query layer (see [coordination.md § Design Decisions](../coordination.md#design-decisions-v0-defaults-revisitable) for the Cypher-inspired query subset).

These exclusions keep the grammar small enough to implement, audit, and reason about in v0. Future extensions will be additive (add new productions; do not change the meaning of existing ones).

---

## Cross-References

- The grammar's tag taxonomy is defined in [01-resource-ontology.md § Composite Identity Tags](01-resource-ontology.md#composite-identity-tags-kind) and [§ Instance Identity Tags](01-resource-ontology.md#instance-identity-tags-kindid).
- Selectors appear on the `resource.selector` field of Grants, defined in [04-manifest-and-resolution.md § Grant (5-Tuple, Held by a Subject)](04-manifest-and-resolution.md#grant-5-tuple-held-by-a-subject).
- Selectors also appear as `tag_predicate` constraints on tool manifests; see [04-manifest-and-resolution.md § Tool Authority Manifest](04-manifest-and-resolution.md#tool-authority-manifest-tool-requirements).
- The runtime's evaluation of `selector_matches(g.resource.selector, call.target_tags, call.context)` is described in [04 § Permission Check § Formal Algorithm](04-manifest-and-resolution.md#formal-algorithm-pseudocode), Step 3.
- The `subset_of` parameterised predicate is exercised in [05-memory-sessions.md § Supervisor Extraction as Two Standard Grants](05-memory-sessions.md#supervisor-extraction-as-two-standard-grants).
