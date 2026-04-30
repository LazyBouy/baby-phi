<!-- Last verified: 2026-04-27 by Claude Code -->

# CH-22 — AgentCatalogListener body + D6.1 second call site

**Plan file token:** `c5f201bb` (generated via `openssl rand -hex 4`)
**Chunk ID:** CH-22 (see [forward-scope §1 CH-22 block](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) and [§5 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md))
**Severity:** HIGH
**Expected effort:** ~1.25 engineer-days
**Chunks enabled after close:** CH-23 (cross-listener acceptance suite verifies CH-22 + CH-21 ordering + idempotency)

---

## §1 — Context & principle

### Why this chunk

[`AgentCatalogListener`](../../../../modules/crates/domain/src/events/listeners.rs#L497) is a P3 stub today — it subscribes to 8 `DomainEvent` variants but its `on_event` body is a `tracing::debug!` no-op. The M5/P3 plan deliberately deferred the body to M5/P8b; CH-22 ships it. The body must:

1. **Mutate the `agent_catalog_entry` row** (1:1 per agent, UNIQUE INDEX on `agent_id` per migration 0005) for the relevant agent — refresh `display_name`, `kind`, `role`, `active`, `profile_snapshot`, `last_seen_at`, `updated_at`.
2. **Update the catalog-system-agent's `system_agent_runtime_status` tile** via the already-shipped `record_system_agent_fire` helper at [`listeners.rs:52-78`](../../../../modules/crates/domain/src/events/listeners.rs#L52) — closes drift D6.1's second call site (CH-21 closes the first).
3. **Honor ADR-0034 D34.5** — read `Agent.active` + `Agent.archived_at` via `repo.get_agent` to compute the catalog row's effective `active` field. Archive wins ties (`archived_at = Some(_)` ⇒ catalog `active = false` regardless of `Agent.active`). Listener is read-only on lifecycle.

This is drift **D6.1** (HIGH) second call site — the first ships with CH-21 (memory-extraction).

### CH-21 dependency classification

Forward-scope §1 lists CH-22's prerequisites as **CH-01 (now done) + CH-21**. Plan-time investigation classified CH-21 as a **soft ordering preference, not a hard blocker**:

- Both chunks close the same drift (D6.1) at independent call sites.
- Both call the same already-shipped helper (`record_system_agent_fire`) into independent rows of `system_agent_runtime_status` (1:1 per agent, no overlap).
- CH-22 reads no output produced by CH-21 (memory-extraction emits `MemoryExtracted` audit events; agent-catalog mutates `agent_catalog_entry` rows; orthogonal data flows).
- CH-23 (cross-listener acceptance) is the chunk that depends on **both** CH-21 + CH-22 having shipped — but that's a downstream dependency, not a CH-22 dependency.

The user explicitly selected CH-22 as next chunk after CH-01 (Q4 — user-decided per chunk-open). CH-22 opens before CH-21 and closes the same drift at its own call site. CH-21 follows independently.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* — M5.1 governing principle.

Applied here: `system-agents.md` §"Agent Catalog Agent" treats the catalog as a reactive cache with 8 trigger events. ADR-0034 D34.5 binds the listener to read `Agent.active`/`archived_at` from the repo, not infer from event payloads. Both must be honored in the body, not merely intended.

### Forward-scope reference

[Forward-scope §1 CH-22 block](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) + [§5 CH-22 row](../forward-scope/22035b2a-remaining-scope-post-m5-p7.md).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"Agent Catalog Agent" reactive updates | *"Subscribes to AgentEvent streams filtered for edge changes on MEMBER_OF, DELEGATES_TO, HAS_AGENT, HAS_PROFILE, and Authority Template fires (Template A/C/D edge creation/removal)."* | partially-honored (8-variant subscription wired at P3 stub; body is a no-op) | honored (body shipped; mutates catalog on every fire) |
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"Agent Catalog Agent" lifecycle events | *"On Agent creation, adds the agent to the catalogue with their profile reference. On role change in a project, updates the catalogue's project-role index. On archival, marks the agent **inactive** but retains the record for audit."* | contradicted (no upsert path today) | honored (body upserts row; archived agents flip to `active = false`) |
| [`system-agents.md`](../../v0/concepts/system-agents.md) | §"Agent Catalog Agent" queryable API | *"Returns agent IDs + profile summaries."* | partially-honored (struct + repo methods + query exist; no rows are ever populated) | honored (rows populated by the new body; queries return real data) |
| [ADR-0034](../../v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md) | §D34.5 conforming criteria for CH-22 | *"The listener MUST consult `Agent.active` via `repo.get_agent(agent_id)`... archived_at = Some(_) ⇒ terminally paused (is_paused = true) regardless of active... the listener body MUST NOT write to agent.active / agent.archived_at itself."* | not-yet-honored (ADR-0034 D34.5 ratified at CH-01 seal; binding contract for CH-22) | honored (body satisfies all 4 conforming criteria) |
| [`agent.md`](../../v0/concepts/agent.md) | §"Lifecycle" durable fields | *"Agents have an active vs disabled vs archived lifecycle. Disable pauses participation; archive is a terminal soft-delete."* | honored (CH-01 shipped the columns + read path) | honored (CH-22 consumes via repo.get_agent at listener time — no schema change) |

**Permissions subtree hook:** none. CH-22 does not touch grants, manifests, selectors, or actions.

**phi-core-mapping hook:** none. The catalog listener mutates baby-phi-only governance nodes (`AgentCatalogEntry`, `SystemAgentRuntimeStatus`); no phi-core types are wrapped or touched. `domain::Agent` is read via `repo.get_agent` (existing surface; no change). The connection point at `sessions/provider.rs::build_agent_context` is unchanged.

---

## §3 — phi-core leverage map

CH-22 touches no phi-core type. The listener handles `domain::events::DomainEvent` variants and mutates baby-phi governance nodes (`AgentCatalogEntry`, `SystemAgentRuntimeStatus`). All work lives in `domain` (listener body + tests) and `server` (config wiring + acceptance test).

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| (none) | — | — | — |

**Expected import-count delta at chunk close:** **0**. CH-22 adds no phi-core imports and removes none.

**Positive close-audit greps** (must return ≥ 1 each):
```bash
grep -n "fn on_event" modules/crates/domain/src/events/listeners.rs   # ≥ 4 (Template A + C + D + AgentCatalog)
grep -n "upsert_agent_catalog_entry" modules/crates/domain/src/events/listeners.rs   # ≥ 1 (CH-22 body wires it)
grep -n "record_system_agent_fire" modules/crates/domain/src/events/listeners.rs   # ≥ 2 (helper definition + AgentCatalogListener call site)
grep -n "CatalogAuditMode" modules/crates/server/src/config.rs   # ≥ 1
grep -n "audit_mode" config/default.toml   # ≥ 1
```

**Forbidden-duplication greps** (must return 0 each):
```bash
grep -rn "^pub struct AgentCatalogEntry\b" modules/crates/ | grep -v "domain/src/model/composites_m5.rs"   # 0
grep -rn "^pub struct SystemAgentRuntimeStatus\b" modules/crates/ | grep -v "domain/src/model/composites_m5.rs"   # 0
bash scripts/check-phi-core-reuse.sh   # exit 0
```

---

## §3.B — K8s microservice readiness check

Per the rule codified by CH-01 P0 (forward-scope §7 Q8 + per-chunk-template §3.B), every chunk evaluates whether its changes introduce new K8s-deployment hurdles.

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** In-process state (`DashMap`, `RwLock`, `AtomicBool`, mutex, `OnceCell`) | Listener struct grows by 1 field (`audit_mode: CatalogAuditMode`, a `Copy` enum). No new shared mutable state | No | — |
| **A2** IPC channel (mpsc, broadcast, oneshot, watch) | None added | No | — |
| **A3** Pod-local resource (file handle, listener socket, sub-process, lock file) | None added | No | — |
| **A4** Migration runner / first-apply race | No new migration; `agent_catalog_entry` + `system_agent_runtime_status` already shipped at migration 0005 | No (existing `CHK8S-D-05` lock-missing issue unaffected) | Cross-ref existing CHK8S-D-05 |
| **A5** Trait-shape requirement | Listener still implements existing `EventHandler` trait; constructor signature gains 1 param. The `EventBus` trait surface (CH-K8S-PREP P-4) is unchanged | No | — |
| **A6** Cross-pod state sharing | `agent_catalog_entry` and `system_agent_runtime_status` rows are durable SurrealDB columns; cross-pod-visible via `SurrealStore::open_remote` (CH-K8S-PREP P-2). No in-process cache. Listener fires on whichever pod consumes the event from the bus | No | — |
| **A7** Audit hash-chain symmetry | New audit event variant `agent_catalog_refreshed` emits ONLY in debug mode (default silent). When emitted, uses the existing `AuditEmitter` impl (no new writer). Single-writer guarantee preserved. Per-listener mode flag does not bypass the chain — it just elects whether to write at all | No | — |

**Conclusion:** CH-22 is **K8s-neutral**. No new blockers introduced; no new entries in `deferred-from-ch-k8s-prep.md` required.

**Conforming criteria for ADR-0033 (CH-K8S-PREP) still satisfied:**
- D33.1 (`SessionRegistry` trait) — untouched.
- D33.2 (`SurrealStore::open_remote`) — listener uses repo trait; remote-DB compatibility unchanged.
- D33.3 (SIGTERM graceful shutdown) — no new spawn tasks added; listener fires synchronously inside `EventBus::emit`.
- D33.4 (`EventBus.shutdown` + `drain`) — no new emitters; shutdown path unaffected.

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D6.1` | [`../../v0/implementation/m5_1/drifts/D6.1.md`](../../v0/implementation/m5_1/drifts/D6.1.md) | HIGH | `discovered → classified → scoped → in-chunk-plan → remediated (second call site)` | First call site closes at CH-21. CH-22's `AgentCatalogListener` body calls `record_system_agent_fire` at the top of every fire, populating the catalog system agent's runtime-status tile. Drift transitions to `remediated` only after BOTH call sites land. **Sequencing decision:** CH-22 ships first (per user-selected order); the drift's Status field flips from `discovered` to `in-chunk-plan` at this chunk's open. Final transition to `remediated` happens at CH-21 seal (drift's owner becomes CH-21 for terminal closure). CH-22's seal records a lifecycle entry: `2026-04-27 — second-call-site shipped via CH-22; awaiting CH-21 first-call-site for full closure`. |

**Drift sequencing rationale:** D6.1 names two call sites; whichever ships first transitions the drift to `in-chunk-plan` (chunk-open) and adds a `partial-closure` lifecycle entry at chunk-seal. The drift only flips to `remediated` when both call sites have shipped. CH-22 SHIPS the second call site first chronologically; its seal does NOT close the drift. CH-21 (whenever it lands) ships the first call site and triggers the final `remediated` transition.

---

## §5 — ADRs drafted

ADR number assigned at plan-drafting time per Q6 rule. Command used:
```bash
ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null \
  | xargs -I{} basename {} .md \
  | grep -oE "^[0-9]{4}" | sort -u | tail -5
# result: 0031, 0032, 0033, 0034 → next free = 0035
```

| # | Title | Drafted-at-phase | Decision summary | Flip to Accepted at |
|---|---|---|---|---|
| **ADR-0035** | AgentCatalogListener `audit_mode` configuration (silent default, debug opt-in) | Step 2 (pre-P1) | Listener fires up to N×session events; emitting an audit event on every fire would 10-100× the audit-log volume on busy orgs without governance benefit (catalog refresh is observability data, not permission-relevant). Default mode is **`silent`** (no audit emission); a `debug` mode emits `agent_catalog_refreshed` audit events for end-to-end traceability during dev + acceptance testing. Configured per-listener via `[listeners.catalog] audit_mode = "silent" \| "debug"` in `config/default.toml` with `PHI_LISTENERS__CATALOG__AUDIT_MODE` env override. Conforming criteria: future listeners that fire on high-volume events (≥1 per session) should adopt the same silent-default pattern unless governance-significant. | Chunk seal (P4) |

ADR file path: [`../../v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md`](../../v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| Post-CH-01 baseline | `cargo test --workspace -- --test-threads=1` = 997 passed | `/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1 2>&1 \| grep -E "^test result:" \| awk -F'[;: ]+' '{s+=$4} END {print s}'` |
| CH-01 | `Agent.active` + `archived_at` columns + `set_agent_active`/`set_agent_archived_at` repo methods + `repo.get_agent` returns the durable values | `grep -n "pub active: bool\|pub archived_at" modules/crates/domain/src/model/nodes.rs` ≥ 2 |
| CH-01 / ADR-0034 D34.5 | Conforming-criteria contract still applies (catalog listener reads via `repo.get_agent`; archive wins ties; listener read-only on lifecycle) | inspect ADR-0034 §D34.5; ensure listener body satisfies all 4 criteria |
| CH-K8S-PREP / ADR-0033 D33.4 | `EventBus.shutdown` + `drain` semantics still hold; listener fires are tracked by the in-flight counter | `grep -n "EmitGuard" modules/crates/domain/src/events/bus.rs` ≥ 1 |
| M5/P3 | `record_system_agent_fire` helper at `listeners.rs:52-78` is unchanged | `grep -n "pub async fn record_system_agent_fire" modules/crates/domain/src/events/listeners.rs` = 1 |
| M5/P3 | Sibling `TemplateAFireListener` / `TemplateCFireListener` / `TemplateDFireListener` body shapes remain canonical reference (CH-22's body mirrors their structure) | `grep -c "fn on_event" modules/crates/domain/src/events/listeners.rs` ≥ 5 (3 templates + memory-extraction stub + agent-catalog) |
| M5/P6 | `agent_catalog_entry` + `system_agent_runtime_status` schemas + repo methods unchanged | `cargo test -j 4 -p domain --test in_memory_m5_test` green; `cargo test -j 4 -p store --test repo_m5_surface_test` green |
| All chunks | 4 CI guards green | `bash scripts/check-doc-links.sh && bash scripts/check-ops-doc-headers.sh && bash scripts/check-phi-core-reuse.sh && bash scripts/check-spec-drift.sh` |

These run at chunk-open AND chunk-seal.

---

## §7 — Phases within the chunk

### P1 — Listener body + config wiring (~0.5d)

**Goal.** Replace the no-op stub body with the production logic that mutates `agent_catalog_entry` rows + calls `record_system_agent_fire` for the catalog system agent. Wire the new `audit_mode` config flag.

**Deliverables.**

1. **`CatalogAuditMode` enum** at [`modules/crates/server/src/config.rs`](../../../../modules/crates/server/src/config.rs) (or co-located with the listener if more natural):
   ```rust
   #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Default)]
   #[serde(rename_all = "snake_case")]
   pub enum CatalogAuditMode {
       #[default]
       Silent,
       Debug,
   }
   ```

2. **Server config extension** at [`modules/crates/server/src/config.rs`](../../../../modules/crates/server/src/config.rs):
   ```rust
   #[derive(Debug, Deserialize, Clone, Default)]
   #[serde(deny_unknown_fields)]
   pub struct ListenersConfig {
       #[serde(default)]
       pub catalog: ListenerCatalogConfig,
   }
   #[derive(Debug, Deserialize, Clone, Default)]
   #[serde(deny_unknown_fields)]
   pub struct ListenerCatalogConfig {
       #[serde(default)]
       pub audit_mode: CatalogAuditMode,
   }
   ```
   Add `pub listeners: ListenersConfig` field on `ServerConfig` with `#[serde(default)]`.

3. **Default config** at [`config/default.toml`](../../../../config/default.toml) — append:
   ```toml
   [listeners.catalog]
   # Audit-mode for AgentCatalogListener fires.
   #   silent (default): no audit events; catalog refresh is observability data, not governance.
   #   debug: emit `agent_catalog_refreshed` audit events on each fire (use during dev + acceptance tests).
   audit_mode = "silent"
   ```

4. **`AgentCatalogListener` constructor + struct** at [`modules/crates/domain/src/events/listeners.rs`](../../../../modules/crates/domain/src/events/listeners.rs#L497) — extend:
   ```rust
   pub struct AgentCatalogListener {
       repo: Arc<dyn Repository>,
       audit: Arc<dyn AuditEmitter>,
       audit_mode: CatalogAuditMode,
   }
   impl AgentCatalogListener {
       pub fn new(
           repo: Arc<dyn Repository>,
           audit: Arc<dyn AuditEmitter>,
           audit_mode: CatalogAuditMode,
       ) -> Self { ... }
   }
   ```

   **Note:** `CatalogAuditMode` lives in the `domain` crate (alongside the listener) so the trait + listener don't depend on the `server` crate. The `server` crate's config struct re-exports it.

5. **`on_event` body** — replace the stub. For each of the 8 event variants, derive `(agent_id, owning_org_hint)` from the payload, then:
   - Fetch the canonical `Agent` row via `repo.get_agent(agent_id).await`. If `None` → log `warn!` + `return` (event arrived for a deleted/missing agent — the row went away between event emission and listener fire; not an error).
   - Compute the catalog row:
     ```rust
     let catalog_active = agent.active && agent.archived_at.is_none();  // ADR-0034 D34.5
     ```
   - For `HasProfileEdgeChanged`: fetch `repo.get_agent_profile_for_agent(agent_id).await`; serialize `profile.blueprint` (the phi-core wrap field) as `serde_json::Value` for `profile_snapshot`. For other variants: preserve the current `profile_snapshot` if any (read-then-rewrite via `repo.get_agent_catalog_entry(agent_id)`).
   - Call `repo.upsert_agent_catalog_entry(&entry).await`. On `Err` → log `error!` + `return` (per ADR-0028 fail-safe).
   - Resolve the catalog system agent's `AgentId` via `repo.get_organization(agent.owning_org).await` → `org.system_agents` array. The catalog agent is identified by its `profile_ref` (slug `"agent-catalog"`) — pick whichever entry matches. If unresolvable → log `warn!` + skip the runtime-status update (catalog mutation already succeeded).
   - Compute `effective_parallelize` from the catalog system agent's profile (`repo.get_agent_profile_for_agent` on the system-agent's id). If profile missing → use the org default from `org.defaults_snapshot.execution_limits` or fall back to `1`.
   - Call `record_system_agent_fire(repo.as_ref(), org_id, catalog_system_agent_id, effective_parallelize, None /* last_error */, now).await`.
   - **If `self.audit_mode == Debug`**: build `agent_catalog_refreshed` audit event (new variant); emit via `self.audit.emit(...).await`. Log error on failure but don't return.
   - **`SessionAborted`** stays a silent no-op (preserves current behavior).

6. **New audit event variant** `agent_catalog_refreshed` at [`modules/crates/domain/src/audit/events/m5/`](../../../../modules/crates/domain/src/audit/events/m5/) — file `agent_catalog.rs`. Builder: `pub fn agent_catalog_refreshed(actor: AgentId, org: OrgId, agent: AgentId, triggering_event_id: AuditEventId, event_kind: &str, now: DateTime<Utc>) -> AuditEvent`. Audit class: `AuditClass::Lite` (debug-only, low retention).

7. **Server bootstrap wire** at [`modules/crates/server/src/main.rs`](../../../../modules/crates/server/src/main.rs) (or wherever `state::build_event_bus_with_m5_listeners` is called) — pass `cfg.listeners.catalog.audit_mode` into the `AgentCatalogListener::new` call.

8. **`build_event_bus_with_m5_listeners` signature** at [`modules/crates/server/src/state.rs`](../../../../modules/crates/server/src/state.rs) — add a `catalog_audit_mode: CatalogAuditMode` parameter; pass through.

**Tests.** No new tests in P1; tests land in P2.

**Concept-alignment check.** `system-agents.md` rows transition from `partially-honored` / `contradicted` to `honored` at the structural layer (body shipped). P2 verification confirms behavior.

**phi-core leverage check.** No phi-core surface touched. `check-phi-core-reuse.sh` green.

**Confidence target.** ≥ 97%.

**Pause discipline.** If `Organization.system_agents` does not carry slugs/profile_refs that distinguish the catalog system agent from memory-extraction, pause via `AskUserQuestion` — the resolver design needs to change.

---

### P2 — Listener body tests (~0.4d)

**Goal.** Cover the listener body's behavior for all 8 event variants + the audit-mode flag + ADR-0034 D34.5 conforming criteria.

**Deliverables.**

1. **Unit tests in `listeners.rs::tests`** — extend the existing `mod tests` block (where the P3 stub test currently lives at `agent_catalog_listener_is_a_noop_at_p3`). Replace + add:
   - `agent_catalog_listener_upserts_row_on_agent_created` — emit `AgentCreated`; assert `repo.get_agent_catalog_entry(agent_id).await` returns `Some(entry)` with `active = true`, `display_name = agent.display_name`, `kind = agent.kind`, `role = agent.role.as_str()`.
   - `agent_catalog_listener_flips_active_on_agent_archived` — pre-seed agent with `active = false` + `archived_at = Some(_)` via `repo.set_agent_active(false)` + `set_agent_archived_at(Some(now))`; emit `AgentArchived`; assert catalog row's `active = false`. (D34.5 archive-wins-ties.)
   - `agent_catalog_listener_archive_wins_when_active_true_but_archived_at_some` — set `agent.active = true` + `agent.archived_at = Some(_)`; emit any of the 8 variants; assert catalog `active = false`. (D34.5 #3 explicitly.)
   - `agent_catalog_listener_listener_is_read_only_on_lifecycle` — pre-seed `agent.active = true` + `archived_at = None`; emit `AgentArchived` event (carrying the same agent_id); after the listener fires, `repo.get_agent(agent_id).await` STILL returns `active = true` + `archived_at = None`. (D34.5 #4 — listener does not write back to agent columns.)
   - `agent_catalog_listener_refreshes_profile_snapshot_on_has_profile_edge_changed` — emit `HasProfileEdgeChanged`; assert `entry.profile_snapshot` contains the new profile's `system_prompt` field as a JSON string.
   - `agent_catalog_listener_touches_last_seen_at_on_session_started` — emit `SessionStarted` at time T1; assert `entry.last_seen_at >= T1`.
   - `agent_catalog_listener_touches_last_seen_at_on_session_ended` — same shape with `SessionEnded`.
   - `agent_catalog_listener_role_index_refresh_on_has_lead_edge_created` — emit `HasLeadEdgeCreated`; assert `entry.updated_at >= event_timestamp` (catalog row touched, even though no field on the entry is "lead-specific" at M5 — the touch is the signal).
   - `agent_catalog_listener_role_index_refresh_on_manages_edge_created` — same pattern with `ManagesEdgeCreated`.
   - `agent_catalog_listener_role_index_refresh_on_has_agent_supervisor_edge_created` — same pattern with `HasAgentSupervisorEdgeCreated`.
   - `agent_catalog_listener_silently_ignores_session_aborted` — emit `SessionAborted`; assert `repo.get_agent_catalog_entry(agent_id).await` returns `None` (no row created) and `audit.events` is empty (regardless of audit_mode).
   - `agent_catalog_listener_emits_audit_in_debug_mode` — construct listener with `audit_mode = Debug`; emit `AgentCreated`; assert `audit.events.len() == 1` with `event_type == "agent_catalog_refreshed"` and `provenance` referencing the triggering event's `event_id`.
   - `agent_catalog_listener_silent_in_default_mode` — construct with `audit_mode = Silent` (default); emit `AgentCreated`; assert `audit.events.is_empty()`.
   - `agent_catalog_listener_skips_when_agent_row_missing` — emit `AgentArchived` for an agent that doesn't exist in the repo; assert no panic, no catalog row created, no audit event (warn-and-skip path).
   - `agent_catalog_listener_runtime_status_tile_advances_on_fire` — confirm `record_system_agent_fire` was called: `repo.fetch_system_agent_runtime_status_for_org(org).await` returns at least 1 row whose `agent_id` is the catalog system agent and `last_fired_at >= event_timestamp`.

2. **Test fixtures** — minor additions to the existing test helpers (`sample_event` etc.) for the 5 event variants the AgentCatalogListener subscribes to but TemplateA/C/D don't trigger directly (`AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged`, `SessionStarted`, `SessionEnded`).

3. **Update P3 stub test** — rename `agent_catalog_listener_is_a_noop_at_p3` → DELETE (replaced by the ~14 behavior tests above); the stub-noop semantics are no longer the contract.

**Tests.** ~14 new unit tests. Baseline 997 → ~1011.

**Concept-alignment check.** All §2 rows confirmed `honored` via behavioral tests.

**phi-core leverage check.** No new phi-core imports.

**Confidence target.** ≥ 97%.

**Pause discipline.** If the `record_system_agent_fire` helper requires data the listener body cannot resolve at fire time (e.g., the catalog system agent itself isn't created at this point in the test setup), pause and surface — fixture-level test setup may need to mirror the org-creation compound tx more faithfully.

---

### P3 — Acceptance test + concept-doc bumps + ADR + drift + audit + seal (~0.35d)

**Goal.** End-to-end acceptance test (HTTP-driven) that exercises the listener through the real event bus, then chunk-seal governance.

**Deliverables.**

1. **Acceptance test** at [`modules/crates/server/tests/acceptance_system_flows_s03.rs`](../../../../modules/crates/server/tests/acceptance_system_flows_s03.rs) — new file (per plan archive C18). Two scenarios:
   - **Scenario A — Agent creation populates catalog.** Spawn `spawn_claimed_with_org`. POST `/api/v0/orgs/:org/agents` with a Human Member. Assert (a) `repo.get_agent_catalog_entry(agent_id).await` returns `Some(entry)` with `active = true`, (b) `repo.fetch_system_agent_runtime_status_for_org(org).await` returns ≥ 1 row with `last_fired_at >= test_start`.
   - **Scenario B — Archive flips catalog active.** Same setup, but the agent is a system agent (so `archive` is supported) — POST `/api/v0/orgs/:org/system-agents/:agent_id/archive` (after CH-01's archive handler shipped the durable flip). Assert catalog row's `active = false` AND `repo.get_agent(agent_id).await.archived_at.is_some()` (verifies the listener fired in response to the chain `AgentArchived event → listener → catalog upsert with archive-wins`).

2. **ADR-0035** at [`docs/specs/v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md`](../../v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md) — Status `Proposed` → `Accepted` at chunk seal. Body sections:
   - **Status:** Accepted (at chunk seal P3).
   - **Decided at:** CH-22 chunk-seal, 2026-04-27.
   - **Context.** Listener fires up to 8 events × every session (every session-launch → `SessionStarted` and every session-finalize → `SessionEnded`). On a busy org with 100 agents and 50 daily sessions, that's ~10,000 fires/day — emitting an audit event per fire would 10-100× the audit-log volume without governance benefit. Catalog refresh is observability data, not permission-relevant.
   - **Decision.** D35.1 — listener gains a `CatalogAuditMode` enum (`Silent` / `Debug`); `Silent` is the default. D35.2 — config flag at `[listeners.catalog] audit_mode` in `config/default.toml` with `PHI_LISTENERS__CATALOG__AUDIT_MODE` env override (per existing nested-config pattern). D35.3 — `Debug` mode emits `agent_catalog_refreshed` audit events with `AuditClass::Lite` (debug-tier retention; default lower than governance-tier). D35.4 — conforming criteria for future high-volume listeners: any listener that fires ≥ 1× per session SHOULD adopt the silent-default pattern unless governance-significant (e.g., grant-mint listeners always audit).
   - **Ratification evidence.** Code locations + test names.
   - **Consequences.** Positive: lean audit log in prod; debug-mode opt-in for dev/test still gives reconstruction power. Negative: `Debug` mode's audit event schema becomes a stable contract. Neutral: per-listener config feels like first instance of "listener-local config" pattern; the same shape can be extended to memory-extraction listener at CH-21 if needed.
   - **Alternatives considered.** Tracing-level gate (rejected — conflates audit chain with log level); always-emit (rejected — log volume); sampling rate (rejected — over-engineered).
   - **Review trigger.** If the M6 query-API for the catalog needs richer audit reconstruction OR if M7b production observability requires audit events for compliance, revisit.

3. **D6.1 drift file** [`drifts/D6.1.md`](../../v0/implementation/m5_1/drifts/D6.1.md) — Status `discovered` → `in-chunk-plan` (chunk-open) → **stays** `in-chunk-plan` (CH-22 ships second call site only; CH-21 is the closure owner). Append lifecycle entries:
   - `2026-04-24 — scoped — assigned to CH-21 (first call site) + CH-22 (second call site) at M5.1/P3 close (backfill)`
   - `2026-04-27 — in-chunk-plan — CH-22 plan approved (build/ch-22-agent-catalog-listener-body-c5f201bb.md); chunk now operating on second call site`
   - `2026-04-27 — partial-closure — CH-22 chunk-seal; second call site shipped (AgentCatalogListener body calls record_system_agent_fire on every fire); awaiting CH-21 first call site for full remediation. Status held at in-chunk-plan; final remediated transition logged at CH-21 seal.`
   `Last verified` bumped to today.

4. **`drifts/README.md`** index — D6.1 Status column reflects partial closure: `in-chunk-plan (CH-22 ✓; CH-21 pending)`.

5. **`_concept-audit-matrix.md`** — flip rows for `system-agents.md §"Agent Catalog Agent" reactive updates` + `lifecycle events` + `queryable API` from `partially-honored`/`contradicted` to `honored`. Add code-evidence cites for `AgentCatalogListener::on_event` + the new acceptance test.

6. **`system-agents.md` verified-header bump** — append CH-22 amendment note: *"AgentCatalogListener body shipped (CH-22, 2026-04-27): catalog rows mutate on 8 trigger variants; D34.5 conforming criteria honored."*

7. **Spawn 1 audit agent** (per the rule: 3 phases = 1 agent). Locked prompt at §11.

**Tests.** +2 acceptance scenarios. ~1011 → ~1013.

**Concept-alignment check.** All §2 rows at target-status.

**phi-core leverage check.** Final green sweep — `check-phi-core-reuse.sh` exit 0; forbidden-duplication greps all 0; positive greps all ≥ 1.

**Confidence target.** ≥ 99% (chunk seal target).

**Pause discipline.** If the audit agent surfaces a finding (concept-doc claim missed, drift transition wrong, ADR sub-decision incomplete), pause and surface to user before seal.

---

## §8 — Tests summary

- **Expected total test count at chunk close:** 997 (post-CH-01 baseline) + ~16 new = **~1013** serialised passing.
  - +0 P1 (config + body, no tests)
  - +14 P2 (listener body unit tests)
  - +2 P3 (acceptance scenarios)
- **Layer breakdown:**
  - Unit (`domain/src/events/listeners.rs::tests`): +14
  - Acceptance (`server/tests/acceptance_system_flows_s03.rs`): +2 (new file)
  - Integration: 0
- **New test files:**
  - `modules/crates/server/tests/acceptance_system_flows_s03.rs`
- **Expected-still-green fragile tests:**
  - `acceptance_system_agents.rs` — disable / archive happy-path tests (they pre-date the listener body; CH-22 must not regress them — specifically the `archive_with_confirm_succeeds_and_flips_durable_archived_at` test should still pass AND should now ALSO cause a catalog upsert side-effect, observable via the listener fire).
  - `acceptance_agents_profile.rs` — entire suite (the new listener fires on `AgentCreated` for every test that creates an agent; must not race or panic).
  - All `listeners.rs::tests` for Template A/C/D (CH-22 only modifies `AgentCatalogListener` and the helper is unchanged; sibling listeners must remain untouched).

---

## §9 — Pre-chunk gate

**Reading list (drafter reads before chunk-open ritual completes):**
1. Concept docs: [`system-agents.md`](docs/specs/v0/concepts/system-agents.md) §"Agent Catalog Agent", [`agent.md`](docs/specs/v0/concepts/agent.md) §"Lifecycle".
2. Drift file: [`D6.1.md`](docs/specs/v0/implementation/m5_1/drifts/D6.1.md).
3. ADR-0034: [`0034-agent-durable-lifecycle.md`](docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md) §D34.5 (conforming criteria CH-22 must satisfy).
4. Process: [`per-chunk-planning-template.md`](docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md) (incl. CH-01-codified §3.B), [`chunk-lifecycle-checklist.md`](docs/specs/v0/implementation/m5_1/process/chunk-lifecycle-checklist.md), [`drift-lifecycle.md`](docs/specs/v0/implementation/m5_1/process/drift-lifecycle.md).
5. Forward-scope: [`22035b2a-...md`](docs/specs/plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §1 CH-22 + §1 CH-21 + §7 Q4/Q5/Q8.
6. [`baby-phi/CLAUDE.md`](CLAUDE.md) §phi-core Leverage rules + §"Orthogonal surfaces".
7. Sibling listener bodies: [`TemplateAFireListener::on_event`, `TemplateCFireListener::on_event`, `TemplateDFireListener::on_event`](modules/crates/domain/src/events/listeners.rs) — canonical pattern reference.
8. Existing P3 stub + `record_system_agent_fire` helper at [`listeners.rs:52`](modules/crates/domain/src/events/listeners.rs#L52).
9. CH-01 plan: [`build/ch-01-agent-durable-lifecycle-2aa37c80.md`](docs/specs/plan/build/ch-01-agent-durable-lifecycle-2aa37c80.md) (style + structure reference).

**Carry-forward invariants** (verified green at chunk-open):
- `cargo test --workspace -- --test-threads=1` = 997 (post-CH-01 baseline).
- 4 CI guards green.
- `git diff --stat HEAD -- modules/` empty after the CH-01 commit lands. (CH-01 commit is currently pending user authorization. CH-22 chunk-open should NOT proceed until CH-01 is committed — otherwise the chunk-open `git diff` carry-forward invariant fails.)
- Highest applied migration is 0007 (CH-01).
- ADR-0034 Accepted with D34.5 binding criteria.

**Pending decisions carried into this chunk:**
- Forward-scope Q4 (chunk ordering): user selected CH-22 next after CH-01. CH-21 dependency reclassified as soft per Phase 1 investigation.
- Forward-scope Q5 (M5 scope): CH-22 closes one HIGH drift partially (D6.1 second call site); CH-21 will close it fully. Per Q5, HIGH drifts must close before M5 tag — the joint CH-21 + CH-22 closure satisfies this.
- ADR-0034 D34.5 conforming criteria are binding.
- Q8 K8s readiness rule applies — §3.B above evaluated and CH-22 declared K8s-neutral.

**Cargo command convention (per user feedback 2026-04-27):** all cargo invocations during P1/P2/P3 implementation use `-j 4` to cap workers. Tests still serialise via `--test-threads=1` (project convention).

**Chunk-ordering note.** No predecessor in §6 has hard dependencies on CH-22's pre-state beyond CH-01.

---

## §10 — Close criteria

**5 aspects (each PASS or FAIL; no partial credit):**

- **Code aspect** — `cargo test -j 4 --workspace -- --test-threads=1` green at ~1013; clippy green under `RUSTFLAGS="-Dwarnings"` with `-j 4`; `cargo fmt --all -- --check` green; `acceptance_system_flows_s03.rs` 2/2 pass.
- **Docs aspect** — D6.1 lifecycle entries present (chunk-open + partial-closure); `_concept-audit-matrix.md` 3 rows flipped; `drifts/README.md` reflects D6.1 partial closure; ADR-0035 Accepted; `system-agents.md` verified-header bumped.
- **phi-core leverage aspect** — import-count delta = **0**; positive greps (§3) ≥ 1 each; forbidden-duplication greps = 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row at target-status; ADR-0034 D34.5 conforming criteria 1-4 verified by P2 unit tests + P3 acceptance tests.
- **K8s readiness aspect** — §3.B 7-axis table populated with conclusions; CH-22 declared K8s-neutral; no new CHK8S-D-XX entries.

**Two confidence % (named numerator/denominator):**

- **Implementation confidence** = `claims-verified-honored-by-tests-and-code-inspection / claims-in-scope-for-chunk` = target **8/8 = 100%**. The 8 claims:
  1. Listener mutates `agent_catalog_entry` row on `AgentCreated`.
  2. Listener flips catalog `active` to false on `AgentArchived` (durable, via D34.5 read).
  3. Listener refreshes `profile_snapshot` on `HasProfileEdgeChanged`.
  4. Listener touches `last_seen_at` on `SessionStarted` + `SessionEnded`.
  5. Listener role-index-touches catalog row on the 3 edge-creation variants (HasLead / Manages / HasAgentSupervisor).
  6. Listener silently ignores `SessionAborted`.
  7. Listener calls `record_system_agent_fire` on every fire (catalog system-agent's runtime-status tile advances).
  8. Audit emission gated by `audit_mode` config flag (silent default; debug emits `agent_catalog_refreshed` events).
- **Documentation confidence** = `doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity / doc-pages-touched-in-chunk` = target **6/6 = 100%**.

Touched doc pages (denominator):
1. ADR-0035 (`m5_2/decisions/0035-agent-catalog-listener-audit-mode.md`)
2. D6.1 drift (lifecycle append)
3. `_concept-audit-matrix.md` (3 row flips)
4. `drifts/README.md` (D6.1 status update)
5. `system-agents.md` header
6. `config/default.toml` (new `[listeners.catalog]` section)

**Composite = min(impl%, doc%, code-pass, leverage-pass, alignment-pass, k8s-readiness-pass).** Target ≥ 97% (chunk seal); ≥ 99% for the P3 seal phase specifically. Composite below target blocks close. No aspect-averaging, no rounding up.

---

## §11 — Post-chunk independent audit plan

**Agent count.** 3 phases (P1/P2/P3) = small chunk → **1 agent** (per per-chunk-planning-template.md guardrail: ≤3 phases = 1 agent).

**Audit aspects (a–e):**
- (a) Code correctness (P1 + P2 land cleanly; tests pass; ADR-0034 D34.5 conforming criteria honored).
- (b) Docs fidelity vs concept docs (ADR-0035 ratification; D6.1 lifecycle entries correct; matrix flipped).
- (c) Concept alignment (`system-agents.md` claims honored; D34.5 binding contract satisfied).
- (d) phi-core leverage (import-count delta = 0; no struct duplication).
- (e) K8s readiness rule applied (§3.B populated; no new ledger entries).

**Auditor constraint.** Fresh `Explore` subagent. Not the implementer.

### Audit Agent — CH-22 close audit (single agent)

> **Prompt** (locked at Step 2; fired at P3 seal):
> You are performing an independent audit of CH-22 (AgentCatalogListener body) in baby-phi at `/root/projects/phi/baby-phi/`. You did NOT write this code or docs.
>
> **Context:** CH-22 ships the `AgentCatalogListener::on_event` body that mutates `agent_catalog_entry` rows + calls `record_system_agent_fire` on every fire, satisfying ADR-0034 D34.5 conforming criteria. The chunk plan is at `docs/specs/plan/build/ch-22-agent-catalog-listener-body-c5f201bb.md`.
>
> Verify these claims against current HEAD. For each report **PASS** or **FAIL** with 1-line evidence:
>
> 1. `modules/crates/domain/src/events/listeners.rs` `AgentCatalogListener::on_event` body matches all 8 subscribed `DomainEvent` variants and calls `repo.upsert_agent_catalog_entry(...)` on each (except `SessionAborted` which is a documented no-op).
> 2. The body calls `repo.get_agent(agent_id).await` BEFORE upserting the catalog entry — verifying ADR-0034 D34.5 #1 (consult Agent.active via repo, not from event payload).
> 3. The catalog row's `active` field is computed as `agent.active && agent.archived_at.is_none()` — verifying D34.5 #2 + #3 (archive wins ties).
> 4. The body does NOT call `repo.set_agent_active` or `repo.set_agent_archived_at` — verifying D34.5 #4 (read-only on lifecycle).
> 5. The body calls `record_system_agent_fire(repo, org, catalog_system_agent_id, ...)` after the upsert succeeds. Verifies D6.1 second call site shipped.
> 6. `CatalogAuditMode` enum exists with `Silent` (default) + `Debug` variants. `AgentCatalogListener` constructor takes the mode as a parameter. Body emits audit events only when `audit_mode == Debug`.
> 7. `config/default.toml` contains `[listeners.catalog] audit_mode = "silent"` (default).
> 8. New file `modules/crates/server/tests/acceptance_system_flows_s03.rs` exists with 2 scenarios (AgentCreated upserts row; archive flips active).
> 9. `cargo test --workspace -- --test-threads=1` passes ~1013 tests (≥ 997 baseline + 16 new). Run with `-j 4`.
> 10. ADR-0035 Accepted at `docs/specs/v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md` with 4 sub-decisions (D35.1–D35.4).
> 11. D6.1 drift Status = `in-chunk-plan` (NOT `remediated` — CH-22 is partial closure, awaiting CH-21); lifecycle entry `2026-04-27 — partial-closure — CH-22 chunk-seal; second call site shipped` present.
> 12. `_concept-audit-matrix.md` rows for `system-agents.md §"Agent Catalog Agent"` 3 sub-claims (reactive updates + lifecycle events + queryable API) flipped to `honored`.
> 13. `system-agents.md` `Last verified` header bumped to 2026-04-27 with CH-22 amendment note.
> 14. §3.B K8s-readiness 7-axis table in the chunk plan concludes K8s-neutral; `grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` returns 8 (unchanged from CH-01).
> 15. `bash scripts/check-phi-core-reuse.sh` returns exit 0; `grep -rn '^pub struct AgentCatalogEntry\b' modules/crates/ | grep -v 'composites_m5.rs'` returns 0 hits.
> 16. The 14 unit tests in `listeners.rs::tests` (per §7 P2) all run + pass; the renamed `agent_catalog_listener_is_a_noop_at_p3` test was deleted (the stub-noop semantics no longer hold).
>
> Report each as PASS/FAIL with 1-line evidence. ≤ 700 words. Read-only.

**Seal-blocking rule.** Audit must report PASS on every check, OR any FAIL must be either (a) fixed in-chunk before seal, (b) reframed via user-approved ADR, or (c) converted to a new drift file with explicit future-chunk assignment before seal.

---

## §12 — Verification section

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health (cap workers per user feedback 2026-04-27)
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: 997 (CH-01 baseline) + ~16 new tests ≈ 1013

# 3. CH-22-specific positive greps
grep -c "fn on_event" modules/crates/domain/src/events/listeners.rs                          # ≥ 5 (3 templates + 2 stubs body-shipped)
grep -n "upsert_agent_catalog_entry" modules/crates/domain/src/events/listeners.rs           # ≥ 1
grep -n "record_system_agent_fire" modules/crates/domain/src/events/listeners.rs             # ≥ 2 (helper + AgentCatalogListener call site)
grep -n "CatalogAuditMode" modules/crates/server/src/config.rs                               # ≥ 1
grep -n "audit_mode" config/default.toml                                                     # ≥ 1
grep -n "agent_catalog_refreshed" modules/crates/domain/src/audit/events/m5/                 # ≥ 1
ls modules/crates/server/tests/acceptance_system_flows_s03.rs                                # 1

# 4. CH-22-specific negative greps (must be 0)
grep -rn "^pub struct AgentCatalogEntry\b" modules/crates/ | grep -v "composites_m5.rs"      # 0
grep -rn "^pub struct SystemAgentRuntimeStatus\b" modules/crates/ | grep -v "composites_m5.rs"  # 0

# 5. Acceptance test
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s03        # 2 pass

# 6. Drift status (D6.1 partial closure)
grep -c '^- \*\*Status\*\*: `in-chunk-plan`' docs/specs/v0/implementation/m5_1/drifts/D6.1.md   # 1
grep -c "partial-closure" docs/specs/v0/implementation/m5_1/drifts/D6.1.md                      # 1

# 7. ADR status
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0035-agent-catalog-listener-audit-mode.md  # 1

# 8. Concept-audit matrix
grep -c "Agent Catalog Agent" docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md  # ≥ 3 rows touched

# 9. K8s readiness (no new ledger entries)
grep -c '^### CHK8S-D-' docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md  # 8 (unchanged)
```

---

## What this plan does NOT do

- **No first call site for D6.1.** CH-21 closes the first call site (memory-extraction listener body). CH-22 ships the second; the drift transitions to `remediated` only when CH-21 also lands.
- **No HTTP catalog endpoint.** `GET /api/v0/orgs/:org/agents/catalog` is not in M5 scope; the catalog is populated by CH-22 + queried programmatically via `repo.list_agent_catalog_entries_in_org` from internal callers (M6 a05 grants-view, page 07 dashboard, etc.).
- **No `query_agents` queryable API.** The concept doc's `query_agents(filters)` tool is M6 scope (a05); CH-22 only populates the underlying rows.
- **No new migration.** `agent_catalog_entry` + `system_agent_runtime_status` schemas already shipped at migration 0005; CH-22 is the first producer of those rows but adds no schema changes.
- **No Template A/C/D listener changes.** The 3 sibling listeners are the canonical reference style for CH-22's body; their code is unchanged.
- **No `is_paused` field on the runtime-status struct.** Per ADR-0034 D34.5, `is_paused` is COMPUTED at read time by the catalog consumer (page 13 listing endpoint or the catalog query API at M6), not persisted. CH-22 only mutates the durable columns.
- **No subscribe-time filter changes.** The listener still subscribes to the same 8 variants + silently ignores `SessionAborted` (P3 stub behavior preserved).

---

## Notes on M5.1/P3 Q&A binding

- **Q1** (storage-backend) — untouched; CH-03 owns.
- **Q2** (selector PEG split) — untouched; CH-06 owns.
- **Q3** (consent triad) — untouched; CH-09/10/11 own.
- **Q4** (chunk ordering) — user-selected CH-22 over CH-21; CH-21 dependency classified as soft per Phase 1 investigation.
- **Q5** (M5 scope) — CH-22 closes one HIGH drift partially (D6.1 second call site); CH-21 closes it fully. Both must land before M5 tag.
- **Q6** (ADR numbering at draft time) — ADR-0035 claimed via `ls … | grep -oE "^[0-9]{4}" | sort -u | tail -5` pattern; honored.
- **Q7** (uniform ExitPlanMode ritual) — this plan is being approved via ExitPlanMode.
- **Q8** (K8s readiness rule, codified by CH-01) — §3.B populated; CH-22 declared K8s-neutral.
