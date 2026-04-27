<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-25 — InboxObject / OutboxObject don't carry embedded AgentMessage value objects

## Identification
- **ID**: D-new-25
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: B — underspecified shape choice
- **Severity**: MEDIUM
- **Tags**: `composite-storage`, `message-routing`

## Concept alignment
- **Concept doc(s)**: [`concepts/ontology.md`](../../../concepts/ontology.md) §"InboxObject/OutboxObject"; [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Composite Classes — Inbox & Outbox"
- **Concept claim**: InboxObject + OutboxObject carry embedded `AgentMessage` value objects (receive/send queues).
- **Contradiction**: Shipped structs at [`nodes.rs:772-784`](../../../../../../modules/crates/domain/src/model/nodes.rs#L772-L784) have only `id`, `agent_id`, `created_at`. No message-list field; no AgentMessage embedding.
- **Classification**: `silent-in-code`

## Remediation
- **Approach**: Extend structs with `pub messages: Vec<AgentMessage>` (or phi-core type wrap). Migration adds FLEXIBLE column. Wire message-routing writers. ~2 days (likely deferred to M6/M7 when inter-agent messaging is in scope).
- **Impl chunk**: M6-DEFERRED-02
- **Risk**: MEDIUM — inter-agent messaging unimplementable without embedding.

## Lifecycle
- 2026-04-24 — `discovered`
