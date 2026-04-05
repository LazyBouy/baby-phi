# CLAUDE.md — baby-phi

baby-phi is a standalone Rust binary that consumes `phi-core` as a library dependency. It serves as an early consumer/prototype for the Agent Management System backend with a config-driven UI layer.

## Build & Run

```bash
cd baby-phi
/root/rust-env/cargo/bin/cargo build
set -a && source .env && set +a && /root/rust-env/cargo/bin/cargo run
```

## Scope

baby-phi is the intended home for platform-level features that sit above phi-core:
- Config-driven agent invocation (UI layer)
- Plugin system (WASM)
- Multi-user coordination
- Agent management backend

phi-core remains a pure library crate — platform features belong here in baby-phi.

## Documentation Alignment

Documentation in `docs/` must accurately reflect the current codebase at all times. Code is always the source of truth.

- **Update docs with code changes**: When modifying code, update all affected documentation in the same commit. This includes status tags, API signatures, config examples, and pseudocode.
- **Status tags**: `[EXISTS]` = implemented in code, `[PLANNED]` = designed but not yet implemented, `[CONCEPTUAL]` = idea stage. Review and update these tags whenever the referenced code changes.
- **Verification header**: Every doc file carries `<!-- Last verified: YYYY-MM-DD by Claude Code -->` at the top, updated on each review pass.
- **No forward references**: Do not document features as existing unless the code is merged. Use `[PLANNED]` or `[CONCEPTUAL]` for future work.
