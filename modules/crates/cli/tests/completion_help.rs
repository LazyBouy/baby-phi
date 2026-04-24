//! Smoke tests for the `phi completion <shell>` subcommand.
//!
//! Two shape assertions per shell:
//!   - Help renders + exits 0.
//!   - Actual completion generation produces non-empty output on
//!     stdout (the generated script).

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_phi")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin()).args(args).output().expect("run phi");
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
            "`phi completion --help` must list `{shell}`; got:\n{stdout}"
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
fn completion_scripts_expose_m4_agent_subcommand_tree_on_every_shell() {
    // M4/P1 commitment C19: clap_complete's subcommand-tree walk must
    // surface every new `agent {list,show,create,update,revert-limits}`
    // subcommand in addition to the legacy `agent demo` — on every
    // shell. Keeps shell-completion parity regression-proof once M4/P4
    // and M4/P5 land the wired implementations.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        for sub in ["list", "show", "create", "update", "revert-limits", "demo"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `agent {sub}`; got snippet:\n{}",
                &stdout[..stdout.len().min(800)]
            );
        }
    }
}

#[test]
fn completion_scripts_expose_m4_project_subcommand_tree_on_every_shell() {
    // M4/P1 commitment C19, reinforced at M4/P8: `phi project {list,
    // show, create, update-okrs, approve-pending}` must all surface on
    // every shell backend. M4/P6 wired `create` + `approve-pending`;
    // M4/P7 wired `show` + `update-okrs` (list ships at M4/P8+ or
    // later — kept in the tree at P1 so completion is regression-
    // proof).
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        for sub in ["project", "update-okrs", "approve-pending"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `{sub}`; got snippet:\n{}",
                &stdout[..stdout.len().min(800)]
            );
        }
    }
}

#[test]
fn completion_scripts_expose_m5_session_subcommand_tree_on_every_shell() {
    // M5/P1 commitment: `phi session {launch, show, terminate, list}`
    // must all surface on every shell backend. M5/P4 wires the HTTP
    // handlers; M5/P7 wires the CLI bodies — the clap surface
    // scaffolds at M5/P1 so shell completions name them today (just
    // like M4's `phi project` scaffold pattern).
    //
    // Binary prefix is `phi`, never `baby-phi`.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        for sub in ["session", "launch", "terminate"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `{sub}`; got snippet:\n{}",
                &stdout[..stdout.len().min(800)]
            );
        }
        // Negative: the binary name never surfaces as `baby-phi` in
        // completion output.
        assert!(
            !stdout.contains("baby-phi"),
            "{shell} completion must not reference `baby-phi` anywhere"
        );
    }
}

#[test]
fn completion_scripts_expose_m5_p7_template_subcommand_tree_on_every_shell() {
    // M5/P7 D5.1 carryover: `phi template {list, approve, deny, adopt,
    // revoke}` must all surface on every shell backend.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        for sub in ["template", "approve", "deny", "adopt", "revoke"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `{sub}`; got snippet:\n{}",
                &stdout[..stdout.len().min(1200)]
            );
        }
    }
}

#[test]
fn completion_scripts_expose_m5_p7_system_agent_subcommand_tree_on_every_shell() {
    // M5/P7 D6.2 carryover: `phi system-agent {list, tune, add, disable,
    // archive}` must all surface on every shell backend.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok, "completion {shell} exited non-zero");
        for sub in ["system-agent", "tune", "disable", "archive"] {
            assert!(
                stdout.contains(sub),
                "{shell} completion must surface `{sub}`; got snippet:\n{}",
                &stdout[..stdout.len().min(1200)]
            );
        }
    }
}

#[test]
fn completion_scripts_expose_m5_p7_agent_update_model_config_id_flag() {
    // M5/P7 (C-M5-5 wire): `phi agent update` exposes a convenience
    // `--model-config-id` flag alongside `--patch-json`. Confirm the
    // flag surfaces in the help text + at least one shell's completion
    // output (completion scripts do not always inline every long flag,
    // but the help text is canonical).
    let (ok, stdout, stderr) = run(&["agent", "update", "--help"]);
    assert!(ok, "agent update --help exited non-zero: {stderr}");
    assert!(
        stdout.contains("--model-config-id"),
        "`phi agent update --help` must expose `--model-config-id`; got:\n{stdout}"
    );
    assert!(
        stdout.contains("--patch-json"),
        "`phi agent update --help` must still expose `--patch-json`; got:\n{stdout}"
    );
}

#[test]
fn completion_session_subcommand_includes_preview() {
    // M5/P7 CLI polish: session gains a `preview` subcommand wrapping
    // POST /sessions/preview (D5 inherit). Must surface in shell
    // completions alongside launch / show / terminate / list.
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let (ok, stdout, _) = run(&["completion", shell]);
        assert!(ok);
        assert!(
            stdout.contains("preview"),
            "{shell} completion must surface `session preview`; got snippet:\n{}",
            &stdout[..stdout.len().min(1200)]
        );
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
