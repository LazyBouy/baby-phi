#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BABY_PHI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$BABY_PHI_DIR"

# ── Build Input.md ─────────────────────────────────────────────────────────────

echo "# External Input" > Input.md
echo "" >> Input.md

# ── 1. Last run status ─────────────────────────────────────────────────────────
# Written at the end of each run so baby-phi knows if the previous run failed.

echo "## Last Run Status" >> Input.md
echo "<!-- failure log from previous run — read this first if you see FAILED -->" >> Input.md
echo "" >> Input.md
if [ -f "last_run_log.md" ]; then
    cat last_run_log.md >> Input.md
else
    echo "(last run succeeded)" >> Input.md
fi
echo "" >> Input.md
echo "---" >> Input.md
echo "" >> Input.md

# ── 2. GitHub issues (with comments) ──────────────────────────────────────────

echo "## Open Issues" >> Input.md
echo "" >> Input.md

if command -v gh &>/dev/null; then
    ISSUES=$(gh issue list \
        --repo LazyBouy/baby-phi \
        --label agent-input \
        --state open \
        --json number,title,body 2>/dev/null || echo "[]")

    if [ "$ISSUES" = "[]" ] || [ -z "$ISSUES" ]; then
        echo "(no open issues with agent-input label)" >> Input.md
    else
        echo "$ISSUES" | jq -c '.[]' | while IFS= read -r issue; do
            NUMBER=$(echo "$issue" | jq -r '.number')
            TITLE=$(echo "$issue" | jq -r '.title')
            BODY=$(echo "$issue" | jq -r '.body')

            printf "## Issue #%s: %s\n\n%s\n\n" "$NUMBER" "$TITLE" "$BODY" >> Input.md

            # Fetch comments for this issue
            COMMENTS=$(gh api "repos/LazyBouy/baby-phi/issues/${NUMBER}/comments" \
                --jq '.[] | "**@\(.user.login):** \(.body)"' 2>/dev/null || echo "")

            if [ -n "$COMMENTS" ]; then
                echo "### Comments" >> Input.md
                echo "$COMMENTS" >> Input.md
                echo "" >> Input.md
            fi
        done
    fi
else
    echo "(gh not available — skipping issue fetch)" >> Input.md
fi

# ── Run the agent ─────────────────────────────────────────────────────────────
# Capture output via tee so we can write it to last_run_log.md on failure.
# set +e prevents pipefail from aborting on non-zero; PIPESTATUS captures cargo's exit code.

set +e
cargo run 2>&1 | tee /tmp/baby_phi_run_output.txt
AGENT_EXIT=${PIPESTATUS[0]}
set -e

# ── Protect bootstrap core — revert any changes before committing ─────────────
# baby-phi cannot modify src/core/ or src/main.rs; evolve.sh enforces this.
# Any modifications are reverted here so they never reach git history.

CORE_DIRTY=false
for PROTECTED in src/core/kernel.rs src/core/mod.rs src/main.rs; do
    if ! git diff --quiet -- "$PROTECTED" 2>/dev/null; then
        echo "[BOOTSTRAP] Reverting protected file: $PROTECTED"
        git checkout -- "$PROTECTED"
        CORE_DIRTY=true
    fi
done
if [ "$CORE_DIRTY" = "true" ]; then
    echo "[BOOTSTRAP] Core files were modified and reverted. Extend via src/agent/ instead."
fi

# ── Write last run status log (for the next run to read) ──────────────────────

ITERATION=$(cat iteration_count 2>/dev/null || echo "?")
if [ "$AGENT_EXIT" -eq 0 ]; then
    echo "(last run succeeded)" > last_run_log.md
else
    {
        printf "**Status:** FAILED | **Exit code:** %s | **Iteration:** %s\n\n" "$AGENT_EXIT" "$ITERATION"
        echo "## Last 60 lines of output"
        printf '```\n'
        tail -60 /tmp/baby_phi_run_output.txt 2>/dev/null || echo "(no output captured)"
        printf '```\n'
    } > last_run_log.md
fi

# ── Commit changes (always — captures failure journal entries too) ─────────────
# Core files are already reverted above, so git add . is safe here.
# .gitignore excludes target/, .env, and other sensitive paths.
# Using git add . means any new folder baby-phi creates is automatically staged.

git add .

git diff --cached --quiet || \
    git commit -m "iteration ${ITERATION}: self-improvement"

exit $AGENT_EXIT
