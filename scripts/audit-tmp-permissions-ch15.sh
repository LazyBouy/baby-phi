#!/usr/bin/env bash
# CH-15 permissions-audit harness.
# Reads .claude/tool-use.log within the cycle window, cross-references settings.json,
# and emits an §A–§H markdown report to stdout.

set -euo pipefail

CYCLE_HEX="c3f46f17"
START_TS="2026-05-08T16:26:02Z"
END_TS="2026-05-08T19:14:50Z"
LOG_PATH="/root/projects/phi/.claude/tool-use.log"
SETTINGS_PATH="/root/projects/phi/.claude/settings.json"
OUT_DIR="/tmp/perm-audit-${CYCLE_HEX}"
mkdir -p "${OUT_DIR}"
WINDOW_FILE="${OUT_DIR}/window.jsonl"
AGG_FILE="${OUT_DIR}/agg.tsv"

# 1. Filter by window.
jq -c "select(.ts >= \"${START_TS}\" and .ts <= \"${END_TS}\")" "${LOG_PATH}" \
  2>/dev/null > "${WINDOW_FILE}" || true

TOTAL=$(wc -l < "${WINDOW_FILE}")
LOG_BYTES=$(stat -c %s "${LOG_PATH}")

echo "TOTAL=${TOTAL}"
echo "LOG_BYTES=${LOG_BYTES}"
echo

# 2. §A — Tool distribution
echo "=== TOOL DISTRIBUTION ==="
jq -r '.tool' "${WINDOW_FILE}" | sort | uniq -c | sort -rn
echo

# 3. Event distribution
echo "=== EVENT DISTRIBUTION ==="
jq -r '.event' "${WINDOW_FILE}" | sort | uniq -c | sort -rn
echo

# 4. PermissionRequest entries (potential hot allow-rule candidates)
echo "=== PERMISSION REQUESTS BY (tool,signature) ==="
jq -r 'select(.event=="PermissionRequest") | [.tool, .input_signature] | @tsv' "${WINDOW_FILE}" \
  | sort | uniq -c | sort -rn

echo
echo "=== PERMISSION REQUEST FULL INPUTS (most recent 3 per signature) ==="
jq -r 'select(.event=="PermissionRequest") | [.input_signature, .input_full] | @tsv' "${WINDOW_FILE}" \
  | sort -u | head -50

echo
echo "=== POSTTOOLUSEFAILURE (hook denials) ==="
jq -r 'select(.event=="PostToolUseFailure") | [.tool, .input_signature, .error_summary] | @tsv' "${WINDOW_FILE}" \
  | sort | uniq -c | sort -rn | head -30

echo
echo "=== BASH-CHECK CLUSTER PROBE (CH-14 rule validation) ==="
echo "-- PermissionRequest entries with check-*.sh in input_full --"
jq -r 'select(.event=="PermissionRequest" and (.input_full | tostring | test("check-.*\\.sh"))) | [.ts, .input_signature, .input_full] | @tsv' "${WINDOW_FILE}"
echo
echo "-- Total PermissionRequests for bash check-*.sh signature --"
jq -r 'select(.event=="PermissionRequest" and (.input_full | tostring | test("bash .*scripts/check-.*\\.sh|bash scripts/check-"))) | .input_full' "${WINDOW_FILE}" | wc -l

echo
echo "=== AUDIT-TMP-CARGO-COUNTS CLUSTER PROBE (CH-14 retro Row 8 rule validation) ==="
jq -r 'select(.event=="PermissionRequest" and (.input_full | tostring | test("audit-tmp-cargo-counts"))) | [.ts, .input_full] | @tsv' "${WINDOW_FILE}"
echo "Count:"
jq -r 'select(.event=="PermissionRequest" and (.input_full | tostring | test("audit-tmp-cargo-counts"))) | .input_full' "${WINDOW_FILE}" | wc -l

echo
echo "=== TOOL-USE-IDS WITH PERMISSIONREQUEST + POSTTOOLUSE (approved-after-prompt) ==="
jq -r '.tool_use_id + " " + .event' "${WINDOW_FILE}" | sort | uniq | awk '{print $1}' | sort | uniq -c | awk '$1>=2 {print}' | head -30

echo
echo "=== AGENT INVOCATIONS ==="
jq -r 'select(.tool=="Task") | .input_full' "${WINDOW_FILE}" | head -30 | cut -c1-200

echo
echo "=== HOTTEST BASH SIGNATURES (PermissionRequest, top 30) ==="
jq -r 'select(.event=="PermissionRequest" and .tool=="Bash") | .input_signature' "${WINDOW_FILE}" \
  | sort | uniq -c | sort -rn | head -30
