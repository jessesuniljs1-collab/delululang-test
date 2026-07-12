//! Phase 5f — the `Custody` trait: the seam between the runtime and *where authority lives*.
//!
//! Stage 1–4 enforced capability scopes host-side, in-process, against the grant data carried by
//! `RootVal`/`CapScope` (still the reference semantics — playbook §1). Stage 5 moves that authority
//! decision behind a trait so the SAME interpreter and WASM host can run against either:
//!
//! - [`EmbeddedCustody`] — dev mode. `check`/`expose` are pass-throughs: the existing in-process
//!   scope checks in `prim.rs` remain the enforcement, so the entire prior conformance suite passes
//!   UNMODIFIED (criterion 11). Labelled `custody: embedded` (invariant 23 does not hold here).
//! - `BrokerClientCustody` (in the `delulu` CLI crate, over IPC) — daemon mode. Every effectful op is
//!   authorized by a broker in another process: synchronous-class ops (`FsWrite`/`Net`/`Declassify`/
//!   `ForeignBind`) round-trip per use; epoch-class ops (`FsRead`/`Clock`/`Rand`/`Console`) validate
//!   against a client-cached [`delulu_broker::Snapshot`] refreshed at most every `--epoch-ms`.
//!
//! The op classification and the epoch-snapshot validator are REUSED from `delulu-broker` — the two
//! custody impls duplicate no policy logic.

pub use delulu_broker::Op;

/// A refusal from custody, ready to become a `Fault` (the interpreter) or a host refusal (WASM).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustodyDenial {
    /// The stable diagnostic code (e.g. `DL1401` broker-unreachable, `DL1403` revoked, `DL0904`
    /// out-of-scope). `&'static` so it drops straight into `Fault { code, .. }`.
    pub code: &'static str,
    pub message: String,
}

impl CustodyDenial {
    pub fn new(code: &'static str, message: impl Into<String>) -> CustodyDenial {
        CustodyDenial { code, message: message.into() }
    }
}

/// The outcome of a custody `check` (spec §4.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustodyDecision {
    Allow,
    Deny(CustodyDenial),
}

/// The authority-decision seam (playbook §1). The interpreter and the WASM host call THIS instead of
/// reaching grant data directly.
pub trait Custody {
    /// Authorize one effectful op (spec §4.1 classes). The impl routes synchronous-class ops to a
    /// broker round-trip and epoch-class ops to the cached snapshot; the caller does not care which.
    fn check(&mut self, op: Op, arg: Option<&str>) -> CustodyDecision;

    /// Reveal a broker-held secret's bytes — a synchronous declassification (spec §4.4), audit-logged
    /// with the calling `span`. Embedded custody never routes here (local secrets reveal in-process);
    /// this is the daemon path where the bytes cross for the first time on `expose`.
    fn expose(&mut self, name: &str, span: Option<&str>) -> Result<String, CustodyDenial>;

    /// Refresh the cached revocation-epoch snapshot (no-op for embedded). The daemon client refreshes
    /// lazily on the next epoch-class check when the cache is older than `--epoch-ms`.
    fn refresh_epoch(&mut self);

    /// The custody label for `delulu authority`/`run` output: `"embedded"` or `"daemon"`.
    fn mode(&self) -> &'static str;
}

/// Embedded/dev custody: a pure pass-through. Every `check` allows (the in-process `prim.rs` scope
/// checks stay the enforcement — zero behavior change, criterion 11); `expose` is never called (the
/// interpreter reveals a local `SecretVal` directly). This is the default in [`crate::Interp::new`].
#[derive(Default)]
pub struct EmbeddedCustody;

impl EmbeddedCustody {
    pub fn new() -> EmbeddedCustody {
        EmbeddedCustody
    }
}

impl Custody for EmbeddedCustody {
    fn check(&mut self, _op: Op, _arg: Option<&str>) -> CustodyDecision {
        // Pass-through: embedded mode keeps the Stage 1–4 in-process enforcement in `prim.rs`.
        CustodyDecision::Allow
    }

    fn expose(&mut self, _name: &str, _span: Option<&str>) -> Result<String, CustodyDenial> {
        // Embedded secrets are local `SecretVal`s revealed in-process; the interpreter never routes
        // a local secret's `expose` through custody. Reaching here is a wiring bug, fail-closed.
        Err(CustodyDenial::new(
            "DL0904",
            "embedded custody does not hold broker secrets (internal wiring error)",
        ))
    }

    fn refresh_epoch(&mut self) {}

    fn mode(&self) -> &'static str {
        "embedded"
    }
}
