<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS amendment — recent_sessions panel API-surface flip (2026-05-11): new subsection `## CH-24 amendment — page-11 recent_sessions panel + M5 stable-code table (2026-05-11)` documenting (a) the resolution of D-CH24-recent-sessions-api-flip via the new dedicated repo method `list_recent_sessions_for_project` + `RecentSessionEntry` view-shape struct per ADR-0059 §D59.1–§D59.3, (b) the full M5 stable-code table aggregating pages 12/13/14 + cross-cutting codes, (c) CLI exit-code mapping, (d) cross-org isolation invariants for the M5 milestone seal. Pairs with `m5/operations/m5-ops-runbook.md` (NEW per F5.A lock) + ADR-0059. Cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P3 amendment — auth_request.access_denied troubleshooting (2026-05-09): new subsection `## CH-18 amendment — auth_request.access_denied troubleshooting (2026-05-09)` documenting the Alerted-class `auth_request.access_denied` audit-event introduced by ADR-0056 §D56.6. The amendment covers (a) the 4xx `ACCESS_DENIED` symptom-class for AR-mutation rejections at `templates/{approve,deny,revoke}.rs` + `projects/create.rs` slot-fill mutation, (b) the silent-list-filter behaviour at the 5 read-side callsites per F3.B.list-filter.a (operator sees fewer ARs than DB rows for non-admin viewers — intentional, no audit-event), (c) the 5 typed-error variants of `AuthRequestAccessError` mapping to operator narratives, (d) the admin/auditor read-bypass deferral via `D-CH18-FOLLOWUP-01` (M6+). Pairs with `m5_2/operations/auth-request-access-acl-operations.md` (full operator runbook) + `m5_2/architecture/auth-request-access-acl.md` (design page) + ADR-0056. Cycle hex c77937bc.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-15 amendment: new troubleshooting section for hard-deny launch gate post-ADR-0054 §D54.6 — common 403 cause is Step 2 NoGrantsHeld; remediation via Template A grant seeding or migration `0015` backfill.) -->
<!-- CH-01 + CH-22 amendments (2026-04-27): durable disable/archive semantics + agent-catalog audit-mode + new "catalog row stale" symptom. See §"CH-01 + CH-22 amendments" below. Full M5/P9 stable-code table still deferred to M5-tag-close. -->
<!-- CH-06 amendment (2026-04-28): selector parse-error troubleshooting. See §"CH-06 amendment — selector parse errors" below. -->

# User guide — Troubleshooting (M5)

**Status**: `[EXISTS]` as of M5/P9 / CH-24 close — full stable-code table
+ CLI exit codes + cross-org isolation invariants now extant under
"CH-24 amendment" subsection below. The amendment subsections
(CH-01/06/15/17/18/21/22/24) carry per-chunk symptoms layered on top of
the inherited M4 base; the CH-24 amendment subsection is the
milestone-rollup full table.

Every M5 HTTP error carries a JSON body `{ "code": "<STABLE_CODE>",
"message": "..." }`. The CLI surfaces the `code` verbatim via
`phi: rejected (<CODE>): <message>` + maps it to one of the exit
codes pinned in [`cli-reference-m5.md`](cli-reference-m5.md).

## Session surface (page 14) — placeholder

Full table lands at P9. Codes to expect (per plan §G12-G18):

- `PARALLELIZE_CAP_REACHED` (409)
- `SESSION_WORKER_SATURATED` (503)
- `AGENT_NOT_FOUND` (404)
- `PROJECT_NOT_FOUND` (404)
- `PERMISSION_CHECK_FAILED_AT_STEP_N` (403 / 400)
- `MODEL_RUNTIME_UNRESOLVED` (400)
- `SESSION_NOT_FOUND` (404)
- `SESSION_ALREADY_TERMINAL` (409)
- `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` (409) — M4 code, becomes reachable at M5/P4 (C-M5-5 flip)

## Authority templates surface (page 12) — placeholder

Full table lands at P9.

## System agents surface (page 13) — placeholder

Full table lands at P9.

## Cross-cutting codes (inherited)

Full table in [`../../m4/user-guide/troubleshooting.md`](../../m4/user-guide/troubleshooting.md):

