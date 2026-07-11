//! `delulu-broker` — DeluluLang custody core (Stage 5, "Custody").
//!
//! Transport-free (ruling 1): this crate is the single source of truth for the grant *tree*, the
//! `⊑` attenuation lattice, and the two validation classes. It contains NO sockets, NO daemon, and
//! NO runtime wiring — those layers (chunks 2+) sit on top of these pure data structures. The
//! Stage-1 embedded broker (`delulu-runtime::broker`) stays as the embedded/dev path; this crate is
//! what the daemon (a later chunk) will own directly.
//!
//! Phase 5a lands the `⊑` attenuation lattice ([`authority`]); the grant tree and validation
//! classes arrive in phases 5b and 5c.

mod path;

pub mod authority;

pub use authority::{attenuation_check, Authority, Scopes};
