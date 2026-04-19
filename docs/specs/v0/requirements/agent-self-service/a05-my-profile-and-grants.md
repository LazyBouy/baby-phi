<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — agent self-service surface -->

# a05 — My Profile and Grants

## 2. Page Purpose + Primary Actor

The Agent's own read view over their Identity node and the Grants they currently hold. Read-only for most fields; the Agent may update `self_description` (self-authored bio) and nothing else on this page. Grants are displayed with their shape + provenance chain so the Agent can understand what they can do and why.

**Primary actor:** any Agent (for LLM Agents, this is the profile-introspection tool surface).

## 3. Available When

- Any Agent with a materialised Identity node (LLM Agents only per [concepts/agent.md § Identity](../../concepts/agent.md#identity-emergent-event-driven)). Human Agents have no system-computed Identity; they see the Grants panel + basic profile only.

## 4. UI Sketch

```
┌───────────────────────────────────────────────────────────────────┐
│ My Profile — agent:coder-acme-3                                   │
├───────────────────────────────────────────────────────────────────┤
│ Profile                                                            │
│   kind         Contract                                            │
│   model        claude-sonnet-default                               │
│   parallelize  2 of 2 used                                         │
│                                                                     │
│ Identity (computed)                                                │
│   Self-description                                                 │
│   ┌─────────────────────────────────────────────────────────────┐ │
│   │ A careful full-stack contractor; strong on TypeScript and   │ │
│   │ Rust; learning distributed tracing.   [Edit]                │ │
│   └─────────────────────────────────────────────────────────────┘ │
│                                                                     │
│   Lived experience                                                 │
│   - sessions_completed: 47     sessions_successful: 44             │
│   - ratings_window: [0.85, 0.90, 0.82, ... n=20]  avg 0.88         │
│   - skills: [typescript, rust, api-design]                         │
│                                                                     │
│   Witnessed experience                                             │
│   - memories_extracted: 0   (not a supervisor)                     │
│                                                                     │
│ Grants I hold (12)                                                 │
│   [read, list, recall, delete] on my inbox_object                  │
│   [read, list, send]           on my outbox_object                 │
│   [read]                       on memory_object agent:self         │
│   [execute]                    on process_exec_object sandboxed    │
│   [read, list]                 on filesystem_object /workspace/**  │
│   ...                                                               │
│   [View full grant detail]                                         │
└───────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-AGENT-a05-R1:** The surface SHALL display the Agent's AgentProfile basics: id, kind, model, parallelize current/cap.
- **R-AGENT-a05-R2:** For LLM Agents, the surface SHALL display the Identity node's `self_description`, `lived` struct fields, `witnessed` struct fields (if non-empty). Embedding vector is NOT displayed — only its similarity-query utility is; the dimensionality/platform-model is a platform-level concern.
- **R-AGENT-a05-R3:** The surface SHALL list Grants the Agent currently holds, via `HOLDS_GRANT` edges. Each grant shows: action set, resource type + selector, constraints, provenance (link to the Auth Request that produced it), and delegable flag.
- **R-AGENT-a05-R4:** A "View full grant detail" expands each grant into its full 5-tuple + authority chain (Grant → Auth Request → approver → owner → ... → bootstrap).

## 6. Write Requirements

- **R-AGENT-a05-W1:** The Agent SHALL be able to update their own `self_description` (≤500 tokens). Update triggers a re-embed of the Identity vector per [admin/13-system-agents-config.md](../admin/13-system-agents-config.md)'s platform-level embedding model configuration.
- **R-AGENT-a05-W2:** The Agent SHALL NOT be able to modify `lived`, `witnessed`, `embedding`, their kind, their model config, their parallelize, or their Grants from this surface. Those changes come from their administrator (Agent profile edits), their work (lived auto-updates), their supervision role (witnessed auto-updates), or Auth Request flows (grants).

## 7. Permission / Visibility Rules

- **Self read** — an Agent always has `[read]` on their own `identity_principal` for their own Identity node. This is a Default Grant issued at Agent creation.
- **Grants list** — the Agent sees their own `HOLDS_GRANT` edges. No other Agent's grants are visible from this surface.
- **Self_description update (W1)** — gated by the same Default Grant; `[modify]` on own profile's self_description field only.

## 8. Event & Notification Requirements

- **R-AGENT-a05-N1:** Updating `self_description` (W1) emits audit event `IdentitySelfDescriptionUpdated { agent_id, description_length, updated_at }` (not alerted; routine bookkeeping).
- **R-AGENT-a05-N2:** When a new Grant is issued to the Agent (via any Auth Request flow elsewhere), the Grants list updates live; the Agent is notified via inbox.
- **R-AGENT-a05-N3:** When a Grant held by the Agent is revoked (upstream), the Grants list updates and the Agent is notified.

## 9. Backend Actions Triggered

- W1 (self_description edit):
  - `self_description` field updated on Identity node.
  - Embedding re-computed using the platform's configured embedding model.
  - `IdentitySelfDescriptionUpdated` audit event.
  - Per [concepts/agent.md § Materialization](../../concepts/agent.md#materialization), the Identity node's reactive triggers already cover this.

## 10. API Contract Sketch

```
GET  /api/v0/agents/{agent_id}/profile
     → 200: { profile, identity: {self_description, lived, witnessed, embedding_dim}, grants: [...] }
     → 403: Not self (for grants list — Identity metadata may be readable by project leads per Template A)

PATCH /api/v0/agents/{agent_id}/identity/self-description
     Body: { self_description }
     → 200: { updated_at, audit_event_id }
     → 400: length exceeds cap

GET  /api/v0/agents/{agent_id}/grants/{grant_id}
     → 200: { grant, authority_chain: [...] }    (structural traverse via DESCENDS_FROM edges)
```

## 11. Acceptance Scenarios

**Scenario 1 — contract agent updates self_description after promotion.**
*Given* `coder-acme-3` has just been promoted from Intern to Contract (not a function of this page; done via the token-economy promotion flow), *When* they edit their `self_description` to reflect newly-built confidence ("…now working across the full stack with care…"), *Then* the text is stored, the embedding is re-computed, and `IdentitySelfDescriptionUpdated` is audit-logged.

**Scenario 2 — supervisor sees witnessed struct populated.**
*Given* a supervisor LLM Agent in [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md) has been running for a quarter and has extracted 180 memories from subordinates' sessions, *When* they view their profile, *Then* the Witnessed Experience panel shows `memories_extracted: 180` and `subordinates_observed: [list of N]`, reflecting the reactive per-extraction updates described in [concepts/agent.md § Concurrent sub-agent supervision](../../concepts/agent.md#witnessed-experience-is-mediated-by-extraction).

**Scenario 3 — grant detail traces to bootstrap.**
*Given* `coder-acme-3` holds a default `[read, list]` grant on their home filesystem, *When* they expand grant detail, *Then* the page shows the full authority chain: grant → Template A adoption Auth Request → Acme's org admin → ... → System Bootstrap Template, ending at `system:genesis`.

## 12. Cross-References

**Concept files:**
- [concepts/agent.md § Identity (Emergent, Event-Driven)](../../concepts/agent.md#identity-emergent-event-driven).
- [concepts/agent.md § Identity Node Content — Provisional Direction](../../concepts/agent.md#identity-node-content--provisional-direction).
- [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain) — the chain this page traces for grant detail.
- [concepts/permissions/04 § Grant as a Graph Node](../../concepts/permissions/04-manifest-and-resolution.md#grant-as-a-graph-node).

**Related admin pages:**
- [admin/09-agent-profile-editor.md](../admin/09-agent-profile-editor.md) — where Agent Profile fields are configured (by admin).
- [admin/13-system-agents-config.md](../admin/13-system-agents-config.md) — the embedding model used for Identity vectors is configured at platform / org level, not here.

**Related agent-self-service pages:**
- [a02-my-auth-requests.md](a02-my-auth-requests.md) — where Auth Requests that produce grants live.

**Org / project layouts exercised:**
- [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md) — where witnessed experience populates.
