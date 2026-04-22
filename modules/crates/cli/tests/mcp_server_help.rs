//! Smoke tests for the `phi mcp-server …` clap surface.

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
fn mcp_server_group_lists_four_subcommands() {
    let help = run_help(&["mcp-server", "--help"]);
    for expected in ["list", "add", "patch-tenants", "archive"] {
        assert!(
            help.contains(expected),
            "`phi mcp-server --help` must mention `{expected}`; got:\n{help}"
        );
    }
}

#[test]
fn mcp_server_add_requires_display_name_endpoint_and_kind() {
    let help = run_help(&["mcp-server", "add", "--help"]);
    assert!(help.contains("--display-name"));
    assert!(help.contains("--endpoint"));
    assert!(help.contains("--kind"));
    assert!(help.contains("--tenants-allowed"));
}

#[test]
fn mcp_server_patch_tenants_requires_confirm_cascade() {
    let help = run_help(&["mcp-server", "patch-tenants", "--help"]);
    assert!(help.contains("--id"));
    assert!(help.contains("--tenants-allowed"));
    assert!(help.contains("--confirm-cascade"));
}

#[test]
fn mcp_server_archive_requires_id() {
    let help = run_help(&["mcp-server", "archive", "--help"]);
    assert!(help.contains("--id"));
}

#[test]
fn top_level_help_lists_mcp_server() {
    let help = run_help(&["--help"]);
    assert!(help.contains("mcp-server"));
}
