# Build Plan: baby-phi v0.1

> **Legend:**
> - `[PLAN: new]` — part of this fresh build plan
> - `[DOCS: ⏳ pending]` — not yet executed
> - `[DOCS: n/a]` — reference/meta section

> **⚠ Production target.** baby-phi v0.1 will be deployed **for real client projects in production**, not as a demo or internal tool. Every milestone below therefore pays its share of production-readiness work (auth, TLS, backup, observability, deployment packaging, runbooks) — this is not a "build it, then harden it later" plan. Milestone **M7b** is a dedicated hardening pass that verifies the previous milestones produced production-grade artefacts; it is a quality gate, not a fix-up phase.

## Context  `[PLAN: new]` `[DOCS: n/a]`

Concept docs (94% calibrated) and 321 requirements across the fresh-install admin journey are ready. This plan moves to **build**. The user-chosen shape:

- **Three parallel surfaces:** Rust HTTP API + Rust CLI + Next.js (SSR) web frontend. CLI is the scriptable/testable surface; the web UI is the human one. Both consume the same REST API.
- **Spine-first hybrid sequencing.** Permission Check engine + graph storage + System Bootstrap flow + Auth Request state machine land first (the load-bearing spine). Then the 14 admin pages + 5 agent-self-service surfaces land as vertical slices in fresh-install journey order (Phase 1→9).
- **Embedded graph DB:** **SurrealDB** (Rust-native, embeddable, graph-capable, supports RocksDB backend). Replaces the SQLite default from `coordination.md`; the concept doc's "v0 default, revisitable" language covers the change.

Archive location for this and future build plans: `baby-phi/docs/specs/plan/build/`.

## Decisions Captured  `[PLAN: new]` `[DOCS: see Impl column]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Surfaces** | API (Rust/axum) + CLI (Rust/clap) + Web UI (Next.js 14 with SSR). Same REST contracts for both clients. | ⏳ |
| **Sequencing** | Spine-first hybrid. M1 is spine. M2–M5 are journey phases 1–9 as vertical slices. M6 is agent self-service. M7 is system flows + NFRs. M8 is acceptance + polish. | ⏳ |
| **Storage** | SurrealDB embedded (RocksDB backend). Replaces the SQLite recommendation in `coordination.md`; that doc's "v0 default, revisitable" tag authorised the change. See §Storage choice: SurrealDB vs Memgraph below for the rationale. | ⏳ |
| **Workspace layout** | Cargo workspace: `phi-core` (existing), `baby-phi` (existing binary, becomes the CLI), plus three new crates: `baby-phi-domain` (graph model + Permission Check engine + state machines), `baby-phi-server` (axum HTTP layer), `baby-phi-store` (SurrealDB adapter + repository traits). Next.js frontend lives in `baby-phi/web/` with its own package.json. | ⏳ |
| **Testing layers** | Unit: property-based on Permission Check + state machines. Acceptance: the 15 reference layouts become fixture builders; every admin-page Section 11 scenario becomes an integration test. NFR: performance + security invariant tests under `tests/nfr/`. | ⏳ |
| **Observability baseline** | Structured JSON logs; Prometheus-format `/metrics` endpoint; audit event emitter writes to SurrealDB and to a shadow append-only log for recoverability. Per `nfr-observability.md`. | ⏳ |
| **CI/CD** | GitHub Actions: fmt + clippy + unit + acceptance + NFR smoke. Release builds tag binaries. | ⏳ |

## phi-core leverage — standing commitments  `[PLAN: new]` `[DOCS: [EXISTS] — see checklist]`

phi-core is the source-of-truth library; baby-phi is a consumer. Every overlapping surface reuses phi-core directly or wraps it — never re-implements (per [`baby-phi/CLAUDE.md §phi-core Leverage`](../../../../../CLAUDE.md)). **Every milestone's detailed planning session MUST apply the four-tier enforcement model below** — treating the CI grep linter as sufficient is explicitly rejected as it cannot catch miss-leverage.

| Tier | Mechanism | Catches | Applies |
|---|---|---|---|
| 1 — CI-enforced | [`scripts/check-phi-core-reuse.sh`](../../../../../scripts/check-phi-core-reuse.sh) + compile-time coercion tests + schema-snapshot forbidden-key tests | Duplicated phi-core types; accidental local-struct swaps; wire-shape drift | Every PR, automatically |
| 2 — Structural | Q1/Q2/Q3 per-phase audit; deliverable-level phi-core tags; positive close-audit grep assertions | Miss-leverage, reductive "None" answers, silent omissions | Every phase `### phi-core leverage` subsection + `### Confidence check` |
| 3 — Governance record | ADRs for non-trivial reuse decisions; [`phi-core-reuse-map.md`](../../v0/implementation/m3/architecture/phi-core-reuse-map.md); `#### Carryovers from M<n>` sections propagating leverage implications forward | Re-litigation of decided boundaries; milestone-to-milestone drift | Milestone close; carryover review at next milestone planning |
| 4 — Independent retrospective | Re-audit agent at milestone close (M2/P8 precedent; M3/P6 planned); post-milestone update to the reuse map + checklist `§Backstory` | Aggregate drift, coordinate failures across phases | Once per milestone |

