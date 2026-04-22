<!-- Last verified: 2026-04-19 by Claude Code -->

# Operations — Docker deployment

The M0 Dockerfile at [`Dockerfile`](../../../../../../Dockerfile) produces a minimal, non-root, tini-initialised `phi-server` image. Kubernetes manifests and Helm charts are M7b deliverables; this page covers what ships today.

## Build context

The Dockerfile sits inside `phi/`, but it expects the build context to be the **parent directory** (`/root/projects/phi/`) so it can `COPY phi-core` alongside `phi`. The `phi-core` path dep in [`Cargo.toml`](../../../../../../Cargo.toml) resolves to `../phi-core`, and Docker's `COPY` can only see paths under the build context.

Two ways to build:

```bash
# From the parent that contains both phi-core/ and phi/
docker build -t phi-server:dev -f phi/Dockerfile .
```

```bash
# Or via docker-compose, which sets the right context automatically
cd phi
docker compose build server
```

## Stage 1 — `builder`

From [`Dockerfile`](../../../../../../Dockerfile):

```
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libclang-dev clang cmake protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*
COPY phi-core /build/phi-core
COPY phi /build/phi
WORKDIR /build/phi
RUN cargo build --release --package server
```

- `RUST_VERSION=1.95` is the minimum supported (blocked by `surrealdb-core` → `blake3` rustc requirement).
- `libclang-dev` + `clang` are required by `surrealdb-librocksdb-sys` via `bindgen`.
- `cmake` and `protobuf-compiler` are pulled in by `rustls` / `surrealdb-core` transitive deps.
- `cargo build --release --package server` builds only the server binary + its dependency graph. The CLI and tests are not built (smaller layer, faster).

Release-profile flags at [`Cargo.toml`](../../../../../../Cargo.toml):
```toml
[profile.release]
lto = true
codegen-units = 1
strip = "debuginfo"
```
These produce a significantly smaller binary at the cost of longer compile time — appropriate for release builds, not dev.

## Stage 2 — `runtime`

```
FROM debian:${DEBIAN_VERSION}-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates tini \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd --system --gid 10001 phi \
  && useradd  --system --uid 10001 --gid phi --home-dir /var/lib/phi phi \
  && mkdir -p /var/lib/phi/data /etc/phi \
  && chown -R phi:phi /var/lib/phi /etc/phi
```

- **Non-root user.** `phi` runs as UID 10001 in group GID 10001. A compromise in the server process cannot escalate to root inside the container.
- **ca-certificates.** Required for future outbound TLS (OAuth IdP calls in M3, audit-stream S3/GCS uploads in M7b).
- **tini.** PID 1 zombie reaper and signal forwarder. Without it, `SIGTERM` sent to the container might not reach the child process, and zombie children from any shell-out could accumulate.

Binary + config layering:
```
COPY --from=builder /build/phi/target/release/phi-server /usr/local/bin/phi-server
COPY phi/config/default.toml /etc/phi/config/default.toml
COPY phi/config/prod.toml    /etc/phi/config/prod.toml
```

Default profile for the image:
```
ENV PHI_PROFILE=prod \
    PHI_STORAGE__DATA_DIR=/var/lib/phi/data
```

Entry point:
```
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/phi-server"]
```

## Runtime contract

### Exposed port

`EXPOSE 8080`. The server listens on whatever `[server].host:[server].port` resolves to — by default `0.0.0.0:8080` in `default.toml` and inherited by `prod.toml`.

### Volume

`/var/lib/phi/data` — the SurrealDB RocksDB file tree. Must be a persistent volume:
- Docker: `-v phi-data:/var/lib/phi/data`
- Compose: declared as a named volume in [`docker-compose.yml`](../../../../../../docker-compose.yml).
- Kubernetes (M7b): PersistentVolumeClaim.

### Env vars at runtime

Consulted by [`ServerConfig::load()`](../../../../../../modules/crates/server/src/config.rs):

| Var | Typical prod value |
|---|---|
| `PHI_PROFILE` | `prod` (set by Dockerfile) |
| `PHI_STORAGE__DATA_DIR` | `/var/lib/phi/data` (set by Dockerfile) |
| `PHI_SERVER__PORT` | `8080` (inherited from `default.toml`) |
| `PHI_SERVER__HOST` | `0.0.0.0` |
| `PHI_TELEMETRY__LOG_FILTER` | `info` |
| `PHI_TELEMETRY__JSON_LOGS` | `true` |
| `PHI_SERVER__TLS__CERT_PATH` | (optional) path inside container |
| `PHI_SERVER__TLS__KEY_PATH` | (optional) path inside container |

See [configuration-profiles.md](configuration-profiles.md) for profile semantics.

### Healthcheck

```
HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/healthz/ready >/dev/null 2>&1 || exit 1
```

- Hits `/healthz/ready`, which probes storage.
- `start-period=10s` gives SurrealDB time to open the RocksDB file tree before the first probe.
- 3 consecutive failures mark the container unhealthy; orchestrators replace it.

If you enable native TLS, the healthcheck command needs adjustment — `wget --no-check-certificate -qO- https://127.0.0.1:8080/healthz/ready …`. The Dockerfile ships the plaintext-mode healthcheck by default because the recommended production posture is plaintext-on-localhost behind a reverse proxy.

## docker-compose

[`docker-compose.yml`](../../../../../../docker-compose.yml) declares two services:

| Service | Purpose |
|---|---|
| `server` | Builds from `Dockerfile` with context `..`. Publishes 8080. Mounts `phi-data` volume. |
| `web` | Runs the Next.js dev server via a `node:22-bookworm-slim` image, bind-mounting `./modules/web`. For local dev only. |

A full local stack is `docker compose up --build` (run from `phi/`). See [`../user-guide/docker-compose.md`](../user-guide/docker-compose.md) for the user-side view.

## Image size

The final image is a few hundred MB (debian-slim base + ~20 MB statically-linked server binary + tini + ca-certificates). A distroless or Alpine+musl variant could cut that further; M7b will measure and decide whether the operational trade-offs (libc compatibility, shell availability for debugging) justify the switch.

## Reverse-proxy posture (recommended)

The Dockerfile-only deploy is fine for a single-node install behind a reverse proxy. For production, the recommended topology is:

```
Internet ──▶ [nginx / Caddy / ALB terminating TLS] ──▶ [phi-server container, plaintext HTTP on internal net]
```

Native TLS (via `axum-server` + `PHI_SERVER__TLS__*`) is supported for simple single-node deploys where no reverse proxy is desired. See [tls-and-transport-security.md](tls-and-transport-security.md).

## What's NOT in the Dockerfile yet (`[PLANNED M7b]`)

- Distroless base image option.
- SBOM generation + signing (`cosign`).
- Kubernetes manifests (Deployment + Service + PVC + ConfigMap + Secret + Ingress + HPA).
- Helm chart.
- Multi-arch builds (linux/amd64 + linux/arm64).
