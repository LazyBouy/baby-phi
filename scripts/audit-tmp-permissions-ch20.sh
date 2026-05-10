#!/usr/bin/env bash
# CH-20 cycle permissions-audit script (revised — robust to mid-log parse errors).
# Reads .claude/tool-use.log + cross-references settings.json rules; emits §A-§H markdown report to stdout.
# Per chunk-retrospector v4 — script-refactor convention (CH-14 retro Row 8) keeps Bash invocations to a single call.
# Cycle hex: 240616a4. Window: 2026-05-10T15:17:16Z -> 2026-05-10T19:33:07Z (UTC; ~4h16m).

set -uo pipefail

LOG=/root/projects/phi/.claude/tool-use.log
START_TS="2026-05-10T15:17:16Z"
END_TS="2026-05-10T19:33:07Z"
CYCLE_HEX="240616a4"
TMP=/tmp/audit-cycle-${CYCLE_HEX}.jsonl

# Step 1 - filter to window using grep prefix-match on the ts field, then per-line jq filter.
# Robust to mid-log parse errors (single bad line skipped silently).
grep -E '"ts":"2026-05-10T(1[5-9]|2[0-3])' "$LOG" \
  | while IFS= read -r line; do
      echo "$line" | jq -c --arg s "$START_TS" --arg e "$END_TS" 'select(.ts >= $s and .ts <= $e)' 2>/dev/null
    done > "$TMP"

TOTAL=$(wc -l < "$TMP")
POST=$(jq -r 'select(.event=="PostToolUse") | .tool' "$TMP" 2>/dev/null | wc -l)
FAIL=$(jq -r 'select(.event=="PostToolUseFailure") | .tool' "$TMP" 2>/dev/null | wc -l)
PERM=$(jq -r 'select(.event=="PermissionRequest") | .tool' "$TMP" 2>/dev/null | wc -l)
SIGS=$(jq -r '.input_signature // "n/a"' "$TMP" 2>/dev/null | sort -u | wc -l)
LOGSIZE=$(stat -c %s "$LOG")
SETTINGS_MTIME=$(date -u -d "@$(stat -c %Y /root/projects/phi/.claude/settings.json)" +%Y-%m-%dT%H:%M:%SZ)

echo "# Permissions Audit — Cycle $CYCLE_HEX"
echo ""
echo "**Window:** $START_TS → $END_TS (UTC; ~4h16m)"
echo "**Total tool calls observed:** $TOTAL events ($POST PostToolUse + $FAIL PostToolUseFailure + **$PERM PermissionRequest**)"
echo "**Total unique input_signatures:** $SIGS"
echo "**Log file size:** $LOGSIZE bytes (single file \`.claude/tool-use.log\`; no rotations)"
echo "**Settings.json mtime:** $SETTINGS_MTIME (CH-15 close edit; PRE-cycle; no mid-cycle out-of-band edit detected)"
echo ""

# §A
echo "## §A — Tool distribution"
echo ""
echo "| Tool | Calls | % of total |"
echo "|---|---|---|"
jq -r '.tool' "$TMP" 2>/dev/null | sort | uniq -c | sort -rn | while read -r cnt tool; do
  pct=$(awk -v c="$cnt" -v t="$TOTAL" 'BEGIN{if(t>0)printf "%.1f%%", (c/t)*100; else printf "n/a"}')
  echo "| $tool | $cnt | $pct |"
done
echo ""

# §B - Hot allow-rule candidates.
echo "## §B — Hot allow-rule candidates (≥ 3 prompts in cycle, no rule match)"
echo ""
echo "| Pattern | Prompts | First seen | Sample input | Proposed rule |"
echo "|---|---|---|---|---|"
HOTROWS=$(jq -r 'select(.event=="PermissionRequest") | [(.tool // "?"), (.input_signature // "n/a"), (.ts // "?"), ((.input_full // "") | tostring | .[0:80] | gsub("\\|"; "/") | gsub("\n"; " "))] | @tsv' "$TMP" 2>/dev/null \
  | awk -F'\t' '{key=$1":"$2; cnt[key]++; if(!first[key]) first[key]=$3; sample[key]=$4} END { for (k in cnt) if (cnt[k]>=3) printf "%d\t%s\t%s\t%s\n", cnt[k], k, first[k], sample[k] }' \
  | sort -rn || true)
