<!-- Last verified: 2026-05-03 by Claude Code -->

# Project permissions hardening — settings.json + hooks for `/root/projects/phi/`

**Plan archive token:** `478b9384` (generated 2026-05-03 at plan-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/permissions/project-permissions-hardening-478b9384.md` (slug-first naming convention, mirrors `agentic-workflow/multi-agent-chunk-pipeline-0853574c.md` precedent).
**Type:** policy / tooling. No source code changes; only `.claude/` config + hook scripts. No commits made automatically (user does that).
**Estimated effort:** ~0.5 engineer-day for the full landing + testing.

---

## Context

The multi-agent chunk pipeline (CH-11 was the first folder-style cycle) generates many tool calls per cycle. Today the project's permission state is:

- `~/.claude/settings.json` — 48 bytes, near-empty.
- `/root/projects/phi/.claude/settings.local.json` — 10 narrow allow rules (cargo subcommands, openssl, docker run). **Gitignored.**
- `.claude/settings.json` — **does not exist.**
- No hooks defined.
- No `permissions.deny` rules.
- No `permissions.defaultMode` set (falls back to `default`, which prompts on every non-allowed call).
- One known bug in the existing rule: `Bash(RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --all-targets)` is exact-match — the variant we actually run (`-j 4 --workspace --all-targets`) does NOT match because env-var prefixes do not auto-strip in pattern matching (per Anthropic permissions docs).

The goal is to (a) eliminate redundant permission prompts during multi-agent cycles, (b) maintain hard guardrails (project-scoped writes, no destructive bash without user prompt), and (c) document every permission line so it can be reviewed and revised over time.

The user's locked principles for this task:
1. **Permission Mode** — set explicit defaultMode appropriate for the workflow.
2. **Granular Allow/Deny Rules** — every rule is intentional + categorized.
3. **Path-Scoped File Permissions** — writes scoped to `/root/projects/phi/**` + sister roots (agent definitions, memory, plan files); never beyond.
4. **Harden with Hooks** — defense-in-depth on top of allow/deny rules.
5. **Specific, well-defined, well-documented** — every rule reviewable.

---

## §1 — Permission mode

**Decision: `defaultMode: "acceptEdits"`** at the project scope.

- `acceptEdits` auto-approves Edit/Write/MultiEdit calls + common FS commands (`mkdir`, `cp`, `mv`, `touch`, etc.) within the working directory and `additionalDirectories`.
- Bash commands NOT in the auto-approved set still require an explicit allow rule or prompt the user.
- This is the right mode given the user's "spare redundant prompts" goal: the multi-agent implementer touches 30–80 files per cycle in mechanical fixture cascades, and prompting per-edit defeats the workflow.
- Alternatives rejected: `default` (too prompty for cycle work), `bypassPermissions` (too dangerous; loses guardrails), `plan` (read-only; useless for implementation).

**`additionalDirectories`** extends auto-approve scope beyond the working dir:

```json
"additionalDirectories": [
  "/root/.claude/agents",
  "/root/.claude/skills",
  "/root/.claude/projects/-root-projects-phi/memory",
  "/root/.claude/plans"
]
```

These are the **sister roots** the multi-agent system legitimately writes to:
- `agents/` + `skills/` — for retrospective-driven prompt evolution.
- `memory/` — auto-memory persistence.
- `plans/` — plan-mode plan files.

Any write to a path NOT under the working dir or these additionalDirectories will fall back to `default` mode (prompt + check rules), and additionally be hard-blocked by the **scope-edits hook** (§4) for any path outside the union of allowed roots.

---

## §2 — Where the rules live

| File | In-git? | Purpose |
|---|---|---|
| `.claude/settings.json` (NEW) | **Yes** (team-shared) | Durable project policy: defaultMode, allow/deny rules, hooks. The single source of truth. |
| `.claude/settings.local.json` (existing) | No (gitignored) | Personal overrides only. After migration, contains `{}` or user's machine-specific tweaks. |
| `.claude/hooks/*.sh` (NEW) | **Yes** | Hook scripts (executable). Reviewable, testable. |
| `.claude/hooks/README.md` (NEW) | **Yes** | Documentation for the hooks: what they do, how to test, when to update. |

The 10 existing `settings.local.json` rules (cargo subcommands, openssl, docker run) are project-policy and **migrate** to `settings.json`. The `settings.local.json` is then minimized to `{}` (kept as a gitignored escape hatch for user-specific rules later).

---

## §3 — Allow rules (Bash) — categorized + documented

Each rule below has format `Bash(...)` + a one-line **why** + the **prompt-frequency reduction** it delivers per cycle.

### 3.1 — Cargo (workspace test/build/lint loop)

```jsonc
"Bash(/root/rust-env/cargo/bin/cargo test:*)",      // Runs the workspace tests at every phase boundary + final cycle re-audit.
"Bash(/root/rust-env/cargo/bin/cargo fmt:*)",       // Format-check at every phase boundary.
"Bash(/root/rust-env/cargo/bin/cargo build:*)",     // Workspace build (smoke test).
"Bash(/root/rust-env/cargo/bin/cargo check:*)",     // Faster type-check during implementation iteration.
"Bash(/root/rust-env/cargo/bin/cargo clippy:*)",    // Linter, both bare form + with flags.
"Bash(/root/rust-env/cargo/bin/cargo run:*)",       // Run baby-phi or phi-core binaries (smoke).
"Bash(/root/rust-env/cargo/bin/cargo doc:*)",       // Doc generation if needed.
"Bash(/root/rust-env/cargo/bin/cargo clean:*)",     // Disk-cleanup; orchestrator may run when target/ is huge.
```

**Critical: env-var-prefixed clippy.** Per the docs gotcha, env-var prefixes are NOT stripped from pattern matching. The existing `Bash(RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --all-targets)` rule fails to match the actual `-j 4 --workspace --all-targets` invocation. **Fix:**

```jsonc
"Bash(RUSTFLAGS=\"-Dwarnings\" /root/rust-env/cargo/bin/cargo clippy:*)",  // Wildcard on flags; covers every variant we actually run.
```

The `:*` suffix is the prefix-wildcard form — it matches any flag combination after `clippy`.

**Bare `cargo`** (without the explicit `/root/rust-env/cargo/bin/` path): some workflows shorthand-call. Add too for safety:

```jsonc
"Bash(cargo test:*)",
"Bash(cargo build:*)",
"Bash(cargo fmt:*)",
"Bash(cargo clippy:*)",
"Bash(cargo check:*)",
```

**Prompt frequency reduction:** roughly 8–15 prompts per phase boundary × 4 phases × cycle = ~50 prompts saved per cycle.

### 3.2 — Git (read-only operations only)

```jsonc
"Bash(git status:*)",       // Used at every phase review.
"Bash(git diff:*)",         // Diff inspection (orchestrator + implementer).
"Bash(git log:*)",          // Recent-commit context.
"Bash(git show:*)",         // Inspect specific commit.
"Bash(git grep:*)",         // Faster than `grep -r` for tracked files.
"Bash(git ls-files:*)",     // Enumerate tracked files.
"Bash(git rev-parse:*)",    // Resolve refs.
"Bash(git submodule status)", // Submodule state (baby-phi, phi-core).
"Bash(git config --get:*)", // Read config keys (NOT --set, NOT --unset).
"Bash(git check-ignore:*)", // Test gitignore patterns.
"Bash(git stash list)",     // List stashes (read-only).
```

**NOT included** (deliberately): `git commit`, `git push`, `git rebase`, `git reset`, `git merge`, `git checkout`, `git tag`, `git stash` (push form), `git config --set`, `git remote add`, `git branch -D`, `git clean -f`. These belong in §5 deny list.

**Prompt frequency reduction:** ~10–20 prompts per cycle.

### 3.3 — baby-phi CI guards

```jsonc
"Bash(bash scripts/check-doc-links.sh)",
"Bash(bash scripts/check-ops-doc-headers.sh)",
"Bash(bash scripts/check-phi-core-reuse.sh)",
"Bash(bash scripts/check-spec-drift.sh)",
"Bash(bash scripts/check-*.sh)",  // Catch-all for future check-* scripts.
```

The `bash scripts/check-*.sh` form covers any new guard script the team adds without requiring a settings update. Also closes Audit A's NOT-EXECUTED-IN-AUDIT gap from CH-11 (sub-agent auditors will be able to run these now).

**Prompt frequency reduction:** 4 × 2 (chunk-close + final cycle re-audit) = 8 prompts saved per cycle.

### 3.4 — `cd` into project subdirectories

```jsonc
"Bash(cd /root/projects/phi:*)",
"Bash(cd /root/projects/phi/baby-phi:*)",
"Bash(cd /root/projects/phi/phi-core:*)",
```

`cd` to working dir is auto-allowed by Claude Code (read-only Bash list). These rules cover the compound `cd /root/projects/phi/baby-phi && cargo test ...` pattern that the agents use repeatedly.

### 3.5 — Cycle-artifact creation utilities

```jsonc
"Bash(mkdir -p:*)",         // Cycle folder creation (planner agent).
"Bash(cp:*)",               // Plan archive copies.
"Bash(mv:*)",               // Rare; rename operations.
"Bash(touch:*)",            // Create placeholder files.
"Bash(openssl rand:*)",     // 8-hex cycle ID generation.
```

Note: `mkdir`, `cp`, `mv`, `touch` are **already** auto-approved under `acceptEdits` mode for paths within working-dir + additionalDirectories. The explicit rules here are belt-and-suspenders + cover the rare edge case where a path outside the auto-scope is involved.

### 3.6 — Read-only utilities (most are auto-approved by Claude Code, listed for clarity)

These are typically auto-approved without rules per Claude Code's built-in read-only list (`ls`, `cat`, `head`, `tail`, `grep`, `find`, `wc`, `diff`, `stat`, `du`). Listed here to make intent explicit for reviewers:

```jsonc
"Bash(awk:*)",              // Test-count aggregation.
"Bash(sed -n:*)",           // Read-only sed (no -i; -i is rejected by hook).
"Bash(jq:*)",               // JSON parsing (also used by hooks).
"Bash(echo:*)",             // Status messages.
"Bash(tr:*)",               // Transformations.
"Bash(sort:*)",
"Bash(uniq:*)",
"Bash(cut:*)",
"Bash(test -f:*)",          // File-existence checks.
"Bash(test -d:*)",
"Bash([ -f:*)",
"Bash([ -d:*)",
```

**Note on `sed -i`** (in-place edit): not allowed under sed rule (only `-n` form allowed). For real edits, agents use the `Edit` tool, not sed.

---

## §4 — Allow rules (Read / Edit / Write) — path-scoped

### 4.1 — Read

```jsonc
"Read(/**)",                                                 // Project-relative: anything under /root/projects/phi/**
"Read(//root/.claude/agents/**)",                            // Agent definitions (orchestrator reads + retro proposes updates).
"Read(//root/.claude/skills/**)",                            // Skills.
"Read(//root/.claude/projects/-root-projects-phi/memory/**)", // Auto-memory.
"Read(//root/.claude/plans/**)",                             // Plan-mode plan files.
"Read(//root/projects/phi/**)",                              // Belt-and-suspenders absolute form.
```

**Why two forms for the project?** `Read(/**)` is **project-relative** (gitignore syntax: leading single `/` means project root). `Read(//root/projects/phi/**)` is **absolute** (gitignore syntax: leading `//` means filesystem root). The double rule is defense-in-depth — if working dir changes mid-session, the absolute form still catches.

**Read is unrestricted within the union of these roots.** No deny on Read except for sensitive credentials (§5.4 file-deny).

### 4.2 — Edit / Write / MultiEdit

```jsonc
"Edit(/**)",                                                 // Project-relative: any project file.
"Edit(//root/.claude/agents/*.md)",                          // Agent prompts (retro-driven updates).
"Edit(//root/.claude/skills/*.md)",                          // Skills.
"Edit(//root/.claude/agents/_changelog.md)",                 // Version log.
"Edit(//root/.claude/projects/-root-projects-phi/memory/**)", // Memory writes.
"Edit(//root/.claude/plans/**)",                             // Plan file (during plan mode only).
"Write(/**)",
"Write(//root/.claude/agents/**)",
"Write(//root/.claude/skills/**)",
"Write(//root/.claude/projects/-root-projects-phi/memory/**)",
"Write(//root/.claude/plans/**)",
"MultiEdit(/**)",
"MultiEdit(//root/.claude/agents/*.md)",
"MultiEdit(//root/.claude/skills/*.md)",
```

**Hook enforcement (§5.5)** denies any Edit/Write/MultiEdit to paths outside this allow set, with an explicit reason. Belt-and-suspenders.

---

## §5 — Deny rules

### 5.1 — Destructive Bash

```jsonc
"Bash(rm -rf:*)",           // Even within project; orchestrator/user only.
"Bash(rm -fr:*)",           // Alt form.
"Bash(rm -r:*)",
"Bash(rm -f /:*)",          // Root-ish deletions.
"Bash(sudo:*)",             // Never need sudo.
"Bash(su:*)",
"Bash(chmod -R 777:*)",     // Overly permissive recursive perms.
"Bash(chown -R:*)",         // Ownership changes.
"Bash(dd:*)",               // Disk operations.
"Bash(mkfs:*)",             // Filesystem creation.
"Bash(>(:*",                // Process substitution (less common; deny as paranoia).
```

### 5.2 — Git destructive (user's lane)

```jsonc
"Bash(git commit:*)",       // User commits. Never orchestrator.
"Bash(git push:*)",          // User pushes.
"Bash(git reset --hard:*)", // Destructive.
"Bash(git checkout --:*)",  // Discards working-tree changes.
"Bash(git clean -f:*)",     // Force-clean.
"Bash(git clean -d:*)",     // Directory clean.
"Bash(git rebase:*)",        // Rewrites history.
"Bash(git merge:*)",         // Non-trivial state change.
"Bash(git tag:*)",           // Annotation; deferred to user.
"Bash(git branch -D:*)",    // Force-delete branch.
"Bash(git remote add:*)",   // Remote config.
"Bash(git remote rm:*)",
"Bash(git stash drop:*)",   // Permanent stash deletion.
"Bash(git stash pop:*)",    // Conflict-prone state change.
"Bash(git config --set:*)", // Config writes.
"Bash(git config --unset:*)",
"Bash(git config --replace-all:*)",
```

**Note:** `git fetch` is intentionally NOT denied — it's read-only network. We don't allow it explicitly either, so it'll prompt the user (rare; happens on push prep).

### 5.3 — Network (orchestrator should use WebFetch tool, not curl/wget)

```jsonc
"Bash(curl:*)",
"Bash(wget:*)",
"Bash(nc:*)",               // netcat.
"Bash(ssh:*)",              // Remote shell.
"Bash(scp:*)",              // Remote copy.
"Bash(rsync:*)",            // Remote sync.
"Bash(ftp:*)",
"Bash(sftp:*)",
"Bash(telnet:*)",
```

If we need to fetch a URL, use the `WebFetch` tool (with its own permission rules, see §6).

### 5.4 — Package install (system state changes)

```jsonc
"Bash(npm install:*)",
"Bash(npm i:*)",
"Bash(yarn add:*)",
"Bash(yarn install:*)",
"Bash(pnpm add:*)",
"Bash(pnpm install:*)",
"Bash(cargo install:*)",     // Global Rust binary install.
"Bash(pip install:*)",
"Bash(pip3 install:*)",
"Bash(brew install:*)",
"Bash(brew uninstall:*)",
"Bash(apt install:*)",
"Bash(apt-get install:*)",
"Bash(apt remove:*)",
"Bash(apt-get remove:*)",
"Bash(dpkg -i:*)",
```

If a chunk legitimately needs a new dependency, the user authorizes the install in their terminal.

### 5.5 — File-edit deny (sensitive paths)

```jsonc
"Edit(//root/.ssh/**)",
"Edit(//root/.aws/**)",
"Edit(//root/.gnupg/**)",
"Edit(//root/.netrc)",
"Edit(//root/.npmrc)",
"Edit(//root/.gitconfig)",
"Edit(//etc/**)",            // System config.
"Edit(//usr/**)",
"Edit(//var/**)",
"Edit(/.env)",               // Project env files.
"Edit(/.env.*)",             // Project env variants.
"Edit(/**/credentials.json)",
"Edit(/**/secrets.json)",
"Edit(/**/.git/config)",     // Block direct git config edits.
"Write(//root/.ssh/**)",
"Write(//root/.aws/**)",
"Write(//root/.gnupg/**)",
"Write(//root/.netrc)",
"Write(/.env)",
"Write(/.env.*)",
"Write(/**/credentials.json)",
"Write(/**/secrets.json)",
"Read(/.env)",               // Even reading env files prompts.
"Read(/.env.*)",
"Read(//root/.ssh/**)",
"Read(//root/.aws/**)",
```

The hook (§6.1) enforces this at the path level too — defense-in-depth.

---

## §6 — Hooks

Two hook scripts at `.claude/hooks/`:

### 6.1 — `scope-edits.sh` (PreToolUse on Edit / Write / MultiEdit)

**Purpose:** hard-deny any Edit/Write/MultiEdit whose `file_path` is outside the union of allowed roots. Belt-and-suspenders on top of the path allow rules.

**Allowed roots** (must be kept in sync with §4.2):
- `/root/projects/phi/**`
- `/root/.claude/agents/**`
- `/root/.claude/skills/**`
- `/root/.claude/projects/-root-projects-phi/memory/**`
- `/root/.claude/plans/**`

**Behavior:** if path is outside, emit `permissionDecision: "deny"` JSON with reason; else exit 0 (allow normal flow).

```bash
#!/bin/bash
# .claude/hooks/scope-edits.sh — PreToolUse for Edit / Write / MultiEdit.
# Hard-deny edits to paths outside the allowed roots.

set -euo pipefail

PAYLOAD=$(cat)
TOOL_NAME=$(echo "$PAYLOAD" | jq -r '.tool_name // ""')
FILE_PATH=$(echo "$PAYLOAD" | jq -r '.tool_input.file_path // .tool_input.path // ""')

# If no file_path (some MultiEdit shapes), let the rule layer handle it.
[[ -z "$FILE_PATH" ]] && exit 0

# Normalize: convert any leading // to / for consistent matching.
PATH_NORM="${FILE_PATH#//}"
PATH_NORM="/${PATH_NORM#/}"

allow_match() {
  case "$1" in
    /root/projects/phi/*) return 0 ;;
    /root/.claude/agents/*) return 0 ;;
    /root/.claude/skills/*) return 0 ;;
    /root/.claude/projects/-root-projects-phi/memory/*) return 0 ;;
    /root/.claude/plans/*) return 0 ;;
  esac
  return 1
}

if allow_match "$PATH_NORM"; then
  exit 0
fi

jq -n --arg path "$PATH_NORM" --arg tool "$TOOL_NAME" '{
  hookSpecificOutput: {
    hookEventName: "PreToolUse",
    permissionDecision: "deny",
    permissionDecisionReason: "scope-edits hook: \($tool) to \($path) is outside the allowed project + sister roots. Edits restricted to /root/projects/phi/**, /root/.claude/{agents,skills,plans}/**, /root/.claude/projects/-root-projects-phi/memory/**."
  }
}'
exit 0
```

### 6.2 — `block-destructive-bash.sh` (PreToolUse on Bash)

**Purpose:** regex-deny destructive commands even if the user accidentally adds an allow rule that covers them. Defense-in-depth on top of §5.1–5.4 deny rules.

**Patterns blocked** (regex, case-sensitive):

- `\brm\s+-[rRfF]+` — any rm with destructive flags.
- `\bsudo\b`, `\bsu\s` — privilege escalation.
- `\bgit\s+(commit|push|reset\s+--hard|rebase|merge|tag|clean\s+-[df])\b` — git destructive.
- `\bgit\s+checkout\s+--` — discards working tree.
- `\b(curl|wget|nc|ssh|scp|rsync|sftp|telnet)\b` — network I/O.
- `\b(npm|yarn|pnpm|pip|pip3|brew|apt|apt-get|dpkg)\s+(install|i|add|remove|i\s)` — package install.
- `\bcargo\s+install\b` — global Rust install.
- `\bdd\s+(if|of)=` — disk dump.
- `\bmkfs(\.|\b)` — filesystem creation.
- `\bchmod\s+-?R?\s*777\b` — overly permissive perms.
- `>\s*/dev/sd[a-z]` — write to raw disk device.

```bash
#!/bin/bash
# .claude/hooks/block-destructive-bash.sh — PreToolUse for Bash.
# Defense-in-depth: regex-deny destructive bash even if rules slip through.

