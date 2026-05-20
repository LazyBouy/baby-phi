<!-- Last verified: 2026-05-19 by Claude Code (CH-28 P-SEAL — filed at chunk-seal per ADR-0063 §D63.16 (iter-5 NEW scope-narrowing): template-tier fan-out for BlueprintUpserted listener arm deferred to M6-DEFERRED-04 (CH-36 a04 supervisor body). CH-28 ships the override-tier listener arm only; template-tier fan-out requires NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` which CH-36 inherits via this drift. Cycle hex `0412eb06`.) -->

# D-CH28-FOLLOWUP-01 — Listener template-tier fan-out for BlueprintUpserted (deferred to M6-DEFERRED-04 / CH-36)

## Identification
- **ID**: D-CH28-FOLLOWUP-01
- **Phase of origin**: CH-28 chunk-seal (2026-05-19) — filed per ADR-0063 §D63.16 (NEW iter-5 scope-narrowing decision).
- **Discovery source**: chunk-implementer v14 — surfaced at P1-BLUEPRINT-STRUCT close (2026-05-19) as a SCOPE-NARROWING of the `BlueprintUpserted` listener arm; iter-5 plan codifies the deferral via §D63.16.
- **Date discovered**: 2026-05-19
- **Status**: `discovered`
- **Bucket**: B — follow-on engine-scope widening (architectural — listener fan-out shape across N agents sharing a template Blueprint).
- **Severity**: LOW
- **Tags**: `blueprint-fanout`, `template-tier`, `m6-deferred`, `m6-deferred-04`, `ch-36-supervisor`, `agent-profile-cardinality`
- **Blocks**: nothing within CH-28; the override-tier listener arm ships per §D63.16 + listener arm at `events/listeners.rs:1089-1106 + 1191-1196` covers the SINGLE-owning-agent case.
- **Blocked-by**: nothing — CH-28 ships the hybrid Blueprint shape (F1.c), edge rename (F2.b), split migrations (F3.b), wire-row strip (§D63.14), read-path synthesis (§D63.15), and override-tier listener arm. Template-tier fan-out is the deferred axis.
- **Closing chunk**: **M6-DEFERRED-04** (CH-36 a04 My Work supervisor body) — per chunk-planner v13 non-terminal-drift rule explicit M*-DEFERRED-NN allocation requirement; NOT `TBD`.

## Concept alignment
- **Concept doc(s)**: [`concepts/agent.md`](../../../concepts/agent.md) §"Soul (Immutable Born Structure)" lines 160–169 (template-Blueprint sharing semantic ratified at CH-28 P-DOCS); [`concepts/ontology.md`](../../../concepts/ontology.md) line 98 + new rows below it (the N:1 cardinality + template-Blueprint sharing surface).
- **Contradiction at CH-28 close**: NONE at the user-facing surfaces. CH-28 closes the cardinality flip + override-tier listener arm; the template-tier fan-out is an additional engine-scope-widening axis that does NOT contradict CH-28's claims. The override-tier listener arm at `events/listeners.rs:1089-1106 + 1191-1196` correctly handles per-agent override-Blueprint upserts (the SINGLE owning-agent case). The template-tier fan-out (one template Blueprint upsert visible to N sibling agents) is a NEW concern surfaced by the F1.c hybrid-Blueprint shape and is documented in-source at `listeners.rs:1094-1124` per the implementer's P1 close report.
- **Classification**: `architectural-deferral` (scope-narrowing decision at P1 close, codified via ADR-0063 §D63.16 at iter-5 plan; M6-DEFERRED-04 allocation per user routing).
- **phi-core leverage status**: `N/A` — `BlueprintUpserted` listener traits + impls are baby-phi-native (in `domain` crate); no phi-core types touched by the fan-out widening.

## Plan vs. reality
- **Plan §5 (CH-28 iter-5) said (§D63.16 LOCKED)**: defer template-tier fan-out for `BlueprintUpserted` to M6-DEFERRED-04 (CH-36) via this drift (`D-CH28-FOLLOWUP-01`). CH-28 ships override-tier coverage only; CH-36 a04 supervisor body inherits the template-tier requirement.
- **Reality at CH-28 chunk-seal**: matches plan exactly. The `BlueprintUpserted` listener arm at `domain/src/events/listeners.rs:1089-1106 + 1191-1196` ships the override-tier coverage (fires for the SINGLE owning agent whose override Blueprint was upserted). The template-tier fan-out (fire for ALL agents whose AgentProfile points at the upserted template Blueprint) is documented in-source at `listeners.rs:1094-1124` per the implementer's P1 close report as a NARROWING. Template-tier listener arm is NOT shipped; corresponding Repository method `list_agents_using_blueprint_template` is NOT shipped.
- **Root cause**: the F1.c hybrid Blueprint shape (USER-LOCKED at gate-1) introduces template-Blueprint sharing where N agents may inherit the same template — a one-template-upsert-visible-to-N-siblings fan-out scenario. Implementing it requires:
  - NEW Repository method `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` traversing `Blueprint → reverse AGENT_PROFILE_USES_BLUEPRINT → AgentProfile → reverse USES_PROFILE → Agent`.
  - Listener template-tier arm dispatching profile-snapshot-refresh events to ALL identified siblings.
  - Acceptance test coverage exercising the fan-out (e.g., 3 agents sharing a template; template upsert visible to all 3).
- **Why narrowed at P1**: rare migration-time-only operation (template upserts are not in the user-facing handler chain); the production CH-28 surface (override-tier per-agent governance) is fully covered. CH-36 a04 supervisor body inherits the SAME fan-out shape (supervisors share tuning templates across pools of supervised agents) — designing the fan-out coherently for both surfaces in CH-36 is more efficient than splitting design across CH-28 + CH-36.

## Where visible in code
- **Files**:
  - `modules/crates/domain/src/events/listeners.rs:1089-1124` — `BlueprintUpserted` listener arm ships override-tier; template-tier narrowing documented in-source.
  - `modules/crates/domain/src/repository.rs` — 4 NEW Repository methods ship at P1-BLUEPRINT-STRUCT (`get_blueprint_for_agent_profile`, `get_agent_blueprint_override_for_agent`, `upsert_blueprint_template`, `upsert_agent_blueprint_override`). The deferred 5th method `list_agents_using_blueprint_template` is NOT added at CH-28.
- **Grep for regression** (CH-28 close baseline vs M6-DEFERRED-04 close target):
  - `grep -n "fn list_agents_using_blueprint_template" modules/crates/domain/src/repository.rs modules/crates/domain/src/in_memory.rs modules/crates/store/src/repo_impl.rs` — CH-28: 0 hits (deferred). M6-DEFERRED-04 target: 3 hits (trait def + 2 impls).
  - `grep -nE 'BlueprintUpserted.*template' modules/crates/domain/src/events/listeners.rs` — CH-28: documented narrowing only. M6-DEFERRED-04 target: a template-tier listener arm with `list_agents_using_blueprint_template` invocation.

## Remediation scope (estimate only)
- **Approach (sketch — CH-36 plan)**:
  1. **NEW Repository method**: add `list_agents_using_blueprint_template(BlueprintId) -> Vec<AgentId>` to the `Repository` trait at `domain/src/repository.rs`. Surreal impl traverses `Blueprint → reverse AGENT_PROFILE_USES_BLUEPRINT → AgentProfile → reverse USES_PROFILE → Agent` via 2 RELATION-edge queries. In-memory impl traverses the equivalent HashMap relations.
  2. **Listener template-tier arm**: extend `BlueprintUpserted` listener at `events/listeners.rs:1089-1124` with a template-row branch (when the upserted Blueprint's `agent_id` is `None`, fire the fan-out). Use `list_agents_using_blueprint_template` to identify siblings; emit profile-snapshot-refresh events for each.
  3. **Acceptance test coverage**: per-supervisor-pool scenarios exercising the fan-out (N agents share a template; template upsert visible to all N via emitted events).
  4. **Convergence with supervisor-tier**: CH-36 a04 supervisor body uses the SAME fan-out for "supervisor adjusts shared tuning template across managed agents".
- **Implementation chunk**: **M6-DEFERRED-04** (CH-36 a04 My Work supervisor body) — per chunk-planner v13 non-terminal-drift rule explicit allocation.
- **Dependencies on other drifts**: none. CH-28 ships all upstream surfaces (hybrid Blueprint shape, edge rename, split migrations, override-tier listener arm, read-path synthesis).
- **Estimated effort**: ~1-1.5 ed (1 method × 2 backends + 1 listener arm extension + 2-3 acceptance scenarios). Likely absorbed cleanly inside CH-36 a04 supervisor body scope.
- **Risk to concept alignment if deferred further**: LOW. The template-tier fan-out is a rare migration-time operation; users don't perceive its absence at the CH-28 surface (override-tier governance covers the per-agent case). If deferred past CH-36, the supervisor-tier fan-out (shared tuning templates) cannot be implemented coherently — that would block CH-36.

## Why filed as a follow-on drift (NOT in-CH-28 carve-out)

User routing decision (codified via iter-5 plan §3.E candidate 6 + §5 ADR §D63.16 + iter-5 re-spawn mandate): the template-tier fan-out is:
- Architectural design > ~1 ed scoped, requiring NEW Repository method + listener arm + acceptance scenarios.
- NOT load-bearing for CH-28's M6 chunk-zero invariants (cardinality flip + edge rename + split migrations all ship the override-tier completely).
- Intersects M6+ feature surface (CH-36 a04 supervisor body inherits the SAME fan-out shape).

Per CLAUDE.md gate-5 in-M5-carve-out-vs-M6-DEFERRED routing criteria, this matches the M6-DEFERRED pattern (NOT load-bearing for current milestone invariants + intersects M6+ feature surface) — routed to M6-DEFERRED-04 (CH-36).

## Lifecycle history
- 2026-05-19 — `discovered` — filed at CH-28 P-SEAL per ADR-0063 §D63.16 (NEW iter-5 scope-narrowing); M6-DEFERRED-04 allocation per chunk-planner v13.

## Cross-references
- [`ADR-0063`](../decisions/0063-agent-profile-cardinality-n-to-1.md) §D63.16 — Listener template-tier fan-out scope-narrowing (NEW iter-5) — Decision body + Pre-existing-behaviour preservation note.
- [`ADR-0063`](../decisions/0063-agent-profile-cardinality-n-to-1.md) §"Consequences ### For CH-36" — Inherited requirement amendment per §D63.16.
- [`m6-forward-scope-8b7a8bcd.md`](../../../../plan/forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 41–48 (CH-28 row) + CH-36 row — CH-36 a04 My Work supervisor body row inherits this drift's requirement.
- Plan archive: [`plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md`](../../../../plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/plan.md) §3.E candidate 6 + §4 + §5 §D63.16 — CH-28 plan body documenting the deferral.
