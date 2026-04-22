# Plan: M1 — Permission Check Spine (build + docs)

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section
>
> **Revisions:**
> - **2026-04-20 r2** (P2-revision, post-P1 self-review): added C15 / D9 / ADR-0015 for type-safe `owned_by` / `created` / `allocated_to` edges — sealed `Principal`+`Resource` marker traits + typed `Edge::new_*` constructors + typed repository helpers. P2 grows from ≈2 → ≈3 days; total M1 tests grow from ≈178 → ≈190 (+ ≥6 `trybuild` compile-fail cases); ADR set grows from 0008–0014 (7) to 0008–0015 (8).
> - **2026-04-20 r1** (initial): plan approved and archived.

## Context  `[STATUS: n/a]`

M0 (scaffolding) shipped cleanly at 99 % confidence. M1 is the **Permission Check spine**: the graph model, Permission Check engine, Auth Request state machine, and System Bootstrap flow that every subsequent milestone sits on. The build-plan M1 entry is only ~15 lines ([build plan §M1](../../projects/phi/phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md)); this plan is the fully-resolved version — every contract pinned, every gap closed up-front, docs authored alongside code rather than after.

**What went wrong in M0 (that we're preventing here):**

1. A production-readiness commitment (TLS) shipped as a config surface with no listener — only caught during audit. **Prevention:** up-front Commitment Ledger (Part 2) checks every prod-readiness row tagged `M1` before P1 starts, and re-checks it at the end of P-final.
2. Several artefacts shipped late (`.env.example`, `package-lock.json`, runbook stub). **Prevention:** these are listed as P1 deliverables, not "we'll get to it."
3. Deep relative doc links broke because the m0/ tree sits six dirs under repo root. **Prevention:** docs tree planned with absolute workspace-relative link pattern documented in Part 5.
4. Build-plan headline counts ("31 nodes", "56+ edges") were stale vs concept doc (actually 37 + 66). **Prevention:** Part 1 surfaces all such deltas before coding starts.

**Archive location for this plan:** `phi/docs/specs/plan/build/<new-8hex>-m1-permission-check-spine.md`. First execution step (Part 8 step 0) is to copy this plan verbatim to that path, alongside the M0 audit plan and the v0.1 build plan.

---

## Part 1 — Pre-implementation gap audit  `[STATUS: ✓ done]`

Parallel Explore-agent sweep of `phi/docs/specs/v0/concepts/` + `requirements/` + current code. Findings:

### Concept-doc / build-plan deltas (must reconcile before coding)

| # | Finding | Source | Fix |
|---|---|---|---|
| G1 | Build plan claims **31 node types**; `concepts/ontology.md:20` defines **37**. | `concepts/ontology.md` §Node Types | Update build-plan count **and** M1 implementation target to 37. |
| G2 | Build plan claims **"56+ edges"**; `concepts/ontology.md:86` defines **66**. | `concepts/ontology.md` §Edge Types | Same — target is 66, not "56+". |
| G3 | Build plan's M1 bullet lists "first user-visible endpoint"; `requirements/admin/01*.md` actually requires **two** endpoints (`GET /status`, `POST /claim`). | `requirements/admin/01-platform-bootstrap-claim.md:92-113` | Both endpoints are M1 deliverables. |
| G4 | Permission Check has an explicit **Step 2a (ceiling enforcement)** not called out in the build plan's "6-step" framing. | `concepts/permissions/04-manifest-and-resolution.md:205-302` | Engine implements 0, 1, 2, **2a**, 3, 4, 5, 6 (seven explicit stages; still "6-step" by the canonical numbering). |
| G5 | System-agent provisioning (memory-extraction-agent, agent-catalog-agent) happens at **org creation**, not bootstrap — explicitly M3, not M1. | `requirements/admin/01*.md:130`; `concepts/system-agents.md` | M1 does not touch system agents. |
| G6 | Auth Request has **9 states** (Draft/Pending/In Progress/Approved/Denied/Partial/Expired/Revoked/Cancelled) and **two-tier 90-day active-window retention** — richer than the build-plan hint suggested. | `concepts/permissions/02-auth-request.md:68-442` | State machine + retention scaffolding both land in M1. |
| G7 | Permission Check **Step 0 = Resource Catalogue lookup** requires a `resources_catalogue` composite to exist per-org; bootstrap must seed the platform-level catalogue as its own `control_plane_object`. | `concepts/permissions/01-resource-ontology.md:179-201`; `requirements/system/s01-*.md` | Catalogue bootstrapping is part of s01. |
| G8 | NFR-observability requires `phi_permission_check_duration_seconds{result, failed_step}` — the `failed_step` label surfaces from the engine. | `requirements/cross-cutting/nfr-observability.md:26-31` | Engine must return structured decision values carrying failed_step. |
| G9 | Build plan prod-readiness table tags **at-rest encryption** as "M1 + M7b", and **schema migrations** as "M1 (first migration) + ongoing". Neither was in the terse M1 bullet. | `build-plan-v01.md §Production-readiness commitments` | Both are M1 deliverables (see Part 4). |

### Current-code extension points (inventory)

- `modules/crates/domain/src/`: stubs only — `model.rs` (NodeId skeleton), `permissions.rs` (Decision enum placeholder), `state_machines.rs` (comment marker), `repository.rs` (trait with only `ping()`). Every M1 type lands here.
- `modules/crates/store/src/lib.rs`: `SurrealStore::open_embedded` + `Repository::ping` only. M1 adds ~50 methods; `#[async_trait]` + `.client()` escape hatch are sufficient.
- `modules/crates/server/src/router.rs`: three health routes only; M1 attaches `/api/v0/bootstrap/{status,claim}` here. `AppState` (`state.rs`) may need extension for audit-event emitter + crypto-provider handle.
- `modules/crates/cli/src/main.rs`: **still consumes legacy `phi/config.toml` and runs a phi-core demo loop.** Clap imported but no subcommand tree. M1 must migrate CLI to layered-config + add `phi bootstrap {status,claim}` subcommands.
- `modules/web/app/`: only `page.tsx` + `layout.tsx`. M1 adds `app/bootstrap/page.tsx` + session-cookie helper promotion in `lib/session.ts` (TODO marker already exists).
- Tests inherited from M0: 4 (3 health + 1 TLS). No domain tests, no acceptance harness, no proptest setup yet. **M1 ships a multi-layer test suite that grows this to ≈178 runnable tests + ≈2,500 proptest branches per CI run — see Part 5 Testing strategy.**
- CI workflows: `rust.yml`, `web.yml`, `spec-drift.yml`, `doc-links.yml`. M1 extends `rust.yml` with proptest + acceptance-test gates; adds no new workflow unless required.

### Confidence: **target ≥ 99 % at first review**

Rubric: the M0 plan scored 75 % at first audit because one prod-readiness commitment shipped half-wired. Here we list every commitment, seed every piece in P1–P9, and make the pre-audit explicit so nothing is implied. See Part 8 for the verification matrix.

---

## Part 2 — Commitment ledger  `[STATUS: ⏳ pending]`

Every row from the build plan's §Production-readiness commitments that touches M1. Each row has an owning phase below and a verification step in Part 7.

| # | Commitment | M1 deliverable | Phase | Verification |
|---|---|---|---|---|
| C1 | Graph model — 9 fundamentals + 8 composites + **37 nodes + 66 edges** | Rust types + SurrealDB schema covering all | P1, P2 | Type enumeration test; schema-probe integration test |
| C2 | Permission Check 6-step (+Step 2a) | `permissions::check(...)` with structured `Decision` | P3 | ≥30 proptests; worked-example trace matches `permissions/04` §Worked trace |
| C3 | Auth Request 9-state machine, per-slot atomic, per-resource aggregation, forward-only revocation | `AuthRequest` type + transition fn + retention window logic | P4 | ≥20 proptests over state diagram |
| C4 | System Bootstrap flow (s01) — credential gen, single-use, platform-admin materialisation atomic | Install-time tool + claim handler + rollback on failure | P5, P6 | Acceptance test: fresh install → claim → grants + audit event correct |
| C5 | First user-visible endpoints | `GET /api/v0/bootstrap/status`, `POST /api/v0/bootstrap/claim` | P6 | Handler integration tests (201 / 400 / 403 / 409 paths) |
| C6 | Audit events (base schema + class tiers + alerted emit path) | `domain::audit` + shadow append-only log + hash-chain seed | P1, P4 | Audit emitter unit tests; hash-chain continuity proptest |
| C7 | Schema migrations (first migration + forward-only + fail-safe) | `store::migrations` module; startup gate in `main.rs` | P1 | Migration happy-path + broken-migration-refuses-to-serve tests |
| C8 | At-rest encryption (AES-256; env-injected master key; per-secret wrap placeholder) | Key-loading layer + encrypted secret columns in schema | P1, P2 | Unit test: encrypt/decrypt round trip; integration: key missing ⇒ startup fails |
| C9 | Resource Catalogue seeding at bootstrap | Platform-level catalogue created as `control_plane_object` on first boot | P5 | Permission Check Step 0 exercises catalogue in acceptance test |
| C10 | Prometheus metric `phi_permission_check_duration_seconds{result, failed_step}` | Histogram registered + recorded in engine | P3 | `/metrics` scrape shows the series after one check |
| C11 | Session cookie stub promoted (real OAuth is M3; M1 ships signed cookie helper) | `server::session` module; web `lib/session.ts` reads it | P6, P8 | Cookie set on successful claim; read on follow-up request |
| C12 | CLI migrated to layered config + `phi bootstrap {status,claim}` | CLI reads `ServerConfig::load()`; clap subcommand tree | P7 | CLI integration test against running server |
| C13 | Acceptance-test harness (reference-layout fixture) | `tests/acceptance/` with `bootstrap_claim.rs` | P9 | One green acceptance test at P9 close |
| C14 | Doc-authoring co-located with code | `docs/specs/v0/implementation/m1/` tree grows per phase | P1–P9 | `doc-links.yml` CI stays green throughout |
| C15 | Type-safe Resource/Principal edges — tightens Risk 1 surfaced during P1 self-review. Sealed `Principal` + `Resource` marker traits; typed `Edge::new_owned_by` / `new_created` / `new_allocated_to` constructors; typed repository helpers for the three untyped-RELATION edges (`owned_by`, `created`, `allocated_to`). Rust now rejects wrong endpoint pairs at compile time even though SurrealDB can't. | `domain::model::principal_resource` module + typed constructors on `Edge` + typed repo methods `upsert_ownership` / `upsert_creation` / `upsert_allocation` | P2 | Unit tests over every trait impl; `trybuild` compile-fail tests prove wrong-pair cases don't build; integration round-trip through each typed helper against real SurrealDB |

---

## Part 3 — Decisions made up-front  `[STATUS: ⏳ pending]`

To avoid mid-build thrashing, these are baked in. Push back in review if any are wrong.

| # | Decision | Rationale |
|---|---|---|
| D1 | **Bootstrap credential delivery** = printed to stdout on `phi-server --bootstrap-init` (one-shot admin command). Stored hashed (argon2id) in `bootstrap_credentials` table with `consumed_at IS NULL`. | Matches 12-factor; no file on disk with the plaintext; admin copies once and loses it like an SSH host key. |
| D2 | **At-rest encryption scope in M1** = envelope encryption for the `secrets_vault` table only, using a master key from `PHI_MASTER_KEY` env var (32-byte base64). Full-DB encryption deferred to M7b. | The concept contract names the vault as the encryption-sensitive surface; broad encryption without KMS is theatre. |
| D3 | **Schema migrations** = hand-written SurrealDB scripts under `modules/crates/store/migrations/{NNNN}_{slug}.surql`; runner walks sorted + records applied versions in a `_migrations` table. | Simple, inspectable; no external migration tool needed. |
| D4 | **Audit event storage** = primary to `audit_events` table (SurrealDB) + shadow NDJSON append to `{data_dir}/audit.log`. Hash-chain foundation: every event carries `prev_event_hash` within its org scope; full off-site stream is M7b. | Cheap recoverability now; full tamper-evident stream comes when we wire S3 object-lock in M7b. |
| D5 | **Docs tree layout** mirrors M0's 4-folder shape (`architecture/`, `user-guide/`, `operations/`, `decisions/`), under `docs/specs/v0/implementation/m1/`. Decisions numbered **0008–0014** (adjacent to M0's 0001–0007). | Consistent navigation across milestones. |
| D6 | **Session cookie scheme in M1** = `HS256`-signed JSON via `tower-cookies` + `jsonwebtoken`; scope is "platform admin session only" (the only principal that exists). OAuth wiring is M3. | Enough for M1; keeps the web surface honest rather than mocked. |
| D7 | **Proptest coverage gate in CI** = ≥ 50 proptest cases per invariant (default), with `PROPTEST_CASES=1000` runnable locally for deeper sweeps. | Keeps CI under 5 min while giving confidence. |
| D8 | **Domain crate does not depend on store crate** — repository is a trait defined in `domain`, implemented in `store`. Tests in domain use an in-memory fake. | Matches M0's `Arc<dyn Repository>` pattern; keeps the crate DAG downward-only. |
| D9 | **Type-safe ownership edges (added 2026-04-20 after P1 self-review Risk 1):** the three edges that accept `Resource`/`Principal` unions (`owned_by`, `created`, `allocated_to`) stay as single SurrealDB RELATION tables (no schema explosion) but gain **two layers of Rust-side safety** — (a) sealed `Principal` + `Resource` marker traits implemented on the relevant `*Id` newtypes (`Principal` on AgentId/UserId/OrgId/ProjectId; `Resource` on every ownable-node ID); (b) typed `Edge::new_*` constructors + typed repository helpers that only accept correctly-typed ID pairs. Callers cannot cross-paste an `AuditEventId` into an `owned_by`'s principal slot. The existing `upsert_edge(edge: Edge)` stays available for tests and for single-typed edges. | The three edges genuinely are type unions — DB-side partitioning would balloon into ~150 concrete-pair tables. Rust-side safety via sealed marker traits is zero runtime cost and keeps the schema clean. ADR-0015 captures the full rationale. |

