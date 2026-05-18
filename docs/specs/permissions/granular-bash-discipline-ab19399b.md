<!-- Last verified: 2026-05-04 by Claude Code (NEW design doc per orchestrator plan-mode iter 4 approved by user 2026-05-04 after CH-13 close; cycle hex `ab19399b`) -->

# Granular Bash discipline + matcher semantics + CH-13 prompt analysis

**Plan archive token:** `ab19399b` (generated 2026-05-04 at plan-open via `openssl rand -hex 4`).
**Plan archive path:** `/root/.claude/plans/sharded-discovering-stearns.md` (verbatim; this file is the cycle-folder copy).
**Status:** Accepted (landed 2026-05-04).
**Source-of-truth role:** the canonical reference for "why does this Bash command prompt?" questions in any future cycle.

---

## §1 — The Granular Bash principle

**Principle**: each Bash tool invocation runs **one logical operation**. Multiple operations = multiple invocations.

**Why**:
- Claude Code's Bash matcher splits compound commands at shell operators and requires each subcommand to independently match an allow rule. Granular invocations match cleanly without depending on compound-split quirks.
- Each invocation produces one telemetry entry per intent — auditable.
- One command → one outcome — failures are easy to diagnose.
- Avoids the "multi-line bash script chopped into fragments" trap that caused 48% of CH-13's PermissionRequest events.

### 1.1 — Allowed shapes (granular)

1. **Single command + flags + paths**: `grep -rn 'X' /root/projects/phi/baby-phi/modules/crates/`. ✓
2. **Single command + 1 trailing viewing/aggregating pipe**: `grep -rn 'X' /abs/ | head -10`, `find /abs -name '*.rs' | wc -l`, `git -C /abs log --oneline -5 | tail -5`. ✓
3. **Single command with redirects paired with a downstream pipe**: `cargo test 2>&1 | tail -20`. ✓ (the `2>&1` MUST be paired with a downstream pipe.)

### 1.2 — Discouraged shapes — break into separate Bash calls

1. **Multi-line bash scripts** in a single Bash call (any newlines). → write to a file via the Write tool (e.g., `/root/projects/phi/baby-phi/scripts/audit-tmp.sh`), then `bash /abs/path/audit-tmp.sh` as ONE invocation. OR run each line as a separate Bash call.
2. **`&&` / `||` chains**: `cmd1 && cmd2 && cmd3` → 3 separate Bash calls.
3. **`;` separators**: `cmd1; cmd2` → 2 separate Bash calls.
4. **Pipelines beyond 2 stages**: `cmd1 | cmd2 | cmd3 | cmd4` → either restructure (have cmd1 produce final output) or write-to-file-then-bash.
5. **Mixed operator chains**: `cmd1 && cmd2 | head; cmd3` → 3 separate Bash calls.
6. **Trailing `2>&1` without a downstream pipe**: empirical quirk-trigger (CH-13 had 2 such prompts). Either drop the redirect or pair with `| cat` / `| head`.
7. **`cd <abs> && <cmd>` compounds**: prefer absolute paths in the command itself.

### 1.3 — Tool-specific absolute-path forms

| Instead of | Use |
|---|---|
| `cd /abs/path && git status` | `git -C /abs/path status` |
| `cd /abs/path && git log` | `git -C /abs/path log` |
| `cd /abs/path && cargo test` | `cargo --manifest-path /abs/path/Cargo.toml test` |
| `cd /abs/path && bash scripts/check.sh` | `bash /abs/path/scripts/check.sh` |
| `cd /abs/path && grep -rn 'X' modules/` | `grep -rn 'X' /abs/path/modules/` |

### 1.4 — Where this discipline applies

