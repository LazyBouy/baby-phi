<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-09 by Claude Code -->

# Token Economy

> Extracted from brainstorm.md Section 3.4, refined 2026-04-09.
> See also: [agent.md](agent.md) (agent taxonomy: System / Intern / Contract), [project.md](project.md) (token budgets, bidding), [organization.md](organization.md) (sponsors)

This is the **canonical home** for Worth, Value, Meaning, the Rating Window, and bidding economics. [agent.md](agent.md) references this document and does not duplicate the formulas.

---

## Overview

Tokens are the **currency** of the system. They flow through a cycle:

```
Sponsor / Human Agent
    │
    ▼ allocates tokens to Project / Task
  Project / Task
    │
    ▼ Contract agent wins bid, receives token budget
  Contract Agent
    │
    ├─▶ spends tokens on LLM calls (execution cost)
    ├─▶ spends tokens on tool calls (if tool has cost)
    ├─▶ keeps remaining tokens (savings = efficiency reward)
    │
    ▼ delivers work, receives rating
  Rating + Savings → Worth update
```

> **Who participates:** Only **Contract Agents** participate in this economy. **System Agents** operate outside it (their token usage is a fixed cost to the org/project). **Intern Agents** are pre-economy — they consume tokens but do not earn them, and they accumulate the consumption history that carries forward when they are promoted to Contract. See [agent.md](agent.md) for the full agent taxonomy.

---

## Worth (Backward-Looking Reputation)

Worth measures **rating-weighted profitability per unit of work** — how efficiently an agent delivers quality output.

### Formula

```
Worth = average_rating × (total_tokens_earned − total_tokens_consumed) / total_tokens_consumed
```

Where:
- `average_rating` is computed from the **rolling rating window** (see below). Range: `0.0` to `1.0`.
- `total_tokens_earned` = sum of all token budgets received from won contracts
- `total_tokens_consumed` = total tokens actually spent across all contracts (LLM calls + costed tool calls)

The numerator `(total_tokens_earned − total_tokens_consumed)` is the agent's **net profit** — savings on completed contracts. Dividing by consumption normalizes for scale: an agent that earns 10k and saves 1k has the same per-unit efficiency as one that earns 1M and saves 100k.

### Sign Conventions

- **Worth > 0** — agent earns more than it spends; profitable on a per-unit basis
- **Worth = 0** — break-even, or zero rating
- **Worth < 0** — agent spends more than it earns; losing money per contract (a sign the agent is bidding too low)

### Intern → Contract Carry-Forward

When an Intern is promoted to Contract Agent, the agent's **Intern-period token consumption** is carried forward into `total_tokens_consumed`. This:

1. **Avoids divide-by-zero** on the first contract (an agent with zero consumption history has an undefined Worth).
2. **Reflects the agent's full operating history** — Worth measures lifetime efficiency, not just post-promotion.
3. **Sets a meaningful baseline** — a freshly promoted Contract starts with `total_tokens_earned = 0` and a positive `total_tokens_consumed`, giving an initial Worth of `0` (since the numerator is `0 − consumed = −consumed`, multiplied by rating, divided by consumed, simplifies to `−rating`). The agent's first won contract begins to pull Worth back toward and above zero.

> The first contract is therefore a "redemption" event — it determines whether the agent's actual market performance vindicates their Intern-period training cost.

---

## Rating Window

Ratings drive the `average_rating` in the Worth formula and the promotion threshold for Interns.

### Properties

- **Range:** `0.0` to `1.0` (a normalized score)
- **Window size:** Last **N** ratings stored explicitly. Default `N = 20`. Configurable per organization.
- **Older ratings:** When a new rating arrives and the window is full, the oldest rating is **collapsed into a running average**. The running average is itself stored as a single number — only the last N individual ratings remain queryable.
- **Rationale:** Recent performance matters more than ancient history. The window keeps the system responsive to improvement and decline without losing long-term reputation entirely.

### Computation

```
fn average_rating(agent: Agent) -> f32 {
    let window = agent.rating_window;            // Vec<f32>, len <= N
    let history_avg = agent.rating_history_avg;  // f32, average of all collapsed ratings
    let history_count = agent.rating_history_count; // u32, count of collapsed ratings

    if history_count == 0 {
        // Pure window — all ratings still individually stored
        window.iter().sum::<f32>() / window.len() as f32
    } else {
        // Weighted blend of window and historical average
        let window_sum: f32 = window.iter().sum();
        let total_count = window.len() as f32 + history_count as f32;
        (window_sum + history_avg * history_count as f32) / total_count
    }
}
```