---

## Part 4 — Implementation phases  `[STATUS: ⏳ pending]`

Nine phases, strictly sequenced so each phase's output is the next phase's input. Every phase closes with: code green (fmt/clippy/test), docs updated under `docs/specs/v0/implementation/m1/`, and the relevant Commitment-ledger row ticked.

### P1 — Foundation (≈3 days)

1. **Graph model types** in `modules/crates/domain/src/model/`:
   - `fundamentals.rs` — 9 enums (FilesystemObject, NetworkEndpoint, …).
   - `composites.rs` — 8 struct types.
   - `nodes.rs` — 37 node types with their field shape (refs: `concepts/ontology.md:20-85`).
   - `edges.rs` — 66 edge types as a tagged enum (refs: `concepts/ontology.md:86-201`).
   - `ids.rs` — `NodeId`, `EdgeId`, `OrgId`, `AgentId`, `GrantId`, `AuthRequestId` (all `Uuid` newtypes).
   - Every public type `#[derive(Serialize, Deserialize, Clone, Debug)]`; `serde(tag = "kind")` on the enums.
2. **SurrealDB schema** in `modules/crates/store/migrations/0001_initial.surql`:
   - One table per node type (`DEFINE TABLE <node_type> SCHEMAFULL`).
   - Typed edges via SurrealDB `RELATE` with edge-table definitions.
   - Indexes on `id`, `owning_org`, and the fields named in `concepts/ontology.md` as query-critical.
   - `_migrations` meta-table.
   - `bootstrap_credentials`, `secrets_vault`, `audit_events` utility tables.
