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

cargo run

# ── Commit changes ────────────────────────────────────────────────────────────

ITERATION=$(cat iteration_count 2>/dev/null || echo "?")

git add journal.md iteration_count 2>/dev/null || true
git add src/agent.rs src/main.rs 2>/dev/null || true
git add Cargo.toml Cargo.lock 2>/dev/null || true
git add LEARNINGS.md 2>/dev/null || true

git diff --cached --quiet || \
    git commit -m "iteration ${ITERATION}: self-improvement"