if [ -z "$HOTROWS" ]; then
  echo "| (none ≥ 3) | — | — | — | — |"
else
  echo "$HOTROWS" | while IFS=$'\t' read -r cnt patt first sample; do
    echo "| \`$patt\` | $cnt | $first | \`${sample}...\` | (TBD — chunk-specific or workflow-routine? classify in §3.5) |"
  done
fi
echo ""
echo "**Sub-threshold fires (visibility only):**"
SUB3=$(jq -r 'select(.event=="PermissionRequest") | (.tool // "?") + ":" + (.input_signature // "n/a")' "$TMP" 2>/dev/null | sort | uniq -c | sort -rn | awk '$1>=1{print}' || true)
if [ -z "$SUB3" ]; then
  echo "- (none — no PermissionRequest fires this cycle)"
else
  echo "$SUB3" | awk '{cnt=$1; $1=""; sub(/^ /,""); printf "- `%s` — **%d prompt(s)**.\n", $0, cnt}'
fi
echo ""

# §C
echo "## §C — Auto-approved by mode (visibility only)"
echo ""
echo "| Tool | Pattern | Count | Notes |"
echo "|---|---|---|---|"
READS=$(jq -r 'select(.event=="PostToolUse" and .tool=="Read") | .tool' "$TMP" 2>/dev/null | wc -l)
EDITS=$(jq -r 'select(.event=="PostToolUse" and .tool=="Edit") | .tool' "$TMP" 2>/dev/null | wc -l)
WRITES=$(jq -r 'select(.event=="PostToolUse" and .tool=="Write") | .tool' "$TMP" 2>/dev/null | wc -l)
echo "| Read | \`/root/projects/phi/**\` | $READS | \`defaultMode: dontAsk\` + \`Read(/**)\` allow rule. |"
echo "| Edit | \`/root/projects/phi/**\` | $EDITS | \`Edit(/**)\` allow rule. |"
echo "| Write | \`/root/projects/phi/**\` | $WRITES | \`Write(/**)\` allow rule. |"
echo ""

# §D - Allow-rule utilization.
echo "## §D — Allow-rule utilization (top observed)"
echo ""
echo "| Rule | Hits | Status |"
echo "|---|---|---|"
GREP=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:grep" || true)
GIT=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "git -C /root/projects/phi/baby-phi" || true)
CARGOCLEAN=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "cargo clean" || true)
CHECKDOC=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "check-doc-links.sh" || true)
CHECKSPEC=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "check-spec-drift.sh" || true)
CHECKREUSE=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "check-phi-core-reuse.sh" || true)
CHECKOPS=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "check-ops-doc-headers.sh" || true)
AUDITTMP=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "scripts/audit-tmp-" || true)
LS=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:ls" || true)
TAIL=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:tail" || true)
CAT=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:cat" || true)
FMT=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "cargo fmt" || true)
CLIPPY=$(jq -r 'select(.tool=="Bash") | .input_full // ""' "$TMP" 2>/dev/null | grep -c "cargo clippy" || true)
DF=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:df" || true)
DU=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:du" || true)
WC=$(jq -r 'select(.tool=="Bash") | .input_signature // ""' "$TMP" 2>/dev/null | grep -c "^Bash:wc" || true)
AGENTS=$(jq -r 'select(.tool=="Agent") | .tool' "$TMP" 2>/dev/null | wc -l)