3. **Migration runner** in `modules/crates/store/src/migrations.rs`:
   - On `SurrealStore::open_embedded`, walk `migrations/*.surql` sorted; skip versions already in `_migrations`; apply in a transaction; record.
   - `startup_gate`: if any migration errors, `main.rs` aborts with a clear message — fail-safe.
4. **Audit framework skeleton** in `modules/crates/domain/src/audit.rs`:
   - `AuditEvent { event_id, event_type, actor_agent_id?, target_entity_id?, timestamp, diff, audit_class, provenance_auth_request_id? }`.
   - `AuditClass` enum (Silent / Logged / Alerted).
   - `AuditEmitter` trait — writes to repository + shadow NDJSON; computes `prev_event_hash` within org scope.
5. **At-rest encryption layer** in `modules/crates/store/src/crypto.rs`:
   - `MasterKey` newtype loaded from `PHI_MASTER_KEY` (fails startup if missing when `secrets_vault` is touched).
   - `seal(plaintext)` / `open(ciphertext)` using `aes-gcm` 0.10.
   - Applied to the single `secrets_vault.value` column via a repository wrapper.
6. **Docs authored in this phase:**
   - `docs/specs/v0/implementation/m1/README.md` (index, same shape as M0 README).
   - `architecture/overview.md`, `architecture/graph-model.md`, `architecture/schema-migrations.md`, `architecture/audit-events.md`, `architecture/at-rest-encryption.md`.
   - `decisions/0009-surrealdb-schema-layout.md`, `0012-forward-only-migrations.md`, `0013-audit-events-class-and-chain.md`, `0014-at-rest-encryption-envelope.md`.

### P2 — Repository trait expansion + type-safe edges (≈3 days)

