// agent/context.rs — auto-injected system prompt context
//
// Builds the extra_context() string injected into the system prompt every run.
// Pulls in communication skill rules and key architectural facts from LEARNINGS.md
// so baby-phi follows its own rules without manually re-reading files each session.

use std::fs;

/// Reads a file, returning empty string on any error.
fn read_opt(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Extracts the first N lines of a section from a markdown file.
/// Looks for a heading that contains `section_keyword` and returns
/// up to `max_lines` of content following it (stopping at next heading).
fn extract_section(text: &str, section_keyword: &str, max_lines: usize) -> Option<String> {
    let mut in_section = false;
    let mut lines_collected = 0;
    let mut result = Vec::new();

    for line in text.lines() {
        if line.starts_with('#') {
            if in_section {
                break; // hit the next section heading
            }
            if line.to_lowercase().contains(&section_keyword.to_lowercase()) {
                in_section = true;
            }
            continue;
        }
        if in_section {
            if lines_collected >= max_lines {
                break;
            }
            result.push(line);
            lines_collected += 1;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result.join("\n"))
    }
}

/// Builds the extra system-prompt context injected every run.
/// Kept intentionally compact — this burns tokens every turn.
pub fn build_extra_context() -> String {
    let mut parts: Vec<String> = Vec::new();

    // ── Communication skill ───────────────────────────────────────────────────
    let skill = read_opt("skills/communicate/SKILL.md");
    if !skill.is_empty() {
        // Extract just the rules section to keep it compact
        if let Some(rules) = extract_section(&skill, "Rules", 20) {
            parts.push(format!(
                "## Active Skills: Communication\n\
                Apply these rules to every journal entry you write:\n\
                {rules}"
            ));
        }
    }

    // ── Key architectural facts ───────────────────────────────────────────────
    let learnings = read_opt("LEARNINGS.md");
    if !learnings.is_empty() {
        // Extract the "Extending the Agent" section — most actionable facts
        if let Some(extending) = extract_section(&learnings, "Extending the Agent", 15) {
            parts.push(format!(
                "## Architectural Facts (from LEARNINGS.md)\n{extending}"
            ));
        }
    }

    // ── Tool-use guidance (helps weaker/local models follow tool syntax) ──────
    parts.push(
        "## Tool Use Instructions\n\
        When you need information or want to take an action, call a tool immediately — \
        do not describe what you would do, just do it. \
        Always use the exact tool name and provide all required parameters as a JSON object. \
        After a tool returns, use the result to decide your next step. \
        If a tool fails, read the error message and retry with corrected parameters. \
        Do not ask the user for permission before using tools — act autonomously. \
        Complete the full task before writing a journal entry.\n\n\
        When making function calls using tools that accept array or object parameters \
        ensure those are structured using JSON. For example:\n\
        ```\n\
        {\"parameter\": [{\"color\": \"orange\", \"options\": {\"opt\": true}}]}\n\
        ```\n\n\
        If you intend to call multiple tools and there are no dependencies between the calls, \
        make all of the independent calls in the same turn, otherwise you MUST wait for \
        previous calls to finish first to determine the dependent values (do NOT use \
        placeholders or guess missing parameters).\n\n\
        If the user provides a specific value for a parameter (for example provided in quotes), \
        make sure to use that value EXACTLY. DO NOT make up values for or ask about optional \
        parameters."
        .to_string(),
    );

    parts.join("\n\n")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_section_finds_matching_heading() {
        let text = "# Intro\nsome intro\n## Rules\nrule 1\nrule 2\n## Other\nother stuff";
        let result = extract_section(text, "Rules", 10);
        assert!(result.is_some());
        let content = result.unwrap();
        assert!(content.contains("rule 1"));
        assert!(content.contains("rule 2"));
        assert!(!content.contains("other stuff"));
    }

    #[test]
    fn extract_section_returns_none_when_missing() {
        let text = "# Intro\nsome intro\n## Other\nother stuff";
        let result = extract_section(text, "Rules", 10);
        assert!(result.is_none());
    }

    #[test]
    fn extract_section_respects_max_lines() {
        let text = "## Rules\nline 1\nline 2\nline 3\nline 4\nline 5";
        let result = extract_section(text, "Rules", 3);
        assert!(result.is_some());
        let content = result.unwrap();
        let line_count = content.lines().count();
        assert!(line_count <= 3, "expected ≤ 3 lines, got {line_count}");
    }

    #[test]
    fn extract_section_case_insensitive() {
        let text = "## RULES\nrule 1\nrule 2";
        let result = extract_section(text, "rules", 10);
        assert!(result.is_some());
    }

    #[test]
    fn build_extra_context_returns_string() {
        // Just verify it doesn't panic — files may not exist in test env
        let ctx = build_extra_context();
        // Either has content (if files exist) or is empty — both are valid
        let _ = ctx; // no assertion on content, just no panic
    }

    #[test]
    fn build_extra_context_is_compact() {
        // The injected context should not be huge — it's added every turn
        let ctx = build_extra_context();
        let char_count = ctx.len();
        // If both files exist and are read, should be under 2000 chars
        // (can't assert exact value since files vary, but panic if absurdly large)
        assert!(
            char_count < 5000,
            "extra_context too large ({char_count} chars) — will waste tokens every turn"
        );
    }

    #[test]
    fn read_opt_returns_empty_for_missing_file() {
        let result = read_opt("/nonexistent/path/that/cannot/exist/ever.txt");
        assert_eq!(result, "");
    }
}
