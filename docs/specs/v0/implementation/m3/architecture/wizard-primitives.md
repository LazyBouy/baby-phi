<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Web wizard primitives

**Status: [EXISTS]** — scaffolded in M3/P1; consumed by P4's org-creation wizard + future M4+ project wizard.

Four reusable React components in
`modules/web/app/components/wizard/`:

- **`StepShell`** — step container: heading + content slot + error-alert slot.
- **`StepNav`** — Back / Next / Save-draft / Submit buttons with disabled-state logic (Back disabled on step 1; Next disabled while current step invalid; Submit visible only on review step).
- **`DraftContext`** — React Context wrapping `sessionStorage` primary + `localStorage` fallback. Exposes `useDraft<T>(key)` hook per ADR-0021.
- **`ReviewDiff`** — before/after panel for the wizard review step; renders field-level diffs client-side.

## Pattern for M4+ reuse

Any multi-step admin surface (M4's project-creation wizard, future
agent-provisioning wizard, etc.) mounts `StepShell` + `StepNav` per
step + wraps in `DraftContext`. Per-step validation lives in the
step component; the wizard shell just orchestrates nav + final
submit. No M3-specific coupling.

See:
- [`../decisions/0021-wizard-autosave-session-storage.md`](../decisions/0021-wizard-autosave-session-storage.md) — ADR for the draft-persistence choice.
- [`../../../../../../modules/web/app/components/wizard/`](../../../../../../modules/web/app/components/wizard/) — component source.

## phi-core leverage

None — web UI primitive. Phi-core has no web-tier surface.