1. **Type-safe edge foundation** (addresses Risk 1 from the P1 self-review; see D9 + C15):
   - Create `modules/crates/domain/src/model/principal_resource.rs`. Use a **sealed-trait pattern** (`pub(crate) mod sealed { pub trait Sealed {} }`) so external crates cannot add rogue `Principal`/`Resource` impls.
   - Define `pub trait Principal: sealed::Sealed { fn node_id(&self) -> NodeId; }` and `pub trait Resource: sealed::Sealed { fn node_id(&self) -> NodeId; }`.
   - Implement `Principal` on `AgentId`, `UserId`, `OrgId`, `ProjectId` (matches concept doc §Governance Wiring).
   - Implement `Resource` on every node-ID newtype that can be owned — `NodeId` (generic fallback), `AgentId` (agents are also resources), `SessionId`, `MemoryId`, and every other M1-present `*Id`. Implementations are one-liners forwarding to `node_id()`.
   - Add typed constructors on `Edge`: `Edge::new_owned_by<R: Resource, P: Principal>(resource: &R, principal: &P) -> Edge`, `Edge::new_created<P: Principal, R: Resource>(creator: &P, resource: &R) -> Edge`, `Edge::new_allocated_to<P1: Principal, P2: Principal>(from: &P1, to: &P2) -> Edge`.
   - Add `trybuild` dev-dep + `modules/crates/domain/tests/edge_type_safety/compile_fail/` directory with at least 3 `compile_fail` cases proving wrong-pair constructions don't build (e.g. passing a `ConsentId` as the Principal to `new_owned_by`).
2. In `domain/src/repository.rs`, add methods (names illustrative; full list in P2 sub-plan during exec):
   - Grants: `create_grant`, `revoke_grant`, `list_grants_for_principal`, `get_grant`.
   - Auth Requests: `create_auth_request`, `transition_slot`, `get_auth_request`, `list_active_auth_requests_for_resource`.
   - Nodes: `upsert_node`, `get_node`, `neighbours`, `traverse_authority_chain`.
   - **Typed ownership helpers** (new, from D9/C15): `upsert_ownership<R: Resource, P: Principal>(resource: &R, owner: &P, auth_request: Option<AuthRequestId>)`; `upsert_creation<P: Principal, R: Resource>(creator: &P, resource: &R)`; `upsert_allocation<P1: Principal, P2: Principal>(from: &P1, to: &P2, resource: &ResourceRef, auth_request: AuthRequestId)`. These compile-check the endpoint types at every callsite; the existing generic `upsert_edge(edge: Edge)` remains available for tests and for edges whose endpoints are already single-typed.
   - Bootstrap: `put_bootstrap_credential`, `consume_bootstrap_credential`, `get_admin_agent`.
   - Catalogue: `seed_catalogue_entry`, `catalogue_contains`.
   - Audit: `write_audit_event`, `last_event_hash_for_org`.
3. Implement all in `store/src/lib.rs` against SurrealDB. Keep each method under 30 lines — anything longer earns a helper. The typed helpers delegate to the generic `upsert_edge` internally but narrow the callsite surface.
4. In-memory fake `InMemoryRepository` in `domain/tests/common/` for unit tests. The fake exposes the same typed-helper surface so tests exercise the same type safety callers hit in production.
5. **Docs:**
   - `architecture/storage-and-repository.md` (promoted from M0 stub; now covers grant/auth-request/audit surface + a dedicated section on the type-safe ownership-edge helpers).
   - `decisions/0015-type-safe-ownership-edges.md` (new ADR — documents the sealed-marker-trait approach, why DB-side partitioning was rejected, and the `trybuild` compile-fail strategy).
   - Amendment note in `decisions/0009-surrealdb-schema-layout.md` §Consequences, pointing at 0015 as the compile-time mitigation for the three untyped RELATIONs.

### P3 — Permission Check engine (≈3 days)

1. In `domain/src/permissions/engine.rs`:
   - `Decision = Allowed { resolved_grants } | Denied { failed_step, reason } | Pending { awaiting_consent }`.
   - `check(ctx: &CheckContext, manifest: &Manifest) -> Decision`.
   - Internal helpers `step_0_catalogue`, `step_1_expand_manifest`, `step_2_resolve_grants`, `step_2a_ceiling`, `step_3_match_reaches`, `step_4_constraints`, `step_5_scope_resolution`, `step_6_consent_gating`.
   - Every step returns a typed intermediate that the next step consumes. No panics.
2. Instrument with `tracing::instrument` + record histogram `phi_permission_check_duration_seconds{result, failed_step}`.
3. Property tests across ≈5 files, ≈10–12 `proptest!` invariants total (each expanded by `PROPTEST_CASES=100` in CI → ≈1,000–1,200 branches). Canonical invariants:
   - No grant ⇒ Denied at step 3.
   - Catalogue missing ⇒ Denied at step 0.
   - Constraint mismatch ⇒ Denied at step 4.
   - Template grant missing consent ⇒ Pending at step 6.
   - Worked example from `permissions/04` §Worked trace ⇒ byte-for-byte identical decision.
   - Ceiling always clamps strictly (step 2a never widens).
   - Adding a non-matching grant never changes an Allowed decision.
4. **Docs:** `architecture/permission-check-engine.md` (with ASCII pipeline diagram + worked trace cross-ref); `decisions/0008-permission-check-as-pipeline.md`.

### P4 — Auth Request state machine (≈3 days)

1. In `domain/src/auth_requests/`:
   - `state.rs` — 9-state enum + per-slot / per-resource / request-level aggregation functions.
   - `transitions.rs` — `transition_slot(req, slot_idx, new_state)` guarded by the legal-transition table; returns `Result<AuthRequest, TransitionError>`.
   - `retention.rs` — active-window tracking (`active_until` derived from grant liveness + 90d).
   - `revocation.rs` — forward-only revocation emits audit event; cascades to sub-grants.
2. Property tests across ≈4 files, ≈8–10 `proptest!` invariants total (each expanded by `PROPTEST_CASES=100` in CI → ≈800–1,000 branches). Canonical invariants:
   - Illegal transition never succeeds.
   - Aggregation rules match the table in `concepts/permissions/02` line-by-line.
   - Revocation never un-revokes a sub-grant (forward-only monotonicity).
   - Expiry is terminal: no transition out of Expired.
   - Per-slot approvals are independent (one approver's state never affects another's).
   - Active-window countdown monotonically non-increasing.
