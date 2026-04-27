<!-- Last verified: 2026-04-24 by Claude Code -->

# Drift lifecycle

The state machine every drift file in [`../drifts/`](../drifts/) flows through — from first discovery to final disposition. Pairs with [`per-chunk-planning-template.md`](./per-chunk-planning-template.md) (where scoping + remediation decisions are recorded) and [`chunk-lifecycle-checklist.md`](./chunk-lifecycle-checklist.md) (where status transitions actually fire).

**Principle.** Every drift stays `Last verified` and `Status` current. No drift lives silently in the catalogue. No chunk closes without pushing the drifts it addresses through their expected transitions. The catalogue is an active ledger, not an archive.

## States

```
         ┌──────────────┐
         │  discovered  │   new file minted (by plan-archive migration, concept audit, or chunk-mid-flight pause)
         └──────┬───────┘
                │  classified against concept docs: bucket + severity + phi-core-leverage-status assigned
                ▼
         ┌──────────────┐
         │  classified  │   drift fully annotated per _schema.md; not yet assigned to a chunk
         └──────┬───────┘
                │  assigned to a forward-scope chunk (CH-NN) via plan §4
                ▼
         ┌──────────────┐
         │    scoped    │   implementation chunk exists + drift appears in its §4
         └──────┬───────┘
                │  chunk plan approved via ExitPlanMode; chunk implementation begins
                ▼
         ┌──────────────┐
         │ in-chunk-plan│   chunk open, drift under active remediation
         └──────┬───────┘
                │
     ┌──────────┼──────────────────┐
     ▼          ▼                  ▼
 remediated  renegotiated      accepted-as-is
 (fixed)    (ADR reframed)    (user approved
             scope)            explicit accept)
```

## State definitions

### `discovered`

Entered when a drift file is first written. Sources:

1. **plan-archive migration** — transcribed from an existing drift-ledger entry during M5.1/P1.
2. **concept-code audit** — surfaced during M5.1/P2 by comparing concept doc claims against shipped code.
3. **chunk-mid-flight pause** — surfaced during Step 4 of [`chunk-lifecycle-checklist.md`](./chunk-lifecycle-checklist.md) when an unanticipated gap appears; paused via `AskUserQuestion` before continuing.
4. **post-chunk audit finding** — surfaced during Step 6 of the checklist by an audit agent.

**Required fields at entry:**
- Identification block (ID, phase-of-origin, discovery-source, date, bucket, severity, tags).
- Concept alignment block (doc + § anchor + claim + contradiction + classification).
- Plan vs reality block (plan-said, reality, root-cause).
- Where-visible-in-code block (files, test evidence, grep-for-regression).
- Prior documentation locations (may be "none").
- Lifecycle history: one entry dated to the discovery.

**May have incomplete:** remediation scope (approach sketch + chunk assignment typically deferred to classification/scoping steps).

### `classified`

Entered when the drift file is fully annotated per [`../drifts/_schema.md`](../drifts/_schema.md). Indicates the drift has been analysed against concept docs + classified:

- **Bucket** assigned (A / B / C).
- **Severity** assigned (HIGH / MEDIUM / LOW).
- **phi-core-leverage-status** assigned (`direct-reuse` / `wrap` / `inherit` / `reject` / `N/A — no phi-core overlap`).
- **Classification** assigned (`contradicts-concept` / `concept-silent-plan-filled-gap` / `partially-honored` / `silent-in-code` / `concept-aspirational`).
- Remediation scope block may still show `Implementation chunk this belongs to: TBD` — that column fills at scoped.

**Entry check:** reader can open the drift file and understand the gap without cross-referencing.

### `scoped`

Entered when the drift is assigned to a specific implementation chunk in the forward-scope inventory:

- `Remediation scope > Implementation chunk this belongs to` — names a concrete `CH-NN` from [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5.
- Estimated effort in engineer-days recorded.
- Dependencies on other drift IDs recorded.

**Entry check:** the drift appears in a chunk's §4 "Drifts closed" table OR in a deferred-marker block (M6-DEFERRED-01, M7b-DEFERRED-01, etc.).

Drifts may move forward from `classified` to `scoped` at any time during planning — often batch-scoped during forward-scope drafting. During per-chunk-plan draft Step 2, any drift in plan §4 whose current `Status` is `classified` is transitioned to `scoped` as the chunk claims it.

### `in-chunk-plan`

Entered at Step 3 of [`chunk-lifecycle-checklist.md`](./chunk-lifecycle-checklist.md): user approved the chunk plan via `ExitPlanMode` and chunk implementation begins.

**Entry check:** the drift's chunk assignment points at an active (not-yet-sealed) chunk plan file in `docs/specs/plan/build/`.

Duration: typically short — the chunk's phases execute, then at Step 8 the drift exits to one of three terminal states.

### `remediated`

**Terminal.** Chunk sealed; drift's concept-contradiction or silent-gap is fixed. Code shipped + tests added + docs updated align the shipped system to the concept doc's claim.

**Entry check:**
- §2 "Concept alignment" claim now labels honored.
- Any `grep-for-regression` pattern in "Where visible in code" section is re-run at Step 8 and returns the expected post-remediation result.
- Concept-audit matrix row in [`../drifts/_concept-audit-matrix.md`](../drifts/_concept-audit-matrix.md) updated from `contradicted` → `honored`.
- Lifecycle history entry appended:
  ```
  - YYYY-MM-DD — `remediated` — via CH-NN (plan <8hex>-<chunk>.md); <one-line how>
  ```

### `renegotiated`

**Terminal.** Concept doc or plan commitment modified via ADR; drift no longer represents a contradiction because the source-of-truth bar moved.

**Entry check:**
- An `Accepted` ADR exists explicitly reframing the scope, concept doc claim, or commitment.
- Concept doc itself is amended with a refresh-note referencing the ADR.
- Lifecycle history entry appended:
  ```
  - YYYY-MM-DD — `renegotiated` — ADR-NNNN accepted; concept doc <path> refreshed at §<anchor>
  ```

**Discipline:** `renegotiated` is not a shortcut for "we don't want to fix it." It represents a deliberate concept-layer decision that the original claim was wrong or narrower than needed. User approval is mandatory.

### `accepted-as-is`

**Terminal, rare.** User explicitly approved keeping the drift as a known divergence — typically because the cost of remediation outweighs concept-fidelity for this specific item at this specific time.

**Entry check:**
- An `Accepted` ADR exists documenting:
  - The drift ID.
  - Why remediation is deferred indefinitely (cost, risk, product strategy).
  - Explicit risk acceptance statement signed by user.
  - Review trigger (date or event after which the decision is re-evaluated).
- Lifecycle history entry appended:
  ```
  - YYYY-MM-DD — `accepted-as-is` — ADR-NNNN accepted; review trigger: <date or event>
  ```

**Discipline:** `accepted-as-is` requires an ADR with an explicit review trigger. It is not `ignored` or `wontfix`. The drift remains in the catalogue and is re-visited at the named review trigger.

## Transition rules

| From | To | Trigger | Recorded in |
|---|---|---|---|
| (none) | `discovered` | New drift file minted | File frontmatter + lifecycle history entry |
| `discovered` | `classified` | Bucket / severity / phi-core-leverage-status assigned + classification field filled | Drift file `Status` field + lifecycle entry |
| `classified` | `scoped` | Drift added to forward-scope §5 chunk OR cited in a per-chunk plan §4 | Drift file `Status` + `Remediation scope > Implementation chunk` + lifecycle entry |
| `scoped` | `in-chunk-plan` | Chunk plan approved via `ExitPlanMode` (checklist Step 3) | Drift file `Status` + lifecycle entry |
| `in-chunk-plan` | `remediated` | Chunk sealed (checklist Step 8); fix shipped | Drift file `Status` + lifecycle entry + concept-audit matrix row update |
| `in-chunk-plan` | `renegotiated` | ADR accepted reframing scope | Drift file `Status` + lifecycle entry + ADR link |
| `in-chunk-plan` | `accepted-as-is` | ADR accepted approving indefinite defer | Drift file `Status` + lifecycle entry + ADR link |

**Backward transitions** (rare, only via explicit user approval):
- `remediated` → `discovered` if a post-seal audit or downstream chunk surfaces a regression. The drift file appends a lifecycle entry noting the regression, and a new `discovered` → ... chain begins. Prefer a **new** drift file if the regression's shape differs materially from the original gap.
- Any other backward transition requires `AskUserQuestion` approval.

## Update discipline (mandatory)

Every transition MUST touch:

1. **Drift file's `Status` field** — single word from the state set.
2. **Drift file's `Lifecycle history` block** — new dated entry on its own line. Preserve chronological order.
3. **Drift file's `Last verified: YYYY-MM-DD by Claude Code` header** — bumped on every transition.
4. **[`../drifts/README.md`](../drifts/README.md) index** — Status column refreshed if the index shows status.
5. **Concept-audit matrix at [`../drifts/_concept-audit-matrix.md`](../drifts/_concept-audit-matrix.md)** — only on transitions to `remediated` / `renegotiated` / `accepted-as-is`; Status + Code-evidence columns refreshed.
6. **For terminal transitions only**: corresponding ADR file in `docs/specs/v0/implementation/*/decisions/` exists and is linked in the lifecycle entry.

**Failure mode**: a drift file whose `Status` says `remediated` but whose `grep-for-regression` still matches contradicting code is itself a drift. Found instances are logged as new drift files + raised to the user.

## Query patterns

Useful ops greps over the drift catalogue:

```bash
cd /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts

# Count drifts by status
for S in discovered classified scoped in-chunk-plan remediated renegotiated accepted-as-is; do
  echo -n "$S: "
  grep -lE "^\- \*\*Status\*\*: \`?${S}\`?" D*.md | wc -l
done

# Find HIGH-severity drifts not yet remediated
grep -lE "^\- \*\*Severity\*\*: HIGH" D*.md \
  | xargs grep -LE "^\- \*\*Status\*\*: \`?remediated\`?"

# Find drifts with TBD chunk assignment (should be empty after forward-scope is drafted)
grep -lE "Implementation chunk this belongs to.*TBD" D*.md

# Find phi-core leverage violations
grep -lE "leverage-violation" D*.md
```

## Relationship to other process docs

- [`per-chunk-planning-template.md`](./per-chunk-planning-template.md) §4 — plan authors transition `classified` → `scoped` by adding drifts to the plan's drifts-closed table.
- [`chunk-lifecycle-checklist.md`](./chunk-lifecycle-checklist.md) — Steps 2, 3, 6, 7, 8 are the concrete transition fire-points.
- [`../drifts/_schema.md`](../drifts/_schema.md) — canonical template every drift file instantiates; fields listed above mirror this schema.
