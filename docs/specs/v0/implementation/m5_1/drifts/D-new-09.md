<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-09 — Standard action vocabulary has no Rust constants/enums; actions stored as free-form `Vec<String>`

## Identification
- **ID**: D-new-09
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `action-vocabulary`, `typed-action-model`, `bucket-b-hardening`
- **Blocks**: D-new-07 (validator needs to check against a closed set); D-new-10 (matrix enforcement needs typed actions)
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) §"Standard Action Vocabulary"
- **Concept claim**: 33 named actions defined — discover, list, inspect, read, copy, export, create, modify, append, delete, execute, invoke, send, connect, bind, listen, delegate, approve, escalate, allocate, transfer, store, retain, recall, configure, install, enable, disable, spend, reserve, exceed, observe, log, attest.
- **Contradiction**: Actions are `Vec<String>` throughout (`Grant.action`, `Manifest.actions`, `ToolAuthorityManifest.actions`). No constants, no enum, no closed set. Any string is accepted.
- **Classification**: `silent-in-code` — closed vocabulary not reified
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (concept 03): Standard vocabulary defined.
- **Reality (shipped state at current HEAD)**: Free-form strings. No validation.
- **Root cause**: `concept-doc-not-consulted` at M1 — shipped free-form to unblock permission-engine MVP.

## Where visible in code
- **File(s)**: grep `pub action: String` or similar in `nodes.rs` Grant, `manifest.rs` — all free-form.
- **Test evidence**: No tests assert action vocabulary closure.
- **Grep for regression**: `grep -rn "pub const ACTION_\|enum Action " modules/crates/domain/src/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Add `domain::permissions::action::Action` enum with the 33 variants + `TryFrom<&str>` + `as_str()`. Migrate Grant / Manifest / ToolAuthorityManifest callers to parse + validate. Or keep `String` at storage but validate on parse.
- **Implementation chunk this belongs to**: CH-04
- **Dependencies on other drifts**: none
- **Estimated effort**: 1.5 engineer-days (enum + parser + migration of call sites + tests).
- **Risk to concept alignment if deferred further**: MEDIUM — grants can use misspelled/invented actions that silently fail to match anything; compounds with D-new-07 manifest validator.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none)
- Code comments: none
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 2 report)
