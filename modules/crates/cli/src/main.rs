//! `phi` CLI — operator-facing subcommands for the phi platform.
//!
//! M1 ships two subcommand groups:
//!
//! - `phi bootstrap {status,claim}` — exercises the
//!   `/api/v0/bootstrap/*` HTTP surface the server lands in P6.
//! - `phi agent demo` — runs the legacy phi-core agent-loop demo
//!   that shipped pre-M1; preserved as a subcommand so we don't regress
//!   the prototype while M2+ adds real agent-management subcommands.
//!
//! Configuration precedence for the server URL:
//!
//!   1. `--server-url <URL>` flag
//!   2. `PHI_API_URL` environment variable
//!   3. `{scheme}://{server.host}:{server.port}` from `ServerConfig::load()`
//!      (the same layered TOML stack the server uses; see
//!      [`server::config`]).

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

mod commands;
pub mod exit;
pub mod session_store;

#[derive(Debug, Parser)]
#[command(
    name = "phi",
    version,
    about = "phi platform CLI",
    long_about = "phi platform CLI. M1/P7 wires `bootstrap` subcommands + \
preserves the phi-core agent-loop demo under `agent demo`."
)]
pub struct Cli {
    /// Override the server base URL. If unset, falls back to
    /// `PHI_API_URL`, then to the layered `ServerConfig::load()`.
    #[arg(long, env = "PHI_API_URL", global = true)]
    server_url: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// System Bootstrap subcommands (platform-admin claim flow).
    Bootstrap {
        #[command(subcommand)]
        cmd: BootstrapCommand,
    },
    /// Run the bundled phi-core agent loop. Legacy demo retained for
    /// prototype continuity; real agent management lands in M2+.
    Agent {
        #[command(subcommand)]
        cmd: AgentCommand,
    },
    /// Session-management subcommands. M2/P1 ships `status` + `logout`;
    /// credential-based re-login lands with M2/P4.
    Login {
        #[command(subcommand)]
        cmd: commands::login::LoginCommand,
    },
    /// Credentials-vault subcommands (M2/P4).
    Secret {
        #[command(subcommand)]
        cmd: commands::secrets::SecretCommand,
    },
    /// Model-provider subcommands (M2/P5).
    ModelProvider {
        #[command(subcommand)]
        cmd: commands::model_provider::ModelProviderCommand,
    },
    /// MCP-server subcommands (M2/P6). Covers list/add/patch-tenants/archive.
    McpServer {
        #[command(subcommand)]
        cmd: commands::mcp_server::McpServerCommand,
    },
    /// Platform-defaults subcommands (M2/P7). Read / update the singleton
    /// row of execution + agent + context + retry defaults.
    PlatformDefaults {
        #[command(subcommand)]
        cmd: commands::platform_defaults::PlatformDefaultsCommand,
    },
    /// Emit shell-completion scripts (M2/P8). Delegates to
    /// `clap_complete::generate`; output goes to stdout. Offline.
    Completion {
        /// Target shell. Picked from clap_complete's supported set.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Organization subcommands (M3). `create`/`list`/`show` ship in
    /// M3/P4; `dashboard` ships in M3/P5. The clap surface is
    /// scaffolded in M3/P1 so shell completions name them today.
    Org {
        #[command(subcommand)]
        cmd: commands::org::OrgCommand,
    },
    /// Project subcommands (M4). `create` ships at M4/P6;
    /// `list`/`show`/`update-okrs` ship at M4/P7. The clap surface
    /// is scaffolded at M4/P1 so shell completions name them today.
    Project {
        #[command(subcommand)]
        cmd: commands::project::ProjectCommand,
    },
    /// Session subcommands (M5 / page 14 — First Session Launch).
    /// `launch` (default tails SSE / `--detach`) + `show` +
    /// `terminate` + `list` ship at M5/P4 (business logic) + M5/P7
    /// (CLI body). The clap surface is scaffolded at M5/P1 so shell
    /// completions name them today.
    Session {
        #[command(subcommand)]
        cmd: commands::session::SessionCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BootstrapCommand {
    /// Probe `GET /api/v0/bootstrap/status` and print the current state.
    Status,
    /// Submit `POST /api/v0/bootstrap/claim` with the supplied
    /// credential + human identity.
    Claim {
        /// The `bphi-bootstrap-…` credential printed by
        /// `phi-server bootstrap-init`.
        #[arg(long)]
        credential: String,
        /// Display name for the new platform admin.
        #[arg(long = "display-name")]
        display_name: String,
        /// Channel kind for the admin's first contact point.
        #[arg(long = "channel-kind", value_enum)]
        channel_kind: ChannelKindArg,
        /// Handle for that channel (e.g. `@alex`).
        #[arg(long = "channel-handle")]
        channel_handle: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    /// Run the phi-core demo agent loop against the legacy `config.toml`.
    Demo {
        /// Optional prompt override (defaults to the marketing-email
        /// prompt that shipped pre-M1).
        prompt: Option<String>,
    },
    /// List agents in an org. **[M4/P4]**
    List {
        #[arg(long = "org-id")]
        org_id: String,
        /// Optional role filter (`executive` / `admin` / `member` /
        /// `intern` / `contract` / `system`).
        #[arg(long)]
        role: Option<String>,
        /// Optional text search over display_name.
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show a single agent by id. **[M4/P4]**
    Show {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Create an agent (Human or LLM). **[M4/P5]**
    Create {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        name: String,
        /// `human` or `llm`.
        #[arg(long)]
        kind: String,
        /// Role per 6-variant enum (enforces `is_valid_for(kind)`
        /// server-side).
        #[arg(long)]
        role: String,
        /// Model-config id for LLM agents (required when `kind=llm`).
        #[arg(long = "model-id")]
        model_id: Option<String>,
        /// System-prompt override for the agent's blueprint.
        #[arg(long = "system-prompt")]
        system_prompt: Option<String>,
        /// Concurrent-session cap (default 1).
        #[arg(long, default_value_t = 1)]
        parallelize: u32,
        /// Optional ExecutionLimits override overrides — absent =
        /// inherit from org snapshot (ADR-0023); present = override
        /// (ADR-0027).
        #[arg(long = "override-max-turns")]
        override_max_turns: Option<usize>,
        #[arg(long = "override-max-tokens")]
        override_max_tokens: Option<usize>,
        #[arg(long = "override-max-duration-secs")]
        override_max_duration_secs: Option<u64>,
        #[arg(long = "override-max-cost")]
        override_max_cost: Option<f64>,
        #[arg(long)]
        json: bool,
    },
    /// Update an agent profile (diff-producing). **[M4/P5]**
    Update {
        #[arg(long)]
        id: String,
        /// JSON patch body matching the `PATCH
        /// /api/v0/agents/:id/profile` contract.
        #[arg(long = "patch-json")]
        patch_json: String,
        #[arg(long)]
        json: bool,
    },
    /// Revert an agent's ExecutionLimits to the org snapshot
    /// (DELETE the agent_execution_limits override row). **[M4/P5]**
    RevertLimits {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ChannelKindArg {
    Slack,
    Email,
    Web,
}

impl ChannelKindArg {
    fn as_wire(self) -> &'static str {
        match self {
            ChannelKindArg::Slack => "slack",
            ChannelKindArg::Email => "email",
            ChannelKindArg::Web => "web",
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Bootstrap { cmd } => commands::bootstrap::run(cli.server_url, cmd).await,
        Command::Agent { cmd } => commands::agent::run(cli.server_url, cmd).await,
        Command::Login { cmd } => commands::login::run(cmd).await,
        Command::Secret { cmd } => commands::secrets::run(cli.server_url, cmd).await,
        Command::ModelProvider { cmd } => commands::model_provider::run(cli.server_url, cmd).await,
        Command::McpServer { cmd } => commands::mcp_server::run(cli.server_url, cmd).await,
        Command::PlatformDefaults { cmd } => {
            commands::platform_defaults::run(cli.server_url, cmd).await
        }
        Command::Completion { shell } => commands::completion::run(shell),
        Command::Org { cmd } => commands::org::run(cli.server_url, cmd).await,
        Command::Project { cmd } => commands::project::run(cli.server_url, cmd).await,
        Command::Session { cmd } => commands::session::run(cli.server_url, cmd).await,
    };
    std::process::exit(code);
}
