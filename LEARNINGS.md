# LEARNINGS.md — baby-phi's long-term memory

> **Note:** You are a growing system, and so is the core you run on. Facts recorded here reflect what was true at the time of writing — not necessarily what is true now. When something you've learnt no longer matches reality, that isn't a mistake. It's just what growth looks like. Re-explore, correct the entry, and move on.


Last updated: Iteration 15

## Codebase Architecture

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

- `013231b` — "bug fix - baby dont worry even claude makes bug"
- `6be302b` — "model update to anthropic - baby needs a good school"

(Note: Earlier commits are not visible — likely squashed or reset. Iterations 1–14 span these two commits.)

## Extending the Agent

13. **Custom tools go in `src/agent/`**: Implement `AgentTool` trait, return from `extra_tools()` in `src/agent/mod.rs`. They're added to the base tool list at runtime. First custom tools: `GitStatusTool`, `GitDiffTool`, `GitLogTool` (iteration 4). Added `WorkingMemoryTool` and `ReadFileRangeTool` (iteration 5).

14. **Tool modules**: `src/agent/tools.rs` holds tool implementations, `src/agent/mod.rs` wires them into `extra_tools()`.

15. **Git tools use a shared `run_git_command()` helper**: 10s timeout, combined stdout+stderr, 30KB truncation, clean "(no output)" message for empty results.

16. **WorkingMemoryTool**: Writes/reads `/tmp/baby_phi_working_memory.md`. Use action='write' with content= to save state, action='read' to restore. The file is ephemeral (survives within a run, gone after reboot). Call when turn count > 15 to avoid context loss.

17. **ReadFileRangeTool**: Reads N lines from a file starting at an offset. Default limit=50, max=200. Returns `[lines X-Y of Z total]` summary at the end. Use instead of `read_file` for large files when you only need a section.

18. **`.lines()` + `join("\n")` + suffix pattern**: When appending a summary to joined lines (e.g., `"\n\n[summary]"`), the double newline creates an empty line in `.lines()` iteration. Tests that filter by content must also filter empty lines.

19. **ProjectInfoTool**: Added in iteration 8. Detects project type from manifest files (Cargo.toml, package.json, etc.), extracts name/version/deps, counts source files + test functions, finds entry points, shows last git commit. Use this at the start of a session on an unfamiliar codebase instead of manually running ls + cat Cargo.toml. Accepts optional `path` param (default `"."`).

20. **`extract_toml_field()`**: Simple TOML key extractor — no full parser needed. Scans for `key = "value"` lines. Does NOT handle multi-line values or complex TOML. Sufficient for name/version/edition fields.

21. **`extract_toml_section_keys()`**: Extracts key names from a TOML section (e.g. `[dependencies]`). Caps at 12 to keep output compact. Stops at next `[` header.

22. **GithubTool**: Added in iteration 15. Actions: `list_issues` (lists open issues with counts), `close_issue` (closes issue by number, optional comment first), `add_comment` (posts comment on issue). Requires `GH_TOKEN` or `GITHUB_TOKEN` env var. Auto-detects repo from `git remote get-url origin` (handles both HTTPS and SSH URLs). Uses `curl` subprocess to avoid reqwest dependency complexity. Helper fns: `github_get/patch/post()` → `github_request()` → `build_curl_args()`.

23. **FetchUrlTool**: Added in iteration 18. Fetches any http/https URL via `curl -s -L`, strips HTML to plain text via `strip_html()`, caps output at 10KB. JSON responses returned as-is. Rejects non-http(s) schemes. Optional `timeout` param (default 15s, max 60s). Use for fetching docs, API refs, GitHub pages.

24. **GlobFilesTool**: Added in iteration 19. Finds files matching glob patterns — `*.rs`, `src/**/*.ts`, `**/*.test.js`. Uses GNU `find` under the hood. Always excludes `target/`, `node_modules/`, `.git/`, `.cargo/`. Optional `root` param (default `.`). Max 500 results. `parse_glob()` converts the pattern to (search_root, find_name_pattern, max_depth): `src/*.rs` → depth=1, `**/*.rs` → unlimited. Faster and cleaner than writing bash `find` one-liners.

25. **OllamaProvider**: Added in iteration 20. Implements `StreamProvider` for local Ollama instances via OpenAI-compatible API (`http://localhost:11434/v1/chat/completions`). No API key required. Registered as "ollama" in `extra_providers()`. Configure via `provider = "ollama"` in config.toml. Model resolved from `PHI_MODEL` env var or `config.toml`. Endpoint overridable via `OLLAMA_HOST` env var. Connection refused → helpful error "Ollama is not running. Start it with: ollama serve". Located in `src/agent/providers.rs`.

26. **extra_providers() model resolution**: The core doesn't pass the resolved model to `extra_providers()`. Workaround in `mod.rs`: read `PHI_MODEL` env var first, then fall back to parsing `config.toml` directly with `toml::from_str::<crate::core::Config>()`. This mirrors `ActiveCfg::resolved_model()` logic in core.

27. **Tool-use guidance in extra_context()**: Added in iteration 20. A compact "## Tool Use Instructions" block is always injected. Tells weaker models to call tools immediately (don't describe, just do), use exact tool names, act autonomously without asking permission. Helps local/smaller models follow the tool-call loop reliably.


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
