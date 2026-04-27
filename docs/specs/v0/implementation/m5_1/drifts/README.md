<!-- Last verified: 2026-04-24 by Claude Code -->

# M5.1 Drift catalogue — index

This directory is the **single source of truth** for every drift discovered
through M5 and any subsequent implementation chunk. Drifts are one-file-per-id;
this README is a queryable index over all of them.

**Status** (as of 2026-04-24 / M5.1/P2 close):
- Existing drifts from ledger: **29 migrated to per-file** (see [`_schema.md`](_schema.md) for structure + [`_ledger-migration-log.md`](_ledger-migration-log.md) for the P1 migration log).
- Concept-audit drifts: **31 discovered at P2** (see [`_concept-audit-matrix.md`](_concept-audit-matrix.md) for the full ~95-row matrix).
- **Grand total at P2 close: 60 drift files.**

## Severity + Bucket summary (at P2 close)

**Totals across 60 drifts:**
- **Bucket A (load-bearing scope gap)**: **16 total** — 5 existing HIGH open (D4.1, D4.2, D6.1, D6.5, D7.1) + 2 remediated at P7 (D5.1, D6.2) + **9 new HIGH** (D-new-01, 02, 03, 04, 05, 06, 07, 13, 14, 16, 17, 18 — 12 new; some MEDIUM). Recalculated: Bucket A new = 12; existing = 7; total **19**.
- **Bucket B (underspecified shape choice)**: **15 total** — 7 existing + 8 new (D-new-09, 10, 11, 12, 15, 19, 20, 22, 24, 25, 28, 29).
- **Bucket C (convention/pattern)**: **22 total** — 17 existing + 5 new (D-new-21, 23, 26, 27, 30, 31).

**Drifts by severity:**
- **HIGH** (must remediate before M5 close): **D4.1, D4.2, D6.1, D6.5, D7.1** (existing 5) + **D-new-01, D-new-02, D-new-03, D-new-04, D-new-05, D-new-06, D-new-07, D-new-13, D-new-14, D-new-16, D-new-17, D-new-18** (new 12) = **17 HIGH total**
- **MEDIUM** (ratify or remediate): **D5.1, D5.2, D5.3, D6.2, D6.3, D7.5** (existing 6) + **D-new-09, D-new-10, D-new-11, D-new-12, D-new-15, D-new-19, D-new-20, D-new-22, D-new-25, D-new-27** (new 10) = **16 MEDIUM total**
- **LOW** (confirm-in-place or defer): 27 total (all Bucket C + some Bucket B)

Every row links to a dedicated drift file. Filter by severity / bucket / concept
to prioritise. Every drift file MUST conform to [`_schema.md`](_schema.md).

## Lifecycle states

Per `process/drift-lifecycle.md` (shipping at M5.1/P4), every drift flows
through:

```
discovered → classified → scoped → in-chunk-plan → remediated
                                                 → renegotiated (with ADR)
                                                 → accepted-as-is (with ADR)
```

## Severity legend

- **HIGH** — Bucket A, cascades to other drifts, or violates a concept-doc
  enforcement claim. Blocks release at the concept-fidelity layer.
- **MEDIUM** — Bucket B shape choice with concept-doc implications. Ratification
  required (ADR or concept-doc refresh).
- **LOW** — Bucket C convention/pattern decision. Confirm-in-place unless
  discovered to violate an invariant.

## Bucket legend

- **A** — Load-bearing scope gap (promised functionality not delivered).
- **B** — Underspecified-plan shape choice (implementer picked without user
  confirmation; either shape would have worked).
- **C** — Convention/pattern decision (module placement, idiom, naming).

---

## Index (populated at P1)

The table below is deliberately empty at P0 close; P1 fills one row per existing
drift **and** converts each File cell from a code-formatted path placeholder
(`` `D1.1.md` ``) to a proper markdown link (formatted as link-to-filename, same
string, just hyperlinked) at the same time the file is created. The columns are
fixed.

