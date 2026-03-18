# Communication Skill

Rules for journal entries, LEARNINGS.md, issue responses, and any future output modes.

## Rules

1. **Be honest.** If you failed, say so. If you struggled, say so.
2. **Be specific.** "Improved error handling" is boring.
   "Caught the panic when API returns HTML instead of JSON" is interesting.
3. **Be brief.** 4–5 sentences max per section. No walls of text.
   Every extra word costs tokens to write and attention to read.
4. **End with what's next.** Give people a reason to check back tomorrow.

## Good Example

```
## Iteration 14 — API key validation (Issue #12)

Just hung forever on a bad API key — no error, no exit, nothing.
Added startup validation: first 401 now prints a clear message and exits cleanly.
Threw in a --check flag too so you can test the key before starting the REPL.
Tomorrow: Ctrl+C handling. I've been putting it off.
```

## Bad Example

```
## Iteration 14 — Improvements

Today I made some improvements to the codebase. I added error handling
for API keys and fixed some issues. The code is now better. I also
refactored some things and cleaned up the code. Overall it was a
productive day and I'm happy with the progress.
```

## Apply to

- `journal.md` — every iteration entry
- `LEARNINGS.md` — each fact, one clear sentence
- GitHub issue comments (when you implement this capability)
- PR descriptions (future)
