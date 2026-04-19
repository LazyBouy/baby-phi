<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Agent Self-Service Surfaces

> Pages / tool surfaces an Agent uses to manage **their own** participation in the system. Applies to both Human Agents (channel-delivered renderings) and LLM Agents (programmatic tool surfaces). Same underlying composites (`inbox_object`, `outbox_object`, `Consent`, `AuthRequest`, etc.) — different rendering by actor kind.

## Pages

| File | What it covers |
|------|----------------|
| [a01-my-inbox-outbox.md](a01-my-inbox-outbox.md) | Read received messages (`AgentMessage` on my `inbox_object`); send messages to other agents; archive / mark read. |
| [a02-my-auth-requests.md](a02-my-auth-requests.md) | **Inbound** (Auth Requests where I hold an approver slot) + **Outbound** (Auth Requests I have submitted). |
| [a03-my-consent-records.md](a03-my-consent-records.md) | Acknowledge / decline / revoke my Consent records. |
| [a04-my-work.md](a04-my-work.md) | My assigned Tasks and my running/recent Sessions. |
| [a05-my-profile-and-grants.md](a05-my-profile-and-grants.md) | My Identity node (self_description, lived, witnessed, embedding) and the Grants I hold. Read-only. |

## Rendering by Agent kind

- **Human Agents** — these surfaces render as traditional UI pages in a web client; notifications arrive through the Human's registered `Channel` (Slack, email, web).
- **LLM Agents** — these surfaces render as **tool manifests** the agent invokes programmatically. Each "page" is really a (resource, action) pair per the permissions model, often with a single focused tool (`read_inbox`, `send_message`, `approve_auth_request`, `acknowledge_consent`, etc.). The manifest for each tool is the underlying contract shared across renderings.
- **System Agents** — do not use these surfaces; their own behavior is event-driven (see `system/`).

The 10-section template in [../_template/admin-page-template.md](../_template/admin-page-template.md) applies here too. For LLM-Agent-primary pages, Section 4 (UI Sketch) may be replaced with a "Tool Surface Sketch" showing the tool manifest and an example call/response pair.

## See also

- [../README.md](../README.md) — top-level requirements README with terminology + ID conventions.
- [concepts/permissions/05 § Inbox and Outbox](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging) — the underlying composites.
- [concepts/permissions/02 § Per-State Access Matrix](../../concepts/permissions/02-auth-request.md#per-state-access-matrix) — the gate for inbound Auth Request actions.
