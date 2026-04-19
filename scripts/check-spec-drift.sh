#!/usr/bin/env bash
# Spec-drift guard.
#
# Fails CI if a requirement id referenced in test or production code no longer
# exists in docs/specs/v0/requirements/. This catches silent deletions of
# requirement ids that tests still depend on.
#
# Patterns guarded (grep -E):
#   R-ADMIN-[0-9]{2}-[A-Z][0-9]+
#   R-AGENT-[0-9]{2}-[A-Z][0-9]+
#   R-SYS-[0-9]{2}-[A-Z][0-9]+
#   R-NFR-[A-Z]+-[A-Z0-9]+
set -euo pipefail

cd "$(dirname "$0")/.."

SPEC_DIR="docs/specs/v0/requirements"
if [[ ! -d "$SPEC_DIR" ]]; then
  echo "spec-drift: requirements directory $SPEC_DIR not found; nothing to check yet."
  exit 0
fi

pattern='R-(ADMIN|AGENT|SYS|NFR)-[A-Z0-9]+-[A-Z0-9]+'

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
