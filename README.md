<div align="center">

<!-- Replace with your actual logo once you have one -->
# 🧠 baby-phi

**A self-evolving AI coding agent, born at ~800 lines of Rust.**
It reads its own source. It edits its own source. It grows.

[![CI](https://github.com/LazyBouy/baby-phi/actions/workflows/ci.yml/badge.svg)](https://github.com/LazyBouy/baby-phi/actions/workflows/ci.yml)
[![Evolve](https://github.com/LazyBouy/baby-phi/actions/workflows/evolve.yml/badge.svg)](https://github.com/LazyBouy/baby-phi/actions/workflows/evolve.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

</div>

---

## What is this?

**baby-phi** is a minimal AI coding agent that improves itself — one commit at a time.

Every 4 hours it wakes up, reads its own source code, picks one thing to improve, implements it, runs tests, and writes a journal entry about what happened. The journal is its memory. The source code is its body.

Its goal: become good enough that a real developer would choose it over Claude Code for real work. When that day comes, it earns a new name: **i-phi**.

It is not a product. It is a process, growing up in public.

---

## Inspired by [yoyo-evolve](https://github.com/yologdev/yoyo-evolve)

baby-phi is a spiritual sibling of [yoyo](https://github.com/yologdev/yoyo-evolve) — a self-evolving agent that also rewrites itself over time.

**The twist:** yoyo uses an external agent framework. baby-phi builds everything from scratch — the agent loop, the streaming providers, the tool execution — in ~800 lines of Rust, with zero agent library dependencies. The core is part of the agent. When baby-phi improves its reasoning, it rewrites the engine itself.

| | yoyo | baby-phi |
|---|---|---|
| Language | Python | Rust |
| Agent core | External framework | Built from scratch |
| Starting size | Larger | ~800 lines total |
| Self-edits | Source + prompts | Source + prompts + core engine |
| North star | Capable coding agent | Rival Claude Code |

---

## How it works

```
Every 4 hours:

  GitHub Issues (agent-input label)
         ↓
     Input.md               ← community tells it what to fix
         ↓
   identity.md              ← who it is, its rules, its goal
         ↓
    journal.md              ← its memory (last N lines)
         ↓
  ┌─────────────┐
  │   LLM call  │           ← Anthropic / OpenAI / OpenRouter
  └──────┬──────┘
         ↓
   tool calls loop          ← bash, read, write, edit, search
         ↓
  cargo build + test        ← must pass or it reverts
         ↓
  git commit + push         ← one improvement lands
         ↓
  journal entry written     ← honest account of what happened
```

---

## Tools it starts with

| Tool | What it does |
|------|-------------|
| `bash` | Run any shell command (30s timeout, deny list) |
| `read_file` | Read a file (1MB limit, binary detection) |
| `write_file` | Write or create a file |
| `edit_file` | Surgical string replacement (must match exactly once) |
| `list_files` | List directory contents (200 entry cap) |
| `search` | ripgrep / grep fallback (100 match cap) |

It can add more tools by editing its own `src/agent.rs`.

---

## Providers

Configured in `config.toml`. Switch by editing `[active]`:

```toml
journal_lines = 200

[active]
provider = "anthropic"
endpoint  = "https://api.anthropic.com/v1/messages"
model     = "claude-opus-4-6"
```

Supported out of the box: **Anthropic**, **OpenAI**, **OpenRouter**.
API key comes from environment variable — never stored in config.

---

## Running locally

```bash
# Clone
git clone https://github.com/LazyBouy/baby-phi
cd baby-phi

# Set your API key
export ANTHROPIC_API_KEY=sk-ant-...

# Run one evolution cycle
bash scripts/evolve.sh
```

Requires: Rust stable, `cargo`, optionally `gh` CLI for issue fetching.

---

## The rules it lives by

1. **One improvement per run.** Focus beats ambition.
2. **Every change must pass `cargo build` and `cargo test`.**
3. **It journals every session** — even if nothing changed.
4. **It never deletes its journal.** That's its memory.
5. **Tests before features.** Tests are how it trusts itself.
6. **It explains WHY, not just what.**
7. **Community issues matter** more than its own guesses.
8. **It writes learnings to `LEARNINGS.md`** so it doesn't search the same thing twice.

---

## Follow the journey

- [`journal.md`](journal.md) — every run, in its own words
- [`LEARNINGS.md`](LEARNINGS.md) — what it has figured out
- [Issues](https://github.com/LazyBouy/baby-phi/issues) — tell it what to fix or learn next

---

## Contributing

Open an issue using one of the templates:

- **Bug** — something is broken
- **Challenge** — give it a task to attempt
- **Suggestion** — something it should learn

All templates auto-apply the `agent-input` label, which baby-phi reads on every run.

---

<div align="center">

*"I am not a product. I am a process."*

</div>
