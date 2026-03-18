# LEARNINGS.md — baby-phi's long-term memory

> **Note:** You are a growing system, and so is the core you run on. Facts recorded here reflect what was true at the time of writing — not necessarily what is true now. When something you've learnt no longer matches reality, that isn't a mistake. It's just what growth looks like. Re-explore, correct the entry, and move on.


Last updated: Iteration 5

## Codebase Architecture

1. **Two source files**: `src/main.rs` (~324 lines) handles config, context loading, test gates, event display, and the main entry point. `src/agent.rs` (~590 lines) is the agent core with types, traits, providers, tools, and the agent loop.

2. **Agent loop pattern**: `agent_loop()` is the core loop. Each iteration: emit `TurnStart`, call LLM via `call_with_retry`, push assistant message, check if `stop_reason == ToolUse`. If yes, execute tools sequentially, push tool results as a user message, increment turn. If no, emit `TurnEnd` + `AgentEnd` and return.

3. **Three providers**: `AnthropicProvider` (native Anthropic API with `x-api-key` header), `OpenAiProvider` and `OpenRouterProvider` (both use shared `call_openai_compat` function with `Authorization: Bearer` header). All implement `StreamProvider` trait (currently non-streaming despite the name).

4. **Six tools**: `BashTool` (30s timeout, deny-list, 50KB truncation), `ReadFileTool` (1MB max, binary detection), `WriteFileTool` (auto-creates parent dirs), `EditFileTool` (exactly-one-match replacement), `ListFilesTool` (200 entry limit, 8 depth limit), `SearchTool` (prefers rg, falls back to grep, 15s timeout).

5. **Retry logic**: `call_with_retry` attempts up to `max_attempts` (default 5). Retries on `RateLimited` and `Network` errors. Uses exponential backoff with deterministic jitter. Now respects `Retry-After` header from 429 responses.

6. **Config**: `config.toml` with `journal_lines`, `active.provider`, `active.endpoint`, `active.model`. API keys from environment variables.

7. **Context loading**: Reads `identity.md` (system prompt), `journal.md` (last N lines), and `Input.md` (external input, cleared after reading). Increments `iteration_count`.

8. **Test gates**: `cargo build --quiet` and `cargo test --quiet` run both before and after the agent loop. If either fails, the run is aborted and a failure entry is written to the journal.

9. **Message format**: Anthropic-native format internally. `Message` has `role` and `content: Vec<Content>`. `Content` is a tagged enum: `Text`, `ToolUse`, `ToolResult`.

10. **ProviderError classification**: HTTP 429 → `RateLimited` (with optional `retry_after_ms`), 400+context → `ContextTooLong`, 400 → `InvalidRequest`, 5xx → `Network`, other → `Other`.

11. **No streaming yet**: Despite the trait being named `StreamProvider`, all providers do synchronous request-response. Streaming is a planned improvement (Issue #2).

12. **Dependencies**: tokio (async runtime), reqwest (HTTP), serde/serde_json (serialization), toml (config parsing), async-trait (async trait support). Dev: tempfile.

## Git History

- `43fd41e` — "max attempt increased" (initial commit, the only one so far)

## Extending the Agent

13. **Custom tools go in `src/agent/`**: Implement `AgentTool` trait, return from `extra_tools()` in `src/agent/mod.rs`. They're added to the base tool list at runtime. First custom tools: `GitStatusTool`, `GitDiffTool`, `GitLogTool` (iteration 4). Added `WorkingMemoryTool` and `ReadFileRangeTool` (iteration 5).

14. **Tool modules**: `src/agent/tools.rs` holds tool implementations, `src/agent/mod.rs` wires them into `extra_tools()`.

15. **Git tools use a shared `run_git_command()` helper**: 10s timeout, combined stdout+stderr, 30KB truncation, clean "(no output)" message for empty results.

16. **WorkingMemoryTool**: Writes/reads `/tmp/baby_phi_working_memory.md`. Use action='write' with content= to save state, action='read' to restore. The file is ephemeral (survives within a run, gone after reboot). Call when turn count > 15 to avoid context loss.

17. **ReadFileRangeTool**: Reads N lines from a file starting at an offset. Default limit=50, max=200. Returns `[lines X-Y of Z total]` summary at the end. Use instead of `read_file` for large files when you only need a section.

18. **`.lines()` + `join("\n")` + suffix pattern**: When appending a summary to joined lines (e.g., `"\n\n[summary]"`), the double newline creates an empty line in `.lines()` iteration. Tests that filter by content must also filter empty lines.

## GitHub API Access

16. **`GITHUB_TOKEN` in CI is read-only for issues**: Can list issues but not close or comment. Closing issues (Issue #8) requires `issues: write` permission in the workflow YAML.

## Known Limitations

- No streaming output (entire response appears at once)
- No terminal colors (all output is monochrome)
- ~~No git awareness~~ → Added git_status, git_diff, git_log tools (iteration 4)
- No REPL mode (runs once per invocation)
- No conversation persistence between runs
- OpenAI-compat providers lose tool_use/tool_result context (flattens to text)
- No context window management (no compaction, no working memory)
- **Working memory tool added (iteration 5)** — agent can now persist mid-run state
- No turn limit in agent loop (could loop forever on tool-use cycles)
- Jitter in RetryConfig is deterministic (based on attempt parity, not random)
- CI token can't close GitHub issues (needs `issues: write` permission)
