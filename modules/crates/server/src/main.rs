use std::net::SocketAddr;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use clap::{Parser, Subcommand};
use domain::events::EventBus;
use server::bootstrap::generate_bootstrap_credential;
use server::{build_router, telemetry, with_prometheus, AppState, ServerConfig, SessionKey};
use store::SurrealStore;

/// phi-server — platform HTTP surface + one-shot bootstrap-init.
#[derive(Debug, Parser)]
#[command(name = "phi-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a single-use bootstrap credential and print it to stdout.
    ///
    /// The credential is hashed with argon2id and stored in the
    /// `bootstrap_credentials` table. The plaintext is printed **once**
    /// here and never persisted — if the admin loses it they must re-run
    /// the install (or use a manual admin override, out of scope for M1).
    BootstrapInit,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = ServerConfig::load()?;
    telemetry::init(&cfg.telemetry);

    match cli.command {
        Some(Command::BootstrapInit) => run_bootstrap_init(&cfg).await,
        None => run_server(cfg).await,
    }
}

async fn run_bootstrap_init(cfg: &ServerConfig) -> anyhow::Result<()> {
    tracing::info!(
        data_dir = %cfg.storage.data_dir.display(),
        "generating bootstrap credential",
    );
    let store = SurrealStore::open_embedded(
        &cfg.storage.data_dir,
        &cfg.storage.namespace,
        &cfg.storage.database,
    )
    .await?;
    let generated = generate_bootstrap_credential(&store).await?;
    // One-shot print to stdout. Deliberately plaintext; an admin copies
    // it into their own notes. We print to stdout (not the tracing log)
    // so it never goes into structured logs.
    println!();
    println!("============================================================");
    println!("BOOTSTRAP CREDENTIAL (save this — shown once):");
    println!();
    println!("  {}", generated.plaintext);
    println!();
    println!("Paste this into the /bootstrap page on first login.");
    println!("============================================================");
    tracing::info!(
        record_id = %generated.record_id,
        "bootstrap credential persisted (hashed)",
    );
    Ok(())
}

async fn run_server(cfg: ServerConfig) -> anyhow::Result<()> {
    tracing::info!(
        data_dir = %cfg.storage.data_dir.display(),
        namespace = %cfg.storage.namespace,
        database = %cfg.storage.database,
        "opening SurrealDB",
    );
    let store = SurrealStore::open_embedded(
        &cfg.storage.data_dir,
        &cfg.storage.namespace,
        &cfg.storage.database,
    )
    .await?;

    let session = SessionKey::from_config(&cfg.session)?;
    let master_key = Arc::new(store::crypto::MasterKey::from_env()?);
    let repo: Arc<dyn domain::Repository> = Arc::new(store);
    let audit: Arc<dyn domain::audit::AuditEmitter> =
        Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    // M4/P3: in-process event bus + Template A fire-listener. The
    // listener subscribes once; every `apply_project_creation` commit
    // emits `HasLeadEdgeCreated` on the bus → listener issues the lead
    // grant via the pure-fn + persists it.
    let event_bus = Arc::new(domain::events::InProcessEventBus::new());
    let template_a_listener = Arc::new(domain::events::TemplateAFireListener::new(
        repo.clone(),
        audit.clone(),
        Arc::new(server::platform::projects::RepoAdoptionArResolver::new(
            repo.clone(),
        )),
        Arc::new(server::platform::projects::RepoActorResolver::new(
            repo.clone(),
        )),
    ));
    event_bus.subscribe(template_a_listener);
    let state = AppState {
        repo,
        session,
        audit,
        master_key,
        event_bus,
    };
    let app = with_prometheus(build_router(state));

    let addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;

    match cfg.server.tls.as_ref() {
        Some(tls) => {
            tracing::info!(
                %addr,
                cert = %tls.cert_path.display(),
                key  = %tls.key_path.display(),
                "phi-server listening (TLS)",
            );
            let rustls = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path).await?;
            axum_server::bind_rustls(addr, rustls)
                .serve(app.into_make_service())
                .await?;
        }
        None => {
            tracing::info!(
                %addr,
                "phi-server listening (plaintext HTTP — terminate TLS at reverse proxy in prod)",
            );
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
