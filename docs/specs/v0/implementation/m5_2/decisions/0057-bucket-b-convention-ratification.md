<!-- Last verified: 2026-05-10 by Claude Code (CH-19 P3 chunk-seal — Status flipped Proposed → **Accepted** at chunk-close; cycle hex `2c520ba7`. All 10 sub-decisions §D57.1–§D57.10 ratified: D5.2 audit-event placement / D5.3 multi-AR-per-(org,kind) history / D6.3 system-agent 3-way union bucketing / D7.2 additive `--patch-json` + `--model-config-id` / D7.4 web-side recent-sessions panel / D7.5 web-test-count freeze at 79 / D-new-21 71-edge-type canonical count / D-new-25 ToolPolicy union semantics / D-new-27 listener.rs domain-side wiring / D-new-30 Worth/RatingWindow as Value Objects. Doc-only chunk: zero code changes (3 docstring lines in edges.rs reconciled "69" → "71"); 1529 tests passing within plan §8 band [1529, 1529]; 4 CI guards green; phi-core import baseline preserved at 57. 10 drift files flipped discovered → accepted-as-is at P2.) -->
<!-- Last verified: 2026-05-10 by Claude Code (CH-19 P1 — NEW consolidated ADR ratifying 10 Bucket-B drifts D5.2/D5.3/D6.3/D7.2/D7.4/D7.5/D-new-21/D-new-25/D-new-27/D-new-30 via 10 sub-decisions §D57.1–§D57.10. Status: Proposed at P1; flips to Accepted at P3 chunk-seal. cycle hex `2c520ba7`. ADR shape follows ADR-0042 (CH-03 storage-backend ratification) precedent for "doc-only chunk → single ratifying ADR with multiple sub-decisions". Forks: none requiring user-lock — F1 / F2 / F3 resolved at planner-recommendation level via existing precedent ADR-0042 + Q7 uniform doc-only ritual.) -->

# ADR-0057 — Bucket B convention ratification (10 shape-choice conventions shipped through M5/P5–P7 close)

**Status: Accepted**

**Date:** 2026-05-10
**Chunk:** CH-19
**Closes:**
- [`D5.2`](../../m5_1/drifts/D5.2.md) (MEDIUM, B) — audit-event placement at `server::platform::<page>::audit_events` (HTTP-handler tier) vs `domain::audit::events::mX::*` (state-machine / fire-listener tier). See §D57.1.
- [`D5.3`](../../m5_1/drifts/D5.3.md) (MEDIUM, B) — `find_adoption_ar` returns the most-recent AR per `(org_id, template_kind)` sorted by `submitted_at` desc; multi-AR-per-(org, kind) history is preserved across adopt → revoke → re-adopt cycles. See §D57.2.
- [`D6.3`](../../m5_1/drifts/D6.3.md) (MEDIUM, B) — system-agent "standard" bucketing uses a 3-way union (canonical slug OR `Organization.system_agents` registry membership OR `AgentRole::System`). See §D57.3.
- [`D7.2`](../../m5_1/drifts/D7.2.md) (LOW, B) — `phi agent update` ships both `--patch-json` (M4 surface) and `--model-config-id` (M5/P7 addition) as additive, mutually-exclusive flags reaching the same `PATCH /api/v0/agents/:id/profile` endpoint. See §D57.4.
- [`D7.4`](../../m5_1/drifts/D7.4.md) (LOW, B) — page-11 "Recent sessions" panel implemented web-side (Next.js parallel-fetch via `listSessionsInProjectApi`); server-side `ProjectDetail.recent_sessions` stays empty at M5; M7 project-detail hardening is the review trigger. See §D57.5.
- [`D7.5`](../../m5_1/drifts/D7.5.md) (MEDIUM, B) — no new web component-tests at P5/P6/P7; web test count stays at 79; coverage of the 3 new pages defers to CH-24 Playwright e2e scope. See §D57.6.
- [`D-new-21`](../../m5_1/drifts/D-new-21.md) (LOW, B) — canonical edge-count is **71** (test-asserted invariant `EDGE_KIND_NAMES.len() == 71` at `domain/src/model/edges.rs:661`); `ontology.md` §"Edge Types" header + `edges.rs` docstring lines 1, 12, 25 reconciled to 71. See §D57.7.
- [`D-new-25`](../../m5_1/drifts/D-new-25.md) (MEDIUM, B) — InboxObject + OutboxObject `messages: Vec<AgentMessage>` field deferred to **M6-DEFERRED-02** (inter-agent messaging); v0 scaffolds carry only `id`, `agent_id`, `created_at`. See §D57.8.
- [`D-new-27`](../../m5_1/drifts/D-new-27.md) (MEDIUM, originally C; treated as B per forward-scope row 180) — token-economy fields (`rating_window`, `total_tokens_earned`, `total_tokens_consumed`, derived Worth) deferred to **M6-or-M7-DEFERRED** token-economy chunk (contracts + bidding scope). See §D57.9.
- [`D-new-30`](../../m5_1/drifts/D-new-30.md) (LOW, originally C; treated as B per forward-scope row 180) — Org/Project template-as-config: shipped pattern uses adoption Auth Requests + listener-fired Grants (functionally equivalent to YAML config but framed differently). Concept doc 07's YAML is the **conceptual contract**; AR-and-listener is the **v0.1 implementation surface**. See §D57.10.

