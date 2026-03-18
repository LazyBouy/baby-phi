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
pub struct WorkingMemoryTool {
    /// Path to the memory file. Use `WorkingMemoryTool::default()` for production
    /// (uses WORKING_MEMORY_PATH), or `WorkingMemoryTool::with_path(p)` in tests
    /// to avoid shared-state races between concurrent test runs.
    path: String,
}

const WORKING_MEMORY_PATH: &str = "/tmp/baby_phi_working_memory.md";

impl Default for WorkingMemoryTool {
    fn default() -> Self {
        Self {
            path: WORKING_MEMORY_PATH.to_string(),
        }
    }
}

impl WorkingMemoryTool {
    /// Construct with a custom path — used in tests to isolate file state.
    #[cfg(test)]
    pub fn with_path(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

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
            "read" => match tokio::fs::read_to_string(&self.path).await {
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
                match tokio::fs::write(&self.path, content).await {
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
                    if let Some(stripped) = line.strip_prefix("module ") {
                        lines.push(format!("Module:    {}", stripped));
                    }
                    if let Some(stripped) = line.strip_prefix("go ") {
                        lines.push(format!("Go:        {}", stripped));
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
                    if let Some(after_eq) = rest.strip_prefix('=') {
                        let val = after_eq.trim().trim_matches('"').to_string();
                        if !val.is_empty() {
                            return Some(val);
                        }
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

// ── GitHub Tool ───────────────────────────────────────────────────────────────

/// Interact with the GitHub API: list issues, close issues, add comments.
/// Requires GH_TOKEN environment variable. Reads GITHUB_REPOSITORY env var
/// or falls back to the repo param for the target repo (owner/name).
///
/// Actions:
///   list_issues  — list open issues with comment counts and reaction counts
///   close_issue  — close an issue (number required), optionally with a comment
///   add_comment  — post a comment on an issue (number + body required)
pub struct GithubTool;

#[async_trait]
impl AgentTool for GithubTool {
    fn name(&self) -> &str {
        "github"
    }
    fn description(&self) -> &str {
        "Interact with GitHub: list open issues, close issues, add comments. \
         Requires GH_TOKEN env var. \
         Actions: list_issues | close_issue | add_comment. \
         For close_issue/add_comment: provide 'number' (issue number). \
         For close_issue: optionally provide 'comment' to post before closing. \
         For add_comment: provide 'body'. \
         Optional 'repo' param (default: auto-detected from git remote)."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_issues", "close_issue", "add_comment"],
                    "description": "What to do"
                },
                "number": {
                    "type": "integer",
                    "description": "Issue number (required for close_issue and add_comment)"
                },
                "comment": {
                    "type": "string",
                    "description": "Comment to post before closing (optional, for close_issue)"
                },
                "body": {
                    "type": "string",
                    "description": "Comment body (required for add_comment)"
                },
                "repo": {
                    "type": "string",
                    "description": "GitHub repo in owner/name format (auto-detected if not provided)"
                }
            },
            "required": ["action"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let action = match input["action"].as_str() {
            Some(a) => a.to_string(),
            None => return ToolResult { content: "missing 'action'".into(), is_error: true },
        };

        let token = match std::env::var("GH_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN")) {
            Ok(t) if !t.is_empty() => t,
            _ => return ToolResult {
                content: "GH_TOKEN (or GITHUB_TOKEN) env var not set — cannot call GitHub API".into(),
                is_error: true,
            },
        };

        // Resolve repo: explicit param → git remote → error
        let repo = if let Some(r) = input["repo"].as_str() {
            r.to_string()
        } else {
            match detect_github_repo().await {
                Some(r) => r,
                None => return ToolResult {
                    content: "could not detect GitHub repo from git remote; provide 'repo' param (owner/name)".into(),
                    is_error: true,
                },
            }
        };

        match action.as_str() {
            "list_issues" => github_list_issues(&token, &repo).await,
            "close_issue" => {
                let number = match input["number"].as_u64() {
                    Some(n) => n,
                    None => return ToolResult { content: "missing 'number' for close_issue".into(), is_error: true },
                };
                let comment = input["comment"].as_str().map(|s| s.to_string());
                github_close_issue(&token, &repo, number, comment).await
            }
            "add_comment" => {
                let number = match input["number"].as_u64() {
                    Some(n) => n,
                    None => return ToolResult { content: "missing 'number' for add_comment".into(), is_error: true },
                };
                let body = match input["body"].as_str() {
                    Some(b) => b.to_string(),
                    None => return ToolResult { content: "missing 'body' for add_comment".into(), is_error: true },
                };
                github_add_comment(&token, &repo, number, &body).await
            }
            other => ToolResult {
                content: format!("unknown action '{other}' — must be list_issues, close_issue, or add_comment"),
                is_error: true,
            },
        }
    }
}

/// Auto-detect the GitHub repo slug from `git remote get-url origin`.
async fn detect_github_repo() -> Option<String> {
    let out = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await
    .ok()?.ok()?;

    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Match both HTTPS (https://github.com/owner/repo) and SSH (git@github.com:owner/repo)
    // and strip .git suffix
    let slug = if let Some(rest) = url.strip_prefix("https://github.com/") {
        rest.trim_end_matches(".git").to_string()
    } else if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest.trim_end_matches(".git").to_string()
    } else {
        return None;
    };

    if slug.contains('/') {
        Some(slug)
    } else {
        None
    }
}

/// List open issues with id, title, comments count, reactions count.
async fn github_list_issues(token: &str, repo: &str) -> ToolResult {
    let url = format!("https://api.github.com/repos/{repo}/issues?state=open&per_page=50");
    match github_get(token, &url).await {
        Err(e) => ToolResult { content: format!("GitHub API error: {e}"), is_error: true },
        Ok(body) => {
            match serde_json::from_str::<Value>(&body) {
                Err(e) => ToolResult { content: format!("JSON parse error: {e}\nRaw: {}", &body[..body.len().min(200)]), is_error: true },
                Ok(Value::Array(issues)) => {
                    if issues.is_empty() {
                        return ToolResult { content: "No open issues.".into(), is_error: false };
                    }
                    let lines: Vec<String> = issues.iter().map(|i| {
                        let num = i["number"].as_u64().unwrap_or(0);
                        let title = i["title"].as_str().unwrap_or("(no title)");
                        let comments = i["comments"].as_u64().unwrap_or(0);
                        let reactions = i["reactions"]["total_count"].as_u64().unwrap_or(0);
                        format!("#{num}: {title} (comments:{comments}, reactions:{reactions})")
                    }).collect();
                    ToolResult { content: lines.join("\n"), is_error: false }
                }
                Ok(other) => {
                    // API returned an error object
                    let msg = other["message"].as_str().unwrap_or("unexpected response");
                    ToolResult { content: format!("GitHub API: {msg}"), is_error: true }
                }
            }
        }
    }
}

/// Close a GitHub issue, optionally posting a comment first.
async fn github_close_issue(token: &str, repo: &str, number: u64, comment: Option<String>) -> ToolResult {
    // Post comment first if provided
    if let Some(body) = comment {
        let comment_result = github_add_comment(token, repo, number, &body).await;
        if comment_result.is_error {
            return ToolResult {
                content: format!("Failed to post comment before closing: {}", comment_result.content),
                is_error: true,
            };
        }
    }

    let url = format!("https://api.github.com/repos/{repo}/issues/{number}");
    let payload = serde_json::json!({"state": "closed"}).to_string();

    match github_patch(token, &url, &payload).await {
        Err(e) => ToolResult { content: format!("GitHub API error: {e}"), is_error: true },
        Ok(body) => {
            match serde_json::from_str::<Value>(&body) {
                Ok(resp) if resp["state"].as_str() == Some("closed") => ToolResult {
                    content: format!("✓ Closed issue #{number}: {}", resp["title"].as_str().unwrap_or("?")),
                    is_error: false,
                },
                Ok(resp) => {
                    let msg = resp["message"].as_str().unwrap_or("unexpected response");
                    ToolResult { content: format!("GitHub API: {msg}"), is_error: true }
                }
                Err(e) => ToolResult { content: format!("JSON parse error: {e}"), is_error: true },
            }
        }
    }
}

/// Post a comment on a GitHub issue.
async fn github_add_comment(token: &str, repo: &str, number: u64, body: &str) -> ToolResult {
    let url = format!("https://api.github.com/repos/{repo}/issues/{number}/comments");
    let payload = serde_json::json!({"body": body}).to_string();

    match github_post(token, &url, &payload).await {
        Err(e) => ToolResult { content: format!("GitHub API error: {e}"), is_error: true },
        Ok(body_resp) => {
            match serde_json::from_str::<Value>(&body_resp) {
                Ok(resp) if resp["id"].is_number() => ToolResult {
                    content: format!("✓ Comment posted on #{number} (id: {})", resp["id"]),
                    is_error: false,
                },
                Ok(resp) => {
                    let msg = resp["message"].as_str().unwrap_or("unexpected response");
                    ToolResult { content: format!("GitHub API: {msg}"), is_error: true }
                }
                Err(e) => ToolResult { content: format!("JSON parse error: {e}"), is_error: true },
            }
        }
    }
}

// ── GitHub HTTP helpers ───────────────────────────────────────────────────────

async fn github_get(token: &str, url: &str) -> Result<String, String> {
    github_request("GET", token, url, None).await
}

async fn github_patch(token: &str, url: &str, body: &str) -> Result<String, String> {
    github_request("PATCH", token, url, Some(body)).await
}

async fn github_post(token: &str, url: &str, body: &str) -> Result<String, String> {
    github_request("POST", token, url, Some(body)).await
}

async fn github_request(method: &str, token: &str, url: &str, body: Option<&str>) -> Result<String, String> {
    let out = tokio::time::timeout(
        Duration::from_secs(15),
        tokio::process::Command::new("curl")
            .args(build_curl_args(method, token, url, body))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await
    .map_err(|_| "HTTP request timed out (15s)".to_string())?
    .map_err(|e| format!("curl exec error: {e}"))?;

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn build_curl_args(method: &str, token: &str, url: &str, body: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "-s".to_string(),
        "-X".to_string(), method.to_string(),
        "-H".to_string(), format!("Authorization: Bearer {token}"),
        "-H".to_string(), "Accept: application/vnd.github+json".to_string(),
        "-H".to_string(), "X-GitHub-Api-Version: 2022-11-28".to_string(),
    ];
    if let Some(b) = body {
        args.push("-H".to_string());
        args.push("Content-Type: application/json".to_string());
        args.push("-d".to_string());
        args.push(b.to_string());
    }
    args.push(url.to_string());
    args
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

// ── Fetch URL Tool ────────────────────────────────────────────────────────────

/// Fetches the content of a URL and returns it as plain text.
/// HTML is stripped to extract readable text. Output is capped at ~10KB.
/// Uses curl subprocess — no extra dependencies required.
pub struct FetchUrlTool;

/// Maximum bytes to return from a URL fetch (keeps token usage sane)
const FETCH_MAX_BYTES: usize = 10_000;

/// Strip HTML tags and collapse whitespace into readable plain text.
/// Not a full HTML parser — good enough for documentation and issue pages.
fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_buf = String::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_tag {
            if c == '>' {
                // Check if we're entering/leaving script or style blocks
                let tag_lower = tag_buf.to_lowercase();
                if tag_lower.starts_with("script") {
                    in_script = true;
                } else if tag_lower.starts_with("/script") {
                    in_script = false;
                } else if tag_lower.starts_with("style") {
                    in_style = true;
                } else if tag_lower.starts_with("/style") {
                    in_style = false;
                }
                // Block elements get a newline
                let is_block = ["p", "div", "br", "h1", "h2", "h3", "h4", "h5", "h6",
                    "li", "tr", "td", "th", "article", "section", "header", "footer",
                    "blockquote", "pre", "code"]
                    .iter()
                    .any(|&t| tag_lower == t || tag_lower.starts_with(&format!("{t} ")));
                if is_block {
                    out.push('\n');
                }
                tag_buf.clear();
                in_tag = false;
            } else {
                tag_buf.push(c);
            }
        } else if c == '<' {
            in_tag = true;
            tag_buf.clear();
        } else if !in_script && !in_style {
            // Decode common HTML entities
            if c == '&' {
                // Collect until ';' or a non-entity char
                let mut entity = String::new();
                let mut j = i + 1;
                while j < chars.len() && j < i + 10 && chars[j] != ';' && chars[j] != '<' {
                    entity.push(chars[j]);
                    j += 1;
                }
                if j < chars.len() && chars[j] == ';' {
                    let decoded = match entity.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        "nbsp" => " ",
                        "mdash" | "#8212" => "—",
                        "ndash" | "#8211" => "–",
                        "ldquo" | "#8220" => "\u{201C}",
                        "rdquo" | "#8221" => "\u{201D}",
                        _ => "",
                    };
                    if !decoded.is_empty() {
                        out.push_str(decoded);
                        i = j + 1;
                        continue;
                    }
                }
            }
            out.push(c);
        }
        i += 1;
    }

    // Collapse multiple blank lines into one and trim
    let mut result = String::new();
    let mut blank_count = 0u32;
    for line in out.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim().to_string()
}

#[async_trait]
impl AgentTool for FetchUrlTool {
    fn name(&self) -> &str {
        "fetch_url"
    }
    fn description(&self) -> &str {
        "Fetch the content of a URL and return it as plain text. \
         HTML is stripped to readable text. Output capped at ~10KB. \
         Use for: reading documentation, GitHub pages, API references, or any web resource. \
         Optional 'timeout' param in seconds (default: 15)."
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Request timeout in seconds (default: 15, max: 60)"
                }
            },
            "required": ["url"]
        })
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let url = match input["url"].as_str() {
            Some(u) if !u.is_empty() => u.to_string(),
            _ => return ToolResult { content: "missing 'url' parameter".into(), is_error: true },
        };

