<!-- Last verified: 2026-05-16 by Claude Code (CH-25 P3 — amendment appended below for owner-grant auto-issue rule per ADR-0060. The CEO/creator-Agent gains auto-owner authority over the resulting Org via `Edge::Owns` + synth-grant in `step_2_resolve_grants`. Cycle hex `1e01618e`.) -->
<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — verified-header re-stamp per plan §3.C; body content unchanged from CH-17 close. CH-24 milestone-seal cycle — first-session walkthrough re-verified PASS against current HEAD; happy-path + Template A grant + SSE tail amendments all still correct. Cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-05-09 by Claude Code (CH-17 amendment: live SSE tail at GET /api/v0/sessions/:id/events — operators now see events as they happen rather than only at session-end; CLI default-tail flips to real stream consumption; new --no-tail flag preserves the prior detach-equivalent behaviour. Drift D7.1 closed via ADR-0055; cycle hex 40c4d759.) -->
<!-- Last verified: 2026-05-08 by Claude Code (CH-15 amendment: hard-deny launch gate post-CH-15 + Template A grant-seeding requirement for happy-path launches; D4.1 closed.) -->
<!-- CH-02 amendment (2026-04-24): real `agent_loop()` + MockProvider expectations + `mock_response` operator field. Full prose tour deferred to M5-tag-close. -->
<!-- CH-22 amendment (2026-04-27): note that catalog row gets seeded on first agent creation — visible side-effect users can verify post-launch. -->
<!-- CH-15 amendment (2026-05-08): every Decision::Denied at the engine returns 403 (drift D4.1 closed). Template A grants on `session_object` are required for the lead agent to launch. -->

# First session launch — walkthrough

**Status**: [PARTIAL M5/P4] — full prose tour deferred to M5-tag-close; CH-02 + CH-22 amendments below describe the operator-visible behaviour shipped post-M5/P4.

## End-to-end shape (high level)

`phi bootstrap claim` → first org → first agent → first project → first session → verified memory + catalog updates.

The full transcript with screenshots + common-pitfall callouts ships at M5-tag-close. Below is what changed since the M5/P4 stub seeded.

## CH-02 amendment — what users see during a session (2026-04-24)

A first session at M5 (post-CH-02) is **deterministic** — the agent runs `phi_core::agent_loop()` against a `MockProvider`, not a real LLM. Real LLM providers ship at M7. Default behaviour:

1. Operator submits `POST /api/v0/orgs/:org/projects/:project/sessions` with a prompt.
2. Launch returns `LaunchReceipt { session_id, first_loop_id, ... }`.
3. The spawned agent_loop emits `AgentStart → TurnStart → MessageUpdate("Acknowledged.") → TurnEnd → AgentEnd`.
4. `BabyPhiSessionRecorder` persists Session + LoopRecordNode + TurnNode rows; `governance_state` flips to `Completed`.
5. `GET /api/v0/sessions/:id` shows the persisted transcript with the assistant message `"Acknowledged."`.

### Pinning a deterministic test response (`mock_response`)

To make a specific agent return a custom canned string instead of `"Acknowledged."`:

```bash
phi agent update --id <agent-uuid> --mock-response "I'm a fixture; I always say this."
# OR via HTTP:
curl -X PATCH http://localhost:8080/api/v0/agents/<agent-uuid>/profile \
  -H "Content-Type: application/json" \
  -d '{"mock_response": "I'\''m a fixture; I always say this."}'
```

The `mock_response` field lives on the baby-phi `AgentProfile` (NOT on the phi-core blueprint). Migration `0006` adds the column. Set it to `null` to revert to the default `"Acknowledged."`.

### Cancellation behaviour

`POST /api/v0/sessions/:id/terminate` cancels the loop in flight. Pre-CH-02 the synthetic feeder always finished before terminate could race; post-CH-02 the loop honours the cancellation token and the resulting `AgentEnd { rejection: Some("cancelled") }` is mapped to `governance_state = Aborted`.

## CH-15 amendment — Permission Check is real at session launch (2026-05-08)

Pre-CH-15, the launch handler advisory-logged every `Decision::Denied` from steps 1–6 and proceeded to spawn the agent task regardless. CH-15 closes that gap:

- Every `Decision::Denied` now returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_<N>` where `<N>` is the FailedStep variant's numeric label (0..6).
- The lead agent must hold a Template A grant on `session_object` (selector form `tags contains "project:<id>" AND tags contains #kind:session`). Production wiring mints this automatically on `HasLeadEdgeCreated`.
- Migration `0015_template_a_session_object_grant.surql` backfills legacy single-grant Template A holders so pre-CH-15 sessions continue to work after the deploy.
- Every step-1-to-6 deny emits a `platform.session.launch_denied` audit event (Alerted) with `failed_step` + `reason_kind` for dashboard correlation.

If a launch unexpectedly returns 403 with a `PERMISSION_CHECK_FAILED_AT_STEP_2: NoGrantsHeld` body, the lead is missing the paired `session_object` grant — verify Template A's adoption AR is approved and `HasLeadEdgeCreated` was emitted at project creation.

See [`session-launch.md`](../architecture/session-launch.md) Step 3 + [ADR-0054](../../m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md).

