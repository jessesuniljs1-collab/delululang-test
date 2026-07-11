//! `delulu-broker` — DeluluLang custody core (Stage 5, "Custody").
//!
//! Transport-free (ruling 1): this crate is the single source of truth for the grant *tree*, the
//! `⊑` attenuation lattice, and the two validation classes. It contains NO sockets, NO daemon, and
//! NO runtime wiring — those layers (chunks 2+) sit on top of these pure data structures. The
//! Stage-1 embedded broker (`delulu-runtime::broker`) stays as the embedded/dev path; this crate is
//! what the daemon (a later chunk) will own directly.
//!
//! Layout so far:
//! - [`authority`] — phase 5a: the `⊑` attenuation lattice and the never-widening intersection.
//! - [`tree`] — phase 5b: the in-memory grant tree (issue/attenuate/revoke/inspect/tree).
//! - [`diag`] — broker denials and their mapping to `delulu_diag::Diagnostic`.
//! - [`ids`] / [`time`] — the injectable GrantId source and TTL clock (ruling 3).
//!
//! (Phase 5c adds the synchronous/epoch validation classes.)

mod path;

pub mod authority;
pub mod diag;
pub mod ids;
pub mod time;
pub mod tree;

pub use authority::{attenuation_check, Authority, Scopes};
pub use diag::Denial;
pub use ids::{IdSource, OsIdSource, SeqIdSource};
pub use time::{ClockSource, ManualClock, SystemClock};
pub use tree::{Broker, EffState, GrantId, Holder, Node, RevokeOutcome, State};