3. **Docs:** `architecture/auth-request-state-machine.md` (with ASCII state diagram + transition table); `decisions/0010-per-slot-aggregation.md`.

### P5 — System Bootstrap flow s01 (≈2 days)

1. `server/src/bootstrap/` module:
   - `init.rs` — generates credential (32 random bytes, base64), hashes with argon2id, stores hashed, prints plaintext once to stdout via `--bootstrap-init` subcommand on the server binary.
   - `claim.rs` — single-transaction implementation of R-SYS-s01-1 through s01-6:
     1. Lookup credential by digest + `consumed_at IS NULL`.
     2. Create Bootstrap Auth Request (auto-approved, provenance `template:system_bootstrap`).
     3. Create Human Agent node.
     4. Create Inbox/Outbox composite nodes.
     5. Seed platform-level Resources Catalogue.
     6. Issue `[allocate]` Grant on `system:root` to the new Human Agent.
     7. Mark credential consumed.
     8. Emit alerted `PlatformAdminClaimed` audit event.
     - Any step error ⇒ full rollback; credential stays unconsumed.
2. **Docs:** `architecture/bootstrap-flow.md` (sequence diagram in ASCII); `decisions/0011-bootstrap-credential-single-use.md`; `user-guide/first-bootstrap.md`.

### P6 — HTTP endpoints + session cookie (≈2 days)

1. `server/src/handlers/bootstrap.rs`:
   - `GET /api/v0/bootstrap/status` → 200 with `{claimed, admin_agent_id?}`.
   - `POST /api/v0/bootstrap/claim` → 201 on success with the full payload from `requirements/admin/01:92-113`; 400/403/409 error shapes match the requirement exactly.
2. `server/src/session.rs`:
   - `sign(session)` → HS256 JWT in a `phi_kernel_session` cookie (Secure, HttpOnly, SameSite=Lax).
   - `verify(cookie)` for follow-up requests.
3. Wire in `router.rs` under `/api/v0/` scope; attach `tower_cookies::CookieManagerLayer` as middleware.
4. Handler integration tests: 201 / 400 (bad channel) / 403 (bad token) / 409 (already claimed).
5. **Docs:** `architecture/server-topology.md` (extend M0 version with the new route table); `user-guide/http-api-reference.md` (new).

### P7 — CLI subcommands (≈2 days)

1. Migrate `modules/crates/cli/src/main.rs`:
   - Drop legacy `phi/config.toml` reader.
   - `ServerConfig::load()` for config (same layered path as server).
   - Clap tree: `phi bootstrap {status,claim}` (first subcommand group; later milestones add `phi org …`, `phi grant …`, etc.).
   - HTTP client via `reqwest`; respects `PHI_API_URL`.
   - Pretty-prints successful claim (prints the audit event id + next-step URL).
2. Keep the existing phi-core agent-loop demo in a `phi agent demo` subcommand so we don't regress the prototype. (Short-term; retiring the demo altogether is an M2+ cleanup item.)
3. CLI integration test in `cli/tests/bootstrap_cli.rs`: spawn server, hit claim, assert exit code + stdout shape.
4. **Docs:** `user-guide/cli-usage.md`.

### P8 — Web bootstrap page (≈2 days)

1. `modules/web/app/bootstrap/page.tsx`:
   - SSR probe of `/api/v0/bootstrap/status`; if claimed ⇒ render "already assigned" terminal view.
   - Else render claim form (credential input + display name + channel kind dropdown + handle).
   - Form submit → `POST /api/v0/bootstrap/claim`; on success show audit event id + redirect stub to `/` (Phase 2 is M2+).
2. `modules/web/lib/session.ts`: read signed cookie (from P6) and return `SessionUser`.
3. `modules/web/lib/api.ts`: add `getBootstrapStatus()` + `postBootstrapClaim(payload)`.
4. Playwright-less manual smoke: `npm run dev` + go through happy path + screenshot captured in docs.
5. **Docs:** `user-guide/web-usage.md` (includes the screenshot); `architecture/web-topology.md` extension.

### P9 — Acceptance harness + final seal (≈2 days)

1. `phi/tests/acceptance/common/mod.rs`: fixture builder that boots the server against a temp data dir + unique namespace.
2. `phi/tests/acceptance/bootstrap_claim.rs`:
   - Fresh install path (no admin yet) ⇒ status reports `claimed: false`; claim succeeds; follow-up status reports `claimed: true, admin_agent_id: _`; audit event present in DB with class `Alerted`; grant rows present; credential marked consumed.
   - Reused credential ⇒ 403 `BOOTSTRAP_ALREADY_CONSUMED`; no audit event emitted.
   - Second claim after success ⇒ 409.
3. Extend `.github/workflows/rust.yml` with an `acceptance` job (feature-flagged if needed).
4. **Docs:** `operations/schema-migrations-operations.md`, `operations/at-rest-encryption-operations.md`, `operations/bootstrap-credential-lifecycle.md`, `operations/audit-log-retention.md`, `user-guide/troubleshooting.md` (M1-specific entries appended).
5. Final doc-links CI run must stay green.

---

## Part 5 — Testing strategy  `[STATUS: ⏳ pending]`

M0 shipped 4 tests because M0 was scaffolding. M1 is the first milestone where **business logic** lives — Permission Check decisions, Auth Request transitions, audit events, bootstrap atomicity. Test volume has to match, and the test layers have to be chosen so each class of bug has the right catcher. This section makes the test plan explicit so Part 4 phases don't ship with a thin suite.

### Layers

