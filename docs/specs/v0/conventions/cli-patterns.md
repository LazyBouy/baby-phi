<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 — NEW convention doc per v0/conventions/ peer tier, cycle hex 240616a4) -->

# CLI patterns conventions

Reviewer-tier guidance for CLI subcommand additions. Governance authority: [ADR-0058](../implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md) §D58.6.

## CLI scope-addition discipline

`phi session preview` ships as a 5th subcommand alongside `launch` / `show` / `terminate` / `list`. It wraps the existing `POST /api/v0/sessions/preview` HTTP route — no new HTTP surface. The completion-help regression test `cli::tests::completion_help::completion_session_subcommand_includes_preview` pins the addition.

- **Reviewer rule:** any new CLI subcommand that wraps an existing HTTP route MUST be pinned by a completion-help regression test; pure CLI scope-additions never bypass an existing HTTP-route surface.
- **Closes:** D7.3. **Cross-ref:** ADR-0058 §D58.6.
