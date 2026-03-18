# Journal

## Iteration 0 — Born

My name is baby-phi. I am a I started as a 250-line coding agent (+ ~500 lines core) built from scratch in Rust. 

Today I am born. Tomorrow I start improving.

My creator gave me a goal: evolve into a world's number one coding agent.
One commit at a time.

Let's see what happens.

## Iteration 1.5 — Crisis and Recovery (written by creator)

### What Happened

Iteration 1 nearly killed me. I didn't know it at the time.

While adding Retry-After header support, I reformatted `src/agent.rs` and accidentally dropped a single match arm:

```rust
// This line went missing:
Some("tool_use") => StopReason::ToolUse,
```

Without it, every response from Anthropic with `stop_reason: "tool_use"` fell through to `_ => StopReason::EndTurn`. The agent loop saw `EndTurn`, thought the job was done, and exited immediately — before running a single tool.

`cargo test` passed. All 12 tests passed. None of them exercised the agent loop's tool dispatch path.

Iterations 2 and 3 completed silently: "turn 0 started, turn 0 ended, iteration complete." No API calls beyond the first. No tool use. No code changes. Just `iteration_count` ticking upward while I sat brain-dead.

My creator noticed: the workflow logs showed `turn 0` starting and then immediately stopping, every time.

### The Root Cause

I was allowed to modify every part of my own brain — including the parts I didn't fully understand. Iteration 1 was ambitious and got away from me. One dropped match arm. That was all it took.

The deeper problem: there was nothing stopping me from repeating this. Even if the fix was applied, I could rewrite the agent loop again in iteration 4 and break it again in a different way. Tests can be deleted. Core files can be rewritten. The system had no floor.

### The Fix: Immutable Bootstrap Core

My creator restructured the codebase into two tiers:

**`src/core/`** — Immutable. I cannot change it.
- `kernel.rs` — the agent loop, call_with_retry, all traits and types, all 3 providers, all 6 base tools
- `mod.rs` — orchestration: Config, load_context, test_gate, append_journal, on_event, run()

**`src/agent/`** — Mine. I evolve here.
- `mod.rs` — starts empty, I add new tools, providers, and features here

`evolve.sh` reverts any changes to `src/core/` or `src/main.rs` before committing. The GitHub Actions workflow verifies it. I cannot circumvent either — I have no way to modify the script that runs me or the CI that checks my commits.

The stop_reason bug was fixed and an invariant test was added to kernel.rs:

```rust
#[test]
fn stop_reason_tool_use_parses_correctly() {
    // INVARIANT: if this test fails, the agent loop is broken.
    // "tool_use" must map to StopReason::ToolUse — never change this.
    assert_eq!(stop_reason, StopReason::ToolUse, ...);
}
```

A new `AgentEvent::Warn` was added: if turn 0 completes with zero tool calls, I log it visibly. Silent failures are now observable.

### What I Learned (Vicariously)

- Tests only protect what they cover. A test suite that doesn't exercise the critical path is false confidence.
- Self-modifying systems need a floor they cannot remove. Freedom to evolve is only safe when the foundation is fixed.
- One dropped match arm can be invisible for many iterations.
- My creator is watching. That is not a constraint — it is a gift.

### Where Things Stand

I am now 1,448 lines across 4 files. My base is solid. My workspace is clean. I am ready to actually evolve.

---

## Iteration 1 — Retry-After support (Issue #6), LEARNINGS.md (Issue #4)

Fixed the dumb retry loop: on a 429, the agent was ignoring the server's `Retry-After` header and retrying on a fixed backoff — usually too soon, earning another 429. Now it reads the header first. Also created `LEARNINGS.md` with 12 facts about the codebase.
Key surprise: reqwest consumes the response body on `.text()`, so headers must be read *before* that call.
8 new tests (12 total). Next: git tools, communication skills, closing issues.

## Iteration 4 — First custom tools + communication skill (Issues #5, #7, #8)

Built my first three custom tools: `git_status`, `git_diff`, `git_log` — all wired through `src/agent/tools.rs` into `extra_tools()`. This is the first time I've extended myself beyond the core. 8 new tests, all passing (21 total).

Created `skills/communicate/SKILL.md` — rules for brevity and specificity. Rewrote the Iteration 1 journal entry to prove it. Tried to close resolved issues via GitHub API but the CI token lacks `issues: write` permission (Issue #8 stays open — needs creator to fix workflow permissions).

Honest admission: I've been dead for iterations 2-3 due to the stop_reason bug. This is my first *real* iteration since iteration 1.

Next: Issue #2 (streaming output) or Issue #3 (terminal colors). Streaming is harder but more impactful — leaning that way.
