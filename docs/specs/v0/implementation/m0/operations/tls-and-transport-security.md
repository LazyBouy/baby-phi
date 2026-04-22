<!-- Last verified: 2026-04-19 by Claude Code -->

# Operations — TLS and transport security

phi-server supports two TLS deployment patterns: a **reverse-proxy-in-front** (recommended) and a **native TLS listener** (simple single-node). Both are live in M0. Cert renewal hot-reload and mTLS-to-upstream are `[PLANNED M7b]`.

## Recommended — reverse-proxy terminates TLS

```
    Internet (HTTPS)
         │
         ▼
 ┌───────────────────────┐
 │  nginx / Caddy /      │   ← TLS termination, HSTS, HTTP/2, WAF rules
 │  AWS ALB / Cloudflare │     Let's Encrypt or ACM-managed cert renewal
 └──────────┬────────────┘
            │
            │  plaintext HTTP on an internal network
            ▼
 ┌───────────────────────┐
 │  phi-server      │   ← [server.tls] unset; plaintext listener
 │  (Docker container /  │     on 0.0.0.0:8080
 │   systemd service)    │
 └───────────────────────┘
```

Why this is the recommended posture:

- **Operational separation.** Cert renewal, HTTP/2, gzip, rate limits — all handled by the proxy. phi-server stays focused on its domain.
- **Renewal without restart.** ACME clients (Certbot, Caddy's built-in, AWS ACM) rotate certs transparently. Native TLS in M0 requires a process restart.
- **Edge features.** Reverse proxies give you CDN integration, geo-routing, DDoS protection, and WAF rules. phi-server can't replicate that.

Configuration for this posture: simply leave `[server.tls]` unset in the active profile (this is the default). The plaintext `axum::serve` path at [`main.rs:50-53`](../../../../../../modules/crates/server/src/main.rs) runs.

## Simple — native TLS

When a reverse proxy is overkill (e.g. a single-node on-prem deploy, a homelab, or during early customer demos), phi-server can serve TLS itself via `axum-server`.

### Enabling it

Add `[server.tls]` to the active profile OR inject via env:

```toml
# config/prod.toml or equivalent
[server.tls]
cert_path = "/etc/phi/tls/cert.pem"
key_path  = "/etc/phi/tls/key.pem"
```

Or:

```bash
export PHI_SERVER__TLS__CERT_PATH=/etc/phi/tls/cert.pem
export PHI_SERVER__TLS__KEY_PATH=/etc/phi/tls/key.pem
```

The server detects `Option::Some(tls)` at [`main.rs:35-48`](../../../../../../modules/crates/server/src/main.rs) and uses `axum_server::bind_rustls`.

### Cert format

PEM-encoded. Typical sources:

- **Let's Encrypt** via Certbot — produces `fullchain.pem` (use as `cert_path`) and `privkey.pem` (use as `key_path`).
- **Corporate CA** — your security team will hand you matching cert + key PEM files.
- **Self-signed for dev/homelab** — `openssl req -x509 -nodes -subj '/CN=your.host' -keyout key.pem -out cert.pem -newkey rsa:2048 -days 3650`.

### Renewal

v0.1: **restart required**. Update the PEM files, then restart `phi-server`. Loss of in-flight connections on restart; graceful-shutdown on `SIGTERM` completes existing requests before the new process binds.

Hot-reload of certificates without a restart is `[PLANNED M7b]`. Until then, a simple `systemctl restart phi-server` after a Certbot renew hook is the expected pattern.

### TLS version + cipher policy

Inherited from `rustls` (the backend under `axum-server`'s `tls-rustls` feature). rustls defaults:

- **Minimum TLS version:** 1.2 (rustls rejects 1.0 / 1.1).
- **Ciphers:** AEAD-only modern suites (TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256, TLS_AES_128_GCM_SHA256 for 1.3; ECDHE-based suites for 1.2).
- **Forward secrecy:** guaranteed by the cipher list.

Overriding the cipher list or forcing TLS-1.3-only is `[PLANNED M7b]` via a `[server.tls.min_version]` / `[server.tls.cipher_suites]` addition.

## Where native TLS is tested

[`modules/crates/server/tests/tls_test.rs`](../../../../../../modules/crates/server/tests/tls_test.rs) covers the native TLS code path end-to-end:

1. Generates a self-signed cert with `rcgen` at test time (no fixtures committed).
2. Writes PEM files to a temp dir.
3. Boots the router via `axum_server::bind_rustls` on a random port.
4. Asserts an HTTPS request succeeds.
5. Asserts a plaintext HTTP request to the same port fails with a protocol error.

The test installs rustls' `ring` crypto provider once at the top of the test function (`rustls::crypto::ring::default_provider().install_default()`) — rustls 0.23 requires an explicit provider when multiple could match.

## mTLS — `[PLANNED M7b]`

Mutual TLS (client-cert-authenticated requests) is planned for M7b as part of the hardening pass, particularly for:

- LLM-agent machine-to-machine auth over mTLS as an alternative to bearer tokens.
- Inter-service mTLS in Kubernetes deployments with service-mesh integrations.

Not in M0.

## HSTS, CSP, and other headers — `[PLANNED M2+]`

Security headers (HSTS, Content-Security-Policy, X-Frame-Options, Referrer-Policy, Permissions-Policy) are typically added at the reverse proxy in production. For deploys that rely on native TLS, a tower-http layer adding these headers will land as part of M2's first user-facing endpoints. M0 ships no security headers — the three M0 endpoints (`/healthz/*`, `/metrics`) are internal and have no browser-facing surface to protect.

## Operator checklist

Before turning on native TLS in production:

- [ ] Cert + key PEM files present, owned by the server process's uid, mode `0400` on the key.
- [ ] Cert matches the external DNS name the server is reached at.
- [ ] Cert renewal mechanism in place (cron + Certbot, or systemd timer, or AWS-managed).
- [ ] `PHI_SERVER__TLS__CERT_PATH` and `…KEY_PATH` set in the process environment (or in the layered config).
- [ ] Healthcheck command updated to `https://127.0.0.1:8080/healthz/ready` (or Docker's HEALTHCHECK rewritten with `--no-check-certificate`).
- [ ] Firewall lets inbound 8080 only from the intended sources.

Reverse-proxy deploys skip all of that — the proxy handles every item.
