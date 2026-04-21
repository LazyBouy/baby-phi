#!/usr/bin/env bash
# Ops-doc-header guard.
#
# Walks every markdown file under docs/ops/ and
# docs/specs/v0/implementation/m*/operations/ and asserts that line 1
# carries the `<!-- Last verified: YYYY-MM-DD by Claude Code -->`
# header (same convention as the implementation docs tree).
#
# Prevents ops docs from drifting out of the "versioned with the code"
# convention. M2/P8 makes this a hard gate — ops docs are the front
# line for incident response, so stale ones bleed operator time.
#
# Scope: the header check only. A parallel `**Status: [EXISTS|PLANNED
# Mn/Pn|CONCEPTUAL]**` tag check is a future upgrade (tracked in P8
# notes); M0/M1 ops docs don't yet uniformly carry the status tag, so
# hard-gating on it would retroactively break CI.
set -euo pipefail

cd "$(dirname "$0")/.."

missing_headers=0
total=0

mapfile -t ops_files < <(
  find docs/ops -maxdepth 1 -type f -name '*.md' 2>/dev/null
  find docs/specs/v0/implementation -type f -path '*/operations/*.md' 2>/dev/null
)

if [[ ${#ops_files[@]} -eq 0 ]]; then
  echo "check-ops-doc-headers: no ops docs under docs/ops/ or */operations/ (pre-M0 — expected)."
  exit 0
fi

for md_file in "${ops_files[@]}"; do
  total=$((total + 1))
  first_line=$(head -1 "$md_file")
  if [[ ! "$first_line" =~ Last\ verified:\ [0-9]{4}-[0-9]{2}-[0-9]{2}\ by ]]; then
    echo "check-ops-doc-headers: $md_file missing 'Last verified: YYYY-MM-DD by …' header on line 1"
    missing_headers=$((missing_headers + 1))
  fi
done

if [[ $missing_headers -gt 0 ]]; then
  echo "check-ops-doc-headers: FAIL — scanned $total file(s); $missing_headers missing 'Last verified' headers."
  exit 1
fi

echo "check-ops-doc-headers: all $total ops doc(s) carry the 'Last verified' header. OK."
