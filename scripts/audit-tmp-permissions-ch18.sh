#!/usr/bin/env bash
# Permissions-audit data extraction for CH-18 retrospective (cycle hex c77937bc).
# Window: 2026-05-09T16:28:24Z → 2026-05-10T08:31:25Z (UTC).
# Reads .claude/tool-use.log (JSONL); emits §A–§G aggregate counts.

set -euo pipefail

LOG="/root/projects/phi/.claude/tool-use.log"
START="2026-05-09T16:28:24Z"
END="2026-05-10T08:31:25Z"

echo "=== CH-18 permissions-audit data extraction ==="
echo "Window: $START → $END"
echo

echo "--- §A — total event counts in window ---"
total=$(awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e)c++}END{print c+0}' "$LOG")
echo "Total tool calls: $total"

post=$(awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"event":"PostToolUse"/)c++}END{print c+0}' "$LOG")
echo "PostToolUse: $post"

fail=$(awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"event":"PostToolUseFailure"/)c++}END{print c+0}' "$LOG")
echo "PostToolUseFailure: $fail"

perm=$(awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"event":"PermissionRequest"/)c++}END{print c+0}' "$LOG")
echo "PermissionRequest: $perm"

echo
echo "--- §A — tool distribution in window ---"
awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e){match($0,/"tool":"[^"]*"/);t=substr($0,RSTART+8,RLENGTH-9);c[t]++}}END{for(k in c)print c[k]"\t"k}' "$LOG" | sort -rn

echo
echo "--- §B/§F — PermissionRequest sample (tool / input_signature pairs) in window ---"
awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"event":"PermissionRequest"/){match($0,/"tool":"[^"]*"/);t=substr($0,RSTART+8,RLENGTH-9);match($0,/"input_signature":"[^"]*"/);sig=substr($0,RSTART+19,RLENGTH-20);print t"\t"sig}}' "$LOG" | sort | uniq -c | sort -rn | head -30

echo
echo "--- §E — PostToolUseFailure sample (tool / input_signature) in window ---"
awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"event":"PostToolUseFailure"/){match($0,/"tool":"[^"]*"/);t=substr($0,RSTART+8,RLENGTH-9);match($0,/"input_signature":"[^"]*"/);sig=substr($0,RSTART+19,RLENGTH-20);print t"\t"sig}}' "$LOG" | sort | uniq -c | sort -rn | head -20

echo
echo "--- §A — unique Bash signatures in window (top 30) ---"
awk -F'"ts":"' -v s="$START" -v e="$END" 'NF>1{split($2,a,"\"");if(a[1]>=s && a[1]<=e && /"tool":"Bash"/){match($0,/"input_signature":"[^"]*"/);sig=substr($0,RSTART+19,RLENGTH-20);print sig}}' "$LOG" | sort | uniq -c | sort -rn | head -30

echo
echo "=== END extraction ==="
