// agent/tools.rs — Custom tools developed by baby-phi
//
// Each tool implements the AgentTool trait from core::kernel.
// Tools are registered in extra_tools() in mod.rs.

use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;

use crate::core::{AgentTool, ToolResult};

// ── Git Status Tool ───────────────────────────────────────────────────────────

/// Shows `git status --short` — quick overview of uncommitted changes.
pub struct GitStatusTool;

#[async_trait]
impl AgentTool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }
    fn description(&self) -> &str {
        "Show git working tree status (short format). No parameters needed."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }
    async fn execute(&self, _input: Value) -> ToolResult {
        run_git_command(&["status", "--short"]).await
    }
}

// ── Git Diff Tool ─────────────────────────────────────────────────────────────

/// Shows `git diff` — detailed view of uncommitted changes.
/// Optionally takes a `path` parameter to scope the diff.
pub struct GitDiffTool;

#[async_trait]
impl AgentTool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }
    fn description(&self) -> &str {
        "Show git diff of uncommitted changes. Optional 'staged' (bool) and 'path' (string) params."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "staged": { "type": "boolean", "description": "If true, show staged changes (--cached)" },
                "path": { "type": "string", "description": "Limit diff to this path" }
            }
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let mut args = vec!["diff"];
        if input["staged"].as_bool().unwrap_or(false) {
            args.push("--cached");
        }
        let path_str;
        if let Some(p) = input["path"].as_str() {
            args.push("--");
            path_str = p.to_string();
            args.push(&path_str);
        }
        run_git_command(&args).await
    }
}

// ── Git Log Tool ──────────────────────────────────────────────────────────────

/// Shows recent git log in compact format.
pub struct GitLogTool;

#[async_trait]
impl AgentTool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }
    fn description(&self) -> &str {
        "Show recent git commits. Optional 'count' param (default 10, max 50)."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "count": { "type": "integer", "description": "Number of commits to show (default 10, max 50)" }
            }
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let count = input["count"].as_u64().unwrap_or(10).min(50);
        let count_str = format!("-{}", count);
        run_git_command(&["log", "--oneline", "--no-decorate", &count_str]).await
    }
}

// ── Working Memory Tool ───────────────────────────────────────────────────────

/// Persists a working-memory note so context can be reconstructed cheaply
/// across turns. Writes to /tmp/working_memory.md (survives within a run,
/// gone after reboot — intentionally ephemeral).
///
/// Use this when turn count exceeds ~15 to avoid losing track of what you
/// set out to do. Call with action="write" to save, action="read" to restore.
pub struct WorkingMemoryTool;

const WORKING_MEMORY_PATH: &str = "/tmp/baby_phi_working_memory.md";

#[async_trait]
impl AgentTool for WorkingMemoryTool {
    fn name(&self) -> &str {
        "working_memory"
    }
    fn description(&self) -> &str {
        "Read or write a working-memory note that persists across turns within a run. \
         Use action='write' with content= to save state (goal, progress, key facts). \
         Use action='read' to restore. Write when turn count > 15 to avoid context loss."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read", "write"],
                    "description": "Whether to read or write the working memory note"
                },
                "content": {
                    "type": "string",
                    "description": "The note to write (required when action='write'). Include: goal, done so far, what's left, key facts."
                }
            },
            "required": ["action"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let action = match input["action"].as_str() {
            Some(a) => a,
            None => {
                return ToolResult {
                    content: "missing 'action' (must be 'read' or 'write')".into(),
                    is_error: true,
                }
            }
        };

        match action {
            "read" => match tokio::fs::read_to_string(WORKING_MEMORY_PATH).await {
                Ok(text) if !text.is_empty() => ToolResult {
                    content: text,
                    is_error: false,
                },
                Ok(_) => ToolResult {
                    content: "(working memory is empty)".into(),
                    is_error: false,
                },
                Err(_) => ToolResult {
                    content: "(no working memory found — this is the start of a run)".into(),
                    is_error: false,
                },
            },
            "write" => {
                let content = match input["content"].as_str() {
                    Some(c) => c,
                    None => {
                        return ToolResult {
                            content: "missing 'content' for action='write'".into(),
                            is_error: true,
                        }
                    }
                };
                match tokio::fs::write(WORKING_MEMORY_PATH, content).await {
                    Ok(_) => ToolResult {
                        content: format!("working memory saved ({} bytes)", content.len()),
                        is_error: false,
                    },
                    Err(e) => ToolResult {
                        content: format!("failed to write working memory: {e}"),
                        is_error: true,
                    },
                }
            }
            other => ToolResult {
                content: format!("unknown action '{other}' — must be 'read' or 'write'"),
                is_error: true,
            },
        }
    }
}

// ── Read File Range Tool ──────────────────────────────────────────────────────

/// Read a specific range of lines from a file — frugal alternative to read_file.
/// Instead of loading a 500-line file to find one function, use this with
/// offset + limit to read only what you need.
pub struct ReadFileRangeTool;

