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

use delulu_broker::{Authority, Broker, GrantId, Holder};

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

    // ----- the grant tree (Stage 6): where a plugin's node is born and dies --------------------
    //
    // A plugin's grant is a CHILD NODE of the loading holder's node (invariant 28, R-7). Both
    // custody modes must serve this — embedded keeps an in-process `Broker`, the daemon client
    // routes `Attenuate`/`Revoke` over IPC — so the loader (`plugin.rs`) is written once against
    // this seam and works identically in either. The default impls FAIL CLOSED: a custody with no
    // grant tree confers nothing rather than inventing an allow.

    /// This run's own node — the *holder* a plugin's grant attenuates under (spec §3.1 step 4).
    fn holder_node(&self) -> Option<GrantId> {
        None
    }

    /// Attenuate the holder's node into a child grant (spec §3.1 step 4). `authority ⊑ holder` is
    /// enforced by the broker's `⊑` lattice; a wider request is DL0802 carrying the intersection.
    /// Returns the child's **fresh** `GrantId` (the R-6c binding).
    fn attenuate(&mut self, authority: Authority, holder: Holder) -> Result<GrantId, CustodyDenial> {
        let _ = (authority, holder);
        Err(CustodyDenial::new(
            "DL1401",
            "this run has no grant tree — plugin grants need custody that holds a node (fail closed)",
        ))
    }

    /// Revoke a node, transitively over its subtree (spec §3.3). Returns the **revoking audit
    /// seq** — the number DL0801 carries when a retained plugin reference is later called.
    fn revoke_node(&mut self, target: &GrantId) -> Result<u64, CustodyDenial> {
        let _ = target;
        Err(CustodyDenial::new(
            "DL1401",
            "this run has no grant tree — nothing to revoke (fail closed)",
        ))
    }

    /// Is a node still live, and if not, which audit seq killed it? (R-6c.) **Fail-closed**: the
    /// default — and any custody that cannot answer — reports [`Liveness::Unknown`], which every
    /// caller must treat as dead. An unknown node confers nothing.
    fn liveness(&self, target: &GrantId) -> Liveness {
        let _ = target;
        Liveness::Unknown
    }

    /// Record a verified plugin signature's identity in the audit log (spec §3.1 step 6). Additive;
    /// the **default is a no-op** — a custody with no audit sink records nothing, and the identity
    /// still travels on the returned plugin handle. Embedded custody writes an audit record when its
    /// in-process broker has a sink. (Signatures authenticate origin, not behavior — spec §10.)
    fn note_plugin_signature(&mut self, node: &GrantId, signer: &str) {
        let _ = (node, signer);
    }
}

/// A node's liveness for the R-6c per-call re-check. `Unknown` is not "maybe fine" — callers treat
/// it exactly as dead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liveness {
    Live,
    /// Revoked, carrying the revoking operation's audit seq — the number DL0801 reports, so an
    /// agent's error says *why and when* its authority died.
    Revoked(u64),
    /// The custody cannot answer. Treated as dead, always.
    Unknown,
}

/// Embedded/dev custody: a pure pass-through for effect checks. Every `check` allows (the
/// in-process `prim.rs` scope checks stay the enforcement — zero behavior change, criterion 11);
/// `expose` is never called (the interpreter reveals a local `SecretVal` directly). This is the
/// default in [`crate::Interp::new`].
///
/// **Stage 6:** embedded custody optionally carries an in-process grant **tree** so plugin loads
/// have a real holder node to attenuate under (invariant 28) — the same `delulu_broker::Broker` the
/// daemon owns, just in this process. [`EmbeddedCustody::new`] keeps `None` (byte-identical Stage
/// 1–5 behaviour); [`EmbeddedCustody::with_root`] issues the run's root node and enables the
/// plugin path.
#[derive(Default)]
pub struct EmbeddedCustody {
    /// The in-process grant tree, present only when this run may load plugins.
    tree: Option<Broker>,
    /// The run's own node in `tree` — the holder a plugin's grant attenuates under.
    node: Option<GrantId>,
}

