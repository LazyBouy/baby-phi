#!/usr/bin/env bash
# Doc-link guard.
#
# Walks every markdown file under docs/specs/v0/implementation/**/*.md and
# asserts that every relative link (`]({path})` or `]({path}#{anchor})`)
# resolves to a file that exists on disk. Prevents doc bit-rot: if a code
# file is renamed or moved, any doc that linked to it fails CI.
#
# Also asserts every markdown file that's part of the m0/ tree begins with
# the `<!-- Last verified: YYYY-MM-DD by Claude Code -->` header.
set -euo pipefail

cd "$(dirname "$0")/.."

DOCS_ROOT="docs/specs/v0/implementation"

if [[ ! -d "$DOCS_ROOT" ]]; then
  echo "check-doc-links: no implementation docs tree at $DOCS_ROOT yet (pre-M0 — expected)."
  exit 0
fi

missing_links=0
missing_headers=0

while IFS= read -r -d '' md_file; do
  # 1. Verify the "Last verified" header is on line 1.
  first_line=$(head -1 "$md_file")
  if [[ ! "$first_line" =~ Last\ verified:\ [0-9]{4}-[0-9]{2}-[0-9]{2}\ by ]]; then
    echo "check-doc-links: $md_file missing 'Last verified: YYYY-MM-DD by …' header on line 1"
    missing_headers=$((missing_headers + 1))
  fi

  # 2. Extract every relative link (one per line). Skip external (http/https/mailto)
  #    and plain anchors (#section).
  doc_dir=$(dirname "$md_file")
  while IFS= read -r link; do
    # Strip anchor if present.
    path_only="${link%%#*}"

    # Empty path (just an anchor like `#section`) — skip.
    [[ -z "$path_only" ]] && continue

    # Resolve relative to the markdown file's directory.
    target="$doc_dir/$path_only"
    # Normalise trailing slashes (directory links).
    target="${target%/}"

    if [[ ! -e "$target" ]]; then
      echo "check-doc-links: $md_file references missing path: $path_only (resolved → $target)"
      missing_links=$((missing_links + 1))
    fi
  done < <(
    # Grep every ](…) link body, print the inside. Exclude external links.
    grep -oE '\]\([^)]+\)' "$md_file" 2>/dev/null \
      | sed 's/^](//; s/)$//' \
      | grep -Ev '^(https?://|mailto:)' \
      | grep -Ev '^#'
  )

done < <(find "$DOCS_ROOT" -type f -name '*.md' -print0)

if [[ $missing_links -gt 0 || $missing_headers -gt 0 ]]; then
  echo "check-doc-links: FAIL — $missing_links broken links, $missing_headers missing 'Last verified' headers."
  exit 1
fi

echo "check-doc-links: all markdown under $DOCS_ROOT has valid relative links + verification headers. OK."
