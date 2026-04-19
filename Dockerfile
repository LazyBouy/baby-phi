# Multi-stage image for baby-phi-server.
# - builder: compiles the Rust workspace in release mode
# - runtime: minimal debian-slim with a non-root user
#
# The Next.js web image is built separately from modules/web/Dockerfile (lands with M3).

ARG RUST_VERSION=1.95
ARG DEBIAN_VERSION=bookworm

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libclang-dev clang cmake protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*
# phi-core is a workspace dependency via ../phi-core — copy it alongside.
COPY phi-core /build/phi-core
COPY baby-phi /build/baby-phi
WORKDIR /build/baby-phi
RUN cargo build --release --package server

FROM debian:${DEBIAN_VERSION}-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates tini \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd --system --gid 10001 babyphi \
  && useradd  --system --uid 10001 --gid babyphi --home-dir /var/lib/baby-phi babyphi \
  && mkdir -p /var/lib/baby-phi/data /etc/baby-phi \
  && chown -R babyphi:babyphi /var/lib/baby-phi /etc/baby-phi

COPY --from=builder /build/baby-phi/target/release/baby-phi-server /usr/local/bin/baby-phi-server
COPY baby-phi/config/default.toml /etc/baby-phi/config/default.toml
COPY baby-phi/config/prod.toml    /etc/baby-phi/config/prod.toml

ENV BABY_PHI_PROFILE=prod \
    BABY_PHI_STORAGE__DATA_DIR=/var/lib/baby-phi/data
WORKDIR /etc/baby-phi
USER babyphi:babyphi
EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/healthz/ready >/dev/null 2>&1 || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/baby-phi-server"]
