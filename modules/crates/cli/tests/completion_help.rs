//! Smoke tests for the `baby-phi completion <shell>` subcommand.
//!
//! Two shape assertions per shell:
//!   - Help renders + exits 0.
//!   - Actual completion generation produces non-empty output on
//!     stdout (the generated script).

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_baby-phi")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("run baby-phi");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn completion_help_lists_shell_value_enum() {
    let (ok, stdout, stderr) = run(&["completion", "--help"]);
    assert!(ok, "help exit non-zero: {stderr}");
    // clap_complete ships Bash, Zsh, Fish, PowerShell, Elvish as
    // supported shells. The clap-generated help lists all of them
    // in the `<SHELL>` argument's possible values.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        assert!(
            stdout.to_lowercase().contains(shell),
            "`baby-phi completion --help` must list `{shell}`; got:\n{stdout}"
        );
    }
}

#[test]
fn top_level_help_lists_completion_subcommand() {
    let (ok, stdout, _) = run(&["--help"]);
    assert!(ok);
    assert!(
        stdout.contains("completion"),
        "top-level help must mention `completion`; got:\n{stdout}"
    );
}

#[test]
fn completion_bash_emits_nonempty_script() {
    let (ok, stdout, stderr) = run(&["completion", "bash"]);
    assert!(ok, "completion bash exit non-zero: {stderr}");
    assert!(
        !stdout.is_empty(),
        "bash completion script must be non-empty"
    );
    // clap_complete's bash template sources through `complete -F
    // _<name>`; every script contains this directive.
    assert!(
        stdout.contains("complete -F"),
        "bash completion must register via `complete -F`; got snippet:\n{}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn completion_zsh_emits_nonempty_script() {
    let (ok, stdout, stderr) = run(&["completion", "zsh"]);
    assert!(ok, "completion zsh exit non-zero: {stderr}");
    assert!(!stdout.is_empty());
    // zsh completions start with `#compdef <name>`.
    assert!(
        stdout.contains("#compdef"),
        "zsh completion must start with #compdef; got snippet:\n{}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn completion_fish_emits_nonempty_script() {
    let (ok, stdout, stderr) = run(&["completion", "fish"]);
    assert!(ok, "completion fish exit non-zero: {stderr}");
    assert!(!stdout.is_empty());
    // fish completions are a series of `complete -c <name> …` lines.
    assert!(
        stdout.contains("complete -c"),
        "fish completion must use `complete -c`; got snippet:\n{}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn completion_bash_surfaces_org_dashboard_subcommand() {
    // Regression: clap_complete walks the subcommand tree at script
    // generation time. M3/P1 scaffolded `org dashboard`; M3/P5 wired
    // it. Completion must expose the subcommand so shell users can
    // tab-complete it.
    let (ok, stdout, _) = run(&["completion", "bash"]);
    assert!(ok);
    assert!(
        stdout.contains("dashboard"),
        "bash completion must surface `dashboard` subcommand; got snippet:\n{}",
        &stdout[..stdout.len().min(500)]
    );
}

#[test]
fn completion_scripts_expose_org_subcommand_tree_on_every_shell() {
    // M3/P6 commitment C14: clap_complete's subcommand-tree walk
    // must surface every `org {create,list,show,dashboard}` subcommand
    // on every shell. Keeps shell-completion parity regression-proof
    // when new subcommands land in later milestones.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        // Every shell backend inlines subcommand names verbatim into
        // its generated script, so a case-sensitive substring check
        // is sufficient across backends.
        for sub in ["create", "list", "show", "dashboard"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `org {sub}`; got \
                 snippet:\n{}",
                &stdout[..stdout.len().min(600)]
            );
        }
    }
}

#[test]
fn completion_powershell_emits_nonempty_script() {
    let (ok, stdout, stderr) = run(&["completion", "powershell"]);
    assert!(ok, "completion powershell exit non-zero: {stderr}");
    assert!(!stdout.is_empty());
    // PowerShell completions call `Register-ArgumentCompleter`.
    assert!(
        stdout.contains("Register-ArgumentCompleter"),
        "powershell completion must use Register-ArgumentCompleter; got snippet:\n{}",
        &stdout[..stdout.len().min(500)]
    );
}
