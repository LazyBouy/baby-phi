<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-24 — Channel node schema incomplete (missing address/status/priority/metadata fields + WebUI/API/SMS/Custom kinds)

## Identification
- **ID**: D-new-24
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: LOW
- **Tags**: `schema-enrichment`, `channel-model`

## Concept alignment
- **Concept doc(s)**: [`concepts/human-agent.md`](../../../concepts/human-agent.md) §"Channel (Node Type)"; [`concepts/ontology.md`](../../../concepts/ontology.md) §"Channel"
- **Concept claim**: Channel carries channel_id, type, address, status, priority, metadata.
- **Contradiction**: Shipped `Channel` at [`nodes.rs:754-768`](../../../../../../modules/crates/domain/src/model/nodes.rs#L754-L768) has id, agent_id, kind (with only Slack/Email/Web variants), handle, created_at. Missing: address (distinct from handle), status enum, priority, metadata. Kind enum missing WebUI, API, SMS, Custom.
- **Classification**: `partially-honored`

## Remediation
- **Approach**: Enrich Channel struct per concept. Migration 0006+ adds columns. Likely deferred to when multi-channel routing matters. ~1 day.
- **Impl chunk**: M7-DEFERRED-01
- **Risk**: LOW at M5.

## Lifecycle
- 2026-04-24 — `discovered`
