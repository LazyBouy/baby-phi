// core/mod.rs — baby-phi immutable orchestration layer
// IMMUTABLE: Config loading, run(), test gates, journal, event display.
// baby-phi CANNOT modify this file. evolve.sh reverts any changes before committing.

pub mod kernel;
pub use kernel::*;

use serde::Deserialize;
use std::{io::Write, process::Command};

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct Config {
    pub journal_lines: usize,
    pub active: ActiveCfg,
}

#[derive(Deserialize)]
pub struct ActiveCfg {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string("config.toml")?;
        Ok(toml::from_str(&text)?)
    }

    pub fn api_key(&self) -> Result<String, Box<dyn std::error::Error>> {
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

pub fn read_file_opt(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn last_n_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

pub fn load_context(cfg: &Config) -> (String, String, u64) {
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

pub fn test_gate(label: &str) -> Result<(), String> {
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

pub fn append_journal(entry: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open("journal.md") {
        let _ = writeln!(f, "\n{entry}");
    }
}

// ── Event Display ─────────────────────────────────────────────────────────────

pub fn on_event(event: AgentEvent) {
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
        AgentEvent::Warn(msg) => eprintln!("[WARN] {msg}"),
    }
}

// ── Main Run Loop ─────────────────────────────────────────────────────────────

pub async fn run() {
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

    // Build tool list: base tools from kernel + any extra tools from agent
    let mut tools = default_tools();
    tools.extend(crate::agent::extra_tools());

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

    // No-changes guard: journal if agent ran but modified nothing in src/
    let src_changed = Command::new("git")
        .args(["status", "--porcelain", "src/"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    if !src_changed {
        append_journal(&format!(
            "## Iteration {iteration} — No changes\n(agent ran but made no modifications to src/)"
        ));
        eprintln!("[WARN] Iteration {iteration}: agent completed with no source changes.");
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

// ── Immutable Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_loads() {
        let c = Config::load().expect("config.toml must parse");
        assert!(!c.active.provider.is_empty(), "provider must not be empty");
        assert!(!c.active.endpoint.is_empty(), "endpoint must not be empty");
        assert!(!c.active.model.is_empty(), "model must not be empty");
    }
}
