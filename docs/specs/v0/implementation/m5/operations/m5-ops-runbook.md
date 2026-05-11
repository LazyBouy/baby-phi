<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — NEW file per F5.A lock + plan §7 P-DOCS deliverable 1. Per-page ops runbook index linking 4 m5/operations/* + 10 m5_2/operations/* pages; ~25 stable-code reference table aggregating Page 12 (authority templates) + Page 13 (system agents) + Page 14 (sessions) + Pages 11 (project detail) + cross-cutting codes; 5 incident playbooks (stuck running session, agent-catalog row missing, memory-extraction skipped, page-11 empty recent_sessions, Template A grant gap). CH-24 row added for `recent_sessions` panel per ADR-0059. Cycle hex `5778bb77`.) -->

# Operations runbook — M5 milestone aggregate

**Status**: `[EXISTS]` at M5/P9 close (CH-24).

This is the operator-facing **index** for all M5 vertical surfaces (pages 11–14) and the M5.2 cross-cutting concerns (permission gating, AR-access ACL, memory extraction, agent catalog, identity, selector grammar, multi-scope cascade, allocate/transfer, audit-class composition, live SSE tail). Each section below names the per-page runbook + the symptoms it covers + the cross-cutting codes that intersect.

The aggregated **stable-code reference table** below pins every operator-visible HTTP code + CLI exit code + error symptom that M5 ships. Cross-org isolation invariants apply throughout — see [m5/user-guide/troubleshooting.md](../user-guide/troubleshooting.md) §"Cross-org isolation".

## Per-page runbooks (M5 vertical)

| Page | Surface | Detailed runbook |
|---|---|---|
| Page 11 — Project detail (incl. `recent_sessions` panel) | `GET /api/v0/projects/:id` | *(amendment below for page-11 panel; M4 + M5 page-11 cross-org invariants in [m4 troubleshooting](../../m4/user-guide/troubleshooting.md))* |
| Page 12 — Authority template adoption | `POST/PATCH/DELETE /api/v0/orgs/:org/templates/...` | [`authority-templates-operations.md`](authority-templates-operations.md) |
| Page 13 — System agents config | `GET/POST/PATCH/DELETE /api/v0/orgs/:org/system_agents/...` | [`system-agents-operations.md`](system-agents-operations.md) |
| Page 14 — First session launch | `POST/GET/DELETE /api/v0/sessions/...` | [`session-launch-operations.md`](session-launch-operations.md) |
| s02 / s03 cross-page system flows | listener firings on org/project/session/agent edges | [`system-flows-s02-s03-operations.md`](system-flows-s02-s03-operations.md) |

## Cross-cutting runbooks (M5.2)

| Concern | Surface | Detailed runbook |
|---|---|---|
| Session-launch permission gate | hard-deny at every step 0–6 | [`m5_2/operations/session-launch-permission-gate-operations.md`](../../m5_2/operations/session-launch-permission-gate-operations.md) |
| Session live SSE tail | `GET /api/v0/sessions/:id/events` | [`m5_2/operations/session-live-stream-operations.md`](../../m5_2/operations/session-live-stream-operations.md) |
| AuthRequest per-state ACL | 17 production callsites; `auth_request.access_denied` audit-event | [`m5_2/operations/auth-request-access-acl-operations.md`](../../m5_2/operations/auth-request-access-acl-operations.md) |
| Memory extraction listener | `MemoryExtractionListener` body + Identity counter | [`m5_2/operations/memory-extraction-operations.md`](../../m5_2/operations/memory-extraction-operations.md) |
| Identity (LLM-kind) | `Identity` 4-field shape + creation guards | [`m5_2/operations/identity-operations.md`](../../m5_2/operations/identity-operations.md) |
| Selector grammar | PEG tag-predicate DSL + parse-error codes | [`m5_2/operations/selector-grammar-operations.md`](../../m5_2/operations/selector-grammar-operations.md) |
| Multi-scope cascade | 5-tier scope resolution + contractor model | [`m5_2/operations/multi-scope-cascade-operations.md`](../../m5_2/operations/multi-scope-cascade-operations.md) |
| Authority chain | `walk_provenance_chain` + cascade revocation | [`m5_2/operations/authority-chain-operations.md`](../../m5_2/operations/authority-chain-operations.md) |
| Allocate/transfer cardinality | `apply_transfer_grant` compound-tx | [`m5_2/operations/allocate-transfer-cardinality-operations.md`](../../m5_2/operations/allocate-transfer-cardinality-operations.md) |
| Audit-class composition | strictest-wins composer | [`m5_2/operations/audit-class-composition-operations.md`](../../m5_2/operations/audit-class-composition-operations.md) |

## Stable-code reference table (aggregated)

Every M5 HTTP error carries a JSON body `{ "code": "<STABLE_CODE>", "message": "..." }`. The CLI surfaces the `code` verbatim via `phi: rejected (<CODE>): <message>` + maps it to the exit codes pinned in [`cli-reference-m5.md`](../user-guide/cli-reference-m5.md).

### Session surface (page 14)

| HTTP | Stable code | CLI exit | Symptom | Runbook |
|---|---|---|---|---|
| 400 | `SESSION_INPUT_INVALID` | 2 | Request body failed shape validation | [`session-launch-operations.md`](session-launch-operations.md) |
| 400 | `TERMINATE_REASON_REQUIRED` | 2 | `reason` field empty | [`session-launch-operations.md`](session-launch-operations.md) |
| 401 | `UNAUTHENTICATED` | 1 | No session cookie / bad token | inherited cross-cutting |
| 403 | `FORBIDDEN` | 1 | Viewer not session's starter + not org-member | [`session-launch-operations.md`](session-launch-operations.md) |
| 403 | `AGENT_NOT_MEMBER_OF_PROJECT` | 1 | Agent's `owning_org` mismatch | [`session-launch-operations.md`](session-launch-operations.md) |
| 403 | `PERMISSION_CHECK_FAILED_AT_STEP_<N>` | 1 | hard-deny at engine step N (0–6) | [`session-launch-permission-gate-operations.md`](../../m5_2/operations/session-launch-permission-gate-operations.md) |
| 403 | `TERMINATE_FORBIDDEN` | 1 | Caller not starter + not org Human | [`session-launch-operations.md`](session-launch-operations.md) |
| 404 | `AGENT_NOT_FOUND` / `PROJECT_NOT_FOUND` / `SESSION_NOT_FOUND` / `MODEL_RUNTIME_NOT_FOUND` | 1 | Id unknown | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `PARALLELIZE_CAP_REACHED` | 1 | Agent at `profile.parallelize` limit | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `MODEL_RUNTIME_UNRESOLVED` | 1 | No `model_config_id` bound | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `MODEL_RUNTIME_ARCHIVED` | 1 | Bound runtime archived | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `AGENT_PROFILE_MISSING` | 1 | No profile row | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `SESSION_ALREADY_TERMINAL` | 0 (idempotent) | Already Completed/Aborted/FailedLaunch | [`session-launch-operations.md`](session-launch-operations.md) |
| 409 | `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` | 1 | Agent has live sessions; `model_config_id` change refused | [`session-launch-operations.md`](session-launch-operations.md) |
| 410 | `SESSION_LIVE_STREAM_UNAVAILABLE` | 1 | SSE on a finalised session | [`session-live-stream-operations.md`](../../m5_2/operations/session-live-stream-operations.md) |
| 503 | `SESSION_WORKER_SATURATED` | 1 | Per-worker registry full | [`session-launch-operations.md`](session-launch-operations.md) |

### Authority templates surface (page 12)

| HTTP | Stable code | CLI exit | Symptom | Runbook |
|---|---|---|---|---|
| 400 | `VALIDATION_FAILED` | 2 | Manifest / selector / payload invalid | [`authority-templates-operations.md`](authority-templates-operations.md) |
| 403 | `ACCESS_DENIED` | 1 | AR-mutation rejected; 4 mutation callsites + per-state ACL | [`auth-request-access-acl-operations.md`](../../m5_2/operations/auth-request-access-acl-operations.md) |
| 404 | `TEMPLATE_NOT_FOUND` / `AUTH_REQUEST_NOT_FOUND` | 1 | Id unknown | [`authority-templates-operations.md`](authority-templates-operations.md) |
| 409 | `TEMPLATE_ALREADY_ADOPTED` | 1 | Org already has active adoption AR for this template+kind | [`authority-templates-operations.md`](authority-templates-operations.md) |

### System agents surface (page 13)

| HTTP | Stable code | CLI exit | Symptom | Runbook |
|---|---|---|---|---|
| 403 | `FORBIDDEN_NON_ADMIN_TUNE` | 1 | Non-admin tried to tune a standard system agent | [`system-agents-operations.md`](system-agents-operations.md) |
| 404 | `SYSTEM_AGENT_NOT_FOUND` | 1 | System agent id unknown / not bucketed | [`system-agents-operations.md`](system-agents-operations.md) |
| 409 | `SYSTEM_AGENT_ALREADY_DISABLED` / `..._ARCHIVED` | 0 (idempotent) | Lifecycle terminal state | [`system-agents-operations.md`](system-agents-operations.md) |

### Cross-cutting codes (inherited from M4 + s02/s03)

| HTTP | Stable code | CLI exit | Symptom | Runbook |
|---|---|---|---|---|
| 400 | `VALIDATION_FAILED` | 2 | Shape mismatch | inherited M4 |
| 500 | `AUDIT_EMIT_FAILED` | 1 | Audit-emit path failed after durable write | inherited M4 |
| 500 | `RECORDER_FAILURE` / `COMPOUND_TX_FAILURE` / `REPOSITORY_ERROR` / `AUDIT_EMIT_ERROR` / `SESSION_REPLAY_PANIC` | 1 | Internal failure | [`session-launch-operations.md`](session-launch-operations.md) §"Server-side failures" |
| 500 | `INTERNAL` | 1 | Unexpected | check logs |

### CLI exit-code summary

| Exit | Meaning |
|---|---|
| 0 | Success (incl. idempotent no-op) |
| 1 | Server returned 4xx/5xx with a stable code (operator-actionable) |
| 2 | Local input validation failure (CLI flag combinations / file parse) |

See [`m5/user-guide/cli-reference-m5.md`](../user-guide/cli-reference-m5.md) for per-subcommand exit-code maps.

## Incident playbooks

### IP-1 — Stuck "running" session

**Symptom:** session row in SurrealDB has `governance_state = Running` long after expected completion; live SSE clients on `/sessions/:id/events` hang without keep-alive.

**Likely cause:** the spawn task's `finalise_and_persist` call failed, or the task panicked, or the pod terminated mid-flight.

**Fix:**
1. Check server logs for `sessions::launch recorder finalise failed` or a panic stack.
2. Manually terminate via `POST /sessions/:id/terminate` with a reason; the terminate path is robust to a stuck registry entry.
3. If the registry has an orphan but the session row is still `Running`, the M7b SIGTERM hard-clear path (CHK8S-D-01) is needed; at M5 / single-pod, restart the server — the migration ledger handles a clean restart.

See [`session-launch-operations.md`](session-launch-operations.md) §"Incident playbooks" + [`m7b/architecture/deferred-from-ch-k8s-prep.md` CHK8S-D-01](../../m7b/architecture/deferred-from-ch-k8s-prep.md).

### IP-2 — Agent-catalog row missing for a known agent

**Symptom:** an agent exists (visible via `GET /agents/:id`) but no `agent_catalog_entry` row appears; runtime-status tile shows nothing.

**Likely cause:** the agent was created via a test fixture or a direct repository call that bypasses the production HTTP handler (which emits the lifecycle event the catalog listener consumes).

**Fix:**
1. Verify via `GET /agents/:id?include=catalog` (if extant) or query the catalog table directly.
2. To repopulate, emit a no-op lifecycle event via `PATCH /agents/:id` with a trivial profile update — the catalog listener fires on `AgentProfileUpdated` and upserts the row.
3. Long-term: ensure all agent creation paths go through `POST /api/v0/orgs/:org/agents` or `POST /platform/system_agents/add`.

See [`system-agents-operations.md`](system-agents-operations.md) §"Catalog row missing".

### IP-3 — Memory extraction skipped (no `Memory` rows + Identity counter stays at 0)

**Symptom:** sessions end, but no `Memory` rows appear for the agent, and `Identity.witnessed.memories_extracted` stays at 0.

**Likely cause:** the org's `memory-extraction-agent` system agent is `active = false` or `archived_at` is populated. Per ADR-0040 §D40.3 SKIP-BOTH semantics, the listener short-circuits before minting a Memory or firing telemetry.

**Fix:**
1. Re-enable the system agent via `POST /platform/system_agents/{id}/enable` (CH-01 surface).
2. Past sessions that ended while disabled are NOT replayed at v0 — extraction is forward-only. New sessions will extract.
3. To force-recompute past sessions, the operator-driven recompute path is M6 scope (see successor M6-DEFERRED-04 in [m7b/architecture/deferred-from-ch-k8s-prep.md](../../m7b/architecture/deferred-from-ch-k8s-prep.md)).

See [`m5_2/operations/memory-extraction-operations.md`](../../m5_2/operations/memory-extraction-operations.md) §"Operator-disabled extractor".

### IP-4 — Page-11 `recent_sessions` panel empty after launches (CH-24 amendment)

**Symptom:** operator clicks page 11 for a project with known recent sessions, but the panel shows no rows.

**Likely cause:**
1. (Most common at any HEAD post-CH-24:) The project genuinely has no sessions launched against it yet. Verify via `GET /api/v0/orgs/:org/projects/:proj/sessions` (the unbounded live-list endpoint) — if empty, no sessions to display.
2. (Pre-CH-24 only:) Server runs a binary built before CH-24; the M4 placeholder `Vec::new()` is still hardcoded. **Resolution:** redeploy from a post-CH-24 binary. Check via `grep -n "list_recent_sessions_for_project" $(which phi-server)` or the binary's git SHA.
3. (Rare:) Repository read errored silently — check server logs for `ProjectError::Repository` near a `GET /projects/:id` request. The panel returns empty if the underlying `list_recent_sessions_for_project` query fails (e.g., DB connection drop).

**Fix:**
1. Launch a session and re-check the panel. The new repo method `Repository::list_recent_sessions_for_project(project_id, 10)` returns top-10 newest sessions for the project, ordered by `started_at` DESC.
2. The panel is bounded to 10 rows by `RECENT_SESSIONS_LIMIT` at `server/src/platform/projects/detail.rs`. Projects with > 10 sessions show only the newest 10 in the panel; use the unbounded live-list endpoint for full history.

See ADR-0059 §D59.1 (cardinality), §D59.2 (query shape), §D59.3 (struct shape).

### IP-5 — Template A grant gap (post-CH-15 + CH-17 amendments)

**Symptom:** Project leads get 403 `PERMISSION_CHECK_FAILED_AT_STEP_2: NoGrantsHeld` on `POST /sessions` despite holding a Template A adoption AR.

**Likely cause:** Pre-CH-15 Template A grants minted only `project_grant` (`[Read, Inspect, List]` on `project:<uuid>`); they're missing the paired `session_grant` (`[Read, Inspect, List, Observe]` post-CH-17 on `tags contains "project:<uuid>" AND #kind:session`). The session-launch hard-deny at Step 2 fires when the lead has no `session_object` grant.

**Fix:**
1. Run migration `0015_template_a_session_object_grant.surql` (the migration runner applies it on next boot per ADR-0033 §D33.2 ledger pattern).
2. If post-CH-17 and SSE tail also fails 403, run migration `0016_template_a_session_object_grant_add_observe.surql` (the per-grant action-array extension).
3. Alternative: re-emit `HasLeadEdgeCreated` for the project so the listener mints both grants fresh.

See [`session-launch-permission-gate-operations.md`](../../m5_2/operations/session-launch-permission-gate-operations.md) §"NoGrantsHeld at Step 2".

## Cross-org isolation invariants (M5)

Every M5 surface enforces:
1. Org-membership boundary — a viewer from Org-A reading a resource owned by Org-B receives 403 / 404 (silent-list-filter or hard-deny per the F3.B.list-filter.a policy).
2. Page-11 `recent_sessions` panel for an Org-B project is NOT visible to Org-A viewers (cross-org isolation; verified by `acceptance_m5_sessions.rs::m5_cross_org_isolation_at_session_surface`).
3. AuthRequest mutations are gated by the per-state ACL at every state (CH-18 / ADR-0056 §D56.5); 17 production callsites consult `check_auth_request_access`.

## Cross-references

- [Top-level runbook §M5](../../../../../../docs/ops/runbook.md) — operator-facing aggregated index.
- [M5 troubleshooting (CLI exit codes + symptoms)](../user-guide/troubleshooting.md).
- [M4 troubleshooting (inherited codes + cross-org)](../../m4/user-guide/troubleshooting.md).
- [Session launch operations (page 14 detail)](session-launch-operations.md).
- [Authority templates operations (page 12 detail)](authority-templates-operations.md).
- [System agents operations (page 13 detail)](system-agents-operations.md).
- [System flows s02/s03 operations](system-flows-s02-s03-operations.md).
- [m5_2/operations/* — cross-cutting runbooks](../../m5_2/operations/).
- [ADR-0059 §D59.1–§D59.4 — recent_sessions API-surface flip](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md).