#[async_trait]
impl AgentTool for ReadFileRangeTool {
    fn name(&self) -> &str {
        "read_file_range"
    }
    fn description(&self) -> &str {
        "Read a specific range of lines from a file. Use instead of read_file when you \
         only need part of a large file. 'offset' is the 0-based starting line (default 0), \
         'limit' is max lines to return (default 50, max 200)."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":   { "type": "string",  "description": "File path to read" },
                "offset": { "type": "integer", "description": "0-based line offset to start reading (default 0)" },
                "limit":  { "type": "integer", "description": "Max lines to return (default 50, max 200)" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let path = match input["path"].as_str() {
            Some(p) => p,
            None => {
                return ToolResult {
                    content: "missing 'path'".into(),
                    is_error: true,
                }
            }
        };

        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let limit = (input["limit"].as_u64().unwrap_or(50) as usize).min(200);

        // Check file exists and isn't too large
        match tokio::fs::metadata(path).await {
            Err(e) => {
                return ToolResult {
                    content: format!("not found: {e}"),
                    is_error: true,
                }
            }
            Ok(m) if m.len() > 10_485_760 => {
                // 10MB limit (more generous than read_file since we're chunking)
                return ToolResult {
                    content: format!("file too large to read ({} bytes)", m.len()),
                    is_error: true,
                }
            }
            _ => {}
        }

        let bytes = match tokio::fs::read(path).await {
            Ok(b) => b,
            Err(e) => {
                return ToolResult {
                    content: format!("read error: {e}"),
                    is_error: true,
                }
            }
        };

        if bytes.contains(&0u8) {
            return ToolResult {
                content: "binary file".into(),
                is_error: true,
            };
        }

        let text = String::from_utf8_lossy(&bytes);
        let total_lines = text.lines().count();

        let selected: Vec<&str> = text.lines().skip(offset).take(limit).collect();
        let returned = selected.len();
        let mut content = selected.join("\n");

        // Append a summary so the agent knows context within the file
        content.push_str(&format!(
            "\n\n[lines {}-{} of {} total]",
            offset,
            offset + returned,
            total_lines
        ));

        ToolResult {
            content,
            is_error: false,
        }
    }
}

// ── Shared Helper ─────────────────────────────────────────────────────────────

async fn run_git_command(args: &[&str]) -> ToolResult {
    match tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new("git")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    {
        Err(_) => ToolResult {
            content: "git command timed out (10s)".into(),
            is_error: true,
        },
        Ok(Err(e)) => ToolResult {
            content: format!("git not found or exec error: {e}"),
            is_error: true,
        },
        Ok(Ok(out)) => {
            let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.is_empty() {
                if !combined.is_empty() {
                    combined.push('\n');
                }
                combined.push_str(&stderr);
            }
            if combined.is_empty() {
                combined = "(no output — working tree clean or no changes)".into();
            }
            if combined.len() > 30_000 {
                combined.truncate(30_000);
                combined.push_str("\n[truncated]");
            }
            ToolResult {
                content: combined,
                is_error: !out.status.success(),
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Git tool tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn git_status_runs_without_error() {
        let tool = GitStatusTool;
        let result = tool.execute(json!({})).await;
        assert!(
            !result.is_error,
            "git status should succeed in repo: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn git_diff_runs_without_error() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({})).await;
        assert!(
            !result.is_error,
            "git diff should succeed in repo: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn git_diff_staged_runs_without_error() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({"staged": true})).await;
        assert!(
            !result.is_error,
            "git diff --cached should succeed: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn git_diff_with_path_runs() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({"path": "src/"})).await;
        assert!(
            !result.is_error,
            "git diff with path should succeed: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn git_log_runs_without_error() {
        let tool = GitLogTool;
        let result = tool.execute(json!({})).await;
        assert!(
            !result.is_error,
            "git log should succeed in repo: {}",
            result.content
        );
        assert!(result.content.len() > 5, "git log should have content");
    }

    #[tokio::test]
    async fn git_log_respects_count() {
        let tool = GitLogTool;
        let result = tool.execute(json!({"count": 1})).await;
        assert!(!result.is_error);
        let lines: Vec<&str> = result.content.lines().filter(|l| !l.is_empty()).collect();
        assert!(
            lines.len() <= 2,
            "count=1 should return at most 1 commit, got: {:?}",
            lines
        );
    }

    #[tokio::test]
    async fn git_log_caps_at_50() {
        let tool = GitLogTool;
        let result = tool.execute(json!({"count": 1000})).await;
        assert!(!result.is_error);
    }

    // ── WorkingMemoryTool tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn working_memory_write_and_read_roundtrip() {
        let tool = WorkingMemoryTool;
        let note = "Goal: add working memory tool\nDone: wrote the struct\nLeft: tests";

        // Write
        let write_result = tool
            .execute(json!({"action": "write", "content": note}))
            .await;
        assert!(
            !write_result.is_error,
            "write should succeed: {}",
            write_result.content
        );
        assert!(
            write_result.content.contains("saved"),
            "write should confirm save"
        );

        // Read back
        let read_result = tool.execute(json!({"action": "read"})).await;
        assert!(
            !read_result.is_error,
            "read should succeed: {}",
            read_result.content
        );
        assert_eq!(
            read_result.content, note,
            "read should return exactly what was written"
        );
    }

    #[tokio::test]
    async fn working_memory_read_when_empty() {
        // Clean slate: delete the file first if it exists
        let _ = tokio::fs::remove_file("/tmp/baby_phi_working_memory.md").await;

        let tool = WorkingMemoryTool;
        let result = tool.execute(json!({"action": "read"})).await;
        assert!(
            !result.is_error,
            "read on missing file should not error: {}",
            result.content
        );
        assert!(
            result.content.contains("no working memory"),
            "should report no memory: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn working_memory_missing_action() {
        let tool = WorkingMemoryTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_error, "missing action must be an error");
    }

    #[tokio::test]
    async fn working_memory_unknown_action() {
        let tool = WorkingMemoryTool;
        let result = tool.execute(json!({"action": "delete"})).await;
        assert!(result.is_error, "unknown action must be an error");
    }

    #[tokio::test]
    async fn working_memory_write_without_content() {
        let tool = WorkingMemoryTool;
        let result = tool.execute(json!({"action": "write"})).await;
        assert!(result.is_error, "write without content must be an error");
    }

    // ── ReadFileRangeTool tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn read_file_range_basic() {
        // Use Cargo.toml as a known file
        let tool = ReadFileRangeTool;
        let result = tool
            .execute(json!({"path": "Cargo.toml", "offset": 0, "limit": 5}))
            .await;
        assert!(
            !result.is_error,
            "should read Cargo.toml: {}",
            result.content
        );
        assert!(
            result.content.contains("[package]"),
            "first lines should contain [package]: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_file_range_with_offset() {
        let tool = ReadFileRangeTool;
        // Read from line 0 (first 3 lines)
        let result_start = tool
            .execute(json!({"path": "Cargo.toml", "offset": 0, "limit": 3}))
            .await;
        // Read from line 3 (next chunk)
        let result_offset = tool
            .execute(json!({"path": "Cargo.toml", "offset": 3, "limit": 3}))
            .await;
        assert!(!result_start.is_error);
        assert!(!result_offset.is_error);
        // They should be different content
        let start_lines: Vec<&str> = result_start
            .content
            .lines()
            .filter(|l| !l.starts_with('['))
            .collect();
        let offset_lines: Vec<&str> = result_offset
            .content
            .lines()
            .filter(|l| !l.starts_with('['))
            .collect();
        // The two chunks shouldn't be identical (they read different lines)
        assert_ne!(
            start_lines.join("\n"),
            offset_lines.join("\n"),
            "offset should return different content"
        );
    }

    #[tokio::test]
    async fn read_file_range_caps_limit() {
        let tool = ReadFileRangeTool;
        // Ask for 9999 lines — should be capped at 200 content lines.
        // The result also includes a "[lines X-Y of Z total]" summary at the end.
        let result = tool
            .execute(json!({"path": "src/agent/tools.rs", "offset": 0, "limit": 9999}))
            .await;
        assert!(!result.is_error);
        // Strip the trailing summary line (starts with "[lines") and blank lines before counting
        let content_lines: Vec<&str> = result
            .content
            .lines()
            .filter(|l| !l.trim().starts_with("[lines ") && !l.trim().is_empty())
            .collect();
        assert!(
            content_lines.len() <= 200,
            "limit cap at 200 content lines, got {}",
            content_lines.len()
        );
    }

    #[tokio::test]
    async fn read_file_range_missing_file() {
        let tool = ReadFileRangeTool;
        let result = tool
            .execute(json!({"path": "/nonexistent/file.txt"}))
            .await;
        assert!(result.is_error, "missing file should error");
    }

    #[tokio::test]
    async fn read_file_range_reports_line_range() {
        let tool = ReadFileRangeTool;
        let result = tool
            .execute(json!({"path": "Cargo.toml", "offset": 0, "limit": 5}))
            .await;
        assert!(!result.is_error);
        assert!(
            result.content.contains("[lines "),
            "should include line range summary: {}",
            result.content
        );
    }

    // ── Definition validity (all tools) ──────────────────────────────────────

    #[test]
    fn tool_definitions_are_valid() {
        let tools: Vec<Box<dyn AgentTool>> = vec![
            Box::new(GitStatusTool),
            Box::new(GitDiffTool),
            Box::new(GitLogTool),
            Box::new(WorkingMemoryTool),
            Box::new(ReadFileRangeTool),
        ];
        for tool in &tools {
            let def = tool.definition();
            assert!(!def.name.is_empty(), "tool name must not be empty");
            assert!(
                !def.description.is_empty(),
                "tool description must not be empty"
            );
            assert!(def.parameters.is_object(), "parameters must be an object");
        }
    }
}
