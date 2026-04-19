<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — agent self-service surface -->

# a01 — My Inbox & Outbox

## 2. Page Purpose + Primary Actor

An Agent's surface for reading received messages (`AgentMessage` on their `inbox_object`), sending messages to other agents (appends to own `outbox_object` and target's `inbox_object`), and marking messages read or archived.

**Primary actor:** any Agent (Human or LLM or System). Every Agent has exactly one inbox and one outbox, created at Agent creation in [admin/09-agent-profile-editor.md](../admin/09-agent-profile-editor.md).

## 3. Available When

- Any Agent exists with a non-archived inbox. (All agents, after Phase 5.)
- Prerequisite grants: Default Grants issued at Agent creation — `[read, list, recall, delete]` on own inbox; `[read, list, send]` on own outbox.

## 4. UI Sketch

**For Human Agents (web UI rendering):**

```
┌───────────────────────────────────────────────────────────────────┐
│ My Inbox — agent:founder                        [+ Send Message] │
├───────────────────────────────────────────────────────────────────┤
│ Filter:  ☐ Unread only    ☐ #urgent    thread: [ any ▾ ]        │
│                                                                    │
│ From              Subject                         Time    Status  │
│ ──────────────────────────────────────────────────────────────── │
│ intern-a          Task task-auth-cli complete     10:42   ●unread │
│ lead-acme-1       Bid proposal on market-task-7   09:30   ○read   │
│ memory-extract…   Memory extracted from s-9830    yest.   ○read   │
│                                                                    │
│ [Tabs:  📥 Inbox (3)  📤 Outbox (12)  ]                           │
└───────────────────────────────────────────────────────────────────┘
```

**For LLM Agents (tool surface rendering):**

```yaml
tool: read_inbox
manifest:
  resource: inbox_object
  actions: [read, list]
  constraints:
    tag_predicate: required
  kind: [inbox]
  target_kinds: [inbox]
  delegable: false
  approval: auto

# Example call
read_inbox(filter: { unread: true, thread: null }, limit: 20)
# → 200
# { messages: [ { message_id, sender, subject, body, sent_at, thread_id, priority, read: false } ], has_more: false }
```

```yaml
tool: send_message
manifest:
  resource: inbox_object        # writes to recipient's inbox_object
  actions: [write]              # write-only; no read access on recipient's inbox
  constraints:
    tag_predicate: required
  kind: [inbox]
  approval: auto
  # note: owner's own outbox is also written via default grant
```

## 5. Read Requirements

- **R-AGENT-a01-R1:** The surface SHALL list all `AgentMessage` value objects on the viewing Agent's `inbox_object`, ordered by `sent_at` descending.
- **R-AGENT-a01-R2:** The surface SHALL support filtering by `#urgent`, `#unread`/`#read`, and thread_id.
- **R-AGENT-a01-R3:** The outbox view SHALL list messages the Agent has sent, with recipient, subject, sent_at, and thread_id.
- **R-AGENT-a01-R4:** Full message body SHALL be visible to the owning agent; recipients outside the owning agent's scope SHALL NOT be able to read either inbox or outbox contents.

## 6. Write Requirements

- **R-AGENT-a01-W1:** The owning Agent SHALL be able to send a new message to another Agent, supplying recipient, subject, body, priority, optional thread_id. The action appends the message to the recipient's `inbox_object` and to the sender's `outbox_object`.
- **R-AGENT-a01-W2:** The owning Agent SHALL be able to mark a message as read (`#unread` → `#read` tag transition) and archive messages.
- **R-AGENT-a01-W3:** The owning Agent SHALL be able to delete messages from their own inbox. Deletion is soft — archived with `#deleted` tag; audit retention preserves it.
- **R-AGENT-a01-W4:** An Agent SHALL NOT be able to modify messages in their outbox (append-only); outbox entries are records of sent messages.
- **R-AGENT-a01-W5:** Validation: recipient must be an existing Agent reachable via the sender's org context (same-org by default; cross-org requires an existing cross-org grant).

