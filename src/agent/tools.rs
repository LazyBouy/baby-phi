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
                };
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

// ── Project Info Tool ─────────────────────────────────────────────────────────

/// Gives a structured one-screen summary of the current project.
/// Auto-detects language from manifest files, counts source files and tests,
/// lists top-level dependencies. Helps orient quickly to an unfamiliar codebase.
pub struct ProjectInfoTool;

#[async_trait]
impl AgentTool for ProjectInfoTool {
    fn name(&self) -> &str {
        "project_info"
    }
    fn description(&self) -> &str {
        "Scan the current directory and return a structured project summary: \
         language, name, version, entry point, source file count, test count, \
         and key dependencies. Use this at the start of a session on an unfamiliar \
         codebase instead of manually running ls + cat Cargo.toml + find."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Root directory to scan (default: current directory '.')"
                }
            }
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let root = input["path"].as_str().unwrap_or(".");

        match scan_project(root).await {
            Ok(info) => ToolResult {
                content: info,
                is_error: false,
            },
            Err(e) => ToolResult {
                content: format!("project scan failed: {e}"),
                is_error: true,
            },
        }
    }
}

/// Scans the project root and returns a formatted summary string.
async fn scan_project(root: &str) -> Result<String, String> {
    let mut lines: Vec<String> = Vec::new();

    // ── Detect project type from manifest files ───────────────────────────────
    let candidates = [
        ("Cargo.toml", "Rust"),
        ("package.json", "Node.js/JavaScript"),
        ("pyproject.toml", "Python"),
        ("setup.py", "Python"),
        ("go.mod", "Go"),
        ("pom.xml", "Java/Maven"),
        ("build.gradle", "Java/Gradle"),
        ("Gemfile", "Ruby"),
        ("composer.json", "PHP"),
        ("mix.exs", "Elixir"),
    ];

    let mut lang = "Unknown";
    let mut manifest_path = String::new();
    for (file, language) in &candidates {
        let full = format!("{root}/{file}");
        if tokio::fs::metadata(&full).await.is_ok() {
            lang = language;
            manifest_path = full;
            break;
        }
    }

    lines.push(format!("Language:  {lang}"));

    // ── Parse manifest for name/version/deps ─────────────────────────────────
    if !manifest_path.is_empty() {
        lines.push(format!("Manifest:  {manifest_path}"));

        if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
            // Rust: Cargo.toml
            if manifest_path.ends_with("Cargo.toml") {
                if let Some(name) = extract_toml_field(&content, "name") {
                    lines.push(format!("Name:      {name}"));
                }
                if let Some(version) = extract_toml_field(&content, "version") {
                    lines.push(format!("Version:   {version}"));
                }
                if let Some(edition) = extract_toml_field(&content, "edition") {
                    lines.push(format!("Edition:   {edition}"));
                }
                // Parse dependencies section
                let deps = extract_toml_section_keys(&content, "[dependencies]");
                if !deps.is_empty() {
                    lines.push(format!("Deps:      {}", deps.join(", ")));
                }
            }
            // Node.js: package.json
            else if manifest_path.ends_with("package.json") {
                if let Ok(pkg) = serde_json::from_str::<Value>(&content) {
                    if let Some(name) = pkg["name"].as_str() {
                        lines.push(format!("Name:      {name}"));
                    }
                    if let Some(version) = pkg["version"].as_str() {
                        lines.push(format!("Version:   {version}"));
                    }
                    if let Some(obj) = pkg["dependencies"].as_object() {
                        let dep_names: Vec<&str> =
                            obj.keys().map(|k| k.as_str()).take(10).collect();
                        if !dep_names.is_empty() {
                            lines.push(format!("Deps:      {}", dep_names.join(", ")));
                        }
                    }
                    if let Some(scripts) = pkg["scripts"].as_object() {
                        let script_names: Vec<&str> =
                            scripts.keys().map(|k| k.as_str()).take(8).collect();
                        if !script_names.is_empty() {
                            lines.push(format!("Scripts:   {}", script_names.join(", ")));
                        }
                    }
                }
            }
            // Python: pyproject.toml
            else if manifest_path.ends_with("pyproject.toml") {
                if let Some(name) = extract_toml_field(&content, "name") {
                    lines.push(format!("Name:      {name}"));
                }
                if let Some(version) = extract_toml_field(&content, "version") {
                    lines.push(format!("Version:   {version}"));
                }
            }
            // Go: go.mod
            else if manifest_path.ends_with("go.mod") {
                for line in content.lines().take(5) {
                    if line.starts_with("module ") {
                        lines.push(format!("Module:    {}", &line[7..]));
                    }
                    if line.starts_with("go ") {
                        lines.push(format!("Go:        {}", &line[3..]));
                    }
                }
            }
        }
    }

    // ── Entry point detection ─────────────────────────────────────────────────
    let entry_candidates = [
        "src/main.rs",
        "src/lib.rs",
        "main.go",
        "main.py",
        "index.js",
        "index.ts",
        "app.py",
        "app.js",
        "lib/main.dart",
    ];
    let mut entry_points: Vec<String> = Vec::new();
    for candidate in &entry_candidates {
        let full = format!("{root}/{candidate}");
        if tokio::fs::metadata(&full).await.is_ok() {
            entry_points.push(candidate.to_string());
        }
    }
    if !entry_points.is_empty() {
        lines.push(format!("Entry:     {}", entry_points.join(", ")));
    }

    // ── Count source files ────────────────────────────────────────────────────
    let counts = count_source_files(root).await;
    if let Some((src_count, test_count)) = counts {
        lines.push(format!("Src files: {src_count}"));
        if test_count > 0 {
            lines.push(format!("Tests:     ~{test_count} test functions"));
        }
    }

    // ── Git info ──────────────────────────────────────────────────────────────
    let git_info = tokio::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(root)
        .output()
        .await;
    if let Ok(out) = git_info {
        let last_commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !last_commit.is_empty() {
            lines.push(format!("Last commit: {last_commit}"));
        }
    }

    // ── README presence ───────────────────────────────────────────────────────
    for readme in &["README.md", "README.txt", "README"] {
        if tokio::fs::metadata(&format!("{root}/{readme}")).await.is_ok() {
            lines.push(format!("README:    {readme}"));
            break;
        }
    }

    if lines.is_empty() {
        return Err(format!("no project files found in '{root}'"));
    }

    Ok(lines.join("\n"))
}

