#!/usr/bin/env bash
# Permissions audit script for CH-17 retrospective.
# Reads .claude/tool-use.log, filters by cycle window, aggregates findings.

set -u

LOG="/root/projects/phi/.claude/tool-use.log"
START_TS="2026-05-09T14:11:29Z"
END_TS="2026-05-09T14:14:06Z"
OUT="/tmp/audit-cycle-40c4d759.jsonl"

echo "=== Step 1: filter log by cycle window ==="
echo "Window: ${START_TS} → ${END_TS}"
echo "Log size: $(stat -c %s ${LOG}) bytes"
echo "Total entries in log: $(wc -l < ${LOG})"

# Filter — note: the cycle window is < 3 minutes per the timestamps, but the
# actual cycle work spanned much longer. The plan mtime reflects the LAST plan
# write (iter-2 + 2 trivial-1L verified-header amendments at gate-3 close).
# Walk back to identify the actual cycle start by scanning early CH-17 entries.

# First: find earliest entry mentioning "ch-17" or "40c4d759" to get true window.
EARLIEST=$(grep -E 'ch-17|40c4d759|live-sse-tail' "${LOG}" | head -1 | jq -r '.ts' 2>/dev/null)
LATEST=$(grep -E 'ch-17|40c4d759|live-sse-tail' "${LOG}" | tail -1 | jq -r '.ts' 2>/dev/null)
echo "Earliest CH-17-related entry: ${EARLIEST}"
echo "Latest CH-17-related entry: ${LATEST}"
echo

# Use the broader window to capture the full cycle.
if [[ -n "${EARLIEST}" && -n "${LATEST}" ]]; then
  START_TS="${EARLIEST}"
  END_TS="${LATEST}"
fi
echo "Effective window: ${START_TS} → ${END_TS}"

jq -c "select(.ts >= \"${START_TS}\" and .ts <= \"${END_TS}\")" "${LOG}" > "${OUT}" 2>/dev/null
echo "Filtered entries: $(wc -l < ${OUT})"
echo

echo "=== Step 2: tool distribution (§A) ==="
jq -r '.tool' "${OUT}" | sort | uniq -c | sort -rn
echo

echo "=== Step 2b: event distribution ==="
jq -r '.event' "${OUT}" | sort | uniq -c | sort -rn
echo

echo "=== Step 3: PermissionRequest entries (§B candidates) ==="
PR_COUNT=$(jq -r 'select(.event == "PermissionRequest")' "${OUT}" | wc -l)
echo "Total PermissionRequest entries: $((PR_COUNT / $(jq -c 'select(.event=="PermissionRequest")' "${OUT}" | head -1 | wc -l 2>/dev/null || echo 1)))"
jq -c 'select(.event == "PermissionRequest")' "${OUT}" | head -20
echo
echo "PermissionRequest count by signature:"
jq -r 'select(.event == "PermissionRequest") | "\(.tool):\(.input_signature // .input_full // "n/a")"' "${OUT}" | sort | uniq -c | sort -rn | head -30
echo

echo "=== Step 4: PostToolUseFailure entries (§E hook denials) ==="
jq -r 'select(.event == "PostToolUseFailure") | "\(.tool):\(.input_signature // .input_full // "n/a") | reason: \(.error_summary // "n/a")"' "${OUT}" | sort | uniq -c | sort -rn | head -30
echo

echo "=== Step 5: Bash signature samples (top 30 by frequency) ==="
jq -r 'select(.tool == "Bash") | .input_signature // .input_full // "n/a"' "${OUT}" | sort | uniq -c | sort -rn | head -30
echo

echo "=== Step 6: Total entries in window ==="
echo "Total: $(wc -l < ${OUT})"
echo "Unique tools: $(jq -r '.tool' "${OUT}" | sort -u | wc -l)"
echo "Unique signatures: $(jq -r '.input_signature // .tool' "${OUT}" | sort -u | wc -l)"