## 7. Permission / Visibility Rules

- **Inbox read** — `[read, list]` on own `inbox_object`. Issued at Agent creation.
- **Send message** — `[send]` on own `outbox_object` + `[write]` on recipient's `inbox_object` (scoped write grant — see [permissions/05 § Inbox and Outbox](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging)).
- **Cross-org sending** — requires a grant authorising send to the target org; absent that, send is rejected.
- **Delete / archive** — `[delete]` on own inbox_object.
- **Outbox read** — `[read]` by the owning agent; read by auditors requires explicit grant.

## 8. Event & Notification Requirements

- **R-AGENT-a01-N1:** Sending (W1) emits audit event `AgentMessageSent { sender, recipient, subject_hash, thread_id, priority }`. Content itself is not in the audit log for privacy; it lives on the composite.
- **R-AGENT-a01-N2:** Human Agents receive new-inbox notifications via their `Channel` (Slack/email). LLM Agents receive no push — their next session may poll the inbox.
- **R-AGENT-a01-N3:** `#urgent` messages SHALL bypass batched-digest summarisation and arrive individually.

## 9. Backend Actions Triggered

Send (W1):
- Append `AgentMessage` to recipient's `inbox_object` (atomic with outbox append).
- Append record to sender's `outbox_object`.
- If the recipient is a Human Agent, deliver via their `Channel`.
- Audit event.

Mark-read / delete (W2/W3): tag updates on the composite; no cross-agent effect.

## 10. API Contract Sketch

```
GET  /api/v0/agents/{agent_id}/inbox?unread=true&thread_id=...
     → 200: { messages: [...], has_more: bool }
     → 403: not the owner

POST /api/v0/agents/{agent_id}/outbox/send
     Body: { recipient_agent_id, subject, body, priority, thread_id? }
     → 201: { message_id, delivered_at, audit_event_id }
     → 400: validation (recipient not reachable, empty body)
     → 403: missing cross-org send grant

PATCH /api/v0/agents/{agent_id}/inbox/{message_id}
     Body: { read?: true, archived?: true, deleted?: true }
     → 200
```

## 11. Acceptance Scenarios

**Scenario 1 — intern sends status to lead.**
*Given* [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md) has `lead-stream-a` and `intern-a1`, *When* `intern-a1` sends a message to `lead-stream-a` with subject "Blocker on task-auth" and priority Normal, *Then* the message appears in `lead-stream-a`'s inbox with `#unread` tag, in `intern-a1`'s outbox as a record, and a `AgentMessageSent` audit event is emitted.

**Scenario 2 — marketplace bid negotiation.**
*Given* [organizations/08-marketplace-gig.md](../../organizations/08-marketplace-gig.md) has posted a task and `contract-alpha` wants to bid, *When* `contract-alpha` sends a message to `platform-admin` with subject "Bid: task-NNN" and a body describing approach/tokens/timeline, *Then* the message enters the admin's inbox and becomes the bid record the poster compares.

**Scenario 3 — cross-org send blocked without grant.**
*Given* an Agent in `acme` wants to message an Agent in `beta-corp` but holds no cross-org send grant, *When* they attempt to send, *Then* the action is rejected with `CROSS_ORG_SEND_NOT_AUTHORIZED` and no message is delivered.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/05 § Inbox and Outbox (Agent Messaging)](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging) — the composite definitions, tag vocabulary, default grants.
- [concepts/agent.md § Grounding Principle](../../concepts/agent.md#grounding-principle) — why agents react independently to messages.
- [concepts/ontology.md](../../concepts/ontology.md) — `InboxObject`, `OutboxObject`, `AgentMessage` value object.

**Related admin pages:**
- [admin/09-agent-profile-editor.md](../admin/09-agent-profile-editor.md) — where inbox/outbox are auto-provisioned on Agent creation.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md), [organizations/08-marketplace-gig.md](../../organizations/08-marketplace-gig.md).