- `VALIDATION_FAILED` (400)
- `AUDIT_EMIT_FAILED` (500)
- `UNAUTHENTICATED` (401)
- `INTERNAL` (500)

## CH-01 + CH-22 amendments — symptoms & fixes (2026-04-27)

| Symptom | Likely cause | Fix |
|---|---|---|
| `phi system-agent disable` returns `200` but the agent still appears "active" in old monitoring tooling | Pre-CH-01 tooling reads only the audit log (which has always recorded the disable) but not the durable `agent.active` column | Update the tool to read the new `active` field on the agent row. Both `phi agent show --json` and `GET /api/v0/orgs/:org/agents/:id` now include `{active, archived_at}`. |
| `AUDIT_EMIT_ERROR` after a disable / archive request | Audit-emit path failed AFTER the durable column flip | Durable state IS correct — replay the audit chain rather than re-issuing the request. (CH-01 / ADR-0034 §D34.4 ordering rule.) |
| `agent_catalog_entry` row missing for a known agent | Listener never fired for this agent (likely created via test fixture / direct repo call that bypasses production HTTP handlers) | See [system-agents operations §"Catalog row missing"](../operations/system-agents-operations.md). |
| Audit log flooded with `agent_catalog_refreshed` events in production | `[listeners.catalog] audit_mode = "debug"` left enabled (typically post-investigation) | Set `PHI_LISTENERS__CATALOG__AUDIT_MODE=silent` and restart. Old debug-mode rows are `AuditClass::Silent` (30-day retention) and will age out. |
| Catalog system agent's runtime-status tile shows stale `last_fired_at` | Either (a) no agent-lifecycle events being emitted from production handlers (run a test write to verify), or (b) the catalog system agent isn't resolvable from `org.system_agents` (verify exactly one entry has `display_name == "agent-catalog"`) | See [system-agents operations §"Runtime-status tile stale"](../operations/system-agents-operations.md). |

## CH-06 amendment — selector parse errors (2026-04-28)

CH-06 lights up the full PEG tag-predicate DSL on `Selector` (concept-09). When a grant carries a new-grammar selector (e.g. `tags contains org:acme AND tags contains #kind:session`), the parser may surface parse errors. They appear as `Decision::Denied { failed_step: Match, reason: ... }` with stable codes `P-001`–`P-005`:

| Code | Symptom | Fix |
|---|---|---|
| `P-001` | `unexpected token '<X>' at position <N>` | Identifier doesn't match `[a-zA-Z_][a-zA-Z0-9_-]*` (often a value starting with a digit, like `a:1`). Rename to `a:one` or `a:x1`. |
| `P-002` | `unbalanced parens at position <N>` | Count parens; ensure each `(` has a matching `)` at the same depth. |
| `P-003` | `unknown predicate '<name>'` | Use one of the 6 supported predicates: `contains`, `intersects`, `any_match`, `subset_of`, `empty`, `non_empty`. |
| `P-004` | `invalid glob '<pattern>'` | `any_match` glob must contain `*` or `**`. Use `tags contains` for an exact tag match. |
| `P-005` | `unknown set reference '<name>'` (runtime) | Set-ref name not registered. In M5.2 the noop registry returns `false` for every set-ref; CH-15 wires the production registry. |

For full operational guidance see [m5_2/operations/selector-grammar-operations.md](../../m5_2/operations/selector-grammar-operations.md). For grammar syntax see [m5_2/user-guide/selector-syntax-guide.md](../../m5_2/user-guide/selector-syntax-guide.md).

## CH-16 amendment — Identity errors (2026-04-28)

CH-16 lights up the four-field `Identity` node (concept-`agent.md` § "Identity Node Content"). Two common operator-facing errors land at this surface:

| Symptom | Cause | Fix |
|---|---|---|
| `HumanAgentHasNoIdentity { agent_id }` from a handler | Caller tried to write an Identity for a Human-kind agent. Concept-`human-agent.md` § "No Identity" mandates this rejection. | This is by design. Skip the Identity write for Human kind; the create-time path already does this when `payload.identity` is `None`. If a test fixture or REPL probe hits this, change the call to skip Identity entirely for Human-kind agents. See [concept-`human-agent.md` §"No Identity"](../../../concepts/human-agent.md). |
| `apply_agent_creation` fails with `Llm-kind agent requires Some(Identity)` | Server orchestrator forgot to build the Identity row for an LLM-kind payload. | Build via `Identity::default_for_llm(agent_id, now)`. Production handlers at `agents/create.rs` + `system_agents/add.rs` already do this; only test fixtures or future custom orchestrators need to mirror the pattern. |

