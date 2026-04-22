//! Smoke tests for the `phi platform-defaults …` clap surface.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_phi")
}

fn run_help(args: &[&str]) -> String {
    let out = Command::new(bin()).args(args).output().expect("run phi");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "help exit was non-zero: stdout=`{stdout}` stderr=`{stderr}`"
    );
    stdout
}

#[test]
fn platform_defaults_group_lists_three_subcommands() {
    let help = run_help(&["platform-defaults", "--help"]);
    for expected in ["get", "put", "factory"] {
        assert!(
            help.contains(expected),
            "`phi platform-defaults --help` must mention `{expected}`; got:\n{help}"
        );
    }
}

#[test]
fn platform_defaults_put_requires_file_and_if_version() {
    let help = run_help(&["platform-defaults", "put", "--help"]);
    assert!(help.contains("--file"));
    assert!(help.contains("--if-version"));
    assert!(help.contains("--format"));
}

#[test]
fn platform_defaults_get_offers_include_factory_and_format() {
    let help = run_help(&["platform-defaults", "get", "--help"]);
    assert!(help.contains("--include-factory"));
    assert!(help.contains("--format"));
}

#[test]
fn platform_defaults_factory_has_format_flag() {
    let help = run_help(&["platform-defaults", "factory", "--help"]);
    assert!(help.contains("--format"));
}

#[test]
fn top_level_help_lists_platform_defaults() {
    let help = run_help(&["--help"]);
    assert!(help.contains("platform-defaults"));
}

/// `factory --format json` should work without a running server —
/// it's the only subcommand with no HTTP call.
#[test]
fn factory_command_runs_offline() {
    let out = Command::new(bin())
        .args(["platform-defaults", "factory", "--format", "json"])
        .output()
        .expect("run phi");
    assert!(
        out.status.success(),
        "factory exit was non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The factory baseline carries every phi-core-wrapped field — so
    // at minimum, the output contains these four section names.
    for section in [
        "execution_limits",
        "default_agent_profile",
        "context_config",
        "retry_config",
    ] {
        assert!(
            stdout.contains(section),
            "factory output must contain `{section}` — phi-core field reuse check; got:\n{stdout}"
        );
    }
}
