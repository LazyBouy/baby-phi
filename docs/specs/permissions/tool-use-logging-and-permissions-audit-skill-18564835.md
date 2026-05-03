<!-- Last verified: 2026-05-03 by Claude Code -->

# Tool-use logging + permissions-audit skill — closing the feedback loop

**Plan archive token:** `18564835` (generated 2026-05-03 at plan-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/permissions/tool-use-logging-and-permissions-audit-skill-18564835.md`.
**Type:** observability / process refinement. No source-code changes; touches `.claude/` config + hooks + skills + retrospector agent prompt + CLAUDE.md mirrors.
**Estimated effort:** ~1.05 engineer-day for the full landing + verification.

---

## Context

The prior permissions-hardening cycle (`project-permissions-hardening-478b9384.md`) shipped allow/deny rules + two PreToolUse hooks. §13 of that plan deferred two follow-ups:
- Row 2: PostToolUse logging hook for tool-use telemetry.
- Row 6: Retrospective revisit after 2–3 cycles, comparing prompts-per-cycle deltas + identifying false-positive denials and false-negative escapes.

Today the feedback loop is observational and conversation-bound: the orchestrator and user notice prompts and denials in real time, but nothing aggregates, persists, or trends across cycles. False-positive hook denials (e.g., the verification-time `rm -rf` hits we caught) and false-negative escapes (rare; surface as broken state) are detected only by recall.

The user explicitly wants a thorough refinement loop, not a basic logger. Goals:
1. **Capture every tool call's decision path** — what tool, what input, who decided (rule / hook / mode / user prompt), what verdict.
2. **Capture user-facing prompts** — even successful ones — so we can identify missing allow rules.
3. **Capture failures** — including hook denials with their reasons.
4. **Aggregate + classify** at end-of-cycle into actionable findings (hot allow-rule candidates, dead rules, false-positive hooks, workflow issues).
5. **Integrate into retrospective** — every cycle's retrospective gains a §3.5 Permissions audit section, fed by the skill's output.
6. **Cross-cycle trending** — compare metrics against prior cycles so drift is visible.
7. **Privacy + size discipline** — gitignored log, rotation at 10MB, sensitive-arg redaction, command/path truncation.
8. **Fail-safe** — logging hook must never block real work even on disk-full / lock-contention / malformed envelope.

---

## §1 — Architecture overview

Three pieces, wired together:

```
                                                         ┌──────────────────────┐
                                                         │ chunk-retrospector   │
                                                         │ (agent v1 → v2)      │
                                                         └──────────┬───────────┘
                                                                    │ invokes
                                                                    ▼
   ┌────────────────────┐    .claude/tool-use.log    ┌──────────────────────────┐
   │ log-tool-use.sh    │───────────────────────────▶│ permissions-audit skill  │
   │ (one script,       │   (JSONL, gitignored,      │ (.claude/skills/         │
   │  3 hook events)    │    rotated at 10MB)        │  permissions-audit.md)   │
   └─────────┬──────────┘                            └────────────┬─────────────┘
             │                                                    │ outputs
             │ wired in settings.json:                            ▼
             │ - PostToolUse                                ┌─────────────┐
             │ - PostToolUseFailure                         │ §3.5 of the │
             │ - PermissionRequest                          │ cycle's     │
             ▼                                              │ retro       │
   (every Bash/Edit/Write/Read/Grep/Glob call               └─────────────┘
    appends one JSONL line)
```

The hook captures; the skill analyzes; the retrospector integrates findings into the cycle retrospective; the user reviews + approves any rule changes; standards updates land via the existing retro → standards pipeline (CH-11 precedent).

---

## §2 — `log-tool-use.sh` (PostToolUse + PostToolUseFailure + PermissionRequest hook)

**One script, three event registrations.** The script differentiates events via a **positional argument passed in the hook command**. Each event registration in settings.json passes its own event name as `$1`. Belt-and-suspenders: if the stdin envelope happens to contain a `hook_event_name` field, the script reads it and validates it matches `$1` (logs warning to stderr on mismatch but trusts `$1` as source of truth). This makes the script schema-version-independent — if Claude Code changes the envelope shape across releases, the registration arg still works.

### 2.1 — Script responsibilities

1. **Read stdin JSON envelope.**
2. **Extract** these fields (using `jq` with `// ""` defaulting):
   - Event name (`PostToolUse` / `PostToolUseFailure` / `PermissionRequest`).
   - `tool_name`.
   - `tool_input` (the command for Bash, file_path for Edit/Write/MultiEdit, pattern for Grep, etc.).
   - `tool_output` summary (PostToolUse only — first 200 chars, redacted).
   - `duration_ms` (PostToolUse only).
   - `tool_use_id` (correlates events for the same call across PreToolUse → PostToolUse).
   - `turn_index`.
3. **Compute a normalized "input_signature"** for aggregation:
   - For Bash: extract first word of the command (the program), then first non-flag argument.
     Example: `cargo test -j 4 --workspace` → signature `cargo:test`. `bash scripts/check-doc-links.sh` → signature `bash:scripts/check-doc-links.sh`.
   - For Edit / Write: collapse the file path to its top-2 directory levels.
     Example: `/root/projects/phi/baby-phi/modules/crates/domain/src/...` → signature `/root/projects/phi/baby-phi/...`.
   - For Read / Grep / Glob: same path collapse.
   - Signature is for clustering in the audit skill, not for security decisions.
4. **Truncate** `tool_input` to 1000 chars (with `…` marker on truncation) for logging.
5. **Redact** environment-variable assignments containing `SECRET|TOKEN|PASSWORD|KEY|CREDENTIAL` (case-insensitive). Replace value with `<redacted>`. The whole rest of the command is preserved.
6. **Compose JSONL line** (one object) and append to `.claude/tool-use.log`.
7. **Rotate** if log file size > 10 MB before appending: `mv .claude/tool-use.log .claude/tool-use.log.1` (and shift `.1→.2 .. .4→.5`, dropping `.5`). Keep last 5 rotations.
8. **Concurrency safety:** acquire `flock` on `.claude/tool-use.log.lock` for the rotation+append critical section. Use `flock -n` (non-blocking) with 1s retry; if still locked, skip the log entry rather than block the workflow. **Never block the tool call.**
9. **Self-skip:** if the tool input refers to `.claude/tool-use.log*` itself (Read/Edit/Write on the log file), don't log — avoid meta-recursion.
10. **Always exit 0.** This is a logging hook; failure to log must not affect the workflow.

### 2.2 — JSONL schema

One JSON object per line. Schema:

```json
{
  "ts": "2026-05-04T01:23:45.678Z",
  "event": "PostToolUse | PostToolUseFailure | PermissionRequest",
  "tool": "Bash | Edit | Write | MultiEdit | Read | Grep | Glob | Agent | WebFetch | ...",
  "tool_use_id": "toolu_01...",
  "turn_index": 12,
  "input_signature": "cargo:test",
  "input_full": "cd /root/projects/phi/baby-phi && /root/rust-env/cargo/bin/cargo test -j 4 ...",
  "outcome": "success | failure | prompted",
  "duration_ms": 8421,
  "output_summary": "1319 passed; 0 failed; 1 ignored",
  "error_summary": null,
  "redacted": false,
  "version": 1
}
```

Field semantics:
- `event` — which hook event fired (PostToolUse / PostToolUseFailure / PermissionRequest).
- `outcome` — high-level result: `success` (PostToolUse), `failure` (PostToolUseFailure), `prompted` (PermissionRequest).
- `duration_ms` — populated only for PostToolUse + PostToolUseFailure.
- `output_summary` — first 200 chars of tool_output, line breaks normalized to spaces. Null for events without output.
- `error_summary` — populated for PostToolUseFailure (first 200 chars of error). Null otherwise.
- `redacted` — true if any env-var redaction was applied to `input_full`.
- `version` — schema version (currently 1; bump on breaking changes).

The `decided_by` (which gate approved the call) is **not directly observable** from PostToolUse alone. The audit skill infers it by correlating PermissionRequest + PostToolUse via `tool_use_id`:
- PermissionRequest seen → `decided_by: "user-prompt-approved"` or `"user-prompt-denied"` (depending on whether PostToolUse follows).
- No PermissionRequest seen → `decided_by: "auto-approved"` (mode or allow-rule). Ambiguous between mode + rule; the audit skill cross-references settings.json to disambiguate.

### 2.3 — Concrete script outline

```bash
#!/bin/bash
# .claude/hooks/log-tool-use.sh — PostToolUse / PostToolUseFailure / PermissionRequest.
# Append a JSONL telemetry record for every tool call. Never block the workflow.

set -uo pipefail   # NOT set -e: a single failure must not break the pipeline.

LOG_PATH="${CLAUDE_PROJECT_DIR:-/root/projects/phi}/.claude/tool-use.log"
LOCK_PATH="${LOG_PATH}.lock"
ROTATE_BYTES=$((10 * 1024 * 1024))  # 10 MB
KEEP_ROTATIONS=5

PAYLOAD=$(cat)

# Skip self-references to the log file itself.
case "$PAYLOAD" in
  *tool-use.log*) exit 0 ;;
esac

# Event name comes from the hook registration's positional arg (source of truth).
# settings.json wires this script three times, each with a different $1:
#   "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PostToolUse"
#   "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PostToolUseFailure"
#   "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PermissionRequest"
# If stdin happens to also carry hook_event_name, validate consistency (warn to stderr on mismatch, trust $1).
EVENT="${1:-unknown}"
ENVELOPE_EVENT=$(echo "$PAYLOAD" | jq -r '.hook_event_name // .event_name // empty')
if [[ -n "$ENVELOPE_EVENT" && "$ENVELOPE_EVENT" != "$EVENT" ]]; then
  echo "log-tool-use.sh: event mismatch — arg=$EVENT, envelope=$ENVELOPE_EVENT (trusting arg)" >&2
fi

# Extract fields with jq (defaulting on missing).
TOOL=$(echo "$PAYLOAD" | jq -r '.tool_name // ""')
TOOL_USE_ID=$(echo "$PAYLOAD" | jq -r '.tool_use_id // ""')
TURN_INDEX=$(echo "$PAYLOAD" | jq -r '.turn_index // 0')
DURATION_MS=$(echo "$PAYLOAD" | jq -r '.duration_ms // null')

# Tool input — varies by tool.
INPUT_FULL=$(echo "$PAYLOAD" | jq -r '
  if .tool_input.command then .tool_input.command
  elif .tool_input.file_path then .tool_input.file_path
  elif .tool_input.path then .tool_input.path
  elif .tool_input.pattern then .tool_input.pattern
  else (.tool_input | tojson)
  end // ""
')

# Redaction: strip env-var values for sensitive vars.
REDACTED=false
if echo "$INPUT_FULL" | grep -qE '\b(SECRET|TOKEN|PASSWORD|KEY|CREDENTIAL)[A-Z_]*=[^[:space:]]+'; then
  INPUT_FULL=$(echo "$INPUT_FULL" | sed -E 's/\b(SECRET|TOKEN|PASSWORD|KEY|CREDENTIAL)([A-Z_]*)=[^[:space:]]+/\1\2=<redacted>/g')
  REDACTED=true
fi

# Truncate input to 1000 chars.
if [[ ${#INPUT_FULL} -gt 1000 ]]; then
  INPUT_FULL="${INPUT_FULL:0:1000}…"
fi

# Compute input_signature (first word + first non-flag arg, or path collapse for file tools).
INPUT_SIGNATURE=$(compute_signature "$TOOL" "$INPUT_FULL")
# (compute_signature is a small inline helper — see §2.4 below)

# Outcome from event.
case "$EVENT" in
  PostToolUse) OUTCOME=success ;;
  PostToolUseFailure) OUTCOME=failure ;;
  PermissionRequest) OUTCOME=prompted ;;
  *) OUTCOME=unknown ;;
esac

# Output / error summary (first 200 chars, line breaks normalized).
OUTPUT_SUMMARY=$(echo "$PAYLOAD" | jq -r '.tool_output // null' | head -c 200 | tr '\n' ' ')
ERROR_SUMMARY=$(echo "$PAYLOAD" | jq -r '.error // .tool_error // null' | head -c 200 | tr '\n' ' ')

# Compose JSONL line.
LINE=$(jq -nc \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)" \
  --arg event "$EVENT" --arg tool "$TOOL" \
  --arg tool_use_id "$TOOL_USE_ID" --argjson turn "$TURN_INDEX" \
  --arg sig "$INPUT_SIGNATURE" --arg input "$INPUT_FULL" \
  --arg outcome "$OUTCOME" --argjson duration "$DURATION_MS" \
  --arg out "$OUTPUT_SUMMARY" --arg err "$ERROR_SUMMARY" \
  --argjson redacted $REDACTED \
  '{ts:$ts, event:$event, tool:$tool, tool_use_id:$tool_use_id, turn_index:$turn,
    input_signature:$sig, input_full:$input, outcome:$outcome,
    duration_ms:$duration, output_summary:$out, error_summary:$err,
    redacted:$redacted, version:1}'
)

# Acquire lock with 1s timeout; if locked, skip silently.
(
  if flock -w 1 9; then
    # Rotate if needed.
    if [[ -f "$LOG_PATH" ]] && [[ $(stat -c%s "$LOG_PATH" 2>/dev/null || echo 0) -gt $ROTATE_BYTES ]]; then
      for i in $(seq $((KEEP_ROTATIONS - 1)) -1 1); do
        [[ -f "${LOG_PATH}.${i}" ]] && mv "${LOG_PATH}.${i}" "${LOG_PATH}.$((i+1))"
      done
      mv "$LOG_PATH" "${LOG_PATH}.1"
    fi
    echo "$LINE" >> "$LOG_PATH"
  fi
) 9>"$LOCK_PATH"

exit 0
```

### 2.4 — `compute_signature` helper

Inline function in the script:

```bash
compute_signature() {
  local tool="$1" input="$2"
  case "$tool" in
    Bash)
      # First non-empty word + first non-flag argument.
      local cmd args first_word first_arg
      cmd=$(echo "$input" | awk '{$1=$1; print}')
      first_word=$(echo "$cmd" | awk '{print $1}')
      first_arg=$(echo "$cmd" | awk '{for(i=2;i<=NF;i++) if(substr($i,1,1)!="-") {print $i; exit}}')
      echo "${first_word##*/}:${first_arg:0:50}"
      ;;
    Edit|Write|MultiEdit|Read|NotebookEdit)
      # Path collapse: keep first 4 path segments.
      echo "$input" | awk -F/ '{out=""; for(i=1;i<=NF && i<=5;i++) out=out (i>1?"/":"") $i; print out "/..."}' 
      ;;
    Grep|Glob)
      # Pattern as-is, truncated.
      echo "${input:0:80}"
      ;;
    Agent|WebFetch)
      echo "$input" | head -c 80
      ;;
    *)
      echo "${tool}:${input:0:50}"
      ;;
  esac
}
```

### 2.5 — Hook registration in `settings.json`

Three additions to `hooks` block. **Each registration passes the event name as a positional arg** so the script can differentiate without relying on the stdin envelope's schema:

```json
{
  "PostToolUse": [
    {
      "matcher": ".*",
      "hooks": [
        { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PostToolUse", "timeout": 3 }
      ]
    }
  ],
  "PostToolUseFailure": [
    {
      "matcher": ".*",
      "hooks": [
        { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PostToolUseFailure", "timeout": 3 }
      ]
    }
  ],
  "PermissionRequest": [
    {
      "matcher": ".*",
      "hooks": [
        { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-tool-use.sh PermissionRequest", "timeout": 3 }
      ]
    }
  ]
}
```

`matcher: ".*"` matches every tool (Bash, Edit, Write, Read, Grep, Glob, Agent, WebFetch, MultiEdit, NotebookEdit, etc.). Timeout 3s is generous; the script runs in <100ms typically.

**Why arg-based, not stdin-based event detection:**
- The stdin JSON envelope's exact field name for the event (`hook_event_name`? `event_name`? something else?) is not authoritatively documented and may vary across Claude Code versions.
- The hook command line is fully under our control in `settings.json`. Passing `$1` is explicit + version-independent.
- The script still cross-checks the envelope (if `hook_event_name` is present, validate it matches `$1`) for diagnostic value. On mismatch, log warning + trust `$1`.

This makes the script robust to upstream schema changes: when Claude Code v.next ships and the envelope shape shifts, the script keeps working.

---

## §3 — `permissions-audit` skill

**Purpose:** read `.claude/tool-use.log` (+ rotations), filter by cycle window, classify findings, output a markdown report consumable by the retrospector.

### 3.1 — Inputs

The skill is invoked by the chunk-retrospector agent. Inputs:
- **Cycle hex** — for naming the report.
- **Cycle window** — start/end timestamps. Default behavior: derive from cycle folder file mtimes (`stat -c %Y plan.md` for start; `stat -c %Y cycle-audit.md` for end). If those aren't available, default to last 7 days.
- **Settings.json path** — for cross-referencing rules. Default: `$CLAUDE_PROJECT_DIR/.claude/settings.json`.
- **Prior retros** — optional list of prior `retrospective.md` paths for cross-cycle trending.

### 3.2 — Procedure

1. **Glob** for `.claude/tool-use.log*` files (current + rotations). Concatenate.
2. **Parse JSONL**: skip malformed lines (`jq -c '. // empty'`).
3. **Filter by timestamp range**: keep entries where `ts >= start AND ts <= end`.
4. **Compute aggregates** by `(event, tool, input_signature)`:
   - Total count.
   - First/last timestamp.
   - Sample full inputs (3 examples, deduplicated).
5. **Cross-reference settings.json** rules (allow + deny, flat list):
   - For each unique `(tool, input_signature)`: simulate rule matching.
     - If a deny rule matches → `decided_by: "deny-rule:<rule>"`.
     - Else if an allow rule matches → `decided_by: "allow-rule:<rule>"`.
     - Else → `decided_by: "auto-approved-by-mode"` (acceptEdits) or `"user-prompt"` (correlation by tool_use_id with PermissionRequest events).
   - Track which rules matched at least once (rule utilization map).
6. **Correlate PermissionRequest + PostToolUse by `tool_use_id`**:
   - If both events present → user prompted then approved.
   - PermissionRequest only → user prompted then denied (or call abandoned).
7. **Classify into report sections**:
   - **Tool distribution** — count + % per tool.
   - **Hot allow-rule candidates** — input_signatures with PermissionRequest count ≥ 3 and no settings.json allow-rule match. Propose a rule pattern derived from the signature.
   - **Auto-approved (mode-driven)** — input_signatures decided by `acceptEdits` mode without explicit rule. Visibility-only; no action.
   - **Allow-rule utilization** — every rule in settings.json with hit count this cycle. Rules with 0 hits are flagged.
   - **Hook denials** — PostToolUseFailure entries where `error_summary` contains "scope-edits hook" or "block-destructive-bash hook". Group by hook + reason.
   - **High-frequency rejected patterns** — input_signatures denied ≥ 5×. Likely a workflow issue, not a rule issue. Flag for design review.
   - **Cross-cycle trends** — if prior retros provided, extract their `Total tool calls observed` + `Hot allow-rule candidates` numbers and tabulate deltas.
8. **Output a markdown report** (full template in §3.4). Skill prints to stdout; calling agent embeds it.

### 3.3 — Output spec

```markdown
# Permissions Audit — Cycle <hex>

**Window:** <start ISO> → <end ISO> (UTC)
**Total tool calls observed:** <N>
**Total unique input_signatures:** <M>
**Log file size:** <bytes> (across <K> files: current + <K-1> rotations)

## §A — Tool distribution
> Share of total tool calls captured in the cycle window. Helps spot anomalies (e.g., Bash usage way up could indicate cycle drift toward shell-driven workflow).

| Tool | Calls | % of total |
|---|---|---|
| Bash | 412 | 47.3% |
| Read | 230 | 26.4% |
| Edit | 156 | 17.9% |
| ... | ... | ... |

## §B — Hot allow-rule candidates (≥ 3 prompts in cycle, no rule match)

| Pattern | Prompts | First seen | Sample input | Proposed rule |
|---|---|---|---|---|
| `cargo:nextest` | 5 | 2026-05-03T22:14Z | `cargo nextest run -p domain` | `Bash(cargo nextest:*)` |
| ... | ... | ... | ... | ... |

> **Recommendation:** add the proposed rules to `settings.json` allow list.

## §C — Auto-approved by mode (visibility only)

| Tool | Pattern | Count | Notes |
|---|---|---|---|
| Edit | `/root/projects/phi/baby-phi/...` | 87 | acceptEdits + project-relative path. Healthy. |
| ... | ... | ... | ... |

## §D — Allow-rule utilization

| Rule | Hits | Status |
|---|---|---|
| `Bash(cargo test:*)` | 12 | active |
| `Bash(cargo build:*)` | 0 | unused this cycle |
| `Bash(/root/rust-env/cargo/bin/cargo doc:*)` | 0 | unused for 3 cycles → removal candidate |
| ... | ... | ... |

## §E — Hook denials (review for false-positives)

| Hook | Pattern | Count | Sample reason | False-positive? |
|---|---|---|---|---|
| block-destructive-bash | `rm -rf` substring in test payload | 2 | "rm with destructive flags blocked" | yes (verification testing) |
| scope-edits | `/etc/passwd` | 1 | "outside the allowed project + sister roots" | no (correct deny) |
| ... | ... | ... | ... | ... |

> **Skill heuristic for false-positive flagging:** if the same pattern was denied multiple times in immediate succession (within 60s), flag as "likely test/verification, review."

## §F — High-frequency rejected patterns (workflow issue, not rule issue)

| Pattern | Reject count | Likely cause |
|---|---|---|
| (none this cycle) | | |

> Patterns appearing here suggest the agent or orchestrator is repeatedly trying something blocked. The fix is usually NOT to add an allow rule — it's to review the workflow.

## §G — Cross-cycle trends

| Metric | 2 cycles ago | 1 cycle ago | This cycle | Δ vs prior |
|---|---|---|---|---|
| Total tool calls | n/a | 612 | 871 | +259 (+42%) |
| Unique input_signatures | n/a | 84 | 96 | +12 |
| Hot allow-rule candidates | n/a | 0 | 1 | +1 |
| Hook denials | n/a | 0 | 3 | +3 (verification testing — expected) |
| Dead allow rules | n/a | 0 | 1 | +1 (cargo doc) |

## §H — Skill findings + proposed standards updates

> Roll these into the retrospective's §5 Standards updates table.

1. **Add allow rule** `Bash(cargo nextest:*)` (closes §B row 1).
2. **Remove allow rule** `Bash(/root/rust-env/cargo/bin/cargo doc:*)` (3 consecutive cycles of zero hits per §D).
3. **Investigate** the 2 block-destructive-bash false-positives in §E — recommend documenting verification-testing workaround in `.claude/hooks/README.md`.

---
*Generated by `.claude/skills/permissions-audit.md` v1 from `.claude/tool-use.log`*.
```

### 3.4 — Skill file structure

`.claude/skills/permissions-audit.md`:

```markdown
---
name: permissions-audit
description: Read .claude/tool-use.log + settings.json, classify findings (hot allow-rule candidates, dead rules, hook denials, workflow issues), output a markdown report for the retrospector. Used at end-of-cycle.
version: 1
---

# permissions-audit

Analyze tool-use telemetry for the cycle window + cross-reference settings.json rules. Produce a markdown report with proposed standards updates.

## Inputs (caller provides)

1. **Cycle hex** — for the report header.
2. **Cycle window**: `start_ts` (ISO 8601), `end_ts` (ISO 8601). If absent, default to:
   - `start_ts = stat -c %Y <cycle folder>/plan.md`
   - `end_ts = stat -c %Y <cycle folder>/cycle-audit.md` (or `now` if cycle-audit not yet written).
3. **Settings path** — default `$CLAUDE_PROJECT_DIR/.claude/settings.json`.
4. **Prior retros** — list of paths to prior `retrospective.md` files (for §G cross-cycle trends). Default: empty (skip §G).

## Procedure

1. Glob `.claude/tool-use.log*`.
2. Parse JSONL; filter by `[start_ts, end_ts]`.
3. Aggregate by `(event, tool, input_signature)`.
4. Cross-reference settings.json allow + deny rules.
5. Correlate PermissionRequest ↔ PostToolUse by `tool_use_id`.
6. Classify into §A–§H per the output spec.
7. Print markdown to stdout.

## Cross-reference algorithm

For each unique `(tool, input_signature)` aggregate:
- Convert tool + signature back into a sample command string (use the most-recent `input_full` from the aggregate).
- For each `allow` and `deny` rule pattern in `settings.json`: simulate Claude Code's matching:
  - `Bash(prefix:*)` → does the command start with `prefix `?
  - `Bash(exact)` → does the command equal `exact`?
  - `Edit(path-glob)` → does the file_path match the gitignore-style pattern?
  - `WebFetch(domain:X)` → does the URL host equal `X`?
- Record the FIRST matching rule (Claude Code's precedence: deny first, then allow).
- If no rule matches → categorize by mode: `acceptEdits` covers Edit/Write/MultiEdit + safe FS commands inside working-dir + additionalDirectories.

## Heuristics

- **Hot candidate:** PermissionRequest count ≥ 3 AND no allow-rule match AND no auto-approve-by-mode.
- **Dead rule:** allow-rule with 0 hits this cycle. **Removal candidate** only after ≥ 3 consecutive cycles of zero hits — the prior-retro input is required for this judgment. Without prior retros, mark "unused this cycle" (informational).
- **False-positive hook denial flag:** same pattern denied ≥ 2 times within 60 seconds. Suggests verification/testing context.
- **High-frequency reject:** same input_signature denied ≥ 5 times in cycle. Workflow issue.

## Output

Markdown report per the §H output spec. Single stdout stream.

## Quality bar

- Every section §A–§H present (with "(none this cycle)" when empty).
- Every "Proposed rule" cell shows valid Claude Code rule syntax.
- Every "False-positive?" cell has a yes/no/unclear classification with one-line rationale.
- Cross-cycle table populated with available data; cells marked "n/a" when prior cycle data missing.

## Reference

Settings precedence + rule pattern syntax: prior plan archive `baby-phi/docs/specs/permissions/project-permissions-hardening-478b9384.md` §2 + §3.
```

---

## §4 — Retrospector agent: v1 → v2

The retrospector currently doesn't invoke any skill. Bumping version to integrate `permissions-audit`.

### 4.1 — Frontmatter change

```yaml
---
name: chunk-retrospector
description: ...
model: opus
tools: Read, Grep, Glob, Bash, Write
skills: permissions-audit             # ← NEW (was: skills: (none))
version: 2                             # ← bumped from 1
---
```

### 4.2 — Procedure step addition

Insert between current steps 5 and 6 (between "Grep prior retrospectives" and "Draft 6 sections"):

> **5b. Invoke the `permissions-audit` skill.** Pass:
> - cycle hex
> - cycle window (start = `stat -c %Y plan.md`; end = `stat -c %Y cycle-audit.md`)
> - prior retros: glob `baby-phi/docs/specs/plan/build/*/retrospective.md` sorted by mtime, take last 3
>
> Capture the skill's stdout output. Verify the report covers §A–§H. Do NOT inline the entire report into the retrospective body — extract the actionable findings (§B Hot candidates, §D dead rules, §E false-positive hook flags, §H findings) into the retrospective's new §3.5 section. Append the full audit report as an appendix at the end of the retrospective doc, marked `## Appendix — Permissions audit (full)`.

### 4.3 — New retro section §3.5

Insert between current §3 and §4 in the retrospective file structure:

```markdown
## §3.5 — Permissions audit findings

> Auto-extracted from the `permissions-audit` skill (full report appended at end of doc).
> Window: <start> → <end> (UTC). Tool calls: <N>. Unique signatures: <M>.

### Hot allow-rule candidates
<from skill §B>

### Dead allow rules (removal candidates)
<from skill §D, only rules with N ≥ 3 cycles of zero hits>

### Hook false-positive flags
<from skill §E, only rows flagged "yes">

### Cross-cycle trend signal
<one-paragraph commentary on skill §G — e.g., "Tool calls grew 42% vs prior cycle, mostly Read+Bash; no concerning patterns.">

### Audit-driven standards updates proposed
<from skill §H, also rolled into §5 below for orchestrator/user review>
```

### 4.4 — Quality bar additions

Add to the retrospector's "Quality bar embedded in prompt":

> - **§3.5 must be present** with all four sub-sections (Hot candidates / Dead rules / Hook false-positive flags / Cross-cycle trend) — populated from the skill, OR explicitly "(none this cycle)" if nothing surfaced.
> - **Appendix must be present** — the full skill output, copy-pasted verbatim.
> - **Standards updates from §3.5 must appear in §5 too** — don't double-track; cross-reference.

---

## §5 — `.gitignore` addition

Append to `/root/projects/phi/.gitignore` (or create if missing):

```
# Tool-use telemetry log + rotations + lock file (per plan tool-use-logging-and-permissions-audit-skill-18564835.md)
.claude/tool-use.log
.claude/tool-use.log.*
.claude/tool-use.log.lock
```

---

## §6 — Critical files

**Step 0 of execution (before any other file is written):**
- Token already generated at plan-open: `18564835`.
- Copy this plan from `/root/.claude/plans/sharded-discovering-stearns.md` to `baby-phi/docs/specs/permissions/tool-use-logging-and-permissions-audit-skill-18564835.md`.
- Add a row to `baby-phi/docs/specs/permissions/_index.md` for this plan (above the prior `478b9384` row; `478b9384` row's Status changes to "Accepted (extended by 18564835)").
- Verify with `bash baby-phi/scripts/check-doc-links.sh`.

**New (created during this landing):**
- `baby-phi/docs/specs/permissions/tool-use-logging-and-permissions-audit-skill-18564835.md` — archived plan (Step 0).
- `.claude/hooks/log-tool-use.sh` — three-event capture script (executable).
- `.claude/skills/permissions-audit.md` — analysis skill.
- `/root/projects/phi/.gitignore` — extended (or created) with the tool-use.log entries.

**Modified:**
- `.claude/settings.json` — three new hook event registrations under `hooks` (PostToolUse, PostToolUseFailure, PermissionRequest).
- `.claude/agents/chunk-retrospector.md` — frontmatter `version: 1 → 2`; `skills: (none) → permissions-audit`; procedure step 5b inserted; retro template §3.5 + Appendix sections added; quality bar bullets added.
- `.claude/agents/_changelog.md` — 3 new entries (retrospector v1→v2; new skill `permissions-audit` v1; new hook `log-tool-use.sh` v1).
- `.claude/hooks/README.md` — add a new section "log-tool-use.sh (PostToolUse + PostToolUseFailure + PermissionRequest)" documenting purpose, JSONL schema, redaction policy, rotation, lock contention, self-skip rule. Cross-reference the skill.
- `baby-phi/docs/specs/permissions/_index.md` — add row for this plan.
- `/root/projects/phi/CLAUDE.md` + `/root/projects/phi/baby-phi/CLAUDE.md` — append a one-paragraph note in the "Multi-agent chunk pipeline" section: "Telemetry: every tool call is logged to `.claude/tool-use.log` (gitignored) by the `log-tool-use.sh` PostToolUse hook. The `permissions-audit` skill analyzes the log at retro time and feeds §3.5 of the cycle retrospective."

**Unchanged:**
- All source code, tests, ADRs, drifts, concept docs.
- The two PreToolUse hooks (`scope-edits.sh`, `block-destructive-bash.sh`) — untouched.
- The allow/deny rules in settings.json — untouched.
- The plan-time per-chunk-template — untouched (retro template lives in retrospector agent prompt, not in the plan template).

---

## §7 — Verification recipe

```bash
cd /root/projects/phi

# === 1. Files in place + executable ===
ls -l .claude/hooks/log-tool-use.sh
test -x .claude/hooks/log-tool-use.sh && echo "log-tool-use.sh: executable ✓"
ls -l .claude/skills/permissions-audit.md
ls -l .gitignore && grep -E 'tool-use\.log' .gitignore && echo ".gitignore: tool-use.log entries present ✓"

# === 2. settings.json valid + has 3 new hook event registrations ===
jq . .claude/settings.json > /dev/null && echo "settings.json: valid JSON ✓"
jq '.hooks | keys' .claude/settings.json
# Expect: ["PermissionRequest", "PostToolUse", "PostToolUseFailure", "PreToolUse"]

# === 3. Hook script smoke tests (each event type) ===
# PostToolUse smoke
echo '{"hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"echo hi"},"tool_output":"hi","duration_ms":42,"tool_use_id":"toolu_test_1","turn_index":0}' \
  | bash .claude/hooks/log-tool-use.sh
tail -1 .claude/tool-use.log | jq .
# Expect: JSONL line with event=PostToolUse, tool=Bash, outcome=success, duration_ms=42.

# PostToolUseFailure smoke
echo '{"hook_event_name":"PostToolUseFailure","tool_name":"Bash","tool_input":{"command":"false"},"error":"exit 1","tool_use_id":"toolu_test_2","turn_index":0}' \
  | bash .claude/hooks/log-tool-use.sh
tail -1 .claude/tool-use.log | jq '.event, .outcome, .error_summary'
# Expect: "PostToolUseFailure", "failure", "exit 1"

# PermissionRequest smoke
echo '{"hook_event_name":"PermissionRequest","tool_name":"Bash","tool_input":{"command":"new-cmd --help"},"tool_use_id":"toolu_test_3","turn_index":0}' \
  | bash .claude/hooks/log-tool-use.sh
tail -1 .claude/tool-use.log | jq '.event, .outcome'
# Expect: "PermissionRequest", "prompted"

# === 4. Redaction smoke ===
echo '{"hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"SECRET_TOKEN=abc123 cargo run"},"tool_output":"","duration_ms":1,"tool_use_id":"toolu_redact","turn_index":0}' \
  | bash .claude/hooks/log-tool-use.sh
tail -1 .claude/tool-use.log | jq '.input_full, .redacted'
# Expect: "SECRET_TOKEN=<redacted> cargo run", true

# === 5. Self-skip smoke ===
SIZE_BEFORE=$(wc -l < .claude/tool-use.log)
echo '{"hook_event_name":"PostToolUse","tool_name":"Read","tool_input":{"file_path":"/root/projects/phi/.claude/tool-use.log"},"tool_use_id":"toolu_self","turn_index":0}' \
  | bash .claude/hooks/log-tool-use.sh
SIZE_AFTER=$(wc -l < .claude/tool-use.log)
test "$SIZE_BEFORE" = "$SIZE_AFTER" && echo "self-skip: no log entry written for log-file Read ✓"

# === 6. Rotation smoke (force low threshold via env var; document this in the script's leading comment) ===
# Script exposes ROTATE_BYTES_OVERRIDE for testing. Set to 1024 to force rotation on small writes.
# (Implementation detail: implementer adds this env var hook.)

# === 7. Skill smoke ===
# Read .claude/skills/permissions-audit.md and confirm it parses + has the documented sections.
cat .claude/skills/permissions-audit.md | grep -E '^## ' | head -10
# Expect: Inputs, Procedure, Cross-reference algorithm, Heuristics, Output, Quality bar, Reference.

# === 8. End-to-end (manual; run after restart so settings reload) ===
# Restart Claude Code session. Trigger 5–10 tool calls naturally (cargo test, edits, etc.). Then:
wc -l .claude/tool-use.log
# Expect: ≥ 5 lines, one per tool call.
jq -c '. | {event, tool, outcome}' .claude/tool-use.log | sort | uniq -c | sort -rn
# Expect: distribution showing PostToolUse:Bash:success, PostToolUse:Edit:success, etc.

# === 9. Skill end-to-end ===
# Once a real cycle runs (CH-12 or any future), spawn chunk-retrospector. Confirm:
# - Retro file at <cycle folder>/retrospective.md has §3.5.
# - Appendix at end has the full audit report.
# - Standards updates in §5 reference §3.5 findings.

# === 10. CI guards still green ===
cd /root/projects/phi/baby-phi
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh
# All four should exit 0.
```

---

## §8 — What this plan does NOT do

- Does NOT modify the existing PreToolUse hooks (`scope-edits.sh`, `block-destructive-bash.sh`) — those stay v1.
- Does NOT change any allow/deny rule in `settings.json` — only ADDS the hooks block entries.
- Does NOT modify project source code, tests, ADRs, drifts, concept docs.
- Does NOT auto-apply rule changes proposed by the audit. Every proposal flows through retrospective → user-review → standards update (CH-11 precedent).
- Does NOT capture file contents or environmental context — only command/path strings (truncated, redacted).
- Does NOT introduce a database or external service. The log is a flat file; the skill is a markdown procedure invoked by an agent.
- Does NOT add a CI job to validate the hook script. Plan §13 of the prior permissions plan flagged shellcheck CI as a deferred item; same defer applies here.
- Does NOT capture orchestrator-internal events (subagent spawns, internal reasoning) — only user-facing tool calls.

---

## §9 — Future revisions / open items

1. **Per-cycle log slicing** — add a `cycle_id` field to the JSONL schema, populated by orchestrator-side context. Today the skill correlates by timestamp window; with a `cycle_id` it could be exact. Defer until cycle window has caused a real misclassification.
2. **Permission-prompt outcome correlation** — add a fourth event type (or use Stop hook) to capture the user's eventual decision after a PermissionRequest. Today we infer from PostToolUse presence/absence; explicit capture would be more reliable.
3. **Trend dashboards** — generate an HTML or markdown dashboard rolling up audits across N cycles. Defer until ≥ 5 cycles of data exist.
4. **Privacy escalation** — if the project ever handles personal data flowing through Bash commands (e.g., names, emails), expand the redaction patterns. Document the policy.
5. **Shellcheck + bats CI for hooks** — promote `.claude/hooks/*.sh` to a CI guard. Same defer as prior plan.
6. **Skill-level cross-cycle agreement check** — if the same standards update is proposed by 2+ consecutive retros and not applied, escalate to user with stronger language.
7. **MCP tool capture** — when an MCP server is added, ensure the hook captures `mcp__server__tool` calls correctly (the schema should already accommodate it via the generic `tool_name` field; verify on first MCP integration).

---

## §10 — Estimated effort

~1.05 engineer-day:

- 0.05d — Step 0: archive plan to `baby-phi/docs/specs/permissions/tool-use-logging-and-permissions-audit-skill-18564835.md` + update `_index.md`.
- 0.30d — write `.claude/hooks/log-tool-use.sh` (the §2.3 + §2.4 script with all branches, redaction, rotation, locking, self-skip) + `chmod +x`.
- 0.40d — write `.claude/skills/permissions-audit.md` (the full §3.4 procedure + cross-reference algorithm + output template + heuristics).
- 0.10d — `.claude/agents/chunk-retrospector.md` v1 → v2 update (frontmatter + procedure step 5b + new template §3.5 + Appendix + quality bar bullets).
- 0.05d — `.claude/settings.json` hooks block additions + `.gitignore` entries.
- 0.05d — `.claude/agents/_changelog.md` 3 new entries + `.claude/hooks/README.md` extension + CLAUDE.md addenda.
- 0.10d — verification (run §7 recipe sections 1–7; defer §7.8 + §7.9 to first real cycle).

After landing, the next multi-agent cycle (CH-12 likely) exercises the loop end-to-end. CH-12's retrospective will be the first to include §3.5 with real audit data. Cycle-3 (CH-13) will be the first to have prior-retro data populating the cross-cycle trend table.

A 2–3 line addendum in CH-12's cycle-audit will record:
- Did the hook capture the cycle's tool calls cleanly (any malformed lines, lock contentions, self-skip violations)?
- Did the skill output the full §A–§H structure?
- Were the proposed standards updates actionable (i.e., did the user approve any)?