For full operational guidance see [m5_2/operations/identity-operations.md](../../m5_2/operations/identity-operations.md). For the four-field shape + creation timing see [m5_2/user-guide/identity-overview.md](../../m5_2/user-guide/identity-overview.md).

## CH-21 amendment — Memory-extraction errors (2026-04-28)

CH-21 ships the heuristic memory-extraction listener body (concept-`system-agents.md` § "Memory Extraction Agent"; ADR-0040). Two common operator-facing situations land at this surface:

| Symptom | Cause | Fix |
|---|---|---|
| Extraction not firing — sessions end but no `Memory` rows appear and `Identity.witnessed.memories_extracted` stays at 0 | The org's `memory-extraction-agent` system agent is `active = false` or `archived_at` is populated; per ADR-0040 §D40.3 SKIP-BOTH the listener short-circuits before minting a Memory or firing telemetry. | Re-enable the system agent via `POST /platform/system_agents/{id}/enable` (CH-01 surface). Past sessions that ended while disabled are NOT replayed — extraction is forward-only at v0. See [m5_2/operations/memory-extraction-operations.md](../../m5_2/operations/memory-extraction-operations.md) §"Operator-disabled extractor". |
| Identity counter is stale relative to the Memory rows that exist | `Repository::upsert_identity` errored after `create_memory` succeeded — fail-safe semantics (ADR-0028) leave the Memory durable while the Identity counter has a 1-event gap. | The next successful extraction self-heals (the counter is incremental, not derived). To force-resync, an operator-driven recompute path is M6 / future-CH work (see successor M6-DEFERRED-04). Forensic context: structured logs carry the `event_id` of the failing fire. |

For full operational guidance see [m5_2/operations/memory-extraction-operations.md](../../m5_2/operations/memory-extraction-operations.md). For the v0 heuristic body + what M6 LLM upgrade adds, see [m5_2/user-guide/memory-extraction-overview.md](../../m5_2/user-guide/memory-extraction-overview.md).

## CH-15 amendment — session launch hard-deny (2026-05-08)

CH-15 (drift D4.1 closure) flips Permission Check at session launch from advisory-only to hard-deny on every step 0–6.

| Symptom | Cause | Fix |
|---|---|---|
| `POST /sessions` returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_2: NoGrantsHeld` for the project lead | Pre-CH-15 single-grant Template A holders need the paired `session_object` grant (per ADR-0054 §D54.3) | Run migration `0015_template_a_session_object_grant.surql` to backfill, OR re-emit `HasLeadEdgeCreated` for the project so the listener mints both grants. |
| `POST /sessions` returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_0: CatalogueMiss` | Resource URI not declared in owning org's `resources_catalogue` | Seed the catalogue entry. The launch builder uses class-level reach (target_uri = "") so Step 0 should not fire post-CH-15 — catalogue misses now indicate a deeper issue. |
| `POST /sessions/preview` returns `decision.outcome = denied` but the operator wants the launch to succeed | The lead is missing the paired `session_object` grant | Check the Permission Check trace on the receipt — `failed_step` + `reason` identify the missing reach. Mint the Template A grants OR run migration `0015`. |
| `platform.session.launch_denied` audit events spike post-deploy | Expected — every previously-advisory deny now hard-denies + emits an audit event. | Verify Template A grants are seeded for active leads. `failed_step` + `reason_kind` on each event identify which gate is firing. |

For the per-step deny playbook + audit-event dictionary entry see [m5_2/operations/session-launch-permission-gate-operations.md](../../m5_2/operations/session-launch-permission-gate-operations.md).

## CH-18 amendment — auth_request.access_denied troubleshooting (2026-05-09)

CH-18 (drift D-new-12 closure, ADR-0056) lands the AuthRequest per-state ACL enforcement: the typed predicate `domain::auth_requests::access::check_auth_request_access` gates 17 production callsites (4 mutation + 5 read + 8 submit). Mutation callsites that reject the principal emit a new Alerted-class `auth_request.access_denied` audit-event.

