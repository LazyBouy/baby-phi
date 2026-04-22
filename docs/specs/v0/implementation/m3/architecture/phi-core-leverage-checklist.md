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

## Enforcement — four-tier model

A single grep linter is insufficient. phi-core leverage is guarded
by four tiers, each catching a failure the other tiers cannot. The
checklist below is **only one of the tiers**; documented here so
reviewers and future planners know what else is in place and don't
regress to "the script passes, ship it".

### Tier 1 — CI-enforced, zero-human-judgement

Catches: duplication, type swaps, wire-shape drift.

| Mechanism | What it catches | Failure surface |
|---|---|---|
| [`scripts/check-phi-core-reuse.sh`](../../../../../../scripts/check-phi-core-reuse.sh) | Parallel redeclarations of phi-core types under `modules/crates/` (e.g. a local `struct AgentProfile`) | Duplication only — **blind to miss-leverage** where a surface *should* wrap phi-core but doesn't. |
| Compile-time coercion tests (e.g. `fn is_phi_core_agent_profile(_: &phi_core::…::AgentProfile) {}` then `is_phi_core_agent_profile(&built_pair[0].1.blueprint)`) | Accidental swap of a phi-core wrap for a local struct — the test stops compiling. | Only the call-sites the test exercises. Add one per non-trivial wrap. |
| Schema-snapshot tests with **forbidden-key** lists (M3/P5's triple-tier: unit + acceptance + web) | A reviewer re-adding a phi-core-wrapping field to a deliberately-stripped wire shape. | Only the wire shapes with explicit tests. Add one whenever a response *strips* phi-core. |

### Tier 2 — Structural, reviewer-enforced (this checklist)

Catches: miss-leverage, silent omissions, reductive "None" answers.

- Q1/Q2/Q3 per-phase audit in the plan (see §§1–5 above).
- **Deliverable-level** phi-core tags — not phase-level summaries.
- Positive close-audit grep assertions in `### Confidence check`
  ("this import MUST exist"), not only negative ones.
- Pre-audit runs **before** implementation (P5.0 precedent) when a
  phase touches anything data-plane or wire-shape-ish.

### Tier 3 — Governance record (ADRs + reuse map)

Catches: re-litigation of already-decided boundaries; drift in the
"where does each phi-core type live?" answer.

- [`phi-core-reuse-map.md`](phi-core-reuse-map.md) — durable table of type-level wraps, updated per milestone close. Includes the "why `Organization` is NOT a wrap of `phi_core::Session`" argument (D11).
- ADRs for non-trivial reuse decisions: [ADR-0019 non-retroactive snapshot](../decisions/), [ADR-0023 inherit-from-snapshot](../decisions/0023-system-agents-inherit-from-org-snapshot.md).
- The build plan's `#### Carryovers from M<n>` subsections carry phi-core implications forward to the next milestone's detailed planning session (M3 added carryovers to both M4 and M5 sections of the base plan).

### Tier 4 — Independent retrospective

Catches: aggregate drift, coordinate failures across phases, cases
where each phase looked fine but the milestone overall missed.

- Independent re-audit **agent** at milestone close (M2/P8 precedent; M3/P6 planned). Target: ≥99% composite confidence. The agent re-walks the Q1/Q2/Q3 discipline across every phase's shipped code without the author's context.
- Post-milestone update to the reuse map + the leverage-checklist `§Backstory` (add a new slip entry if one occurred, so the next milestone's planner reads the failure mode).

## What each tier does NOT catch

Calling out the gaps explicitly so a future reviewer knows not to
treat "tier X passes" as sufficient:

- **Tier 1 does not catch miss-leverage.** The grep only fires on
  *present* parallel types. A phase that forgets to wrap phi-core at
  all looks identical to a phase that correctly has no phi-core
  overlap.
- **Tier 2 is human-applied.** Reviewer fatigue + reductive answers
  ("leverage = None") have already slipped through once (M3/P3 — see
  §Backstory). This is why Tier 4 exists.
- **Tier 3 is a static record.** An ADR pinned yesterday doesn't
  catch a code change made today that silently contradicts it.
- **Tier 4 is milestone-coarse.** A mid-milestone slip can merge + sit
  uncaught for ~2 weeks.

## Known gaps (candidates for future hardening)

Not yet implemented; noting here so they don't slip out of scope:

- **PR-level phi-core gate.** A per-PR template with Q1/Q2/Q3
  checkboxes would catch slips within hours rather than at phase
  close. Candidate for M7b's CI-hardening phase.
- **Automated miss-leverage detection.** A grep for baby-phi struct
  definitions that *mirror* (by field-name set) a phi-core type
  would catch cases the current linter misses. Candidate for M6+ when
  the type inventory stabilises.
- **"Why did this wrap get deferred?" audit.** When a phase chooses
  not to wrap despite an overlap, require an explicit ADR-tier
  justification (not just a plan comment). Would have surfaced the
  M3/P3 slip before P3 opened.

## Reviewer rejection criteria (Tier 2 baseline)

Reviewers should reject any plan's `### phi-core leverage` subsection
that:
- Lacks a Q1/Q2/Q3 split.
- Says "None" without Q3's explicit module-walk rejection list.
- Has only negative close-audit assertions.
- Has phase-level summary tags instead of deliverable-level tags.
- Cites only Tier 1 (the grep linter) as evidence of leverage
  discipline — Tier 1 cannot catch miss-leverage.