**Durable references** (consult at every milestone open):
- [phi-core leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) — the Q1/Q2/Q3 discipline + tier descriptions + known gaps.
- [phi-core reuse map](../../v0/implementation/m3/architecture/phi-core-reuse-map.md) — durable type-level wrap table.
- [ADR-0023 — inherit-from-snapshot](../../v0/implementation/m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — template for non-trivial reuse ADRs.

**Known unaddressed gaps** (candidates for future CI hardening; see checklist for details):
- No PR-level phi-core gate (a checklist in the PR template) — candidate for M7b.
- No automated miss-leverage detection (grep for baby-phi structs whose field-set mirrors a phi-core type) — candidate for M6+.

M3 Backstory: P3's plan labelled leverage as "None" on reductive reasoning; user pushback at the P2→P3 handoff surfaced the slip. Root cause was single-question audit ("does this phase leverage phi-core?"); structural fix was the Q1/Q2/Q3 split. Documented so M4+ planners don't repeat the pattern.

## Production-readiness commitments  `[PLAN: new]` `[DOCS: ⏳ pending]`

Every item below is an explicit deliverable mapped to a milestone. None are "we'll get to it later."

| Concern | Commitment | Milestone |
|---------|------------|-----------|
| **Authentication (beyond bootstrap)** | Real human login — OAuth 2.0 (PKCE) against configurable IdP (Google/Microsoft/Okta) + local-password fallback for dev. MFA supported when the IdP provides it. Session tokens are server-signed cookies (SSR-friendly) with sliding expiry. LLM-agent authentication uses short-lived machine tokens minted by the platform admin. | M3 (first non-bootstrap users exist) |
| **Transport security (TLS)** | TLS 1.3 everywhere. Production deployment uses a reverse proxy (nginx/Caddy) terminating TLS; baby-phi-server also supports native TLS via `axum-rustls` for simple deploys. Automatic cert renewal via Let's Encrypt where applicable. No plaintext HTTP served in production. | M0 (server skeleton) |
| **At-rest encryption** | SurrealDB data files encrypted at rest. v0.1 uses AES-256 with a master key loaded from the credentials vault OR environment-injected via the deployment's secret manager (Kubernetes Secrets, AWS Secrets Manager, etc.). Secret/credential entries in the vault are additionally wrapped with per-entry keys for defence in depth. | M1 (spine) + M7b (hardening verification) |
| **Backup & restore** | Automated daily `surreal export` dumps to off-site storage (S3/GCS) with 30-day retention. Point-in-time recovery up to 24h via WAL-shipping. **Tested restore drill** as part of M7b — not just "it's configured." Documented runbook. | M7 (tooling) + M7b (drill) |
| **Rate limiting + abuse** | Per-endpoint rate limits (tower-governor or equivalent). Per-tenant quotas. Request size caps. Per-principal concurrent-session caps already enforced by `parallelize` + Permission Check — extended here to per-IP/per-token rate limiting on the public API surface. | M7b |
| **Audit-log tamper resistance** | Audit events written to SurrealDB with a hash-chain (each event references the hash of the previous one within its org's scope). Offsite stream to append-only S3/GCS bucket with object-lock enabled. Tamper detection is a simple hash-chain walk. | M7 + M7b |
| **Observability (production-grade)** | OpenTelemetry traces (OTLP exporter); logs shipped via journald/Vector/Loki; Prometheus scraped by the deployment's monitoring stack. On-call paging integration is a runbook concern, not a code concern — paging targets configured per-deployment. SLO dashboard (Grafana JSON) shipped with the release. | M7 + M7b |
| **Health checks** | Separate `/healthz/live` (process alive) and `/healthz/ready` (DB reachable, migrations applied, dependencies healthy). Used by orchestrators for rolling deploys. | M0 (skeleton) + M7b (production-grade semantics) |
| **Deployment packaging** | Dockerfile (multi-stage, non-root user, minimal base). Docker Compose for local dev. Kubernetes manifests (Deployment + Service + PersistentVolumeClaim + ConfigMap + Secret) shipped as reference. Helm chart is a v0.2 conversation. | M0 (Dockerfile) + M7b (k8s manifests, compose) |
| **Configuration management** | All runtime config via env vars + a layered config file (dev / staging / prod). Secrets NEVER in config files; always injected via environment. 12-factor-compatible. | M0 |
| **Schema migrations** | Versioned SurrealDB schema with forward-only migrations (`baby-phi-domain::migrations`). Every migration tested in CI against a representative dataset. Migration runs on startup; failed migrations refuse to serve (fail-safe). | M1 (first migration) + ongoing |
| **Release process** | SemVer. `CHANGELOG.md` updated per release. Staging env runs the main branch continuously; production pinned to tagged releases. Rollback strategy: the previous N Docker images retained, schema migrations reversible where possible or accompanied by compensating migrations. | M8 + ongoing |
| **Security scanning** | `cargo audit` in CI (RustSec advisories). `cargo deny` for licence / supply-chain gates. `npm audit` on the web tree. Dependabot enabled. SAST via `cargo clippy -W clippy::pedantic` on critical crates. | M0 (CI gates) + M7b (full scan) |
| **Load testing** | `tests/nfr/load/` uses a k6 or goose script against the NFR-performance targets (100 Permission Checks/s sustained, etc.). Run in staging pre-release. | M7b |
| **Chaos testing** | Basic failure-injection suite: kill DB mid-session, simulate DB-full, simulate network partition between baby-phi-server and the embedded DB (when we move to standalone-server tier). Covers SurrealDB backup/restore and retries. | M7b |
| **GDPR / data-subject rights** | `DELETE /api/v0/agents/{id}?right_to_erasure=true` removes user-owned content respecting Auth Request retention (some audit records legally must survive erasure; the API returns a report of what was erased vs retained with legal justification). | M7b |
| **Runbook** | `docs/ops/runbook.md` covers: deploy, upgrade, rollback, backup, restore, incident response, known issues + workarounds. Written *during* M7b, not after. | M7b |
| **Architecture diagrams** | Sequence diagrams for bootstrap, org creation, session launch, memory extraction. Component diagram of the three-crate workspace. Published in `docs/architecture/`. | M7b |
| **Security invariants as property tests** | The 17 invariants in `nfr-security.md` are every one of them wired to a proptest that runs in CI. Full coverage is a release gate. | M7 + M7b |

### Non-negotiable release gates (M8)

Before v0.1 ships, the following must be green:

1. Every admin page's acceptance scenarios pass.
2. Every NFR-security property test passes.
3. Load test meets or exceeds NFR-performance targets at 1.5× headroom.
4. Backup & restore drill succeeds end-to-end on a populated DB.
5. Security scan shows no high- or critical-severity vulnerabilities unpatched.
6. Runbook reviewed by a second engineer (or the user).
7. Staging environment has run the release candidate for ≥72h without unacknowledged alerts.

### Staged beyond v0.1 (called out explicitly so they don't silently fall off)

Not in v0.1 — explicit follow-on work:

- KMS integration (AWS KMS / GCP KMS / HashiCorp Vault) for master-key management. v0.1 uses env-var-injected keys; KMS is a v0.2 upgrade.
- Cross-region replication via SurrealDB TiKV backend (v0.1 is single-region, single-node).
- Zero-downtime deploys with active schema migrations (v0.1 has brief maintenance windows on migration).
- Compliance certifications (SOC 2, ISO 27001). v0.1 produces audit-grade artefacts; formal certification is a business process, not a code one.
- Multi-tenant physical isolation (separate DB per tenant). v0.1 uses logical isolation via the org + grant model; physical isolation is a scale decision.

## Architecture sketch  `[PLAN: new]` `[DOCS: n/a]`

```
baby-phi/ (workspace root)
├── phi-core/                       # existing library; agent loop, providers, tools
├── baby-phi-domain/   (NEW)        # graph model + Permission Check + state machines
├── baby-phi-store/    (NEW)        # SurrealDB adapter, repository traits
├── baby-phi-server/   (NEW)        # axum HTTP API, endpoint handlers
├── baby-phi/          (existing)   # CLI binary, clap subcommands hitting the API
├── web/               (NEW)        # Next.js 14 + App Router + SSR
├── docs/specs/                     # existing concepts + requirements (source of truth)
├── tests/                          # acceptance + NFR integration tests
└── Cargo.toml                      # workspace manifest
```

**Dependency flow (strict, downward only):**
```
baby-phi (CLI)   ─┐
baby-phi-server ─┼─▶ baby-phi-domain ─▶ baby-phi-store ─▶ SurrealDB
web (Next.js)   ─┘                      (uses phi-core for agent/session types)
```

The domain crate is the shared library both CLI and server link against directly — CLI can hit the domain without going through HTTP for local operations, but for consistency + single-code-path the v0.1 CLI routes through the local-loopback HTTP client.

## Storage choice: SurrealDB vs Memgraph  `[PLAN: new]` `[DOCS: n/a — rationale]`

Both were on the table under "embedded graph DB." A compact comparison plus why we picked SurrealDB:

| Dimension | **SurrealDB** | **Memgraph** |
|-----------|----------------|----------------|
| **Embeddability** | Runs embedded in-process via the `surrealdb` Rust crate. Can also run as a separate server when needed. | Runs as a separate server process (C++ binary). No first-class embedded story for Rust. |
| **Language** | Written in Rust. Zero-FFI integration into `baby-phi-store`. | Written in C++. Rust client crates talk over Bolt protocol (network hop even for local dev). |
| **Query language** | SurrealQL — SQL-flavoured with native graph traversal (`SELECT ... FROM ->edge->node`, `RELATE`). | Cypher (Neo4j-compatible). |
| **Data model** | Multi-model: document fields on nodes + graph edges + time-series. Natural fit for our hybrid ontology (AgentProfile has nested structs; audit events are time-series; grants + edges are graph). | Graph-native only. Document-like fields encoded as property maps. |
| **Backend storage** | In-memory, RocksDB (embedded), or TiKV (distributed). RocksDB is the v0 pick. | In-memory primary with disk snapshots; not configurable the same way. |
| **Maturity** | Younger (v2.0 released 2024, stable); rapidly developing ecosystem. | Older, battle-tested in graph-analytics workloads. |
| **Graph analytics (PageRank, community detection, centrality)** | Basic; more work needed for advanced algorithms. | Strong out of the box. |
| **Operational overhead** | One binary, one process, zero external dependencies. | Requires a separate process; container or system service needed even for local dev. |

### Why SurrealDB wins for baby-phi v0.1

1. **Embedded = simpler ops.** SurrealDB loads into `baby-phi-server`'s process directly. Memgraph would force a second process even for local dev, turning docker-compose from optional into mandatory.
2. **Rust-native, zero FFI.** `surrealdb = "2.x"` in `Cargo.toml` is the whole integration — no Bolt protocol, no network hop, no serialization tax.
3. **Multi-model matches the ontology.** The baby-phi data model is genuinely hybrid: Agent nodes carry nested phi-core structs (`AgentProfile`, `ModelConfig`, `ExecutionLimits`) — that's document shape — plus participate in edges (`MEMBER_OF`, `HAS_LEAD`, `DESCENDS_FROM`) — that's graph shape. Audit events are time-series. Memgraph would force us to encode the document parts as property maps and build a separate time-series store; SurrealDB handles all three natively.
4. **Future migration path is open.** SurrealQL is close enough to SQL that migrating to any SQL-native store is mostly query rewrites. Migrating OUT of Memgraph/Cypher would be a larger effort.
5. **v0 scale doesn't need Memgraph's strengths.** Graph analytics (PageRank, centrality, community detection) are Memgraph's standout features. baby-phi v0 doesn't need them — the graph traversal we need is "walk `DESCENDS_FROM` to the root," "list grants that match this selector," "find all sessions tagged `X`" — all trivial in SurrealQL.

### Tradeoffs (honest)

- **SurrealDB is younger** and less battle-tested at scale than Memgraph. v0's scale target is "one org, ~50 agents, ~10 projects" — comfortably within SurrealDB's proven envelope. If usage patterns at v1 reveal we need Memgraph's analytics, migration is a v1 decision, not a v0 blocker.
- **SurrealQL is a custom language.** The team has to learn it. It's close enough to SQL that the ramp is small; estimated learning cost is a day, not a week.
- **Fewer third-party tools** (visualizers, GUIs) than Memgraph. Not critical for v0 since our admin UI is the primary inspection surface.

### If this choice turns out wrong

The `baby-phi-store` crate is the sole adapter. Swapping SurrealDB for Memgraph (or any other store) is a one-crate rewrite plus query-language migration. The domain crate is storage-agnostic — it talks to a `Repository` trait, not to SurrealDB directly. This is a deliberate safety valve.

### Scaling path — from embedded to distributed

"Embedded" is the v0.1 starting point, not a dead-end. SurrealDB's architecture lets you move up tiers **without changing query code** — only the connection string changes. Concretely:

| Tier | When | How | Query code change? |
|------|------|-----|--------------------|
| **Embedded + RocksDB** (v0.1) | Single-tenant, ~100k nodes / ~1M edges / ≤100 concurrent sessions. baby-phi-server + DB in one process. | `Surreal::new::<RocksDb>("data/baby-phi.db")`. Comfortable up to hundreds of GB of data on a single machine. | — |
| **Standalone SurrealDB server** | baby-phi-server needs to scale horizontally, or you want to separate DB lifecycle from app lifecycle. | Run `surreal start --bind 0.0.0.0:8000 file:data/baby-phi.db`; switch the client to `Surreal::new::<Client>("ws://host:8000")`. | **None.** Same SurrealQL. |
| **SurrealDB cluster with TiKV** | Multi-region, HA, or data size outgrows a single node. | Swap RocksDB for TiKV backend; deploy SurrealDB cluster. | **None.** Same SurrealQL. |
| **Leave SurrealDB entirely** | SurrealDB itself is the bottleneck or wrong tool. | Rewrite `baby-phi-store` crate to target a different DB (Postgres, Neo4j, Memgraph, etc.). Domain crate unchanged. | Full query rewrite. Mitigated by the Repository trait. |

### Data migration — between SurrealDB instances

Three mechanisms, in order of increasing portability:

1. **RocksDB file copy.** Fastest; moves the entire DB between hosts of the same SurrealDB version. Suitable for same-version upgrades or host moves.
2. **`surreal export` / `surreal import`.** Dumps the DB as SurrealQL scripts (CREATE statements + RELATE edges). Portable across versions that share the export format. Good for major-version upgrades, disaster recovery, and migrating between embedded and standalone.
3. **HTTP API streaming.** `SELECT * FROM <table>` in batches, re-insert at the destination. Universal and auditable; slow for very large DBs. Also the format used when migrating OUT of SurrealDB entirely.

### Honest weak spots on scale

These are SurrealDB's current weaknesses. They are not blockers for v0.1 but are worth knowing:

- **Backup/restore tooling is less mature** than Postgres/MySQL equivalents. You will write your own backup scripts; plan ~1 day of work during M7.
- **Smaller operational knowledge base.** Fewer community runbooks for "SurrealDB at scale"; more pioneering required past the embedded tier.
- **Query optimizer is less mature.** For complex multi-hop traversals, expect to hand-tune queries. Mitigated in v0 because our traversal patterns are shallow (authority chains typically ≤6 hops).
- **TiKV backend is newer** than the RocksDB embedded backend. If the scaling path pushes you to TiKV, expect more operational investment than the Postgres-cluster equivalent.

### Practical implication for baby-phi

For v0.1 the embedded tier is the right choice — low operational overhead, single-binary deploys, fast local dev. The **migration commitment** we're making: if baby-phi usage grows past the embedded tier, we move to the standalone-server tier first (zero query code change, day of ops work). Only if SurrealDB itself is the wrong tool do we exercise the Repository-trait safety valve. That's a spectrum from "free" to "bounded effort" at every step, which is what you want in a v0 storage choice.

## Milestones (build order)  `[PLAN: new]` `[DOCS: ⏳ pending]`

### M0 — Project scaffolding (≈1 week)

- **First action: archive this plan verbatim** to `baby-phi/docs/specs/plan/build/<random>-build-plan-v01.md` (8-hex-char token). Creates the `plan/build/` folder if it doesn't exist. Matches the convention used for prior plans (`plan/d95fac8f-…`, `plan/54b1b2cb-…`, `plan/requirements/e2781622-…`).
- Cargo workspace set up with 4 new crates (`baby-phi-domain`, `baby-phi-store`, `baby-phi-server`, `web/`).
- SurrealDB embedded (via `surrealdb` Rust crate), RocksDB backend, healthcheck endpoint.
- `/metrics` skeleton via `axum-prometheus` or equivalent.
- Next.js 14 scaffold with App Router + SSR, Tailwind set up, auth placeholder (cookie + session stub; real auth comes with M1 bootstrap).
- GitHub Actions: fmt + clippy + tests on every PR; `RUSTFLAGS="-Dwarnings"` per phi-core convention.
- Documentation-alignment CI check: verifies every `requirements/admin/*.md`'s `R-ADMIN-*` IDs can still be grepped (regression guard on spec drift).

### M1 — Permission Check spine (≈2–3 weeks)

**Goal:** every subsequent milestone builds on a rock-solid Permission Check + Auth Request engine.

- **Graph model:** Rust types for all 9 fundamentals + 8 composites + 37 nodes + 66 edges from `concepts/ontology.md` (count corrected from the earlier "31 + 56+" approximation during M1 pre-audit). SurrealDB schema (tables + indices).
- **Permission Check engine** in `baby-phi-domain` — the 6-step formal algorithm from `permissions/04`. Property-based tests over randomly-generated grant sets.
- **Auth Request state machine** per `permissions/02` — atomic per-slot transitions, per-resource aggregation, forward-only revocation. Property tests over the state diagram.
- **System Bootstrap flow (s01):** bootstrap credential generation at install, single-use consumption, platform admin materialisation.
- **First user-visible endpoint:** `POST /api/v0/bootstrap/claim` (admin page 01, R-ADMIN-01-W1). CLI: `baby-phi bootstrap claim`. Web UI: Phase 1 page.
- **First acceptance test:** fresh install → claim → platform admin exists with the `[allocate]` on `system:root` grant, traceable to the Bootstrap Adoption Auth Request.

### M2 — Platform setup, Phase 2 (≈2 weeks)

Admin pages 02–05 (model providers, MCP servers, credentials vault, platform defaults).

- Resource Catalogue precondition (Permission Check Step 0) is now exercised by every write.
- Audit event pipeline writing alerted-class events for sensitive changes.
- CLI + API + Web UI for each page. Acceptance scenarios from pages 02–05 become integration tests.

### M3 — Organization creation + dashboard, Phase 3–4 (≈2 weeks)

Admin pages 06 (org creation wizard) and 07 (org dashboard).

- Two-system-agents (`memory-extraction-agent`, `agent-catalog-agent`) provisioned per `concepts/system-agents.md` at org creation time.
- Adoption Auth Requests for enabled Authority Templates auto-materialised + auto-approved by the CEO.
- Wizard multi-step with autosaving draft. Wizard produces a complete org matching the 10 reference layouts.
- Dashboard shows agents/projects/Auth-Requests/alerts/budget per `admin/07`.

### M4 — Agents + Projects, Phase 5–6 (≈2–3 weeks)

Admin pages 08–11. Auto-creates inbox/outbox composites on agent creation (s03). OKRs embedded on Project nodes. Shape-A + Shape-B (co-owned) project flows.

- Template A fires (s05) on `HAS_LEAD` edge creation.
- Auth Requests for Shape B project creation route through two org admins per `admin/10-W3`.
- `parallelize` field enforced at session-start time.

#### Carryovers from M3 — must-pick-up at M4 detailed planning

These items were deliberately **deferred** at M3/P5 close (org dashboard). They sit at the M3→M4 boundary — the M3 dashboard surfaces them as zero-valued fields or stubbed behaviour today, and M4's agent-roster + project-creation work fills them in. Pinning here so the M4 detailed planning session picks them up rather than re-discovering during execution.

- **C-M4-1 — `ViewerRole::ProjectLead` filtering on the org dashboard.** M3/P5 implements only `Admin` (CEO) and `Member` (any org member). The `ProjectLead` variant exists on the enum with a `snake_case` wire form (`"project_lead"`) so the M3→M4 API contract stays stable, but the orchestrator today never returns it. Per `admin/07-organization-dashboard.md §7` + §11-Scenario-2, a project lead should see a **filtered dashboard**: project-scoped `AgentsSummary` (only agents in their project), their led projects in `ProjectsSummary`, no `PendingAuthRequests` tile (they don't approve at org level), no `AlertedEventsCount` tile (no `[read]` on org audit log), and "View Projects" instead of "Create Project" in CTA cards. **Decision needed at M4 planning**: the `resolve_viewer_role` helper in `server/src/platform/orgs/dashboard.rs` needs (a) a `HAS_LEAD` edge walk to detect project-lead membership (the edge variant exists since M3/P1 but no writes happen yet) and (b) the filtered branches implemented. **Files affected at M4:** `server/src/platform/orgs/dashboard.rs::resolve_viewer_role` + filter helpers, `modules/web/app/(admin)/organizations/[id]/dashboard/DashboardClient.tsx` (panel visibility already gates on `viewer.can_admin_manage`; will need `role === "project_lead"` filters), new acceptance scenarios in `acceptance_orgs_dashboard.rs` matching `admin/07 §11-Scenario-2`. See M3 plan §P5 "Viewer-role resolution" subsection + `m3/architecture/org-dashboard.md`.

- **C-M4-2 — `AgentsSummary` sub-buckets for role-differentiated humans.** M3/P5 exposes `{ total, human, llm }`. The requirements doc (`admin/07-organization-dashboard.md §4` + §R2) prescribes `{ Human, Intern, Contract, System }` — a 4-way split that needs an `AgentRole` discriminator on the `Agent` node. `AgentKind::Human` today has no sub-type. **Decision needed at M4 planning**: either (a) add `Agent.role: Option<AgentRole>` enum field with `Employee / Intern / Contract` (→ migration 0004 + repo surface + wizard page 09 role picker), or (b) treat role as a pure view layer concept derived from grant/tag metadata. Option (a) is more coherent with governance semantics (the role drives approval-chain eligibility for Shape B projects). **Files affected at M4:** `domain/src/model/nodes.rs::Agent` (add `role`), `store/migrations/0004_*.surql` (add column + ASSERT), `server/src/platform/orgs/dashboard.rs::count_agents` (expand to 4 buckets), `modules/web/lib/api/orgs.ts::AgentsSummaryWire` (add fields), `modules/web/app/(admin)/organizations/[id]/dashboard/DashboardClient.tsx::AgentsSummary` (render the 4 counters). See M3/P5's "Deviations from requirements doc §10" table in `m3/architecture/org-dashboard.md`.

- **C-M4-3 — `ProjectsSummary.{shape_a, shape_b}` breakdown.** M3/P5 returns `{ active: 0, shape_a: 0, shape_b: 0 }` because the `Project` struct isn't persisted yet. `list_projects_in_org` returns `Vec<ProjectId>` at M3; the SurrealDB impl returns `vec![]` (empty stub) until the project table lands. **Decision needed at M4 planning**: when `Project { id, owning_org, shape: ProjectShape, okrs, leads, ... }` lands in `domain/src/model/nodes.rs` + migration 0004, the dashboard's `ProjectsSummary` fill logic must be updated to count by shape. The contract's `shape_a` / `shape_b` field names must match whatever `ProjectShape` serde form M4 settles on (current pre-decision: `shape_a` / `shape_b`). **Files affected at M4:** `domain/src/model/nodes.rs::Project` (new), `domain/src/repository.rs::list_projects_in_org` (return type changes from `Vec<ProjectId>` to `Vec<Project>` — coordinated change with in-memory + SurrealDB impls), `server/src/platform/orgs/dashboard.rs::ProjectsSummary` fill logic.

- **C-M4-4 — `HAS_LEAD` edge writes at project creation.** Related to C-M4-1: the `HasLead { id, from: ProjectId, to: AgentId }` edge variant exists since M3/P1 but has **zero** production writes today. Project creation (page 10, Shape A + Shape B) is where the edge is persisted. **Decision needed at M4 planning**: the project-creation compound tx (matching M3's `apply_org_creation` shape) must include `RELATE project -> has_lead -> agent` statements. The edge is also the trigger for Template A fires (s05) — scheduled for M5 but listed under M4's active bullet above. **Files affected at M4:** project-creation handler + repo method (new), `store/migrations/0004_*.surql` (if the edge needs a new index), `domain/src/model/edges.rs` (already exists, no change expected). See M3 plan §P1 commitment C2's `HasLead` addition.

- **C-M4-5 — `list_projects_in_org` return-type migration.** Tied to C-M4-3. The trait signature today is `fn list_projects_in_org(&self, org: OrgId) -> Vec<ProjectId>`; the M3/P5 dashboard uses only `.len()`. When M4 introduces the `Project` struct with persisted fields (shape, name, OKRs, leads), callers (including the dashboard) will want the full struct. **Decision needed at M4 planning**: change trait signature to `Vec<Project>` in one coordinated change (domain trait + in-memory impl + SurrealDB impl + every caller). The M3/P5 dashboard will absorb this via a one-line refactor (count shape variants). Documenting here so it's not a surprise.

- **C-M4-6 — CEO alerted-events visibility when audit-log grant model matures.** M3/P5 shows the `AlertedEventsCount` tile only when `viewer.can_admin_manage` is true (i.e. the CEO). Per `admin/07 §7`, the real gate is "holder of `[read, list, inspect]` on `control_plane_object:audit_log`". At M3 the CEO is the only holder at org creation, so the heuristic is correct. At M4 when agent-roster management introduces additional authority delegations (e.g. Template C delegations), additional holders may emerge, and the dashboard filter should query grants explicitly rather than pattern-match on CEO status. **Decision needed at M4 planning**: replace `viewer.can_admin_manage` with a grant-walk helper (`viewer_has_read_on(audit_log_resource)`). Low-risk deferral — M3's heuristic produces identical behaviour in the absence of the M4 delegation surface.

**Why these belong at M4, not later**: M4 is the first milestone to introduce Project persistence + agent-role differentiation + `HAS_LEAD` edge writes + wider audit-log grant delegation. Waiting would leave the M3 dashboard with stubbed counters (safe but misleading) for longer than necessary, and creates drift between the requirements doc (`admin/07`) and the shipped surface.

**phi-core leverage implications for M4 detailed planning**: all six carryovers are **baby-phi-native** (governance-plane extensions: role enums, project shapes, grant walks, dashboard projections). None introduce new phi-core transit. The existing `DashboardSummary` wire-shape strip (no `defaults_snapshot`) continues to hold; M4 should re-run the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) Q1/Q2/Q3 walk against each added deliverable and specifically assert that the `AgentRole` enum, `Project` struct, and `ProjectShape` discriminator have no phi-core counterpart (confirmed — phi-core has no HR/role concept and no project/OKR primitive). Positive-grep at M4 close: `grep -En '^use phi_core::' server/src/platform/orgs/dashboard.rs` stays at **0 lines** after the M4 rewrite.

### M5 — Template adoption, system agents config, first session — Phase 7–9 (≈2 weeks)

Admin pages 12–14. Completes the fresh-install journey.

- `memory-extraction-agent` implementation (s02) subscribes to `SessionEnd` events.
- `agent-catalog-agent` implementation (s03) subscribes to edge changes.
- First session launch preview runs the full Permission Check (Steps 0–6) per `admin/14-R3`.
- Post-session verification checklist (page 14, N4) confirms memory extraction fired and catalog updated.

#### Carryovers from M3 — must-pick-up at M5 detailed planning

These items were deliberately **deferred** at M3/P4 close (org creation). They fall naturally into M5's first-session + template-firing scope; pinning them here so the M5 detailed planning session picks them up rather than re-discovering them during execution.

- **C-M5-1 — `Template` graph node persistence at adoption time.** M3/P4 does **not** persist `Template { id, name, kind, created_at }` nodes because the existing `template.name` UNIQUE INDEX (migration 0001) would collide when two orgs adopt the same kind (e.g. both orgs adopt Template A → two rows with `name = "template:a"`). Adoption today carries kind info only via the AR's resource URI (`org:<id>/template:<kind>`). **Decision needed at M5 planning**: either (a) rename the template node per org (`template:<kind>:org:<id>`) and keep the UNIQUE INDEX as-is, or (b) collapse to one shared Template node per kind by changing the UNIQUE INDEX to `(kind)` and making the node a platform-level pattern, or (c) drop the UNIQUE INDEX entirely and allow multiple rows per `(kind, org)` pair. Option (b) aligns with the "Template is a reusable pattern, adoption is the per-org act" semantic but requires a migration + a back-reference mechanism for `auth_request.provenance_template`. **Files affected at M5:** `modules/crates/store/migrations/0004_*.surql`, `server/src/platform/orgs/create.rs` (re-enable Template node creation in the compound tx), `domain/src/model/nodes.rs::Template` (may need `adopted_by_org: Option<OrgId>` + `adopted_at: Option<DateTime<Utc>>` fields per migration strategy). See M3 plan §P4 "Template node persistence deferral" subsection + `m3/architecture/org-creation.md`.

- **C-M5-2 — `UsesModel` edge wiring at session launch.** M3/P4 does **not** create `UsesModel` edges from system agents (or any agent) to the org's chosen `ModelRuntime`. Root cause: the `uses_model` relation table from migration 0001 is typed `FROM agent TO model_config`, pointing at the vestigial `model_config` table. M2/P6 introduced the live `model_runtime` table as the actual home for phi-core `ModelConfig` wraps; SurrealDB's `TYPE RELATION` constraint rejects edges with a mismatched `TO`. Today M3 persists `Organization.default_model_provider: ModelProviderId` (an id); the dashboard reads it. At session launch the invoker needs an agent→runtime binding. **Decision needed at M5 planning**: migrate `uses_model` to `FROM agent TO model_runtime` (add `DEFINE TABLE uses_model TYPE RELATION FROM agent TO model_runtime;` in migration 0004, remove / archive the old `model_config` table and its `uses_model` schema). Then wire `UsesModel` in M5's session-launch handler — at invoke time the agent's owning org's `default_model_provider` drives the edge (or a per-agent override once per-agent nodes are introduced). Per ADR-0023 the `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig` remain read-through-snapshot at M5 unless a real per-agent override is required. **Files affected at M5:** `modules/crates/store/migrations/0004_*.surql`, `domain/src/model/edges.rs` (no enum change — `UsesModel` variant already exists), `server/src/<session-launch>/...` (new surface at M5), session-launch acceptance tests must assert each spawned `phi_core::Session` has a resolvable `ModelRuntime` via the edge. See M3 plan §D12 / ADR-0023 + §P4 "UsesModel edge wiring" subsection.

**Why these belong at M5, not later**: both are blocking for session launch (page 14). `C-M5-1` is required because M5's first-session path will fire adoption-driven grants via `provenance_template`, which today dangles. `C-M5-2` is required because the session launcher must resolve "which model config does this agent invoke?" — without the edge, the resolution path is a string id lookup, which is the exact two-source-of-truth problem the phi-core reuse mandate wants to avoid.

**phi-core leverage implications for M5 detailed planning**: both carryovers bear on transit of `phi_core::ModelConfig` (via `ModelRuntime.config`). The M5 plan's `### phi-core leverage` subsections (Q1/Q2/Q3 per the M3 [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md)) must explicitly walk these when the session-launch handler's payload is defined.

### M6 — Agent self-service surfaces (≈2 weeks)

5 pages under `requirements/agent-self-service/`:

- a01 inbox/outbox (tool surface for LLM agents; web-UI rendering for Humans).
- a02 Auth Requests (inbound/outbound).
- a03 Consent records.
- a04 My work.
- a05 My profile + grants (with authority-chain traversal per NFR-observability R6).

### M7 — Remaining system flows + NFR wiring (≈2 weeks)

- s04 full state-machine observability (was scaffolded in M1; full audit + metrics now).
- s05 template-adoption grant fires (wired in M4; broaden to all 5 templates).
- s06 periodic triggers — retention archival, secret rotation reminders, heartbeat, token-budget snapshot.
- NFR observability: audit event schema finalised; Prometheus metrics list from `nfr-observability.md` implemented; OpenTelemetry traces wired.
- NFR performance pass: measure p95/p99 against `nfr-performance.md` targets; optimize hotspots.
- NFR security: property tests for the 17 security invariants in `nfr-security.md`.
- Backup tooling: scheduled `surreal export` with off-site upload; restore script.
- Audit log hash-chain: every audit event carries the hash of its predecessor within its org's scope.

### M7b — Production-readiness hardening (≈2–3 weeks)  `[PLAN: new]` `[DOCS: ⏳ pending]`

**Dedicated hardening milestone. The whole milestone is a quality gate — every item below must be green before M8.**

- **Backup & restore drill** — populate a DB mirroring a realistic org, run backup, destroy primary, restore from backup, verify full state equivalence. Documented.
- **Load test** in staging — goose/k6 script runs the NFR-performance targets at 1.5× headroom for 30 minutes; p95/p99 latency captured in the release notes.
- **Chaos tests** — kill-DB-mid-session, DB-full, network-partition-to-standalone-DB. Each has an expected behaviour spec; system matches.
- **Security scan pass** — `cargo audit`, `cargo deny`, `npm audit`, Trivy on the Docker image. All high/critical resolved.
- **Security invariants green** — the 17 invariants from `nfr-security.md` each have a passing proptest; CI gates on them.
- **Full OpenTelemetry wiring** — traces flow end-to-end for a bootstrap → org-create → session-launch scenario. Sample JSON of the trace is checked in as a fixture.
- **Deployment artefacts** — Dockerfile (multi-stage, non-root, minimal base), docker-compose.yml for local dev, Kubernetes reference manifests (Deployment + Service + PVC + ConfigMap + Secret + Ingress + HorizontalPodAutoscaler).
- **Real auth wired** — OAuth 2.0 PKCE against a test IdP; local-password path for dev; session cookie handling hardened (Secure, HttpOnly, SameSite=Lax, sliding expiry).
- **TLS configured** — native `axum-rustls` path tested; reverse-proxy-with-TLS-termination documented as the recommended production pattern.
- **At-rest encryption active** — SurrealDB data files encrypted with AES-256; key loaded from environment; key-rotation procedure documented.
- **Audit log off-site stream** — audit events stream to an append-only S3/GCS bucket with object-lock (tamper-evident); retention policies match `nfr-observability.md`.
- **Rate limiting** — per-endpoint + per-principal rate limits enforced; tested against abuse scenarios.
- **GDPR right-to-erasure** — API and test coverage for data-subject deletion, preserving audit records that must survive erasure.
- **Runbook written and reviewed** — `docs/ops/runbook.md` covers deploy, upgrade, rollback, backup, restore, 5 common incident scenarios, known issues.
- **Architecture diagrams** — sequence diagrams for 4 critical flows + component diagram in `docs/architecture/`.
- **Staging environment continuous for ≥72h** on the release candidate with no unacknowledged alerts before M8 release.

### M8 — Release v0.1 (≈1 week)

**Pre-release gates** (all must be green — see §Non-negotiable release gates in Production-readiness commitments):

1. Every admin page's Section 11 acceptance scenarios pass.
2. Every NFR-security property test passes.
3. Load test meets NFR-performance targets at 1.5× headroom.
4. Backup & restore drill succeeds on populated DB.
5. Security scan: zero high/critical unpatched.
6. Runbook reviewed by a second engineer.
7. Staging has run the RC for ≥72h clean.

**Release work:**

- End-to-end "hello world" smoke test scripted (fresh install → bootstrap → create minimal-startup → launch session → memory extracted).
- The 15 reference layouts become fixture builders used across the test suite.
- Documentation cross-check: every concept rule cited in a requirement is exercised by at least one test; coverage report in release notes.
- `CHANGELOG.md` for v0.1.
- Docker image built, signed, pushed.
- Git tag `v0.1.0`.

**Total estimate: 17–22 weeks (≈4–5 months).** The M7b hardening milestone added 2–3 weeks vs the original plan; that time is what makes the build production-ready rather than demo-ready.

## Cross-cutting strategy  `[PLAN: new]` `[DOCS: ⏳ pending]`

### Testing

- **Unit** — in each crate. Property-based on state machines (proptest) and Permission Check.
- **Acceptance** — `tests/acceptance/` harness that loads a reference layout (one of the 15) into a fresh DB, runs the scenario, asserts on DB state + audit events.
- **NFR** — `tests/nfr/`: performance benchmarks (criterion), security invariants (proptest), observability presence (grep the audit log for required event shapes).
- **No mocked DB in acceptance tests** — real SurrealDB instance per test; fast enough at v0 scale.

### Observability baseline (NFR-observability)

- `tracing` crate for structured logs.
- `axum-prometheus` for `/metrics`.
- Custom `audit_event` sink writing to both SurrealDB and a shadow append-only log for recoverability.
- Every API handler emits an audit event on write; read handlers emit `silent`-class traces only.

### CI/CD gates

- `cargo fmt -- --check`
- `RUSTFLAGS="-Dwarnings" cargo clippy --all-targets`
- `cargo test` (unit + integration)
- `cargo test --features nfr` (performance benchmarks)
- Next.js: `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build`
- Spec-drift guard: grep `R-ADMIN-*` IDs referenced in tests still exist in the requirements files.

### Concept doc alignment

Per `baby-phi/CLAUDE.md`: docs must reflect code. During each milestone, if implementation surfaces a concept-doc gap or contradiction, fix both the code and the doc in the same commit. Update `Last verified` headers.

## Critical Files (new)  `[PLAN: new]` `[DOCS: n/a — reference list]`

| Path | Purpose |
|------|---------|
| `baby-phi/Cargo.toml` (modified) | Workspace declaration — add 3 crates |
| `baby-phi/baby-phi-domain/` | Graph model, Permission Check, state machines |
| `baby-phi/baby-phi-store/` | SurrealDB adapter |
| `baby-phi/baby-phi-server/` | axum HTTP handlers |
| `baby-phi/web/` | Next.js 14 frontend |
| `baby-phi/tests/acceptance/` | Layout-driven integration tests |
| `baby-phi/tests/nfr/` | Performance + security NFR tests |
| `baby-phi/.github/workflows/` | CI gates |
| `baby-phi/docs/specs/plan/build/<random>-build-plan-v01.md` | Archived copy of this plan |

## Files to reuse from phi-core

- `phi-core::AgentProfile`, `phi-core::ModelConfig`, `phi-core::ExecutionLimits` — surfaces in admin/09 agent profile editor payloads, in domain model, and in SurrealDB schema for Agent nodes.
- `phi-core::Session`, `phi-core::LoopRecord`, `phi-core::Turn`, `phi-core::Message` — session execution history; baby-phi wraps with org/project context.
- `phi-core::agent_loop()` + `phi-core::agent_loop_continue()` — session execution; baby-phi calls these during first-session-launch (admin/14).
- `phi-core::AgentEvent` stream — the source event stream that s02/s03/s04/s05/s06 flows subscribe to.

## Execution discipline per milestone  `[PLAN: new]` `[DOCS: ⏳ pending]`

Each milestone (M0–M8) is multi-session work. The plan archive step is the first bullet of M0 above; no separate "B0 archive" phase is needed. Within a milestone:

1. Set up a scoped TODO list covering the milestone's admin pages + system flows + tests.
2. Build domain + store changes first, then server handlers, then CLI, then web UI page.
3. Land per-page acceptance tests before moving to the next page.
4. Update docs (if implementation surfaces a concept gap, fix both code and doc in the same commit per `baby-phi/CLAUDE.md`).
5. Tag milestone completion with a git tag (`v0.1-m0`, `v0.1-m1`, …).

## Verification  `[PLAN: new]` `[DOCS: ⏳ pending]`

- **Per-milestone:** every admin page in that milestone has its acceptance scenarios green as integration tests.
- **End-to-end (post-M8):** fresh install, run the minimal-startup scenario top-to-bottom via the CLI; observe the session run; observe memory extraction and catalog update; `cargo test --workspace` all green; `npm run build` succeeds.
- **NFR measurement:** a benchmark suite produces a report comparable to `nfr-performance.md`'s targets; all security invariants from `nfr-security.md` have passing property tests.
- **Spec-alignment check:** every requirement ID referenced in a test exists in the requirements files; every concept section cited by ≥1 requirement has ≥1 exercising test.

## What Stays Unchanged  `[PLAN: new]` `[DOCS: n/a — scope guard]`

- Concept docs remain the source of truth; the build implements them. Corrections during build land in both code and doc in the same commit.
- The 15 reference layouts are not modified — they become test fixtures.
- phi-core is consumed as a library; no changes unless a specific extension is needed (would be tracked as a separate plan).
- Scope: this plan covers v0.1 admin + agent-self-service + system flows + NFRs. Steady-state ops pages (audit log viewer, grant browser, tenant management, etc.) are out of scope for v0.1 — they are a v0.2 conversation.

## Open questions (non-blocking)  `[PLAN: new]` `[DOCS: n/a]`

Picked up during build; none block start:

- Auth for admin sessions: cookie-based SSR session, JWT, or OAuth? Decide in M0 alongside Next.js scaffold.
- Next.js deployment target: Vercel (hosted) or self-hosted Node? Decide in M8 pre-release.
- Local-dev experience: docker-compose with SurrealDB + baby-phi-server + Next.js dev server? Or native-run scripts?
