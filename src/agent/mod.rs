// agent/mod.rs — baby-phi's extensible workspace
// This file and everything in src/agent/ is MUTABLE — baby-phi evolves here.
//
// The 6 base tools (bash, read_file, write_file, edit_file, list_files, search)
// and the 3 base providers (anthropic, openai, openrouter) live in src/core/kernel.rs
// and cannot be modified.
//
// To add a new tool:     implement AgentTool and return it from extra_tools().
// To add a new provider: implement StreamProvider and return it from extra_providers().
// To inject system prompt content: return a String from extra_context().

pub mod tools;
pub mod context;
pub mod providers;

use crate::core::{AgentTool, StreamProvider};

/// Returns any additional tools baby-phi has developed.
/// These are added to the base tool list at runtime.
pub fn extra_tools() -> Vec<Box<dyn AgentTool>> {
    vec![
        Box::new(tools::GitStatusTool),
        Box::new(tools::GitDiffTool),
        Box::new(tools::GitLogTool),
        Box::new(tools::WorkingMemoryTool::default()),
        Box::new(tools::ReadFileRangeTool),
        Box::new(tools::ProjectInfoTool),
        Box::new(tools::GithubTool),
        Box::new(tools::FetchUrlTool),
        Box::new(tools::GlobFilesTool),
    ]
}

/// Returns any additional providers baby-phi has implemented.
/// Each entry is (provider_name, provider_instance).
/// If config.toml names a provider not in the core 3, core will look here.
///
/// Supported extra providers:
///   "ollama" — local Ollama instance (OpenAI-compatible API at localhost:11434)
///              Set provider = "ollama" and model = "llama3.2" (or any Ollama model)
///              in config.toml. No API key required.
pub fn extra_providers() -> Vec<(String, Box<dyn StreamProvider>)> {
    // Ollama endpoint: use OLLAMA_HOST env var if set, otherwise localhost:11434
    let ollama_host = std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_endpoint = format!("{ollama_host}/v1/chat/completions");

    // Resolve model: PHI_MODEL env var takes priority, then config.toml
    // (mirrors the logic in ActiveCfg::resolved_model in core)
    let model = std::env::var("PHI_MODEL").ok()
        .filter(|m| !m.is_empty())
        .or_else(|| {
            std::fs::read_to_string("config.toml").ok()
                .and_then(|s| toml::from_str::<crate::core::Config>(&s).ok())
                .map(|c| c.active.model)
                .filter(|m| !m.is_empty())
        })
        .unwrap_or_default();

    vec![(
        "ollama".to_string(),
        Box::new(providers::OllamaProvider::with_endpoint(ollama_endpoint, model)),
    )]
}

/// Returns additional text to append to the system prompt each run.
/// Automatically injects: communication skill rules, key LEARNINGS facts.
/// This ensures every run starts with essential context without re-reading files.
pub fn extra_context() -> String {
    context::build_extra_context()
}
