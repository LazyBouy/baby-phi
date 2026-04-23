#!/usr/bin/env bash
# Spec-drift guard.
#
# Fails CI if a requirement id referenced in test or production code no longer
# exists in docs/specs/v0/requirements/. This catches silent deletions of
# requirement ids that tests still depend on.
#
# Patterns guarded (grep -E):
#   R-ADMIN-[0-9]{2}-[A-Z][0-9]+           e.g. R-ADMIN-12-R1 (M5 page 12)
#   R-AGENT-[0-9]{2}-[A-Z][0-9]+           e.g. R-AGENT-a04-R2 (M6 pages)
#   R-SYS-[A-Za-z0-9]+-[A-Z0-9]+           e.g. R-SYS-s01-1, R-SYS-s02-4
#   R-NFR-[A-Z]+-[A-Z0-9]+                 e.g. R-NFR-PERF-1
#
# The middle segment must accept lowercase — system-flow ids are
# `s01`, `s02`, `s03`, `s05` with a lowercase `s` prefix; agent
# pages use `a01`..`a05`. M5/P0 broadened the pattern to catch
# R-SYS-s02-* / R-SYS-s03-* / R-SYS-s05-* references introduced
# by the memory-extraction + agent-catalog + template-C/D fire
# listeners. Prior pattern `[A-Z0-9]+` was uppercase-only and
# silently missed M1 R-SYS-s01-* references (fixed retroactively
# at M5/P0).
set -euo pipefail

cd "$(dirname "$0")/.."

SPEC_DIR="docs/specs/v0/requirements"
if [[ ! -d "$SPEC_DIR" ]]; then
  echo "spec-drift: requirements directory $SPEC_DIR not found; nothing to check yet."
  exit 0
fi

pattern='R-(ADMIN|AGENT|SYS|NFR)-[A-Za-z0-9]+-[A-Z0-9]+'

mapfile -t referenced < <(
  grep -REho "$pattern" \
    --include='*.rs' --include='*.ts' --include='*.tsx' --include='*.toml' \
    modules tests 2>/dev/null \
  | sort -u
)

if [[ ${#referenced[@]} -eq 0 ]]; then
  echo "spec-drift: no requirement ids referenced in code yet (pre-M1 — expected)."
  exit 0
fi

missing=()
for id in "${referenced[@]}"; do
  if ! grep -Rq "\\b$id\\b" "$SPEC_DIR"; then
    missing+=("$id")
  fi
done

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "spec-drift: the following requirement ids are referenced by code but no longer exist in $SPEC_DIR:"
  printf '  - %s\n' "${missing[@]}"
  exit 1
fi

echo "spec-drift: ${#referenced[@]} referenced ids all present in $SPEC_DIR. OK."
