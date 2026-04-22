//! `baby-phi` CLI — operator-facing subcommands for the baby-phi platform.
//!
//! M1 ships two subcommand groups:
//!
//! - `baby-phi bootstrap {status,claim}` — exercises the
//!   `/api/v0/bootstrap/*` HTTP surface the server lands in P6.
//! - `baby-phi agent demo` — runs the legacy phi-core agent-loop demo
//!   that shipped pre-M1; preserved as a subcommand so we don't regress
//!   the prototype while M2+ adds real agent-management subcommands.
//!
//! Configuration precedence for the server URL:
//!
//!   1. `--server-url <URL>` flag
//!   2. `BABY_PHI_API_URL` environment variable
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
    name = "baby-phi",
    version,
    about = "baby-phi platform CLI",
    long_about = "baby-phi platform CLI. M1/P7 wires `bootstrap` subcommands + \
preserves the phi-core agent-loop demo under `agent demo`."
)]
pub struct Cli {
    /// Override the server base URL. If unset, falls back to
    /// `BABY_PHI_API_URL`, then to the layered `ServerConfig::load()`.
    #[arg(long, env = "BABY_PHI_API_URL", global = true)]
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
}

#[derive(Debug, Subcommand)]
enum BootstrapCommand {
    /// Probe `GET /api/v0/bootstrap/status` and print the current state.
    Status,
    /// Submit `POST /api/v0/bootstrap/claim` with the supplied
    /// credential + human identity.
    Claim {
        /// The `bphi-bootstrap-…` credential printed by
        /// `baby-phi-server bootstrap-init`.
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
enum AgentCommand {
    /// Run the phi-core demo agent loop against the legacy `config.toml`.
    Demo {
        /// Optional prompt override (defaults to the marketing-email
        /// prompt that shipped pre-M1).
        prompt: Option<String>,
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
        Command::Agent { cmd } => commands::agent::run(cmd).await,
        Command::Login { cmd } => commands::login::run(cmd).await,
        Command::Secret { cmd } => commands::secrets::run(cli.server_url, cmd).await,
        Command::ModelProvider { cmd } => commands::model_provider::run(cli.server_url, cmd).await,
        Command::McpServer { cmd } => commands::mcp_server::run(cli.server_url, cmd).await,
        Command::PlatformDefaults { cmd } => {
            commands::platform_defaults::run(cli.server_url, cmd).await
        }
        Command::Completion { shell } => commands::completion::run(shell),
        Command::Org { cmd } => commands::org::run(cli.server_url, cmd).await,
    };
    std::process::exit(code);
}
