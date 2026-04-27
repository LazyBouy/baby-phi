use std::net::SocketAddr;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use clap::{Parser, Subcommand};
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

/// Open the configured SurrealDB backend — either embedded RocksDB at
/// `cfg.storage.data_dir` (default) or remote per `cfg.storage.remote.uri`
/// when `cfg.storage.mode = "remote"` (CH-K8S-PREP P-2 / ADR-0033).
async fn open_store_for_config(cfg: &ServerConfig) -> anyhow::Result<SurrealStore> {
    use server::config::StorageMode;
    let store = match cfg.storage.mode {
        StorageMode::Embedded => {
            SurrealStore::open_embedded(
                &cfg.storage.data_dir,
                &cfg.storage.namespace,
                &cfg.storage.database,
            )
            .await?
        }
        StorageMode::Remote => {
            if cfg.storage.remote.uri.is_empty() {
                anyhow::bail!(
                    "storage.mode = remote requires storage.remote.uri (set \
                     PHI_STORAGE__REMOTE__URI=ws://... or update config)"
                );
            }
            SurrealStore::open_remote(
                &cfg.storage.remote.uri,
                &cfg.storage.namespace,
                &cfg.storage.database,
            )
            .await?
        }
    };
    Ok(store)
}

async fn run_bootstrap_init(cfg: &ServerConfig) -> anyhow::Result<()> {
    tracing::info!(
        data_dir = %cfg.storage.data_dir.display(),
        mode = ?cfg.storage.mode,
        "generating bootstrap credential",
    );
    let store = open_store_for_config(cfg).await?;
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
        mode = ?cfg.storage.mode,
        "opening SurrealDB",
    );
    let store = open_store_for_config(&cfg).await?;

    let session = SessionKey::from_config(&cfg.session)?;
    let master_key = Arc::new(store::crypto::MasterKey::from_env()?);
    let repo: Arc<dyn domain::Repository> = Arc::new(store);
    let audit: Arc<dyn domain::audit::AuditEmitter> =
        Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    // M4/P3 + M5/P3: in-process event bus with 5 listeners subscribed
    // (Template A + C + D fire listeners + memory-extraction stub +
    // agent-catalog stub). See `state::build_event_bus_with_m5_listeners`.
    let event_bus_impl =
        server::state::build_event_bus_with_m5_listeners(repo.clone(), audit.clone());
    let event_bus: Arc<dyn domain::events::EventBus> = event_bus_impl;
    // M5/P4: per-worker session registry (cancellation + concurrency cap).
    let session_registry = server::state::new_session_registry();
    let session_max_concurrent = cfg.session.max_concurrent;
    // Cloned for the SIGTERM drain — `state` moves into the router below.
    let registry_for_shutdown = std::sync::Arc::clone(&session_registry);
    // Cloned for the SIGTERM bus drain (CH-K8S-PREP P-4) — same Arc<dyn EventBus>
    // the live AppState holds; calling shutdown+drain on this clone affects
    // every emit() routed through the running handlers.
    let event_bus_for_shutdown: Arc<dyn domain::events::EventBus> = event_bus.clone();
    let shutdown_timeout = std::time::Duration::from_secs(cfg.shutdown.timeout_secs);
    let state = AppState {
        repo,
        session,
        audit,
        master_key,
        event_bus,
        session_registry,
        session_max_concurrent,
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
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let drain_for_signal = shutdown_timeout;
            tokio::spawn(async move {
                wait_for_shutdown_signal().await;
                tracing::info!("shutdown signal received — stopping TLS listener");
                handle_clone.graceful_shutdown(Some(drain_for_signal));
            });
            axum_server::bind_rustls(addr, rustls)
                .handle(handle)
                .serve(app.into_make_service())
                .await?;
        }
        None => {
            tracing::info!(
                %addr,
                "phi-server listening (plaintext HTTP — terminate TLS at reverse proxy in prod)",
            );
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(wait_for_shutdown_signal())
                .await?;
        }
    }

    // CH-K8S-PREP P-3 — after the HTTP listener stops accepting new
    // requests, drain in-flight agent-loop tasks (ADR-0031 §D31.5).
    // Each spawn task emits a final SessionEnded/SessionAborted via
    // the recorder; those emits are still delivered because
    // `event_bus.shutdown()` runs AFTER `graceful_shutdown(registry)`
    // (otherwise late session-finalisation events would be silently
    // dropped — see CHK8S-D-07).
    tracing::info!(
        timeout_secs = cfg.shutdown.timeout_secs,
        live_sessions = registry_for_shutdown.len(),
        "draining live agent_loop tasks before exit",
    );
    match server::shutdown::graceful_shutdown(registry_for_shutdown, shutdown_timeout).await {
        Ok(()) => tracing::info!("graceful shutdown complete — all sessions drained"),
        Err(timeout) => tracing::warn!(
            remaining = timeout.remaining,
            "graceful shutdown timed out — some sessions still running; \
             M7b will hard-flip these to FailedLaunch on exit",
        ),
    }

    // CH-K8S-PREP P-4 — once spawn tasks have all finalised, signal
    // the event bus to drop any further emits (defensive — there
    // shouldn't be any after this point) and wait for any in-flight
    // emit tasks to complete. ADR-0033.
    event_bus_for_shutdown.shutdown().await;
    match event_bus_for_shutdown.drain(shutdown_timeout).await {
        Ok(()) => tracing::info!("event-bus drain complete — all in-flight emits finished"),
        Err(err) => tracing::warn!(
            remaining = err.remaining,
            "event-bus drain timed out — {err}",
        ),
    }

    Ok(())
}

/// Resolve when SIGTERM or SIGINT arrives. K8s sends SIGTERM at pod
/// termination; Ctrl-C in dev sends SIGINT. Per CH-K8S-PREP P-3 /
/// ADR-0031 §D31.5.
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate())
        .expect("install SIGTERM handler — required for graceful shutdown");
    let mut sigint = signal(SignalKind::interrupt())
        .expect("install SIGINT handler — required for graceful shutdown");

    tokio::select! {
        _ = sigterm.recv() => tracing::info!("SIGTERM received"),
        _ = sigint.recv() => tracing::info!("SIGINT received"),
    }
}