impl EmbeddedCustody {
    /// Effect-check-only custody, with no grant tree (Stage 1–5 behaviour, unchanged).
    pub fn new() -> EmbeddedCustody {
        EmbeddedCustody { tree: None, node: None }
    }

    /// Embedded custody with an in-process grant tree rooted at `authority` — the run's own node.
    /// Plugin grants attenuate under it, so `⊑` (R-7) and transitive revocation are enforced by the
    /// real broker even in embedded mode.
    pub fn with_root(authority: Authority) -> EmbeddedCustody {
        let mut tree = Broker::new();
        let node = tree.issue(Holder::new("host", "embedded run", ""), authority, None);
        EmbeddedCustody { tree: Some(tree), node: Some(node) }
    }

    /// The in-process tree, for host-side inspection (`delulu grants tree` in embedded mode, tests).
    pub fn broker(&self) -> Option<&Broker> {
        self.tree.as_ref()
    }

    /// Re-root the holder: subsequent [`Custody::attenuate`] calls are checked against `node`'s
    /// authority instead of the run's own.
    ///
    /// This is how **R-7 composition** works: a plugin that loads a sub-plugin is itself the holder,
    /// so the sub-grant is checked against the *plugin's* node — never the host's. A plugin can
    /// therefore never hand its child more than it holds, at any depth (criterion 4).
    pub fn set_holder(&mut self, node: GrantId) {
        self.node = Some(node);
    }

    pub fn broker_mut(&mut self) -> Option<&mut Broker> {
        self.tree.as_mut()
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

    fn holder_node(&self) -> Option<GrantId> {
        self.node.clone()
    }

    fn attenuate(&mut self, authority: Authority, holder: Holder) -> Result<GrantId, CustodyDenial> {
        let (Some(tree), Some(node)) = (self.tree.as_mut(), self.node.clone()) else {
            return Err(CustodyDenial::new(
                "DL1401",
                "this embedded run has no grant tree — construct custody with `EmbeddedCustody::with_root` to load plugins (fail closed)",
            ));
        };
        // The SAME broker call the daemon makes: `⊑` is checked once, in one place (R-7).
        tree.attenuate(&node, authority, holder, None).map_err(denial_to_custody)
    }

    fn revoke_node(&mut self, target: &GrantId) -> Result<u64, CustodyDenial> {
        let (Some(tree), Some(node)) = (self.tree.as_mut(), self.node.clone()) else {
            return Err(CustodyDenial::new("DL1401", "this embedded run has no grant tree (fail closed)"));
        };
        tree.revoke(&node, target).map(|o| o.by_seq).map_err(denial_to_custody)
    }

    fn liveness(&self, target: &GrantId) -> Liveness {
        let Some(tree) = self.tree.as_ref() else { return Liveness::Unknown };
        match tree.effective_state(target) {
            Some(delulu_broker::EffState::Live) => Liveness::Live,
            Some(delulu_broker::EffState::Revoked { by_seq }) => Liveness::Revoked(by_seq),
            // An expired lease and an unknown node both confer nothing — dead, fail-closed.
            Some(delulu_broker::EffState::Expired { .. }) | None => Liveness::Unknown,
        }
    }

    fn note_plugin_signature(&mut self, node: &GrantId, signer: &str) {
        // Write the identity into the in-process audit log when a sink is attached (spec §3.1 step
        // 6). No-op with no tree/sink — observability, never enforcement (invariant 26 / trap 6).
        if let Some(tree) = self.tree.as_mut() {
            tree.record_plugin_signature(node, signer);
        }
    }
}

/// Map a broker [`delulu_broker::Denial`] to a [`CustodyDenial`], preserving its registered code.
/// The `&'static` code comes from the broker's own `code()` — no second code table.
fn denial_to_custody(d: delulu_broker::Denial) -> CustodyDenial {
    let diag = d.to_diagnostic();
    CustodyDenial { code: d.code(), message: diag.message }
}
