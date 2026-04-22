<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — running locally

Three things can run locally in M0: the HTTP server, the CLI demo, and the web UI dev server. They're independent — you can run any combination.

## HTTP server (`phi-server`)

### Default (dev profile, plaintext, loopback)

```bash
cd /root/projects/phi/phi
PHI_PROFILE=dev /root/rust-env/cargo/bin/cargo run -p server
```

Expected log (pretty-formatted because dev profile sets `json_logs = false`):

```
INFO  opening SurrealDB, data_dir=data/phi-dev.db, namespace=phi, database=v0
INFO  phi-server listening (plaintext HTTP — terminate TLS at reverse proxy in prod), addr=127.0.0.1:8080
```

Probe it from another terminal:

```bash
curl http://127.0.0.1:8080/healthz/live
# {"status":"ok"}

curl http://127.0.0.1:8080/healthz/ready
# {"status":"ok","storage":"ok"}

curl http://127.0.0.1:8080/metrics | head
# # HELP axum_http_requests_total Total number of HTTP requests
# # TYPE axum_http_requests_total counter
# …
```

Press `Ctrl+C` to stop. SurrealDB's RocksDB files persist under `data/phi-dev.db/` — safe to delete between runs.

### Staging / prod profile

```bash
PHI_PROFILE=prod /root/rust-env/cargo/bin/cargo run -p server
```

Prod profile binds `0.0.0.0:8080`, writes JSON logs, uses `/var/lib/phi/data` (you may need to `mkdir -p /var/lib/phi/data && chown $USER`). See [`../operations/configuration-profiles.md`](../operations/configuration-profiles.md).

### Overriding ports

```bash
PHI_SERVER__PORT=9090 /root/rust-env/cargo/bin/cargo run -p server
```

### Native TLS (self-signed, for local testing)

```bash
# Generate a self-signed cert
openssl req -x509 -nodes \
    -subj '/CN=localhost' \
    -keyout key.pem -out cert.pem \
    -newkey rsa:2048 -days 3650

# Point the server at it
PHI_SERVER__TLS__CERT_PATH=$PWD/cert.pem \
PHI_SERVER__TLS__KEY_PATH=$PWD/key.pem \
/root/rust-env/cargo/bin/cargo run -p server

# Probe
curl -k https://127.0.0.1:8080/healthz/live
```

For production TLS guidance, see [`../operations/tls-and-transport-security.md`](../operations/tls-and-transport-security.md).

## CLI demo (`phi`)

The M0 CLI is the legacy phi-core demo — it runs a single agent loop against an OpenRouter model. Clap-based subcommands hitting the HTTP API arrive in M1+.

```bash
# One-time: populate .env
cp .env.example .env
# Edit .env and fill in OPENROUTER_API_KEY

# Run
cd /root/projects/phi/phi
set -a && source .env && set +a
/root/rust-env/cargo/bin/cargo run -p cli
```

Optional: pass a custom prompt as the first argument.

```bash
/root/rust-env/cargo/bin/cargo run -p cli -- "Write a one-paragraph release note for phi v0.1."
```

Output streams to stdout; the completed session is saved under `workspace/session/` per the config at [`config.toml`](../../../../../../config.toml).

## Web UI (Next.js dev server)

```bash
cd /root/projects/phi/phi/modules/web
npm install         # first time only; idempotent
npm run dev
```

Opens on `http://localhost:3000`. The single M0 page renders the server's `/healthz/live` + `/healthz/ready` JSON — useful to verify the full stack works end-to-end.

If the server isn't running, the web page will still render, but the health probe will error out and show the error message. This is intentional — M0's page is for connectivity verification, not a real UI.

### Pointing the web UI at a different server

```bash
PHI_API_URL=http://192.168.1.50:8080 npm run dev
```

The URL is read at request time by [`lib/api.ts`](../../../../../../modules/web/lib/api.ts) and by [`next.config.mjs`](../../../../../../modules/web/next.config.mjs)'s rewrite.

## Running all three together

Open three terminals and run each command above. Each listens on its own port:

| Surface | Default port |
|---|---|
| `phi-server` | 8080 |
| `modules/web` dev server | 3000 |
| CLI demo | no listener; agent loop streams to stdout |

Alternatively, use `docker compose up --build` for a containerized version — see [docker-compose.md](docker-compose.md).

## Stopping

- **Server:** `Ctrl+C`. `tokio::net::TcpListener` is dropped cleanly; in-flight requests are not drained in M0 (graceful shutdown is `[PLANNED M7b]`).
- **CLI:** `Ctrl+C` or wait for the agent loop to finish.
- **Web dev server:** `Ctrl+C`. Next.js cleans up its watchers.

## Resetting local state

```bash
# Wipe the dev SurrealDB
rm -rf data/phi-dev.db

# Wipe the CLI session history
rm -rf workspace/session

# Clear Rust build artefacts (last resort; expensive to rebuild)
cargo clean

# Clear Next.js build cache
rm -rf modules/web/.next
```
