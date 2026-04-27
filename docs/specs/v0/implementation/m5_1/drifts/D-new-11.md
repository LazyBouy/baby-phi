<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-11 — Composite instances don't auto-emit their `{kind}:{id}` self-identity tag at creation

## Identification
- **ID**: D-new-11
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `composite-ontology`, `auto-tagging`
- **Blocks**: D-new-06 (scope resolution depends on instance tags); D-new-08 (frozen tag enforcement needs to know the structural tags)
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Instance Identity Tags"
- **Concept claim**: Every composite instance carries a self-identity tag `{kind}:{instance_id}` (e.g. `session:sess-42`) at creation, in addition to the `#kind:{name}` type tag.
- **Contradiction**: `kind_tag()` auto-adds `#kind:session`, but no code auto-adds `session:sess-42` as an instance tag. Session + Memory + other composite creation paths do not populate a tag-list field with the instance self-identity.
- **Classification**: `partially-honored` (kind tag honored; instance tag missing)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Both `#kind:{name}` AND `{kind}:{id}` tags auto-added at creation.
- **Reality (shipped state at current HEAD)**: Only kind tag auto-logic. Instance self-identity is implicit in node id but not reified as a tag.
- **Root cause**: `concept-doc-not-consulted` during M1 composite-ontology work.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/model/composites.rs:81-92`](../../../../../../modules/crates/domain/src/model/composites.rs#L81-L92) `kind_tag()` — auto-emits `#kind:*` only.
- **Test evidence**: None testing instance self-identity tag presence.
- **Grep for regression**: Check composite-creation sites emit both tags.

## Remediation scope (estimate only)
- **Approach (sketch)**: Add `instance_tag(kind, id) -> String` helper returning `format!("{kind}:{id}")`. Wire into each composite creation path (session, memory, inbox_object, outbox_object, auth_request, etc.).
- **Implementation chunk this belongs to**: CH-06
- **Dependencies on other drifts**: none
- **Estimated effort**: 1 engineer-day.
- **Risk to concept alignment if deferred further**: MEDIUM — foundational tag invariant; selectors that match `session:sess-42` against instance tags don't have the expected tag to match against.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
