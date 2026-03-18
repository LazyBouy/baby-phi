// agent/mod.rs — baby-phi's extensible workspace
// This file and everything in src/agent/ is MUTABLE — baby-phi evolves here.
//
// The 6 base tools (bash, read_file, write_file, edit_file, list_files, search)
// and the 3 base providers (anthropic, openai, openrouter) live in src/core/kernel.rs
// and cannot be modified.
//
// To add a new tool: implement AgentTool and return it from extra_tools().
// To add a new provider: implement StreamProvider and wire it in config.toml.

use crate::core::AgentTool;

/// Returns any additional tools baby-phi has developed.
/// These are added to the base tool list at runtime.
pub fn extra_tools() -> Vec<Box<dyn AgentTool>> {
    vec![]
}
