# CLAUDE.md — phi

phi is the agent management platform that consumes `phi-core` as a library. As of M0 it is a **Cargo workspace** with three parallel surfaces (CLI, HTTP API, Next.js web UI) sharing one domain layer.

## Workspace layout

```
phi/                       (workspace root — virtual Cargo.toml)
├── modules/
│   ├── crates/
│   │   ├── cli/                CLI binary (clap). Package `cli`, binary `phi`.
│   │   ├── domain/             Graph model + Permission Check + state machines.
│   │   ├── store/              SurrealDB (embedded, RocksDB) adapter.
│   │   └── server/             axum HTTP API. Package `server`, binary `phi-server`.
│   └── web/                    Next.js 14 (App Router + SSR).
├── config/                     Layered TOML configs (default + dev/staging/prod).
├── docs/specs/                 Concepts + requirements (source of truth for v0).
├── docs/specs/plan/            Plan archives (git-ignored by convention).
├── scripts/                    Ops helpers (spec-drift check, …).
├── Dockerfile                  Multi-stage build for phi-server.
├── docker-compose.yml          Local dev stack (server + web).
└── deny.toml                   cargo-deny policy.
```

Dependency flow (strict, downward):
```
cli              ┐
server          ─┼─▶ domain ─▶ store ─▶ SurrealDB
web (Next.js)  ──┘             (plus phi-core for agent/session types)
```

Package names are deliberately terse (`cli`, `domain`, `store`, `server`); the shipped binary names keep the product prefix (`phi`, `phi-server`) via explicit `[[bin]] name`.

## Build & Run

All cargo commands must use `/root/rust-env/cargo/bin/cargo`. CI enforces `RUSTFLAGS="-Dwarnings"`.

```bash
# From phi/ (the workspace root):
/root/rust-env/cargo/bin/cargo build --workspace
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace

# Run the HTTP server (reads config/default.toml + config/<profile>.toml +
# PHI_* env vars; see modules/crates/server/src/config.rs):
PHI_PROFILE=dev /root/rust-env/cargo/bin/cargo run -p server

# Run the existing CLI demo (still reads phi/config.toml):
set -a && source .env && set +a
/root/rust-env/cargo/bin/cargo run -p cli

# Web UI (from phi/modules/web/):
npm install && npm run dev
```

## Scope

Every platform-level feature sits in this workspace — phi-core stays a pure agent-loop library. Features tracked in the v0.1 build plan (`docs/specs/plan/build/`):

- Permission Check engine + Auth Request state machine + graph model (M1).
- Admin pages 01–14 as CLI + API + Web UI vertical slices (M2–M5).
- Agent self-service surfaces a01–a05 (M6).
- System flows s02–s06 (M7).
- Production hardening — OAuth 2.0, TLS, at-rest encryption, backup/restore, OpenTelemetry, rate limiting, GDPR erasure, runbook (M7b).

## phi-core Leverage (first-class mandate)

phi is a **consumer** of phi-core, not a parallel implementation. Every surface that overlaps with an existing `phi_core::` type MUST reuse it directly or wrap it — **never re-implement**. This is not a style preference; it is a two-source-of-truth problem that compounds per milestone.

**Rules of engagement:**

1. **Before introducing any struct/enum/trait** whose shape overlaps with something in phi-core, check `phi-core/src/` first. If phi-core ships it, import it.
   - Direct reuse (`use phi_core::X`) — use phi-core's type as-is.
   - Wrap (`pub struct Y { inner: phi_core::X, ... }`) — extend with phi-only governance fields.
   - Build from scratch — only if phi-core has **no** counterpart (e.g., permission-check engine, credentials vault, tenant sets, audit hash-chain).