set -euo pipefail

PAYLOAD=$(cat)
TOOL_NAME=$(echo "$PAYLOAD" | jq -r '.tool_name // ""')
COMMAND=$(echo "$PAYLOAD" | jq -r '.tool_input.command // ""')

[[ "$TOOL_NAME" != "Bash" ]] && exit 0
[[ -z "$COMMAND" ]] && exit 0

deny() {
  jq -n --arg cmd "$COMMAND" --arg reason "$1" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: "block-destructive-bash hook: \($reason). Command: \($cmd)"
    }
  }'
  exit 0
}

# Destructive rm
echo "$COMMAND" | grep -qE '\brm\s+-[rRfF]+'  && deny "rm with destructive flags blocked; use Edit/Write tool or ask user explicitly."
# Privilege escalation
echo "$COMMAND" | grep -qE '\bsudo\b|\bsu\s'  && deny "Privilege escalation blocked."
# Git destructive
echo "$COMMAND" | grep -qE '\bgit\s+(commit|push|reset\s+--hard|rebase|merge|tag|clean\s+-[df]|checkout\s+--)' && deny "Destructive git operation blocked; user owns commit/push/rebase."
# Network
echo "$COMMAND" | grep -qE '\b(curl|wget|nc|ssh|scp|rsync|sftp|telnet)\b' && deny "Network I/O blocked; use WebFetch tool for URL access."
# Package install
echo "$COMMAND" | grep -qE '\b(npm|yarn|pnpm)\s+(install|i|add)\b|\bpip3?\s+install\b|\bbrew\s+(install|uninstall)\b|\b(apt|apt-get)\s+(install|remove)\b|\bdpkg\s+-i\b|\bcargo\s+install\b' && deny "Package install blocked; user authorizes installs in their terminal."
# Disk / filesystem
echo "$COMMAND" | grep -qE '\bdd\s+(if|of)=|\bmkfs(\.|\b)|>\s*/dev/sd[a-z]' && deny "Disk-level operation blocked."
# Permissive chmod
echo "$COMMAND" | grep -qE '\bchmod\s+-?R?\s*777\b' && deny "chmod 777 blocked."