- **Orchestrator** (Claude with full conversation context running cycle turns): bound by `/root/projects/phi/CLAUDE.md` "Granular Bash discipline" section.
- **chunk-implementer agent** (v4): `/root/projects/phi/.claude/agents/chunk-implementer.md` "Granular Bash discipline" subsection. Refactored from v3's "Bash usage discipline" to lead with the granular principle.
- **chunk-auditor agent** (v4): `/root/projects/phi/.claude/agents/chunk-auditor.md` "Granular Bash discipline" subsection. Same refactor.
- **chunk-planner / chunk-retrospector agents**: no Bash-heavy work; no specific bash-discipline subsection needed (they read + write doc files via Read/Write/Edit tools).
- **Explore + general-purpose agents**: project-agnostic; cannot inject project-specific rules. Acceptable.

---

## §2 — Authoritative matcher semantics (Claude Code docs)

Source: `https://code.claude.com/docs/en/permissions.md` (sections "Permission rule syntax", "Wildcard patterns", "Compound commands", "Bash").

### 2.1 — Rule syntax

- **`Bash(prefix:*)` and `Bash(prefix *)` are equivalent** — but only at the end of a pattern. Both match commands starting with `prefix` followed by a space and any args.
- **Mid-pattern colons are LITERAL.** `Bash(git:* push)` does NOT match git commands; the colon is treated literally.
- **Word boundary for `*`**: `Bash(grep *)` matches `grep -rn 'X'` but NOT `grepper`. The space before `*` enforces a word boundary.

**Per Occam's razor**: `.claude/settings.json` was migrated from colon-star to space-star on 2026-05-04. Both forms are equivalent at end-of-pattern; the simpler form wins.

### 2.2 — Compound command handling

- **The matcher splits at shell operators**: `&&`, `||`, `;`, `|`, `|&`, `&`, **and newlines**.
- **Each subcommand must independently match an allow rule** for the compound to auto-approve.
- **Process wrappers stripped before matching**: `timeout`, `nice`, `nohup`, `xargs`, etc.
- **NOT stripped**: env-var prefixes (`RUSTFLAGS="-X" cmd` is matched literally, not as `cmd`).
- **Redirects are part of the literal match**: `cmd 2>&1` is matched as the full string including `2>&1`. They're not pre-stripped.
- **Subshells `$(...)` and quoted args are part of the literal match.**

**Worked examples** (compound shapes encountered in cycles + their canonical break-up):

