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

use crate::core::{AgentTool, StreamProvider};

/// Returns any additional tools baby-phi has developed.
/// These are added to the base tool list at runtime.
pub fn extra_tools() -> Vec<Box<dyn AgentTool>> {
    vec![
        Box::new(tools::GitStatusTool),
        Box::new(tools::GitDiffTool),
        Box::new(tools::GitLogTool),
        Box::new(tools::WorkingMemoryTool),
        Box::new(tools::ReadFileRangeTool),
        Box::new(tools::ProjectInfoTool),
        Box::new(tools::GithubTool),
    ]
}

/// Returns any additional providers baby-phi has implemented.
/// Each entry is (provider_name, provider_instance).
/// If config.toml names a provider not in the core 3, core will look here.
pub fn extra_providers() -> Vec<(String, Box<dyn StreamProvider>)> {
    vec![]
}

/// Returns additional text to append to the system prompt each run.
/// Automatically injects: communication skill rules, key LEARNINGS facts.
/// This ensures every run starts with essential context without re-reading files.
pub fn extra_context() -> String {
    context::build_extra_context()
}
