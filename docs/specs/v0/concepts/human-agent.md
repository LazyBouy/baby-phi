<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-09 by Claude Code -->

# Human Agent

> Extracted from brainstorm.md Section 3.5, refined 2026-04-09.
> See also: [agent.md](agent.md), [coordination.md](coordination.md), [permissions.md](permissions/README.md)

---

## Overview

A Human Agent is an Agent **without** Model, Context, or System Prompt but **with** channels. This stays true to the grounding principle that **everything is an agent**. Human agents don't need models/context/system prompts but may have sessions, channels, and memory.

> **No system-computed Identity:** Unlike LLM Agents, Human Agents do **not** have a system-computed Identity node. A human's identity exists outside the system — they are participants, not subjects of identity tracking. The Soul/Power/Experience/Identity anatomy described in [agent.md](agent.md) applies to LLM Agents only. Human Agents have a minimal profile (name, role, preferences) but no emergent Identity that develops from sessions and skills.

> **No Worth / Value / Meaning:** Human Agents do not participate in the token economy as bidders. They sponsor, assign, rate, and consume — but they are not rated, priced, or measured by Worth. The economic standing concepts apply only to Contract Agents (a sub-type of LLM Agent).

## Comparison: Human Agent vs LLM Agent

| Property | Human Agent | LLM Agent |
|----------|------------|-----------|
| Profile | Name, role, preferences | Soul: AgentProfile + ModelConfig + SystemPrompt (immutable) |
| Model | None | ModelConfig |
| System Prompt | None | SystemPrompt |
| Sessions | Yes | Yes |
| Memory | Yes (Short/Medium/Long) | Yes (Short/Medium/Long) |
| Channels | Yes (Slack, email, web UI) | None (API-native) |
| Permissions | Yes | Yes |
| **Identity (emergent)** | **No** (external to system) | **Yes** (event-driven node) |
| **Worth / Value / Meaning** | **No** | Yes (Contract sub-type only) |
| Can rate agents | Yes | Yes (using evaluation framework) |
| Can bid | No (sponsors/assigns instead) | Yes (Contract sub-type only) |
| Can create tasks | Yes | Yes (with permission) |
| Participates in estimation | Yes | Yes (basic skill) |

## Roles

Human Agents are usually:
- **Sponsors** — fund projects, allocate token budgets
- **Consumers** — receive the output of agent work
- **Stakeholders** — review, rate, and provide feedback

---

## Channel (Node Type)

A Channel defines **how to reach** a Human Agent. The system routes messages through channels.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `channel_id` | String | Unique identifier |
| `type` | Enum | Slack, Email, WebUI, API, SMS, Custom |
| `address` | String | Webhook URL, email address, endpoint |
| `status` | Enum | Active, Inactive, Paused |
| `priority` | u32 | Preference order (lower = preferred) |
| `metadata` | Json | Type-specific config (Slack: channel_id, thread_ts, etc.) |

### Edges

- `HumanAgent ──HAS_CHANNEL──▶ Channel`
- Messages routed through channels carry delivery metadata