| Symptom | Cause | Fix |
|---|---|---|
| `POST /v0/templates/.../{approve,deny,revoke}` returns 4xx `ACCESS_DENIED` | Principal X is not authorised for the AR's current state × intended op cell of concept doc 02's per-state matrix. Most common: X is not a slot-approver and not the AR's requestor. | Verify principal is in `ar.resource_slots[*].approvers[*]` for slot ops or is `ar.requestor` for owner ops. The `auth_request.access_denied` audit-event's `error_kind` field tells you which deny class fired (`not_authorised_for_modify` / `unfilled_approver_slot_only` / `requestor_only_operation` / `operation_forbidden_in_state` / `not_authorised_for_read`). |
| Slot-approver who has already filled their slot gets `ACCESS_DENIED` on retry | `error_kind == unfilled_approver_slot_only` — once a slot is filled, the matrix's legal op for the slot-holder is `Reconsider` (re-edit own slot until closed-terminal), not `Approve` / `Deny` again. | Use the Reconsider endpoint, not Approve / Deny. |
| Dashboard shows fewer ARs than the DB has for a non-admin viewer | **Intentional behaviour** per F3.B.list-filter.a (silent post-filter at `dashboard.rs:273,293` + `show.rs:63`). Non-requestor / non-slot-approver / non-bootstrap viewers see strictly fewer ARs. | If the viewer should see all ARs (compliance-audit use case), this is the `D-CH18-FOLLOWUP-01` deferral — admin/auditor read-bypass tracked at M6+ admin-classifier wiring. |
| Admin (org CEO) gets denied reading another agent's AR | `error_kind == not_authorised_for_read` — CH-18 classifies all non-requestor / non-slot-approver / non-bootstrap principals as "Other Agent" → DENY. Admin/auditor classification is deferred to M6+ via `D-CH18-FOLLOWUP-01`. | M6+ admin-classifier wiring will add an `Agent.role` lookup OR Permission Check delegation. At v2 the deny is intentional. |
| `auth_request.access_denied` audit-event volume spikes post-deploy | UI bug allowing the "approve" / "deny" / "revoke" button to render for non-slot-approvers (the silent list-filter should have hidden the AR before the click). OR a bot invoking AR mutations with the wrong principal. | Verify the dashboard list-filter is wired (`viewer_agent_id` plumbed through `dashboard.rs::compute_dashboard_summary` to the post-filter). Audit `actor_agent_id` distribution in the audit events. |

For the full operator runbook + 5-variant typed-error reference + audit-event dictionary entry see [m5_2/operations/auth-request-access-acl-operations.md](../../m5_2/operations/auth-request-access-acl-operations.md).

## CH-24 amendment — page-11 recent_sessions panel + M5 stable-code table (2026-05-11)

CH-24 (M5 milestone seal; cycle hex `5778bb77`) flips the page-11 `recent_sessions` panel from M4 placeholder (`Vec::new()` hardcoded since M4) to a real query via the dedicated Repository method `list_recent_sessions_for_project(project_id, limit)` per ADR-0059 §D59.1–§D59.3. The placeholder’s inline doc-comment at `server/src/platform/projects/detail.rs:33` promised the flip during M5/P4 but the deliverable never landed; CH-24 closes the gap in-chunk via the new phase **P-FLIP-RECENT-SESSIONS** (mid-cycle scope expansion approved at gate-2 user-lock).

