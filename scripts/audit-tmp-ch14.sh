#!/usr/bin/env bash
# CH-14 permissions audit — extracts cycle window data from .claude/tool-use.log
# Window: 2026-05-08T10:03:20Z (plan.md mtime) → 2026-05-08T14:29:59Z (cycle-audit.md mtime)
set -euo pipefail

LOG=/root/projects/phi/.claude/tool-use.log
OUT=/tmp/ch14-audit
mkdir -p "$OUT"
START="2026-05-08T10:03:20Z"
END="2026-05-08T14:29:59Z"

# Filter cycle-window entries.
jq -c --arg s "$START" --arg e "$END" '. as $r | select(.ts >= $s and .ts <= $e and .version == 1)' "$LOG" > "$OUT/cycle.jsonl" 2>/dev/null || true

echo "===== TOTAL ENTRIES IN WINDOW ====="
wc -l "$OUT/cycle.jsonl"

echo
echo "===== EVENT BREAKDOWN ====="
jq -r '.event' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn

echo
echo "===== TOOL DISTRIBUTION (PostToolUse) ====="
jq -r 'select(.event=="PostToolUse") | .tool' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn

echo
echo "===== PermissionRequest entries (signatures) ====="
jq -r 'select(.event=="PermissionRequest") | "\(.tool)\t\(.input_signature)\t\(.input_full)"' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn | head -40

echo
echo "===== PostToolUseFailure entries ====="
jq -r 'select(.event=="PostToolUseFailure") | "\(.tool)\t\(.input_signature)\t\(.error_summary)"' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn | head -40

echo
echo "===== Top input signatures across all events ====="
jq -r '"\(.event)\t\(.tool)\t\(.input_signature)"' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn | head -30

echo
echo "===== Bash signatures with high frequency (PostToolUse only) ====="
jq -r 'select(.event=="PostToolUse" and .tool=="Bash") | .input_signature' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn | head -25

echo
echo "===== Bash check-*.sh hits (verify CH-08 retro Row 7 cluster) ====="
jq -r 'select(.tool=="Bash") | "\(.event)\t\(.input_full)"' "$OUT/cycle.jsonl" | grep -E "check-(doc-links|ops-doc-headers|phi-core-reuse|spec-drift)" | sort | uniq -c | sort -rn | head -20

echo
echo "===== Sample input_full values for high-count bash signatures ====="
jq -r 'select(.event=="PostToolUse" and .tool=="Bash") | .input_signature' "$OUT/cycle.jsonl" | sort | uniq -c | sort -rn | head -10 | awk '{$1=""; print substr($0,2)}' | while read -r SIG; do
  echo "--- SIG: $SIG ---"
  jq -r --arg s "$SIG" 'select(.event=="PostToolUse" and .tool=="Bash" and .input_signature==$s) | .input_full' "$OUT/cycle.jsonl" | head -3
done

echo
echo "===== bash-check cluster sigs (check-*.sh) PermissionRequest count specifically ====="
jq -r 'select(.event=="PermissionRequest" and .tool=="Bash" and (.input_full|test("check-(doc-links|ops-doc-headers|phi-core-reuse|spec-drift)")))' "$OUT/cycle.jsonl" | wc -l
