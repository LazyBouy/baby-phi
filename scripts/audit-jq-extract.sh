#!/usr/bin/env bash
# audit-jq-extract.sh — extract + aggregate JSONL telemetry log entries.
#
# Usage:
#   bash audit-jq-extract.sh <jsonl-file> <jq-filter> [head_N=30]
#
# Wraps the `jq -r '<filter>' file | sort | uniq -c | sort -rn | head -N`
# 4-stage pipeline so retrospector cross-cycle-aggregation becomes a single
# Bash invocation. Per granular-bash-discipline §2.2 worked-example added
# 2026-05-18 (CH-02c-i-phi retro Row P6): the inline 4-stage pipeline shape
# generates PermissionRequest fires because the 2-stage cap is enforced
# empirically; wrap-in-script is the canonical answer.
#
# The <jq-filter> is the body inside `jq -r '...'` — e.g.:
#   'select(.event == "PostToolUse") | .tool_input // empty'
#
# Output:
#   N <signature-string>   (one per line; descending by count)
#
# Exit codes:
#   0 — pipeline ran to completion (output may be empty if filter matched nothing).
#   1 — invalid arguments.
#   2 — input file does not exist.

set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: bash audit-jq-extract.sh <jsonl-file> <jq-filter> [head_N=30]" >&2
  exit 1
fi

jsonl_file="$1"
jq_filter="$2"
head_n="${3:-30}"

if [[ ! -f "$jsonl_file" ]]; then
  echo "audit-jq-extract: file not found: $jsonl_file" >&2
  exit 2
fi

jq -r "$jq_filter" "$jsonl_file" \
  | sort \
  | uniq -c \
  | sort -rn \
  | head -n "$head_n"
