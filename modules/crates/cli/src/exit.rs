//! Named CLI exit codes — the stable contract shell scripts pin on.
//!
//! M1 established the 0–3 ladder (`EXIT_OK`, `EXIT_TRANSPORT`,
//! `EXIT_REJECTED`, `EXIT_INTERNAL`) inline in
//! [`commands::bootstrap`](super::commands::bootstrap). M2 extends it
//! with two new levels so page-04+ subcommands can distinguish richer
//! failure modes:
//!
//! - `EXIT_PRECONDITION_FAILED` (4) — a step that must precede this
//!   command hasn't happened (e.g. `baby-phi secret reveal` with no
//!   saved session, or `mcp-server patch` against an archived
//!   server). Shell scripts can use this to fail fast rather than
//!   retrying.
//! - `EXIT_CASCADE_ABORTED` (5) — a destructive cascade (e.g. an MCP
//!   tenant-narrowing that would revoke more grants than the user
//!   approved) was aborted mid-way. Distinct from `EXIT_REJECTED`
//!   because the rejection came after partial work, not from input
//!   validation.
//!
//! Hoisted into its own module so every M2+ subcommand can depend on
//! the same constants (no copy/paste drift).

/// Command completed successfully.
pub const EXIT_OK: i32 = 0;

/// Transport / IO failure — server unreachable, DNS, timeout.
/// Shell scripts should typically retry with backoff.
pub const EXIT_TRANSPORT: i32 = 1;

/// Server returned a 4xx with a stable, user-facing `code`. Fix the
/// input; don't retry with the same payload.
pub const EXIT_REJECTED: i32 = 2;

/// Server returned 5xx or an unexpected shape. Operator-investigable.
pub const EXIT_INTERNAL: i32 = 3;

/// A required precondition is missing (no saved session, referenced
/// entity is archived, etc.). New in M2 / P1.
pub const EXIT_PRECONDITION_FAILED: i32 = 4;

/// A destructive cascade (tenant-narrowing, bulk revoke) was aborted
/// after partial work. New in M2 / P1; used by `mcp-server patch`
/// starting at M2 / P6.
pub const EXIT_CASCADE_ABORTED: i32 = 5;