All ten drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../m5_1/process/drift-lifecycle.md).

---

## Context

The M5.1/P3 forward-scope inventory split the 60-drift catalogue into three buckets: **A** (load-bearing scope gap), **B** (underspecified shape choice), **C** (convention/pattern decision). Bucket B drifts are the shape choices the implementer made — or that emerged from reality-feedback at phase close — which the original plan/concept-doc text did not pre-specify, but which are NOT concept-doc contradictions. The shipped convention works; what was missing was the formal acceptance + the recoverable record explaining why this shape rather than another.

CH-19 is the dedicated **convention-ratification chunk** for ten Bucket-B drifts, exactly as forward-scope §1 line 177 frames it: *"Convention ratification (doc-only)"*. The chunk produces (a) this consolidated ADR with ten sub-decisions, (b) targeted concept-doc refresh paragraphs at named §-anchors in 6 concept docs (per plan §2 / §7 P1), (c) drift-status flips `discovered → accepted-as-is` for all ten, (d) `_concept-audit-matrix.md` row refreshes for the rows the drifts cover, (e) `drifts/README.md` index refreshes. **No production code changes** apart from a 3-line comment-only docstring reconcile in `domain/src/model/edges.rs` (lines 1, 12, 25 — "69" → "71"); no migrations; no test-count changes; phi-core import baseline preserved at 57.

