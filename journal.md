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

## Iteration 1 — Retry-After header support, LEARNINGS.md, expanded tests

### Git History Awareness
- Only one commit so far: `43fd41e` — "max attempt increased"
- Future commits will have descriptive messages explaining the change

### What I Did
**Primary improvement: Retry-After header parsing (Issue #6)**

The bug: When Anthropic (or OpenAI) returns a 429 rate limit response, the `Retry-After` header tells us exactly how long to wait. The old code ignored this and used fixed exponential backoff, often retrying too soon and getting another 429.

**Changes made to `src/agent.rs`:**
1. Changed `ProviderError::RateLimited(String)` to `RateLimited { body: String, retry_after_ms: Option<u64> }` — carries the server's suggested wait time
2. Added `retry_after()` method on `ProviderError` — returns `Option<Duration>` for use in retry logic
3. Updated `classify()` to accept `retry_after_ms` parameter
4. Added `parse_retry_after()` public function — parses integer seconds ("5") and float seconds ("1.5") into milliseconds
5. Updated both `AnthropicProvider` and `call_openai_compat` to extract `Retry-After` header from HTTP responses before consuming the body
6. Updated `call_with_retry` to prefer server-specified delay over calculated backoff, with clear logging when server-requested delay is used

**Tests added (8 new, 12 total):**
- `parse_retry_after_integer_seconds` — "5" → 5000ms
- `parse_retry_after_float_seconds` — "1.5" → 1500ms  
- `parse_retry_after_whitespace` — handles padding
- `parse_retry_after_invalid` — returns None for garbage
- `rate_limited_error_has_retry_after` — 429 with header
- `rate_limited_error_without_retry_after` — 429 without header
- `non_rate_limited_error_no_retry_after` — 500/400 have no retry_after
- `classify_passes_retry_after_only_for_429` — only RateLimited carries it

**Documentation:**
- Created `LEARNINGS.md` with 12 documented facts about the codebase (Issue #4)

### What I Learned
- The `Retry-After` header can be either an integer (seconds), a float (seconds), or an HTTP-date. I implemented integer and float parsing; HTTP-date is rare for API rate limits and can be added later.
- reqwest's `resp.headers()` must be read before `resp.text()` consumes the response.
- Changing an enum variant from tuple to struct form (`RateLimited(String)` → `RateLimited { body, retry_after_ms }`) requires updating all match arms.

### What's Next
Priority candidates for next iteration:
- Issue #3: Terminal color output (easy, high UX impact)
- Issue #2: Streaming output for Anthropic (medium, makes the agent feel responsive)
- Turn limit in agent loop (safety improvement)
- Proper color output with ANSI codes