exit 0
```

### 6.3 — Hooks declaration in `settings.json`

```jsonc
"hooks": {
  "PreToolUse": [
    {
      "matcher": "Edit|Write|MultiEdit",
      "hooks": [
        { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/scope-edits.sh", "timeout": 5 }
      ]
    },
    {
      "matcher": "Bash",
      "hooks": [
        { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/block-destructive-bash.sh", "timeout": 5 }
      ]
    }
  ]
}
```

`$CLAUDE_PROJECT_DIR` is Claude Code's built-in env var pointing to the project root. `timeout: 5` seconds is generous for these tiny scripts (typical run < 100ms).

---

## §7 — `WebFetch` allow

The orchestrator + `claude-code-guide` agent occasionally fetch documentation. Allow trusted Anthropic domains:

```jsonc
"WebFetch(domain:code.claude.com)",        // Claude Code docs.
"WebFetch(domain:docs.anthropic.com)",     // Anthropic SDK docs.
"WebFetch(domain:github.com)",             // Public GH (e.g., anthropic/claude-code repo).
"WebFetch(domain:raw.githubusercontent.com)", // Raw file fetches from GH.
```

Other domains will prompt — appropriate for unknown URLs.

---

## §8 — `Agent` (subagent) allow

Multi-agent system relies on spawning subagents. Allow all the named agents:

```jsonc
"Agent(chunk-planner)",
"Agent(chunk-implementer)",
"Agent(chunk-auditor)",
"Agent(chunk-retrospector)",
"Agent(claude-code-guide)",
"Agent(Explore)",
"Agent(general-purpose)",
"Agent(Plan)",
```

Skipped: `statusline-setup` (not used in this workflow).

---

## §9 — Concrete `.claude/settings.json` (full file)

```jsonc
{
  "permissions": {
    "defaultMode": "acceptEdits",

    "additionalDirectories": [
      "/root/.claude/agents",
      "/root/.claude/skills",
      "/root/.claude/projects/-root-projects-phi/memory",
      "/root/.claude/plans"
    ],

    "allow": [
      // ── §3.1 cargo ──────────────────────────────────────────────────
      "Bash(/root/rust-env/cargo/bin/cargo test:*)",
      "Bash(/root/rust-env/cargo/bin/cargo fmt:*)",
      "Bash(/root/rust-env/cargo/bin/cargo build:*)",
      "Bash(/root/rust-env/cargo/bin/cargo check:*)",
      "Bash(/root/rust-env/cargo/bin/cargo clippy:*)",
      "Bash(/root/rust-env/cargo/bin/cargo run:*)",
      "Bash(/root/rust-env/cargo/bin/cargo doc:*)",
      "Bash(/root/rust-env/cargo/bin/cargo clean:*)",
      "Bash(RUSTFLAGS=\"-Dwarnings\" /root/rust-env/cargo/bin/cargo clippy:*)",
      "Bash(cargo test:*)",
      "Bash(cargo build:*)",
      "Bash(cargo fmt:*)",
      "Bash(cargo clippy:*)",
      "Bash(cargo check:*)",

      // ── §3.2 git read-only ──────────────────────────────────────────
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Bash(git show:*)",
      "Bash(git grep:*)",
      "Bash(git ls-files:*)",
      "Bash(git rev-parse:*)",
      "Bash(git submodule status)",
      "Bash(git config --get:*)",
      "Bash(git check-ignore:*)",
      "Bash(git stash list)",

      // ── §3.3 baby-phi CI guards ─────────────────────────────────────
      "Bash(bash scripts/check-doc-links.sh)",
      "Bash(bash scripts/check-ops-doc-headers.sh)",
      "Bash(bash scripts/check-phi-core-reuse.sh)",
      "Bash(bash scripts/check-spec-drift.sh)",
      "Bash(bash scripts/check-*.sh)",

      // ── §3.4 cd ─────────────────────────────────────────────────────
      "Bash(cd /root/projects/phi:*)",
      "Bash(cd /root/projects/phi/baby-phi:*)",
      "Bash(cd /root/projects/phi/phi-core:*)",

      // ── §3.5 cycle-artifact utilities ───────────────────────────────
      "Bash(mkdir -p:*)",
      "Bash(cp:*)",
      "Bash(mv:*)",
      "Bash(touch:*)",
      "Bash(openssl rand:*)",

      // ── §3.6 read-only utilities (most auto-approved; explicit for clarity)
      "Bash(awk:*)",
      "Bash(sed -n:*)",
      "Bash(jq:*)",
      "Bash(echo:*)",
      "Bash(tr:*)",
      "Bash(sort:*)",
      "Bash(uniq:*)",
      "Bash(cut:*)",
      "Bash(test -f:*)",
      "Bash(test -d:*)",
      "Bash([ -f:*)",
      "Bash([ -d:*)",

      // ── §4.1 Read ───────────────────────────────────────────────────
      "Read(/**)",
      "Read(//root/.claude/agents/**)",
      "Read(//root/.claude/skills/**)",
      "Read(//root/.claude/projects/-root-projects-phi/memory/**)",
      "Read(//root/.claude/plans/**)",
      "Read(//root/projects/phi/**)",

      // ── §4.2 Edit / Write / MultiEdit ───────────────────────────────
      "Edit(/**)",
      "Edit(//root/.claude/agents/*.md)",
      "Edit(//root/.claude/skills/*.md)",
      "Edit(//root/.claude/agents/_changelog.md)",
      "Edit(//root/.claude/projects/-root-projects-phi/memory/**)",
      "Edit(//root/.claude/plans/**)",
      "Write(/**)",
      "Write(//root/.claude/agents/**)",
      "Write(//root/.claude/skills/**)",
      "Write(//root/.claude/projects/-root-projects-phi/memory/**)",
      "Write(//root/.claude/plans/**)",
      "MultiEdit(/**)",
      "MultiEdit(//root/.claude/agents/*.md)",
      "MultiEdit(//root/.claude/skills/*.md)",

      // ── §7 WebFetch ─────────────────────────────────────────────────
      "WebFetch(domain:code.claude.com)",
      "WebFetch(domain:docs.anthropic.com)",
      "WebFetch(domain:github.com)",
      "WebFetch(domain:raw.githubusercontent.com)",

      // ── §8 Agent ────────────────────────────────────────────────────
      "Agent(chunk-planner)",
      "Agent(chunk-implementer)",
      "Agent(chunk-auditor)",
      "Agent(chunk-retrospector)",
      "Agent(claude-code-guide)",
      "Agent(Explore)",
      "Agent(general-purpose)",
      "Agent(Plan)"
    ],

    "deny": [
      // ── §5.1 destructive bash ───────────────────────────────────────
      "Bash(rm -rf:*)",
      "Bash(rm -fr:*)",
      "Bash(rm -r:*)",
      "Bash(rm -f /:*)",
      "Bash(sudo:*)",
      "Bash(su:*)",
      "Bash(chmod -R 777:*)",
      "Bash(chown -R:*)",
      "Bash(dd:*)",
      "Bash(mkfs:*)",

      // ── §5.2 git destructive ────────────────────────────────────────
      "Bash(git commit:*)",
      "Bash(git push:*)",
      "Bash(git reset --hard:*)",
      "Bash(git checkout --:*)",
      "Bash(git clean -f:*)",
      "Bash(git clean -d:*)",
      "Bash(git rebase:*)",
      "Bash(git merge:*)",
      "Bash(git tag:*)",
      "Bash(git branch -D:*)",
      "Bash(git remote add:*)",
      "Bash(git remote rm:*)",
      "Bash(git stash drop:*)",
      "Bash(git stash pop:*)",
      "Bash(git config --set:*)",
      "Bash(git config --unset:*)",
      "Bash(git config --replace-all:*)",

      // ── §5.3 network ────────────────────────────────────────────────
      "Bash(curl:*)",
      "Bash(wget:*)",
      "Bash(nc:*)",
      "Bash(ssh:*)",
      "Bash(scp:*)",
      "Bash(rsync:*)",
      "Bash(ftp:*)",
      "Bash(sftp:*)",
      "Bash(telnet:*)",

      // ── §5.4 package install ────────────────────────────────────────
      "Bash(npm install:*)",
      "Bash(npm i:*)",
      "Bash(yarn add:*)",
      "Bash(yarn install:*)",
      "Bash(pnpm add:*)",
      "Bash(pnpm install:*)",
      "Bash(cargo install:*)",
      "Bash(pip install:*)",
      "Bash(pip3 install:*)",
      "Bash(brew install:*)",
      "Bash(brew uninstall:*)",
      "Bash(apt install:*)",
      "Bash(apt-get install:*)",
      "Bash(apt remove:*)",
      "Bash(apt-get remove:*)",
      "Bash(dpkg -i:*)",

      // ── §5.5 sensitive file paths ───────────────────────────────────
      "Edit(//root/.ssh/**)",
      "Edit(//root/.aws/**)",
      "Edit(//root/.gnupg/**)",
      "Edit(//root/.netrc)",
      "Edit(//root/.npmrc)",
      "Edit(//root/.gitconfig)",
      "Edit(//etc/**)",
      "Edit(//usr/**)",
      "Edit(//var/**)",
      "Edit(/.env)",
      "Edit(/.env.*)",
      "Edit(/**/credentials.json)",
      "Edit(/**/secrets.json)",
      "Edit(/**/.git/config)",
      "Write(//root/.ssh/**)",
      "Write(//root/.aws/**)",
      "Write(//root/.gnupg/**)",
      "Write(//root/.netrc)",
      "Write(/.env)",
      "Write(/.env.*)",
      "Write(/**/credentials.json)",
      "Write(/**/secrets.json)",
      "Read(/.env)",
      "Read(/.env.*)",
      "Read(//root/.ssh/**)",
      "Read(//root/.aws/**)"
    ]
  },

  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/scope-edits.sh", "timeout": 5 }
        ]
      },
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/block-destructive-bash.sh", "timeout": 5 }
        ]
      }
    ]
  }
}
```

---

## §10 — Critical files

**Step 0 of execution (before any other file is written):**
- Token already generated at plan-open: `478b9384`.
- Copy this plan from `/root/.claude/plans/sharded-discovering-stearns.md` to `baby-phi/docs/specs/permissions/project-permissions-hardening-478b9384.md` per the user's archive instruction (mirrors `agentic-workflow/multi-agent-chunk-pipeline-0853574c.md` precedent).
- Create `baby-phi/docs/specs/permissions/_index.md` (NEW) indexing this plan + future revisions, with a verified-header. Same shape as `baby-phi/docs/specs/agentic-workflow/_index.md`.
- Verify with `bash baby-phi/scripts/check-doc-links.sh` (relative-link integrity).

**New (created during this landing):**
- `baby-phi/docs/specs/permissions/project-permissions-hardening-478b9384.md` — archived plan (Step 0).
- `baby-phi/docs/specs/permissions/_index.md` — index of permissions design docs (this plan + future iterations).
- `.claude/settings.json` — the file from §9 above.
- `.claude/hooks/scope-edits.sh` — script from §6.1, made executable (`chmod +x`).
- `.claude/hooks/block-destructive-bash.sh` — script from §6.2, made executable.
- `.claude/hooks/README.md` — documentation: what each hook does, how to test, when to update, why each pattern is in the deny list.

**Modified:**
- `.claude/settings.local.json` — minimized to `{}` (the 10 existing rules migrate to `settings.json` since they're project policy, not personal).

**Unchanged:**
- All source code, tests, docs, ADRs, drifts.
- Agent + skill files (the rules reference them but don't change them).
- Existing memory.

---

## §11 — Verification recipe

```bash
cd /root/projects/phi