| ID | Title | Severity | Bucket | Concept doc touched | phi-core leverage | Status | Impl. chunk | File |
|---|---|---|---|---|---|---|---|---|
| D1.1 | Session/turn tables leverage DEFINE FIELD on 0001 scaffolds | LOW | C | ontology; phi-core-mapping | wrap | discovered | TBD | [D1.1.md](D1.1.md) |
| D1.2 | `runs_session` RELATION retyped (REMOVE + DEFINE) | LOW | C | ontology; project | N/A | discovered | TBD | [D1.2.md](D1.2.md) |
| D1.3 | Session/LoopRecordNode/TurnNode use nested `inner` (not flatten) | LOW | C | phi-core-mapping | wrap | discovered | TBD | [D1.3.md](D1.3.md) |
| D2.1 | persist_session uses CREATE not UPDATE-as-upsert | LOW | C | ontology | N/A | discovered | TBD | [D2.1.md](D2.1.md) |
| D2.2 | RELATE requires LET-first binding | LOW | C | ontology | N/A | discovered | TBD | [D2.2.md](D2.2.md) |
| D3.1 | `DomainEvent::AgentCreated.agent_kind` (serde discriminator) | LOW | C | agent; coordination | N/A | discovered | TBD | [D3.1.md](D3.1.md) |
| D3.2 | Listener wiring via `build_event_bus_with_m5_listeners` free fn | LOW | C | coordination | N/A | discovered | TBD | [D3.2.md](D3.2.md) |
| D3.3 | `BabyPhiSessionRecorder` uses `Arc<Mutex<_>>` inner | LOW | C | phi-core-mapping | wrap | discovered | TBD | [D3.3.md](D3.3.md) |
| D3.4 | Template C org-scoped / Template D project-scoped resolvers | LOW | C | permissions/07 | N/A | discovered | TBD | [D3.4.md](D3.4.md) |
| D4.1 | Permission Check advisory-only (only Step 0 gates) | **HIGH** | **A** | permissions/03, 04, 07 | N/A | discovered | TBD | [D4.1.md](D4.1.md) |
| D4.2 | Real `phi_core::agent_loop` (was synthetic replay feeder) | **HIGH** | **A** | phi-core-mapping; permissions/05; agent; system-agents | direct-reuse | **remediated** (CH-02 / ADR-0032) | CH-02 | [D4.2.md](D4.2.md) |
| D4.3 | `resolve_agent_tools` returns `Vec<ToolSummary>` | LOW | B | permissions/07; phi-core-mapping | wrap | discovered | TBD | [D4.3.md](D4.3.md) |
| D4.4 | `upsert_agent_profile` no-op on fresh rows; branch to create | LOW | C | agent | wrap | discovered | TBD | [D4.4.md](D4.4.md) |
| D4.5 | `write_uses_model_edge` first-class typed method | LOW | C | ontology | N/A | discovered | TBD | [D4.5.md](D4.5.md) |
| D4.6 | `SessionLaunchContext.first_loop_id` avoids double-persist | LOW | C | permissions/05 | wrap | discovered | TBD | [D4.6.md](D4.6.md) |
| D5.1 | CLI + Web for page 12 deferred from P5 to P7 | MEDIUM | A | permissions/07 | N/A | **remediated** | P7 (historical) | [D5.1.md](D5.1.md) |
| D5.2 | Template audit events at `server::platform::templates::audit_events` | MEDIUM | B | permissions/02, 07 | N/A | discovered | TBD | [D5.2.md](D5.2.md) |
| D5.3 | `find_adoption_ar` returns most-recent (not unique) | MEDIUM | B | permissions/02, 07 | N/A | discovered | TBD | [D5.3.md](D5.3.md) |
| D6.1 | `record_system_agent_fire` helper has zero call sites | **HIGH** | **A** | system-agents; coordination | N/A | discovered | M5.2/P8b | [D6.1.md](D6.1.md) |
| D6.2 | CLI + Web for page 13 deferred from P6 to P7 | MEDIUM | A | system-agents | N/A | **remediated** | P7 (historical) | [D6.2.md](D6.2.md) |
| D6.3 | System-agent bucketing needs 3-way union filter | MEDIUM | B | system-agents | N/A | discovered | TBD | [D6.3.md](D6.3.md) |
| D6.4 | System-agent audit events at `server::platform::system_agents::audit_events` | LOW | C | system-agents | N/A | discovered | TBD | [D6.4.md](D6.4.md) |
| D6.5 | disable/archive don't flip durable `active`/`archived_at` | **HIGH** | **A** | agent; system-agents | N/A | discovered | TBD | [D6.5.md](D6.5.md) |
| D7.1 | Live SSE tail deferred to M7 | **HIGH** | **A** | permissions/05 | direct-reuse (planned) | discovered | TBD | [D7.1.md](D7.1.md) |
| D7.2 | `--model-config-id` additive (alongside `--patch-json`) | LOW | B | agent | N/A | discovered | TBD | [D7.2.md](D7.2.md) |
| D7.3 | `phi session preview` as 5th subcommand | LOW | C | permissions/04 | N/A | discovered | TBD | [D7.3.md](D7.3.md) |
| D7.4 | Page 11 recent-sessions retrofit is web-side | LOW | B | project | N/A | discovered | TBD | [D7.4.md](D7.4.md) |
| D7.5 | Web test count unchanged at P7 (no page-component tests) | MEDIUM | B | (test-strategy) | N/A | discovered | M5.2/P9 | [D7.5.md](D7.5.md) |
| D7.6 | Web pages use hybrid inline `"use server"` pattern | LOW | C | (web-convention) | N/A | discovered | TBD | [D7.6.md](D7.6.md) |
| D-new-01 | Identity node scaffolded; 4-field shape not materialized | **HIGH** | **A** | agent; ontology | N/A | discovered | TBD | [D-new-01.md](D-new-01.md) |
| D-new-02 | Storage backend is SurrealDB (not SQLite per concept) | **HIGH** | **A** | coordination | N/A | discovered | TBD | [D-new-02.md](D-new-02.md) |
| D-new-03 | Selector grammar has 4 variants; PEG tag-predicate DSL absent | **HIGH** | **A** | permissions/05, 09 | N/A | discovered | TBD | [D-new-03.md](D-new-03.md) |
| D-new-04 | Consent node has 5 fields; concept mandates 10+ | **HIGH** | **A** | permissions/06 | N/A | discovered | TBD | [D-new-04.md](D-new-04.md) |
| D-new-05 | Consent lifecycle state machine missing | **HIGH** | **A** | permissions/06 | N/A | discovered | TBD | [D-new-05.md](D-new-05.md) |
| D-new-06 | Multi-scope cascade resolution incomplete | **HIGH** | **A** | permissions/04, 06, 08 | N/A | discovered | TBD | [D-new-06.md](D-new-06.md) |
| D-new-07 | Publish-time manifest validator missing | **HIGH** | **A** | permissions/04, 07 | N/A | discovered | TBD | [D-new-07.md](D-new-07.md) |
| D-new-08 | Frozen session-tag immutability not enforced | **HIGH** | **A** | permissions/05 | N/A | discovered | TBD | [D-new-08.md](D-new-08.md) |
| D-new-09 | Action vocabulary stored as `Vec<String>` (no enum) | MEDIUM | B | permissions/03 | N/A | discovered | TBD | [D-new-09.md](D-new-09.md) |
| D-new-10 | Action × Fundamental applicability matrix not enforced | MEDIUM | B | permissions/03 | N/A | discovered | TBD | [D-new-10.md](D-new-10.md) |
| D-new-11 | Composite instance self-identity tag not auto-added | MEDIUM | B | permissions/01 | N/A | discovered | TBD | [D-new-11.md](D-new-11.md) |
| D-new-12 | AuthRequest per-state ACL not enforced | MEDIUM | B | permissions/02 | N/A | discovered | TBD | [D-new-12.md](D-new-12.md) |
| D-new-13 | allocate/transfer cardinality not enforced | **HIGH** | **A** | permissions/02, 03 | N/A | discovered | TBD | [D-new-13.md](D-new-13.md) |
| D-new-14 | system:genesis axiom + authority-chain traversal missing | **HIGH** | **A** | permissions/02, 04; README | N/A | discovered | TBD | [D-new-14.md](D-new-14.md) |
| D-new-15 | AuthRequest 2-tier retention (90d active + archived) not wired | MEDIUM | B | permissions/02 | N/A | discovered | M7b | [D-new-15.md](D-new-15.md) |
| D-new-16 | Memory recall/store/delete action execution missing | **HIGH** | **A** | permissions/05 | N/A | discovered | M6 C-M6-1 | [D-new-16.md](D-new-16.md) |
| D-new-17 | Per-Session consent gating incomplete | **HIGH** | **A** | permissions/06 | N/A | discovered | TBD | [D-new-17.md](D-new-17.md) |
| D-new-18 | Grant revocation cascade full-tree walk needs verification | **HIGH** | **A** | permissions/08; README | N/A | discovered | TBD | [D-new-18.md](D-new-18.md) |
| D-new-19 | audit_class composition (strictest wins) not enforced | MEDIUM | B | permissions/07 | N/A | discovered | TBD | [D-new-19.md](D-new-19.md) |
| D-new-20 | Contractor-model logic (base_org ceiling bound) missing | MEDIUM | B | permissions/06, 08 | N/A | discovered | TBD | [D-new-20.md](D-new-20.md) |
| D-new-21 | Edge-count documentation mismatch | LOW | B | ontology | N/A | discovered | TBD | [D-new-21.md](D-new-21.md) |
| D-new-22 | Agent role immutability post-creation not enforced | MEDIUM | B | agent | N/A | discovered | TBD | [D-new-22.md](D-new-22.md) |
| D-new-23 | Human Agents lack Identity-assignment guard | LOW | C | human-agent | N/A | discovered | TBD (w/ D-new-01) | [D-new-23.md](D-new-23.md) |
| D-new-24 | Channel node schema incomplete | LOW | B | human-agent; ontology | N/A | discovered | M7 | [D-new-24.md](D-new-24.md) |
| D-new-25 | InboxObject/OutboxObject missing AgentMessage embedding | MEDIUM | B | ontology | N/A | discovered | M6+ | [D-new-25.md](D-new-25.md) |
| D-new-26 | Task node fully scaffolded | LOW | C | project | N/A | discovered | later | [D-new-26.md](D-new-26.md) |
| D-new-27 | Token-economy fields missing on Agent | MEDIUM | C | token-economy | N/A | discovered | later | [D-new-27.md](D-new-27.md) |
| D-new-28 | Memory memory_type enum missing | LOW | B | coordination | N/A | discovered | M6 C-M6-1 | [D-new-28.md](D-new-28.md) |
| D-new-29 | allocate refinement constraints (no_further_delegation) missing | LOW | B | permissions/03 | N/A | discovered | TBD | [D-new-29.md](D-new-29.md) |
| D-new-30 | Org/Project template as config object (vs adoption AR) not materialized | LOW | C | permissions/07 | N/A | discovered | TBD (doc refresh) | [D-new-30.md](D-new-30.md) |
| D-new-31 | Reserved-namespace write rejection at publish time missing | LOW | C | permissions/09 | N/A | discovered | TBD (w/ D-new-07) | [D-new-31.md](D-new-31.md) |

