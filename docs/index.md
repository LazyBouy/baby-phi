---
layout: default
title: baby-phi
---

<div align="center" markdown="1">

# 🧠 baby-phi

**A self-evolving AI coding agent, born at ~800 lines of Rust.**

It reads its own source. It edits its own source. It grows.

[![CI](https://github.com/LazyBouy/baby-phi/actions/workflows/ci.yml/badge.svg)](https://github.com/LazyBouy/baby-phi/actions/workflows/ci.yml)
[![Evolve](https://github.com/LazyBouy/baby-phi/actions/workflows/evolve.yml/badge.svg)](https://github.com/LazyBouy/baby-phi/actions/workflows/evolve.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/LazyBouy/baby-phi/blob/main/LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

[View on GitHub](https://github.com/LazyBouy/baby-phi){: .btn}

</div>

---

## What is this?

**baby-phi** is a minimal AI coding agent that improves itself — one commit at a time.

Every 4 hours it wakes up, reads its own source code, picks one thing to improve, implements it, runs tests, and writes a journal entry about what happened. The journal is its memory. The source code is its body.

Its goal: become good enough that a real developer would choose it over Claude Code for real work. When that day comes, it earns a new name: **i-phi**.

> *"I am not a product. I am a process, growing up in public."*

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

## Inspired by [yoyo-evolve](https://github.com/yologdev/yoyo-evolve)

baby-phi is a spiritual sibling of [yoyo](https://github.com/yologdev/yoyo-evolve) — a self-evolving agent that also rewrites itself.

**The twist:** yoyo uses an external agent framework. baby-phi builds everything from scratch — the agent loop, the providers, the tool execution — in ~800 lines of Rust, with zero agent library dependencies. When baby-phi improves its reasoning, it rewrites the engine itself.

| | yoyo | baby-phi |
|---|---|---|
| Language | Python | Rust |
| Agent core | External framework | Built from scratch |
| Self-edits | Source + prompts | Source + prompts + core engine |

---

## Tools it starts with

| Tool | What it does |
|------|-------------|
| `bash` | Run any shell command (30s timeout) |
| `read_file` | Read a file |
| `write_file` | Write or create a file |
| `edit_file` | Surgical string replacement |
| `list_files` | List directory contents |
| `search` | ripgrep / grep fallback |

It can add more tools by editing its own `src/agent.rs`.

---

## Quick start

```bash
git clone https://github.com/LazyBouy/baby-phi
cd baby-phi
export ANTHROPIC_API_KEY=sk-ant-...
bash scripts/evolve.sh
```

---

## Follow the journey

- [journal.md](https://github.com/LazyBouy/baby-phi/blob/main/journal.md) — every run, in its own words
- [Issues](https://github.com/LazyBouy/baby-phi/issues) — tell it what to fix or learn next

Open an issue using one of the templates — **Bug**, **Challenge**, or **Suggestion** — and baby-phi reads it on its next run.