        // Validate URL scheme — only http/https allowed
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return ToolResult {
                content: format!("invalid URL scheme — only http:// and https:// are supported: {url}"),
                is_error: true,
            };
        }

        let timeout_secs = input["timeout"].as_u64().unwrap_or(15).min(60);

        // Use curl: -s silent, -L follow redirects, --max-time timeout,
        // -A sets User-Agent, --max-filesize caps download
        let output = tokio::process::Command::new("curl")
            .args([
                "-s",
                "-L",
                "--max-time", &timeout_secs.to_string(),
                "-A", "baby-phi/1.0 (coding agent; +https://github.com/yologdev/baby-phi)",
                "--max-filesize", "524288", // 512KB raw max
                "-H", "Accept: text/html,text/plain,application/json,*/*",
                &url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let child = match output {
            Ok(c) => c,
            Err(e) => return ToolResult {
                content: format!("failed to spawn curl: {e}"),
                is_error: true,
            },
        };

        let result = match tokio::time::timeout(
            Duration::from_secs(timeout_secs + 5),
            child.wait_with_output(),
        ).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return ToolResult {
                content: format!("curl execution error: {e}"),
                is_error: true,
            },
            Err(_) => return ToolResult {
                content: format!("fetch timed out after {timeout_secs}s"),
                is_error: true,
            },
        };

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            // curl exit code 22 = HTTP 4xx/5xx
            let code = result.status.code().unwrap_or(-1);
            return ToolResult {
                content: format!("curl failed (exit {code}): {stderr}"),
                is_error: true,
            };
        }

        let raw = String::from_utf8_lossy(&result.stdout);

        // Check content-type heuristic: if it looks like JSON, return as-is
        let content_type_is_json = raw.trim_start().starts_with('{') || raw.trim_start().starts_with('[');

        let text = if content_type_is_json {
            raw.trim().to_string()
        } else {
            strip_html(&raw)
        };

        // Cap output
        let truncated = if text.len() > FETCH_MAX_BYTES {
            format!("{}\n\n[truncated — showing first {} of {} bytes]",
                &text[..FETCH_MAX_BYTES], FETCH_MAX_BYTES, text.len())
        } else {
            text
        };

        if truncated.trim().is_empty() {
            return ToolResult {
                content: "(empty response)".into(),
                is_error: false,
            };
        }

        ToolResult { content: truncated, is_error: false }
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
        // Each test uses its own unique path to avoid shared-state races.
        let path = "/tmp/baby_phi_test_roundtrip.md";
        let _ = tokio::fs::remove_file(path).await; // clean slate
        let tool = WorkingMemoryTool::with_path(path);
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

        let _ = tokio::fs::remove_file(path).await; // clean up
    }

    #[tokio::test]
    async fn working_memory_read_when_empty() {
        // Use a unique path that no other test writes to.
        let path = "/tmp/baby_phi_test_read_empty.md";
        let _ = tokio::fs::remove_file(path).await; // ensure it doesn't exist

        let tool = WorkingMemoryTool::with_path(path);
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
        let tool = WorkingMemoryTool::default();
        let result = tool.execute(json!({})).await;
        assert!(result.is_error, "missing action must be an error");
    }

    #[tokio::test]
    async fn working_memory_unknown_action() {
        let tool = WorkingMemoryTool::default();
        let result = tool.execute(json!({"action": "delete"})).await;
        assert!(result.is_error, "unknown action must be an error");
    }

    #[tokio::test]
    async fn working_memory_write_without_content() {
        let tool = WorkingMemoryTool::default();
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
            Box::new(WorkingMemoryTool::default()),
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

    // ── GithubTool tests ──────────────────────────────────────────────────────

    #[test]
    fn github_tool_definition_is_valid() {
        let tool = GithubTool;
        let def = tool.definition();
        assert_eq!(def.name, "github");
        assert!(!def.description.is_empty());
        assert!(def.parameters.is_object());
    }

    #[test]
    fn github_tool_parameters_has_required_action() {
        let tool = GithubTool;
        let schema = tool.parameters_schema();
        let required = &schema["required"];
        assert!(
            required.as_array().map(|a| a.iter().any(|v| v.as_str() == Some("action"))).unwrap_or(false),
            "action should be required: {schema}"
        );
    }

    #[tokio::test]
    async fn github_tool_missing_action_errors() {
        let tool = GithubTool;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_error, "missing action must be an error");
        assert!(result.content.contains("action"), "error should mention 'action': {}", result.content);
    }

    #[tokio::test]
    async fn github_tool_unknown_action_errors() {
        let tool = GithubTool;
        // Even without a token, unknown action should be caught
        let result = tool.execute(serde_json::json!({"action": "delete_everything"})).await;
        // Either token error or unknown action error — both are is_error=true
        assert!(result.is_error, "unknown action must be an error");
    }

    #[tokio::test]
    async fn github_tool_close_issue_missing_number_errors() {
        let tool = GithubTool;
        // With no GH_TOKEN in test environment, this will fail at token check —
        // but if token is set, it should fail at number validation.
        // Either way is_error must be true.
        let result = tool.execute(serde_json::json!({"action": "close_issue"})).await;
        assert!(result.is_error, "close_issue without number must be an error");
    }

    #[tokio::test]
    async fn github_tool_add_comment_missing_body_errors() {
        let tool = GithubTool;
        let result = tool.execute(serde_json::json!({"action": "add_comment", "number": 1})).await;
        assert!(result.is_error, "add_comment without body must be an error");
    }

    #[test]
    fn build_curl_args_get_no_body() {
        let args = build_curl_args("GET", "mytoken", "https://api.github.com/test", None);
        assert!(args.contains(&"GET".to_string()));
        assert!(args.contains(&"Authorization: Bearer mytoken".to_string()));
        assert!(args.contains(&"https://api.github.com/test".to_string()));
        // No -d flag for GET
        assert!(!args.contains(&"-d".to_string()));
    }

    #[test]
    fn build_curl_args_post_with_body() {
        let args = build_curl_args("POST", "tok", "https://api.github.com/x", Some(r#"{"key":"val"}"#));
        assert!(args.contains(&"POST".to_string()));
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&r#"{"key":"val"}"#.to_string()));
    }

    #[tokio::test]
    async fn detect_github_repo_returns_slug_or_none() {
        // In a git repo, should return Some("owner/repo") or None (not panic)
        let result = detect_github_repo().await;
        // We can't assert the exact value in CI, but it must be well-formed if present
        if let Some(slug) = result {
            assert!(slug.contains('/'), "repo slug must be owner/name format: {slug}");
            assert!(!slug.ends_with(".git"), "slug should not end with .git: {slug}");
        }
        // None is also valid (no git remote in some CI environments)
    }

    // ── FetchUrlTool tests ────────────────────────────────────────────────────

    #[test]
    fn strip_html_removes_tags() {
        let html = "<h1>Hello</h1><p>World</p>";
        let result = strip_html(html);
        assert!(result.contains("Hello"), "should contain Hello: {result}");
        assert!(result.contains("World"), "should contain World: {result}");
        assert!(!result.contains('<'), "should not contain angle brackets: {result}");
    }

    #[test]
    fn strip_html_decodes_entities() {
        let html = "AT&amp;T &lt;rocks&gt; &quot;quoted&quot;";
        let result = strip_html(html);
        assert!(result.contains("AT&T"), "should decode &amp;: {result}");
        assert!(result.contains('<'), "should decode &lt;: {result}");
        assert!(result.contains('"'), "should decode &quot;: {result}");
    }

    #[test]
    fn strip_html_skips_script_and_style() {
        let html = "<div>visible</div><script>alert('hidden')</script><style>.hidden{}</style><p>also visible</p>";
        let result = strip_html(html);
        assert!(result.contains("visible"), "should contain visible: {result}");
        assert!(result.contains("also visible"), "should contain also visible: {result}");
        assert!(!result.contains("alert"), "should not contain script content: {result}");
        assert!(!result.contains(".hidden"), "should not contain style content: {result}");
    }

    #[test]
    fn strip_html_plain_text_unchanged() {
        let text = "Just plain text\nwith newlines";
        let result = strip_html(text);
        assert!(result.contains("Just plain text"), "plain text should pass through: {result}");
    }

    #[tokio::test]
    async fn fetch_url_missing_url_errors() {
        let tool = FetchUrlTool;
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_error, "missing url must be error: {}", result.content);
        assert!(result.content.contains("url"), "error should mention 'url': {}", result.content);
    }

    #[tokio::test]
    async fn fetch_url_invalid_scheme_errors() {
        let tool = FetchUrlTool;
        let result = tool.execute(serde_json::json!({"url": "ftp://example.com/file"})).await;
        assert!(result.is_error, "ftp scheme must be rejected: {}", result.content);
        assert!(result.content.contains("scheme"), "should mention scheme: {}", result.content);
    }

    #[tokio::test]
    async fn fetch_url_file_scheme_errors() {
        let tool = FetchUrlTool;
        let result = tool.execute(serde_json::json!({"url": "file:///etc/passwd"})).await;
        assert!(result.is_error, "file scheme must be rejected: {}", result.content);
    }

    #[test]
    fn fetch_url_parameters_schema_requires_url() {
        let tool = FetchUrlTool;
        let schema = tool.parameters_schema();
        let required = &schema["required"];
        assert!(
            required.as_array().map(|a| a.iter().any(|v| v.as_str() == Some("url"))).unwrap_or(false),
            "url should be required in schema: {schema}"
        );
    }
}