2. **Known reuse surfaces** (non-exhaustive; see `docs/specs/v0/concepts/phi-core-mapping.md` for the full list):
   - Config / agent blueprint → `phi_core::agents::profile::AgentProfile`, `phi_core::config::{parser, schema}`.
   - Providers → `phi_core::provider::{model::ModelConfig, model::ApiProtocol, registry::ProviderRegistry, traits::{StreamProvider, StreamConfig, StreamEvent}, retry::RetryConfig}`.
   - Tools → `phi_core::types::tool::{AgentTool, ToolResult}`, `phi_core::mcp::{client::McpClient, types::*, tool_adapter::McpToolAdapter}`.
   - Execution / context → `phi_core::context::{execution::ExecutionLimits, ContextConfig, CompactionStrategy}`, `phi_core::types::usage::{CacheConfig, ThinkingLevel}`.
   - Sessions / events → `phi_core::types::event::AgentEvent`, `phi_core::session::model::{Session, LoopRecord, Turn, LoopStatus}`, `phi_core::session::recorder::SessionRecorder`.

3. **Enforcement.** `scripts/check-phi-core-reuse.sh` runs in CI and fails on forbidden duplications (e.g., any `struct ExecutionLimits`, `struct ModelConfig`, `struct McpClient`, `struct AgentProfile`, `struct AgentEvent`, `struct Session`, etc. defined anywhere under `modules/crates/`). The lint is advisory during a milestone's foundation phase and flips to hard-gate at the next re-audit.

4. **Reviewer checklist.** In code review, reject any PR that introduces a new type whose field set matches a phi-core type; require the phi-core import instead. When in doubt, consult the phi-core-reuse-map doc for the milestone.

5. **`thiserror` must track phi-core's version** (currently `"2"`). Version drift breaks `#[from]` conversions at runtime with cryptic "implementations not found" errors.

**Orthogonal surfaces that are NOT phi-core duplicates** (these are intentionally phi-only — do not conflate):
- `domain::audit::AuditEvent` (governance write log, hash-chain, retention tier) vs `phi_core::types::event::AgentEvent` (agent-loop telemetry stream) — see `implementation/m1/architecture/audit-events.md`.
- `server::session::SessionClaims` (HTTP cookie JWT) vs `phi_core::session::Session` (persisted execution trace) — see `implementation/m1/architecture/server-topology.md`.
- `domain::model::ToolDefinition` (permission metadata node) vs `phi_core::types::tool::AgentTool` (runtime trait) — see `implementation/m1/architecture/graph-model.md`.
- `server::config::ServerConfig` (HTTP infrastructure TOML) vs `phi_core::config::schema::AgentConfig` (agent blueprint YAML/TOML/JSON with `${VAR}`) — see `implementation/m1/architecture/overview.md`.
- `domain::Agent` (governance principal — identity, kind, role, org membership, lifecycle: `active`, `archived_at`) vs `phi_core::Agent` (runtime trait — prompting/state/control interface) and `phi_core::BasicAgent` (runtime in-memory impl of that trait) — wholly orthogonal layers. baby-phi tracks *who the agent is in the org*; phi-core executes the runtime loop. Connection at session-launch time is **ID-only**: `domain::AgentId.to_string()` flows into `phi_core::types::context::AgentContext.agent_id` via `sessions/provider.rs::build_agent_context`. baby-phi is per-request stateless; it never instantiates `phi_core::Agent` / `BasicAgent`. (Per ADR-0034 §D34.6; revisit only if a future milestone introduces long-lived in-memory chat agents.) See `docs/specs/v0/concepts/phi-core-mapping.md` §"Connection point" for full integration flow.

When the line is unclear, err toward reuse and ask in review.

## Documentation Alignment

Documentation in `docs/` must accurately reflect the current codebase at all times. Code is always the source of truth.

- **Update docs with code changes**: When modifying code, update all affected documentation in the same commit. This includes status tags, API signatures, config examples, and pseudocode.
- **Status tags**: `[EXISTS]` = implemented in code, `[PLANNED]` = designed but not yet implemented, `[CONCEPTUAL]` = idea stage. Review and update these tags whenever the referenced code changes.
- **Verification header**: Every doc file carries `<!-- Last verified: YYYY-MM-DD by Claude Code -->` at the top, updated on each review pass.
- **No forward references**: Do not document features as existing unless the code is merged. Use `[PLANNED]` or `[CONCEPTUAL]` for future work.
- **Spec-drift guard**: `scripts/check-spec-drift.sh` runs in CI. If a requirement id (`R-ADMIN-*`, `R-AGENT-*`, `R-SYS-*`, `R-NFR-*`) referenced in code disappears from `docs/specs/v0/requirements/`, CI fails.

