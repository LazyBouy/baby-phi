#!/usr/bin/env bash
# wait-for-pid.sh — block until a PID is no longer running.
#
# Usage:
#   bash wait-for-pid.sh <pid> [interval_seconds] [max_seconds]
#
# Wraps the `until ! kill -0 $pid ...; do sleep N; done` shell idiom so the
# polling loop becomes a single Bash invocation. Per granular-bash-discipline
# §2.2 worked-example added 2026-05-17 (CH-02b-i-phi retro Row 4): the inline
# `until <check>; do sleep N; done` shape generates PermissionRequest fires
# because the matcher splits at the `;` and `until` is not allow-listed.
# Wrap-in-script is the canonical answer.
#
# Exit codes:
#   0 — the PID terminated within the budget (or never existed).
#   1 — invalid arguments.
#   2 — exceeded max_seconds budget; PID still running.

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: bash wait-for-pid.sh <pid> [interval_seconds=2] [max_seconds=300]" >&2
  exit 1
fi

pid="$1"
interval="${2:-2}"
max="${3:-300}"

elapsed=0
until ! kill -0 "$pid" 2>/dev/null; do
  if [[ "$elapsed" -ge "$max" ]]; then
    echo "wait-for-pid: PID $pid still running after ${max}s budget" >&2
    exit 2
  fi
  sleep "$interval"
  elapsed=$((elapsed + interval))
done

echo "wait-for-pid: PID $pid exited (waited ${elapsed}s)"
exit 0