| Layer | Location | Purpose | Runs in | Count target |
|---|---|---|---|---|
| **Unit — pure** | `domain/src/**` (inline `#[cfg(test)]`) + `domain/tests/unit_*.rs` | Functions with no I/O: enum counts, serde round-trips, algorithm step helpers, hash-chain computation, crypto `seal`/`open` | `cargo test -p domain` (fast, no DB) | ~50 |
| **Property — invariants** | `domain/tests/*_props.rs` | Invariants over randomly-generated inputs: permission-check decision correctness, state-machine legality, aggregation rules, revocation monotonicity, hash-chain continuity | Same binary as unit; `PROPTEST_CASES` knob controls depth | **≈25 proptest functions** spread across ≈13 files (P3 ≈ 5 files, P4 ≈ 4 files, rest ≈ 4 files). Each function = 1 invariant, expanded to `PROPTEST_CASES=100` random inputs in CI. Total per CI run: **25 × 100 = ≈2,500 input branches** (10× at `PROPTEST_CASES=1000` locally). |
| **Integration — storage** | `store/tests/*.rs` | Repository methods against a real embedded SurrealDB (tempdir per test); migration runner happy + fail-safe paths; crypto column wrapper; `prev_event_hash` linking across writes; typed ownership-helper round trips (`upsert_ownership` / `upsert_creation` / `upsert_allocation`) | `cargo test -p store` | ~92 |
| **Compile-fail — type safety** | `domain/tests/edge_type_safety/compile_fail/*.rs` | `trybuild` cases proving wrong-pair endpoint types (e.g. `ConsentId` as Principal) are rejected at compile time for the three untyped-RELATION edges | `cargo test -p domain --test edge_type_safety` | ≥6 (`trybuild` compile-fail) |
| **Integration — server** | `server/tests/*.rs` | Handler + state + repo wired together; bootstrap endpoints 201 / 400 / 403 / 409; session sign/verify round-trip; metric surfacing; health + TLS (M0 inheritance) | `cargo test -p server` | ~25 (includes the 4 M0 tests) |
| **Integration — CLI** | `cli/tests/*.rs` | Spawn server, invoke CLI subcommand, assert exit code + stdout shape + JSON parity with HTTP API | `cargo test -p cli` | ~8 |
| **Acceptance — E2E** | `phi/tests/acceptance/*.rs` | Full-system scenarios over a booted server: fresh-install bootstrap, reused credential, already-claimed, Resource Catalogue exercised via Permission Check, `/metrics` exposes new series | `cargo test --workspace --test 'acceptance_*'` (release profile, separate CI job) | ~6 |
| **Web — unit** | `modules/web/__tests__/*.test.ts` | Session helper, API client wrappers, cookie parser | `npm test` | ~10 |
| **Web — SSR smoke** | `modules/web/__tests__/bootstrap.test.tsx` | `/bootstrap` SSR renders the claim form when unclaimed; terminal view when claimed; form submission error-path | `npm test` | ~4 |
|  |  |  | **Total M1 runnable tests** | **≈190** (+ ≥6 `trybuild` compile-fail cases) |

Plus **≈2,500 proptest branches per CI run** — an order of magnitude more at `PROPTEST_CASES=1000` locally. This is the coverage depth a Permission Check engine deserves; anything less invites the class of bug where a rare grant-topology denies when it should allow (or vice versa) and we only find out in production.

### Coverage philosophy

- **Algorithm correctness ⇒ proptest.** Permission Check, Auth Request transitions, hash-chain walks, aggregation rules. Proptest is how we catch the edge cases no human thinks of. Every proptest names the invariant it's checking in its fn name (`no_grant_denies_at_step_3`, `revocation_is_forward_only`, etc.).
- **Wire correctness ⇒ integration.** Repository methods must be tested against a real SurrealDB, not a mock. (Matches prior feedback: mocked DBs hide migration-time breakage.)
- **Contract correctness ⇒ acceptance.** Every R-ADMIN-01-* and R-SYS-s01-* requirement maps to ≥ 1 acceptance scenario that exercises it through the real HTTP surface.
- **Error-path parity.** For every success test there is ≥ 1 matching failure test (bad input, DB error, missing key, consumed credential, race). No success-only suites.
- **No new `#[ignore]` without a linked issue.** Keeps the gate honest.

### Per-phase test budget

Part 4's phases each close with a green test layer. The verification matrix in Part 8 ties each commitment-ledger row back to specific test files.

| Phase | Tests authored | Suite |
|---|---|---|
| P1 | graph-model enumeration, serde round-trips, migration runner (happy + broken), crypto round-trip, missing-key-fails-startup, audit hash-chain seed unit | unit + integration (~25) |
| P2 | repository methods per surface (CRUD per type × error paths) + typed ownership-helper round trips + marker-trait impls (unit) + `trybuild` compile-fail cases proving wrong-pair edges don't build | integration (~92) + compile-fail (≥6) |
| P3 | 6-step engine proptests + worked-trace byte match + metric recording | proptest + unit (~12 files, 15 unit) |
| P4 | Auth Request 9-state transitions, per-slot/per-resource/req-level aggregation, forward-only revocation, 90-day retention window | proptest + unit (~8 files, 10 unit) |
| P5 | bootstrap flow atomicity: each sub-step rollback path; credential single-use | unit + integration (~8) |
| P6 | bootstrap handler 201/400/403/409; session cookie sign/verify | integration (~12) |
| P7 | CLI subcommand parse + end-to-end against spawned server | integration (~8) |
| P8 | SSR smokes (claim form renders, terminal view renders, failed submit) + session helper unit | web (~14) |
| P9 | acceptance scenarios (6 end-to-end flows) | acceptance (~6) |

### Fixtures and helpers (planned up-front so they're not reinvented per phase)

- `domain/tests/common/mod.rs` — `FakeRepo` (in-memory `Repository` impl), `sample_manifest()`, `sample_grant()`, `sample_auth_request()` builders, `proptest` strategies for grant sets + manifests.
- `store/tests/common/mod.rs` — `temp_surreal()` boots a fresh embedded store in a tempdir, returns a `SurrealStore` handle that drops the dir on scope exit.
- `phi/tests/acceptance/common/mod.rs` — `spawn_test_server()` spins up the real server binary on a random free port, returns `{url, shutdown_tx, data_dir}`; `#[tokio::test]` async.
- `modules/web/__tests__/helpers.ts` — `mockFetch(responses)`, `renderWithSession(tree, user)`.

