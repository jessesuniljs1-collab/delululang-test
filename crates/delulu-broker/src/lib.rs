//! `delulu-broker` — DeluluLang custody core (Stage 5, "Custody").
//!
//! Transport-free (ruling 1): this crate is the single source of truth for the grant *tree*, the
//! `⊑` attenuation lattice, and the two validation classes. It contains NO sockets, NO daemon, and
//! NO runtime wiring — those layers (chunks 2+) sit on top of these pure data structures. The
//! Stage-1 embedded broker (`delulu-runtime::broker`) stays as the embedded/dev path; this crate is
//! what the daemon (a later chunk) will own directly.
//!
//! Layout:
//! - [`authority`] — phase 5a: the `⊑` attenuation lattice and the never-widening intersection.
//! - [`tree`] — phase 5b: the in-memory grant tree (issue/attenuate/revoke/inspect/tree).
//! - [`validate`] — phase 5c: the synchronous/epoch validation classes + revocation epochs.
//! - [`audit`] — phase 5d: the append-only, hash-chained audit log (blake3) + `verify`/`tail`/`query`.
//! - [`lease`] — phase 5e: MAC-signed portable lease tokens (delegate/redeem/rotate_key).
//! - [`secrets`] — phase 5g: the broker-resident secret store + `expose`/`secret_map`.
//! - [`diag`] — broker denials and their mapping to `delulu_diag::Diagnostic`.
//! - [`ids`] / [`time`] — the injectable GrantId source and TTL clock (ruling 3).

mod path;

pub mod audit;
pub mod authority;
pub mod diag;
pub mod ids;
pub mod lease;
pub mod secrets;
pub mod time;
pub mod tree;
pub mod validate;

pub use audit::{
    canonical_json, query, render_ts_utc, tail, verify, AuditEntry, AuditError, AuditLog,
    AuditRecord, AuditSink, MemSink, QueryFilter, VerifiedStats, GENESIS_HASH, OBSERVABILITY_PHRASE,
};
pub use authority::{attenuation_check, Authority, Scopes};
pub use diag::Denial;
pub use ids::{IdSource, OsIdSource, SeqIdSource};
pub use lease::{load_or_create_key, Token};
pub use secrets::SecretStore;
pub use time::{ClockSource, ManualClock, SystemClock};
pub use tree::{Broker, EffState, GrantId, Holder, Node, RevokeOutcome, State};
pub use validate::{Decision, Op, OpClass, Snapshot};

/// Crate-internal: pure-lexical "is `child` within `parent`?" (no filesystem access, ruling 2).
pub(crate) fn path_is_within(child: &str, parent: &str) -> bool {
    path::is_descendant_or_equal(child, parent)
}