| Symptom | Cause | Fix |
|---|---|---|
| Page-11 `recent_sessions` panel shows empty for a project with known launches | (Most common post-CH-24): project genuinely has no live or recent sessions. | Launch a session and re-check. Verify via `GET /api/v0/orgs/:org/projects/:proj/sessions` (unbounded live-list endpoint). |
| Same symptom on a pre-CH-24 binary | Pre-CH-24 server has the M4 placeholder `recent_sessions: Vec::new()` hardcoded at `detail.rs:229`. | Redeploy from a post-CH-24 binary. Verify via `grep -n "list_recent_sessions_for_project" $(which phi-server)` or git SHA. |
| Page-11 panel shows only 10 rows for a project with > 10 sessions | **Intentional** per ADR-0059 §D59.1 — `RECENT_SESSIONS_LIMIT = 10` query-side cap. | For full history use the unbounded live-list endpoint `GET /api/v0/orgs/:org/projects/:proj/sessions`. |
| Page-11 panel sends an extra request per row to resolve agent display names | **Intentional** at v0 — `RecentSessionEntry` ships with 6 fields (id, project_id, agent_id, started_at, ended_at, status); `started_by_display_name` is deferred to a follow-up M6 chunk pending Agent-table-join design (per ADR-0059 §D59.3-FOLLOWUP). | The renderer resolves display names via secondary `GET /api/v0/agents/:id` until the follow-up chunk lands. |
| Page-11 panel for Org-B project is visible to an Org-A viewer | **Bug** — cross-org isolation violation. Acceptance test `m5_cross_org_isolation_at_session_surface` should catch this; if it surfaces in production, file a HIGH-severity drift. | Hard-deny / silent-list-filter is the expected behaviour. Verify `viewer_agent_id` plumbed through the project-detail handler. |

For the full operational guidance see [m5/operations/m5-ops-runbook.md](../operations/m5-ops-runbook.md) §"IP-4 Page-11 recent_sessions panel empty after launches".

### Full M5 stable-code reference table

Aggregated from pages 12 + 13 + 14 + cross-cutting M4 inheritance. Each row maps `HTTP status × stable code × CLI exit code × symptom → remediation pointer`.

**Session surface (page 14)**

| HTTP | Stable code | CLI exit | Symptom | Remediation |
|---|---|---|---|---|
| 400 | `SESSION_INPUT_INVALID` | 2 | Request body failed shape validation | Re-submit with valid shape per [session-launch-operations.md](../operations/session-launch-operations.md) |
| 400 | `TERMINATE_REASON_REQUIRED` | 2 | `reason` field empty | Supply non-empty `reason` |
| 401 | `UNAUTHENTICATED` | 1 | No session cookie / expired token | Re-authenticate |
| 403 | `FORBIDDEN` | 1 | Viewer not session starter + not org-member | Use a session belonging to the org/agent |
| 403 | `AGENT_NOT_MEMBER_OF_PROJECT` | 1 | Agent's `owning_org` mismatches project's org | Create agent in correct org |
| 403 | `PERMISSION_CHECK_FAILED_AT_STEP_<N>` | 1 | Hard-deny at engine step N (0–6) | See [session-launch-permission-gate-operations.md](../../m5_2/operations/session-launch-permission-gate-operations.md) §"Step N detail" |
| 403 | `TERMINATE_FORBIDDEN` | 1 | Caller not starter + not org Human | Terminate from authorised session |
| 404 | `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` / `SESSION_NOT_FOUND` / `MODEL_RUNTIME_NOT_FOUND` | 1 | Id unknown | Verify ids |
| 409 | `PARALLELIZE_CAP_REACHED` | 1 | Agent at `profile.parallelize` limit | Wait or tune `parallelize` |
| 409 | `MODEL_RUNTIME_UNRESOLVED` | 1 | No `model_config_id` bound | Bind via `PATCH /agents/:id/profile` |
| 409 | `MODEL_RUNTIME_ARCHIVED` | 1 | Bound runtime archived | Re-bind to active runtime |
| 409 | `AGENT_PROFILE_MISSING` | 1 | No profile row | Create profile |
| 409 | `SESSION_ALREADY_TERMINAL` | 0 | Already Completed/Aborted/FailedLaunch | No action — idempotent |
| 409 | `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` | 1 | Agent has live sessions; `model_config_id` change refused (C-M5-5) | End active sessions first |
| 410 | `SESSION_LIVE_STREAM_UNAVAILABLE` | 1 | SSE on a finalised session | Use `GET /api/v0/sessions/:id` for terminal detail |
| 503 | `SESSION_WORKER_SATURATED` | 1 | Per-worker registry full | Tune `config.session.max_concurrent` |

**Authority templates surface (page 12)**