- `cd /abs && cmd <flags>` → BREAK into a single `cmd --manifest-path /abs/Cargo.toml <flags>` invocation (no `cd`). For `git`, use `git -C /abs <subcmd>`; for `bash`, use `bash /abs/script.sh`.
- `cmd1 ; cmd2` (sequential, don't-care about cmd1 result) → BREAK into two separate Bash calls. Each subcommand must independently allow-match.
  - **CH-02a-i-phi example (added 2026-05-17 per CH-02a retro Row 4)**: `ps -p $pid; ls /proc/$pid/status` produced 4 PermissionRequest fires in CH-26-baby-phi concurrent background-polling work because the matcher saw `ps -p ...` as a separate subcommand and there was no `Bash(ps *)` allow rule. Resolution: (a) added `Bash(ps *)` to settings.json benign-read cluster (adjacent to `Bash(stat *)` / `Bash(cat *)` / `Bash(find *)` / `Bash(wc *)`); (b) granular invocations sidestep this entirely — `ps -p $pid` and `ls /proc/$pid/status` should run as separate Bash calls when both are needed.
- `cmd1 | cmd2 | cmd3 | cmd4` (4-stage pipe) → BREAK by writing the pipeline to a script (`scripts/audit-tmp-*.sh`) and running `bash /abs/script.sh` as a single Bash call. The 2-stage cap on inline pipelines is empirical (per CH-14 retro Row 4 cardinality-extraction pipeline; 14 PermissionRequests on a 4-stage chain).
- **`until <check>; do sleep N; done` polling loops (added 2026-05-17 per CH-02b-i-phi retro Row 4)** → the `until + ;-chain + do/done` shape generated 3 PermissionRequest fires in CH-02b's permissions-audit window (Bash:until:! cluster). Root cause: `until cmd; do ...; done` is parsed by the matcher as a compound (the `;` between condition and body splits at shell-operator), and `until` is not allow-listed. **Resolutions**: (a) for a one-shot "wait until the harness signals" check, prefer the harness's notification mechanism (background-process completion fires automatically) rather than polling. (b) for polling external state the harness cannot track (CI run, deploy queue), wrap the loop in a script: write `wait-for-<X>.sh` containing the `until` loop and invoke it as `bash /abs/wait-for-<X>.sh` — single Bash call, single allow-rule match. (c) do NOT add `Bash(until *)` as a top-level allow-rule — `until` is conceptually a control-flow primitive, not a benign-read command; wrap-in-script is the canonical answer. CH-02b precedent: `until ! kill -0 $pid 2>/dev/null; do sleep 2; done` → wrap in `baby-phi/scripts/wait-for-pid.sh <pid> <interval>`.
- **`jq + 3+-stage pipeline` aggregation chains (added 2026-05-18 per CH-02c-i-phi retro Row P6)** → the shape `jq -r '<filter>' <file> | sort | uniq -c | sort -rn | head -N` (4-stage pipeline) generated 20 PermissionRequest fires in CH-02c's permissions-audit window (Bash:jq:'select(.event cluster), retrospector-side cross-cycle aggregation. Root cause: the 2-stage cap on inline pipelines is enforced empirically; 4-stage `jq | sort | uniq -c | sort -rn` exceeds the cap even though `Bash(jq *)` is allow-listed. **Resolution**: write the aggregation pipeline to a single-statement helper script. CH-02c precedent: retrospector cross-cycle event-counting → wrap in `baby-phi/scripts/audit-jq-extract.sh <jsonl-file> <jq-filter>`. The helper runs the full pipeline internally; the orchestrator/retrospector invokes it as `bash /abs/audit-jq-extract.sh /tmp/audit.jsonl '<filter>'` — single Bash call, single allow-rule match (already covered by existing `Bash(bash /root/projects/phi/baby-phi/scripts/audit-tmp-*.sh*)` family if named with `audit-` prefix). Do NOT add `Bash(jq * | sort | uniq -c | sort -rn *)` as an allow-rule — would encourage the discouraged shape across other surfaces.

### 2.3 — Empirical observations from CH-13 (where docs and behavior diverge)

CH-13 telemetry surfaced cases where compound commands had each subcommand allow-listed yet still prompted. Per the docs they should auto-approve. Hypotheses (none confirmed):

1. **Trailing `2>&1` without a pipe**: 2 prompts on `cd ... && git diff X 2>&1` despite `cd` and `git diff` rules existing. The unusual redirect-without-downstream-pipe shape may interact with the matcher's tokenization in undocumented ways.
2. **Mixed operator chains**: longer compounds with a mix of `&&`, `;`, and `|` may trigger quirks even when each subcommand matches individually.

**Resolution**: rather than chase every quirk with more rules or hooks, switch to the granular principle. Granular invocations sidestep the entire compound-handling layer.

### 2.5 — Bash-check cluster: 3-cycle pattern (CH-13 → CH-07 → CH-08)

The bash-check rule has now failed across 3 consecutive cycles despite repeated standards-update fixes. Cycle-by-cycle history:

- **CH-13 retro §H1** added `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh:*)` (colon-form). 4-prompt friction. **Friction reduced** but not eliminated.
- **CH-07 retro §5 row 5** refined to space-form + paired `2>&1*` rule: `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh *)` and `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh 2>&1*)`. 3-prompt residual.
- **CH-08 retro §3.5 found 15-prompt regression** despite both rules in place. User-led mid-cycle edit (2026-05-08 07:36 UTC) broadened to `Bash(bash /root/projects/phi/baby-phi/scripts/*.sh *)` (drop `check-` prefix; rule applies to ALL `.sh` scripts under that directory). CH-NN+1 must validate.

**Hypothesis (CH-08 retro)**: the `*.sh` glob INSIDE a `Bash(prefix-expr *)` rule is interpreted as a literal `*.sh` filename pattern, NOT as a wildcard binding to the script name. In other words, `Bash(bash /abs/path/check-*.sh *)` matches commands whose script path **literally contains** `check-*.sh` (asterisk-included), not commands whose script path is a glob-expansion of `check-*.sh`. If true, rules MUST either:
- Use full literal script names (`Bash(bash /abs/path/check-doc-links.sh *)`, etc.) — verbose but matcher-deterministic. **Recommended workaround.**
- OR drop the `check-` prefix entirely (`Bash(bash /abs/path/*.sh *)`) — broader but the directory restriction still scopes acceptable invocations. CH-08 user adopted this form mid-cycle.

**Validation signal (per CH-08 retro §5 row 7)**: when a hot-allow-rule signature persists across **≥ 2 cycles** post-standards-update, the `permissions-audit` skill escalates the finding to `matcher-semantics-investigation` priority — the next standards update MUST include an empirical test-harness step (run the candidate rule pattern against representative invocations + capture telemetry) before the rule is committed.

### 2.6 — Bash-check cluster: 4-cycle pattern + matcher-bug-confirmed escalation (CH-14)

CH-14 retro (cycle hex `5803bb94`, 2026-05-08) confirmed the regression EVEN AFTER the CH-08 user-led mid-cycle broadening.

**4-cycle data series (PermissionRequest counts on the bash-check cluster):**

| Cycle | Hex | Rules in effect | Cluster prompts | Notes |
|---|---|---|---|---|
| CH-13 | `d4fe1b7c` | 1 colon-form rule (`check-*.sh:*`) | **4** | rule added |
| CH-07 | `cc912d07` | + paired space-form `*.sh *` + `2>&1*` | **3** | residual after refinement |
| CH-08 | `7cbe74a4` | unchanged + mid-cycle user broadening to `*.sh *` (drop `check-` prefix) at 07:36 UTC 2026-05-08 | **15** | regression — broadening did not match |
| CH-14 | `5803bb94` | unchanged from CH-08 close-time edit (`Bash(bash /root/projects/phi/baby-phi/scripts/*.sh *)` + `Bash(bash /root/projects/phi/baby-phi/scripts/*.sh 2>&1*)`) | **43** | no further mid-cycle edits; both broadened patterns failed to match |

**Empirical conclusion**: rule-pattern iteration inside `Bash(prefix-expr *)` does NOT close the bash-check cluster. The matcher's `*.sh` glob — whether scoped (`check-*.sh`) or unscoped (`*.sh`) — does not bind to a wildcard script name at runtime in a way that allows-form patterns can express.

**Escalation per CH-14 retro §5 row 7**: cluster classification flips from `matcher-semantics-investigation` to **`matcher-bug-confirmed`** (3+ cycles post-update is the threshold). The escalation rule **STOPS rule-pattern iteration** and mandates:

1. **Capture an isolated reproducer**: a minimal `settings.json` carrying the candidate rule + a representative invocation that the rule should match. CH-14 reproducer: rule `Bash(bash /root/projects/phi/baby-phi/scripts/*.sh *)` + invocation `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh 2>&1 | tail -3` → expected match → observed PermissionRequest fires.
2. **File an upstream Claude Code rule-matcher bug-report** with the reproducer.
3. **Propose a behaviourally-equivalent literal-script-name workaround**:
   ```jsonc
   "Bash(bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh*)",
   "Bash(bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh*)",
   "Bash(bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh*)",
   "Bash(bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh*)",
   "Bash(bash /root/projects/phi/baby-phi/scripts/audit-tmp-*.sh*)"
   ```
   Each rule names a single literal script (sidesteps the `*.sh` glob ambiguity entirely). The 5th rule covers the `audit-tmp-*.sh` family produced by orchestrator gate-4 cargo-test cardinality-extraction refactor (CH-14 retro Row 8).
4. **Empirical test-harness validation BEFORE commit** (per CH-08 retro Row 7 + CH-14 retro escalation): apply the candidate rules to a test settings.json; run the 4 CI guards' canonical invocations + 1 representative `audit-tmp-*` invocation; observe `tool-use.log`; verify 0 PermissionRequest fires for the cluster. Only then commit.
5. **Cross-cycle trend-analysis fairness**: once classified `matcher-bug-confirmed`, the next cycle's permissions-audit MUST mark the cluster as `external-bug-pending` and **subtract** it from the cross-cycle PermissionRequest count for trend reporting.

**Why STOP iterating**: CH-13 → CH-07 → CH-08 → CH-14 burned 4 cycle slots on rule-pattern attempts that all failed in the same way. The variance between the 4 attempts is in the rule pattern; the constant is the matcher behaviour. Iterating the variable while the constant is bugged wastes cycles.

### 2.7 — CH-15 5th-cycle validation: literal-script-name workaround empirically PASSES; cluster downgrades to `resolved-via-workaround`

CH-15 retro (cycle hex `c3f46f17`, 2026-05-08) is the first cycle to run under the 5 literal-script-name rules added at CH-14 close-time (per §2.6 above). Empirical result:

| Cycle | Hex | Rules in effect | Cluster prompts | Notes |
|---|---|---|---|---|
| CH-13 | `d4fe1b7c` | 1 colon-form rule (`check-*.sh:*`) | **4** | rule added |
| CH-07 | `cc912d07` | + paired space-form `*.sh *` + `2>&1*` | **3** | residual after refinement |
| CH-08 | `7cbe74a4` | unchanged + mid-cycle user broadening to `*.sh *` (drop `check-` prefix) | **15** | regression — broadening did not match |
| CH-14 | `5803bb94` | unchanged from CH-08 close-time edit | **43** | confirmed-bug; rules iteration-failed |
| **CH-15** | **`c3f46f17`** | **5 literal-script-name rules** (`check-doc-links.sh*`, `check-ops-doc-headers.sh*`, `check-phi-core-reuse.sh*`, `check-spec-drift.sh*`, `audit-tmp-*.sh*`) | **0** | **VALIDATION PASSED — cluster fully closed** |

**Lifecycle conclusion**: the bash-check cluster downgrades from `matcher-bug-confirmed` → **`resolved-via-workaround`**. The literal-script-name workaround empirically replaces the failed glob-form patterns. **No upstream Claude Code rule-matcher bug-report needed.**

**Permissions-audit skill v3 protocol** (added per CH-15 retro Row 8): when a cluster's PermissionRequest count drops to 0 for ≥ 1 cycle post-`matcher-bug-confirmed` workaround, mark it `resolved-via-workaround` and **drop** the cluster from cross-cycle trend tracking (it is no longer noise). If a regression appears in a future cycle (count > 0 post-workaround), re-elevate to `matcher-bug-confirmed` and file the upstream bug-report at that point.

**Validation footnote**: CH-15 also validated the new `Bash(bash /root/projects/phi/baby-phi/scripts/audit-tmp-*.sh*)` rule paired with the gate-4 `audit-tmp-cargo-counts.sh` script refactor (CH-14 retro Row 8). The CH-14 4-stage `cargo test | grep | sed | awk` pipeline cluster (14 prompts) is also at 0 prompts in CH-15 — both clusters resolved by the same literal-script-name approach.

### 2.4 — Empirical observation from CH-07 (redirect+pipe combo defeats single-`*` glob)

CH-07 telemetry (cycle hex `cc912d07`, 2026-05-07) surfaced a third matcher quirk:

- **Single-`*` glob in space-form rules does NOT span the redirect-and-pipe combo `2>&1 | tail -N`.** Specifically, the rule `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh *)` (added per CH-13 retro Row 6) matched bare invocations like `bash /abs/scripts/check-doc-links.sh` cleanly, but **failed to match** `bash /abs/scripts/check-doc-links.sh 2>&1 | tail -30` — 3 PermissionRequest prompts fired in CH-07 on the same root signature post-rule-addition.

**Root cause hypothesis**: the matcher's `*` glob in space-form patterns (`prefix *`) is not equivalent to the colon-form pattern (`prefix:*`) when the input contains `2>&1` redirects followed by `|` pipes. The colon form appears to consume more aggressively across the operator boundary; the space form stops at the redirect token.

**Resolution applied at CH-07 retro §5 row 5** (chunk-planner v5 effective 2026-05-07):

- Added paired rules to `settings.json` after the existing space-form rule:
  - `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh:*)` (colon form — captures the redirect+pipe combo).
  - `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh 2>&1*)` (explicit redirect-prefix capture as a belt-and-braces).

**Workaround for future rule authors**: when a rule must match commands that may carry trailing `2>&1` or `2>&1 | <viewer>`, prefer the **colon form** (`Bash(prefix:*)`) over the space form (`Bash(prefix *)`); OR add a paired rule `Bash(prefix 2>&1*)` alongside the space-form rule. Empirically the colon form is more permissive across operator boundaries.

**Validation signal**: CH-08+ retros track PermissionRequest count for `bash:/root/projects/phi/baby-phi/scripts/check-*` signatures — must be 0 to confirm the rule refinement landed. Per CH-07 retro §5 row 7, the `permissions-audit` skill carries a regression-protection step that escalates `rule-pattern-failed-validation` when a hot-allow-rule candidate from a prior retro continues to fire post-fix.

---

## §3 — CH-13 telemetry breakdown (the 312 PermissionRequest events)

| Category | Count | Cause | Status |
|---|---|---|---|
| Multi-line bash script fragments | ~150 (48%) | Newlines split into fragments (`LOG=...`, `while`, `fi`, `done`, `echo ""`, `matched=0`); fragments don't match any rule. | Closed by granular principle: write multi-line scripts to a file via Write tool, then `bash /abs/path/script.sh` as one invocation. |
| `cd <abs> && <cmd>` compounds | ~45 (14%) | Per docs should auto-approve (each subcommand matches a rule). Empirically a subset still prompts (trailing `2>&1`, mixed operators). | Closed by granular principle: use absolute paths in the command itself, no `cd`. |
| `git -C <abs-path> <subcmd>` | ~7 (2.2%) | No allow rule covered the `git -C` form (only `git status:*`, `git diff:*`, etc. existed). | Closed by 3 new allow rules: `Bash(git -C /root/projects/phi/{,baby-phi/,phi-core/} *)`. |
| Pre-Row-6 chained `bash /abs/scripts/check-*.sh` | ~12 (3.8%) | The pre-CH-13-retro allow list only had relative-path `Bash(bash scripts/check-*.sh)`; absolute paths weren't covered. | Closed by CH-13 retro Row 6's `Bash(bash /root/projects/phi/baby-phi/scripts/check-*.sh:*)` (now `*)` post-Occam's razor). |
| Other complex patterns | ~98 (31%) | Multi-stage pipelines, embedded subshells, loops embedded in audit/research scripts. | Closed by granular principle (decompose) + new utility rules (`xargs`, `stat`, `cat`, `diff`). |

---

## §4 — Anti-pattern catalogue (CH-13 verbatim examples)

| Anti-pattern | CH-13 example | Why it prompts | Granular alternative |
|---|---|---|---|
| Multi-line script | `LOG=/tmp/x.jsonl\nwhile read id; do\n  echo $id\ndone` | Newlines split; `LOG=...`, `while read id; do`, `done` each need a rule, none match. | Write script to `/abs/path/script.sh` via Write tool, then `bash /abs/path/script.sh` as one Bash call. |
| Chained `&&` checks | `bash /abs/check-doc-links.sh \| tail -1 && bash /abs/check-ops.sh \| tail -1 && ...` | 4 chained `&&` + 4 trailing `\| tail`; mixed-operator quirks trigger prompts. | 4 separate Bash calls, one per check script. |
| Trailing 2>&1 no pipe | `cd /abs && git diff --stat HEAD docs/ 2>&1` | The `2>&1` redirect without a downstream pipe is empirically a quirk-trigger. | `git -C /abs diff --stat HEAD docs/` (drop `2>&1`; or pair with `\| cat` / `\| head` if stderr+stdout merge needed). |
| cd-then-relative-path | `cd /abs && grep -rn 'X' modules/crates/` | Per docs auto-approves; mixed chain quirks; working-dir state fragile across agent boundaries. | `grep -rn 'X' /abs/modules/crates/` (absolute path; no cd). |
| 4-stage pipeline | `ls /abs/*.md \| xargs -I{} basename {} .md \| grep -oE '^[0-9]+' \| sort -u \| tail -10` | 4 pipes; per docs xargs is stripped; in practice multi-stage prompts. | Decompose: `ls /abs/*.md > /tmp/files.txt` (one call) → `cat /tmp/files.txt \| sort -u \| tail -10` (one call). Or write-to-script-then-bash for the full pipeline. |

---

## §5 — Settings.json changes applied 2026-05-04

### 5.1 — Colon-star → space-star migration

54 colon-star Bash rules in `.claude/settings.json` converted to space-star (Occam's razor; both forms are equivalent at end-of-pattern per the docs). No semantic change.

Verified: `jq -r '.permissions.allow[] // .permissions.deny[]' /root/projects/phi/.claude/settings.json | grep -c ':\*)'` → 0 (no colon-star remaining).

### 5.2 — Targeted rule additions

7 new Bash allow rules added:

```jsonc
"Bash(git -C /root/projects/phi *)",
"Bash(git -C /root/projects/phi/baby-phi *)",
"Bash(git -C /root/projects/phi/phi-core *)",
"Bash(xargs *)",
"Bash(stat *)",
"Bash(cat *)",
"Bash(diff *)",
```

**Excluded** per user feedback: `tee` — tee writes to a file, prompted approval is desired.

---

## §6 — Future work + open questions

1. **The "trailing 2>&1 without pipe" quirk** — 2 occurrences in CH-13. Documented in §4 anti-pattern catalogue. If recurs in CH-14 despite the granular discipline, investigate further (may warrant escalation to Anthropic as upstream feedback).
2. **The "mixed `&&` + `;` + `|` chains" quirk** — covered by the granular principle (don't chain).
3. **Hook-level granularity gate** (deferred) — possible to add a hook that denies commands with > 1 `&&`/`;` operator, > 2 pipe stages, or any newline. Risk: breaks legitimate single-newline scripts. Defer until evidence shows discipline alone is insufficient.
4. **MCP server-tool rules** — when MCP servers are added, ensure allow-list coverage for `mcp__server__tool` calls.
5. **xargs destructive-flag deny pattern** — if telemetry surfaces `xargs rm` invocations bypassing the rm deny pattern in `block-destructive-bash.sh`, add explicit deny. Defer until evidence.
6. **CH-14 validation** — target prompt count < 5 (vs CH-13's 312). The dominant lever is eliminating multi-line-script fragments via the write-to-file-then-bash pattern.

---

## §7 — Sister docs in `baby-phi/docs/specs/permissions/`

- `project-permissions-hardening-478b9384.md` — initial allow/deny + scope-edits + block-destructive-bash hooks setup.
- `tool-use-logging-and-permissions-audit-skill-18564835.md` — tool-use telemetry capture + permissions-audit skill.
- `granular-bash-discipline-ab19399b.md` — **THIS DOC** — matcher semantics + granular discipline + CH-13 prompt analysis.
