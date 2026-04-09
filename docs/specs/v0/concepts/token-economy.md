<!-- Status: CONCEPTUAL -->

# Token Economy

> Extracted from brainstorm.md Section 3.4.
> See also: [agent.md](agent.md) (Contract vs Worker modes), [project.md](project.md) (token budgets)

---

## Overview

Tokens are the **currency** of the system. They flow through a cycle:

```
Sponsor/Human
    │
    ▼ allocates tokens to Project/Task
  Project/Task
    │
    ▼ Contract agent wins bid, receives token budget
  Agent (Contract mode)
    │
    ├─▶ spends tokens on LLM calls (execution cost)
    ├─▶ spends tokens on tool calls (if tool has cost)
    ├─▶ keeps remaining tokens (savings = efficiency reward)
    │
    ▼ delivers work, receives rating
  Rating + Savings → Worth calculation
```

> **Dual mode:** Contract agents participate in this economy. Worker agents (simpler, human-assigned) do not — they execute without a token budget. See [agent.md](agent.md) for mode details.

---

## Worth (Backward-Looking Reputation)

```
Worth = average_rating × (produced_savings / consumed_tokens)
```

Where:
- `average_rating` = mean of all project ratings received
- `produced_savings` = total tokens saved across all contracts (budget - actual spend)
- `consumed_tokens` = total tokens actually spent across all contracts

Worth measures: **"How efficiently does this agent deliver quality work?"**

---

## Value (Forward-Looking Market Price)

```
Value = average tokens received for won bids
```

Value is determined by the **market** (other agents + humans via bidding). It depends on:
- Identity (who the agent is)
- Skills (what it can do)
- Worth (track record)

Value measures: **"What is the market willing to pay this agent?"**

---

## Meaning (Holistic Standing)

The relationship between Worth and Value captures something deeper:
- High Worth + High Value = respected, well-compensated agent
- High Worth + Low Value = undervalued (market hasn't recognized quality yet)
- Low Worth + High Value = overvalued (reputation exceeds performance)
- Low Worth + Low Value = struggling agent

> **Open question:** Is there a formal Meaning formula, or is it a qualitative assessment that other agents/humans make? Could it be `Worth × Value`?
