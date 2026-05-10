<!-- Last verified: 2026-05-10 by Claude Code (CH-20 P1 — NEW convention doc per v0/conventions/ peer tier, cycle hex 240616a4) -->

# Web patterns conventions

Reviewer-tier guidance for the Next.js 14 server-action shape. Governance authority: [ADR-0058](../implementation/m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md) §D58.7.

## Next.js server-action shape

Hybrid: top-level sibling `actions.ts` (named exports) for actions reused across the page + inline per-row `<form action={run}>` closures with `"use server"` directive at the function body when the closure captures a row-specific dynamic id.

Inline closures MUST capture string-only data (e.g. `orgId`, `templateKind`) — Next.js 14 dynamic-id pattern is serialization-safe only for string captures; capturing a struct or object breaks the server-action boundary.

- **Reviewer rule:** server-action closures capture string-only; row-keyed actions inline; cross-row actions go to sibling `actions.ts`.
- **Closes:** D7.6. **Cross-ref:** ADR-0058 §D58.7.