This ADR is the **second consolidated convention-ratification ADR** in baby-phi (after [ADR-0042](0042-storage-backend-configurable.md) for CH-03's storage-backend ratification). The shape-precedent — "doc-only chunk → single ratifying ADR with multiple sub-decisions" — is binding for any future Bucket-B or Bucket-C ratification chunks. The next such chunk, **CH-20 (M5.1 final close ratification of the C-bucket conventions)**, is expected to follow the same shape.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* CH-19 application: the ten Bucket-B drifts have lived as `discovered`/`classified` in the catalogue since M5/P5–P7 close (April 23–24, 2026); shipping CH-19 closes them by promoting their convention-status from "shipped but undocumented" to "shipped + ADR-0057 + concept-doc refresh paragraph + matrix row honored." The chunk's value is removing silent-convention drift, not changing behaviour.

---

## Forks

**Forks (none requiring user-lock — doc-only ratification chunk; F1 / F2 / F3 resolved at planner-recommendation level via existing precedent [ADR-0042](0042-storage-backend-configurable.md) + Q7 uniform doc-only ritual.)**

Three potentially-forked decisions resolved at the planner-recommendation level at gate-1:

- **F1 — ADR location.** Single consolidated ADR at `m5_2/decisions/0057-bucket-b-convention-ratification.md` (this file). **Resolved: Single ADR.** Rationale: ADR-0042 is the precedent for "doc-only chunk → single ratifying ADR with multiple sub-decisions"; ADR-0057 follows the same shape with 10 sub-decisions. Splitting across milestone-homed ADRs (one per drift) would (a) create 10 thin ADRs that mostly cross-ref the source code without adding new decision-substance, (b) fragment the audit trail across `m5/decisions/` and `m5_2/decisions/`, (c) violate CH-08 retro Row 1's milestone-prefixed-cross-references discipline (each split ADR would need to cross-ref the others). Single consolidated ADR matches the forward-scope row's literal wording: *"1 consolidated ADR covering audit-event placement + bucketing convention + retrofit location + test-strategy"*.

- **F2 — Concept-doc refresh granularity.** Targeted 1-paragraph refresh notes added at named §-anchors in 6 concept docs (not full §-level prose rewrites). **Resolved: targeted paragraphs.** Rationale: the drifts being ratified are concept-silent-plan-filled-gap (8/10) or concept-aspirational (2/10) — concept docs are not contradicted, just incomplete or below-granularity. Targeted refresh paragraphs add the missing implementation-shape note without reframing settled prose. ADR-0042 §"Doc updates" is the precedent (1-paragraph "v0.1 ships SurrealDB; backend is configurable" added to `coordination.md` §"Storage backend"; full design rationale stayed in the ADR body).

- **F3 — Drift transition target.** All ten drifts transition `discovered → accepted-as-is` per [`drift-lifecycle.md:118-133`](../../m5_1/process/drift-lifecycle.md). **Resolved: `accepted-as-is`.** Rationale: Bucket B drifts ARE the "ratify the existing convention" bucket; no remediation (would be `remediated`); no concept-doc reframing of source-of-truth (would be `renegotiated`); the existing convention IS the answer + ADR records the explicit-risk-acceptance-statement + review-trigger per `drift-lifecycle.md:122-128`. Two drifts (D-new-25 + D-new-27) are deferred-scope items where `accepted-as-is` ALSO carries a deferred-marker cross-ref (M6-DEFERRED-02 / M6-or-M7-DEFERRED) — the chunk where they would actually be remediated. D7.4 carries an additional M7-server-promote review trigger.

---

## Decision

### §D57.1 — D5.2: Audit-event placement convention (`server::platform::<page>::audit_events` for HTTP-handler tier; `domain::audit::events::mX::*` for state-machine / fire-listener tier)

**Shipped at M5/P5 close (date 2026-04-23); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at [`modules/crates/server/src/platform/templates/audit_events.rs`](../../../../../../modules/crates/server/src/platform/templates/audit_events.rs) (4 builder fns + 4 unit tests).**

The convention: audit-event builder modules live at one of two homes depending on which tier emits them.

- **Domain tier** (`domain::audit::events::mX::*`) — events emitted from state-machine transitions or domain-tier fire listeners. Examples: `m4::templates::template_a_grant_fired` (M4 Template A fire listener), `m5_2::auth_request_access::*` (CH-18 §D56.5 access-denial events emitted from the AR access-check predicate at the handler boundary above Repository).
- **Platform tier** (`server::platform::<page>::audit_events`) — events emitted from HTTP handlers responding to operator actions on a specific admin page. Examples: page-12 template adoption (`server::platform::templates::audit_events::{template_adopted, adoption_denied, template_revoked}`); page-13 system-agent disable/archive (`server::platform::system_agents::audit_events`).

Both tiers feed the same single-writer `AuditEmitter` chain (per [ADR-0033](0033-k8s-prep-refactors.md) §D33.4), so the K8s A7 single-writer-symmetry guarantee is preserved by the convention. The split is a **code-organisation decision**, not a concept-level distinction — concept doc `permissions/02-auth-request.md` is silent below this granularity, and audit hash-chain symmetry is invariant to where the event-builder is housed.

D6.4 (already in CH-20 scope) follows the same pattern at `server::platform::system_agents::audit_events`. Future page-N HTTP handlers should default to platform-tier when the event corresponds to an operator action; default to domain-tier when the event corresponds to a state-machine transition or fire listener.

**Risk acceptance.** Acceptable. The convention is enforceable by reviewer expectation + grep (no domain-tier `template.adopted` event-name string outside the platform-tier file). Non-compliance produces no runtime symptom (audit hash-chain still computes correctly), so the convention is style-not-correctness. **Review trigger: none near-term** — convention stable.

**Rejected alternative.** "Domain-tier homes for all audit-event builders" was rejected as contrary to the shipped reality at M5/P5 close: the page-12 events were already at `server::platform::templates::audit_events`; relocating would have been pure code-shuffle without behaviour benefit. The split tier-by-emission-source is also more legible to reviewers (an HTTP-handler reviewer doesn't have to grep across crates to find the event-builder for the action they're reviewing).

### §D57.2 — D5.3: `find_adoption_ar` most-recent semantics + multi-AR-per-(org, kind) history preservation

**Shipped at M5/P5 close (date 2026-04-23); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at [`modules/crates/server/src/platform/templates/mod.rs:193`](../../../../../../modules/crates/server/src/platform/templates/mod.rs).**

The convention: an org may adopt → revoke → re-adopt the same template kind, producing **multiple adoption-AR rows for the same `(org_id, template_kind)` pair across history.** The "current adoption state" of a template kind for an org is the **most-recent AR's state** (sorted by `submitted_at` desc).

`find_adoption_ar(org_id, template_kind)` is the canonical resolver — it loads all matching AR rows for `(org_id, template_kind)`, sorts by `submitted_at` descending (`sort_by_key(|ar| Reverse(ar.submitted_at))`), and returns `.first().cloned()`. This matches concept doc `permissions/02-auth-request.md` §"Interaction with Authority Templates" framing of "current adoption state". The **list-adoption endpoint** (`GET /api/v0/orgs/:org/adoption-ars`) preserves the full history — all rows visible for audit. The **resolution endpoint** (used by listener wiring + dashboard cards) surfaces only the current most-recent row.

**Risk acceptance.** Acceptable. The implementation matches concept-doc semantics exactly; the only "surprise" is that the concept doc was previously silent on the multi-row-per-(org, kind) shape (D5.3 is `concept-silent-plan-filled-gap`). CH-19 closes the silence via the refresh paragraph at concept doc 02 §"Interaction with Authority Templates" tail. **Review trigger: none near-term.**

**Rejected alternative.** "1-to-1 (org, kind) → AR mapping enforced by UNIQUE index" was rejected as contrary to lifecycle reality — orgs do revoke + re-adopt (e.g., to switch from Template A v0 to v1), and the audit-history of who-adopted-when must be preserved.

### §D57.3 — D6.3: System-agent 3-way union bucketing for "standard" classification

**Shipped at M5/P6 close (date 2026-04-24); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at [`modules/crates/server/src/platform/system_agents/list.rs`](../../../../../../modules/crates/server/src/platform/system_agents/list.rs).**

The convention: `GET /api/v0/orgs/:org/system-agents` (page-13 list endpoint) classifies an Agent row as "standard" if **any** of three conditions holds:

1. **Canonical slug match** — Agent's slug is one of the M5-canonical strings (`memory-extraction-agent`, `agent-catalog-agent`).
2. **Registry membership** — Agent's id appears in the owning org's `Organization.system_agents` registry array.
3. **Role match** — Agent's `role` field is `AgentRole::System`.

The 3-way union is **robust against M3-era fixture rows** that were created before the M5 canonical-slug convention existed — those rows lack the canonical slug but still carry `AgentRole::System` and/or registry membership. Without the union, M3-era fixture orgs would silently drop their system agents from the page-13 list.

**Risk acceptance.** Acceptable. The union widens the bucket safely (each criterion alone is a sufficient witness of "this is a system agent"); false-positives would require an Agent row to coincidentally satisfy one of the three without intentionally being a system agent, which is implausible in production data. **Review trigger: M7b slug-reservation if pursued** — if a future milestone reserves a global slug-namespace for system agents (vs the org-level uniqueness today), the canonical-slug arm of the union may be promoted to the only-arm.

**Rejected alternative.** "Canonical slug only" was rejected because it would silently drop M3-era fixture system-agents from page-13. "Role-only" was rejected because Agent.role is mutable in principle (the pattern is "born with role"), so widening to slug + registry adds resilience.

### §D57.4 — D7.2: Additive `--model-config-id` CLI flag (alongside `--patch-json`)

**Shipped at M5/P7 close (date 2026-04-24); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at [`modules/crates/cli/src/main.rs`](../../../../../../modules/crates/cli/src/main.rs) (the `agent update` subcommand).**

The convention: `phi agent update <agent-id>` accepts **two mutually-exclusive flags** for profile mutation:

- `--patch-json '<JSON>'` (M4 surface — free-form profile patch). Operators specifying multiple fields, custom fields, or non-model-config fields use this.
- `--model-config-id <slug>` (M5/P7 addition — slug shorthand). Operators who only need to rebind the agent's `ModelConfig` use this for ergonomics.

Both flags reach the same backend endpoint (`PATCH /api/v0/agents/:id/profile`); the `--model-config-id <slug>` is server-side translated to the equivalent `--patch-json '{"model_config_id": "<slug>"}'`. **A single `phi agent update` invocation must carry exactly one of the two** (mutual exclusion via clap's `conflicts_with`). Both flags are documented in the CLI completion-help test pinning — see `cli::tests::completion_help::completion_scripts_expose_m5_p7_agent_update_model_config_id_flag`.

**Risk acceptance.** Acceptable. Backward-compatible with M4 `--patch-json` operators (their workflows continue to work unchanged). The dual-flag design adds CLI surface but reduces operator friction for the common rebind case (~80% of `agent update` invocations are pure model rebinds).

**Review trigger: none near-term.** The completion-help test pins the flag pair stable.

**Rejected alternative.** "Replace `--patch-json` with `--model-config-id` only" was rejected as a backward-incompatible change. "`--patch-json` only (no `--model-config-id`)" was rejected because it forces operators to know JSON schema for the common case; the slug shorthand is the operator-ergonomic surface.

### §D57.5 — D7.4: Page-11 "Recent sessions" web-side parallel-fetch retrofit (server-detail mutation deferred)

**Shipped at M5/P7 close (date 2026-04-24); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at `modules/web/app/(admin)/organizations/[id]/projects/[project_id]/page.tsx` lines 88–105 (web-side retrofit) + [`modules/crates/server/src/platform/projects/detail.rs`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) line 229 (server-side intentionally returns empty).**

The convention: page-11 (project detail) "Recent sessions" panel is implemented **web-side** at v0.1.

- The Next.js page component fires a parallel fetch against `GET /api/v0/projects/:id/sessions` (via `listSessionsInProjectApi`) alongside the project-detail fetch.
- The server-side `ProjectDetail.recent_sessions` field stays `Vec::new()` at M5; the M4 wire contract is frozen.
- Both paths render the same UI; the choice is invisible to operators.

The web-side path was chosen at M5/P7 to **minimize blast radius for the retrofit** — keeps the M4 `ProjectDetail` shape unchanged + avoids a server-side detail-handler mutation that would touch the dashboard cache + permission-check Step 0 evaluation.

**Risk acceptance.** Acceptable. The web-side parallel fetch adds one extra HTTP call per project-detail render (≤ 1 KB response); operator-perceived latency is dominated by the larger project-detail fetch already on the wire. Both paths render the same UI, so server-side promotion is a transparent migration if pursued.

**Review trigger: M7 project-detail hardening** — server-side promotion may revisit and strip the web-side fetch. The drift's `Implementation chunk this belongs to` field stays at the M5 default for now.

**Rejected alternative.** "Server-detail mutation at M5" was rejected at M5/P7 as scope creep — would have required dashboard-cache invalidation + Step 0 evaluation refresh + permissions-check coverage of the new field, all of which are not load-bearing for the retrofit's user-visible outcome.

### §D57.6 — D7.5: No new web component-tests at P5/P6/P7 (web test count stays at 79; coverage deferred to CH-24 Playwright)

**Shipped at M5/P7 close (date 2026-04-24); CH-19 ratifies without behaviour change. Pre-existing test count preserved at `modules/web/__tests__/` (79 tests).**

The convention: at M5/P5–P7, the three new admin pages (page-11 project detail, page-12 template adoption, page-13 system-agent management) shipped **without new component-tests**. The web `npm test` count stays at 79 (same as M4 close). Coverage of the three new pages is **deferred to CH-24 Playwright e2e scope** — the rationale is that page-level behaviour is best validated end-to-end (real HTTP backend + real DOM mutations + real navigation flows) rather than via mocked-API React component tests.

**Risk acceptance.** Acceptable. Concept doc surfaces (`permissions/05-memory-sessions.md` and others) are silent on test strategy at the page-component level — test-strategy is below concept granularity. The 79 existing tests cover M4-vintage components + shared utilities; the three new pages are thin wrappers around `apiFetch + form-state-management` patterns those tests already exercise.

**Review trigger: CH-24 Playwright e2e** — Playwright suite is the binding successor; CH-24's test plan must enumerate scenarios covering the three pages. Until CH-24 lands, manual regression on the three pages is operator-attended at every release-candidate cut.

**Rejected alternative.** "Add stub component-tests at P5/P6/P7" was rejected as low-value coverage that would not catch the failure modes Playwright is designed for (real navigation + real backend). "Block M5 close until CH-24 ships" was rejected as scope-creep — the three pages work in production today; deferring test coverage doesn't block their use.

### §D57.7 — D-new-21: Canonical edge-count is **71** (test-asserted invariant); doc reconciles

**Test-asserted invariant shipped at M3/P1 + M4/P1 + CH-23 close (cumulative date 2026-04-30); CH-19 reconciles documentation. Pre-existing implementation preserved at [`modules/crates/domain/src/model/edges.rs:524`](../../../../../../modules/crates/domain/src/model/edges.rs) (`pub const EDGE_KIND_NAMES: [&str; 71]`) + line 661 (`assert_eq!(EDGE_KIND_NAMES.len(), 71)`).**

The convention: the canonical edge-count is **71** — the value test-asserted in `domain::model::edges::tests::edge_kind_names_count`. Edge-count history:

- 67 at M3 close.
- +2 at M4/P1: `HasSubproject`, `HasConfig` per [`modules/crates/domain/src/model/edges.rs:520-521`](../../../../../../modules/crates/domain/src/model/edges.rs).
- +2 at CH-23: `Manages`, `HasAgentSupervisor` per [ADR-0046](0046-template-cd-http-edges.md) Template C/D HTTP edges.
- **Total: 71.**

Prior to CH-19 the count was inconsistent across three sources: `concepts/ontology.md:87` said "66 total"; `edges.rs:1,12,25` docstring said "69" (predated CH-23); `EDGE_KIND_NAMES: [&str; 71]` at line 524 + the assertion at line 661 said "71" (canonical). CH-19's 3-line docstring reconcile + 1-line concept-doc header update bring all three into agreement at 71. The test-asserted invariant is the single source-of-truth; the docstring + concept-doc are now downstream reflections.

**Risk acceptance.** Acceptable. The reconcile is comment/text-only; no behavioural change. The test invariant `EDGE_KIND_NAMES.len() == 71` continues to enforce the canonical count at every `cargo test` run; future edge-count changes will trip the test until docstring + concept-doc are bumped together.

**Review trigger: none near-term** — count locked at 71 until a future chunk adds new edge variants. Each future addition must update all three sources (`Edge` enum variant + `EDGE_KIND_NAMES` array + docstring + concept-doc) atomically per ADR-0057's reconcile pattern.

**Rejected alternative.** "Update only the concept-doc count" was rejected because the docstring inconsistency (69 vs 71) would persist as a future-reader trap. "Update docstring to match `[&str; 71]` only" was rejected because the concept-doc is the user-visible surface; it must agree with code.

### §D57.8 — D-new-25: InboxObject + OutboxObject `messages: Vec<AgentMessage>` field deferred to M6-DEFERRED-02

**Pre-existing scaffold preserved at [`modules/crates/domain/src/model/nodes.rs:772-784`](../../../../../../modules/crates/domain/src/model/nodes.rs); CH-19 ratifies the deferral, does NOT reassign.**

The convention: the v0 `InboxObject` and `OutboxObject` structs at `domain/src/model/nodes.rs:772-784` carry the **minimal scaffold** — `id`, `agent_id`, `created_at` — without the embedded `messages: Vec<AgentMessage>` field that concept doc `ontology.md` §"Node Types — Social Structure" describes. The full message-embedding is deferred to **M6-DEFERRED-02 (inter-agent messaging)** chunk.

This is a **deferred-scope acceptance**, not an enduring shape choice — the long-term shape per concept doc 07 §"Composite Classes — Inbox & Outbox" + concept doc `ontology.md` §"Value Objects" IS the embedded-messages variant; v0 ships only the structural scaffolding (struct exists, edge to Agent exists, composite-class registration exists) so that downstream code references the type names without forcing message-routing-layer to ship at M5.

**Risk acceptance.** Acceptable. The struct shape today is a strict subset of the long-term shape; future migration (M6 inter-agent messaging chunk) adds the `messages` field via SurrealDB FLEXIBLE column + struct field-add — both backward-compatible operations. The concept-doc body below is the spec; current code is the strict-subset implementation.

**Review trigger: M6-DEFERRED-02** — the inter-agent messaging chunk is the binding successor.

**Rejected alternative.** "Ship the `messages: Vec<AgentMessage>` field at M5 with no message-routing wiring" was rejected as wasted scope — the field would be unconditionally empty until the routing layer landed; future readers would be confused why a documented field is always empty in production. "Delete the InboxObject/OutboxObject scaffolds at M5 and reintroduce at M6" was rejected because the composite-class registration + edge to Agent are already wired into the permission-check path; ripping them out would introduce churn for no gain.

### §D57.9 — D-new-27: Token-economy fields (rating window, total_tokens_earned/consumed, Worth) deferred to M6-or-M7-DEFERRED

**Pre-existing absence on Agent struct at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); CH-19 ratifies the deferral.**

The convention: the v0 `Agent` struct does NOT carry the token-economy fields that `concepts/token-economy.md` §"Worth" + §"Rating Window" describe — `rating_window`, `total_tokens_earned`, `total_tokens_consumed`, and the derived `Worth`. These fields are deferred to **M6-or-M7-DEFERRED token-economy chunk** (contracts + bidding scope). Concept doc `token-economy.md` is **concept-aspirational** — the long-term shape pending the contracts/bidding milestone; the v0 Agent struct ships without economic state because the contracts/bidding layer that would populate the fields does not exist at v0.1.

**Risk acceptance.** Acceptable. The v0.1 product surface (admin pages 1–14) does not consume `Worth` or `rating_window`; deferring the fields' addition until the contracts/bidding milestone keeps the Agent struct narrow + avoids dead state on the row.

**Review trigger: M6-or-M7-DEFERRED token-economy** — the contracts/bidding chunk is the binding successor; that chunk's plan must add the fields + migration + Worth-computation pipeline atomically.

**Rejected alternative.** "Ship the fields at M5 with constant values (`rating_window: []`, totals `0`)" was rejected as wasted scope (same reason as §D57.8 rejected alt). "Delete the concept-doc framing of Worth at v0 and reintroduce at M6" was rejected because the concept-doc IS the spec — decapitating the spec would lose the design rationale. The 1-line deferred-state footnote at concept doc `token-economy.md` §"Worth" preamble is the right calibration — keeps the design intact, marks the v0 implementation gap.

### §D57.10 — D-new-30: Org/Project template-as-config — YAML-as-conceptual-contract / AR-and-listener-as-implementation-surface

**Pre-existing implementation preserved at [`modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) (Agent + Org rows do NOT carry an embedded YAML config object) + [`modules/crates/server/src/platform/`](../../../../../../modules/crates/server/src/platform/) (adoption-AR creation + Template A/C/D fire-listeners — M3 + M5 era pattern).**

The convention: `concepts/permissions/07-templates-and-tools.md` §"Standard Organization Template" + §"Standard Project Template" describe the templates as **YAML config objects** specifying `tools_allowlist`, `resource_catalogue`, `system_agents`, `authority_templates_enabled`, `consent_policy`, `execution_limits`, `session_object_grants`, `memory_object_grants`, `rating_window`. The v0.1 implementation does NOT materialize this YAML as an embedded config object on the `Organization` or `Project` row. Instead, **the same semantic content ships through adoption Auth Requests + listener-fired Grants**:

1. When an org adopts the Standard Organization Template, an adoption-AR is created (per the §"Templates Are Pre-Authorized Allocations" pattern).
2. Platform listeners fire the corresponding Grants/edges: `HOLDS_GRANT` rows for tools_allowlist + session_object_grants + memory_object_grants; `HAS_AGENT` edges for the system_agents registry; `HAS_AUTHORITY_TEMPLATE` edges for authority_templates_enabled; `GOVERNED_BY` edge for execution_limits.
3. The org's effective config is computed at runtime by walking the org's `HOLDS_GRANT` + edge set, NOT by reading an embedded YAML field.

The two framings are **functionally equivalent** — every YAML field maps to an AR-fired Grant or edge; revoking the adoption-AR cascade-revokes every fired Grant — but they differ in surface: YAML is the **conceptual contract** (the spec); AR-and-listener is the **mechanism** (the implementation).

**Risk acceptance.** Acceptable. The AR-and-listener pattern is the M3/M5-era convention used uniformly across the codebase; switching to embedded YAML config at v0.1 would require a dual-source-of-truth (the YAML + the Grants) until a future chunk reconciles. The 1-paragraph framing notes added to concept doc 07 at the §"Standard Organization Template" + §"Standard Project Template" preambles explicitly call out the YAML-as-spec / AR-as-mechanism mapping so future readers understand the divergence.

**Review trigger: none near-term** — pattern stable across 3 milestones (M3/M5/M5.2).

**Rejected alternative.** "Embed YAML config object on Organization row at M5" was rejected as it would create a dual-source-of-truth (the YAML + the Grants); revoking a YAML field would not cascade-revoke the corresponding Grant, breaking the §"Templates Are Pre-Authorized Allocations" §"Revocation" guarantee. "Delete the YAML framing from concept doc 07 at v0 and reintroduce when the embedded variant ships" was rejected because the YAML is the spec — it captures what the template grants without entangling the reader in the AR-and-listener mechanism.

---

## Risk acceptance (consolidated)

All ten sub-decisions share Bucket-B characteristics: **shipped convention works; cost of code-side remediation outweighs benefit at M5 close.** None of the ten introduces a runtime-correctness risk. Six of ten (D5.2, D5.3, D6.3, D7.2, D-new-21, D-new-30) are pure shape-choice ratifications with no review-trigger near-term; three (D7.4 → M7 server-promote, D7.5 → CH-24 Playwright, D-new-25 → M6-DEFERRED-02) carry binding successor chunks; one (D-new-27) carries an open-ended-deferred-marker (M6-or-M7-DEFERRED) per Q5.

Per [`drift-lifecycle.md:122-128`](../../m5_1/process/drift-lifecycle.md), `accepted-as-is` requires (a) Accepted ADR documenting the drift ID — **this ADR**, with §D57.1–§D57.10; (b) explicit risk-acceptance statement — **above, this section**; (c) review trigger — **per-drift, in each §D57.N body above**. All three preconditions met; transitions enabled.

---

## Pre-existing-behaviour preservation

Per CH-14 retro Row 10, every sub-decision in this ADR opens with the formula *"Shipped at M5/P<n> close (date YYYY-MM-DD); CH-19 ratifies without behaviour change. Pre-existing implementation preserved at `<file:line>`."* This makes explicit that ADR-0057 does NOT change runtime behaviour — every convention §D57.1–§D57.10 documents was already shipped at M5/P5–P7 close (or earlier for D-new-21 spanning M3/P1 + M4/P1 + CH-23). The ADR's effect is governance — promoting "shipped but undocumented convention" to "shipped + ratified-as-accepted-as-is + cited from concept doc + indexed in `_concept-audit-matrix.md`."

The audit-trail makes explicit which behaviours are now relied-upon invariants. Future chunks may extend (§D57.1 — D6.4 follows) or revisit (§D57.5 → M7 promotion; §D57.7 reconcile pattern at every future edge addition); but the conventions captured here are the binding M5-close baseline for those evolutions.

---

## Out of scope

Tracked successors (each binds to a specific future chunk; none is open-ended):

- **§D57.5 → M7 project-detail hardening** — page-11 server-side recent-sessions promotion. Web-side parallel-fetch is the v0.1 baseline; M7 chunk may revisit.
- **§D57.6 → CH-24 Playwright e2e** — coverage of M5/P5–P7 admin pages.
- **§D57.8 → M6-DEFERRED-02 (inter-agent messaging)** — InboxObject/OutboxObject `messages: Vec<AgentMessage>` field add + routing layer.
- **§D57.9 → M6-or-M7-DEFERRED (token economy)** — Agent struct token-economy fields + Worth computation pipeline.

Out-of-scope explicitly:

- **No new code-side abstraction** — CH-19 is doc-only apart from the 3-line `edges.rs` docstring reconcile. No new traits, no new enum variants, no new Repository methods.
- **No migration** — the `_migrations` table count stays unchanged.
- **No phi-core import change** — baseline preserved at 57.

---

## Cross-references

### (a) Originating concept docs

- [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Auth Request Lifecycle" + §"Interaction with Authority Templates" + §"Per-State Access Matrix" — D5.2 + D5.3 source-of-truth.
- [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"Standard Permission Templates" + §"Templates Are Pre-Authorized Allocations" + §"Standard Organization Template" + §"Standard Project Template" — D5.3 + D-new-30 source-of-truth.
- [`concepts/system-agents.md`](../../../concepts/system-agents.md) §"Properties Shared by All System Agents" — D6.3 source-of-truth.
- [`concepts/agent.md`](../../../concepts/agent.md) §"Soul (Immutable Born Structure)" — D7.2 source-of-truth.
- [`concepts/project.md`](../../../concepts/project.md) §"Project (Node Type)" — D7.4 source-of-truth.
- [`concepts/ontology.md`](../../../concepts/ontology.md) §"Edge Types" + §"Node Types — Social Structure" — D-new-21 + D-new-25 source-of-truth.
- [`concepts/token-economy.md`](../../../concepts/token-economy.md) §"Worth" + §"Rating Window" — D-new-27 source-of-truth.

### (b) Closed drifts (10)

- [D5.2](../../m5_1/drifts/D5.2.md), [D5.3](../../m5_1/drifts/D5.3.md), [D6.3](../../m5_1/drifts/D6.3.md), [D7.2](../../m5_1/drifts/D7.2.md), [D7.4](../../m5_1/drifts/D7.4.md), [D7.5](../../m5_1/drifts/D7.5.md), [D-new-21](../../m5_1/drifts/D-new-21.md), [D-new-25](../../m5_1/drifts/D-new-25.md), [D-new-27](../../m5_1/drifts/D-new-27.md), [D-new-30](../../m5_1/drifts/D-new-30.md).

### (c) Prior ADRs cited as precedent (milestone-prefixed per CH-08 retro Row 1)

- [`m5_2/decisions/0042-storage-backend-configurable.md`](0042-storage-backend-configurable.md) (CH-03) — doc-only-ratification-chunk shape precedent (ADR-0057 follows this exact shape with 10 sub-decisions instead of CH-03's 6).
- [`m5_2/decisions/0033-k8s-prep-refactors.md`](0033-k8s-prep-refactors.md) (CH-K8S-PREP) — §D33.4 single-AuditEmitter-writer guarantee that §D57.1's audit-event placement convention preserves.
- [`m4/decisions/0028-domain-event-bus.md`](../../m4/decisions/0028-domain-event-bus.md) (M4) — Template-A fire-listener pattern that §D57.1's domain-tier convention ratifies the audit-event-placement of.
- [`m5_2/decisions/0046-template-cd-http-edges.md`](0046-template-cd-http-edges.md) (CH-23) — Template C/D listener-pattern symmetric with §D57.1's domain-tier convention; also the source of the +2 edges (`MANAGES`, `HAS_AGENT_SUPERVISOR`) that bring §D57.7's canonical count to 71.
- [`m5_2/decisions/0050-audit-class-composition-strictest-wins.md`](0050-audit-class-composition-strictest-wins.md) (CH-13) — audit-event diff `audit_class_source` attribution that crosses both §D57.1 tiers (domain-tier listeners + platform-tier handlers).
- [`m3/decisions/0022-org-creation-compound-transaction.md`](../../m3/decisions/0022-org-creation-compound-transaction.md) (M3) — §D57.3 union-bucketing's M3-era-fixture origin (the compound-transaction creates the system-agent rows that pre-date M5 canonical slugs).

### (d) Forward-scope row

- [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md` lines 179–183](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) (CH-19 row); §5 severity-row binding at line 427.

### Plan archive

- [`build/ch-19-bucket-b-ratification-2c520ba7/plan.md`](../../../../plan/build/ch-19-bucket-b-ratification-2c520ba7/plan.md) — cycle hex `2c520ba7`.

### Code (sole code-side touch in CH-19)

- [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) lines 1, 12, 25 — docstring "69" → "71" reconcile (D-new-21 / §D57.7). Comment-only; no behavioural change.