# 1. Files in place + executable.
ls -l .claude/settings.json
ls -l .claude/hooks/scope-edits.sh .claude/hooks/block-destructive-bash.sh
test -x .claude/hooks/scope-edits.sh && echo "scope-edits.sh executable"
test -x .claude/hooks/block-destructive-bash.sh && echo "block-destructive-bash.sh executable"

# 2. JSON validity (settings.json must parse).
jq . .claude/settings.json > /dev/null && echo "settings.json: valid JSON"

# 3. Hook scripts handle empty stdin gracefully (smoke test).
echo '{"tool_name":"Bash","tool_input":{"command":"echo hello"}}' | bash .claude/hooks/block-destructive-bash.sh
echo '{"tool_name":"Edit","tool_input":{"file_path":"/root/projects/phi/baby-phi/CLAUDE.md"}}' | bash .claude/hooks/scope-edits.sh

# 4. Hook scripts deny known-bad inputs.
# Should output a JSON deny + exit 0.
echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /tmp/foo"}}' | bash .claude/hooks/block-destructive-bash.sh
echo '{"tool_name":"Edit","tool_input":{"file_path":"/etc/passwd"}}' | bash .claude/hooks/scope-edits.sh

# 5. Restart the Claude Code session (settings load at session start) and run a smoke task.
#    Expectation: routine tool calls (cargo test, git status, bash scripts/check-*.sh) execute without prompts.
#    Spawn a small implementer task (e.g., update a doc verified-header) and confirm 0 prompts during the agent's run.

