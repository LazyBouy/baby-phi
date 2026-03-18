mod agent;
use agent::{
    agent_loop, AgentEvent, AgentTool, AnthropicProvider, BashTool, EditFileTool, ListFilesTool,
    Message, OpenAiProvider, OpenRouterProvider, ReadFileTool, RetryConfig, SearchTool,
    StreamProvider, WriteFileTool,
};
use serde::Deserialize;
use std::{io::Write, process::Command};

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Config {
    journal_lines: usize,
    active: ActiveCfg,
}

#[derive(Deserialize)]
struct ActiveCfg {
    provider: String,
    endpoint: String,
    model: String,
}

impl Config {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string("config.toml")?;
        Ok(toml::from_str(&text)?)
    }

    fn api_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let var = match self.active.provider.as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            other => return Err(format!("unknown provider '{other}'").into()),
        };
        std::env::var(var).map_err(|_| format!("{var} not set").into())
    }
}

// ── Context Loading ───────────────────────────────────────────────────────────

fn read_file_opt(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn last_n_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

fn load_context(cfg: &Config) -> (String, String, u64) {
    let identity = read_file_opt("identity.md");
    let journal = read_file_opt("journal.md");
    let journal_excerpt = last_n_lines(&journal, cfg.journal_lines);
    let external = read_file_opt("Input.md");

    // Clear Input.md after reading
    let _ = std::fs::write("Input.md", "# External Input\n\n(empty)\n");

    // Increment iteration count
    let count: u64 = read_file_opt("iteration_count").trim().parse().unwrap_or(0);
    let next = count + 1;
    let _ = std::fs::write("iteration_count", next.to_string());

    let user_msg = format!("## Recent Journal\n{journal_excerpt}\n\n## External Input\n{external}");

    (identity, user_msg, next)
}

// ── Test Gate ─────────────────────────────────────────────────────────────────

fn test_gate(label: &str) -> Result<(), String> {
    for (name, args) in [
        ("build", vec!["build", "--quiet"]),
        ("test", vec!["test", "--quiet"]),
    ] {
        let out = Command::new("cargo")
            .args(&args)
            .output()
            .map_err(|e| format!("cargo {name}: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("[{label}] cargo {name} failed:\n{stderr}"));
        }
    }
    Ok(())
}

fn append_journal(entry: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open("journal.md") {
        let _ = writeln!(f, "\n{entry}");
    }
}

// ── Event Display ─────────────────────────────────────────────────────────────

fn on_event(event: AgentEvent) {
    match event {
        AgentEvent::TurnStart { turn } => println!("\n── turn {turn} ──────────────────────"),
        AgentEvent::TextDelta(t) => {
            print!("{t}");
            let _ = std::io::stdout().flush();
        }
        AgentEvent::ToolStart { name, input } => println!("\n[{name}] {input}"),
        AgentEvent::ToolEnd {
            name,
            output,
            is_error,
        } => {
            let tag = if is_error { "ERR" } else { "OK" };
            let preview: String = output.lines().take(5).collect::<Vec<_>>().join("\n");
            println!("[{name}/{tag}] {preview}");
        }
        AgentEvent::TurnEnd | AgentEvent::AgentEnd => println!(),
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cfg = Config::load().unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        std::process::exit(1)
    });
    let api_key = cfg.api_key().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });

    // Pre-run test gate
    if let Err(e) = test_gate("before") {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let (system, user_msg, iteration) = load_context(&cfg);
    println!("baby-phi — iteration {iteration}");
    println!(
        "provider: {} / model: {}",
        cfg.active.provider, cfg.active.model
    );

    // Build provider
    let provider: Box<dyn StreamProvider> = match cfg.active.provider.as_str() {
        "anthropic" => Box::new(AnthropicProvider {
            endpoint: cfg.active.endpoint.clone(),
            api_key: api_key.clone(),
            model: cfg.active.model.clone(),
        }),
        "openai" => Box::new(OpenAiProvider {
            endpoint: cfg.active.endpoint.clone(),
            api_key: api_key.clone(),
            model: cfg.active.model.clone(),
        }),
        "openrouter" => Box::new(OpenRouterProvider {
            endpoint: cfg.active.endpoint.clone(),
            api_key: api_key.clone(),
            model: cfg.active.model.clone(),
        }),
        other => {
            eprintln!("unknown provider '{other}'");
            std::process::exit(1);
        }
    };

    let tools: Vec<Box<dyn AgentTool>> = vec![
        Box::new(BashTool::new()),
        Box::new(ReadFileTool::new()),
        Box::new(WriteFileTool::new()),
        Box::new(EditFileTool::new()),
        Box::new(ListFilesTool::new()),
        Box::new(SearchTool::new()),
    ];

    let mut messages = vec![Message::user(user_msg)];
    let retry = RetryConfig::default();

    if let Err(e) = agent_loop(
        &mut messages,
        provider.as_ref(),
        &tools,
        &system,
        &retry,
        &mut on_event,
    )
    .await
    {
        let entry = format!("## Iteration {iteration} — FAILED\nProvider error: {e}");
        append_journal(&entry);
        eprintln!("agent error: {e}");
        std::process::exit(1);
    }

    // Post-run test gate
    if let Err(e) = test_gate("after") {
        let entry = format!("## Iteration {iteration} — BUILD FAILED\n{e}");
        append_journal(&entry);
        eprintln!("{e}");
        std::process::exit(1);
    }

    println!("\niteration {iteration} complete.");
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentTool, BashTool, EditFileTool, ProviderError, SearchTool};
    use serde_json::json;

    #[test]
    fn config_loads() {
        let c = Config::load().expect("config.toml must parse");
        assert!(!c.active.provider.is_empty(), "provider must not be empty");
        assert!(!c.active.endpoint.is_empty(), "endpoint must not be empty");
        assert!(!c.active.model.is_empty(), "model must not be empty");
    }

    #[tokio::test]
    async fn bash_tool_basic() {
        let t = BashTool::new();
        let r = t.execute(json!({"command": "echo hello"})).await;
        assert!(!r.is_error, "echo should succeed");
        assert_eq!(r.content.trim(), "hello");
    }

    #[tokio::test]
    async fn edit_tool_rejects_missing_old_string() {
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(f.path(), "abc").unwrap();
        let t = EditFileTool::new();
        let r = t
            .execute(json!({
                "path": f.path().to_str().unwrap(),
                "old_string": "xyz",
                "new_string": "def"
            }))
            .await;
        assert!(r.is_error, "must error when old_string not found");
    }

    #[test]
    fn search_tool_does_not_crash() {
        let t = SearchTool::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let r = rt.block_on(t.execute(json!({"pattern": "fn main", "path": "src/"})));
        assert!(!r.is_error, "search must not error (rg or grep)");
        assert!(r.content.contains("main"), "must find fn main in src/");
    }

    // ── Retry-After parsing tests ─────────────────────────────────────────────

    #[test]
    fn parse_retry_after_integer_seconds() {
        use crate::agent::parse_retry_after;
        assert_eq!(parse_retry_after("5"), Some(5000));
        assert_eq!(parse_retry_after("0"), Some(0));
        assert_eq!(parse_retry_after("120"), Some(120_000));
    }

    #[test]
    fn parse_retry_after_float_seconds() {
        use crate::agent::parse_retry_after;
        assert_eq!(parse_retry_after("1.5"), Some(1500));
        assert_eq!(parse_retry_after("0.5"), Some(500));
        assert_eq!(parse_retry_after("30.0"), Some(30_000));
    }

    #[test]
    fn parse_retry_after_whitespace() {
        use crate::agent::parse_retry_after;
        assert_eq!(parse_retry_after("  10  "), Some(10_000));
        assert_eq!(parse_retry_after(" 2.5 "), Some(2500));
    }

    #[test]
    fn parse_retry_after_invalid() {
        use crate::agent::parse_retry_after;
        assert_eq!(parse_retry_after("not-a-number"), None);
        assert_eq!(parse_retry_after(""), None);
    }

    #[test]
    fn rate_limited_error_has_retry_after() {
        let err = ProviderError::classify(429, "rate limited", Some(5000));
        assert!(err.is_retryable());
        assert_eq!(
            err.retry_after(),
            Some(std::time::Duration::from_millis(5000))
        );
    }

    #[test]
    fn rate_limited_error_without_retry_after() {
        let err = ProviderError::classify(429, "rate limited", None);
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn non_rate_limited_error_no_retry_after() {
        let err = ProviderError::classify(500, "server error", None);
        assert!(err.is_retryable()); // Network errors are retryable
        assert_eq!(err.retry_after(), None);

        let err = ProviderError::classify(400, "bad request", None);
        assert!(!err.is_retryable());
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn classify_passes_retry_after_only_for_429() {
        // Even if we accidentally pass retry_after_ms for a non-429 status,
        // only 429 produces RateLimited variant
        let err = ProviderError::classify(500, "server error", Some(5000));
        assert_eq!(err.retry_after(), None); // Network variant has no retry_after
    }
}