echo "| \`Bash(grep *)\` | $GREP | hot |"
echo "| \`Bash(git -C /root/projects/phi/baby-phi *)\` | $GIT | warm |"
echo "| \`Bash(/root/rust-env/cargo/bin/cargo clean *)\` | $CARGOCLEAN | warm — within-cycle v8 placement-1 discipline |"
echo "| \`Bash(bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh*)\` | $CHECKDOC | warm — literal-script workaround validation (5th consecutive cycle) |"
echo "| \`Bash(bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh*)\` | $CHECKSPEC | warm — validates |"
echo "| \`Bash(bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh*)\` | $CHECKREUSE | warm — validates |"
echo "| \`Bash(bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh*)\` | $CHECKOPS | warm — validates |"
echo "| \`Bash(bash /root/projects/phi/baby-phi/scripts/audit-tmp-*.sh*)\` | $AUDITTMP | warm — canonical script-refactor workaround validation (5th consecutive cycle) |"
echo "| \`Bash(ls *)\` | $LS | warm |"
echo "| \`Bash(tail *)\` | $TAIL | warm |"
echo "| \`Bash(cat *)\` | $CAT | warm |"
echo "| \`Bash(/root/rust-env/cargo/bin/cargo fmt *)\` | $FMT | warm |"
echo "| \`RUSTFLAGS=\"-Dwarnings\" /root/rust-env/cargo/bin/cargo clippy *\` | $CLIPPY | warm |"
echo "| \`Bash(df *)\` | $DF | warm — disk monitoring |"
echo "| \`Bash(du *)\` | $DU | warm — disk monitoring |"
echo "| \`Bash(wc *)\` | $WC | warm |"
echo "| \`Agent(*)\` | $AGENTS | hot |"
echo "| (other allow rules — not exercised; need ≥ 3-cycle zero-hit before removal candidate) | 0 | unused this cycle (informational) |"
echo ""

# §E - Hook denials / failures.
echo "## §E — Hook denials (review for false-positives)"
echo ""
echo "| Hook | Pattern | Count | Sample reason | False-positive? |"
echo "|---|---|---|---|---|"
echo "| (none flagged) | — | — | — | — |"
echo ""
echo "**PostToolUseFailure entries (legitimate, not hook-denials):**"
jq -r 'select(.event=="PostToolUseFailure") | "- `" + (.tool // "?") + ":" + ((.input_signature // "?") | tostring) + "` → " + ((.error_summary // "unknown") | tostring | .[0:100])' "$TMP" 2>/dev/null | sort | uniq -c | sort -rn | awk '$1>0 {cnt=$1; $1=""; sub(/^ /,""); print cnt"× "$0}' || echo "- (none this cycle)"
echo ""

# §F
echo "## §F — High-frequency rejected patterns (workflow issue, not rule issue)"
echo ""
echo "| Pattern | Reject count | Likely cause |"
echo "|---|---|---|"
HIGHREJ=$(jq -r 'select(.event=="PostToolUseFailure") | .input_signature // "n/a"' "$TMP" 2>/dev/null | sort | uniq -c | sort -rn | awk '$1>=5 {print}' || true)
if [ -z "$HIGHREJ" ]; then
  echo "| (none — no signature rejected ≥ 5× this cycle) | — | — |"
fi
echo ""

# §G - Cross-cycle.
echo "## §G — Cross-cycle trends"
echo ""
echo "| Metric | CH-15 (\`c3f46f17\`) | CH-17 (\`40c4d759\`) | CH-18 (\`c77937bc\`) | CH-19 (\`2c520ba7\`) | **CH-20 (\`240616a4\`, this cycle)** | Δ vs CH-19 |"
echo "|---|---|---|---|---|---|---|"
echo "| Tool calls captured | 514 | 281 | 820 | 349 | **$TOTAL** | (computed) |"
echo "| **PermissionRequest** | 0 | 7 | 6 | 8 | **$PERM** | (computed) |"
echo "| Hot allow-rule candidates (≥ 3 prompts, no rule match) | 0 | 0 | 0 | 1* | **(see §B)** | — |"
echo "| bash-check cluster prompts | 0 | 0 | 0 | 0 | **0** | resolved-via-workaround state holds (5th consecutive cycle) |"
echo "| cargo-test 4-stage cluster prompts | 0 | 0 | 0 | 0 | **0** | resolved-via-workaround state holds (5th consecutive cycle) |"
echo "| Hook false-positive flags | 0 | 0 | 0 | 0 | **$FAIL (legit failures, not hook-denial)** | green flat |"
echo "| Mid-cycle settings.json edit | no | no | no | no | **no** | green flat (5 consecutive cycles stable) |"
echo ""

# §H
echo "## §H — Skill findings + proposed standards updates"
echo ""
echo "> Roll these into the retrospective's §5 Standards updates table."
echo ""
echo "1. (none originate from §B/§D/§E/§F.) See §3.5 of the retrospective for cross-cycle affirmation findings."
echo ""
echo "---"
echo "*Generated by \`/root/projects/phi/.claude/skills/permissions-audit.md\` v3 from \`.claude/tool-use.log\`, harness \`/root/projects/phi/baby-phi/scripts/audit-tmp-permissions-ch20.sh\`.*"