## CH-17 amendment — live SSE tail (2026-05-09)

Pre-CH-17, `phi session launch` printed `(live tail deferred to M7 — phi session show --id <id> inspects terminal state)` at line 228 of `cli/src/commands/session.rs` and operators only saw events post-finalisation via the `show` subcommand. CH-17 closes drift D7.1: a live SSE tail ships at `GET /api/v0/sessions/:id/events`.

What changed in the operator workflow:

- `phi session launch ...` (default) — events stream live as the agent_loop runs. The CLI subscribes to `/events`, prints each `AgentEvent` as it arrives (TurnStart, MessageUpdate, ContentBlock, TurnEnd, AgentEnd...), and exits when the SSE stream closes (session finalises) or you Ctrl-C.
- `phi session launch --no-tail ...` — NEW flag. Returns the JSON receipt only; no live tail. Parallel to `--detach` (preserved for wire stability).
- Direct `curl` works too:

```bash
curl -N http://localhost:8080/api/v0/sessions/<session-uuid>/events \
  -H "Cookie: <your session cookie>"
# event: agent_event
# data: {"kind":"TurnStart",...}
# event: agent_event
# data: {"kind":"MessageUpdate",...}
# (...)
# : keep-alive   # every 30s when no events flow
```

The SSE gate uses `Action::Observe` on `session_object` (NOT the launch's `[Read, Inspect, List]`). Project leads automatically have observe-grants because Template A's mint extends to `[Read, Inspect, List, Observe]` post-CH-17 (migration `0016` backfills legacy holders). Other actors who need to tail sessions need an explicit observe-grant — debug 403s by checking the `permission_check.decision` body in the audit-event row `platform.session.live_stream_denied` (Alerted-class).

Lagged consumers (slow client / LB hiccup) receive a typed `event: lagged` SSE event with the missed count, then the stream closes — reconnect to resume. See [`m5_2/operations/session-live-stream-operations.md`](../../m5_2/operations/session-live-stream-operations.md) for the full reconnect playbook + 410-on-finalised semantics.

See [ADR-0055](../../m5_2/decisions/0055-sse-broadcast-fanout-and-keepalive.md) for the full design rationale (broadcast buffer = 64; 30s keep-alive; F5.B `Action::Observe` lock; F5.B.subfork.a migration 0016).

## CH-22 amendment — verifying catalog side-effects (2026-04-27)

After your first session launches, the agent-catalog system agent's runtime tile advances (`last_fired_at` updates) because session start + end emit listener-trigger events. To verify:

```bash
curl http://localhost:8080/api/v0/orgs/<org-uuid>/system-agents | jq '.standard[] | select(.display_name == "agent-catalog")'
# Expect last_fired_at to be near "now" after a session run.
```

Per-agent catalog rows are also visible — each agent your org creates / updates / archives gets one entry that mirrors the agent's lifecycle state (`active = false` if disabled or archived).

## CH-25 amendment — owner-grant rule (2026-05-16)

When you create your first Org via the bootstrap-claim wizard (or via `POST /api/v0/orgs`), the compound transaction now emits TWO new edges in the same tx:

1. `Edge::Owns { from: AgentId, to: OwnedResourceId::Org(<id>) }` — captures ownership semantics. The CEO Agent created by the wizard owns the resulting Org.
2. `Edge::Created { from: NodeId, to: NodeId }` — captures provenance. The creator-Agent → Org provenance link is now first-class.

The CEO automatically gains `[Action::Allocate, Action::Transfer]` Authority over `org:<id>` through the Permission Check engine's synth-owner-grant rule (per [ADR-0060](../../m5_3/decisions/0060-agent-as-creator-and-owner.md)) — **WITHOUT** any persisted explicit grant. The synth fires inside `step_2_resolve_grants` whenever the calling Agent's `Owns` edges surface in `CheckContext.agent_owned_orgs/projects`.

**Operator-visible effect**: M6+ admin pages for Orgs (page 1) and Projects (page 3) can plan against a unified owner-grant model. A CEO can authorize operations on their own Org without explicit `[Allocate]` grants being pre-seeded — the engine infers the authority from the Owns edge.

The same Owns edge is emitted at project creation: the lead Agent (per Decision-3 user-lock: lead = creator at both Shape A and Shape B materialise paths) gains auto-owner authority on the resulting Project.

To inspect ownership relationships, see the operations runbook [`m5_3/operations/agent-ownership-operations.md`](../../m5_3/operations/agent-ownership-operations.md).

## Cross-references

- [requirements/admin/14-first-session-launch.md](../../../requirements/admin/14-first-session-launch.md).
- [CLI reference M5](cli-reference-m5.md) — `phi session launch` flag list.
- [Session launch architecture](../architecture/session-launch.md).
- [Session launch operations runbook](../operations/session-launch-operations.md).
- [ADR-0032](../../m5_2/decisions/0032-mock-provider-at-m5.md) — MockProvider at M5 (CH-02).
- [System agents walkthrough](system-agents-walkthrough.md) — CH-22 catalog-listener + audit-mode operator tour.
- [ADR-0060](../../m5_3/decisions/0060-agent-as-creator-and-owner.md) — CH-25 owner-grant rule.
