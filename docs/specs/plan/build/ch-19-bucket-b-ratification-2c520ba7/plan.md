<!-- Last verified: 2026-05-10 by Claude Code (chunk-planner v10; plan v1; cycle hex `2c520ba7`) -->

# CH-19 — Bucket B ratification (ADR-driven shape choices)

**Cycle hex:** `2c520ba7`
**Slug:** `ch-19-bucket-b-ratification`
**Type:** **doc-only** — no code change, no test count change, no migration, +0 phi-core delta.
**Forward-scope row:** [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md:179-183`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (§5 row also at line 427).
**Per-chunk template:** [`m5_1/process/per-chunk-planning-template.md`](../../../v0/implementation/m5_1/process/per-chunk-planning-template.md).
**Drifts closed in this chunk:** D5.2, D5.3, D6.3, D7.2, D7.4, D7.5, D-new-21, D-new-25, D-new-27, D-new-30 (10 drifts; transition `discovered → accepted-as-is` for nine; D7.4 is `accepted-as-is` with M7-server-promote-deferral note; D-new-25 + D-new-27 are `accepted-as-is` with deferred-marker cross-ref to M6-DEFERRED-02 and M6-or-M7-DEFERRED).
**ADR drafted:** ADR-0057 — Bucket B convention ratification (10 sub-decisions D57.1–D57.10).
**ADR home:** `m5_2/decisions/0057-bucket-b-convention-ratification.md` (next-free slot per `ls modules/.../decisions/ | tail -1` returning `0056-...`; CH-18's `0056-auth-request-per-state-acl-enforcement.md`).

---

## Forks for orchestrator

**(none requiring user-lock.)** All three potentially-forked decisions resolved at planner-recommendation level via existing precedent + Q7's "uniform doc-only ritual but no code-fork forks":

- **F1 — ADR location.** Single consolidated ADR at `m5_2/decisions/0057-bucket-b-convention-ratification.md`. **Recommendation: Single ADR.** Rationale: ADR-0042 (CH-03 storage-backend ratification) is the precedent for "doc-only chunk → single ratifying ADR with multiple sub-decisions"; ADR-0057 follows the same shape with 10 sub-decisions. Splitting across milestone-homed ADRs (one per drift) would (a) create 10 thin ADRs that mostly cross-ref the source code without adding new decision-substance, (b) fragment the audit trail across `m5/decisions/` and `m5_2/decisions/`, (c) violate CH-08 retro Row 1's milestone-prefixed-cross-references discipline (each split ADR would need to cross-ref the others). Single consolidated ADR matches the forward-scope row's literal wording: *"1 consolidated ADR covering audit-event placement + bucketing convention + retrofit location + test-strategy"*.

- **F2 — Concept-doc refresh granularity.** Targeted 1-paragraph refresh notes added at named §-anchors in 6 concept docs (not full §-level prose rewrites). **Recommendation: targeted paragraphs.** Rationale: the drifts being ratified are concept-silent-plan-filled-gap (8/10) or concept-aspirational (2/10) — concept docs are not contradicted, just incomplete or below-granularity. Targeted refresh paragraphs add the missing implementation-shape note without reframing settled prose. ADR-0042 §"Doc updates" is the precedent (1-paragraph "v0.1 ships SurrealDB; backend is configurable" added to `coordination.md` §"Storage backend"; full design rationale stayed in the ADR body).

- **F3 — Drift transition target.** All ten drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../../v0/implementation/m5_1/process/drift-lifecycle.md). **Recommendation: `accepted-as-is`.** Rationale: Bucket B drifts ARE the "ratify the existing convention" bucket; no remediation (would be `remediated`); no concept-doc reframing of source-of-truth (would be `renegotiated`); the existing convention IS the answer + ADR records the explicit-risk-acceptance-statement + review-trigger per `drift-lifecycle.md:122-128`. Two drifts (D-new-25 + D-new-27) are deferred-scope items where `accepted-as-is` ALSO carries a deferred-marker cross-ref (M6-DEFERRED-02 / M6-or-M7-DEFERRED) — the chunk where they would actually be remediated. D7.4 carries an additional M7-server-promote review trigger.

**Cross-cycle pattern note (v10 self-discipline check):** planner-recommendation here does NOT diverge from a `tighter-scope` option — there is no tighter-scope option for a doc-only ratification chunk. The cross-cycle user-lock-divergence v10 rule (CH-15 + CH-17 + CH-18 → 3-of-4 cycle pattern) does NOT trigger; CH-19 is a genuine zero-fork doc-only cycle.

---

## §1 — Context & principle

**Why this chunk.** The M5.1/P3 forward-scope inventory split the 60-drift catalogue into three buckets: A (load-bearing scope gap), B (underspecified shape choice), C (convention/pattern decision). Bucket B drifts represent shape choices the implementer made (or that emerged from reality-feedback at phase close) which the original plan/concept-doc text did not pre-specify — but which are NOT concept-doc contradictions. The shipped convention works; what's missing is the formal acceptance + the recoverable record explaining why this shape rather than another. CH-19 is the dedicated **convention-ratification** chunk for the 10 Bucket-B drifts, exactly as forward-scope §1 line 177 frames it: *"Convention ratification (doc-only)"*. The chunk produces (a) one consolidated ADR-0057 with ten sub-decisions, (b) targeted concept-doc refresh paragraphs at named §-anchors in 6 concept docs, (c) drift-status flips `discovered → accepted-as-is` for all ten, (d) concept-audit-matrix row refreshes for the rows the drifts cover, (e) drifts/README.md index refreshes. No production code changes; no migrations; no test changes; phi-core import baseline unchanged.

**Quality-over-speed restatement.** *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-19 application: the ten Bucket-B drifts have lived as `discovered`/`classified` in the catalogue since M5/P5–P7 close (April 23–24, 2026); shipping CH-19 closes them by promoting their convention-status from "shipped but undocumented" to "shipped + ADR-0057 + concept-doc refresh paragraph + matrix row honored." The chunk's value is removing silent-convention drift, not changing behaviour.

**Forward-scope reference.** [forward-scope row at line 179-183](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md). Severity-row binding at line 427 of §5 table: `MED | 2d | various (refresh paragraphs) | — | yes`. The chunk's §6 severity entry is `MED` (closes 6 MEDIUM-severity Bucket-B drifts + 4 LOW-severity).

---

## §2 — Concept alignment walk

Every concept doc whose claims a Bucket-B drift touches. **All ten drifts are `concept-silent-plan-filled-gap` or `concept-aspirational` per their `Classification` field — NONE are `contradicts-concept`.** Status at chunk-open documents the silence/aspiration; status at chunk-close documents the refresh paragraph that closes the silence.

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`concepts/permissions/02-auth-request.md`](../../../v0/concepts/permissions/02-auth-request.md) | §"Auth Request Lifecycle" (line 7) + §"Interaction with Authority Templates" (line 210) | *"An org's adoption of a template is expressed via an AR. Over time an org may adopt, revoke, re-adopt — multiple ARs for the same (org, kind) exist in history. The 'current' adoption state is the most-recent AR's state."* (D5.3 paraphrase per drift body line 19) | silent-in-code (concept silent on multi-AR-per-(org,kind) history; plan implied 1-to-1) | honored (refresh paragraph in §"Interaction with Authority Templates" makes multi-AR-per-(org,kind) history explicit + cross-refs `find_adoption_ar` most-recent semantics) |
| [`concepts/permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) | §"Standard Permission Templates" preamble (line 8) + §"Templates Are Pre-Authorized Allocations" (line 14) | *"Templates and the Auth Request mechanism are not two separate things — every template is a pre-authorized allocation expressed through the Auth Request mechanism itself."* (existing concept text at line 14-37, supports D5.3) AND *"adopting a template"* (existing text supports D5.3 multi-adoption framing) | partially-honored (D5.3 multi-AR-per-(org,kind) implicit; D-new-30 template-as-config aspirational framing tension) | honored (cross-ref note at §"Templates Are Pre-Authorized Allocations" tail clarifies adoption-AR pattern is the v0 implementation of the YAML-config-object framing earlier in the doc; concept doc 07 is consistent post-CH-19) |
| [`concepts/system-agents.md`](../../../v0/concepts/system-agents.md) | §"Purpose" (line 10) + §"Properties Shared by All System Agents" (line 21) | *"Two v0 system agents per org: memory-extraction + agent-catalog. Additional system agents are org-specific."* (D6.3 close paraphrase) | partially-honored (concept silent on bucketing-filter robustness; M3-era fixture rows pre-date M5 canonical slugs) | honored (refresh paragraph at §"Properties Shared by All System Agents" tail documents 3-way union bucketing convention: canonical slug OR `Organization.system_agents` registry OR `AgentRole::System`) |
| [`concepts/agent.md`](../../../v0/concepts/agent.md) | §"Soul (Immutable Born Structure)" line 158 (AgentProfile binds ModelConfig) | *"Agents bind to a model_config_id via their profile."* (D7.2 paraphrase per drift body line 19; concept silent on operator-flag shape) | silent-in-code (concept silent on `--model-config-id` flag shape; plan ambiguous between additive and replace) | honored (refresh paragraph at §"Soul" tail records additive-flag convention: `--patch-json` + `--model-config-id` coexist mutually-exclusive; both reach `PATCH /api/v0/agents/:id/profile`) |
| [`concepts/project.md`](../../../v0/concepts/project.md) | §"Project (Node Type)" — preamble (line 13) | concept silent on "recent sessions" UI surfaces (project context includes session history) | silent-in-code (D7.4: concept doesn't pre-specify server-detail-vs-web-parallel-fetch) | honored (NEW sub-§ "Recent sessions UI surface (web-side)" added under §"Project (Node Type)" — documents v0.1 web-side parallel-fetch + M7 server-side promotion review trigger) |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"Edge Types" header (line 87 — currently *"Edge Types (66 total)"*) | edge-type total count (M3 had 67; M4/P1 added 2 = 69 in some places, 71 in code) | partially-honored (D-new-21: 3 sources, 3 different counts — `ontology.md:87` "66"; `edges.rs:1,12,25` "69"; `edges.rs:520,524` `EDGE_KIND_NAMES: [&str; 71]` test-asserted invariant) | honored (concept doc updated to **71 total**; matches the test-asserted invariant `EDGE_KIND_NAMES.len() == 71` at `edges.rs:661`) |
| [`concepts/permissions/05-memory-sessions.md`](../../../v0/concepts/permissions/05-memory-sessions.md) | (no §-anchor change — out-of-chunk; CH-24 Playwright covers D7.5) | concept silent on test-strategy (drift body line 18: *"concept docs are silent on test strategy (below concept granularity)"*) | silent-in-code (D7.5: web tests deferred to M5.2/P9 Playwright per CH-24) | accepted-as-is (no concept-doc refresh required; ADR-0057 §D57.6 records the test-strategy decision; deferred-marker → CH-24 Playwright scope) |
| [`concepts/ontology.md`](../../../v0/concepts/ontology.md) | §"Node Types — Social Structure (phi extensions)" line 82-83 (InboxObject/OutboxObject) | *"InboxObject + OutboxObject carry embedded `AgentMessage` value objects (receive/send queues)."* (existing concept text at line 82 — *"Messages are AgentMessage value objects embedded on it"*) AND value-object table at line 222-223 | silent-in-code (D-new-25: shipped structs at `nodes.rs:772-784` have `id`, `agent_id`, `created_at` — no `messages: Vec<AgentMessage>` field; concept itself is consistent — code just hasn't materialized yet) | accepted-as-is (deferred to M6-DEFERRED-02 inter-agent-messaging chunk; ADR-0057 §D57.8 records the M6 deferral + ontology.md gets a 1-line deferred-state footnote at the InboxObject/OutboxObject row referencing CH-19 + M6-DEFERRED-02) |
| [`concepts/token-economy.md`](../../../v0/concepts/token-economy.md) | §"Worth (Backward-Looking Reputation)" line 38 + §"Rating Window" line 73 | *"Worth = average_rating × (total_tokens_earned − total_tokens_consumed) / total_tokens_consumed"* + *"rolling rating window (size N=20 default)"* (existing concept text) | concept-aspirational (D-new-27: Agent struct has none of these fields; concept itself is aspirational pending the contracts/bidding milestone) | accepted-as-is (deferred to M6-or-M7-DEFERRED token-economy chunk; ADR-0057 §D57.9 records the M6/M7 deferral; token-economy.md gets a 1-line deferred-state footnote at the §"Worth" preamble referencing CH-19 + M6-or-M7-DEFERRED) |
| [`concepts/permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) | §"Standard Organization Template" line 74 + §"Standard Project Template" line 274 | *"Org template YAML specifies tools_allowlist, resource_catalogue, system_agents, authority_templates_enabled, consent_policy, execution_limits, session_object_grants, memory_object_grants, rating_window."* (existing aspirational YAML-config framing) | concept-aspirational (D-new-30: shipped pattern uses adoption ARs + listener-fired Grants — functionally equivalent to YAML config but differs from concept's framing) | honored (refresh paragraph in §"Standard Organization Template" preamble documents "v0.1 ships the YAML-config semantics through adoption-AR + listener-fired-Grant pattern; the YAML config above is the **conceptual contract**; the AR is the v0.1 **implementation surface**"; same pattern at §"Standard Project Template") |
| [`concepts/permissions/02-auth-request.md`](../../../v0/concepts/permissions/02-auth-request.md) | (D5.2 — concept silent below code-organisation granularity) | *"Audit events capture governance state transitions. The code-organisation of the event-builder modules is below concept granularity."* (D5.2 drift body line 19) | silent-in-code (D5.2: concept silent on domain-tier-vs-platform-tier event-builder placement) | accepted-as-is (no concept-doc refresh required — ADR-0057 §D57.1 records the convention `domain::audit::events::mX::*` for state-machine events; `server::platform::<page>::audit_events` for HTTP-handler events) |

**Coverage:** 6 concept docs touched (`permissions/02-auth-request.md`, `permissions/07-templates-and-tools.md`, `system-agents.md`, `agent.md`, `project.md`, `ontology.md`, `token-economy.md` — 7 actually). Permissions subtree hooked: `permissions/02` + `permissions/07` (per per-chunk-template §2 rule), so `permissions/README.md` MUST be cited as the entry-invariants source. **`permissions/README.md` is cited via existing matrix row line 138-139 cross-ref**; CH-19 does not modify `permissions/README.md` directly.

**phi-core-mapping hook:** N/A — none of the 10 drifts touch a phi-core type. `concepts/phi-core-mapping.md` does NOT appear in the table per its row-coverage rule.

---

## §3 — phi-core leverage map

**Doc-only chunk → zero phi-core surface change.** No phi-core types are imported, wrapped, duplicated, or rejected by CH-19's deliverables.

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Expected import-count delta at chunk close:** **+0** (zero — doc-only). Baseline at chunk-open: 57 (verified via canonical grep below). Predicted at chunk-close: 57 (Δ +0).

**Positive close-audit greps (canonical baseline preservation):**
```bash
# Canonical phi-core baseline (CH-15 retro Row 3; chunk-planner v8+ canonical form; NO trailing ::)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (unchanged)
```
Verified at chunk-open 2026-05-10 via the same command → returned `57`.

**Forbidden-duplication greps:** N/A — no phi-core type duplicated by CH-19 (doc-only). The standing `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` guard MUST stay green at chunk-close (re-runs as part of §12 verification recipe).

**Cascade-artifact discipline (v3+):** N/A — no struct-cascade, no enum-additive cascade, no wire-mapping cascade. The discipline applies to code chunks; CH-19 is doc-only.

**Cross-cycle phi-core baseline anchor:** CH-18 closed at 57 (Δ +0); CH-19 expected to close at 57 (Δ +0); the baseline carries forward into CH-20 unchanged.

---

## §3.B — K8s microservice readiness check

**Doc-only chunk → all 7 axes `no impact`.** The chunk modifies only markdown files under `docs/specs/v0/concepts/`, `docs/specs/v0/implementation/m5_1/drifts/`, and `docs/specs/v0/implementation/m5_2/decisions/`. No Rust code changes. No new in-process state, IPC, pod-local resources, migrations, trait shapes, cross-pod state, or audit emitters.

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`, `RwLock`, etc.) | none — doc-only | **no** | n/a |
| **A2** | New IPC channel (`mpsc`, `broadcast`, etc.) | none — doc-only | **no** | n/a |
| **A3** | New pod-local resource | none — doc-only | **no** | n/a |
| **A4** | Migration runner / first-apply race | none — no migration | **no** | n/a |
| **A5** | Trait-shape requirement | none — `Repository` trait stable | **no** | n/a |
| **A6** | Cross-pod state sharing | none — doc-only | **no** | n/a |
| **A7** | Audit hash-chain symmetry — **specifically D5.2 ADR-0057 §D57.1.** D5.2 ratifies the convention `domain::audit::events::mX::*` (state-machine emitters) vs `server::platform::<page>::audit_events` (HTTP-handler emitters). Both kinds of emitter call into the same single-writer `AuditEmitter` injection, so single-writer guarantee is preserved. CH-19 documents the convention; it does NOT introduce a new audit writer. | doc-only ratification of an existing convention | **no** | n/a — convention preserves single-writer guarantee per ADR-0033 §D33.4 |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — N/A (chunk does not touch the registry).
- D33.2 (`SurrealStore::open_remote`) — N/A (chunk does not add storage operations).
- D33.3 (SIGTERM graceful shutdown) — N/A (chunk does not add `tokio::spawn` tasks).
- D33.4 (`EventBus.shutdown` + `drain`) — N/A (chunk does not add EventBus emitters/listeners; D5.2 ratification documents an existing convention that already complies).

**Conclusion paragraph.** **K8s-neutral.** All 7 axes evaluate `no impact`; no new ledger entry required at `m7b/architecture/deferred-from-ch-k8s-prep.md`. ADR-0057 §D57.1's ratification of the audit-event-placement convention is structurally K8s-positive (it documents the convention that preserves A7 single-writer symmetry — operators reading the ADR understand WHY domain-tier vs platform-tier emitters both flow through the single AuditEmitter chain).

---

## §3.C — User-facing documentation impact map

**Tier evaluation per Q9 (CH-22 codification).** CH-19 is doc-only; the affected docs are predominantly **governance-tier** (ADRs, drifts, concept-audit matrix, concept-doc refresh paragraphs). User-facing tier impact is **MINIMAL** — Bucket B drifts are mostly silent-in-code conventions invisible to operators. Two minor user-facing surfaces affected: D7.2 (CLI `--model-config-id` flag — already documented in CLI completion-help test pinning the flag at `cli::tests::completion_help`; CH-19 only ratifies the convention, no new operator instruction) and D7.4 (web "Recent sessions" panel — already operating in production at M5/P7 close; CH-19 only ratifies the location, no UX change).

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/<milestone>/architecture/<feature>.md` | **none directly.** No new architecture doc; ADR-0057 itself functions as the architecture-tier record for the 10 conventions (precedent: ADR-0042 served the same purpose for CH-03's storage-backend ratification — no separate architecture/*.md was added). | (a) update in-chunk: **none** — ADR-0057 is the architecture-tier surface |
| **Operations** | `docs/specs/v0/implementation/<milestone>/operations/<feature>-operations.md` | **none directly.** D7.2 CLI flag is already documented via the test pin; D5.2 audit-event placement convention is internal-engineering-only (operators see the events via the `audit_events` table; placement is invisible to them). | (a) update in-chunk: **none** |
| **User-guide** | `docs/specs/v0/implementation/<milestone>/user-guide/{<feature>-walkthrough,cli-reference-mN,troubleshooting}.md` | **`m5/user-guide/cli-reference-m5.md` MAY contain a 1-line cross-ref to ADR-0057 §D57.4** (D7.2 ratification) — verify at draft-time whether CLI reference doc covers `agent update`. **Amend-don't-add precedence (CH-17 retro Row 3):** if the CLI reference already documents `--patch-json` + `--model-config-id`, add the ADR-0057 §D57.4 cross-ref as a "CH-19 amendment — `--model-config-id` additive convention (2026-05-10)" subsection rather than fragmenting users across a new doc. | (b) defer: **deferred to M5-tag-close** with successor M5-tag-close batch (open-ended deferrals are not permitted; M5-tag-close IS the bounded successor per Q9 grandfathering) — checked at chunk-open: `cli-reference-m5.md` is a stub with `[PLANNED]` markers (per `m5/user-guide/cli-reference-m5.md` body); no live operator content yet. CH-19 does not amend stubs; the cross-ref lands when the stub is filled at M5-tag-close. **Successor: M5-tag-close.** |

**Rule compliance:** every doc the chunk's code makes stale is listed; defer-decisions cite a bounded successor (`M5-tag-close`); no open-ended deferrals.

---

## §3.D — Forward-scope-vs-concept-doc precedence

**Pre-flight check (v9+ MANDATORY procedure).** The forward-scope row at line 179-183 enumerates 10 drift IDs but does NOT introduce any new closed-set vocabulary (action verbs, fundamental kinds, audit-event class tiers, migration order, schema field names). Bucket B is by definition shape-choice ratification of EXISTING conventions; the chunk does not extend any closed set.

**Mechanical procedure executed:**

1. **Action verbs:** N/A — no new action verb. ADR-0057 sub-decisions D57.1–D57.10 do not touch `Action::CANONICAL`. The closed-set invariant `Action::CANONICAL.len() == 34` (CH-04 / ADR-0043; verified verbatim at `domain/src/permissions/action.rs:282`) is preserved unchanged.
2. **Fundamental kinds:** N/A — no new fundamental kind.
3. **Selector grammar predicates:** N/A — no new predicate.
4. **Audit-event class tiers:** N/A — D5.2 ratifies an EXISTING placement convention; no new event class.
5. **Migration order:** N/A — no migration.
6. **Schema field names:** N/A — no schema change.

**Verdict:** `forward-scope row literal text` matches `concept-doc canonical phrasing` for all 10 drifts. **NO contradiction; NO CRITICAL fork required; auto-approval criteria not blocked by §3.D.** The cross-cycle pattern-watch (CH-15 + CH-17 both required iter-2 re-spawn from incorrect closed-set claims) does NOT apply here — CH-19 makes no closed-set contradiction claim.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Bucket | Transition | ADR sub-decision | Notes |
|---|---|---|---|---|---|---|
| D5.2 | [`m5_1/drifts/D5.2.md`](../../../v0/implementation/m5_1/drifts/D5.2.md) | MEDIUM | B | discovered → accepted-as-is | §D57.1 | Audit events at `server::platform::<page>::audit_events` (HTTP-handler tier) vs `domain::audit::events::mX::*` (state-machine tier). Convention ratified; D6.4 (already in CH-20 scope) follows the same pattern at `server::platform::system_agents::audit_events`. |
| D5.3 | [`m5_1/drifts/D5.3.md`](../../../v0/implementation/m5_1/drifts/D5.3.md) | MEDIUM | B | discovered → accepted-as-is | §D57.2 | `find_adoption_ar` returns most-recent AR (sorted by `submitted_at` desc at `templates/mod.rs:193`) — matches concept's "current adoption state". Multi-AR-per-(org,kind) history is preserved in the list endpoint. |
| D6.3 | [`m5_1/drifts/D6.3.md`](../../../v0/implementation/m5_1/drifts/D6.3.md) | MEDIUM | B | discovered → accepted-as-is | §D57.3 | 3-way union bucketing for "standard system agents": canonical slug OR `Organization.system_agents` registry membership OR `AgentRole::System`. Robust against M3-era fixture rows pre-dating M5 canonical slugs. |
| D7.2 | [`m5_1/drifts/D7.2.md`](../../../v0/implementation/m5_1/drifts/D7.2.md) | LOW | B | discovered → accepted-as-is | §D57.4 | `--model-config-id` ships ADDITIVELY alongside `--patch-json` (mutually exclusive at runtime, both reach `PATCH /api/v0/agents/:id/profile`). Backward-compatible with M4 `--patch-json` operators. |
| D7.4 | [`m5_1/drifts/D7.4.md`](../../../v0/implementation/m5_1/drifts/D7.4.md) | LOW | B | discovered → accepted-as-is | §D57.5 | Page-11 "Recent sessions" retrofit is web-side parallel-fetch (not server-detail mutation). Server-side `ProjectDetail.recent_sessions` stays empty at M5; web-side calls `listSessionsInProjectApi`. **Review trigger: M7 project-detail hardening** — server-side promotion may revisit. |
| D7.5 | [`m5_1/drifts/D7.5.md`](../../../v0/implementation/m5_1/drifts/D7.5.md) | MEDIUM | B | discovered → accepted-as-is | §D57.6 | No new web component-tests at P5/P6/P7; web test count stays at 79. **Coverage of the 3 new pages defers to CH-24 Playwright e2e scope.** Test-strategy decision recorded; deferred-marker → CH-24 Playwright. |
| D-new-21 | [`m5_1/drifts/D-new-21.md`](../../../v0/implementation/m5_1/drifts/D-new-21.md) | LOW | B | discovered → accepted-as-is | §D57.7 | Edge-count: code is `EDGE_KIND_NAMES: [&str; 71]` (test-asserted invariant at `edges.rs:661`); docstring at `edges.rs:1,12,25` says "69"; concept doc `ontology.md:87` says "66". **Doc-side reconcile to 71** (canonical count = test-asserted invariant). `ontology.md` line 87 + `edges.rs` lines 1, 12, 25 updated to 71 to match the invariant. |
| D-new-25 | [`m5_1/drifts/D-new-25.md`](../../../v0/implementation/m5_1/drifts/D-new-25.md) | MEDIUM | B | discovered → accepted-as-is | §D57.8 | InboxObject/OutboxObject `messages: Vec<AgentMessage>` field deferred to M6-DEFERRED-02 (inter-agent messaging). Concept-doc refresh adds 1-line deferred-state footnote at `ontology.md:82-83` referencing CH-19 + M6-DEFERRED-02. **Drift's own `Implementation chunk this belongs to` field stays `M6-DEFERRED-02`** — CH-19 ratifies the deferral, does NOT reassign. |
| D-new-27 | [`m5_1/drifts/D-new-27.md`](../../../v0/implementation/m5_1/drifts/D-new-27.md) | MEDIUM | B (originally classified C; treated as B per forward-scope row 180) | discovered → accepted-as-is | §D57.9 | Token-economy fields (`rating_window`, `total_tokens_earned/consumed`, Worth) deferred to M6-or-M7-DEFERRED. Concept-doc refresh adds 1-line deferred-state footnote at `token-economy.md` §"Worth" preamble. |
| D-new-30 | [`m5_1/drifts/D-new-30.md`](../../../v0/implementation/m5_1/drifts/D-new-30.md) | LOW | B (originally classified C) | discovered → accepted-as-is | §D57.10 | Org/Project template-as-config: shipped pattern uses adoption ARs + listener-fired Grants (M3/M5 pattern) — functionally equivalent to YAML config-object. Concept-doc refresh in `permissions/07-templates-and-tools.md` §"Standard Organization Template" + §"Standard Project Template" preambles documents the YAML-as-conceptual-contract / AR-as-implementation-surface mapping. |

**Drift bucket reconciliation:** The forward-scope row (line 180) lists D-new-27 + D-new-30 alongside the other Bucket-B drifts despite their drift files classifying them as Bucket C. The forward-scope assignment is the binding scope-decision; this plan honors it. Drift files' `Bucket` field stays as classified (C); the lifecycle-history entries note the CH-19 chunk-claim regardless of bucket.

**Drift transition discipline (per `drift-lifecycle.md:118-133`):** `accepted-as-is` requires (a) Accepted ADR documenting the drift ID, (b) explicit risk-acceptance statement, (c) review trigger. ADR-0057 covers (a) for all ten via §D57.1–§D57.10; risk-acceptance statement appears once in ADR §"Risk acceptance" (consolidated; ten drifts share Bucket-B characteristics — shipped convention works, cost of code-side remediation outweighs benefit at M5 close); review triggers are per-drift (D7.4 → M7 project-detail; D-new-25 → M6-DEFERRED-02; D-new-27 → M6-or-M7-DEFERRED; D7.5 → CH-24 Playwright; the rest → no near-term review trigger; review-on-demand via `_concept-audit-matrix.md` if concept-doc text changes upstream).

---

## §5 — ADRs drafted

**ADR number assignment (Q6 procedure executed at draft-time):** Ran `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/*/decisions/*.md | xargs -I{} basename {} .md | grep -oE "ADR-[0-9]+" | sort -u | tail -5` (functional equivalent: `ls m5_2/decisions/ | tail`) → returned `0056-auth-request-per-state-acl-enforcement.md` as highest. **Next-free: ADR-0057.** Home: `m5_2/decisions/0057-bucket-b-convention-ratification.md`.

**ADR-0057 — Bucket B convention ratification (10 sub-decisions covering audit-event placement + AR-resolution semantics + system-agent bucketing + CLI flag shape + web retrofit location + test-strategy + edge-count + M6 deferrals + template-as-config framing).**

- **Status at chunk-plan draft:** Proposed.
- **Flip to Accepted:** at chunk-seal (P3 deliverable per §7 below). Single ADR; ten sub-decisions all flip together.
- **Drafted-at-phase:** P1 (ADR body + sub-decisions written); flipped to Accepted at P3 (chunk-seal).
- **Decision-summary (one line):** Ratify the ten Bucket-B shape-choice conventions shipped through M5/P5–P7 close + the two M6+ deferred-scope items, with explicit risk-acceptance + per-drift review triggers.
- **Closes:** D5.2 (§D57.1), D5.3 (§D57.2), D6.3 (§D57.3), D7.2 (§D57.4), D7.4 (§D57.5), D7.5 (§D57.6), D-new-21 (§D57.7), D-new-25 (§D57.8), D-new-27 (§D57.9), D-new-30 (§D57.10) — all transition `discovered → accepted-as-is`.

**ADR-body checklist (v2026-05-04 per CH-13 retrospective; v2026-05-08 per CH-14 retrospective Row 10):**

1. **§"Forks" header with explicit user-lock outcome.** Single line: *"Forks (none requiring user-lock — doc-only ratification chunk; F1 / F2 / F3 resolved at planner-recommendation level via existing precedent ADR-0042 + Q7 uniform doc-only ritual)."* Direct-approval cycle.

2. **§"Cross-references" with all 4 categories.**
   - **(a) Originating concept-docs:** [`permissions/02-auth-request.md` §"Auth Request Lifecycle"](../../../v0/concepts/permissions/02-auth-request.md), [`permissions/07-templates-and-tools.md` §"Standard Permission Templates"](../../../v0/concepts/permissions/07-templates-and-tools.md), [`system-agents.md` §"Properties Shared by All System Agents"](../../../v0/concepts/system-agents.md), [`agent.md` §"Soul"](../../../v0/concepts/agent.md), [`project.md` §"Project (Node Type)"](../../../v0/concepts/project.md), [`ontology.md` §"Edge Types"](../../../v0/concepts/ontology.md), [`token-economy.md` §"Worth"](../../../v0/concepts/token-economy.md).
   - **(b) Closed drifts:** D5.2, D5.3, D6.3, D7.2, D7.4, D7.5, D-new-21, D-new-25, D-new-27, D-new-30.
   - **(c) Prior ADRs cited as precedent (MILESTONE-PREFIXED per CH-08 retro Row 1):**
     - [`m5_2/decisions/0042-storage-backend-configurable.md`](../../../v0/implementation/m5_2/decisions/0042-storage-backend-configurable.md) (CH-03; doc-only-ratification-chunk shape precedent).
     - [`m5_2/decisions/0033-k8s-prep-refactors.md`](../../../v0/implementation/m5_2/decisions/0033-k8s-prep-refactors.md) (CH-K8S-PREP; §D33.4 single-AuditEmitter-writer guarantee that D5.2's convention preserves).
     - [`m4/decisions/0028-domain-event-bus.md`](../../../v0/implementation/m4/decisions/0028-domain-event-bus.md) (M4; Template-A fire-listener pattern that D5.2 ratifies the audit-event-placement of).
     - [`m5_2/decisions/0046-template-cd-http-edges.md`](../../../v0/implementation/m5_2/decisions/0046-template-cd-http-edges.md) (CH-23; Template C/D listener-pattern symmetric with D5.2's domain-tier convention).
     - [`m5_2/decisions/0050-audit-class-composition-strictest-wins.md`](../../../v0/implementation/m5_2/decisions/0050-audit-class-composition-strictest-wins.md) (CH-13; audit-event diff `audit_class_source` attribution that crosses both D5.2 tiers — domain-tier listeners + platform-tier handlers).
     - [`m3/decisions/0022-org-creation-compound-transaction.md`](../../../v0/implementation/m3/decisions/0022-org-creation-compound-transaction.md) (M3; D6.3 union-bucketing's M3-era-fixture origin).
   - **(d) Forward-scope row:** [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md` lines 179-183](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (CH-19 row); §5 severity table line 427.

3. **Pre-existing-behaviour preservation note (CH-14 retro Row 10):** ADR-0057 does NOT change runtime behaviour — every sub-decision documents a convention SHIPPED at M5/P5–P7 close. Format applied per sub-decision: each §D57.N body opens with *"Shipped at M5/P<n> close (date YYYY-MM-DD); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at `<file:line>`."* — the audit-trail makes explicit which behaviours are now relied-upon invariants.

**ADR cross-reference precedent map (planner self-discipline):** ADR-0057 is the **second consolidated convention-ratification ADR** in baby-phi (after ADR-0042 for CH-03's storage-backend ratification). The shape-precedent is binding for any future `Bucket B` or `Bucket C` ratification chunks (e.g., CH-20 — see §6).

---

## §6 — Prior-chunk regression re-verification

Doc-only chunk → minimal upstream invariants. The ten drifts' shipped conventions all live in code that prior chunks (M3 / M4 / M5/P5–P7 era) shipped. CH-19 verifies those conventions still hold at chunk-open AND at chunk-seal (per template §6 rule).

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| M5/P5 (templates) | `server::platform::templates::audit_events` exists with 4 builder fns + 4 unit tests | `ls /root/projects/phi/baby-phi/modules/crates/server/src/platform/templates/audit_events.rs` (expect file exists) |
| M5/P5 (templates) | `find_adoption_ar` sorts by `submitted_at` desc and returns `.first().cloned()` | `grep -n "submitted_at\|sort_by" /root/projects/phi/baby-phi/modules/crates/server/src/platform/templates/mod.rs \| head -3` (expect ≥ 1 hit on `sort_by_key.*Reverse(ar.submitted_at)`) |
| M5/P6 (system-agents) | 3-way union bucketing in `system_agents/list.rs` | `grep -n "system_agent_ids.contains\|AgentRole::System" /root/projects/phi/baby-phi/modules/crates/server/src/platform/system_agents/list.rs` (expect ≥ 2 hits) |
| M5/P7 (CLI) | `--model-config-id` + `--patch-json` coexist as `Option<String>` | `grep -n "model_config_id\|patch_json" /root/projects/phi/baby-phi/modules/crates/cli/src/main.rs \| head -3` (expect ≥ 2 hits) |
| M5/P7 (web) | Web-side parallel-fetch via `listSessionsInProjectApi` | `grep -n "listSessionsInProjectApi" "/root/projects/phi/baby-phi/modules/web/app/(admin)/organizations/[id]/projects/[project_id]/page.tsx"` (expect ≥ 1 hit) |
| M5/P7 (web tests) | Web test count stays at 79 (no new component tests) | (informational only — Playwright deferral; no in-cycle re-verification) |
| M4/P1 (edges) | `EDGE_KIND_NAMES.len() == 71` test-asserted invariant | `grep -n "EDGE_KIND_NAMES.len\|EDGE_KIND_NAMES: \[&str;" /root/projects/phi/baby-phi/modules/crates/domain/src/model/edges.rs` (expect ≥ 2 hits including `[&str; 71]`) |
| M3/P1 (composites) | InboxObject + OutboxObject minimal scaffolds at `nodes.rs:772-784` | `grep -nE "pub struct (InboxObject\|OutboxObject)" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs \| head` (expect 2 hits) |
| M0+ (Agent struct) | Agent struct does NOT carry token-economy fields | `grep -nE "rating_window\|total_tokens_earned\|total_tokens_consumed" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` (expect 0 hits) |
| M3+ (templates) | Template-adoption-AR pattern in use; no embedded YAML config object on Org/Project | `grep -nE "template_yaml_config\|tools_allowlist:" /root/projects/phi/baby-phi/modules/crates/domain/src/model/nodes.rs` (expect 0 hits) |

**Pre-CH-19 chunk-open verification (run at gate-1 before ExitPlanMode):** all ten greps above MUST return their expected counts. If any deviate, the corresponding drift's `Reality (shipped state at current HEAD)` description has drifted since drift-file authoring (2026-04-23 / 2026-04-24); planner re-spawn may be required to refresh §2 + §4. **Verified at chunk-open 2026-05-10:** all ten greps run + returned expected counts (citations in §2 + §4 above are anchored to CURRENT-HEAD line numbers).

---

## §7 — Phases within the chunk

Three phases: P1 (ADR + concept-doc refresh), P2 (drift-file lifecycle + matrix), P3 (chunk-seal paperwork + status flips).

### P1 — ADR-0057 draft + concept-doc refresh paragraphs

**Goal.** Author ADR-0057 with all ten sub-decisions; add targeted refresh paragraphs to 6 concept docs at named §-anchors per §2. Single coherent doc-write phase; no code touched.

**Deliverables.**
1. Write `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md`. Status: **Proposed** at this phase. Sections: top-matter (status / date / chunk / closes) + Context + 10 sub-decisions D57.1–D57.10 + Risk acceptance + Forks + Cross-references + Pre-existing-behaviour preservation. Word-count target: ≤ 4000 words (consistent with ADR-0042's 3500).
2. Refresh paragraph at `concepts/permissions/02-auth-request.md` §"Interaction with Authority Templates" (line 210) — multi-AR-per-(org,kind) history note (D5.3).
3. Refresh paragraph at `concepts/permissions/07-templates-and-tools.md` §"Standard Organization Template" preamble (line 74) AND §"Standard Project Template" preamble (line 274) — YAML-as-conceptual-contract / AR-as-implementation-surface mapping (D-new-30).
4. Refresh paragraph at `concepts/permissions/07-templates-and-tools.md` §"Templates Are Pre-Authorized Allocations" tail (line 37 anchor) — multi-adoption-AR cross-ref note (D5.3 secondary).
5. Refresh paragraph at `concepts/system-agents.md` §"Properties Shared by All System Agents" tail (line 21+) — 3-way union bucketing convention (D6.3).
6. Refresh paragraph at `concepts/agent.md` §"Soul (Immutable Born Structure)" tail (line 158+) — `--patch-json` + `--model-config-id` additive-flag convention (D7.2).
7. NEW sub-§ at `concepts/project.md` §"Project (Node Type)" tail (line 13+) titled "Recent sessions UI surface (web-side at v0.1)" — D7.4 ratification + M7 review-trigger note.
8. Update `concepts/ontology.md` §"Edge Types" header at line 87 from `(66 total)` to `(71 total)` — D-new-21.
9. Update `modules/crates/domain/src/model/edges.rs` lines 1, 12, 25 docstring "69" → "71" — D-new-21 (code-comment-only, no code-behaviour change). **NOTE: this is the SOLE code-side touch of the cycle.** Single-doc-comment-only — no `cargo test` impact, no clippy impact, no runtime behaviour change. Caught at the audit-cycle: `EDGE_KIND_NAMES: [&str; 71]` test invariant ALREADY enforces the canonical count; CH-19 reconciles the docstring without changing the test.
10. 1-line deferred-state footnote at `concepts/ontology.md` line 82-83 (InboxObject/OutboxObject row) — D-new-25 → M6-DEFERRED-02.
11. 1-line deferred-state footnote at `concepts/token-economy.md` §"Worth" preamble (line 38) — D-new-27 → M6-or-M7-DEFERRED.

**Tests.** None — no test changes.

**Concept-alignment check.** Every §2 row's status flips at this phase from open-state → target-status (target-status will be `honored` for 5 rows, `accepted-as-is` for 5 rows; verified by re-reading the refresh paragraph against the concept-doc surrounding prose for each).

**phi-core leverage check.** N/A — no phi-core changes.

**User-facing doc updates.** Per §3.C: no user-facing-tier doc updates land in P1 (deferred to M5-tag-close batch per §3.C); ADR-0057 IS the architecture-tier surface (precedent: ADR-0042 served same role at CH-03).

**Confidence target.** ≥ 99% (ADR + concept-doc refresh paragraphs are pure prose; high precision achievable). Deliverable count denominator: 11 deliverables; numerator at phase-close: 11.

**Pause discipline.** Pause via `AskUserQuestion` if (a) any concept-doc refresh paragraph mid-flight surfaces a contradiction with EXISTING concept-doc text NOT anticipated in §2 (would require concept-doc-renegotiation, not refresh — escalates beyond Bucket B); (b) any drift's `Reality (shipped state at current HEAD)` description has drifted since drift-file authoring (would require §4 refresh + planner re-spawn); (c) ADR-0057 sub-decision word count exceeds 500 words for any single sub-decision (would suggest the sub-decision is harboring non-Bucket-B scope — re-evaluate against drift-file body before continuing).

### P2 — Drift-file lifecycle entries + concept-audit matrix refreshes + drifts/README.md

**Goal.** Update every drift file's `Status` field + `Lifecycle history` block + `Last verified` header; refresh `_concept-audit-matrix.md` rows for every claim flipping status at chunk-close; update `drifts/README.md` index Status column.

**Deliverables.**
1. Update `Status` field in 10 drift files: `discovered` → `accepted-as-is`. Update `Last verified: YYYY-MM-DD by Claude Code` header on each (date 2026-05-10).
2. Append lifecycle-history entry to each drift file body:
   ```
   - 2026-05-10 — `accepted-as-is` — ratified via CH-19 / ADR-0057 §D57.<N>; review trigger: <per-drift trigger>.
   ```
   Per-drift triggers: D5.2 → no near-term; D5.3 → no near-term; D6.3 → M7b slug-reservation if pursued; D7.2 → no near-term; D7.4 → M7 project-detail hardening; D7.5 → CH-24 Playwright; D-new-21 → no near-term (canonical count locked); D-new-25 → M6-DEFERRED-02; D-new-27 → M6-or-M7-DEFERRED; D-new-30 → no near-term.
3. Update `_concept-audit-matrix.md` row Status + Code-evidence + Covering-drift columns for each row touched per §2 table. **Letter-for-letter status flip per CH-12 retro Row 1 P4 paperwork addendum** — copy-paste the `Target status at chunk-close` value from §2 verbatim into the matrix. Add CH-19 verified-header at top of `_concept-audit-matrix.md` per the existing pattern (line 1-9 of the matrix file already carries 9 headers; CH-19 prepends a 10th).
4. Update `drifts/README.md` index Status column to `accepted-as-is ✓ (CH-19 / ADR-0057 §D57.<N>)` for the ten drift rows.

**Tests.** None.

**Concept-alignment check.** Every §2 row's matrix-side reflection is updated to match the chunk-close target status.

**phi-core leverage check.** N/A.

**User-facing doc updates.** None (per §3.C).

**Confidence target.** ≥ 99%.

**Pause discipline.** Pause if any drift file's existing lifecycle-history is ambiguous about prior state (e.g., missing `classified` entry) — surface to user before silently inserting `accepted-as-is`. Pause if `_concept-audit-matrix.md` rows referenced in §2 don't exist in the current matrix (would mean §2 cited a stale row and matrix-side doesn't carry the claim).

### P3 — Chunk-seal: ADR Accepted + cycle-index row + verified-header bumps

**Goal.** Flip ADR-0057 from Proposed → Accepted; insert cycle-index row at `_cycle-index.md`; bump verified-headers on every touched concept doc per `Documentation Alignment` discipline; final P4 paperwork checklist.

**Deliverables.**
1. Flip ADR-0057 `Status: Proposed` → `Status: Accepted` in `m5_2/decisions/0057-bucket-b-convention-ratification.md`.
2. Insert row in [`_cycle-index.md`](../../_cycle-index.md) "Active cycles" table per CH-17 retro Row 4 paperwork rule:
   ```
   | [`2c520ba7`](ch-19-bucket-b-ratification-2c520ba7/plan.md) | CH-19 — Bucket B ratification (closes 10 drifts D5.2/D5.3/D6.3/D7.2/D7.4/D7.5/D-new-21/D-new-25/D-new-27/D-new-30) | 3 | 1 (audit envelope: small per plan §11) | <iter count from gate-3> | <status> | <retro link> |
   ```
3. Bump `<!-- Last verified: 2026-05-10 by Claude Code (CH-19 amendment — <one-line summary>) -->` header on each touched concept doc (7 docs per §2). Body changes are P1 deliverables; this step ONLY bumps the verified-header description.
4. Bump `Last verified` header on the 10 drift files + `_concept-audit-matrix.md` + `drifts/README.md`.
5. Run §12 verification recipe end-to-end → confirm 4 CI guards green; confirm `cargo test --workspace` test count UNCHANGED (1529 — same as CH-18 close); confirm `clippy --all-targets` green; confirm phi-core baseline 57.
6. Append CH-19 chunk-seal lifecycle entry to ADR file (Status block reflects Accepted; date stamped 2026-05-10).

**Tests.** Run full workspace test as gate-4 sanity check (one-shot at chunk-seal; doc-only chunk does NOT need per-phase test runs). Expected: **1529** (same as CH-18 close per `_cycle-index.md` line "1529/0/2"; doc-only chunk → Δ +0). Run `cargo clean` immediately after the test per CH-18 retro Row 1 / chunk-implementer v8 immediate-post-test cleanup discipline.

**Concept-alignment check.** Re-walk §2 table. All 10 rows confirmed at chunk-close target status.

**phi-core leverage check.** Re-run canonical baseline grep; expect 57; verify Δ +0.

**User-facing doc updates.** None (per §3.C — deferred to M5-tag-close).

**Confidence target.** ≥ 99% composite.

**Pause discipline.** Pause if (a) `cargo test --workspace` test count diverges from 1529 (would mean a concurrent change has landed; doc-only chunk should never change test count); (b) clippy/fmt fail (doc-only should not affect either); (c) check-doc-links.sh fails (most likely failure mode for a doc-heavy chunk; resolve all link hits before sealing).

---

## §8 — Tests summary

**Expected total test count at chunk close:** **1529** (same as CH-18 chunk-close; doc-only chunk → Δ +0).

**Plan §8 v2 band: [1529, 1529].** Asymmetric band (CH-12 retro discipline) is degenerate here — no new tests in scope, so deliverable-listed sum × 1.0 = lower bound = 1529; deliverable-listed sum × 1.20 = upper bound = 1529 (rounding to 1529 since 0 new tests means buffer applies to 0). **Outside the band → AskUserQuestion** per template §8 rule.

**MUST-SHIP / MAY-COVER split (CH-17 retro Row 6):**
- **MUST-SHIP:** none. Doc-only chunk; no new test files in scope.
- **MAY-COVER:** none. Doc-only chunk; no band-floor surrogates in scope.

**Layer breakdown:**
- Unit: 0 added.
- Integration: 0 added.
- Acceptance: 0 added.
- e2e: 0 added.

**Named test files:** none added.

**Named expected-still-green tests:**
- `domain::model::edges::tests::edge_kind_names_count` (`edges.rs:661` — `assert_eq!(EDGE_KIND_NAMES.len(), 71)`). The only test directly proving the D-new-21 canonical count; CH-19 only updates the surrounding docstring + the concept-doc count, NOT the assertion. Verified still green at gate-4.
- `cli::tests::completion_help::completion_scripts_expose_m5_p7_agent_update_model_config_id_flag` (per D7.2 drift body line 31) — pins the additive `--patch-json` + `--model-config-id` shape. CH-19 ratifies the convention; the test continues to enforce it.
- `acceptance_authority_templates::*` adopt→revoke→re-adopt scenarios (per D5.3 drift body line 31) — exercise the multi-AR-per-(org,kind) history path. CH-19 ratifies the convention; tests continue to pass.
- `acceptance_system_agents::list_standard_agents_includes_fixture_seeded_rows` (per D6.3 drift body line 31 — within the 8 P6 scenarios) — exercises the 3-way union bucketing. CH-19 ratifies; test continues to pass.

**Buffer ceiling:** N/A — doc-only chunk does not have an Artifact-C-cascade test-amendment scope (CH-17 retro Row 5 ×1.30 ceiling does not apply).

---

## §9 — Pre-chunk gate

**Reading list (mandatory) — verified COMPLETE at chunk-open 2026-05-10 by planner:**

1. **Concept docs cited in §2** (read in full or relevant §):
   - [`permissions/02-auth-request.md`](../../../v0/concepts/permissions/02-auth-request.md) §"Auth Request Lifecycle" + §"Interaction with Authority Templates" + §"Per-State Access Matrix"
   - [`permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) §"Standard Permission Templates" + §"Templates Are Pre-Authorized Allocations" + §"Standard Organization Template" + §"Standard Project Template"
   - [`system-agents.md`](../../../v0/concepts/system-agents.md) §"Purpose" + §"Properties Shared by All System Agents" + §"Memory Extraction Agent" + §"Agent Catalog Agent"
   - [`agent.md`](../../../v0/concepts/agent.md) §"Soul (Immutable Born Structure)"
   - [`project.md`](../../../v0/concepts/project.md) §"Project (Node Type)"
   - [`ontology.md`](../../../v0/concepts/ontology.md) §"Edge Types" + §"Node Types — Social Structure (phi extensions)" + §"Value Objects"
   - [`token-economy.md`](../../../v0/concepts/token-economy.md) §"Worth" + §"Rating Window"
   - [`permissions/README.md`](../../../v0/concepts/permissions/README.md) (entry-invariants source per §2 hook rule)
2. **Drift files cited in §4** (10 files):
   - D5.2, D5.3, D6.3, D7.2, D7.4, D7.5, D-new-21, D-new-25, D-new-27, D-new-30 — all under [`v0/implementation/m5_1/drifts/`](../../../v0/implementation/m5_1/drifts/)
3. **Prior-chunk plans cited in §6** — N/A (no prior-chunk plan needed; M5/P5–P7 plan archives are referenced via in-line line numbers in drift bodies).
4. [forward-scope §1 + §5 + §7](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) — the chunk row + Q4 (per-chunk ordering) + Q5 (M5-scope defer) + Q7 (uniform doc-only ritual) + Q9 (user-facing doc impact map).
5. [`baby-phi/CLAUDE.md`](../../../../../CLAUDE.md) phi-core Leverage section.
6. **Tag-write Repository conditional (v3 per CH-12 retrospective Row 5):** N/A — CH-19 does NOT introduce or reference a new tag-write Repository method.
7. **Engine.rs Step-N body conditional (v3 per CH-11 retrospective):** N/A — CH-19 does NOT touch `domain::permissions::engine` Step N body.

**Carry-forward invariants (verified green at chunk-open 2026-05-10):**
- `cargo test --workspace` test count: **1529** at HEAD (per CH-18 cycle-audit). Verified against `_cycle-index.md` row for CH-18 — `1529/0/2 within plan §8 v2 band`. Doc-only chunk → expected unchanged at chunk-close.
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh`: green (verified at CH-18 cycle close).
- `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`: green (verified at CH-18 cycle close; CH-19 will exercise this guard at gate-4 since chunk is doc-heavy).
- `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh`: green (verified at CH-18 cycle close).
- `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh`: green (verified at CH-18 cycle close).
- `modules/` git diff against chunk-open HEAD: empty (verified — no preload edits).

**Pending decisions carried into this chunk:**
- Q4 (chunk-ordering): user-decided per-chunk; CH-19 selected by user as next chunk to open (post-CH-18). No predecessor required.
- Q5 (M5-scope defer): CH-19 closes 10 Bucket-B drifts (6 MEDIUM + 4 LOW); ratification path keeps all in M5 close (per Q5 LOW-drifts-via-CH-19/CH-20 rule); MEDIUM-severity drifts here are doc-only ratifications, not code-side defers.
- Q9 (user-facing doc impact map): per §3.C above, no user-facing-tier doc updates in CH-19 scope; M5-tag-close successor for the 1-line CLI reference cross-ref.

**Drift-file `discovered → classified → scoped` transitions:** All ten drifts ARE already at `discovered` per their `Status` field as of chunk-open. They were classified during M5.1/P2 concept-audit. They are scoped by APPEARING in this plan §4. CH-19 advances them through `in-chunk-plan` → `accepted-as-is` per §7 P2/P3 deliverables.

---

## §10 — Close criteria

**4 aspects:**

- **Code aspect.** No code changes EXCEPT the 3 docstring lines in `edges.rs` (lines 1, 12, 25 — "69" → "71" reconcile). cargo test workspace passes at 1529. clippy green under `RUSTFLAGS="-Dwarnings"`. fmt --check green. check-phi-core-reuse.sh green.
- **Docs aspect — governance tier.** ADR-0057 Status: Accepted; 10 drift files Status: accepted-as-is + lifecycle-history entries; `_concept-audit-matrix.md` rows refreshed letter-for-letter from §2 targets per CH-12 retro Row 1; `drifts/README.md` index refreshed; verified-headers bumped on all touched docs.
- **Docs aspect — user-facing tier (post-CH-22).** Per §3.C: 0 user-facing-tier files updated in-chunk; 1 file (`m5/user-guide/cli-reference-m5.md`) deferred to M5-tag-close batch with explicit successor-chunk-reference. Defer-decision documented in §3.C row 3.
- **phi-core leverage aspect.** §3 import-count delta = +0 (predicted); actual at chunk-close: 57 (verified via canonical grep). Forbidden-duplication greps: N/A. check-phi-core-reuse.sh green.
- **Concept alignment aspect.** Every §2 row at chunk-close target status; none remains `contradicted` (no row was contradicted at chunk-open).

**2 confidence %:**
- **Implementation confidence %** = `claims-honored / claims-in-scope`. Claims-in-scope: 10 drift conventions + 11 P1 deliverables + 4 P2 deliverables + 6 P3 deliverables = 31 claims. Target: ≥ 30/31 = 96.7% (≥ 9.7/10 — MEETS plan auto-approval threshold of 9/10). The 1 acceptable miss is reserved for any single concept-doc refresh paragraph that fails the audit's "independent-reader-can-cross-check" test (gets demoted to "rephrasing required" before chunk-seal). **Target: 31/31 = 100%** at chunk-close.
- **Documentation confidence %** = `doc-pages-cross-checkable-without-ambiguity / doc-pages-touched-in-chunk`. Doc-pages touched: ADR-0057 (1) + 7 concept docs (7) + 10 drift files (10) + `_concept-audit-matrix.md` (1) + `drifts/README.md` (1) + `_cycle-index.md` (1) = 21 doc pages. Target: 21/21 = 100%.

**Composite =** `min(impl%, doc%, code-aspect-binary, governance-docs-binary, user-facing-docs-binary, phi-core-binary, concept-alignment-binary)`. **Target: 100%.**

**Explicit close-target discipline:** close report states ALL FIVE measures with named numerators/denominators (per template §10 rule).

**P4 paperwork checklist (CH-11 retro v2026-05-03 + CH-12 retro v2026-05-04):**
- For every modified doc with verified-header (line 1 `<!-- Last verified: ... -->`): confirm the new header description matches the body diff exactly. Mismatch → fix the header before chunk-seal. (≥ 21 docs apply.)
- For every `_concept-audit-matrix.md` row touched: new Status column value MUST be copy-pasted letter-for-letter from plan §2 target column for that row.

**Cargo-clean discipline (CH-18 retro Row 1, USER DIRECTIVE 2026-05-10, TWO placements):**
- (1) Immediate-post-test cleanup: AFTER P3's gate-4 sanity-check `cargo test --workspace` invocation, the implementer MUST run `/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` BEFORE issuing the next cargo invocation. Capture disk-reclaim metric in cycle-audit §7.
- (2) Gate-5-close final cleanup: orchestrator runs `cargo clean` as last action of gate-5 close (post-retro, pre-commit), per existing CLAUDE.md gate-5 rule.

**Cycle-index P-seal rule (CH-17 retro Row 4):** P3 deliverable §7 includes inserting the CH-19 row in `_cycle-index.md` "Active cycles" table — verified at chunk-seal via `grep -n 2c520ba7 _cycle-index.md` returning ≥ 1 hit.

---

## §11 — Post-chunk independent audit plan

**Phase count: 3** (P1 + P2 + P3). Per the audit-envelope-size skill table:
- ≤ 2 phases → Small (1 auditor)
- **3–5 phases → Medium (2 auditors)** — applies here.
- 6+ phases → Large (3 auditors).

**Audit envelope: Medium (2 auditors).** Letters: A (code + phi-core + K8s) + B (concept + docs + ADR).

**Reasoning:** doc-only chunk with 3 phases technically falls in the medium tier. Audit A's surface is light (one docstring touch in `edges.rs`; no production code changes; phi-core delta 0; K8s axes all `no impact`); Audit B's surface is heavy (ADR + 7 concept docs + 10 drift files + matrix + README). The asymmetric audit weight matches the chunk shape.

**Note on parity-pair:** CH-17 (4 phases, Small per skill but 2 auditors dispatched per orchestrator parity-pair precedent). CH-19 is at the Medium boundary; orchestrator may stay at 2 auditors per skill OR consolidate to 1 if Audit A's code surface is judged trivially-small (single docstring touch). **Recommendation: 2 auditors (per skill).**

### Audit A (code + phi-core + K8s) scaffold

```
You are auditing CH-19 in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at docs/specs/plan/build/ch-19-bucket-b-ratification-2c520ba7/plan.md.

Verify each claim with file:line citation:
1. Single code-side touch is `modules/crates/domain/src/model/edges.rs` lines 1, 12, 25 docstring "69" → "71" reconcile (no behavioural change). Run `git diff main -- modules/crates/domain/src/model/edges.rs` — expect ≤ 3 changed lines, all in the docstring/comment region (no enum-variant additions; no test changes).
2. `EDGE_KIND_NAMES.len() == 71` test invariant at `domain/src/model/edges.rs:661` STILL GREEN — run `cargo test -p domain --test <inline> -- edge_kind_names_count` OR `cargo test -p domain edge_kind_names` (auto-discover the test).
3. phi-core leverage delta: predicted +0; actual at audit-time. Run `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect 57 (canonical baseline preserved).
4. cargo test --workspace -- --test-threads=1 green at expected count 1529 (CH-19 doc-only → Δ +0 from CH-18 close).
5. CI guards green: `bash scripts/check-phi-core-reuse.sh` exit 0; `bash scripts/check-doc-links.sh` exit 0 (heavy chunk; doc-link integrity load-bearing); `bash scripts/check-ops-doc-headers.sh` exit 0; `bash scripts/check-spec-drift.sh` exit 0.
6. K8s 7-axis classification per plan §3.B all `no impact`. Confirm A7 single-writer guarantee: `grep -rn "AuditEmitter::emit\b" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect ≥ 1 (existing emitter present, not duplicated).
7. Prior-chunk regression: re-run all 10 §6 grep commands; verify each returns expected count.
8. clippy --workspace --all-targets under RUSTFLAGS="-Dwarnings" green.

After cargo test run: MUST cargo clean immediately per chunk-auditor v7 / CH-18 retro Row 1 USER DIRECTIVE.

PASS/FAIL each. ≤ 600 words.
```

### Audit B (concept + docs + ADR) scaffold

```
You are auditing CH-19's concept-fidelity + docs-fidelity. Read-only.

Verify each claim:
1. ADR-0057 Accepted at `docs/specs/v0/implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md` with sub-decisions D57.1 through D57.10 — one per drift in §4 of plan.
2. ADR-0057 cross-references include all 4 mandatory categories per CH-13 retro / CH-08 retro / CH-14 retro: (a) originating concept-docs, (b) closed drifts (10 IDs), (c) prior ADRs cited as precedent **MILESTONE-PREFIXED** (per CH-08 retro Row 1) — verify ADR-0042 + ADR-0033 + ADR-0028 (m4) + ADR-0046 + ADR-0050 + ADR-0022 (m3) cited with milestone-prefix paths, (d) forward-scope row at line 179-183.
3. Each of D57.1–D57.10 includes the pre-existing-behaviour preservation note per CH-14 retro Row 10 ("Shipped at M5/P<n> close (date YYYY-MM-DD); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at <file:line>.").
4. 10 drift files Status flipped `discovered` → `accepted-as-is`; lifecycle-history entry appended for each (cite drift ID + line range of new entry); verified-header bumped to 2026-05-10 with CH-19 amendment description.
5. drifts/README.md index Status column refreshed for the 10 drift rows to `accepted-as-is ✓ (CH-19 / ADR-0057 §D57.<N>)`.
6. _concept-audit-matrix.md rows touched per plan §2 — Status column flipped letter-for-letter from §2's "Target status at chunk-close" column per CH-12 retro Row 1 P4 paperwork addendum. Spot-check 5 random rows by re-reading plan §2 + matrix.
7. Concept doc verified-headers bumped on 7 docs: permissions/02-auth-request.md, permissions/07-templates-and-tools.md, system-agents.md, agent.md, project.md, ontology.md, token-economy.md.
8. Concept-doc refresh paragraphs landed at the 7 named §-anchors per plan §7 P1 deliverables 2-11. Verify body actually changed (not just verified-header bump). Spot-check 3 random refresh paragraphs.
9. Doc-sync widened sweep (CH-15 retro Row 1, widened): grep ALL `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` (NOT just plan §3.C-listed files) for stale-narrative phrase set: `FOLLOWUP-NN`, `deferred per`, `is NOT emitted`, `not emitted at CH-NN`, `advisory at M5`, `Step 0 only blocking`, `M6+ tightens the gate`, `at M5/P4`, `not blocking at M5`. Classify each match as legitimate historical context vs stale CH-19 narrative. Doc-only chunk should produce 0 matches needing patch (CH-19 does not change runtime behaviour).
10. `_cycle-index.md` "Active cycles" table contains the CH-19 row — `grep -n 2c520ba7 _cycle-index.md` returns ≥ 1 hit (per CH-17 retro Row 4 P-seal paperwork rule).
11. K8s deferred ledger entry: NO new entry added to `m7b/architecture/deferred-from-ch-k8s-prep.md` (chunk is K8s-neutral).
12. Plan archive at `ch-19-bucket-b-ratification-2c520ba7/plan.md` exists with cycle hex `2c520ba7`.
13. Prior-chunk doc invariants intact: re-verify `_concept-audit-matrix.md` line 1 verified-header trailer is still CH-18 (not silently overwritten); CH-19's verified-header should PREPEND, not REPLACE.
14. Forward-scope-vs-concept-doc precedence (§3.D): no closed-set contradiction claim made by CH-19; verified.

After cargo test run: MUST cargo clean immediately per chunk-auditor v7 / CH-18 retro Row 1 USER DIRECTIVE.

PASS/FAIL each. ≤ 600 words.
```

**Audit pass criteria (template §11 inheritance):**
- Any new drift discovered by the audit → its own drift file created BEFORE chunk seals.
- Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval, or converted to a drift file with explicit future-chunk assignment.
- Chunk seal blocked until audit returns clean on both A + B + all audit-discovered drifts explicitly scoped.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards (4 — all green)
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (cargo capped at -j 4 per feedback_cargo_jobs_cap.md)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets -j 4
/root/rust-env/cargo/bin/cargo test --workspace -j 4

# 2.b Cargo-clean discipline (CH-18 retro Row 1 / USER DIRECTIVE 2026-05-10): immediate-post-test cleanup
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 3. Chunk-specific verification

# 3.a phi-core baseline (canonical form, no trailing :: per CH-15 retro Row 3)
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57 (Δ 0 from CH-18 close)

# 3.b Edge-count canonical invariant
grep -nE "EDGE_KIND_NAMES.len|EDGE_KIND_NAMES: \[&str;" modules/crates/domain/src/model/edges.rs
# Expect: ≥ 2 hits, including line 524 ([&str; 71]) and line 661 (assert_eq! ... 71)

# 3.c Edge-count docstring reconcile
grep -nE "// [Cc]ount.*71|71 edge type|71 total" modules/crates/domain/src/model/edges.rs
# Expect: ≥ 3 hits in lines 1-30 (CH-19 P1 deliverable 9 reconcile)

# 3.d Concept doc edge count
grep -nE "Edge Types \(71" docs/specs/v0/concepts/ontology.md
# Expect: 1 hit at line 87

# 3.e ADR-0057 Accepted
grep -nE "Status.*Accepted" docs/specs/v0/implementation/m5_2/decisions/0057-bucket-b-convention-ratification.md
# Expect: 1 hit

# 3.f 10 drift files at accepted-as-is
for d in D5.2 D5.3 D6.3 D7.2 D7.4 D7.5 D-new-21 D-new-25 D-new-27 D-new-30; do
  grep -lE "Status.*accepted-as-is" docs/specs/v0/implementation/m5_1/drifts/${d}.md && echo "✓ $d" || echo "✗ $d"
done
# Expect: 10 ✓ lines

# 3.g Cycle-index row present (CH-17 retro Row 4 P-seal paperwork)
grep -n 2c520ba7 docs/specs/plan/build/_cycle-index.md
# Expect: ≥ 1 hit

# 4. Drift-file status totals (template §12 rule)
grep -l "Status.*accepted-as-is" docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count> + 10 (10 new accepted-as-is from CH-19)

# 5. Final disk-pressure check (post-cargo-clean reclaim metric)
df -h /root | head -3
du -sh /root/projects/phi/baby-phi/target 2>/dev/null
```

---

## Plan-time forks recap (zero forks for user)

**No forks for user-lock.** All decisions resolved at planner-recommendation level via existing precedent. **Direct-approval criteria expected to hold:**
- ✅ no locked forks needing user input (zero forks)
- ✅ scope ≤ 1.5× forward-scope (1 ADR + 7 concept-doc refresh paragraphs + 10 drift lifecycle entries — matches forward-scope row's "1 consolidated ADR + targeted concept-doc refreshes + no code change")
- ✅ zero phi-core leverage delta (Δ +0)
- ✅ no new K8s blocker class (all 7 axes `no impact`)
- ✅ audit envelope ≤ medium (2 auditors per phase-count rule)
- ✅ confidence ≥ 9/10 (target 31/31 = 100%)
- ✅ no new migration (doc-only chunk)

**Orchestrator: ExitPlanMode auto-approval candidate.**
