#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BABY_PHI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$BABY_PHI_DIR"

# ── Fetch GitHub issues → Input.md ────────────────────────────────────────────

echo "# External Input" > Input.md
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
        echo "$ISSUES" | jq -r '.[] | "## Issue #\(.number): \(.title)\n\n\(.body)\n"' >> Input.md
    fi
else
    echo "(gh not available — skipping issue fetch)" >> Input.md
fi

# ── Run the agent ─────────────────────────────────────────────────────────────

cargo run; AGENT_EXIT=$?

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

# ── Commit changes (always — captures failure journal entries too) ─────────────

ITERATION=$(cat iteration_count 2>/dev/null || echo "?")

git add journal.md iteration_count 2>/dev/null || true
git add src/agent/ 2>/dev/null || true
git add Cargo.toml Cargo.lock 2>/dev/null || true
git add LEARNINGS.md 2>/dev/null || true

git diff --cached --quiet || \
    git commit -m "iteration ${ITERATION}: self-improvement"

exit $AGENT_EXIT