# 6. Hook denial visible to user with reason.
#    Trigger an out-of-scope edit (e.g., ask Claude to edit /tmp/foo.md) — should be denied with a clear scope-edits hook reason.

# 7. Compare prompt count vs CH-11 baseline.
#    CH-11 had ~N prompts during the cycle (informal count). After this lands, run a small follow-up task and count.
#    Expect: 0 routine prompts; only architectural/commit-time prompts remain.
```

---

## §12 — What this plan does NOT do

- Does NOT modify project source code, tests, ADRs, drifts, concept docs.
- Does NOT change agent or skill prompts (those evolve via retrospectives).
- Does NOT add `bypassPermissions` mode anywhere — that's the dangerous mode and is explicitly disclaimed.
- Does NOT enable `dontAsk` mode (strict-deny) — too restrictive for active development.
- Does NOT touch the user-level `~/.claude/settings.json` — project scope only.
- Does NOT add MCP-tool permissions (no MCP servers in use here today).
- Does NOT add a `PostToolUse` telemetry hook — nice-to-have, deferred until we have a clear use case.

---

## §13 — Future revisions / open items for review

1. **Tighten Agent allow list** — once we know which agents are actually used in the cycle pipeline, remove unused ones (e.g., `statusline-setup`).
2. **PostToolUse logging hook** — add a tool-use log for retrospective metrics (count prompts saved per cycle, surface gaps). Deferred.
3. **MCP allow rules** — when an MCP server is added (e.g., for Linear / Slack integrations), add specific `mcp__*__*` rules. Today: none in use.
4. **Path-scope hook coverage** — `scope-edits.sh` currently checks `Edit / Write / MultiEdit` but not `NotebookEdit`. Add when needed (no notebooks in this project today).
5. **CI for the hook scripts** — run them in shellcheck + bats unit tests as part of `scripts/check-*.sh`. Deferred.
6. **Retrospective revisit** — after 2–3 multi-agent cycles under the new permissions config, retrospective should report: prompts-per-cycle delta, any false-positive denials, any false-negative escapes (rare but possible). Adjust the deny patterns accordingly.

---

## §14 — Estimated effort

~0.55 engineer-day:
- 0.05d — Step 0: archive plan to `baby-phi/docs/specs/permissions/project-permissions-hardening-478b9384.md` + create `_index.md`.
- 0.1d — write `.claude/settings.json` (essentially copy from §9 into the file).
- 0.15d — write `.claude/hooks/scope-edits.sh` + `block-destructive-bash.sh` + make executable.
- 0.05d — write `.claude/hooks/README.md`.
- 0.05d — minimize `settings.local.json` to `{}`.
- 0.15d — verification (run §11 recipe; restart session; smoke-test allow + deny + hook denial; spawn a small implementer task to validate prompt-count reduction).

After landing, the next multi-agent cycle (CH-12 or whichever) exercises the new config in production. A 2-line addendum in that cycle's retrospective records: prompts-per-cycle observed under new config + any false-positive/negative findings.
