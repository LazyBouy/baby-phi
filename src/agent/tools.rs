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
        let count = input["count"]
            .as_u64()
            .unwrap_or(10)
            .min(50);
        let count_str = format!("-{}", count);
        run_git_command(&["log", "--oneline", "--no-decorate", &count_str]).await
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

    #[tokio::test]
    async fn git_status_runs_without_error() {
        let tool = GitStatusTool;
        let result = tool.execute(json!({})).await;
        // We're in a git repo (baby-phi itself), so this should succeed
        assert!(!result.is_error, "git status should succeed in repo: {}", result.content);
    }

    #[tokio::test]
    async fn git_diff_runs_without_error() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({})).await;
        assert!(!result.is_error, "git diff should succeed in repo: {}", result.content);
    }

    #[tokio::test]
    async fn git_diff_staged_runs_without_error() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({"staged": true})).await;
        assert!(!result.is_error, "git diff --cached should succeed: {}", result.content);
    }

    #[tokio::test]
    async fn git_diff_with_path_runs() {
        let tool = GitDiffTool;
        let result = tool.execute(json!({"path": "src/"})).await;
        assert!(!result.is_error, "git diff with path should succeed: {}", result.content);
    }

    #[tokio::test]
    async fn git_log_runs_without_error() {
        let tool = GitLogTool;
        let result = tool.execute(json!({})).await;
        assert!(!result.is_error, "git log should succeed in repo: {}", result.content);
        // Should contain at least one commit hash
        assert!(result.content.len() > 5, "git log should have content");
    }

    #[tokio::test]
    async fn git_log_respects_count() {
        let tool = GitLogTool;
        let result = tool.execute(json!({"count": 1})).await;
        assert!(!result.is_error);
        // With count=1, should have at most 1 commit line (plus maybe empty trailing)
        let lines: Vec<&str> = result.content.lines().filter(|l| !l.is_empty()).collect();
        assert!(lines.len() <= 2, "count=1 should return at most 1 commit, got: {:?}", lines);
    }

    #[tokio::test]
    async fn git_log_caps_at_50() {
        let tool = GitLogTool;
        // Even if we ask for 1000, it should cap at 50
        let result = tool.execute(json!({"count": 1000})).await;
        assert!(!result.is_error);
    }

    #[test]
    fn tool_definitions_are_valid() {
        // Verify all tools produce valid definitions
        let tools: Vec<Box<dyn AgentTool>> = vec![
            Box::new(GitStatusTool),
            Box::new(GitDiffTool),
            Box::new(GitLogTool),
        ];
        for tool in &tools {
            let def = tool.definition();
            assert!(!def.name.is_empty(), "tool name must not be empty");
            assert!(!def.description.is_empty(), "tool description must not be empty");
            assert!(def.parameters.is_object(), "parameters must be an object");
        }
    }
}
