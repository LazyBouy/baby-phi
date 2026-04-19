use std::net::SocketAddr;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use server::{build_router, telemetry, with_prometheus, AppState, ServerConfig};
use store::SurrealStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ServerConfig::load()?;
    telemetry::init(&cfg.telemetry);

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

    let state = AppState {
        repo: Arc::new(store),
    };
    let app = with_prometheus(build_router(state));

    let addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;

    match cfg.server.tls.as_ref() {
        Some(tls) => {
            tracing::info!(
                %addr,
                cert = %tls.cert_path.display(),
                key  = %tls.key_path.display(),
                "baby-phi-server listening (TLS)",
            );
            let rustls = RustlsConfig::from_pem_file(&tls.cert_path, &tls.key_path).await?;
            axum_server::bind_rustls(addr, rustls)
                .serve(app.into_make_service())
                .await?;
        }
        None => {
            tracing::info!(
                %addr,
                "baby-phi-server listening (plaintext HTTP — terminate TLS at reverse proxy in prod)",
            );
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