## Working Discipline

- **Thoroughness over speed.** When a choice exists between "faster" and "more thorough," always pick thorough. Applies to audits, test coverage, documentation, refactors, and milestone execution. Speed is cheap to regain; shortcuts compound into debt.
- **Phase-by-phase review.** For multi-phase milestones (M1+), pause at each phase boundary for a thorough self-review against the milestone's verification matrix before opening the next phase. Don't chain phases autonomously.
- **Pre-implementation audits.** Before starting a milestone, run a gap-audit against every concept doc, requirement, and production-readiness commitment it touches. Surface deltas (stale counts, implicit assumptions, missing pieces) in the plan rather than discovering them during implementation.

## Multi-agent chunk pipeline

baby-phi chunks (CH-NN) run through a 4-agent pipeline orchestrated by Claude. The orchestrator (Claude with full conversation context) is the **reviewer / approver / process-refiner / retrospective-driver**, not a doer in the chunk lane. Specialized agents own their lanes; the orchestrator gates phase transitions, verifies diffs, audits audit reports, and drives retrospectives.

**Agents** at `/root/projects/phi/.claude/agents/`:
- `chunk-planner` (opus) — drafts the 12-section plan from a forward-scope row.
- `chunk-implementer` (opus) — executes phases per the approved plan.
- `chunk-auditor` (opus) — independent post-implementation audit; writes per-iteration audit log.
- `chunk-retrospector` (opus) — consolidated retrospective + standards-update proposals.

**Skills** at `/root/projects/phi/.claude/skills/`:
- `phi-core-leverage-check`, `k8s-readiness-check`, `chunk-template-fill`, `ci-guards-run`, `chunk-archive-plan`, `audit-envelope-size`.

**Cycle artifact layout** under `docs/specs/plan/build/<slug>-<8hex>/`:
- `plan.md` — cycle plan
- `audit-<letter>-iter<N>.md` — per-iteration audit logs
- `cycle-audit.md` — orchestrator's consolidated final audit
- `retrospective.md` — cycle retrospective

Pre-existing chunks (CH-09, CH-10, CH-23) keep their flat-file legacy layout; the folder convention applies to new cycles only. Index at `docs/specs/plan/build/_cycle-index.md`.

**Orchestrator's gates:**
1. **Plan approval.** Read planner's draft. Auto-approve via ExitPlanMode when Direct-approval criteria hold (no locked forks, scope ≤ 1.5× forward-scope, zero phi-core leverage delta, no new K8s blocker class, audit envelope ≤ medium, confidence ≥ 9/10, no new migration). Otherwise escalate to user via AskUserQuestion + ExitPlanMode.
2. **Per-phase implementation review.** Read diff, run cargo test + clippy myself, verify phi-core grep, confirm test count matches plan §8 expected. **Doc-sync sweep after gate-2 inline corrections** (added 2026-05-08 per CH-14 retro Row 3, cycle hex `5803bb94`; **widened 2026-05-08 per CH-15 retro Row 1, cycle hex `c3f46f17`**): when a gate-2 inline correction changes implementation behaviour OR when a chunk closes a drift with cross-cutting documentary impact, grep ALL `docs/specs/v0/implementation/m*/architecture/*.md` + `m*/operations/*.md` + `m*/user-guide/*.md` (NOT just plan §3.C-listed files) for the canonical stale-narrative phrase set: `FOLLOWUP-NN`, `deferred per`, `is NOT emitted`, `not emitted at CH-NN`, `advisory at M5`, `Step 0 only blocking`, `M6+ tightens the gate`, `at M5/P4`, `not blocking at M5`. Patch any matches BEFORE dispatching auditors. Audit-fix-loop iteration cap counts these patches as Trivial-multi if > 1 line, Trivial-1L if ≤ 1 line.
3. **Audit review.** Read each iteration's audit log; spot-check 1–2 random claims by reading cited file:line.
4. **Final cycle re-audit (mandatory).** After all sub-agent audits go green, I personally re-read every diff, re-run full workspace tests + 4 CI guards, run phi-core-leverage-check + k8s-readiness-check skills, verify all paperwork. Write `cycle-audit.md`. May re-trigger Implementer or Planner re-spawn. Never skipped. **MUST-RUN list (sub-agents cannot execute these reliably):** `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` + the 4 `bash scripts/check-*.sh` CI guards. Sub-agent auditors will mark these claims `NOT-EXECUTED-IN-AUDIT` (sandbox-blocked) — orchestrator closes them at this gate.
5. **Retrospective review.** Read retrospector's draft; propose standards updates to user; apply approved updates with version bumps logged in `.claude/agents/_changelog.md`. **Cargo-clean discipline operates at TWO placements (refined 2026-05-10 per CH-18 retro Row 1, USER DIRECTIVE, cycle hex `c77937bc`)**:

