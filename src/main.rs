use phi_core::{
    agents_from_config, parse_config_file, save_session, AgentEvent, SessionRecorder,
    SessionRecorderConfig, StreamDelta,
};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = parse_config_file(Path::new("config.toml")).unwrap_or_else(|e| {
        eprintln!("Failed to parse config.toml: {e}");
        std::process::exit(1);
    });

    let agents = agents_from_config(&config).unwrap_or_else(|e| {
        eprintln!("Failed to build agents: {e}");
        std::process::exit(1);
    });

    let (name, mut agent_arc) = agents.into_iter().next().expect("No agents configured");
    println!("Agent: {}", name);

    let agent = Arc::get_mut(&mut agent_arc).expect("Failed to get mutable agent reference");

    // Resolve tools from config via registry
    let registry = phi_core::tools::ToolRegistry::new().with_defaults();
    let tools = registry.resolve(&config.tools.enabled);
    agent.set_tools(tools);

    let prompt = std::env::args().nth(1).unwrap_or_else(|| {
        "Write a marketing email for our new AI consulting service that helps \
         mid-size companies automate their customer support with AI agents."
            .to_string()
    });

    println!("=== NexusAI Marketing Email Agent ===\n");
    println!("Prompt: {}\n", prompt);
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
                print!("{}", delta);
                std::io::stdout().flush().ok();
            }
            AgentEvent::ToolExecutionStart { tool_name, .. } => {
                println!("\n[tool: {}]", tool_name);
            }
            AgentEvent::ToolExecutionEnd {
                tool_name,
                is_error,
                ..
            } => {
                let status = if *is_error { "failed" } else { "done" };
                println!("[tool: {} — {}]", tool_name, status);
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

    // Persist session
    recorder.flush();
    let session_dir = Path::new("workspace/session");
    for session in recorder.drain_completed() {
        match save_session(&session, session_dir) {
            Ok(path) => println!("Session saved to: {}", path.display()),
            Err(e) => eprintln!("Failed to save session: {e}"),
        }
    }
}
