<!-- Status: CONCEPTUAL -->

# Human Agent

> Extracted from brainstorm.md Section 3.5.
> See also: [agent.md](agent.md), [coordination.md](coordination.md)

---

## Overview

A Human Agent is an Agent **without** Model, Context, or System Prompt but **with** channels. This stays true to the grounding principle that **everything is an agent**. Human agents don't need models/context/system prompts but may have sessions, channels, and memory.

## Comparison: Human Agent vs LLM Agent

| Property | Human Agent | LLM Agent |
|----------|------------|-----------|
| Soul (Profile) | Name, role, preferences | Full AgentProfile + ModelConfig + SystemPrompt |
| Model | None | ModelConfig |
| System Prompt | None | SystemPrompt |
| Sessions | Yes | Yes |
| Memory | Yes (Short/Medium/Long) | Yes (Short/Medium/Long) |
| Channels | Yes (Slack, email, web UI) | None (API-native) |
| Permissions | Yes | Yes |
| Can rate agents | Yes | Yes (using evaluation framework) |
| Can bid | No (sponsors/assigns instead) | Yes (Contract mode) |
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
