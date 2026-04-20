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

mod commands;

#[derive(Debug, Parser)]
#[command(
    name = "baby-phi",
    version,
    about = "baby-phi platform CLI",
    long_about = "baby-phi platform CLI. M1/P7 wires `bootstrap` subcommands + \
preserves the phi-core agent-loop demo under `agent demo`."
)]
struct Cli {
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
    };
    std::process::exit(code);
}