### Gating rules (workflow-level detail in Part 7)

- `cargo test --workspace --locked` — standard CI job.
- `cargo test --workspace --test 'acceptance_*'` — separate `acceptance` job, release profile, slower; gates merges.
- Proptest job runs with `PROPTEST_CASES=100` in CI; any failure captures the reduced seed so the repro runs locally.
- `npm test` + `npm run build` gate the web surface.
- Flakes are not tolerated: a failing test retried green once in CI must open a tracking issue referenced in the next PR.

---

## Part 6 — Documentation  `[STATUS: ⏳ pending]`

Root: `phi/docs/specs/v0/implementation/m1/`. Layout mirrors M0.

```
implementation/m1/
├── README.md                                navigation index
├── architecture/
│   ├── overview.md                          system map with M1 extensions
│   ├── graph-model.md                       9+8+37+66 with types ↔ concept refs
│   ├── permission-check-engine.md           6-step pipeline + ASCII diagram
│   ├── auth-request-state-machine.md        9-state diagram + transition table
│   ├── bootstrap-flow.md                    s01 sequence diagram
│   ├── audit-events.md                      base shape, classes, hash-chain seed
│   ├── schema-migrations.md                 forward-only runner, startup gate
│   ├── at-rest-encryption.md                envelope encryption for the vault
│   ├── server-topology.md                   M0 extension; route table grows
│   ├── web-topology.md                      M0 extension; /bootstrap page + cookie
│   └── storage-and-repository.md            M0 extension; repo methods + typed ownership-edge helpers
├── user-guide/
│   ├── first-bootstrap.md                   end-to-end walkthrough (CLI + web)
│   ├── cli-usage.md                         phi bootstrap {status,claim}
│   ├── web-usage.md                         /bootstrap page walkthrough
│   ├── http-api-reference.md                /api/v0/bootstrap/* contract
│   └── troubleshooting.md                   M1 error codes + recovery
├── operations/
│   ├── schema-migrations-operations.md      applying, rolling-back-ish, audit
│   ├── at-rest-encryption-operations.md     master-key rotation stub (M7b full)
│   ├── bootstrap-credential-lifecycle.md    generation → delivery → consumption
│   └── audit-log-retention.md               class-tier retention; 90d window
└── decisions/
    ├── 0008-permission-check-as-pipeline.md
    ├── 0009-surrealdb-schema-layout.md          (P2 amendment: cross-ref 0015)
    ├── 0010-per-slot-aggregation.md
    ├── 0011-bootstrap-credential-single-use.md
    ├── 0012-forward-only-migrations.md
    ├── 0013-audit-events-class-and-chain.md
    ├── 0014-at-rest-encryption-envelope.md
    └── 0015-type-safe-ownership-edges.md        (sealed marker traits + typed helpers)
```

**Writing conventions** (same as M0; enforced by `doc-links.yml`):

- `<!-- Last verified: YYYY-MM-DD by Claude Code -->` on line 1 of every file.
- Status tags: `[EXISTS]`, `[PLANNED M<n>]`, `[CONCEPTUAL]`.
- Every code-rooted claim links to `modules/crates/…` with file:line. Rationale-heavy claims link to the archived build plan or to a concept doc.
- ASCII diagrams only.
- Docs for a phase land in the **same commit** as that phase's code (like the M0 doc-code-same-commit rule).

---

## Part 7 — CI / CD extensions  `[STATUS: ⏳ pending]`

