<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-04 — Consent node carries only 5 fields; concept mandates 10+ (state / requested_at / responded_at / revocable / provenance / nested scope.*)

## Identification
- **ID**: D-new-04
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap
- **Severity**: HIGH
- **Tags**: `ontology-gap`, `consent-model`, `concept-contradiction`
- **Blocks**: D-new-05 (state machine needs state field); D-new-17 (per-session consent needs approval-mode/response path)
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/06-multi-scope-consent.md`](../../../concepts/permissions/06-multi-scope-consent.md) §"Consent Node (New Node Type)"
- **Concept claim (verbatim / paraphrase)**: Consent carries `(consent_id, agent_id, scope.org, scope.templates, scope.actions, state, requested_at, responded_at, revoked_at, revocable, provenance)` — 11 fields with nested `scope` sub-structure.
- **Contradiction**: Shipped `Consent` struct at [`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) has only 5 fields: `id`, `subordinate`, `scoped_to`, `granted_at`, `revoked_at`. Missing: `state`, `requested_at`, `responded_at`, `revocable`, `provenance`, nested `scope.templates`, `scope.actions`.
- **Classification**: `contradicts-concept`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (concept 06 §"Consent Node"): 11-field Consent with nested scope.
- **Reality (shipped state at current HEAD)**: 5-field struct; no state machine; no nested scope; no provenance trail.
- **Root cause**: `concept-doc-not-consulted` — Consent node landed minimally at M2 ConsentPolicy work without materializing the full shape.

## Where visible in code
- **File(s)**: [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) — `Consent` struct (grep `^pub struct Consent`)
- **Test evidence**: Consent round-trip serde tests exist for the 5 fields; concept's additional fields not tested.
- **Grep for regression**: `grep -A12 "^pub struct Consent " modules/crates/domain/src/model/nodes.rs` — current: 5 fields; post-remediation: 11.

## Remediation scope (estimate only)
- **Approach (sketch)**: Extend Consent struct per concept spec (6 new fields + nested `ConsentScope { templates, actions, org }`). Migration 0006 (or 0007) adds the columns. Repo methods update. Wire with state machine (D-new-05) as single chunk.
- **Implementation chunk this belongs to**: CH-09
- **Dependencies on other drifts**: none
- **Estimated effort**: 2 engineer-days (with state machine; solo ~1 day).
- **Risk to concept alignment if deferred further**: HIGH — One-Time + Per-Session consent policies can't function without these fields; concept-mandated lifecycle is unimplementable.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report + user-verification)
