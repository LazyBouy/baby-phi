<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-16 — Memory `recall` / `store` / `delete` action execution not implemented (no recall_memory tool, no store-action path)

## Identification
- **ID**: D-new-16
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap (part of Memory contract C-M6-1)
- **Severity**: HIGH
- **Tags**: `memory-operations`, `tool-execution`, `cascades-to-M6`
- **Blocks**: M5.2/P8 memory-extraction listener (extractor needs store action to write memories); Memory contract C-M6-1
- **Blocked-by**: D4.2 (real agent_loop needed to drive the extractor); D-new-03 (tag-predicate selectors for recall matching)

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) §"Standard Actions Applied to Memory"
- **Concept claim**: `recall` retrieves memory matching a tag predicate; `store` creates memory with chosen tag set; agents `delete` their own memories.
- **Contradiction**: No `recall_memory` tool exists; no `store_memory` action path in listener; no delete-own-memory handler. Memory node struct exists but has no operations path.
- **Classification**: `silent-in-code`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said**: Memory operations live as agent tools (recall/store/delete).
- **Reality (shipped state at current HEAD)**: Memory storage struct only; no operational tools.
- **Root cause**: `cascading-upstream-deferral` — Memory contract explicitly pinned as C-M6-1 in base plan.

## Where visible in code
- **File(s)**: Memory struct in [`nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); no recall/store/delete tool impls.
- **Test evidence**: None.
- **Grep for regression**: `grep -rn "recall_memory\|store_memory\|MemoryRecallTool" modules/crates/` — expect hits post-remediation.

## Remediation scope (estimate only)
- **Approach (sketch)**: Part of M6 Memory-contract work (C-M6-1). Define `MemoryRecallTool`, `MemoryStoreTool`, `MemoryDeleteTool` impls of `phi_core::AgentTool`. Wire into M5.2/P8 extractor (writes via store); agents recall via tool invocation.
- **Implementation chunk this belongs to**: M6-DEFERRED-01
- **Dependencies on other drifts**: D4.2, D-new-03
- **Estimated effort**: 4 engineer-days at M6 (alongside Memory node tier + contract).
- **Risk to concept alignment if deferred further**: HIGH if M6 slips; foundational to Memory model.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (base plan C-M6-1 referenced)
- Code comments: Memory struct doc acknowledges deferred operations
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 3 report)