1. **`rust.yml`**: add a `proptest` job invoking `cargo test -p domain --test '*_props' -- --test-threads 1` with `PROPTEST_CASES=100` (≥ D7's 50 minimum; leaves headroom).
2. **`rust.yml`**: add an `acceptance` job that builds in release and runs `cargo test --workspace --test acceptance_*`.
3. **`spec-drift.yml`**: extend grep set with `R-SYS-*` (bootstrap system flow ids).
4. **`doc-links.yml`**: unchanged; existing rules cover the new `m1/` tree once it's populated.
5. **No new workflow file** unless a genuine need surfaces during P9.

---

## Part 8 — Verification matrix  `[STATUS: ⏳ pending]`

Before declaring M1 done, each row of Part 2's Commitment Ledger maps to a green test.

> **Proptest count unit:** a "proptest file" groups related invariants (typically 2–3 per file). Each invariant is a single `proptest!` function expanded to `PROPTEST_CASES=100` random inputs in CI. So the "≥5 proptest files" target for C2 below ≈ 10–15 invariants × 100 cases ≈ 1,000–1,500 input branches; C3's "≥4 files" ≈ 800–1,200 branches; together with the smaller proptest sets in P1 (hash-chain, crypto) and P2 (repository invariants) they roll up to the workspace-wide **≈2,500 branches per CI run** figure from Part 5.



| # | Commitment | Test / check |
|---|---|---|
| C1 | Graph model counts | `domain/tests/model_counts.rs` asserts 9 / 8 / 37 / 66 enumerated |
| C2 | Permission Check | ≥5 proptest files green + worked-trace byte-match |
| C3 | Auth Request state machine | ≥4 proptest files green |
| C4 | s01 end-to-end | acceptance: `bootstrap_claim_success` |
| C5 | HTTP endpoints | handler tests for 201 / 400 / 403 / 409 |
| C6 | Audit events | unit + proptest on hash-chain continuity |
| C7 | Schema migrations | `store/tests/migrations.rs` |
| C8 | At-rest encryption | `store/tests/crypto_roundtrip.rs` + `missing_key_fails_startup` |
| C9 | Resource Catalogue | exercised inside the acceptance test |
| C10 | Metric surface | acceptance asserts `phi_permission_check_duration_seconds_count > 0` on `/metrics` |
| C11 | Session cookie | handler test: set-cookie on claim; verify on follow-up |
| C12 | CLI | `cli/tests/bootstrap_cli.rs` green |
| C13 | Acceptance harness | all acceptance tests green under `--test-threads 1` |
| C14 | Docs co-located | doc-links CI green; every phase's doc set present |
| C15 | Type-safe ownership edges | Unit tests covering every `Principal`/`Resource` trait impl; ≥6 `trybuild` compile-fail cases in `domain/tests/edge_type_safety/compile_fail/`; integration round-trip through `upsert_ownership` / `upsert_creation` / `upsert_allocation` against real SurrealDB |

**First-review confidence target: ≥ 99 %.** Rubric: one HIGH-severity miss drops ≈15 pp; one MEDIUM ≈5 pp. Target leaves room for at most one stylistic LOW.

---

## Part 9 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** — copy to `phi/docs/specs/plan/build/<8hex>-m1-permission-check-spine.md`. Generate the 8-hex token with `openssl rand -hex 4`. (~2 min)
1. **Reconcile build-plan counts** — update `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` §M1 to say "37 nodes + 66 edges" instead of "31 + 56+". Same commit as the implementation kick-off. (~5 min)
2. **P1** — foundation: types + schema + migrations + audit skeleton + crypto layer + P1 docs. (~3 days)
3. **P2** — type-safe edge foundation (marker traits + typed `Edge::new_*` constructors + `trybuild` compile-fail tests) + repository expansion (incl. typed ownership helpers) + ADR-0015 + P2 docs. (~3 days)
4. **P3** — Permission Check engine + proptests + P3 docs. (~3 days)
5. **P4** — Auth Request state machine + proptests + P4 docs. (~3 days)
6. **P5** — s01 Bootstrap flow + P5 docs. (~2 days)
7. **P6** — HTTP endpoints + session cookie + P6 docs. (~2 days)
8. **P7** — CLI migration + subcommands + P7 docs. (~2 days)
9. **P8** — Web bootstrap page + P8 docs. (~2 days)
10. **P9** — Acceptance harness + ops docs + final doc-links pass. (~2 days)
11. **Re-audit** — independent Explore-agent run against Part 8's matrix; target ≥ 99 %. Remediate LOW findings in the same session.
12. **Tag milestone** — `git tag v0.1-m1` in `phi` submodule; update submodule pointer in parent repo.

**Total estimate: ~3 weeks of focused work (≈22 calendar days).** Still within the build plan's "2–3 weeks" envelope for M1; the extra day covers P2's type-safe-edge foundation (added after P1 self-review).

---

## Part 10 — Critical files  `[STATUS: n/a]`

Will be modified:
- `phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md` (count corrections in §M1)
- `phi/modules/crates/domain/src/{lib,model,permissions,state_machines,repository,audit}.rs` (P1–P4)
- `phi/modules/crates/store/src/{lib,migrations,crypto}.rs` (P1–P2)
- `phi/modules/crates/server/src/{router,state,session,bootstrap,handlers/bootstrap}.rs` (P5–P6)
- `phi/modules/crates/cli/src/main.rs` + new `cli/src/commands/bootstrap.rs` (P7)
- `phi/modules/web/app/bootstrap/page.tsx` + `web/lib/{session,api}.ts` (P8)
- `phi/.github/workflows/rust.yml` (P6)
- `phi/.github/workflows/spec-drift.yml` (R-SYS-* add)
- `phi/config/{default,dev,staging,prod}.toml` (add `master_key` + `bootstrap` sections commented for dev)

Will be created (total ~50 new files):
- `phi/docs/specs/plan/build/<8hex>-m1-permission-check-spine.md` (plan archive)
- `phi/modules/crates/store/migrations/0001_initial.surql` (+ later migrations as needed)
- `phi/modules/crates/domain/src/model/principal_resource.rs` (P2 — sealed `Principal` + `Resource` marker traits per D9/C15)
- `phi/modules/crates/domain/tests/{model_counts,permission_check_props,auth_request_props}.rs` + proptest support
- `phi/modules/crates/domain/tests/edge_type_safety/compile_fail/*.rs` (P2 — `trybuild` cases rejecting wrong-pair endpoint types)
- `phi/modules/crates/server/tests/{bootstrap_handler_test,session_cookie_test}.rs`
- `phi/modules/crates/cli/tests/bootstrap_cli.rs`
- `phi/tests/acceptance/{common/mod,bootstrap_claim}.rs`
- `phi/docs/specs/v0/implementation/m1/README.md`
- `phi/docs/specs/v0/implementation/m1/architecture/*.md` (11 files)
- `phi/docs/specs/v0/implementation/m1/user-guide/*.md` (5 files)
- `phi/docs/specs/v0/implementation/m1/operations/*.md` (4 files)
- `phi/docs/specs/v0/implementation/m1/decisions/*.md` (8 ADRs: 0008–0015)

---

## What stays unchanged  `[STATUS: n/a]`

- Concept docs (`docs/specs/v0/concepts/`) are the source of truth; only the build plan's headline counts are corrected (to match the concept docs). No concept-doc content changes in M1 unless implementation surfaces a real semantic error.
- `phi-core` is consumed as a library; M1 introduces no new `phi-core` dependencies beyond the types already used by CLI's existing demo.
- M0 artefacts (`m0/*`, M0 ADRs 0001–0007, M0 CI workflows, M0 runbook stub) stay as-is; M1 adds adjacent artefacts without rewriting any M0 page.
- The 15 reference layouts stay fixture material; none are consumed in M1 (first consumer is M2+).

---

## Open items (non-blocking, surface during P-whatever)  `[STATUS: n/a]`

Track in P-by-P exec notes, not here:

- Does `phi-server --bootstrap-init` belong in the server binary or a dedicated `phi-admin` binary? Current plan keeps it in the server binary (D1); revisit if it feels noisy.
- Should the Prometheus histogram labels include `org_id`? Likely yes for multi-tenant, but M1 has one org, so it's a latent M2 question.
- Audit shadow-log format: NDJSON vs length-prefixed binary? Default NDJSON (human-greppable); binary is a perf optimisation, not an M1 concern.
