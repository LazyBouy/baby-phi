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
    pub max_turns: u32,
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

pub fn load_context(cfg: &Config) -> (String, String, u64, bool) {
    let identity = read_file_opt("identity.md");
    let journal = read_file_opt("journal.md");
    let journal_excerpt = last_n_lines(&journal, cfg.journal_lines);
    let external = read_file_opt("Input.md");

    // Non-trivial external input: not empty and not just the placeholder text
    let has_external_input = !external.trim().is_empty() && !external.trim().contains("(empty)");

    // Clear Input.md after reading
    let _ = std::fs::write("Input.md", "# External Input\n\n(empty)\n");

    // Increment iteration count
    let count: u64 = read_file_opt("iteration_count").trim().parse().unwrap_or(0);
    let next = count + 1;
    let _ = std::fs::write("iteration_count", next.to_string());

    let user_msg = format!("## Recent Journal\n{journal_excerpt}\n\n## External Input\n{external}");

    (identity, user_msg, next, has_external_input)
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
        AgentEvent::Debug(_) => {} // handled by the PHI_DEBUG wrapper; never reaches here
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

    let (base_system, user_msg, iteration, has_external_input) = load_context(&cfg);
    println!("baby-phi — iteration {iteration}");
    println!(
        "provider: {} / model: {}",
        cfg.active.provider, cfg.active.model
    );

    // System prompt: identity + any dynamic content from agent
    let extra = crate::agent::extra_context();
    let system = if extra.is_empty() {
        base_system
    } else {
        format!("{base_system}\n\n{extra}")
    };

    // Build provider: check core 3 first, then agent's extra_providers()
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
            let extras = crate::agent::extra_providers();
            match extras.into_iter().find(|(name, _)| name == other) {
                Some((_, p)) => p,
                None => {
                    eprintln!("unknown provider '{other}'");
                    std::process::exit(1);
                }
            }
        }
    };

    // Build tool list: base tools from kernel + any extra tools from agent
    let mut tools = default_tools();
    tools.extend(crate::agent::extra_tools());

    let mut messages = vec![Message::user(user_msg)];
    let retry = RetryConfig::default();

    let debug = std::env::var("PHI_DEBUG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let mut handler = |event: AgentEvent| {
        if let AgentEvent::Debug(ref msg) = event {
            if debug {
                eprintln!("[DEBUG] {msg}");
            }
        } else {
            on_event(event);
        }
    };

    let turns_used = match agent_loop(
        &mut messages,
        provider.as_ref(),
        &tools,
        &system,
        &retry,
        cfg.max_turns,
        &mut handler,
    )
    .await
    {
        Ok(turns) => turns,
        Err(e) => {
            let entry = format!("## Iteration {iteration} — FAILED\nProvider error: {e}");
            append_journal(&entry);
            eprintln!("agent error: {e}");
            std::process::exit(1);
        }
    };

    // No-changes guard: journal if agent ran but modified nothing in src/
    // src_changed  — git status --porcelain src/    (only src/ directory)
    // non_src_changed — git status --porcelain      (all files; since we're in the !src_changed
    //                   branch, any hit here is a non-src file: journal.md, LEARNINGS.md, etc.)
    let src_changed = Command::new("git")
        .args(["status", "--porcelain", "src/"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    if !src_changed {
        let non_src_changed = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);
        let reason = if turns_used == 0 {
            "turn 0: agent responded with no tool calls — task may have been unclear or there was nothing to do".to_string()
        } else if turns_used >= cfg.max_turns {
            format!(
                "max_turns ({}) reached — agent was cut off mid-task",
                cfg.max_turns
            )
        } else if !has_external_input {
            format!(
                "agent completed {turns_used} turns but no external input was provided — consider opening a GitHub issue"
            )
        } else if non_src_changed {
            format!(
                "agent completed {turns_used} turns and modified non-src/ files (LEARNINGS.md, journal, etc.) but not src/"
            )
        } else {
            format!(
                "agent completed {turns_used} turns with input but chose not to modify src/ — may need a clearer task"
            )
        };
        append_journal(&format!(
            "## Iteration {iteration} — No changes\n(agent ran but made no modifications to src/) — reason: {reason}"
        ));
        eprintln!("[WARN] Iteration {iteration}: agent completed with no source changes. reason: {reason}");
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

// ── Interactive Mode ──────────────────────────────────────────────────────────

pub async fn run_interactive() {
    use std::io::{self, BufRead, Write as _};

    let cfg = Config::load().unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        std::process::exit(1)
    });
    let api_key = cfg.api_key().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });

    // System prompt: identity + any dynamic content from agent
    let base_system = read_file_opt("identity.md");
    let extra = crate::agent::extra_context();
    let system = if extra.is_empty() {
        base_system
    } else {
        format!("{base_system}\n\n{extra}")
    };

    // Build provider: same logic as run()
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
            let extras = crate::agent::extra_providers();
            match extras.into_iter().find(|(name, _)| name == other) {
                Some((_, p)) => p,
                None => {
                    eprintln!("unknown provider '{other}'");
                    std::process::exit(1);
                }
            }
        }
    };

    let mut tools = default_tools();
    tools.extend(crate::agent::extra_tools());
    let retry = RetryConfig::default();

    // Conversation history persists for the entire session
    let mut messages: Vec<Message> = vec![];

    println!("baby-phi interactive (blank line to send, Ctrl+D to exit)");
    println!(
        "provider: {} / model: {}",
        cfg.active.provider, cfg.active.model
    );

    let stdin = io::stdin();
    loop {
        print!("\nphi> ");
        let _ = io::stdout().flush();

        // Collect multi-line input; blank line submits
        let mut input = String::new();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) if l.is_empty() => break,
                Ok(l) => {
                    input.push_str(&l);
                    input.push('\n');
                }
                Err(_) => return, // EOF / Ctrl+D
            }
        }
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }

        messages.push(Message::user(trimmed));

        let debug = std::env::var("PHI_DEBUG")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let mut handler = |event: AgentEvent| {
            if let AgentEvent::Debug(ref msg) = event {
                if debug {
                    eprintln!("[DEBUG] {msg}");
                }
            } else {
                on_event(event);
            }
        };

        if let Err(e) = agent_loop(
            &mut messages,
            provider.as_ref(),
            &tools,
            &system,
            &retry,
            cfg.max_turns,
            &mut handler,
        )
        .await
        // discard turns_used — no journaling in interactive mode
        .map(|_| ())
        {
            eprintln!("error: {e}");
        }
    }
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