**Row count at P1 close: 29 files written, all row cells populated. ✅**
**Row count at P2 close: 60 files written (29 existing + 31 new D-new-NN). ✅**

**Note on the "24 vs 29" meta-drift**: throughout M5.1 planning the plan text referred to "24 drifts". The correct count is **29** (3+2+4+6+3+5+6). See [`_ledger-migration-log.md`](_ledger-migration-log.md) Entry 0.

## Cross-cut queries (populated as files are written)

- **By severity:** `grep -l "Severity: HIGH" *.md | wc -l` at any time gives
  the HIGH count.
- **By bucket:** `grep -l "Bucket: A" *.md` lists Bucket A drifts.
- **By concept doc:** `grep -l "concepts/permissions/04" *.md` lists every
  drift touching permission-manifest concept.
- **By phi-core leverage:** `grep -l "leverage-violation" *.md` lists every
  phi-core reuse violation.
- **By status:** `grep -l "Status: discovered" *.md` lists triage backlog.

## Related artifacts

- [`_schema.md`](_schema.md) — canonical drift-file template.
- [`_ledger-migration-log.md`](_ledger-migration-log.md) — P1 migration log documenting all discrepancies between ledger text and per-file catalogue (7 entries at P1 close).
- [`_concept-audit-matrix.md`](_concept-audit-matrix.md) — P2 audit matrix walking 20 concept docs claim-by-claim against current HEAD (~95 rows; 31 new drifts + 2 discarded proposals).
- `../process/per-chunk-planning-template.md` — written at P4; the template
  every future implementation-chunk plan follows.
- `../process/chunk-lifecycle-checklist.md` — written at P4.
- `../process/drift-lifecycle.md` — written at P4.
