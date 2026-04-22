# Plan: M2 — Platform Setup (admin pages 02–05)

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section

## Context  `[STATUS: n/a]`

M1 (Permission Check spine) shipped at 100 % confidence: 36-method Repository, pure Permission Check engine with 8-step pipeline, Auth Request 9-state machine, System Bootstrap flow with atomic claim, session-cookie JWT, at-rest encryption envelope, audit-event schema with hash-chain, 299 Rust + 14 Web tests green.

**M2 is the first milestone where handlers actually call the Permission Check engine** (bootstrap bypassed it) and emit audit events through a real `AuditEmitter`. It also introduces the "Template E Auth Request, auto-approved by the self-interested platform admin" pattern that every non-bootstrap write uses. Admin pages 02 (Model Providers), 03 (MCP Servers), 04 (Credentials Vault), 05 (Platform Defaults) all ship behind the existing session cookie.

The build-plan M2 entry is ~5 lines ([build plan §M2](../../projects/phi/phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md)); this plan is the fully-resolved version. Every contract is pinned up-front, the first non-bootstrap code paths for Permission Check + audit-emit land as reusable infrastructure (not per-page reinvention), docs co-land per phase, and a per-page vertical-slice shape keeps each commit reviewable.

**What M1 taught us (applied preventively to M2):**

1. Post-milestone audits catch every numerical drift eventually. M1 remediated 33→35 method count, 5→6 session test count, testing-posture table arithmetic, "11 files" → "10 files". **Prevention for M2**: the commitment ledger (Part 2) carries explicit numeric targets; the testing-posture table is authoritative from P1 (not retrofitted at P9); re-audits run at P3 close and P-final.
2. Shared utilities pay for themselves after the second user. M1 didn't need `handler_support`; M2 has ≥8 new handlers that would each reimplement Permission Check invocation + HTTP-code mapping + audit emission. **Prevention**: P3 ships `handler_support` as the first-class module before P4 begins.
3. Doc-links CI tolerates `[PLANNED M1/Pn]` placeholders; bulk path-depth fixes are cheap if done once at P1 and not retroactively. **Prevention**: seed `implementation/m2/` tree at P1 with correct `../../../../../../modules/` depth from the start.
4. Per-phase test-count claims need verification against `cargo test` output, not arithmetic. **Prevention**: Part 5 fixes M1's convention (cargo-level counts in rows, "gains" column for user-meaningful figures like trybuild fixture count).

**Archive location for this plan:** `phi/docs/specs/plan/build/<8-hex>-m2-platform-setup.md` (first execution step is to copy this plan verbatim to that path, matching the M1 archiving convention).

---

## Part 1 — Pre-implementation gap audit  `[STATUS: ⏳ pending]`

Cross-check of admin pages 02–05 requirements, concept docs, and current M1 code. Findings:

