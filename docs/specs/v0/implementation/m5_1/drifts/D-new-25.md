<!-- Last verified: 2026-05-10 by Claude Code (CH-19 P2: Status flipped `discovered` → `accepted-as-is`; ratified via CH-19 / ADR-0057 §D57.8; cycle hex `2c520ba7`.) -->
<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-25 — InboxObject / OutboxObject don't carry embedded AgentMessage value objects

## Identification
- **ID**: D-new-25
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `accepted-as-is`
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
- 2026-05-10 — `accepted-as-is` — ratified via CH-19 (cycle hex `2c520ba7`) / ADR-0057 §D57.8; the `accepted-as-is` is for the **deferral itself**, not the implementation surface; review trigger: M6-DEFERRED-02 (inter-agent messaging chunk); the drift's `Implementation chunk this belongs to: M6-DEFERRED-02` field stays as future-remediation marker (CH-19 ratifies the deferral; M6-DEFERRED-02 is when `messages: Vec<AgentMessage>` field-add lands); concept-doc `ontology.md` carries 1-line deferred-state footnote at the InboxObject/OutboxObject row referencing CH-19 + M6-DEFERRED-02.