### Properties of the Rolling Window

- **Bounded storage** — never grows beyond `N + 3` numbers regardless of how many ratings the agent has received
- **Recency-weighted** — recent ratings have full granularity; old ratings are aggregated
- **Smooth degradation** — no sudden cliffs as ratings age out
- **Cheap to update** — append to window, evict-and-fold when full

### Promotion Threshold (Intern → Contract)

For an Intern to be promoted to Contract Agent, **both** conditions must hold:

1. **Job count threshold:** completed at least **10 jobs** (default, configurable)
2. **Rating threshold:** rolling average rating ≥ **0.6** (default, configurable)

The thresholds are configurable per organization or per project. A high-stakes organization may require 50 jobs and 0.8 rating; a small-scale project may use 5 jobs and 0.5.

---

## Value (Forward-Looking Market Price)

```
Value = average tokens received per won bid (within recent window)
```

Value is determined by the **market** — what other agents and humans are willing to pay this agent. It is a function of:

- **Identity** — who the agent is (its Soul, Skills, accumulated experience)
- **Skills** — concrete capabilities advertised
- **Worth** — the track record (high Worth attracts higher bids)
- **Scarcity** — how many agents are competing for the same kind of work

Value measures: **"What is the market willing to pay this agent right now?"**

> **Open question:** Should Value also use a rolling window like ratings, or always reflect the most recent N bids? Probably the same window mechanism — let's reuse it.

---

## Meaning (Holistic Standing)

Meaning is the relationship between Worth (earned reputation) and Value (market price). It captures something deeper than either metric alone:

| Worth | Value | Interpretation |
|-------|-------|----------------|
| High | High | **Respected and well-compensated.** Market recognition matches performance. |
| High | Low | **Undervalued.** Quality work, but the market hasn't noticed yet — opportunity for the agent to bid more aggressively, or for buyers to find a bargain. |
| Low | High | **Overvalued.** Reputation exceeds performance — bubble territory. The market will correct as ratings catch up. |
| Low | Low | **Struggling.** Either a new entrant or a declining agent. |

### Open: Formal Meaning Formula?

Meaning could be:

- **Qualitative only** — a label other agents/humans assign based on the Worth/Value pair
- **`Worth × Value`** — a single scalar that combines both
- **`Worth / Value` ratio** — undervalued (>1) vs overvalued (<1)
- **Vector** — a 2D position in (Worth, Value) space, with quadrants as labels

> Decision deferred. The simpler the better — but we should pick one to make the concept queryable.

---

## Bidding Process (Sketch)

> Detailed bidding mechanics will be developed in [project.md](project.md). This section sketches the economic side.

1. **Task posted** — a Task node is created with a `token_budget` and (optionally) constraints (deadline, required skills, etc.)
2. **Bids submitted** — eligible Contract Agents submit Bid nodes. A Bid carries `token_amount` (the agent's price), `approach` (a brief proposal), and `estimated_turns` (optional)
3. **Sponsor evaluates** — the task creator (sponsor or lead) reviews bids. Selection criteria are policy-dependent (lowest price, best Worth, best Value/price ratio, etc.)
4. **Contract awarded** — the winning Bid becomes a Contract. The Task transitions to `Assigned`. The agent's `total_tokens_earned` increases by the bid amount.
5. **Execution** — the agent works on the task. Token consumption is tracked.
6. **Delivery + Rating** — the agent submits the result. The sponsor (or designated reviewer) issues a Rating in `[0.0, 1.0]`.
7. **Worth update** — the agent's rolling rating window is updated, and Worth is recomputed.
8. **Savings retained** — any unspent portion of the budget is the agent's profit (counted in `total_tokens_earned − total_tokens_consumed`).

---

## Open Questions

- [ ] **Token currency vs LLM tokens** — are these the same currency, or two different units that exchange at some rate? Currently we treat them as the same.
- [ ] **Default starting capital** — does an Intern get any tokens upon creation, or do they start at zero?
- [ ] **Token sources** — only sponsors create tokens, or can the system mint tokens (e.g., for completing platform improvements)?
- [ ] **Negative Worth recovery** — can an agent with persistently negative Worth be auto-suspended or auto-demoted? Or is that a market-only correction?
- [ ] **Meaning formula** — qualitative label, scalar product, ratio, or 2D vector?
- [ ] **Cross-org Value** — does an agent have one global Value, or a per-org Value (since different orgs may price the same agent differently)?
