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

### 2.3 — Empirical observations from CH-13 (where docs and behavior diverge)

CH-13 telemetry surfaced cases where compound commands had each subcommand allow-listed yet still prompted. Per the docs they should auto-approve. Hypotheses (none confirmed):

1. **Trailing `2>&1` without a pipe**: 2 prompts on `cd ... && git diff X 2>&1` despite `cd` and `git diff` rules existing. The unusual redirect-without-downstream-pipe shape may interact with the matcher's tokenization in undocumented ways.
2. **Mixed operator chains**: longer compounds with a mix of `&&`, `;`, and `|` may trigger quirks even when each subcommand matches individually.

**Resolution**: rather than chase every quirk with more rules or hooks, switch to the granular principle. Granular invocations sidestep the entire compound-handling layer.

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
