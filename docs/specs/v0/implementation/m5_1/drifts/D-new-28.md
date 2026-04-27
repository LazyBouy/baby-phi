<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-28 — Memory node missing `memory_type` enum (user/feedback/project/reference) per Claude Code model

## Identification
- **ID**: D-new-28
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: LOW
- **Tags**: `memory-model`, `enum-typing`

## Concept alignment
- **Concept doc(s)**: [`concepts/coordination.md`](../../../concepts/coordination.md) §"Memory types"
- **Concept claim**: Adopt 4 memory types from Claude Code's model: `user`, `feedback`, `project`, `reference`. As `memory_type` enum on Memory node.
- **Contradiction**: Memory node has `tags: Vec<String>` but no typed `memory_type` enum.
- **Classification**: `silent-in-code` (tag-based approach may supersede enum-based; decide)

## Remediation
- **Approach**: Design decision: keep tag-based classification OR add enum. If enum, part of Memory contract C-M6-1 work. ~1 day.
- **Impl chunk**: CH-19 (+ M6 review)
- **Risk**: LOW.

## Lifecycle
- 2026-04-24 — `discovered`
