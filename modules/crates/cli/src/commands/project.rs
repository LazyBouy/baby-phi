//! `phi project` — project-management subcommands (M4).
//!
//! **Shape at M4/P1:** four subcommands (`list`, `show`, `create`,
//! `update-okrs`) are fully parsed by clap and registered in shell
//! completions but return
//! [`EXIT_NOT_IMPLEMENTED`](crate::exit::EXIT_NOT_IMPLEMENTED) until
//! M4/P6 (create) and M4/P7 (list/show/update-okrs) land the server
//! business logic.
//!
//! ## phi-core leverage
//!
//! Q1 **none** — this module introduces no `use phi_core::…` imports.
//! The project surface is pure phi governance (OKRs, project
//! shape, resource boundaries) with zero phi-core types in transit.

use std::path::PathBuf;

use crate::exit::EXIT_NOT_IMPLEMENTED;

/// Clap subcommand surface for `phi project`.
#[derive(Debug, clap::Subcommand)]
pub enum ProjectCommand {
    /// List projects in an org. **[M4/P7]**
    List {
        #[arg(long = "org-id")]
        org_id: String,
        /// Optional shape filter — `shape_a` or `shape_b`.
        #[arg(long)]
        shape: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show a single project by id. **[M4/P7]**
    Show {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Create a project (Shape A immediate or Shape B pending-approval). **[M4/P6]**
    Create {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        name: String,
        /// `shape_a` (single-org immediate) or `shape_b` (co-owned
        /// two-approver). Defaults to `shape_a`.
        #[arg(long, default_value = "shape_a")]
        shape: String,
        #[arg(long = "co-owner-org-id")]
        co_owner_org_id: Option<String>,
        #[arg(long = "lead-agent-id")]
        lead_agent_id: String,
        /// Comma-separated list of additional member agent ids.
        #[arg(long = "member-ids")]
        member_ids: Option<String>,
        /// Optional path to an OKR JSON file (matches the domain
        /// Objective + KeyResult shape; validated server-side).
        #[arg(long = "okrs-file")]
        okrs_file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Apply an OKR patch (create / update / delete Objectives and
    /// KeyResults in place). **[M4/P7]**
    UpdateOkrs {
        #[arg(long)]
        id: String,
        /// JSON patch payload — a `[{kind,op,payload}]` array matching
        /// the M4/P7 `PATCH /api/v0/projects/:id/okrs` contract.
        #[arg(long = "patch-json")]
        patch_json: String,
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(_server_url: Option<String>, cmd: ProjectCommand) -> i32 {
    match cmd {
        ProjectCommand::List { .. } => scaffold("project list", "M4/P7"),
        ProjectCommand::Show { .. } => scaffold("project show", "M4/P7"),
        ProjectCommand::Create { .. } => scaffold("project create", "M4/P6"),
        ProjectCommand::UpdateOkrs { .. } => scaffold("project update-okrs", "M4/P7"),
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
