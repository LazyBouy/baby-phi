#!/usr/bin/env bash
# phi-core leverage lint (D16 / C19 in the M2 plan; also baked into
# phi/CLAUDE.md).
#
# phi is a consumer of phi-core. Every surface that overlaps with
# something phi-core already ships must import or wrap the phi-core
# type — never re-implement. This lint fails CI if any `.rs` under
# `modules/crates/` re-declares a known phi-core type name.
#
# Orthogonal surfaces (same name, different layer) are allow-listed
# below so the lint stays actionable — these are deliberate, not
# duplicates:
#
#   - domain::model::AgentProfile wraps phi_core::AgentProfile (M1/P0
#     established the wrap; the phi struct carries governance
#     fields on top of a `blueprint: phi_core::AgentProfile` field).
#     The phi struct is the authoritative platform-governance node;
#     the phi-core struct is the embedded execution blueprint. Skipping.
#
# Everything else stays a hard denylist — the build must break before
# any PR lands that quietly clones a phi-core surface.
#
# Advisory during P1; flips to hard-gate at the P3 close re-audit.
set -euo pipefail

cd "$(dirname "$0")/.."

SCAN_ROOT="modules/crates"

# Types phi-core owns; no phi crate may redeclare them. Hard denial
# — any match anywhere under $SCAN_ROOT fails the lint.
FORBIDDEN=(
    "ExecutionLimits"
    "ModelConfig"
    "McpClient"
    "RetryConfig"
    "ContextConfig"
    "AgentEvent"
    "LoopRecord"
    "SessionRecorder"
    "CompactionStrategy"
    "StreamProvider"
    "StreamConfig"
    "ProviderRegistry"
    "ApiProtocol"
    "CacheConfig"
    "ThinkingLevel"
    "McpToolInfo"
    "McpToolAdapter"
    "Turn"
    "AgentTool"
)

# Types phi-core owns, but baby-phi wraps at the governance-node
# tier. The wrap is permitted ONLY in
# `modules/crates/domain/src/model/nodes.rs`; a redeclaration
# anywhere else fails the lint. Added at M5/P0 for the Session
# wrap (ADR-0029) — the wrap literally carries the same name
# (`struct Session { inner: phi_core::session::model::Session, ... }`)
# so we cannot just add it to FORBIDDEN.
WRAP_FORBIDDEN=(
    "Session"
)

WRAP_ALLOWED_FILE="modules/crates/domain/src/model/nodes.rs"

hits=0

for name in "${FORBIDDEN[@]}"; do
    # Match `pub struct X` / `struct X` / `pub enum X` / `enum X`
    # at line start (allow indentation) so attribute lines like
    # `#[serde(rename = "X")]` don't trip the lint.
    matches=$(grep -rn --include="*.rs" -E "^[[:space:]]*(pub[[:space:]]+)?(struct|enum|trait)[[:space:]]+${name}\b" "$SCAN_ROOT" || true)
    if [[ -n "$matches" ]]; then
        echo "check-phi-core-reuse: FORBIDDEN redeclaration of phi-core type '$name':"
        echo "$matches" | sed 's/^/  /'
        hits=$((hits + 1))
    fi
done

for name in "${WRAP_FORBIDDEN[@]}"; do
    matches=$(grep -rn --include="*.rs" -E "^[[:space:]]*(pub[[:space:]]+)?(struct|enum|trait)[[:space:]]+${name}\b" "$SCAN_ROOT" || true)
    if [[ -n "$matches" ]]; then
        # Filter out the wrap-allowed file.
        filtered=$(echo "$matches" | grep -v "^${WRAP_ALLOWED_FILE}:" || true)
        if [[ -n "$filtered" ]]; then
            echo "check-phi-core-reuse: FORBIDDEN redeclaration of phi-core wrap type '$name' outside $WRAP_ALLOWED_FILE:"
            echo "$filtered" | sed 's/^/  /'
            hits=$((hits + 1))
        fi
    fi
done

# Allow-listed exceptions (checked by name explicitly so the allowance
# is legible).  Removing an exception here MUST come with a CLAUDE.md
# update documenting the new invariant.
#
# - AgentProfile: phi's struct is a platform-governance node that
#   wraps phi_core::AgentProfile as a `blueprint` field (per
#   concepts/phi-core-mapping.md + the M1/P0 reshape). It is the
#   phi-authoritative surface; the wrap field is phi-core's.
ALLOWLIST_EXPLAINED=(
    "AgentProfile: wraps phi_core::AgentProfile as a blueprint field — see m1/architecture/graph-model.md §AgentProfile wraps phi-core"
)
if [[ ${#ALLOWLIST_EXPLAINED[@]} -gt 0 ]]; then
    :  # explanatory only; no enforcement here
fi

if [[ $hits -gt 0 ]]; then
    echo ""
    echo "check-phi-core-reuse: FAIL — $hits forbidden redeclaration(s)."
    echo "See phi/CLAUDE.md §phi-core Leverage for the reuse mandate."
    exit 1
fi

echo "check-phi-core-reuse: no forbidden phi-core redeclarations under $SCAN_ROOT. OK."
