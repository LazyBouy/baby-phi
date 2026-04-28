<!-- Last verified: 2026-04-28 by Claude Code -->

# Selector grammar — operations runbook

> **Audience:** SREs, on-call engineers, and operators triaging Permission Check denials caused by selector parse errors. Pair this page with [`selector-grammar.md`](../architecture/selector-grammar.md) (design) and [`selector-syntax-guide.md`](../user-guide/selector-syntax-guide.md) (syntax reference).

---

## Parse error codes

The selector grammar parser surfaces stable error codes. Each `Decision::Denied` whose `failed_step` is `FailedStep::Match` and whose `reason` is a parse error includes one of:

| Code | Symptom | Likely cause | Fix |
|---|---|---|---|
| `P-001` | `unexpected token '<X>' at position <N>` | Stray character or invalid identifier (e.g., a value starting with a digit like `1` instead of `s-1`) | Inspect the input near `position`; ensure identifiers start with `[a-zA-Z_]` and only contain `[a-zA-Z0-9_-]` |
| `P-002` | `unbalanced parens at position <N>` | Missing `)` for a `(` opener (or vice versa) | Count parens in the selector; verify each `(` has a matching `)` at the same depth |
| `P-003` | `unknown predicate '<name>'` | Predicate after `tags` is not one of the 6 supported forms | Use one of: `contains`, `intersects`, `any_match`, `subset_of`, `empty`, `non_empty` |
| `P-004` | `invalid glob '<pattern>' (must contain * or **)` | `any_match` glob is a literal string with no wildcard | Add `*` (single-segment) or `**` (multi-segment) to the pattern, or switch to `tags contains` |
| `P-005` | `unknown set reference '<name>'` (runtime) | `subset_of foo(args)` referenced a set the registry didn't recognise | Check the registered set-ref names; in M5.2 the noop registry returns `false` for every set-ref so this code is reserved for the production registry that lands at CH-15 |

These codes are surfaced via `SelectorParseError::code()`. For Decision logging they appear inside the `failed_step` payload (audit-event structure recorded by the engine).

---

## Triaging a Permission Check denial

When a tool call denies at Step 3 (Match) due to a selector issue, the operator path is:

1. **Pull the audit event** for the denial. The `failed_step` is `Match` and the `reason` carries the structured data.
2. **Locate the offending grant** by `grant_id` in the catalogue. If the grant URI is a legacy shape (`*`, `system:root`, `<prefix>**`, `#kind:<name>`), the parser's fast-path is in play — no grammar parse error possible. The denial is a normal "selector did not match target" outcome.
3. **If the grant URI is a full grammar expression** (`tags contains org:acme AND ...`), validate it with the offline parser:
   ```bash
   # baby-phi/ workspace root
   /root/rust-env/cargo/bin/cargo run -q -p cli -- selector parse "tags contains org:acme AND tags contains #kind:session"
   ```
   (CH-19 will add this CLI surface; until then, run a unit-test fixture that calls `parse_selector(input)` directly.)
4. **Cross-check the target's tags** in the audit event payload — every reserved-prefix tag (`#kind:<name>`, `<name>:<id>`) must appear on every composite/node instance per ADR-0037.

---

## Audit-event entry: `DeniedReason::SelectorParseError`

The engine surfaces a parse error during selector evaluation as a `Decision::Denied` with:

- `failed_step: FailedStep::Match`
- A reason containing the parse-error code (P-001..P-005), the offending position, and the input string

Operators reading the audit log can identify the failing grant by the `grant_id` and the input text.

---

## Instance-identity tag emission ops note

Per ADR-0037 every composite / node creation handler emits the canonical `(#kind:<name>, <name>:<id>)` pair. If a grant relies on `tags contains <name>:<id>` and produces no match:

1. Confirm the target has been migrated past 0008. Pre-migration rows have `tags = []`; post-migration old rows still have `tags = []` until first read (the store-side reader paths emit the canonical pair into in-memory shapes but do not write back). For a grant to match an old row directly, the row must be re-saved through a write path or the operator must run the one-shot backfill (see "Backfill old rows" below).
2. Verify the target was created through one of the wired creation paths (see ADR-0037 §D37.3 table). If it was created through a non-wired path, file a new drift.

### Backfill old rows

There is no automatic backfill in CH-06. Operators who need to backfill `tags` columns on pre-migration rows can run a one-shot SurrealDB query per table:

```sql
-- Example for the session table; repeat per affected table.
UPDATE session SET tags = ['#kind:session', concat('session:', record::id(id))]
  WHERE array::len(tags) = 0;
```

The store-side readers (`AuthRequestRow::into_domain`, `McpServerRow::into_domain`) materialise the canonical pair on read for two specific tables (auth_request, mcp_server) so live traffic on those tables doesn't need a backfill. For the other 8 tables (token_budget_pool, agent_execution_limits, agent_catalog_entry, system_agent_runtime_status, shape_b_pending_projects, inbox_object, outbox_object, session) the read path returns the stored `tags` verbatim — old rows show empty until rewritten or backfilled.

CH-15 will reconsider whether to add read-side emission for the other 8 tables once the supervisor-extraction memory flow exposes the live constraint.

---

## Cross-References

- ADR-0036 — selector grammar design
- ADR-0037 — instance-identity-tag rollout
- [`m5_2/architecture/selector-grammar.md`](../architecture/selector-grammar.md)
- [`m5_2/user-guide/selector-syntax-guide.md`](../user-guide/selector-syntax-guide.md)
- [`concepts/permissions/09-selector-grammar.md`](../../../concepts/permissions/09-selector-grammar.md)
