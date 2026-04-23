//! `phi session` — first-session-launch subcommands (M5).
//!
//! **Shape at M5/P1:**
//!
//! - `phi session launch` — scaffold; ships at M5/P4.
//! - `phi session show` — scaffold; ships at M5/P4.
//! - `phi session terminate` — scaffold; ships at M5/P4.
//! - `phi session list` — scaffold; ships at M5/P4.
//!
//! The clap surface is scaffolded at M5/P1 so shell completions
//! (`phi completion <shell>`) name the four subcommands today — the
//! completion-regression test in `cli/tests/completion_help.rs`
//! tracks this invariant across milestones.
//!
//! Binary prefix is `phi` (never `baby-phi`) per the M2+ CLI naming
//! discipline; `phi session launch`, not `baby-phi session launch`.
//!
//! ## phi-core leverage
//!
//! Q1 at M5/P1 **none** — this file ships scaffolds only; the
//! `AgentEvent` tail renderer lands at M5/P7 (`launch` body) at which
//! point a single `use phi_core::types::event::AgentEvent;` may be
//! added for SSE deserialisation.

use crate::exit::EXIT_NOT_IMPLEMENTED;

/// Clap subcommand surface for `phi session`. All variants scaffold
/// to [`EXIT_NOT_IMPLEMENTED`] at M5/P1 and ship at M5/P4 (business
/// logic) + M5/P7 (SSE tail + `--detach` polish).
#[derive(Debug, clap::Subcommand)]
pub enum SessionCommand {
    /// Launch a first session. **[M5/P4 + P7]**
    ///
    /// Default tails events live via SSE until session end or
    /// SIGINT (which sends `terminate`). `--detach` returns the
    /// `session_id` + first `loop_id` immediately (per ADR-0031 D4).
    Launch {
        #[arg(long = "agent-id")]
        agent_id: String,
        #[arg(long = "project-id")]
        project_id: String,
        /// Initial prompt the session runs against.
        #[arg(long)]
        prompt: String,
        /// Return immediately without tailing events. Default is to
        /// tail live.
        #[arg(long)]
        detach: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a single session drill-down by id. **[M5/P4]**
    Show {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Terminate a running session. **[M5/P4]**
    Terminate {
        #[arg(long)]
        id: String,
        /// Optional reason — surfaced in the audit event +
        /// `SessionAborted` governance event.
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List sessions in a project. **[M5/P4]**
    List {
        #[arg(long = "project-id")]
        project_id: String,
        /// Show only sessions with `governance_state = running`.
        #[arg(long = "active-only")]
        active_only: bool,
        #[arg(long)]
        json: bool,
    },
}

/// Dispatcher for `phi session <cmd>`. Every arm returns
/// [`EXIT_NOT_IMPLEMENTED`] at M5/P1; the real wiring lands at
/// M5/P4 (HTTP handlers) + M5/P7 (CLI body + SSE tail).
pub async fn run(_server_url_override: Option<String>, cmd: SessionCommand) -> i32 {
    match cmd {
        SessionCommand::Launch { .. } => scaffold("session launch", "M5/P4 + P7"),
        SessionCommand::Show { .. } => scaffold("session show", "M5/P4"),
        SessionCommand::Terminate { .. } => scaffold("session terminate", "M5/P4"),
        SessionCommand::List { .. } => scaffold("session list", "M5/P4"),
    }
}

fn scaffold(cmd: &str, target_milestone: &str) -> i32 {
    eprintln!(
        "`phi {cmd}` is scaffolded but not yet wired to the server. \
         Implementation lands at {target_milestone}. \
         Retry once the release notes mark {target_milestone} as shipped.",
    );
    EXIT_NOT_IMPLEMENTED
}
