<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-30 — Org / Project templates as embedded config objects not materialized (tools_allowlist, resource_catalogue, execution_limits, grants schemas)

## Identification
- **ID**: D-new-30
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: C — convention/pattern decision (template-as-config vs template-as-AR)
- **Severity**: LOW
- **Tags**: `template-shape`, `config-objects`

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"Standard Organization Template" + §"Standard Project Template"
- **Concept claim**: Org template YAML specifies tools_allowlist, resource_catalogue, system_agents, authority_templates_enabled, consent_policy, execution_limits, session_object_grants, memory_object_grants, rating_window. Project template specifies filesystem/session/memory grants + execution_limits + consent_policy inheritance.
- **Contradiction**: Organization + Project structs don't carry embedded template config. Templates are adopted via AuthRequest nodes (M3/M5 pattern) — this works but differs from concept's "YAML config object" framing.
- **Classification**: `concept-aspirational` — shipped pattern (adoption ARs) is functionally equivalent to the YAML config; concept text is older/aspirational.

## Remediation
- **Approach**: Either refresh concept doc to reflect adoption-AR pattern OR ship config-object model at M7+. Most likely first choice (concept refresh). ~0.5 day.
- **Impl chunk**: CH-19
- **Risk**: LOW — shipped pattern works.

## Lifecycle
- 2026-04-24 — `discovered`
