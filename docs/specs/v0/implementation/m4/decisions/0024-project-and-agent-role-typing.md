<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0024 — Project + AgentRole typing decisions

**Status: Accepted** — flipped at M4/P1 close after the type set landed in `domain/src/model/nodes.rs` + `composites_m4.rs` + migration 0004.

## Context

M4 introduces the first-class `Project` node + a 6-variant `AgentRole` enum spanning Human (Executive / Admin / Member) and LLM (Intern / Contract / System) agents. Concept docs define the shapes; M4 pins the exact Rust typing + serde form for wire-contract stability across M4→M5→M8.

## Decision

1. **`Project` struct** is a top-level graph node (not a composite sibling of `Organization`). `Objective` + `KeyResult` are embedded value objects on the Project row (not separate nodes). `ResourceBoundaries` is a composite referencing the owning org(s)' catalogue entries by id+kind.
2. **`ProjectShape { A, B }`** enum serde form is `"shape_a"` / `"shape_b"`. Matches the M3/P5 dashboard's pre-existing `AgentsSummary.shape_a` / `.shape_b` field names — wire-contract stability.
3. **`AgentRole`** is a single 6-variant enum (not two enums keyed by kind). `is_valid_for(AgentKind) -> bool` enforces cross-kind correctness at create + edit: Executive/Admin/Member → Human-only; Intern/Contract/System → LLM-only. Serde `snake_case`.
4. **Role lives on `Agent` (not `AgentProfile`).** Role is governance identity; blueprint is execution configuration. Role immutable post-creation at M4 scope.
5. **`AgentKind` stays binary** (Human / LLM). Role refines within kind.

## Consequences

**Positive:** unambiguous domain model for agents; dashboard counters populate 6 + `unclassified` buckets cleanly; concept-doc amendment at P0 captures the expanded taxonomy.

**Negative:** the single-enum choice means `is_valid_for` must be called everywhere that accepts a (kind, role) pair — forgotten validation leaves invalid states representable in memory. Mitigated by a newtype wrapper at service boundaries in P5 if the invariant proves fragile.

**Neutral:** 6 variants today; future extensions (e.g., Shadow agents for M7+) would bump the enum. Wire-contract evolution follows the serde `snake_case` convention.

## References

- [M4 plan §D1, §D2, §D3, §D12](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [`concepts/agent.md §Agent Roles`](../../../concepts/agent.md#agent-roles) — amended at M4/P0.
- [`concepts/project.md`](../../../concepts/project.md).
