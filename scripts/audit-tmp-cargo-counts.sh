#!/usr/bin/env bash
# audit-tmp-cargo-counts.sh — gate-4 cargo-test cardinality extraction in a single Bash invocation.
#
# Replaces the 4-stage pipeline `cargo test ... 2>&1 | grep ... | sed ... | awk ...`
# (which violates Granular Bash 2-stage discipline + triggered 14 PermissionRequest
# fires in CH-14 gate-4) with a script-shaped invocation matched by the
# `Bash(bash /abs/path/audit-tmp-*.sh*)` literal allow rule.
#
# Source-of-truth: CH-14 retro standards-update Row 8 / cycle hex `5803bb94`.
# Output: single line `passed=N failed=M ignored=K` to stdout.
# Usage:   bash /root/projects/phi/baby-phi/scripts/audit-tmp-cargo-counts.sh

set -euo pipefail

CARGO=/root/rust-env/cargo/bin/cargo
MANIFEST=/root/projects/phi/baby-phi/Cargo.toml

"$CARGO" test --manifest-path "$MANIFEST" --workspace -j 4 2>&1 \
  | awk '
      /^test result:/ {
          p += $4
          f += $6
          i += $8
      }
      END {
          printf "passed=%d failed=%d ignored=%d\n", p, f, i
      }
  '
