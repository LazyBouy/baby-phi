<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — docker-compose

`docker compose up --build` boots the full baby-phi stack — HTTP server + Next.js dev server — in containers. Great when you want to exercise the production-like topology locally without installing a full Rust toolchain.

## Prerequisites

- Docker 24+.
- Docker Compose v2 (`docker compose` — two words — not the old `docker-compose` binary).
- Enough disk space for the builder stage (~3 GB) and the final image (~300 MB).

## Services

From [`docker-compose.yml`](../../../../../../docker-compose.yml):

| Service | Image | Ports | Notes |
|---|---|---|---|
| `server` | built from `Dockerfile` | `8080:8080` | Runs `baby-phi-server` with `BABY_PHI_PROFILE=dev`. Mounts named volume `baby-phi-data` at `/var/lib/baby-phi/data`. |
| `web` | `node:22-bookworm-slim` | `3000:3000` | Bind-mounts `./modules/web` and runs `npm install && npm run dev`. For **local dev only** — hot-reloads on file changes. |

## Boot

From the workspace root (`baby-phi/`):

```bash
docker compose up --build
```

First run:

- The `server` image builds — ~8–10 minutes for the Rust release build (LTO enabled, single codegen unit).
- The `web` service downloads Node 22 (~50 MB) and runs `npm install` into a volume.
- Both come up; server logs "baby-phi-server listening", web logs "Ready in …".

Subsequent boots reuse the built `server` image and the `web` node_modules volume — restart in ~10 s.

Probe the stack:

```bash
curl http://127.0.0.1:8080/healthz/ready
# {"status":"ok","storage":"ok"}

open http://localhost:3000    # or xdg-open / firefox / etc.
```

## Background mode

```bash
docker compose up --build -d        # detach
docker compose logs -f server       # follow server logs
docker compose logs -f web          # follow web logs
docker compose down                 # stop + remove containers
docker compose down -v              # also delete volumes (wipes DB)
```

## Volume story

- `baby-phi-data` (named volume) — SurrealDB's RocksDB file tree. Survives `docker compose down`; wiped with `down -v`.
- `web-node-modules` (named volume) — `node_modules` for the web dev server. Avoids rebuilding every boot; wiped with `down -v`.
- `./modules/web` — bind-mounted read-write into the web container. Code changes on the host trigger Next.js hot-reload inside the container.

## Env-var overrides

To change a server config field, set it in `docker-compose.yml` or pass via an env file:

```yaml
services:
  server:
    environment:
      BABY_PHI_PROFILE: dev
      BABY_PHI_SERVER__PORT: "9090"
      BABY_PHI_TELEMETRY__LOG_FILTER: "info,server=trace"
```

Or:

```bash
BABY_PHI_SERVER__PORT=9090 docker compose up --build
```

## Pointing the web at a different server

Compose's `depends_on: server` + internal Docker network lets the web service reach the server at `http://server:8080`. This is hard-coded in `docker-compose.yml` via `BABY_PHI_API_URL: http://server:8080`.

If you want the web to hit a server running on the host (not in compose), override:

```bash
BABY_PHI_API_URL=http://host.docker.internal:8080 docker compose up web
```

## When compose is the right tool

- Demoing the full stack without a Rust toolchain installed.
- Reproducing an environment for a bug report ("does it repro in the compose stack?").
- Running a team member's baby-phi server while you work on the web UI.

## When it isn't

- **Iterative Rust development** — the Docker build takes much longer than a local `cargo run -p server`. Use native runs.
- **Production** — the compose file is dev-oriented (profile `dev`, bind-mounts, dev server). Production uses the `Dockerfile` directly with Kubernetes or similar; see [`../operations/deployment-docker.md`](../operations/deployment-docker.md).
- **Performance testing** — compose adds Docker networking overhead; benchmark on bare metal.

## Troubleshooting

| Symptom | Likely cause |
|---|---|
| `Error response from daemon: pull access denied for node:22-bookworm-slim` | Docker not logged in or offline. `docker login`; check network. |
| Server container restarts repeatedly | Check `docker compose logs server`. Usually storage permission issue — the named volume's mount point has unexpected permissions. Try `docker compose down -v` to reset. |
| Web container fails on `npm install` | Clear the node-modules volume: `docker compose down -v && docker compose up --build`. |
| Port 8080 / 3000 already in use | Another local service is bound. Change the host-side port in the `ports:` block (e.g. `"18080:8080"`). |
| Builder runs out of memory | Rust release builds can need 4+ GB; raise Docker Desktop's RAM allocation, or build without LTO by editing `[profile.release]` in `Cargo.toml`. |

## What compose does NOT ship yet

- Reverse proxy (TLS termination) — planned for M7b.
- SurrealDB sidecar mode — v0.1 is embedded; standalone mode is a v0.2 conversation.
- Production-quality healthchecks (HTTP + log-level alerting) — current compose inherits the Dockerfile HEALTHCHECK; production setups add their own.