| HTTP | Stable code | CLI exit | Symptom | Remediation |
|---|---|---|---|---|
| 400 | `VALIDATION_FAILED` | 2 | Manifest / selector / payload invalid | Re-validate per [authority-templates-operations.md](../operations/authority-templates-operations.md) |
| 403 | `ACCESS_DENIED` | 1 | AR-mutation rejected per per-state ACL (4 mutation callsites) | See [auth-request-access-acl-operations.md](../../m5_2/operations/auth-request-access-acl-operations.md) |
| 404 | `TEMPLATE_NOT_FOUND` / `AUTH_REQUEST_NOT_FOUND` | 1 | Id unknown | Verify ids |
| 409 | `TEMPLATE_ALREADY_ADOPTED` | 1 | Org already has active adoption AR | Revoke or re-adopt |

**System agents surface (page 13)**

| HTTP | Stable code | CLI exit | Symptom | Remediation |
|---|---|---|---|---|
| 403 | `FORBIDDEN_NON_ADMIN_TUNE` | 1 | Non-admin tried to tune standard system agent | Use admin-class principal |
| 404 | `SYSTEM_AGENT_NOT_FOUND` | 1 | System agent id unknown / not bucketed | Verify id + bucketing per [system-agents-operations.md](../operations/system-agents-operations.md) |
| 409 | `SYSTEM_AGENT_ALREADY_DISABLED` / `..._ARCHIVED` | 0 | Lifecycle terminal | No action — idempotent |

**Cross-cutting (inherited M4 / s02/s03 / panels)**

| HTTP | Stable code | CLI exit | Symptom | Remediation |
|---|---|---|---|---|
| 400 | `VALIDATION_FAILED` | 2 | Shape mismatch | Re-submit |
| 500 | `AUDIT_EMIT_FAILED` | 1 | Audit-emit failed AFTER durable write | Replay audit chain |
| 500 | `RECORDER_FAILURE` / `COMPOUND_TX_FAILURE` / `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` / `SESSION_REPLAY_PANIC` | 1 | Internal failure | Check logs + audit chain |
| 500 | `INTERNAL` | 1 | Unexpected | Check logs |

### CLI exit-code mapping

| Exit | Meaning |
|---|---|
| 0 | Success (incl. idempotent no-op) |
| 1 | Server returned 4xx/5xx with a stable code (operator-actionable) |
| 2 | Local input validation failure (CLI flag combinations / file parse) |

Per-subcommand details in [`cli-reference-m5.md`](cli-reference-m5.md).

### Cross-org isolation invariants

1. **Org-membership boundary** — a viewer from Org-A reading a resource owned by Org-B receives 403 / 404 (silent-list-filter or hard-deny per the F3.B.list-filter.a policy).
2. **Page-11 `recent_sessions` panel** for an Org-B project is NOT visible to Org-A viewers (verified by `acceptance_m5_sessions.rs::m5_cross_org_isolation_at_session_surface`).
3. **AuthRequest mutations** gated by per-state ACL (CH-18 / ADR-0056 §D56.5); 17 production callsites consult `check_auth_request_access`.
4. **Live SSE tail** at `GET /sessions/:id/events` gated by `Action::Observe` on `session_object` (CH-17 / ADR-0055); legacy Template A grants need migration `0016` to render the SSE field.

## Cross-references

- [Top-level runbook §M5](../../../../../../docs/ops/runbook.md) — operator-facing aggregated index.
- [M5 ops runbook (aggregate index)](../operations/m5-ops-runbook.md) — **NEW at CH-24** per F5.A lock; cross-page runbook.
- [M4 troubleshooting](../../m4/user-guide/troubleshooting.md) — inherited codes + cross-org isolation invariants.
- [M5 plan §P9 deliverables](../../../../plan/build/m5-templates-system-agents-sessions-01710c13.md).
- [M5.2 selector-grammar-operations](../../m5_2/operations/selector-grammar-operations.md) — CH-06 selector parse error runbook.
- [M5.2 identity-operations](../../m5_2/operations/identity-operations.md) — CH-16 Identity row runbook.
- [M5.2 memory-extraction-operations](../../m5_2/operations/memory-extraction-operations.md) — CH-21 memory-extraction listener runbook.
- [M5.2 auth-request-access-acl-operations](../../m5_2/operations/auth-request-access-acl-operations.md) — CH-18 AuthRequest per-state ACL operator runbook + 5-variant typed-error reference + `auth_request.access_denied` audit-event dictionary.
- [ADR-0059 — recent-sessions API-surface flip](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) — CH-24 panel flip with 4 sub-decisions.
