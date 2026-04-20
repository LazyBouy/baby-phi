<!-- Last verified: 2026-04-20 by Claude Code -->

# Bootstrap credential — lifecycle

M1 ships the single-use bootstrap-credential flow described in
[architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md)
and [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md).
This page is the operator runbook — generation, delivery, consumption,
and the recovery paths when things go wrong.

## Generation (install-time)

On a fresh data directory, run:

```bash
baby-phi-server bootstrap-init
```

What happens under the hood
([`server/src/bootstrap/init.rs`](../../../../../../modules/crates/server/src/bootstrap/init.rs)):

1. 32 CSPRNG bytes from `rand::OsRng`.
2. Base64url-no-pad encode → prefix with `bphi-bootstrap-`.
3. Argon2id hash the result (PHC-encoded, includes its own salt).
4. Persist the hash via
   [`Repository::put_bootstrap_credential`](../../../../../../modules/crates/domain/src/repository.rs)
   — the plaintext is **never** written to storage or to logs.
5. Print the plaintext exactly once to stdout, inside a one-time
   banner.
6. Process exits.

The plaintext lives in exactly three places for that single moment:

- The admin's terminal scrollback (until they close the window).
- `/dev/stdout` of the `bootstrap-init` process.
- The admin's password manager (if they pasted it there).

It is never in a file, never in an env var, never in a process
memory dump of the running server.

## Delivery

The admin is responsible for capturing the plaintext. The recommended
flow:

1. Run `baby-phi-server bootstrap-init` at a real terminal (not in a
   CI log, not in a shared tmux session).
2. Copy the `bphi-bootstrap-…` line into a password manager entry
   named something like "baby-phi platform admin bootstrap — <date>".
3. Close the terminal.
4. Use the captured plaintext at first login (CLI / web / HTTP) to
   claim the admin role.

Stdout-only delivery is a deliberate anti-footgun design
([ADR-0011 §Delivery](../decisions/0011-bootstrap-credential-single-use.md#delivery--stdout-once-on-baby-phi-server-bootstrap-init))
— there is no operationally-valuable way to keep the plaintext on
disk between install-time and claim-time.

## Consumption

The single-use contract is enforced at the repository level: the
`bootstrap_credentials` row has a `consumed_at` column that the
[atomic `apply_bootstrap_claim`](../architecture/bootstrap-flow.md#atomicity--the-single-load-bearing-guarantee)
method stamps as part of the seven-writes transaction. Once
`consumed_at` is set, `execute_claim` rejects any attempt to reuse
the plaintext with `BOOTSTRAP_ALREADY_CONSUMED` (403).

If the atomic transaction fails for any reason (network partition,
storage error, agent-id collision), the credential **stays
unconsumed** and the admin may retry.

## Recovery paths

### Lost the plaintext before claiming

You closed the terminal without saving the plaintext. The server
holds only the argon2id hash, so brute-forcing the plaintext from
the hash is infeasible (a 256-bit random through argon2id — you'd
need millennia of commodity hardware).

Recovery:

- **Fresh install** is the only M1-supported path. Wipe
  `data_dir` and rerun `bootstrap-init`. This is acceptable
  because the platform hasn't done anything yet — no orgs, no
  agents, no data.
- **Manual DB override** is out of scope for M1. M7b may ship
  a `baby-phi admin regenerate-bootstrap-credential` flow.

### Suspect the plaintext leaked (e.g. committed to a terminal recording)

M1 does not ship in-band rotation. The credential is single-use so
the leak is bounded: once a legitimate admin claims, the credential
is burnt regardless. Mitigation:

1. If no admin has been claimed yet: claim immediately from a
   trusted machine. That invalidates the credential.
2. If an admin has already been claimed and the suspicion is about
   the hash leaking (e.g. a data-dir snapshot ended up somewhere
   it shouldn't): the hash is argon2id so brute-forcing the
   plaintext is infeasible; no action required.
3. If the suspicion is about the admin's session being
   compromised: M1 has no session revocation yet — the TTL
   (default 12h) is the only defence. A forced re-auth lands in
   M3 alongside OAuth.

### The claim succeeded but the admin is the wrong person

Scenario: the wrong human captured the stdout and beat the intended
admin to the claim form. Since the credential is consumed, there's
no way to re-run the first-install flow. Options:

- The current admin (the attacker) can't be demoted in M1 without
  a manual DB edit. Planned in M2 with the "transfer platform
  admin" admin flow.
- Practical mitigation: **reinstall** on a fresh data dir. All
  org / agent state is lost. Only acceptable because nothing
  valuable exists yet at first-install time.

### Multiple installs on the same host

Don't. Each `bootstrap-init` adds a row to `bootstrap_credentials`;
previous credentials remain in the table. `execute_claim` scans all
unconsumed credentials and consumes the first match. In practice
this means stale unconsumed credentials from a previous install are
still accepting — which is why the operator-facing advice is **one
install per data dir**, period. Reinstall means a brand-new data
dir.

## What M1 does NOT ship (deferred to M2+ / M7b)

- In-band rotation (`baby-phi admin rotate-bootstrap-credential`).
- Expiry on unconsumed credentials.
- Transfer-of-admin workflow.
- Break-glass admin recovery without reinstall.

## Cross-references

- [architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md)
  — the atomic s01 flow.
- [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md) —
  the decision record.
- [user-guide/first-bootstrap.md](../user-guide/first-bootstrap.md) —
  the admin-facing walkthrough.
- [requirements/admin/01](../../../requirements/admin/01-platform-bootstrap-claim.md)
  — the source-of-truth requirement spec.