/// Simple TOML key=value extractor (no full parser needed — just reads `key = "value"` lines).
fn extract_toml_field(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(key) {
            if let Some(rest) = trimmed.strip_prefix(key) {
                let rest = rest.trim();
                if rest.starts_with('=') {
                    let val = rest[1..].trim().trim_matches('"').to_string();
                    if !val.is_empty() {
                        return Some(val);
                    }
                }
            }
        }
    }
    None
}

/// Extracts the keys from a TOML section (e.g. `[dependencies]`).
/// Returns keys up to 12 items to keep output compact.
fn extract_toml_section_keys(content: &str, section_header: &str) -> Vec<String> {
    let mut in_section = false;
    let mut keys = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section_header {
            in_section = true;
            continue;
        }
        if in_section {
            // A new section starts
            if trimmed.starts_with('[') {
                break;
            }
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            // Extract key from `key = ...` or `key.feature = ...`
            if let Some(key) = trimmed.split('=').next() {
                let k = key.trim().to_string();
                if !k.is_empty() && keys.len() < 12 {
                    keys.push(k);
                }
            }
        }
    }

    keys
}

/// Count source files by extension and grep for test functions.
async fn count_source_files(root: &str) -> Option<(usize, usize)> {
    // Use find to count files with source extensions
    let extensions = ["rs", "go", "py", "js", "ts", "java", "rb", "ex", "exs"];

    let mut count_args = vec![
        root.to_string(),
        "-type".to_string(),
        "f".to_string(),
        "(".to_string(),
    ];
    for (i, ext) in extensions.iter().enumerate() {
        if i > 0 {
            count_args.push("-o".to_string());
        }
        count_args.push("-name".to_string());
        count_args.push(format!("*.{ext}"));
    }
    count_args.push(")".to_string());
    // Exclude common non-source dirs
    let full_args: Vec<&str> = count_args.iter().map(|s| s.as_str()).collect();

    let find_out = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("find")
            .args(&full_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await;

    let src_count = match find_out {
        Ok(Ok(out)) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.contains("/target/") && !l.contains("/node_modules/"))
            .count(),
        _ => return None,
    };

    // Count test functions via grep
    let grep_out = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("grep")
            .args(["-r", "--include=*.rs", "#[test]", root])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await;

    let test_count = match grep_out {
        Ok(Ok(out)) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.contains("/target/"))
            .count(),
        _ => 0,
    };

    Some((src_count, test_count))
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
        let result = tool.execute(json!({"path": "/nonexistent/file.txt"})).await;
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
            Box::new(ProjectInfoTool),
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

    // ── ProjectInfoTool tests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn project_info_detects_rust_project() {
        // This repo is a Rust project — Cargo.toml is present
        let tool = ProjectInfoTool;
        let result = tool.execute(json!({})).await;
        assert!(
            !result.is_error,
            "project_info should succeed: {}",
            result.content
        );
        assert!(
            result.content.contains("Rust"),
            "should detect Rust language: {}",
            result.content
        );
        assert!(
            result.content.contains("baby-phi"),
            "should find project name: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn project_info_reports_dependencies() {
        let tool = ProjectInfoTool;
        let result = tool.execute(json!({})).await;
        assert!(!result.is_error);
        // Cargo.toml has tokio, reqwest, serde, serde_json, toml, async-trait
        assert!(
            result.content.contains("tokio"),
            "should list tokio dependency: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn project_info_reports_entry_point() {
        let tool = ProjectInfoTool;
        let result = tool.execute(json!({})).await;
        assert!(!result.is_error);
        // src/main.rs exists
        assert!(
            result.content.contains("main.rs"),
            "should find src/main.rs: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn project_info_counts_source_files() {
        let tool = ProjectInfoTool;
        let result = tool.execute(json!({})).await;
        assert!(!result.is_error);
        assert!(
            result.content.contains("Src files:"),
            "should report source file count: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn project_info_with_explicit_path() {
        let tool = ProjectInfoTool;
        let result = tool.execute(json!({"path": "."})).await;
        assert!(!result.is_error, "explicit '.' path should work");
    }

    #[tokio::test]
    async fn project_info_on_nonexistent_path() {
        let tool = ProjectInfoTool;
        let result = tool
            .execute(json!({"path": "/tmp/definitely_not_a_project_12345"}))
            .await;
        // Should error gracefully — no project files found
        assert!(
            result.is_error || result.content.contains("Unknown"),
            "nonexistent path should report unknown or error: {}",
            result.content
        );
    }

    #[test]
    fn extract_toml_field_finds_name() {
        let content = r#"[package]
name = "my-project"
version = "1.2.3"
"#;
        assert_eq!(extract_toml_field(content, "name"), Some("my-project".into()));
        assert_eq!(extract_toml_field(content, "version"), Some("1.2.3".into()));
        assert_eq!(extract_toml_field(content, "missing"), None);
    }

    #[test]
    fn extract_toml_section_keys_finds_deps() {
        let content = r#"[package]
name = "test"

[dependencies]
tokio = "1"
serde = { version = "1", features = ["derive"] }
reqwest = "0.12"

[dev-dependencies]
tempfile = "3"
"#;
        let keys = extract_toml_section_keys(content, "[dependencies]");
        assert!(keys.contains(&"tokio".to_string()));
        assert!(keys.contains(&"serde".to_string()));
        assert!(keys.contains(&"reqwest".to_string()));
        // Should NOT include dev-dependencies
        assert!(!keys.contains(&"tempfile".to_string()));
    }

    #[test]
    fn extract_toml_section_keys_caps_at_12() {
        // Generate a section with 20 keys
        let mut content = String::from("[dependencies]\n");
        for i in 0..20 {
            content.push_str(&format!("dep{i} = \"1\"\n"));
        }
        let keys = extract_toml_section_keys(&content, "[dependencies]");
        assert!(
            keys.len() <= 12,
            "should cap at 12 deps, got {}",
            keys.len()
        );
    }
}
