//! System Bootstrap flow (s01).
//!
//! Two sub-modules:
//!
//! - [`init`] — generates and persists a single-use bootstrap credential
//!   on a fresh install (`phi-server bootstrap-init`). The plaintext
//!   is printed to stdout once; the hash (argon2id) is stored in the
//!   `bootstrap_credentials` table.
//! - [`claim`] — the atomic s01 flow invoked by
//!   `POST /api/v0/bootstrap/claim` (the HTTP handler lands in P6). Takes
//!   a validated claim request + `&dyn Repository` and commits every
//!   side-effect in one transaction (see `Repository::apply_bootstrap_claim`).
//!
//! Source of truth:
//! `docs/specs/v0/requirements/system/s01-bootstrap-template-adoption.md`
//! and `docs/specs/v0/requirements/admin/01-platform-bootstrap-claim.md`.

pub mod claim;
pub mod credential;
pub mod init;

pub use claim::{execute_claim, ClaimError, ClaimInput, ClaimOutcome, ClaimRejection};
pub use credential::{hash_credential, verify_credential, BOOTSTRAP_PREFIX};
pub use init::{generate_bootstrap_credential, GeneratedCredential};