| # | Finding | Source | Fix |
|---|---|---|---|
| G1 | **Template E is not yet modelled.** `AuthRequest.provenance_template: Option<TemplateId>` exists but no `TemplateKind` enum, no helper mints a pre-Approved Template-E request. All four pages' writes are Template E. | `model/nodes.rs:288`; `requirements/admin/{02,03,04,05}` §W | Add `TemplateKind::{SystemBootstrap, A, B, C, D, E, F}` (minimum: `SystemBootstrap + E` for M2). Add pure `domain::templates::e::build_auto_approved_request(requestor, resource, scope, at) -> AuthRequest` that returns an `AuthRequest` already in `Approved` state with the requestor filling its single approver slot — mirrors `bootstrap::claim`'s construct-pre-approved pattern. |
| G2 | **Cascading revocation on MCP tenant narrowing has no Repository method.** `Repository::update_auth_request` is a full-replace only; `requirements/admin/03` W3 requires forward-only cascade to every tenant grant whose `org_id` was dropped. | `repository.rs:163`; `requirements/admin/03§W3` | Add `Repository::narrow_mcp_tenants(mcp_id, new_allowed) -> Result<Vec<(OrgId, AuthRequestId, Vec<GrantId>)>>` that performs index lookup + batch revoke inside one SurrealQL transaction. Handler emits one `McpServerTenantAccessRevoked` + per-AR `auth_request.revoked` events. |
| G3 | **Reveal-secret `purpose: reveal` constraint is not expressible.** `step_4_constraints` only checks constraint-context *key presence*, not *value match*. | `permissions/engine.rs:370-391`; `requirements/admin/04§7` | Widen `Manifest` with `constraint_requirements: HashMap<String, serde_json::Value>`; widen step-4 to "**key present AND value equals requirement**". Literal-match is enough for M2; full pattern lattice lands in M3. |
| G4 | **Platform defaults need "snapshot at creation time" companion.** M1 has no per-org config row. A bare singleton would make the "non-retroactive" contract pseudo. | `requirements/admin/05§W3` | Add `PlatformDefaults` singleton composite + `OrganizationDefaults` (a `control_plane_object` child, consumed by M3's org-creation wizard). M2 ships only the singleton + the invariant proptest; the consumer wires in M3. |
| G5 | **`*HealthDegraded` audit events have no background probe.** Pages 02+03 emit them; no M1 infra exists for scheduled probes. | `requirements/admin/02§N3, 03§N4` | Ship event *shape* only for M2; real probe infra (tokio interval) is M7b. A test-only trigger endpoint `POST /api/v0/platform/{model-providers,mcp-servers}/{id}/_probe_health` gated behind `#[cfg(feature = "dev_probe")]` exercises the event shape in tests. |
| G6 | **No `AuditEmitter` impl exists.** M1 defined the trait; bootstrap bypassed it (built events inline and stuffed them through `apply_bootstrap_claim`). M2 has ≥14 distinct event types across 4 pages. | `audit.rs:114`; `server/src/bootstrap/claim.rs` | Ship `store::SurrealAuditEmitter` in P3 — synchronous: look up `last_event_hash_for_org`, compute `hash_event`, write via `write_audit_event`. Instance lives on `AppState`. Shadow NDJSON deferred to M7b. |
| G7 | **No handler helper for `AuthenticatedSession`.** M1 ships `verify_from_cookies` but every M2 write handler would reimplement the cookie → agent_id → 401 dance. | `session.rs:159` | P3 ships `AuthenticatedSession(AgentId, SessionClaims)` axum `FromRequestParts` extractor with a single 401 `ApiError` path on missing/invalid cookie. |
| G8 | **No `check_permission` wrapper.** Engine returns `Decision`; handlers need a one-line "call engine + map to HTTP + record metric" helper. | `permissions/engine.rs:38` | P3 ships `handler_support::check_permission(state, ctx, manifest) -> Result<Vec<ResolvedReach>, ApiError>` with exhaustive `Decision → ApiError` mapping (D11). |
| G9 | **`secrets_vault` table is described but not shipped.** M1 wrote crypto helpers; the actual table + columns never landed in migration 0001. | `store/migrations/0001_initial.surql`; `crypto.rs:78-95` | `0002_platform_setup.surql` creates `secrets_vault SCHEMAFULL` with `id, slug, custodian_id, value_ciphertext_b64, nonce_b64, last_rotated_at, sensitive, created_at`. |
| G10 | **`tenants_allowed` has no domain type.** Modelled neither on any composite nor anywhere else. | `model/composites.rs:20-45` | New `TenantSet::{All, Only(Vec<OrgId>)}` enum. New `ModelRuntime` + `ExternalService` composite-instance structs in `model/composites_m2.rs`. |
| G11 | **Platform-defaults singleton has no natural id scheme.** s01 seeds `system:root`; nothing else seeds named control-plane rows. | `repository.rs:219-231`; `concepts/permissions/01` | Use a well-known deterministic UUID (SHA-256 of `"control_plane_object:platform-defaults"` → UUID bytes). Store in a dedicated `platform_defaults` table with a `UNIQUE INDEX` on a constant `singleton` column. |
| G12 | **`SecretId` vs `SecretRef` conflated.** Page 02 takes a `secret_ref` string; page 04 mints secrets with UUIDs. | `model/ids.rs` | Add `SecretId(Uuid)` newtype + keep `SecretRef(String)` as the human-readable id (e.g. `anthropic-api-key`). Store resolves `SecretRef → SecretId` at persist time for referential integrity. |
| G13 | **Audit `diff` is a free-form JSON blob.** 14+ event types, hand-authored diffs will drift between handler and test. | `audit.rs:53` | Each event type gets a dedicated `fn build_<event>_event(...) -> AuditEvent` constructor in `domain::audit::events::m2::{secrets,providers,mcp,defaults}`. Handlers call builders; builders own the diff shape. |
| G14 | **Template E auto-approve must not violate aggregation.** `transitions::transition_slot` may reject `Unset → Approved` depending on the legal-transition table. | `auth_requests/transitions.rs` | Don't transition — **construct** the `AuthRequest` struct already in `Approved` state (mirrors `bootstrap::claim`). Audit step at P2 verifies the aggregation function accepts a pre-approved slot without complaint. |
| G15 | **Acceptance harness has no pre-claimed variant.** Every M2 handler requires a session cookie from an already-claimed admin. | `server/tests/acceptance_common/mod.rs:146` | Add `spawn_claimed() -> (Acceptance, ClaimedAdmin { agent_id, session_cookie, authed_client })` that runs the mint+claim once and returns a reqwest client preconfigured with the cookie header. |
| G16 | **Prometheus recorder is process-global.** M1's `install_prometheus_layer` uses a `OnceLock`; only one acceptance test can pass `with_metrics: true` per process. | `acceptance_common/mod.rs:128-142` | Keep the `OnceLock` gate; designate one cross-page `acceptance_metrics.rs` as the only M2 `with_metrics: true` consumer. Revisit with `serial_test` crate if M3 needs more. |
| G17 | **Doc-links CI covers per-milestone trees.** M2 adds `implementation/m2/` + 4 new ADRs (0016–0019). | `.github/workflows/doc-links.yml` | Seed the `m2/` tree at P1 (not retroactively). Path depth is `../../../../../../modules/` from `architecture/` or `decisions/` or `operations/` or `user-guide/` (same as M1). |
| **G18** | **`phi-core` reuse is implicit, not explicit.** M2 surfaces overlap heavily with shipped phi-core APIs (Part 1.5 below). Without a hard reuse mandate we risk duplicating `ExecutionLimits`, `ModelConfig`, `McpClient`, `AgentProfile`, `parse_config_file`, etc. That's wasted code + drift over time. | `phi-core/src/{provider,mcp,context,agents,config,types}/*.rs` vs planned phi composites | **Reuse is a commitment, not a suggestion**: every M2 composite that overlaps with a phi-core type MUST either (a) use the phi-core type directly, or (b) wrap it as a field. New phi structs are permitted ONLY where phi-core has no counterpart (page 04 vault; Permission-Check constraint machinery; audit-event envelopes). See §1.5 for the per-page reuse map + D16 for the enforcement mechanism. |
| **G19** | **Instance-URI grants silently fail the Permission Check engine.** `resolve_grant` has three cases: (A) URI names a fundamental — fundamentals = {that one}, selector = `Any`; (B) URI names a composite — fundamentals = constituents, selector = `Any`, kind refinement added; (C) opaque instance URI (e.g. `secret:anthropic-api-key`, `provider:42`, `mcp:memory`) — fundamentals = **empty**, selector parsed from URI. Case-C grants can never be candidates at Step 3 (`covers(fundamental, _)` returns false on the empty set). Only `system:root` is special-cased. The M2 plan's **D11** ("reveal is a Permission-Check invocation; unlocks M3 delegated reveal without changing handlers") silently depends on per-instance grants working — they currently don't. **Surfaced during P4 implementation** (the reveal path): the P4 shipping workaround issues a class-wide `[read]` grant on the `secret_credential` fundamental (Case A) scoped only by catalogue + constraint — adequate for M2's single-admin model but blocks M3's delegated-custody story and would cost a data migration later. | `domain/src/permissions/expansion.rs:116-158` (resolve_grant); `server/src/platform/secrets/add.rs` (P4 workaround) | **P4.5 detour** (before P5): add an optional `fundamentals: Vec<Fundamental>` field to `Grant` (`#[serde(default)]` for DB back-compat); extend `resolve_grant` with a 4th case that prefers the persisted value when non-empty; update `add_secret` (P4) to issue `secret:<slug>` grants with `fundamentals = [SecretCredential]`. Existing grant-constructor callsites get the default empty vec. Adds one proptest covering the instance-URI match path. See §P4.5 for the full scope. |

### Confidence target: **≥ 97 % at first review**, ≥ 99 % after post-P9 remediation.

Lower than M1's 99 % first-review target because G1/G3/G14 introduce new contract surfaces that the pre-audit can't fully de-risk without prototype. Mitigated by the P3 close re-audit (shared infrastructure must be solid before per-page work begins).

---

## Part 1.5 — phi-core reuse map  `[STATUS: ⏳ pending]`

**Principle** (per G18 + D16): phi is a consumer of phi-core. Every M2 surface that overlaps with an existing phi-core type uses the phi-core type directly or wraps it — we do **not** re-implement what phi-core already ships.

Legend: ✅ **direct reuse** (use as-is); 🔌 **wrap** (phi struct holds a phi-core type as a field); 🚫 **no phi-core counterpart** (phi builds from scratch).

| Surface | phi-core type / API (absolute path) | M2 use site | Mode |
|---|---|---|---|
| **Page 02 — Model Providers** | | | |
| Provider binding | `phi_core::provider::model::ModelConfig` (`/root/projects/phi/phi-core/src/provider/model.rs`) | `ModelRuntime.config: ModelConfig` on the phi composite | 🔌 |
| Supported providers enum | `phi_core::provider::model::ApiProtocol` | `ProviderKind` in phi = type alias to `ApiProtocol` (single source of truth) | ✅ |
| Provider factory | `phi_core::provider::registry::ProviderRegistry::default()` | Enumerate in the web UI dropdown; resolve for health-probe | ✅ |
| Cache / thinking | `phi_core::types::usage::{CacheConfig, ThinkingLevel}` | Fields on `ModelRuntime`; exposed in page 02 UI | ✅ |
| Stream contract | `phi_core::provider::traits::{StreamProvider, StreamConfig, StreamEvent}` | Not exercised by M2 handlers (session launch is M5); reuse at that point | ✅ (deferred) |
| **Page 03 — MCP Servers** | | | |
| Client | `phi_core::mcp::client::McpClient` + `connect_stdio` / `connect_http` | `ExternalService` composite stores config; health-probe constructs a `McpClient` | ✅ |
| Discovery | `phi_core::mcp::types::{McpToolInfo, ServerInfo}` + `McpClient::list_tools()` | Populate the "registered MCP servers" table with live tool counts | ✅ |
| Tool adapter | `phi_core::mcp::tool_adapter::McpToolAdapter` | Not exercised by M2 admin pages (tool invocation is M5); reuse then | ✅ (deferred) |
| Health probe | (none in phi-core) | Build a thin wrapper around `list_tools()` with 2 s timeout + 3 retries | 🚫 |
| **Page 04 — Credentials Vault** | | | |
| Env var substitution | `phi_core::config::parser::substitute_env_vars()` (if public; else pattern) | Only tangentially — vault resolves `secret_ref → plaintext`, handler pastes plaintext into `ModelConfig.api_key` before a downstream call | 🚫 (logic is vault-native) |
| Material storage | (none in phi-core) | phi's `SecretCredential` composite + M1's `seal`/`unseal` | 🚫 |
| **Page 05 — Platform Defaults** | | | |
| Execution limits | `phi_core::context::execution::ExecutionLimits` | `PlatformDefaults.execution_limits: ExecutionLimits` | 🔌 |
| Agent profile | `phi_core::agents::profile::AgentProfile` | `PlatformDefaults.default_agent_profile: AgentProfile` | 🔌 |
| Context / compaction | `phi_core::context::{ContextConfig, CompactionStrategy}` | Fields on `PlatformDefaults` | 🔌 |
| Retry tuning | `phi_core::provider::retry::RetryConfig` | Field on `PlatformDefaults` | 🔌 |
| Layered config parser | `phi_core::config::parser::{parse_config, parse_config_file, parse_config_auto}` + `phi_core::config::schema::AgentConfig` | Baby-phi's `ServerConfig::load()` already layered; platform-defaults editor uses these parsers for YAML/TOML import/export | ✅ |
| **M7 / s04 audit adjacency (not M2, but adjacent)** | | | |
| Event stream | `phi_core::types::event::AgentEvent` (all variants) | Future M7 pipeline subscribes via `Agent::prompt_messages_with_sender(messages, tx)` | ✅ (deferred) |
| Session / Turn / LoopRecord | `phi_core::session::model::{Session, LoopRecord, Turn, LoopStatus}` | Persisted via `SessionRecorder` (M5+) | ✅ (deferred) |

**What phi-core does NOT provide and M2 must build**:
- Credentials vault (page 04 entire surface).
- Tool-authority manifests / Permission Check constraint lattice (phi's domain entirely).
- MCP health-probe (thin wrapper as noted).
- `TenantSet` / tenant-narrowing semantics (platform-level concept not in phi-core's scope).
- `PlatformDefaults` container struct itself (it *composes* phi-core types; the container is phi-only).
- Template E auto-approval (permission-system concept not in phi-core).

**Concrete enforcement** (per D16):
1. Where any M2 PR introduces a new `struct` or `enum` whose field set matches a phi-core type, reviewers reject it and require the phi-core import.
2. P3 close audit spot-checks every M2 composite for direct phi-core imports.
3. `scripts/check-phi-core-reuse.sh` (new — cheap grep-based lint; see Part 7) flags tell-tale duplications:
   - `grep -rn "struct ExecutionLimits" modules/crates/` must return zero hits (only phi-core defines it).
   - `grep -rn "struct ModelConfig" modules/crates/` must return zero hits.
   - `grep -rn "pub struct AgentProfile" modules/crates/` must return zero hits (only phi-core defines this name — phi's governance analogue is renamed per §1.6).
   - `grep -rn "struct McpClient" modules/crates/` must return zero hits.
   - Runs in CI as an advisory-then-hard gate after the P3 close audit locks the policy.

---

## Part 1.6 — M1 residual drift (P0 remediation before M2 opens)  `[STATUS: ⏳ pending]`

Two deep-sweep phi-core-reuse audits of M1's shipped code (file-by-file sweep + Cargo dep/utility sweep) found **two material items**, three doc-only items, and confirmed everything else clean. Fixing them before P1 begins avoids compounding debt:

| # | Finding | Severity | Fix (P0) |
|---|---|---|---|
| **R1** | **`AgentProfile` name collision.** `domain::model::nodes::AgentProfile` (phi) has fields `{id, agent_id, display_name, parallelize, created_at}` — a platform-governance node capturing concurrent-session caps. `phi_core::AgentProfile` has `{profile_id, name, description, system_prompt, thinking_level, temperature, max_tokens, config_id, skills, workspace}` — an execution blueprint for the agent loop. Same name, orthogonal concerns; M4's agent-provisioning will want **both** (governance + execution blueprint). Leaving the collision lets M4 ship with an ambiguous import, at which point renaming hits the whole Repository trait + wire types + docs. | 🚨 Material | **P0.R1** — rename phi's struct to `AgentGovernanceProfile`. Touches: `domain/src/model/nodes.rs`, `domain/src/repository.rs` (`create_agent_profile` method keeps its name, arg type renamed), `in_memory.rs`, `store/src/repo_impl.rs`, `migrations/0001_initial.surql` (squashed since M1 is pre-release), tests + M1 docs. M4 then adds `phi_core::AgentProfile` as a wrapped field. |
| **R2** | **`thiserror` version drift.** phi workspace uses `thiserror = "1"`; phi-core uses `thiserror = "2"`. thiserror 1/2 error types have binary-incompatible layouts — any place phi wraps a phi-core error with `#[from]` silently fails at runtime with "implementations not found". The collision doesn't bite M1 because no error-type wrapping across the phi-core boundary is exercised yet; M2's `AuditEmitter` + M4's agent-loop integration will trip it. | 🚨 Material | **P0.R2** — bump `thiserror` to `"2"` in `/root/projects/phi/phi/Cargo.toml:32` (workspace); recompile; verify `cargo test --workspace` stays green. 15-minute fix. |
| R3 | **`AuditEvent` vs `AgentEvent` are distinct, not duplicates.** `domain::audit::AuditEvent` (platform governance write log with hash chain + class tier) is orthogonal to `phi_core::types::event::AgentEvent` (agent-loop turn telemetry). Surfaces look similar from the outside; they're genuinely different concerns. | ⚠️ Cosmetic | **P0.R3** — add a distinction paragraph to `docs/specs/v0/implementation/m1/architecture/audit-events.md`. |
| R4 | **`SessionClaims` (HTTP session) vs `phi_core::Session` (persistent execution trace).** M1's HS256 cookie + `SessionClaims` carries admin identity across requests; phi-core's `Session` persists agent-loop turn history. Different layers; M5+ will have both. | ⚠️ Cosmetic | **P0.R3** (same doc as R3) — add a paragraph distinguishing HTTP session vs agent-loop session to `docs/specs/v0/implementation/m1/architecture/server-topology.md` (§Session cookie). |
| R5 | **`ToolDefinition` (phi permissions metadata) vs `phi_core::AgentTool` (runtime tool).** Both are "tool" surfaces but at different layers — phi's is a policy/audit node; phi-core's is a runtime `execute(params)` trait. M5+ adapter will bridge them. | ⚠️ Cosmetic | **P0.R3** (same doc pass) — add a one-line note to `docs/specs/v0/implementation/m1/architecture/graph-model.md` distinguishing the two when `ToolDefinition` is referenced. |
| R6 | **`ServerConfig::load` (layered TOML + `PHI__KEY` env override) vs `phi_core::parse_config_file` (TOML/YAML/JSON + `${VAR}` interpolation).** Both are "config parsers" but for different shapes — `ServerConfig` is infrastructure (host/port/data-dir/session-secret); `AgentConfig` is agent blueprint. Earlier exploration briefly suggested consolidating; closer look shows they're legitimately orthogonal. M2 page 05 **does** consume `parse_config` for agent-defaults import/export — the correct reuse boundary. | ✅ Clean (but document) | **P0.R3** (same doc pass) — add a paragraph to `docs/specs/v0/implementation/m1/architecture/overview.md` §Configuration explaining the `ServerConfig` / `AgentConfig` parsing separation so M2+ authors don't mis-merge them. |

**Confirmed clean** (no P0 action): Permission Check engine, Auth Request state machine, Repository trait, store crypto (`seal`/`unseal`), forward-only migrations, session cookie machinery, bootstrap credential flow, every node type except R1, all utility functions (UUID, time, base64, JSON, retry, hash-chain). CLI `agent demo` is a textbook phi-core consumer (imports `phi_core::{agents_from_config, parse_config_file, save_session, AgentEvent, SessionRecorder, StreamDelta}` directly). Feature flags on shared crates (`uuid`, `chrono`) and version pins are consistent except R2.

**P0 execution**: one tagged commit labelled `m2-p0-phi-core-reuse-residuals` covering R1 (rename) + R2 (thiserror bump) + R3 (doc pass for R3/R4/R5/R6). Final `cargo test --workspace` green. No new tests; existing tests continue to pass. Tracked as **C20** in the commitment ledger + P0 in the execution order.

---

## Part 2 — Commitment ledger  `[STATUS: ⏳ pending]`

| # | Commitment | M2 deliverable | Phase | Verification |
|---|---|---|---|---|
| C1 | New node/composite types + ID newtypes | `SecretId`, `ModelProviderId`, `McpServerId`; `ModelRuntime`, `ExternalService`, `SecretCredential`, `PlatformDefaults` composite-instance structs; `TenantSet`, `ProviderKind`, `ExternalServiceKind`, `RuntimeStatus`, `TemplateKind` enums | P1 | `domain/tests/m2_model_counts.rs` asserts added types serde round-trip + `TemplateKind::ALL.len()` ≥ 7 |
| C2 | Schema migration applies + forward-only | `store/migrations/0002_platform_setup.surql` with `secrets_vault`, `model_providers`, `mcp_servers`, `platform_defaults` tables + indexes; `_migrations` row recorded | P1 | `store/tests/migrations_0002_test.rs` — fresh DB applies; noop on already-applied DB; broken migration refuses to serve |
| C3 | Repository trait expanded + both impls green | ~17 new methods (secrets ×5, providers ×3, mcp ×4, defaults ×2, bulk-revoke ×2, catalogue ×1) | P2 | `store/tests/repo_m2_surface_test.rs` + `domain/tests/in_memory_m2_test.rs` exercise every method; both impls identical |
| C4 | Template E pure-fn helper | `domain::templates::e::build_auto_approved_request` returns Approved `AuthRequest` | P2 | `domain/tests/template_e_props.rs` — ≥ 3 proptest invariants over random (scope, resource, actor) |
| C5 | `handler_support` module | `AuthenticatedSession` extractor, `check_permission`, `emit_audit`, shared `ApiError`, exhaustive `Decision → HTTP` mapping | P3 | `server/tests/handler_support_test.rs` — 401 unauth + 500 internal + one test per `FailedStep` variant (8 total) |
| C6 | `SurrealAuditEmitter` + hash-chain | `store::SurrealAuditEmitter` implementing `AuditEmitter` trait; instance on `AppState` | P3 | `store/tests/audit_emitter_chain_test.rs` — 3-event sequence with chain continuity |
| C7 | Acceptance-harness claimed-admin fixture | `spawn_claimed()` + `ClaimedAdmin` struct + pre-cookied reqwest client | P3 | `server/tests/spawn_claimed_smoke.rs` — harness compiles + returns usable cookie |
| C8 | Page 04 credentials vault | `GET / POST /secrets`, `POST /{id}/rotate`, `POST /{id}/reveal`, `POST /{id}/reassign-custody` + at-rest encryption round-trip; 7 audit event types | P4 | `server/tests/platform_secrets_test.rs` + `domain/tests/vault_roundtrip_props.rs` |
| C9 | Page 02 model providers | `GET / POST /model-providers`, `POST /{id}/archive` + Template E auto-approve + catalogue seed | P5 | `server/tests/platform_model_providers_test.rs` — register emits AR + Grant + catalogue seed + audit event in one tx |
| C10 | Page 03 MCP servers (incl. cascading revocation) | `GET / POST / PATCH /mcp-servers`, `POST /{id}/archive` + cascade | P6 | `server/tests/platform_mcp_servers_test.rs` + `domain/tests/mcp_cascade_props.rs` — monotonic grant-count reduction |
| C11 | Page 05 platform defaults | `GET / PUT /platform/defaults` + non-retroactive inheritance | P7 | `server/tests/platform_defaults_test.rs` + `domain/tests/platform_defaults_non_retroactive_props.rs` |
| C12 | Prometheus `phi_permission_check_duration_seconds{result, failed_step}` wired through real handlers | Histogram records under real HTTP traffic | P3 (wiring), P4–P7 (emission) | `server/tests/acceptance_metrics.rs` — scrape `/metrics` after a 403 and assert labels |
| C13 | Audit events for all M2 writes (correct class + chain) | 14+ event types with Alerted default for sensitive surfaces; `prev_event_hash` populated | P3 (emitter), P4–P7 (usage) | `server/tests/audit_chain_m2_test.rs` — 4-write sequence with hash-chain continuity |
| C14 | CLI subcommands for all 4 pages + `login` | `phi {login, secret, model-provider, mcp-server, platform-defaults}` | P4–P7 (per page), P8 (polish) | `cli/tests/help_snapshot.rs` (insta snapshots for every subcommand's `--help`) + `cli/tests/platform_cli_test.rs` |
| C15 | Web UI admin layout shell + 4 pages | `app/(admin)/layout.tsx` with auth gate + sidebar nav; one SSR-probe page + Server Action + Client Component per page; shared `<ApiErrorAlert />` | P4–P7 (per page), P1 (shell) | `modules/web/__tests__/admin_layout.test.tsx` + per-page smoke tests |
| C16 | Operations docs + troubleshooting updated | 4 new ops runbooks + M2 additions to schema-migrations + at-rest-encryption + audit-log-retention + troubleshooting with M2 stable codes | P4–P7 (per page), P8 (seal) | `check-doc-links.sh` + new `check-ops-doc-headers.sh` both CI-green |
| C17 | doc-links CI green throughout | `implementation/m2/` tree seeded P1; fleshed per phase | P1–P8 | `.github/workflows/doc-links.yml` passes every PR |
| C18 | `check_permission` mapping exhaustiveness | Every `FailedStep` variant maps to a distinct `ApiError.code`; compile-time exhaustive `match` | P3 | `server/tests/permission_check_mapping_test.rs` — runtime check over `FailedStep::ALL` |
| **C19** | **phi-core reuse mandate enforced** (G18 / D16) | Every M2 composite overlapping a phi-core type either imports or wraps it — no re-implementations; `scripts/check-phi-core-reuse.sh` green in CI | P1 (script lands), P3 close (audit), P-final (re-verify) | `scripts/check-phi-core-reuse.sh` zero-hit on forbidden duplications; P3 audit + P-final audit each spot-check `Cargo.toml` for `phi-core = { workspace = true }` and the M2 composite struct fields for phi-core type references |
| **C20** | **M1 residual drift resolved before P1** (§1.6) | Baby-phi's `AgentProfile` renamed to `AgentGovernanceProfile`; audit-vs-agent-event distinction documented in `m1/architecture/audit-events.md` | P0 | `cargo test --workspace` green after rename; `grep -n "pub struct AgentProfile" modules/crates/domain/src/` returns zero hits; `check-phi-core-reuse.sh` (once landed in P1) stays clean |
| **C21** | **Instance-URI grants resolve in the engine** (G19 / D17). Retrospective commitment added after P4 surfaced the gap. | `Grant.fundamentals: Vec<Fundamental>` field with `#[serde(default)]`; 4th case in `domain::permissions::expansion::resolve_grant` that prefers the persisted vec; `add_secret` reissues its `[read]` grant on `secret:<slug>` (instance URI) with `fundamentals = [SecretCredential]`; every existing grant constructor sweep-updated (bootstrap, tests). | P4.5 (between P4 and P5) | New proptest `domain/tests/instance_uri_grant_match_props.rs` — grants with explicit fundamentals + instance-URI selector cover their matching call target at Step 3; grants without the field fall back to URI-derived semantics (regression guard on M1 behaviour). `acceptance_secrets.rs` revise-check: add emits a `secret:<slug>` grant (not a `secret_credential` class grant); reveal still green. |

Target: **21 commitments closed** at P-final. Plus re-audit at P3 close + P-final (C17 + C19 + C20 drift guards + C21 instance-URI regression guard).

---

## Part 3 — Decisions made up-front  `[STATUS: ⏳ pending]`

To avoid mid-build thrashing. Push back in review if any are wrong.

| # | Decision | Rationale |
|---|---|---|
| D1 | `AuthenticatedSession` is an **axum extractor**, not a per-handler `verify_from_cookies` call. | Matches `State<AppState>` extractor symmetry; impossible to forget; failure → fixed 401 `ApiError` shape. |
| D2 | Template E auto-approve logic lives in the **domain crate** (`domain::templates::e::build_auto_approved_request`). | Pure function, no I/O, proptest-friendly. Handlers call it; approval rule is not scattered across server code. |
| D3 | Vault encryption scope = per-entry seal-with-fresh-nonce, one master key. No per-entry DEK layer in M2. | Matches M1's shipping architecture; full KMS / key-rotation tooling is M7b. |
| D4 | Migration `0002_platform_setup.surql` is **ONE** migration, not four per-page. | Forward-only migrations reason better as coarse snapshots; four sub-migrations multiply transactional boundaries without rollback benefit. |
| D5 | Platform defaults storage = **dedicated `platform_defaults` table** with `UNIQUE INDEX` on a constant `singleton` column. | Enforces singleton invariant at the DB layer; a row on `control_plane_object` couldn't. We ALSO seed a catalogue entry so Permission-Check Step 0 resolves. |
| D6 | `AuditEmitter` impl is **synchronous** (inline hash-chain walk + primary write). | Linearizability with the audited write is what the hash-chain contract needs; async queue is M7b. |
| D7 | Cascading revocation is a **Repository method** (`narrow_mcp_tenants`) that returns affected grant+AR ids; handler emits audit events in a loop. | Keeps DB walk inside the txn boundary (no TOCTOU); audit events are domain-class concerns, not storage. |
| D8 | Acceptance harness gets `spawn_claimed()` + keeps `spawn(with_metrics)` as low-level primitive. | Delta-minimal: M1 tests keep working; M2 tests get a shorter path. |
| D9 | `ApiError` becomes `server::handler_support::errors::ApiError` (promoted from `handlers/bootstrap.rs`). | Central shape; pre-emptive M1-cleanup deferred to M2 opening so we don't do it twice. |
| D10 | `check_permission` HTTP mapping: `Catalogue→403 CATALOGUE_MISS`, `Expansion→400 MANIFEST_EMPTY`, `Resolution→403 NO_GRANTS_HELD`, `Ceiling→403 CEILING_EMPTIED`, `Match→403 NO_MATCHING_GRANT`, `Constraint→403 CONSTRAINT_VIOLATION`, `Scope→403 SCOPE_UNRESOLVABLE`, `Consent→202 AWAITING_CONSENT`. | 403 for access denials; 202 for consent-pending (UI needs a toast, not an error); 400 for malformed manifests (caller-side bug). |
| D11 | **"Reveal" is a Permission-Check invocation with `purpose=reveal`**, not a handler-bypass special case. | Engine stays the single source of permission truth (widens step-4 per G3). Unlocks M3's "delegated reveal grant" case without changing handlers. |
| D12 | Audit-event builder functions live in `domain::audit::events::m2` as **pure functions**. Handlers call them; the returned `AuditEvent` carries `prev_event_hash = None` (the emitter fills it). | Pins diff shapes in one place (G13); keeps handlers free of JSON literals; hermetic tests. |
| D13 | Per-page vertical slicing: **P4–P7 ship one page at a time — Rust handler + CLI subcommand + Web page + ops doc in one phase**. | Each phase is reviewable end-to-end; avoids M1's "all CLI at P7, all web at P8" batching, which is riskier when surfaces × pages = 4 × 3 = 12 things instead of 1 × 3 = 3. |
| D14 | CLI session persistence = `$XDG_CONFIG_HOME/phi/session` file with `0600` permissions. Keyring deferred to M3+ when OAuth lands. | Re-auth-each-time is untenable (bootstrap credential is single-use). File at mode 0600 is secure enough for M2's workstation-user model. |
| D15 | Admin web pages sit under `app/(admin)/` Route Group with a shared layout + `requireAdminSession()` HOF. | Route Group leaves `/` and `/bootstrap` outside the gate; HOF > Next middleware because middleware runs on Edge and would need a separate JWT path. |
| **D16** | **phi-core reuse is a first-class mandate, not a suggestion.** Every M2 composite that overlaps a phi-core type MUST import it or wrap it; new phi structs only where phi-core has no counterpart (see §1.5 reuse map). Enforced by a CI lint (`scripts/check-phi-core-reuse.sh`) + P3 close / P-final audits. | phi is a consumer of phi-core; duplicating `ExecutionLimits`, `ModelConfig`, `McpClient`, `AgentProfile`, `parse_config_file`, `AgentEvent`, `Session` etc. would create a two-source-of-truth problem that compounds per milestone. Phase-1 exploration confirmed 8 of 10 M2 overlap surfaces have direct phi-core counterparts. |
| **D17** | **Grant carries explicit `fundamentals: Vec<Fundamental>` alongside the resource URI** (new field, `#[serde(default)]`). The engine's `resolve_grant` prefers this value when non-empty; otherwise falls back to the existing URI-derived logic. Handlers that issue instance-URI grants (e.g. `secret:<slug>`) populate it; handlers issuing class-URI or `system:root` grants leave it empty to preserve M1 semantics. | Addresses G19. Instance-URI grants now work without a URI-scheme convention or a per-class parser. Storage stays forward-compatible (existing rows deserialize with an empty vec, which triggers the legacy URI-derivation path). Unlocks M3's per-instance delegated reveal / custody handoff **without** a data migration. The alternative — parsing a `<class>:<instance>` convention inside the engine — would proliferate special cases across every M2+ composite (`secret:`, `provider:`, `mcp:`, `platform-defaults:` …) and pollute the engine with naming trivia. |

---

## Part 4 — Implementation phases  `[STATUS: ⏳ pending]`

Nine phases — P0 (M1 residual cleanup) then P1–P8. Each closes with `cargo fmt/clippy/test` + `npm test/typecheck/lint/build` + `doc-links` all green, the commitment-ledger row(s) ticked, and the `m2/` docs tree incremented.

### P0 — M1 residual phi-core reuse cleanup (~1 day)

**Goal** (C20 + §1.6): close every M1 phi-core drift item surfaced by the two retrospective audits (R1–R6) before anything new lands on top. **Two material items** (rename + thiserror bump) + **one doc pass** covering four orthogonal-surface distinctions.

1. **R1 — Rename `AgentProfile` → `AgentGovernanceProfile`** in `domain/src/model/nodes.rs`.
2. **R1 cascade** through `domain/src/repository.rs` (method signature only — `create_agent_profile` method name kept, arg type renamed), `in_memory.rs`, `store/src/repo_impl.rs`, `store/migrations/0001_initial.surql` (**squash into 0001** since M1 is pre-release and there are no production DBs — cleaner than a rename migration), any `M1/architecture/*.md` and `storage-and-repository.md` that cite the old name, any test files.
3. **R2 — Bump `thiserror` to `"2"`** in `/root/projects/phi/phi/Cargo.toml:32` (workspace). This aligns phi with phi-core (which uses thiserror 2) and prevents the silent `#[from]` breakage that would surface the first time M2 wraps a phi-core error in a phi error variant.
4. **R3 doc pass** — one PR that adds four short distinction paragraphs across M1 docs:
   - `docs/specs/v0/implementation/m1/architecture/audit-events.md` — `AuditEvent` (governance write log) vs `phi_core::AgentEvent` (agent-loop telemetry).
   - `docs/specs/v0/implementation/m1/architecture/server-topology.md` §Session cookie — `SessionClaims` (HTTP session cookie) vs `phi_core::Session` (persistent execution trace).
   - `docs/specs/v0/implementation/m1/architecture/graph-model.md` — node `ToolDefinition` (permission metadata) vs `phi_core::AgentTool` (runtime tool).
   - `docs/specs/v0/implementation/m1/architecture/overview.md` §Configuration — `ServerConfig` (HTTP/storage layered TOML with `PHI__KEY` overrides) vs `phi_core::parse_config_file` / `AgentConfig` (agent-blueprint schema with `${VAR}` interpolation); they're orthogonal, and M2 page 05 consumes `parse_config` for agent-defaults import/export as the correct reuse boundary.
5. **Gates**: `cargo fmt/clippy/test --workspace` green; workspace total stays at 299+14 = 313.

**Files modified** (~13): `Cargo.toml` (workspace thiserror bump), `nodes.rs`, `repository.rs`, `in_memory.rs`, `repo_impl.rs`, `0001_initial.surql`, M1 docs (audit-events.md, server-topology.md, graph-model.md, overview.md, storage-and-repository.md), test files.

**Dependencies**: none (this phase precedes all M2 work).

### P1 — Foundation: node types + IDs + migration + docs tree + web shell (~2–3 days)

**Goal:** every M2 surface can start being built against stable types + a running migration + an admin-layout shell.

1. **IDs** in `domain/src/model/ids.rs`: add `SecretId`, `ModelProviderId`, `McpServerId` via existing `id_newtype!` macro.
2. **Composites** in new `domain/src/model/composites_m2.rs` — **every phi-core-overlapping field is a direct reuse or a wrap per §1.5**:
   - `ModelRuntime { config: phi_core::ModelConfig, secret_ref: SecretRef, tenants_allowed: TenantSet, status: RuntimeStatus, archived_at: Option<DateTime<Utc>> }` — `config` holds the full phi-core binding.
   - `ExternalService { endpoint: String, kind: ExternalServiceKind, secret_ref: Option<SecretRef>, tenants_allowed: TenantSet, status: RuntimeStatus, archived_at }` — pure phi struct; the live `McpClient` is instantiated on demand (not stored) from this config.
   - `SecretCredential { slug, custodian, last_rotated_at, sensitive; sealed-value is store-side only }` — no phi-core counterpart.
   - `PlatformDefaults { execution_limits: phi_core::ExecutionLimits, default_agent_profile: phi_core::AgentProfile, context_config: phi_core::ContextConfig, retry_config: phi_core::RetryConfig, default_retention_days, default_alert_channels, model_provider_defaults, updated_at, version }` — the top-level container is phi-only; every phi-core-overlapping field is the phi-core type exactly.
   - Enums: `TenantSet`, `ExternalServiceKind`, `RuntimeStatus` (phi-only). `ProviderKind = phi_core::ApiProtocol` (re-export / type alias — single source of truth).
3. **`TemplateKind`** on `Template` node in `model/nodes.rs`: enum with `SystemBootstrap + A..F`. Existing bootstrap template rows default to `SystemBootstrap`.
4. **Migration** `store/migrations/0002_platform_setup.surql`: `secrets_vault`, `model_providers`, `mcp_servers`, `platform_defaults` tables with the full column sets + unique index on `platform_defaults.singleton`.
5. **Docs tree seed** `docs/specs/v0/implementation/m2/{README,architecture,user-guide,operations,decisions}/`:
   - `README.md` with the 8-phase status table (all `[PLANNED M2/Pn]` at P1 close).
   - Empty placeholder `.md` files with `<!-- Last verified: YYYY-MM-DD by Claude Code -->` headers so doc-links CI passes from commit 1.
6. **Web admin shell**:
   - `modules/web/app/(admin)/layout.tsx` — Server Component; calls `requireAdminSession()`; redirects to `/bootstrap` if unauthenticated; renders `<AdminSidebar />`.
   - `modules/web/app/(admin)/components/{AdminSidebar,AdminHeader}.tsx`.
   - `modules/web/app/components/{ApiErrorAlert,ConfirmDialog,DataTable}.tsx` — shared primitives used by every M2 page.
   - `modules/web/lib/session.ts` extended with `requireAdminSession()` HOF.
   - `modules/web/lib/api/{errors,forward-cookie}.ts` — stable-code table + cookie-forwarding helper.
7. **CLI scaffolding** for later phases:
   - `modules/crates/cli/src/session_store.rs` — XDG config file at `$XDG_CONFIG_HOME/phi/session` with 0600 perms (D14).
   - `modules/crates/cli/src/exit.rs` — named exit-code constants (extends M1's 0/1/2/3 ladder with 4 "precondition failed" + 5 "cascade-aborted").
   - `modules/crates/cli/src/commands/login.rs` — `phi login --credential <bphi-bootstrap-…>` (M2-only re-auth stopgap; OAuth is M3).
8. **phi-core reuse lint** `scripts/check-phi-core-reuse.sh` (per D16 + C19): greps every `.rs` under `modules/crates/` for forbidden re-declarations of phi-core types (`struct ExecutionLimits`, `struct ModelConfig`, `struct AgentProfile`, `struct McpClient`, `struct RetryConfig`, `struct ContextConfig`, `struct AgentEvent`, `struct Session`, `struct LoopRecord`). Zero hits required. Lands in P1 as advisory; flipped to hard gate after the P3 close audit.

**Files created** (~20): composites_m2.rs, migration, `m2/` tree seeds (~10 md files + README), admin shell (~5 tsx files), lib helpers (~3 ts files), CLI modules (~3 rs files).

**Files modified**: `domain/src/model/{ids,nodes,mod}.rs`; `store/migrations/mod.rs` (migration list); `modules/crates/cli/src/main.rs` (add `login` subcommand).

**Tests added (~15)**: `domain/tests/m2_model_counts.rs`, `store/tests/migrations_0002_test.rs` (apply/noop/broken), composite serde round-trip unit tests, `TemplateKind::ALL` count test; `modules/web/__tests__/admin_layout.test.tsx` (shell renders, redirect works).

### P2 — Repository expansion + Template E pure fn + constraint value-match (~2–3 days)

**Goal:** the persistence + engine surfaces M2 handlers need are green before any handler lands.

1. **Template E**: `domain/src/templates/{mod,e}.rs` with `build_auto_approved_request(requestor, resource, scope, at) -> AuthRequest`. Uses construct-pre-approved pattern (G14); asserts aggregate state is `Approved` in debug.
2. **Constraint value-match** (G3): widen `Manifest` with `constraint_requirements: HashMap<String, serde_json::Value>`; widen `step_4_constraints` in `engine.rs` to verify both key presence AND value equality when requirement present.
3. **Repository trait extension** in `domain/src/repository.rs`: ~17 new methods grouped by concern.
   - Secrets (5): `put_secret`, `get_secret_by_slug`, `list_secrets`, `rotate_secret`, `reassign_secret_custodian`.
   - Model providers (3): `put_model_provider`, `list_model_providers`, `archive_model_provider`.
   - MCP servers (4): `put_mcp_server`, `list_mcp_servers`, `patch_mcp_tenants`, `archive_mcp_server`.
   - Platform defaults (2): `get_platform_defaults`, `put_platform_defaults`.
   - Cascade (2): `narrow_mcp_tenants`, `revoke_grants_by_descends_from`.
   - Catalogue (1): `seed_catalogue_entry_for_composite` (thin wrapper over existing `seed_catalogue_entry`).
4. **SurrealStore impl** in new `store/src/repo_impl_m2.rs` (separate file — `repo_impl.rs` already crosses 1000 lines).
5. **InMemoryRepository impl** in `domain/src/in_memory.rs` with matching method bodies.

**Files created**: `templates/{mod,e}.rs`, `repo_impl_m2.rs`.

**Files modified**: `repository.rs`, `in_memory.rs`, `permissions/{engine,manifest}.rs`, `store/src/lib.rs`.

**Tests added (~25)**: Per-method repo integration tests (17); `template_e_props.rs` (3 invariants); `step_4_constraint_value_match_props.rs` (2 invariants); `in_memory_m2_test.rs` mirror of store surface (3 smoke tests).

### P3 — `handler_support` module + `SurrealAuditEmitter` + `spawn_claimed` (~2–3 days)

**Goal:** pages 02–05 can be built as thin handlers against a robust, tested shared shim. **Re-audit at P3 close** — shared infra must be bulletproof before per-page slices begin.

1. **`server/src/handler_support/`** (new directory):
   - `mod.rs` — public re-exports.
   - `session.rs` — `AuthenticatedSession(AgentId, SessionClaims)` axum extractor; 401 `ApiError { code: "UNAUTHENTICATED" }` on missing/invalid cookie.
   - `permission.rs` — `check_permission(state, ctx, manifest) -> Result<Vec<ResolvedReach>, ApiError>`; exhaustive `Decision → ApiError` mapping (D10); records `phi_permission_check_duration_seconds` histogram.
   - `audit.rs` — `emit_audit(&state.audit, event) -> Result<(), ApiError>`; maps emitter errors to 500 `AUDIT_EMIT_FAILED`.
   - `errors.rs` — shared `ApiError { code: &'static str, message: String }` (promoted from `handlers/bootstrap.rs` per D9).
2. **`SurrealAuditEmitter`** in `store/src/audit_emitter.rs`:
   - `Arc<SurrealStore>`-backed. Synchronous path: `last_event_hash_for_org` → populate `prev_event_hash` → `hash_event` (seals the chain) → `write_audit_event`.
   - `AppState` extends: `audit: Arc<dyn AuditEmitter>` + `master_key: Arc<MasterKey>`.
3. **Acceptance harness** extension: `spawn_claimed()` + `ClaimedAdmin { agent_id, session_cookie, authed_client }` in `tests/acceptance_common/admin.rs` (new file).
4. **Bootstrap handler migration**: update `handlers/bootstrap.rs` to use `handler_support::errors::ApiError` (small delta — imports change, shape identical).
5. **Re-audit at P3 close**: independent Explore sweep against Part 2 rows C1–C7 + C18. Target ≥ 99 %. Remediate LOW findings before opening P4.

**Files created**: `handler_support/{mod,session,permission,audit,errors}.rs`, `store/src/audit_emitter.rs`, `acceptance_common/admin.rs`.

**Files modified**: `state.rs` (add `audit` + `master_key`), `main.rs` (wire `SurrealAuditEmitter`), `handlers/bootstrap.rs` (ApiError import), `tests/acceptance_common/mod.rs` (`mod admin;` export).

**Tests added (~20)**: `server/tests/handler_support_test.rs` (8 `FailedStep` variants + 401 + 500); `store/tests/audit_emitter_chain_test.rs` (3-event chain continuity); `server/tests/spawn_claimed_smoke.rs`; `server/tests/permission_check_mapping_test.rs` (exhaustive match on `FailedStep`).

### P4 — Page 04: Credentials Vault vertical (~3 days) — **first M2 page**

Chosen first because: exercises M1 crypto (never used by a handler); exercises Permission Check with `purpose=reveal` constraint (widest contract test); exercises new `AuditEmitter` with 7 event types; exercises `AuthenticatedSession`. If P4 green, the vertical pattern scales.

1. **Business logic** in `server/src/platform/secrets/{mod,add,rotate,reveal,reassign,list}.rs`:
   - `add_secret` — validate slug unique; seal material; persist; emit `SecretAdded` (Alerted).
   - `rotate_secret` — re-seal new material; update `last_rotated_at`; emit `SecretRotated` (Alerted).
   - `reveal_secret` — check custodian; check Permission Check with `purpose=reveal`; emit `SecretRevealed` (Alerted) **BEFORE** returning plaintext (crash between emit + return still has audit trail); unseal + return plaintext.
   - `reassign_custody` — Template E auth request + re-target row; emit `SecretCustodyReassigned`.
   - `list_secrets` — read path; `[read, list]` only; emit `SecretListRead` (Logged).
2. **Handlers** in `server/src/handlers/platform_secrets.rs`: 5 routes.
3. **Audit events** in `domain/src/audit/events/m2/secrets.rs`: 7 builder fns — `SecretAdded`, `SecretRotated`, `SecretArchived`, `SecretCustodyReassigned`, `SecretRevealed`, `SecretRevealAttemptDenied`, `SecretListRead`.
4. **Route wiring** in `router.rs`: `/api/v0/platform/secrets/*`.
5. **CLI** `cli/src/commands/secrets.rs`: `list`, `add --material-file <PATH>`, `rotate --material-file <PATH>`, `reveal --purpose "<reason>" [--yes-audit]`, `reassign --new-custodian`. `--material-file -` reads from stdin.
6. **Web page** `app/(admin)/secrets/{page,actions,SecretsTable,AddSecretForm,RevealDialog,RotateDialog,ReassignDialog}.tsx`. `RevealDialog` implements the 3-state `idle/confirming/revealed` flow with 30-second countdown + discard-on-navigation.
7. **Ops doc** `operations/secrets-vault-operations.md`: rotation policy, reveal audit trail, master-key rotation placeholder, "I lost my custodian" runbook.
8. **ADR** `decisions/0017-vault-encryption-envelope.md` (or reuse M1's 0014).

**Tests added (~22)**: server handler suite (10 scenarios), `vault_roundtrip_props.rs` (seal→unseal identity; 50 cases), CLI help snapshot + claim-flow smoke (3), web `__tests__/secrets.test.tsx` (4), acceptance `acceptance_secrets.rs` (5 end-to-end scenarios).

### P4.5 — Grant instance-URI fundamentals (~0.5 day) — **retrospective detour**

**Goal** (G19 / D17 / C21): close the engine's instance-URI gap that P4 surfaced before any further M2 page issues instance grants. Ships *before* P5 so `ModelRuntime`, `ExternalService`, and `PlatformDefaults` grants use the new shape from day one rather than paying a migration later.

**Why a detour, not M3?**

- P5 / P6 / P7 each want per-instance grants (e.g. `provider:<id>`, `mcp:<id>`) for the same reasons P4 did — the workaround propagates otherwise.
- The fix is wire-compatible (new field, `#[serde(default)]`, empty → legacy URI-derivation). No migration, no handler API change.
- Deferring to M3 means paying the cost twice: once when M3 introduces delegated custody (the handler changes again) *and* once again as a data migration on every existing class-wide grant. Doing it now costs ~4–6 hrs; deferring costs a day plus backfill risk.

**Scope:**

1. **`Grant` struct extension** in `domain/src/model/nodes.rs`:
   - Add `pub fundamentals: Vec<Fundamental>` with `#[serde(default)]`.
   - Default (empty vec) preserves M1 semantics — `resolve_grant` falls back to URI-derivation when the field is empty.
2. **`resolve_grant` 4th case** in `domain/src/permissions/expansion.rs`:
   - Branch order: `system:root` → fundamental-name (Case A) → composite-name (Case B) → **persisted-fundamentals** (new Case D) → opaque-URI fallback (legacy Case C).
   - New Case D: if `grant.fundamentals` is non-empty, use it verbatim; selector = `Selector::parse(grant.resource.uri)` (instance URI treated as a specific selector).
   - Kind refinement stays `None` for Case D — the persisted fundamentals are authoritative; we don't synthesize a `#kind:` filter.
3. **`add_secret` update** (`server/src/platform/secrets/add.rs`):
   - Grant resource URI flips from `"secret_credential"` (class) to `"secret:<slug>"` (instance).
   - `fundamentals: vec![Fundamental::SecretCredential]` populated explicitly.
   - Per-secret revocation becomes possible (M3 delegated-custody unblocked).
4. **Sweep existing grant constructors**:
   - `server/src/bootstrap/claim.rs` — `system:root` grant stays class-shaped (special-cased). Add `fundamentals: vec![]` for explicitness.
   - Every test fixture that constructs `Grant { … }` literally: add `fundamentals: vec![]` so the struct-init compiles. Mechanical — grep `Grant \{` across the workspace.
5. **New proptest** `domain/tests/instance_uri_grant_match_props.rs`:
   - *Invariant 1*: Grant with explicit `fundamentals = [F]` + URI `kind:instance` + a matching manifest reach + a catalogue-present target URI → `Decision::Allowed`.
   - *Invariant 2*: Grant with **empty** fundamentals + URI that names a fundamental class → same behaviour as today (legacy-path regression guard).
   - *Invariant 3*: Grant with empty fundamentals + opaque instance URI → still denies at Step 3 (preserves M1 behaviour — no accidental admission).
6. **`acceptance_secrets.rs` revision**: existing reveal happy path stays green; one additional assertion verifies the persisted grant's resource URI is `secret:<slug>` (instance), not `secret_credential` (class).
7. **Docs** — update three pages to reflect the shape change:
   - `implementation/m2/architecture/vault-encryption.md` — replace the "scoped by catalogue + constraint" workaround paragraph with a per-slug description.
   - `implementation/m2/architecture/handler-support.md` — note that instance-URI grants now resolve via the persisted-fundamentals field.
   - `implementation/m1/architecture/graph-model.md` — document the new `Grant.fundamentals` field with a note that `None`/empty preserves M1 semantics.

**Files modified** (~8 prod + 1 new test):

- `modules/crates/domain/src/model/nodes.rs` — `Grant.fundamentals` field.
- `modules/crates/domain/src/permissions/expansion.rs` — Case D branch in `resolve_grant`.
- `modules/crates/server/src/platform/secrets/add.rs` — instance-URI grant + explicit fundamentals.
- `modules/crates/server/src/bootstrap/claim.rs` — empty-vec default on the genesis grant.
- Every test file that constructs `Grant { … }` literal (sweep): `domain/tests/*_props.rs`, `server/tests/handler_support_test.rs`, `store/tests/repository_test.rs`.
- `modules/crates/domain/tests/instance_uri_grant_match_props.rs` — new proptest file.
- `docs/specs/v0/implementation/m2/architecture/vault-encryption.md` — revised paragraph.
- `docs/specs/v0/implementation/m1/architecture/graph-model.md` — new-field note.

**Tests added (~5)**: 3 proptest invariants + 1 regression-check assertion in `acceptance_secrets.rs` + 1 unit test for `resolve_grant` Case D.

**Gates** (same as every phase): `cargo fmt/clippy/test --workspace` + `npm test/typecheck/lint/build` + `doc-links` + `check-phi-core-reuse` all green.

**Dependencies**: P4 must be landed (P4.5 builds on the `add_secret` grant-issuance path). P4.5 must land before P5 so `ModelRuntime` grants inherit the new shape without a rework.

### P5 — Page 02: Model Providers vertical (~2 days)

**phi-core reuse** (§1.5): `ModelConfig`, `ApiProtocol`, `ProviderRegistry`, `CacheConfig`, `ThinkingLevel` all imported directly. No parallel `ModelConfig` equivalent in phi.

1. **Business logic** in `server/src/platform/model_providers/{register,archive,list}.rs`. `register` takes a `phi_core::ModelConfig` (constructed from the POST body by a thin wire-translator) + `secret_ref`; persists; emits `ModelProviderRegistered`. `list` returns rows carrying embedded `ModelConfig` values (serde round-trippable — phi-core already derives).
2. **Handlers** `handlers/platform_model_providers.rs`: GET / POST / POST /{id}/archive. Web payload shape mirrors phi-core's `AgentConfig.provider` section where possible.
3. **Audit events** `audit/events/m2/providers.rs`: `ModelProviderRegistered`, `ModelProviderArchived`, `ModelProviderHealthDegraded` (shape only — no probe in M2).
4. **CLI** `cli/src/commands/model_provider.rs`: `list`, `add`, `archive`. `add` accepts `--provider <api-protocol>` with clap-validated values drawn from `phi_core::ApiProtocol`.
5. **Web page** `app/(admin)/model-providers/{page,actions,ProvidersTable,AddProviderForm,ArchiveConfirmDialog}.tsx`. The provider dropdown enumerates `ApiProtocol` variants fetched via a thin `/api/v0/platform/provider-kinds` read endpoint (backed by `ProviderRegistry::default()`).
6. **Ops doc** `operations/model-provider-operations.md`.
7. **ADR** `decisions/0016-template-e-self-interested-auto-approve.md` (Template E pattern documented here since P5 is the first page to use it end-to-end).
8. **C19 verification at phase close**: `grep -rn "struct ModelConfig" modules/crates/` returns zero hits; `Cargo.toml` in domain + server lists `phi-core = { workspace = true }`.

**Tests added (~14)**: handler suite (6: register / archive / no-references / with-references / auth / validation), CLI tests (3), web tests (3), acceptance (2).

### P6 — Page 03: MCP Servers vertical + cascading revocation (~2.5 days)

The most contract-dense M2 page. Cascading revocation is the highest-risk surface.

**phi-core reuse** (§1.5): `McpClient::connect_{stdio,http}`, `McpToolInfo`, `McpClient::list_tools()` all imported directly for the health-probe path. The live `McpClient` is instantiated on demand (short-lived) — not stored on the `ExternalService` composite.

1. **Business logic** in `server/src/platform/mcp_servers/{register,patch,archive,list,health_probe}.rs`:
   - `register/patch_tenants/archive/list` — persistence + cascading revocation flow.
   - `patch_tenants(...)` — when `new_allowed ⊂ old_allowed`, call `Repository::narrow_mcp_tenants`; emit `McpServerTenantAccessRevoked { mcp_id, revoked_orgs: [...] }` (one summary event) + one `auth_request.revoked` per affected AR (D7).
   - `health_probe(external_service) -> RuntimeStatus` — thin wrapper: construct a `phi_core::McpClient` via `connect_stdio` or `connect_http`; call `list_tools()` with a 2 s timeout; 3 retries with exponential backoff; returns `RuntimeStatus::{Ok, Error, Probing}`. This is the only phi-native MCP code — phi-core ships no health API (§1.5 🚫).
2. **Handlers** `handlers/platform_mcp_servers.rs`: GET / POST / PATCH / POST /{id}/archive + feature-gated `POST /{id}/_probe_health` (G5).
3. **Audit events** `audit/events/m2/mcp.rs`: `McpServerRegistered`, `McpServerTenantAccessRevoked`, `McpServerArchived`, `McpServerHealthDegraded`.
4. **CLI** `cli/src/commands/mcp_server.rs`: `list`, `add`, `patch --tenants-allowed <csv> --confirm-cascade`, `archive`.
5. **Web page** `app/(admin)/mcp-servers/{page,actions,McpServersTable,AddServerForm,PatchTenantsDialog,ArchiveConfirmDialog}.tsx`. `PatchTenantsDialog` computes the diff client-side and previews "these N grants will be revoked" before submitting. The tool-count column on `McpServersTable` reads directly from a lightweight `list_tools()` probe at SSR time (cached 60 s).
6. **Ops doc** `operations/mcp-server-operations.md`: tenant-narrowing semantics, cascade audit trail, incident playbook for accidental over-narrowing.
7. **C19 verification at phase close**: `grep -rn "struct McpClient" modules/crates/` returns zero hits (phi-core's is the only one).

**Tests added (~18)**: handler suite (8 scenarios including narrow/narrow-to-empty/patch-to-superset-noop/archive), `mcp_cascade_props.rs` (3 invariants: monotonic grant count, narrow-idempotence, no over-revocation), CLI tests (3), web tests (2), acceptance (2).

### P7 — Page 05: Platform Defaults vertical (~1.5 days)

Smallest page — singleton + non-retroactive invariant.

**phi-core reuse** (§1.5): the bulk of `PlatformDefaults` is phi-core types held as fields — `ExecutionLimits`, `AgentProfile`, `ContextConfig`, `CompactionStrategy`, `RetryConfig`. Config import/export uses `phi_core::config::parser::{parse_config, parse_config_auto}` directly.

1. **Business logic** `server/src/platform/defaults/{get,put,import,export}.rs`:
   - `put_defaults` — diff old vs new (phi-core structs have `Eq` / structural diff); emit `PlatformDefaultsUpdated { diff: {field: {old, new}} }` (Alerted); does NOT touch any org rows (invariant).
   - `import_defaults_from_yaml(yaml: &str) -> PlatformDefaults` — thin wrapper over `phi_core::parse_config` that extracts the overlapping sections into the `PlatformDefaults` envelope.
   - `export_defaults_as_yaml(&PlatformDefaults) -> String` — inverse; serialises the phi-core embedded sections + phi-only fields (retention_days, alert_channels).
2. **Handlers** `handlers/platform_defaults.rs`: GET / PUT + `POST /_import` / `GET /_export` (YAML content-type).
3. **Audit events** `audit/events/m2/defaults.rs`: `PlatformDefaultsUpdated`.
4. **CLI** `cli/src/commands/platform_defaults.rs`: `get [--include-factory] [--json]`, `set --key <dot.path> --value <json-literal>`, `import --file <PATH>`, `export [--format yaml|toml]`.
5. **Web page** `app/(admin)/platform-defaults/{page,actions,DefaultsForm,FactoryDefaultsPanel,ImportExportPanel}.tsx` — single-form layout with side panel showing factory defaults for reference + import/export buttons.
6. **Ops doc** `operations/platform-defaults-operations.md`: non-retroactive inheritance rule, stale-write 409 handling, factory-reset, YAML import/export safety.
7. **ADR** `decisions/0019-platform-defaults-non-retroactive.md`.
8. **C19 verification at phase close**: `grep -rn "struct ExecutionLimits\|struct AgentProfile\|struct RetryConfig\|struct ContextConfig" modules/crates/` returns zero hits. The `PlatformDefaults` struct field types are all qualified `phi_core::...`.

**Tests added (~10)**: handler suite (4: GET / PUT-happy / PUT-diff-shape / unauth), `platform_defaults_non_retroactive_props.rs` (≥50 cases: existing orgs unchanged after PUT), CLI tests (2), web tests (1), acceptance (2).

### P8 — Seal: CLI completion, metrics, spec-drift, runbook, re-audit (~2 days)

1. **CLI completion** `cli/src/commands/completion.rs`: `phi completion {bash,zsh,fish,powershell}` via `clap_complete::generate`.
2. **Cross-page acceptance** `server/tests/acceptance_m2.rs`: one end-to-end scenario: claim → add secret → register provider → register mcp → narrow mcp tenants → verify audit chain length + hash continuity.
3. **Acceptance metrics** `server/tests/acceptance_metrics.rs` (C12): one `with_metrics: true` test (G16) that trips a 403 and scrapes `/metrics` for the histogram.
4. **CI updates** `.github/workflows/rust.yml`:
   - Extend `acceptance` job's `--test` list with the 5 new acceptance binaries (Cargo's `--test` takes exact names, not globs).
   - New `check-ops-doc-headers` script invoked by `doc-links.yml`.
5. **Web smoke** `.github/workflows/web.yml`: add `npm run test:playwright-smoke` (headless-HTTP check, not Playwright-proper — that's M7b) guarded by env var.
6. **Ops updates**:
   - `docs/ops/runbook.md` gets an M2 section aggregating the 4 new page runbooks.
   - `user-guide/troubleshooting.md` appends M2 stable codes with emitting handler + recovery.
7. **Independent re-audit** (mirrors M1's post-P9 audit): 3 parallel Explore agents cover (a) Rust implementation across all phases, (b) documentation + verification matrix, (c) per-page vertical integrity. Target ≥ 99 %. Remediate LOW findings in the same session before M2 closes.

**Tests added (~8)**: completion help snapshots, acceptance-metrics (1), cross-page acceptance (1), doc-header check-script unit test.

**Total phase estimate: ~17 calendar days ≈ 3 weeks.** Within the build-plan M2 envelope.

---

## Part 5 — Testing strategy  `[STATUS: ⏳ pending]`

Following M1's post-audit convention: **row values are `cargo test` pass counts at phase close**; the trybuild row stays at `1` (cargo-level) with a Gains column note; worked-trace + doctests have their own row.

| Layer | New M2 count | Purpose |
|---|---|---|
| Domain unit (new composites + Template E + constraint match) | ~18 | Type shapes + pure helpers |
| Domain proptest (Template E, cascade, vault roundtrip, non-retroactive, constraint-match) | ~10 invariants across 5 files | Behaviour invariants |
| Store unit (migration 0002) | 1 | Migration shape |
| Store integration (repo method per new surface + emitter chain) | ~20 | Persistence round-trip |
| Server unit (handler_support internals) | ~10 | Session extractor + Permission Check mapping + ApiError shape |
| Server integration (4 page handler suites + handler_support + audit chain + acceptance-metrics) | ~50 | HTTP contract |
| Acceptance E2E (one per page + cross-page + metrics) | ~8 scenarios | Real axum + real SurrealDB |
| CLI integration (5 subcommand groups × 2-4 tests + help snapshots) | ~18 | Subcommand + JSON parity |
| Web unit (Node test runner: API wire translators + shared components) | ~12 | Pure translators + `<ApiErrorAlert />` |
| Web SSR smoke (admin layout gate + 4 pages' render path) | ~8 | Auth gate + SSR probe |
| **M2 added total** | **~155 Rust + ~20 Web = ~175 tests** | — |

**Post-M2 workspace target**: 299 (M1 close) + ~155 Rust = ~454 Rust; 14 + 20 = ~34 Web; **~488 combined**, up from M1's 313.

**Key invariants shipping in M2**:

- Permission-check mapping exhaustiveness (compile-time match on `FailedStep`).
- Cascading revocation monotonicity (narrowing never increases live grant count).
- Vault round-trip identity (add/rotate/reveal → same plaintext).
- Non-retroactive platform-defaults inheritance (PUT doesn't mutate existing org snapshots).
- Template E auto-approve produces `Approved` aggregate for every (scope, resource) combo.
- Audit chain continuity across ≥4 writes spanning ≥2 pages.
- Constraint value-match correctness (identical invocation without `purpose=reveal` denies at `FailedStep::Constraint`).

**Fixtures planned**:

- `domain/tests/common/m2_fixtures.rs` — `sample_secret()`, `sample_model_provider()`, `sample_mcp_server()`, `sample_platform_defaults()`, `ten_orgs_fixture()`.
- `server/tests/acceptance_common/admin.rs` — `spawn_claimed()` + `ClaimedAdmin`.

---

## Part 6 — Documentation  `[STATUS: ⏳ pending]`

Root: `phi/docs/specs/v0/implementation/m2/`. Layout mirrors M1.

```
implementation/m2/
├── README.md                                  8-phase index
├── architecture/
│   ├── overview.md                            system map + testing posture
│   ├── platform-catalogue.md                  resources_catalogue seeding + Step 0 integration
│   ├── template-e-auto-approve.md             self-interested auto-approval pattern
│   ├── vault-encryption.md                    per-entry seal / reveal flow
│   ├── cascading-revocation.md                MCP tenant-narrowing semantics
│   ├── handler-support.md                     AuthenticatedSession + check_permission + audit
│   ├── platform-defaults.md                   singleton + non-retroactive inheritance
│   ├── phi-core-reuse-map.md                  D16 / §1.5 durable — which phi-core types M2 wraps / imports
│   └── server-topology.md                     extends M1 with /api/v0/platform/* routes
├── user-guide/
│   ├── platform-setup-walkthrough.md          4-page tour for a new admin
│   ├── model-providers-usage.md
│   ├── mcp-servers-usage.md
│   ├── secrets-vault-usage.md
│   ├── platform-defaults-usage.md
│   └── cli-reference-m2.md                    new subcommands + completion
├── operations/
│   ├── model-provider-operations.md
│   ├── mcp-server-operations.md
│   ├── secrets-vault-operations.md
│   ├── platform-defaults-operations.md
│   └── m2-additions-to-m1-ops.md              schema-migrations + at-rest + audit-retention deltas
└── decisions/
    ├── 0016-template-e-self-interested-auto-approve.md
    ├── 0017-vault-encryption-envelope.md       (if not reusing M1's 0014)
    ├── 0018-handler-support-module.md          AuthenticatedSession + check_permission design
    └── 0019-platform-defaults-non-retroactive.md
```

**Conventions** (same as M1, enforced by `doc-links.yml` + new `check-ops-doc-headers.sh`):

- `<!-- Last verified: YYYY-MM-DD by Claude Code -->` on line 1 of every file.
- Status tags: `[EXISTS]`, `[PLANNED Mn]`, `[CONCEPTUAL]`.
- Code claims link to `modules/crates/…` with `../../../../../../` (6 `../`s) from `m2/{architecture,user-guide,operations,decisions}/`.
- Rationale-heavy claims link to the archived plan or concept doc.
- ASCII diagrams only.
- Docs for a phase land in the **same commit** as that phase's code.

---

## Part 7 — CI / CD extensions  `[STATUS: ⏳ pending]`

1. **`rust.yml` `acceptance` job** — extend the `--test` list to include the 5 new acceptance binaries:
   `--test acceptance_bootstrap --test acceptance_secrets --test acceptance_model_providers --test acceptance_mcp_servers --test acceptance_platform_defaults --test acceptance_m2 --test acceptance_metrics`.
2. **`rust.yml` new `ops-doc-headers` job** invokes new `scripts/check-ops-doc-headers.sh` — greps every `.md` under `docs/ops/` + `docs/specs/v0/implementation/m*/operations/` for the `Last verified` header; fails on missing.
3. **`web.yml` new `smoke` step** — `npm run test:playwright-smoke` (headless HTTP check; not browser Playwright — that's M7b). Runs against a Next server booted in the background. Asserts each admin page returns 200 with cookie + 302 to `/bootstrap` without.
4. **`spec-drift.yml` grep set** extends with `R-ADMIN-0[2345]-*`.
5. **`doc-links.yml`** — unchanged rules; tree seed at P1 (G17) ensures it's green from commit 1.
6. **`rust.yml` new `phi-core-reuse` job** invokes `scripts/check-phi-core-reuse.sh` (C19 / D16). Advisory (warn-only) in P1; flipped to hard gate after P3 close audit confirms the mandate's expectations are realistic.

---

## Part 8 — Verification matrix  `[STATUS: ⏳ pending]`

Before declaring M2 done, each commitment-ledger row maps to a green test.

| # | Commitment | Test / check |
|---|---|---|
| C1 | M2 model counts + IDs | `domain/tests/m2_model_counts.rs` |
| C2 | Migration 0002 applies forward-only | `store/tests/migrations_0002_test.rs` |
| C3 | Repository surface expanded (both impls) | `store/tests/repo_m2_surface_test.rs` + `domain/tests/in_memory_m2_test.rs` |
| C4 | Template E pure-fn helper | `domain/tests/template_e_props.rs` |
| C5 | `handler_support` module | `server/tests/handler_support_test.rs` (8 `FailedStep` variants + 401 + 500) |
| C6 | `SurrealAuditEmitter` + hash-chain | `store/tests/audit_emitter_chain_test.rs` |
| C7 | `spawn_claimed` harness fixture | `server/tests/spawn_claimed_smoke.rs` |
| C8 | Page 04 credentials vault | `server/tests/platform_secrets_test.rs` + `domain/tests/vault_roundtrip_props.rs` + `acceptance_secrets.rs` |
| C9 | Page 02 model providers | `server/tests/platform_model_providers_test.rs` + `acceptance_model_providers.rs` |
| C10 | Page 03 MCP servers + cascade | `server/tests/platform_mcp_servers_test.rs` + `domain/tests/mcp_cascade_props.rs` + `acceptance_mcp_servers.rs` |
| C11 | Page 05 platform defaults | `server/tests/platform_defaults_test.rs` + `domain/tests/platform_defaults_non_retroactive_props.rs` + `acceptance_platform_defaults.rs` |
| C12 | Prometheus histogram wired | `server/tests/acceptance_metrics.rs` |
| C13 | Audit chain continuity (cross-page) | `server/tests/audit_chain_m2_test.rs` |
| C14 | CLI subcommands for all pages + login | `cli/tests/help_snapshot.rs` + `cli/tests/platform_cli_test.rs` |
| C15 | Web admin layout + 4 pages | `modules/web/__tests__/{admin_layout,model_providers,mcp_servers,secrets,platform_defaults}.test.tsx` |
| C16 | Ops docs + troubleshooting | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C17 | doc-links green throughout | `.github/workflows/doc-links.yml` green on every PR |
| C18 | `check_permission` mapping exhaustiveness | `server/tests/permission_check_mapping_test.rs` |
| C19 | phi-core reuse mandate | `scripts/check-phi-core-reuse.sh` zero hits on forbidden duplications + P3 close + P-final audits spot-check composite struct fields |
| C20 | M1 residual drift resolved (R1 rename + R2 thiserror + R3 docs) | `grep -n "pub struct AgentProfile" modules/crates/domain/src/` returns zero hits; `/root/projects/phi/phi/Cargo.toml` workspace has `thiserror = "2"`; `cargo test --workspace` green after P0 commit; 4 M1 docs carry the phi-core-distinction paragraphs (audit-events, server-topology, graph-model, overview) |
| C21 | Instance-URI grants resolve in the engine (G19 / D17) | `domain/tests/instance_uri_grant_match_props.rs` — three proptest invariants (instance-URI-with-fundamentals → Allowed; legacy class-URI → regression-preserved; opaque-URI with empty fundamentals → still denies at Step 3). `acceptance_secrets.rs` asserts the persisted reveal-grant's `resource.uri` is `secret:<slug>`, not `secret_credential`. |

**First-review confidence target: ≥ 97 %**. Post-re-audit: ≥ 99 %. Post-100 %-audit (per M1 precedent): 100 % with all LOW findings closed.

---

## Part 9 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** → `phi/docs/specs/plan/build/<8hex>-m2-platform-setup.md`. Generate token via `openssl rand -hex 4`. (~2 min)
1. **P0** — M1 residual drift cleanup: rename `AgentProfile → AgentGovernanceProfile`; document audit-vs-agent-event distinction. Single tagged commit. (~0.5 day)
2. **P1** — foundation: IDs + composites + migration + docs-tree seed + web shell + CLI scaffolding + phi-core-reuse lint. (~2–3 days)
3. **P2** — Repository expansion + Template E + constraint value-match + both impls. (~2–3 days)
4. **P3** — `handler_support` + `SurrealAuditEmitter` + `spawn_claimed`. **Re-audit at P3 close.** (~2–3 days)
5. **P4** — Page 04 vertical (credentials vault — Rust + CLI + Web + ops). (~3 days)
6. **P4.5** — Retrospective detour: `Grant.fundamentals` field + `resolve_grant` Case D + `add_secret` instance-URI grant + sweep. Surfaced during P4; closes G19 / lands D17 / ticks C21 before any other vertical issues instance grants. (~0.5 day)
7. **P5** — Page 02 vertical (model providers). (~2 days)
8. **P6** — Page 03 vertical (MCP servers + cascade). (~2.5 days)
9. **P7** — Page 05 vertical (platform defaults). (~1.5 days)
10. **P8** — Seal: CLI completion + cross-page acceptance + metrics test + CI updates + runbook + independent re-audit. (~2 days)
11. **Re-audit → remediation → 100 %** (mirrors M1's post-P9 pass).
12. **Tag milestone** — `git tag v0.1-m2` in `phi` submodule (user-managed per M1 precedent).

**Total estimate: ~18 calendar days ≈ 3 weeks.** Within build plan M2's 2-week estimate + 1 week buffer for the shared-infrastructure-first discipline (P0 rename + P3 shared infra re-audit + P4.5 engine-gap closure) that M2 benefits from but M1 did not need.

---

## Part 10 — Critical files  `[STATUS: n/a]`

**New** (total ~55 production files + ~45 test files + ~25 docs):

- `modules/crates/domain/src/model/composites_m2.rs` — M2 composite-instance types.
- `modules/crates/domain/src/templates/{mod,e}.rs` — Template E pure helper.
- `modules/crates/domain/src/audit/events/{mod,m2/{secrets,providers,mcp,defaults}}.rs` — event builders.
- `modules/crates/store/migrations/0002_platform_setup.surql` — schema migration.
- `modules/crates/store/src/repo_impl_m2.rs` — M2 repo method impls.
- `modules/crates/store/src/audit_emitter.rs` — `SurrealAuditEmitter`.
- `modules/crates/server/src/handler_support/{mod,session,permission,audit,errors}.rs` — shared shim for all M2+ handlers.
- `modules/crates/server/src/platform/{mod,secrets,model_providers,mcp_servers,defaults}/**/*.rs` — business logic.
- `modules/crates/server/src/handlers/{platform_secrets,platform_model_providers,platform_mcp_servers,platform_defaults}.rs` — HTTP shims.
- `modules/crates/cli/src/commands/{login,secrets,model_provider,mcp_server,platform_defaults,completion}.rs`.
- `modules/crates/cli/src/{session_store,exit,pretty}.rs` — CLI infra.
- `modules/crates/server/tests/acceptance_common/admin.rs` — `spawn_claimed`.
- `modules/web/app/(admin)/{layout,model-providers,mcp-servers,secrets,platform-defaults}/**/*.{tsx,ts}`.
- `modules/web/app/components/{ApiErrorAlert,ConfirmDialog,DataTable}.tsx`.
- `modules/web/lib/api/{errors,forward-cookie,platform-providers,mcp-servers,secrets,platform-defaults}.ts`.
- `docs/specs/v0/implementation/m2/**/*.md` — full tree, including `architecture/phi-core-reuse-map.md` (publishes §1.5 as a durable doc).
- `scripts/check-ops-doc-headers.sh` — CI script.
- `scripts/check-phi-core-reuse.sh` — CI script enforcing D16 / C19.

**Modified**:

- `modules/crates/domain/src/repository.rs` — +17 methods.
- `modules/crates/domain/src/in_memory.rs` — matching impls.
- `modules/crates/domain/src/permissions/{engine,manifest}.rs` — constraint value-match.
- `modules/crates/domain/src/model/{ids,nodes,mod}.rs` — IDs, `TemplateKind`.
- `modules/crates/store/src/repo_impl.rs` — minor hoisting for shared helpers.
- `modules/crates/server/src/state.rs` — `AuditEmitter` + `MasterKey` on `AppState`.
- `modules/crates/server/src/router.rs` — 4 new nested routers.
- `modules/crates/server/src/main.rs` — instantiate `SurrealAuditEmitter`.
- `modules/crates/server/src/handlers/bootstrap.rs` — import `ApiError` from new common path.
- `modules/crates/cli/src/main.rs` — new subcommand groups.
- `modules/web/app/layout.tsx` — unchanged (route group handles admin).
- `modules/web/lib/session.ts` — `requireAdminSession()` HOF.
- `.github/workflows/{rust,web,spec-drift}.yml` — new jobs + glob/test-list updates.
- `docs/ops/runbook.md` — M2 section.
- `docs/specs/v0/implementation/m1/user-guide/troubleshooting.md` — M2 stable codes (or split to m2/ version).

---

## Part 11 — Open questions (non-blocking)  `[STATUS: n/a]`

Track in per-phase exec notes, not here:

- **Q1** (G5, D-reserved): Background health probe in M2 or M7b? Current plan: shape only in M2; real probe in M7b. Revisit if P4 review finds shape-only tests unconvincing.
- **Q2** (D11): `Consent → 202` vs `Consent → 409`? Current plan: `202` — the UI needs a "work continues" toast, not an error. 5-min product review before P3 closes.
- **Q3** (G6 / D6): Sync `AuditEmitter` stays through M3. Async queue is M7b. Flag if per-org alert delivery in M3 pushes latency above acceptable.
- **Q4** (G14): Template-E replay gap. Pre-approved construct bypasses the transition table; audit-chain verification in M7b replays transitions. Need a `provenance_template: TemplateKind::E` marker so replay special-cases.
- **Q5** (G16): Metrics test single-threaded via `OnceLock`. Switch to `serial_test` crate if M3 has >1 `with_metrics` consumer.
- **Q6** (D14): CLI session file at `0600` is adequate for M2. Evaluate keyring in M3 (when OAuth lands; session already multi-backend).
- **Q7** (G19 / D17): Should the engine synthesize a `#kind:` selector refinement from `grant.fundamentals` when the field is set? P4.5 ships **without** refinement (the persisted fundamentals are authoritative). Revisit if M3's delegated-custody flow wants tag-based refinement for multi-kind grants — M3 can add the refinement without changing the wire shape.
- **Q8** (P4 retrospective — sequential-write TOCTOU): `server::platform::secrets::add::add_secret` persists AR → vault row → catalogue → grant → audit event sequentially. No atomic batch API exists for M2 writes (unlike M1's `apply_bootstrap_claim`). Process-crash between steps leaves orphan rows. **Acceptable for M2** (single admin, manual cleanup via `secret list`) but should be hardened in M3 with an `apply_secret_add` repository method that runs the batch in one SurrealDB transaction. Same gap applies to the P5/P6/P7 verticals that land after P4.5.

---

## What stays unchanged  `[STATUS: n/a]`

- Concept docs (`docs/specs/v0/concepts/`) are the source of truth; only surface-count corrections in the build plan if M2 finds discrepancies.
- M1 ships unchanged (36 Repository methods, Permission Check engine, session cookie, etc.); M2 extends, does not refactor.
- `phi-core` is a library dependency; no new consumption in M2 beyond M1's pattern.
- The 15 reference layouts stay fixture material; first M2 consumer is page 04 via `platform-infra` layout references.
