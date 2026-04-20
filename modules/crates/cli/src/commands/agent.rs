//! `baby-phi agent demo` — the legacy phi-core agent-loop demo.
//!
//! Ported verbatim from pre-M1 `main.rs`. Consumes the legacy
//! `baby-phi/config.toml` at the working directory and streams a
//! single agent turn to stdout. Kept for prototype continuity; real
//! agent-management subcommands (registration, listing, invocation)
//! land in M2+.

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use phi_core::{
    agents_from_config, parse_config_file, save_session, AgentEvent, SessionRecorder,
    SessionRecorderConfig, StreamDelta,
};

use crate::AgentCommand;

pub async fn run(cmd: AgentCommand) -> i32 {
    match cmd {
        AgentCommand::Demo { prompt } => demo(prompt).await,
    }
}

async fn demo(prompt_override: Option<String>) -> i32 {
    let config = match parse_config_file(Path::new("config.toml")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse config.toml: {e}");
            return 1;
        }
    };

    let agents = match agents_from_config(&config) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to build agents: {e}");
            return 1;
        }
    };

    let (name, mut agent_arc) = match agents.into_iter().next() {
        Some(pair) => pair,
        None => {
            eprintln!("No agents configured");
            return 1;
        }
    };
    println!("Agent: {name}");

    let agent = match Arc::get_mut(&mut agent_arc) {
        Some(a) => a,
        None => {
            eprintln!("Failed to get mutable agent reference");
            return 1;
        }
    };

    let registry = phi_core::tools::ToolRegistry::new().with_defaults();
    let tools = registry.resolve(&config.tools.enabled);
    agent.set_tools(tools);

    let prompt = prompt_override.unwrap_or_else(|| {
        "Write a marketing email for our new AI consulting service that helps \
         mid-size companies automate their customer support with AI agents."
            .to_string()
    });

    println!("=== baby-phi agent demo ===\n");
    println!("Prompt: {prompt}\n");
    println!("---\n");

    let mut recorder = SessionRecorder::new(SessionRecorderConfig::default());
    let mut rx = agent.prompt(prompt).await;

    while let Some(event) = rx.recv().await {
        recorder.on_event(event.clone());

        match &event {
            AgentEvent::MessageUpdate {
                delta: StreamDelta::Text { delta },
                ..
            } => {
                print!("{delta}");
                std::io::stdout().flush().ok();
            }
            AgentEvent::ToolExecutionStart { tool_name, .. } => {
                println!("\n[tool: {tool_name}]");
            }
            AgentEvent::ToolExecutionEnd {
                tool_name,
                is_error,
                ..
            } => {
                let status = if *is_error { "failed" } else { "done" };
                println!("[tool: {tool_name} — {status}]");
            }
            AgentEvent::AgentEnd { usage, .. } => {
                println!("\n--- Done ---");
                println!(
                    "Tokens: {} input, {} output, {} total",
                    usage.input, usage.output, usage.total_tokens
                );
            }
            _ => {}
        }
    }

    recorder.flush();
    let session_dir = Path::new("workspace/session");
    for session in recorder.drain_completed() {
        match save_session(&session, session_dir) {
            Ok(path) => println!("Session saved to: {}", path.display()),
            Err(e) => eprintln!("Failed to save session: {e}"),
        }
    }
    0
}
