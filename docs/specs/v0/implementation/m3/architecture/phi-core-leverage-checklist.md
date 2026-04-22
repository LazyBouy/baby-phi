<!-- Last verified: 2026-04-22 by Claude Code -->

# phi-core leverage checklist

**Status: [EXISTS]** — landed mid-M3 as a process-hardening response to
the P3 "leverage = None" slip (see §Backstory below).

## Why this checklist exists

The [baby-phi/CLAUDE.md §phi-core Leverage](../../../../../../CLAUDE.md)
mandate is clear: *every baby-phi surface that overlaps a phi-core type
must reuse it directly or wrap it — never re-implement*. `scripts/check-phi-core-reuse.sh`
hard-gates *duplication*, but the grep-linter cannot catch
**missed-leverage opportunities** where baby-phi *should* have reused a
phi-core type and didn't.

Catching miss-leverage requires a structural audit per phase. The
single-question "how does this phase leverage phi-core?" in the M3 plan
template proved insufficient — reductive answers ("None" / "N/A") slip
through when the author doesn't systematically walk phi-core's surface
against each deliverable. This checklist is the forcing function.

## The checklist (run per phase, before writing `### phi-core leverage`)

### 1. Enumerate deliverables

List every concrete artefact the phase ships: files, structs, traits,
functions, routes, CLI commands, web pages, migrations, tests. *Not*
"extend the foo handler" — name the function signatures.

### 2. For each deliverable, ask Q1 / Q2 / Q3

**Q1 — Direct imports.** Does the new code `use phi_core::…`? List
every expected import path.

**Q2 — Transitive payload.** Does the data this code writes, reads, or
transports carry phi-core types through field types? **Walk the struct
field types one level deep**, then recurse into composites. Field types
like `Vec<Agent>` expand into `Agent`'s fields; `Organization`
expands into `Organization.defaults_snapshot` which wraps 4 phi-core
types.

Transitive carriage counts as leverage even if the wrapper is baby-phi.
P3's `apply_org_creation` tx is the canonical example: the wrapper is
baby-phi plumbing, but the payload ships `phi_core::AgentProfile`
(system-agent blueprints) and `phi_core::ModelConfig` (via
`ModelRuntime`) through the tx.

**Q3 — Candidates considered and rejected.** Walk the phi-core module
inventory below; for each, state whether it applies to the phase and
(if no) why. Silent omission is not allowed — every rejection must be
explicit.

phi-core module inventory to walk (update when phi-core grows):

```
phi-core/src/
├── agent_loop/     # AgentLoopConfig, agent_loop(), agent_loop_continue()
├── agents/         # Agent trait, BasicAgent, AgentProfile, SubAgent, SystemPrompt
├── config/         # AgentConfig, ProfileSection, ProviderSection, CompactionSection
├── context/        # ExecutionLimits, ContextConfig, CompactionStrategy, tokens
├── mcp/            # McpClient, McpTransport, McpToolAdapter, types::*
├── openapi/        # OpenApiToolAdapter (feature-gated)
├── provider/       # ModelConfig, ApiProtocol, StreamProvider, RetryConfig, retry
├── session/        # Session, LoopRecord, Turn, SessionRecorder, LoopStatus
├── tools/          # PrunTool, file tools, built-ins
└── types/          # AgentEvent, Content, Message, AgentMessage, Usage, ToolResult
```

### 3. Decide Direct / Wrap / Inherit / Build

For every deliverable + candidate pair, one of:
- ✅ **Direct reuse** (`use phi_core::X`) — field type or return type is
  the phi-core type itself.
- 🔌 **Wrap** — baby-phi struct has a phi-core type as a field
  (example: `AgentProfile.blueprint: phi_core::AgentProfile`).
- ♻ **Inherit, don't duplicate** — phi-core type lives in one place
  (typically a shared composite like `Organization.defaults_snapshot`)
  and downstream consumers read from there instead of creating a copy.
  See [ADR-0023](../decisions/0023-system-agents-inherit-from-org-snapshot.md).
- 🏗 **Build-from-scratch (baby-phi-native)** — phi-core has no
  counterpart. Must cite **which orthogonal-surface** boundary applies
  (governance plane, permission engine, audit log, etc. per
  baby-phi/CLAUDE.md §Orthogonal surfaces).

### 4. Tag every deliverable inline

In the plan's `### Deliverables` list, each bullet carries a phi-core
tag:

```
1. **Compound tx repo method** `Repository::apply_org_creation(…)`
   [phi-core: phi_core::AgentProfile via AgentProfile.blueprint (×2 system agents);
    phi_core::ModelConfig via ModelRuntime.config resolved through UsesModel edge.
    ♻ Inherit from Organization.defaults_snapshot: ExecutionLimits, ContextConfig,
    RetryConfig (no per-agent duplication — ADR-0023).]
```

Summaries at the phase level are no longer sufficient — every
deliverable carries its own tag.

### 5. Write `### phi-core leverage` as Q1 / Q2 / Q3

Reorganise the subsection with three explicit headings:

```
#### phi-core leverage

**Q1 — Direct imports**
- `phi_core::X` — used in file.rs for reason.
- (none, if phase is truly pure-plumbing with no imports)

**Q2 — Transitive payload**
- `phi_core::Y` flows through struct Z.field.
- (none, if phase touches no phi-core-wrapping data)

**Q3 — Candidates rejected (with reasons)**
- `phi_core::Session` — not relevant, session launch is M5 work.
- `phi_core::ToolResult` — not relevant, this phase doesn't execute tools.
- (…etc. Walk the full module inventory; silence = insufficient.)
```

### 6. Close-audit assertions (positive, not negative)

The phase's `### Confidence check` must list **positive** grep
assertions — things that should exist — not only negative ones.

**Bad (insufficient):**
> phi-core leverage: sanity-grep to confirm no `phi_core::` imports leaked.

**Good:**
> phi-core leverage: grep confirms `AgentProfile.blueprint` field in every
> `create_agent_profile` call site is `phi_core::agents::profile::AgentProfile`
> (not a baby-phi local struct); integration test asserts 0 rows on the
> per-agent `execution_limits` / `retry_policy` / `cache_policy` tables
> after `apply_org_creation` runs (ADR-0023 inherit-from-snapshot
> invariant); `check-phi-core-reuse.sh` zero hits.

Positive assertions catch miss-leverage (a phase that *should* have
used a phi-core type but didn't); negative assertions only catch
duplication.

## Backstory — the M3/P3 leverage slip

M3's original plan labelled P3's `### phi-core leverage` as **"None"**
on the reasoning that P3 ships "graph-transaction plumbing + batch
audit helper + test fixture + per-org chain proptest — all baby-phi
plumbing." That answered Q1 (no direct imports) but skipped Q2 (the
compound tx's *payload* carries `phi_core::AgentProfile` and
`phi_core::ModelConfig` via M2's existing wraps).

The user pushed back during the P2→P3 handoff and surfaced the miss.
P3.0 corrected the plan (`phi-core: AgentProfile × 2 blueprints +
ModelConfig via ModelRuntime`) and ADR-0023 pinned the
inherit-from-snapshot decision for `ExecutionLimits` / `ContextConfig`
/ `RetryConfig`.

The deeper failure was **process**, not content: the single fuzzy
question "does this phase leverage phi-core?" let reductive answers
slip through. A Q1/Q2/Q3 split + deliverable-level tags + positive
close-audit greps is the structural fix.

## Applicable retroactively

M3's P0–P2 phases have been re-audited against the Q1/Q2/Q3 discipline
and the results confirmed clean (P2 genuinely touches no phi-core
surfaces — templates + governance audit events + org-scoped list
methods all sit on baby-phi's orthogonal governance plane).

P3 onwards: every phase's `### phi-core leverage` subsection in the
[M3 plan archive](../../../../plan/build/563945fe-m3-organization-creation.md)
follows the Q1/Q2/Q3 / deliverable-tag / positive-grep structure.

## Applicable forward (M4+)

M4's plan inherits this checklist. Every milestone's Part 1.5 (phi-core
reuse map) cross-links here. The plan-template convention for
`### phi-core leverage` now uses the Q1/Q2/Q3 split by default.

## Enforcement

Structural discipline, not CI-enforced. The existing
`scripts/check-phi-core-reuse.sh` continues to hard-gate duplicated
phi-core type definitions. This checklist catches the complementary
failure mode (miss-leverage) that the grep-linter cannot see.

Reviewers should reject any plan's `### phi-core leverage` subsection
that:
- Lacks a Q1/Q2/Q3 split.
- Says "None" without Q3's explicit module-walk rejection list.
- Has only negative close-audit assertions.
- Has phase-level summary tags instead of deliverable-level tags.