(1) **Immediate-post-test cleanup (NEW per CH-18)**: AFTER each `cargo test --workspace` invocation across the cycle (sub-agent audits A + B, orchestrator gate-4 final test, retrospector permissions-audit script), the invoker MUST run `cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` BEFORE issuing the next cargo invocation. CH-18 evidence: 2 duplicate cargo-test workspace runs accumulated target/ to 146 GB → 100% disk → 1h24m hung. User directive: *"tests should be cleaned up immediately after the run as it may block future tests"* (2026-05-10).

(2) **Gate-5 final close cleanup (CH-17 retro Row 1, USER REQUESTED 2026-05-09, cycle hex `40c4d759`)**: after standards updates landed + cycle-index row flipped to `retro-complete`, orchestrator runs final `cargo clean` as closing step before user commit.

CH-17 retro Row 1's gate-5-close-only placement was insufficient because target/ can balloon DURING gate-4 if multiple test invocations run without cleanup. Both placements are now mandatory.

**Audit-fix loop:**
- **Tactical FAIL** — re-spawn Implementer with audit log path; re-spawn auditors (iter N+1).
- **Architectural FAIL** — re-spawn Planner with audit log path; **always escalate to user**; re-spawn Implementer; re-spawn auditors.
- **Trivial FAIL** — split into two sub-tiers:
  - **Trivial-1L**: ≤ 1-line orchestrator-applied patch on a verified-header / changelog row / index entry → orchestrator verifies in `cycle-audit.md` (no auditor re-spawn). Logged in cycle-audit §"Iteration accounting".
  - **Trivial-multi**: > 1-line trivial patch (small docstring, missed cross-ref, etc.) → re-spawn auditor at iter N+1 as before.
- **Iteration cap**: ≥ 3 iterations on the same finding → STOP, escalate to user.

**Meta-plan archive**: design rationale lives at `docs/specs/agentic-workflow/multi-agent-chunk-pipeline-0853574c.md`. Read this before extending the system (e.g., adding a `phase-planner` agent for M6+ milestone-to-chunks decomposition).

**Quality is non-negotiable.** The user's locked principle: *quality and thoroughness over cycle completion*. The final cycle re-audit cannot be skipped. Every audit FAIL flows into the retrospective's audit-cycle gaps section with a proposed gap-closing change.

**Telemetry + permissions-audit (added 2026-05-03 per `docs/specs/permissions/tool-use-logging-and-permissions-audit-skill-18564835.md`).** Every tool call is logged to `.claude/tool-use.log` (gitignored, JSONL, 10MB rotation) by the `log-tool-use.sh` hook (PostToolUse + PostToolUseFailure + PermissionRequest). At retro time, the chunk-retrospector (v2) invokes the `permissions-audit` skill which reads the log, cross-references `settings.json` rules, and emits an §A–§H markdown report. Findings (hot allow-rule candidates, dead rules, hook false-positive flags, cross-cycle trends) land in §3.5 of the cycle retrospective with the full report appended. Standards updates from the audit flow through the same retro → user-review → standards-update pipeline as agent-prompt updates.
