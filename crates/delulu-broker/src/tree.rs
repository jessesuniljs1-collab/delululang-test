//! Phase 5b — the in-memory grant tree (spec §3): issue / attenuate / revoke / inspect / tree.
//!
//! Every grant is a [`Node`]; every node's authority `⊑` its parent (checked at [`Broker::attenuate`]
//! — violation is DL0802); revoking a node revokes its whole subtree (transitive, idempotent).
//!
//! **Holder-neutrality (spec criterion 9 / playbook trap 2).** A node's [`Holder`] is a descriptive
//! `{kind, desc, peer}` blob that is *stored and displayed, never switched on*. There is ZERO
//! `match`/`if` on `holder.kind` anywhere on any authority-decision path — the `holder_neutrality`
//! test in `tests/` greps this crate's own source to enforce it mechanically.

use std::collections::{HashMap, HashSet};

use crate::audit::{AuditEntry, AuditSink};
use crate::authority::{attenuation_check, Authority};
use crate::diag::Denial;
use crate::ids::{IdSource, OsIdSource};
use crate::time::{ClockSource, SystemClock};

/// An opaque, unguessable grant handle: `g_` + 32 hex chars (spec §3.1). Minted only by an
/// [`IdSource`]; never parsed from or derived from program data.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GrantId(pub(crate) String);

impl GrantId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Reconstruct a `GrantId` from a string already vouched for by an authenticated source (phase
    /// 5e: a node id read out of a token payload *after* its MAC verified; chunk 3: a node id the
    /// IPC client received from its own `issue`/`delegate` round-trip). Not a parse of untrusted
    /// program data — the broker still checks the node actually exists before trusting it
    /// (fail-closed, ruling 6). The client uses it only to key its own single-node epoch snapshot.
    pub fn from_trusted(s: impl Into<String>) -> GrantId {
        GrantId(s.into())
    }
}

impl std::fmt::Display for GrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// *Who* a node was issued/delegated to (spec §3.1 `holder`). Descriptive metadata only: stored,
/// audited, and displayed — NEVER inspected to make an authority decision (criterion 9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder {
    pub kind: String,
    pub desc: String,
    pub peer: String,
}

impl Holder {
    pub fn new(kind: impl Into<String>, desc: impl Into<String>, peer: impl Into<String>) -> Holder {
        Holder { kind: kind.into(), desc: desc.into(), peer: peer.into() }
    }
}

/// The stored lifecycle state of a node (spec §3.1 `state`). `Expired` is computed on demand from
/// the TTL + the pluggable clock (see [`Broker::effective_state`]); only `revoke` writes `Revoked`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Live,
    Revoked { by_seq: u64 },
}

/// The effective state of a node at a point in time: `Revoked` and TTL `Expired` folded together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffState {
    Live,
    Revoked { by_seq: u64 },
    Expired { ttl_millis: i64, now_millis: i64 },
}

/// A grant-tree node (spec §3.1).
#[derive(Clone, Debug)]
pub struct Node {
    pub id: GrantId,
    pub parent: Option<GrantId>,
    pub holder: Holder,
    pub authority: Authority,
    /// TTL as an absolute epoch-millis deadline, or `None` for no expiry.
    pub ttl_millis: Option<i64>,
    pub state: State,
    /// Creation time in epoch millis (from the broker's clock).
    pub created_millis: i64,
    pub audit_seq: u64,
}

/// The in-memory grant tree and its monotone counters (spec §3).
pub struct Broker {
    nodes: HashMap<GrantId, Node>,
    /// The revocation epoch: bumped on every revocation so cached snapshots can detect staleness
    /// (spec §4). Read by [`Broker::epoch`].
    epoch: u64,
    /// The next audit sequence number to assign. Ruling 5: one monotone counter now; every
    /// issue/attenuate/revoke/deny/synchronous-use consumes one, and chunk 2's hash-chained log
    /// will consume the SAME numbering — so records can be emitted later without renumbering.
    audit_seq: u64,
    ids: Box<dyn IdSource>,
    clock: Box<dyn ClockSource>,
    /// The highest reading this broker has ever taken from `clock` — the monotonicity ratchet
    /// (campaign finding P17-B2). See [`Broker::now`] for why it exists.
    clock_high_water: std::cell::Cell<i64>,
    /// Optional audit sink (phase 5d). `None` preserves chunk-1 behavior exactly; when attached,
    /// every seq-consuming op emits exactly one record (invariant 26). Observability, not enforcement
    /// (playbook trap 6): a sink write failure is logged, never allowed to change a decision.
    sink: Option<Box<dyn AuditSink>>,
    /// The broker MAC key for lease tokens (phase 5e), 256-bit. Lazily created at first use; the CLI
    /// injects one loaded from `~/.delulu/broker.key`. `None` until first delegate/redeem/rotate.
    key: Option<[u8; 32]>,
    /// Nonces of single-use tokens already redeemed (phase 5e). A second redemption of a single-use
    /// token whose nonce is here is DL1407. `--multi` tokens are neither checked nor recorded here.
    redeemed: HashSet<String>,
    /// Fingerprint → the local node it was adopted as (RFC 0001 F3/F4). Serves two jobs: it closes
    /// a revocation-evasion path (without it, `grants revoke` on an adopted node could be undone by
    /// simply presenting the same certificate again), and it is how a contact receipt finds the node
    /// it renews.
    adopted: HashMap<String, (GrantId, Option<i64>)>,
    /// Every certificate fingerprint in an adopted chain, keyed by the local node it was adopted as
    /// (finding P20-R4). Consulted on revoke to retire the WHOLE credential, not just the leaf.
    adopted_chain_fps: HashMap<GrantId, Vec<String>>,
    /// Fingerprints belonging to a chain whose adopted node has been revoked. Re-adopting any chain
    /// that contains one of these is refused. This closes the revocation-evasion that a single-adoption
    /// check on the LEAF alone leaves open (P20-R4): extending a revoked chain by one fresh
    /// self-delegation yields a new leaf, so the leaf check passes while the credential is unchanged —
    /// and every extension still contains the anchored root, so retiring the root's fingerprint stops
    /// them all.
    ///
    /// **ADOPT-REPLAY-1**: unlike every other field here, losing this set makes the broker *less*
    /// restrictive (a revoked certificate becomes re-adoptable), so it is the one piece of tree state
    /// the daemon persists across a restart (`revoked_certs.json`). `adopted` above is deliberately
    /// NOT persisted — clearing it on restart is correct, because it only gates re-adoption of a
    /// *live* credential, which after a wiped tree is the intended recovery path.
    revoked_adoption_fps: HashSet<String>,
    /// **ADOPT-REPLAY-1 fail-closed**: set when the persisted revoked-certificate denylist could not
    /// be read at startup (corrupt/tampered). While true, [`Broker::adopt`] refuses every certificate,
    /// because the broker can no longer prove a presented chain was not one it revoked. Mirrors the
    /// guard-policy `poisoned` posture: the daemon keeps running (revoke/inspect/e-stop, and local
    /// `issue` still work) but the specific surface whose safety depends on the lost state fails shut.
    adoptions_poisoned: bool,
    /// The Guard (Stage 5 chunk 6): policy, permits, pending requests, bypass flag, owner code —
    /// all daemon-memory only (the CLI injects a persisted policy + the print-once owner code). A
    /// default-constructed broker carries the default policy (declassify/foreign_c/foreign_python
    /// guarded); guard enforcement applies to delegated (non-root) nodes ONLY (addendum §2.1).
    pub(crate) guard: crate::guard::GuardState,
    /// Strict root-issuance mode (DISC-1). `Some(anchor_pubkey_hex)` ⇒ this broker refuses to create
    /// root authority from an unsigned [`Broker::issue_root`]; a root may enter ONLY by adopting a
    /// certificate chain that verifies against this **pinned** anchor (never a caller-supplied one).
    /// `None` ⇒ the legacy default (unsigned root issuance allowed). Only the anchor **public** half is
    /// held here — never the private key. The security of strict mode reduces to keeping the anchor
    /// **private** key, and this configuration, outside the same-uid adversary's reach: the broker
    /// enforces the mechanism, the deployment provides that custody (a category-7 boundary — see
    /// `docs/design/ROOT_ISSUANCE_TRUST_BOUNDARY.md`).
    strict_anchor: Option<String>,
}

/// The result of a [`Broker::revoke`] call: which nodes this call transitioned to `Revoked`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevokeOutcome {
    /// The audit seq stamped on every node revoked by this call.
    pub by_seq: u64,
    /// Nodes newly revoked by this call (empty on an idempotent no-op re-revoke).
    pub newly_revoked: Vec<GrantId>,
    /// The epoch after this call.
    pub epoch: u64,
}

impl Default for Broker {
    fn default() -> Self {
        Broker::new()
    }
}

impl Broker {
    /// A broker backed by OS randomness (GrantIds) and the system clock (TTL).
    pub fn new() -> Broker {
        Broker::with_sources(Box::new(OsIdSource), Box::new(SystemClock))
    }

    /// A broker with injected determinism sources (ruling 3): tests supply a counter id source and
    /// a manual clock so outcomes are reproducible with no OS entropy and no sleeps.
    pub fn with_sources(ids: Box<dyn IdSource>, clock: Box<dyn ClockSource>) -> Broker {
        Broker {
            nodes: HashMap::new(),
            epoch: 0,
            audit_seq: 1,
            ids,
            clock,
            clock_high_water: std::cell::Cell::new(i64::MIN),
            sink: None,
            key: None,
            redeemed: HashSet::new(),
            adopted: HashMap::new(),
            adopted_chain_fps: HashMap::new(),
            revoked_adoption_fps: HashSet::new(),
            adoptions_poisoned: false,
            guard: crate::guard::GuardState::new(),
            strict_anchor: None,
        }
    }

    /// Attach an audit sink (phase 5d). Additive (ruling 4): the default is no sink, so chunk-1 API
    /// and tests are unchanged. Chainable builder form.
    pub fn with_sink(mut self, sink: Box<dyn AuditSink>) -> Broker {
        self.sink = Some(sink);
        self
    }

    /// Attach/replace the audit sink after construction.
    pub fn set_sink(&mut self, sink: Box<dyn AuditSink>) {
        self.sink = Some(sink);
    }

    /// Inject the broker MAC key (phase 5e). The CLI loads/creates one at `~/.delulu/broker.key`
    /// (ruling 2 — only the CLI knows the path). Chainable builder form.
    pub fn with_key(mut self, key: [u8; 32]) -> Broker {
        self.key = Some(key);
        self
    }

    /// Set/replace the broker MAC key after construction.
    pub fn set_key(&mut self, key: [u8; 32]) {
        self.key = Some(key);
    }

    /// The current revocation epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Bump the epoch (a guard policy edit, addendum ruling 4: uses the same staleness mechanism as
    /// revocation, so a cached client snapshot refreshes within one epoch interval).
    pub(crate) fn bump_epoch(&mut self) {
        self.epoch += 1;
    }

    /// The next audit seq that will be assigned (for tests / introspection).
    pub fn next_audit_seq(&self) -> u64 {
        self.audit_seq
    }

    fn take_seq(&mut self) -> u64 {
        let s = self.audit_seq;
        self.audit_seq += 1;
        s
    }

    /// "Now" in epoch millis — **ratcheted so it can never move backwards** (P17-B2).
    ///
    /// # Why this is not simply `self.clock.now_millis()`
    ///
    /// Expiry compares a node's absolute deadline against this reading. The production clock is
    /// `SystemTime::now()` — a WALL clock, and a wall clock is not monotonic. Observed before this
    /// ratchet existed: a grant deadlined at t=5000 reported `Live` at 1000, `Expired` at 9000, and
    /// **`Live` again** once the clock was set back to 2000 — no revocation, no audit event, and
    /// nothing anywhere recording that authority had been restored.
    ///
    /// That is not an exotic scenario for the platforms this project targets. On satellites,
    /// autonomous aircraft and robots a backwards step is **routine, not adversarial**: GNSS time
    /// acquisition after a cold start, an NTP correction after drift, an RTC read at power-on. And
    /// the uplink lease (RFC 0001 F4) — the bound that exists precisely because revocation cannot
    /// cross a partition — is a wall-clock deadline.
    ///
    /// # Why a ratchet rather than a monotonic clock
    ///
    /// `Instant` cannot be used: certificate `not_before`/`not_after` are **signed absolute
    /// epoch-millis**, so the comparison must stay wall-clock-comparable or a certificate minted by
    /// the ground could not be evaluated here at all. Taking the running maximum keeps the reading
    /// comparable with signed times while making it non-decreasing:
    ///
    /// * a forward jump is accepted and advances the ratchet — correction still works;
    /// * a backward jump is clamped — **what expired stays expired.**
    ///
    /// The ratchet only ever *withholds* authority, never grants it: clamping upward can expire
    /// something early but can never un-expire it. Fail-closed in the direction that matters.
    fn now(&self) -> i64 {
        let reading = self.clock.now_millis();
        let ratcheted = reading.max(self.clock_high_water.get());
        self.clock_high_water.set(ratcheted);
        ratcheted
    }

    // ----- pub(crate) accessors for the `validate` module (which cannot see private fields) -----

    /// "Now" in epoch millis, via the injected clock.
    pub(crate) fn effective_now(&self) -> i64 {
        self.now()
    }

    /// Consume one audit seq (a synchronous-class use / deny records one — ruling 5).
    pub(crate) fn consume_seq(&mut self) -> u64 {
        self.take_seq()
    }

    /// Iterate the tree's nodes (used to build a [`crate::validate::Snapshot`]).
    pub(crate) fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// The MAC key, creating a random one on first use if none was injected (phase 5e — "created at
    /// first use"). Uses OS randomness. In the CLI the key is always injected from disk first, so a
    /// lazily-created key only ever appears in library/test use.
    pub(crate) fn ensure_key(&mut self) -> [u8; 32] {
        if let Some(k) = self.key {
            return k;
        }
        let mut k = [0u8; 32];
        getrandom::fill(&mut k).expect("OS randomness (getrandom) unavailable");
        self.key = Some(k);
        k
    }

    /// Whether this token nonce has already been redeemed (single-use tracking, phase 5e).
    pub(crate) fn is_redeemed(&self, nonce: &str) -> bool {
        self.redeemed.contains(nonce)
    }

    /// Record a token nonce as redeemed (single-use tracking, phase 5e).
    pub(crate) fn mark_redeemed(&mut self, nonce: &str) {
        self.redeemed.insert(nonce.to_string());
    }

    /// The node a certificate chain was adopted as, if it was (RFC 0001 F3/F4). Scoped to the
    /// process lifetime on purpose — see [`Broker::adopt`] for why that is both sufficient and
    /// necessary.
    /// Returns the node and the certificate window it was adopted under — the ceiling a contact
    /// receipt may not lift. Kept together with the node so a caller cannot supply the wrong one.
    pub(crate) fn adopted_node(&self, fingerprint: &str) -> Option<&(GrantId, Option<i64>)> {
        self.adopted.get(fingerprint)
    }

    /// Record a certificate chain as adopted: which node carries it, and its outer window.
    pub(crate) fn mark_adopted(&mut self, fingerprint: &str, node: &GrantId, window: Option<i64>) {
        self.adopted.insert(fingerprint.to_string(), (node.clone(), window));
    }

    /// Does this chain contain a certificate whose credential this broker has revoked (P20-R4)?
    /// If so, adopting it would restore revoked authority and must be refused.
    pub(crate) fn chain_hits_revoked_adoption(&self, chain_fps: &[String]) -> bool {
        chain_fps.iter().any(|fp| self.revoked_adoption_fps.contains(fp))
    }

    /// **ADOPT-REPLAY-1**: the revoked-certificate denylist, sorted, for the daemon to persist after a
    /// revoke. This is the only tree state whose *loss* weakens the broker, so it is the only one that
    /// crosses a restart. Sorted so the on-disk form is stable (a byte-for-byte no-op re-write when the
    /// set is unchanged).
    pub fn revoked_adoption_fps_snapshot(&self) -> Vec<String> {
        let mut v: Vec<String> = self.revoked_adoption_fps.iter().cloned().collect();
        v.sort();
        v
    }

    /// **ADOPT-REPLAY-1**: re-seed the revoked-certificate denylist at startup from the persisted set,
    /// so a revocation made in a previous daemon lifetime still refuses re-adoption. Additive: it only
    /// ever *adds* refusals, never grants authority.
    pub fn restore_revoked_adoption_fps(&mut self, fps: impl IntoIterator<Item = String>) {
        self.revoked_adoption_fps.extend(fps);
    }

    /// **ADOPT-REPLAY-1 fail-closed**: mark that the persisted denylist was unreadable, so every
    /// adoption is refused until an operator repairs it (see the field doc). One-way: nothing clears it
    /// within a lifetime.
    pub fn poison_adoptions(&mut self) {
        self.adoptions_poisoned = true;
    }

    /// Whether adoptions are poisoned (the denylist was unreadable at startup). Read by [`crate::cert`].
    pub(crate) fn adoptions_poisoned(&self) -> bool {
        self.adoptions_poisoned
    }

    /// Remember every certificate fingerprint in an adopted chain, so a later revoke of the node can
    /// retire the whole credential rather than only the leaf that was presented (P20-R4).
    pub(crate) fn record_adopted_chain(&mut self, node: &GrantId, chain_fps: Vec<String>) {
        self.adopted_chain_fps.insert(node.clone(), chain_fps);
    }

    /// Which local node an adopted certificate became — for reporting a renewal back to an
    /// operator. Read-only; the window stays private because only [`crate::cert`] may apply it.
    pub fn adopted_node_public(&self, fingerprint: &str) -> Option<&GrantId> {
        self.adopted.get(fingerprint).map(|(id, _)| id)
    }

    /// Move a node's TTL deadline **forward only** (RFC 0001 F4's contact receipt).
    ///
    /// Monotone by construction: a receipt can extend a lease and can never shorten one, so
    /// replaying an old receipt is a harmless no-op rather than a way to strip authority from a
    /// vehicle. Shortening is `revoke`'s job, and it is a different, audited operation.
    /// Returns the deadline in force afterwards.
    pub(crate) fn extend_ttl(&mut self, id: &GrantId, deadline: i64) -> Option<i64> {
        let n = self.nodes.get_mut(id)?;
        let now = match n.ttl_millis {
            Some(t) if t >= deadline => t,
            _ => {
                n.ttl_millis = Some(deadline);
                deadline
            }
        };
        Some(now)
    }

    /// Bind a node's holder `peer` to the redeeming party (phase 5e). Storage/display ONLY — never a
    /// decision input (criterion 9): this writes `holder.peer`, never reads `holder.kind`.
    pub(crate) fn set_holder_peer(&mut self, id: &GrantId, peer: impl Into<String>) {
        if let Some(n) = self.nodes.get_mut(id) {
            n.holder.peer = peer.into(); // KIND_IS_DATA: peer is descriptive metadata, never switched on
        }
    }

    /// Emit exactly one audit record for a seq-consuming op (invariant 26). No-op when no sink is
    /// attached (chunk-1 behavior). A sink write failure is logged and swallowed: the audit log is
    /// observability, not enforcement (playbook trap 6) — it must never change an outcome.
    ///
    /// (clippy: the 7 data args mirror the spec §7 record shape one-to-one; bundling them into a
    /// struct here would just duplicate `AuditEntry` minus `ts`. Internal helper — tradeoff is fine.)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_op(
        &mut self,
        seq: u64,
        action: &str,
        actor: Option<String>,
        target: Option<String>,
        authority: Option<serde_json::Value>,
        decision: &str,
        span: Option<String>,
    ) {
        if self.sink.is_none() {
            return;
        }
        let ts = self.now();
        let entry = AuditEntry {
            seq,
            ts,
            actor_node: actor,
            action: action.to_string(),
            target,
            authority,
            span,
            decision: decision.to_string(),
        };
        if let Some(sink) = self.sink.as_mut() {
            if let Err(e) = sink.append(entry) {
                eprintln!("delulu-broker: audit sink append failed (observability, not enforcement): {e}");
            }
        }
    }

    /// Record the broker's **effective root-issuance mode** in the audit log at startup.
    ///
    /// # Why this exists — detection where prevention is impossible (the category-7 residual)
    ///
    /// Strict anchored-root mode (DISC-1) is persisted in `<state>/root_policy.json`, which a
    /// same-OS-user process can edit or delete. No amount of code prevents that: to the kernel, that
    /// process and this one are the same principal. The honest response is not to pretend the file is
    /// a boundary, but to make crossing it **non-repudiable** — a downgrade that leaves a permanent,
    /// hash-chained trace is a very different thing from one that leaves a banner nobody read.
    ///
    /// So every start writes what mode it is actually running in. `delulu audit verify` already
    /// proves the chain has not been rewritten; with this record in it, "was this broker ever
    /// serving in legacy mode?" becomes a question the log can answer, and an attacker who downgrades
    /// must either leave the evidence or break chain verification, which is itself the alarm.
    ///
    /// Observability, not enforcement (invariant 26 / trap 6): no decision reads this record. As with
    /// [`Broker::record_plugin_signature`] it is a **no-op with no sink attached**, and then consumes
    /// no seq — so in-memory brokers and the existing accounting are byte-identical.
    pub fn record_root_policy_mode(&mut self, mode: &str, anchor: Option<&str>) {
        if self.sink.is_none() {
            return;
        }
        let seq = self.take_seq();
        self.record_op(
            seq,
            "root-policy-mode",
            None,
            Some(mode.to_string()),
            Some(serde_json::json!({ "mode": mode, "anchor": anchor })),
            "allow",
            None,
        );
    }

    /// Record a verified plugin **signature identity** in the audit log (Stage 6 phase 6h, spec §3.1
    /// step 6). `signer` is the ed25519 public key (lowercase hex). **No-op when no sink is
    /// attached** — and it then consumes NO seq, so a run that never signs plugins is byte-identical
    /// to chunk-1 accounting. Observability, not enforcement (invariant 26 / trap 6): no decision
    /// ever reads this record, and a signature authenticates origin, not behavior (spec §10).
    pub fn record_plugin_signature(&mut self, node: &GrantId, signer: &str) {
        if self.sink.is_none() {
            return;
        }
        let seq = self.take_seq();
        self.record_op(
            seq,
            "plugin-signature",
            Some(node.as_str().to_string()),
            None,
            Some(serde_json::json!({ "signed_by": signer })),
            "allow",
            None,
        );
    }

    /// Enable **strict root-issuance mode** (DISC-1): after this, [`Broker::issue_root`] refuses, and a
    /// root may enter ONLY via [`Broker::adopt`] of a chain that verifies against `anchor_pubkey_hex`
    /// — the pinned anchor overrides any caller-supplied anchor, so a same-uid client cannot substitute
    /// its own. There is deliberately **no IPC/CLI-runtime path to flip this off**: only in-process
    /// daemon-init code calls it, so a same-uid client cannot disable strict mode over the wire. The
    /// residual (a daemon restarted without the flag, or a tampered config file) is a deployment
    /// property named in `ROOT_ISSUANCE_TRUST_BOUNDARY.md` (category 7).
    pub fn require_anchored_roots(&mut self, anchor_pubkey_hex: String) {
        self.strict_anchor = Some(anchor_pubkey_hex);
    }

    /// The pinned anchor iff strict root-issuance mode is on, else `None`.
    pub fn strict_anchor(&self) -> Option<&str> {
        self.strict_anchor.as_deref()
    }

    /// The DISC-1 strict-mode invariant, checkable over live broker state: the ids of any **root**
    /// nodes (no parent) that did NOT enter via an adopted, anchor-verified chain. In strict mode this
    /// **must always be empty** — the only way a root can exist is [`Broker::adopt`], which records the
    /// node as `adopted`; a non-empty result means an unsigned root slipped in (a strict-mode
    /// violation). In the legacy default it is expected to be non-empty (unsigned roots are allowed),
    /// so it is meaningful as an assertion only under `require_anchored_roots`.
    pub fn unjustified_root_nodes(&self) -> Vec<GrantId> {
        let adopted_ids: HashSet<&GrantId> = self.adopted.values().map(|(id, _)| id).collect();
        self.nodes
            .values()
            .filter(|n| n.parent.is_none() && !adopted_ids.contains(&n.id))
            .map(|n| n.id.clone())
            .collect()
    }

    /// The **public** root-issuance API and the DISC-1 security boundary. In strict mode
    /// ([`Broker::require_anchored_roots`]) it refuses with `DL1421`: an unsigned root cannot be
    /// conjured — a root must be adopted from an anchor-verified certificate. In the legacy default it
    /// creates a root exactly as before. **Every** external caller (the daemon's `Issue` dispatch, and
    /// any library user) goes through here; the raw [`Broker::issue`] primitive is `pub(crate)` and is
    /// reached only after verification (by [`Broker::adopt`]), so there is no public bypass of the gate.
    pub fn issue_root(
        &mut self,
        holder: Holder,
        authority: Authority,
        ttl_millis: Option<i64>,
    ) -> Result<GrantId, Denial> {
        if let Some(anchor) = &self.strict_anchor {
            return Err(Denial::StrictRootRequiresAnchor { anchor: anchor.clone() });
        }
        Ok(self.issue(holder, authority, ttl_millis))
    }

    /// The raw root-creation primitive — `pub(crate)`, NOT a public API. Bypasses the strict-mode gate
    /// by construction, so it must be called only from a path that has already established the right to
    /// create a root: today that is [`Broker::adopt`] (after `verify_chain` against the pinned anchor)
    /// and in-crate tests. External root creation goes through [`Broker::issue_root`].
    pub(crate) fn issue(&mut self, holder: Holder, authority: Authority, ttl_millis: Option<i64>) -> GrantId {
        // Canonicalize at the custody boundary (P17-F1/F3). Path scopes have equivalence classes —
        // `./data` and `data` are one path under two names — so without this the same logical grant
        // enters the tree, and the hash chain, under two different hashes. Canonicalization
        // preserves the resolved path exactly, so it cannot change what this grant permits.
        let authority = authority.canonicalized();
        let id = self.ids.next_id();
        let seq = self.take_seq();
        let auth_json = authority.to_json();
        let node = Node {
            id: id.clone(),
            parent: None,
            holder,
            authority,
            ttl_millis,
            state: State::Live,
            created_millis: self.now(),
            audit_seq: seq,
        };
        self.nodes.insert(id.clone(), node);
        self.record_op(seq, "issue", Some(id.as_str().to_string()), None, Some(auth_json), "allow", None);
        id
    }

    /// Attenuate an existing lease into a child grant (spec §3.2 `attenuate`). The child is created
    /// iff `authority ⊑ parent` (else DL0802 carrying the intersection). Attenuating under a dead
    /// (revoked/expired) parent is refused fail-closed. Consumes one audit seq either way (ruling 5).
    pub fn attenuate(
        &mut self,
        parent: &GrantId,
        authority: Authority,
        holder: Holder,
        ttl_millis: Option<i64>,
    ) -> Result<GrantId, Denial> {
        let authority = authority.canonicalized();
        let auth_json = authority.to_json();
        let (seq, res) = self.attenuate_core(parent, authority, holder, ttl_millis);
        match &res {
            Ok(child) => self.record_op(
                seq,
                "attenuate",
                Some(parent.as_str().to_string()),
                Some(child.as_str().to_string()),
                Some(auth_json),
                "allow",
                None,
            ),
            Err(_) => self.record_op(
                seq,
                "attenuate",
                Some(parent.as_str().to_string()),
                None,
                Some(auth_json),
                "deny",
                None,
            ),
        }
        res
    }

    /// The record-free attenuation core (phase 5b enforcement). Returns the consumed audit seq and
    /// the result. `attenuate` wraps it to emit an `"attenuate"` record; `delegate` (phase 5e) wraps
    /// it to emit a single `"delegate"` record — so a delegation is never double-logged. Consumes
    /// exactly one audit seq in every branch (ruling 5), identical to chunk-1 accounting.
    pub(crate) fn attenuate_core(
        &mut self,
        parent: &GrantId,
        authority: Authority,
        holder: Holder,
        ttl_millis: Option<i64>,
    ) -> (u64, Result<GrantId, Denial>) {
        // The real chokepoint: `attenuate` canonicalizes before logging, but `delegate` reaches
        // this core directly. Canonicalizing here too makes "every node in the tree is canonical"
        // hold for both callers rather than for whichever one someone remembered. Idempotent, so
        // the doubled call on the `attenuate` path costs nothing.
        let authority = authority.canonicalized();
        let now = self.now();
        if !self.nodes.contains_key(parent) {
            let seq = self.take_seq(); // the deny still consumes a seq
            return (seq, Err(Denial::UnknownNode { node: parent.clone() }));
        }
        // Fail closed: no child may be born under a dead parent — or under a dead ANCESTOR, which
        // is a different question once expiry is inherited (RFC 0001 F4).
        let parent_eff = self
            .effective_state_inherited(parent, now)
            .unwrap_or(EffState::Expired { ttl_millis: 0, now_millis: now });
        let parent_node = self.nodes.get(parent).expect("presence checked immediately above");
        match parent_eff {
            EffState::Revoked { by_seq } => {
                let seq = self.take_seq();
                return (seq, Err(Denial::Revoked { node: parent.clone(), by_seq }));
            }
            EffState::Expired { ttl_millis, now_millis } => {
                let seq = self.take_seq();
                return (seq, Err(Denial::Expired { node: parent.clone(), ttl_millis, now_millis }));
            }
            EffState::Live => {}
        }
        // The ⊑ check — the mathematical heart (phase 5a).
        if let Err(intersection) = attenuation_check(&authority, &parent_node.authority) {
            let seq = self.take_seq();
            return (
                seq,
                Err(Denial::Attenuation {
                    holder: parent.clone(),
                    requested: Box::new(authority),
                    intersection: Box::new(intersection),
                }),
            );
        }
        let id = self.ids.next_id();
        let seq = self.take_seq();
        let node = Node {
            id: id.clone(),
            parent: Some(parent.clone()),
            holder,
            authority,
            ttl_millis,
            state: State::Live,
            created_millis: now,
            audit_seq: seq,
        };
        self.nodes.insert(id.clone(), node);
        (seq, Ok(id))
    }

    /// Revoke `target` on behalf of `caller` (spec §3.2). Allowed only if `target` is the caller's
    /// own node or a descendant of it (§3.2 — no upward/lateral reach). Transitive over the whole
    /// subtree; idempotent; bumps the epoch and consumes one audit seq, stamping `revoked_by_seq`
    /// on every node it transitions.
    pub fn revoke(&mut self, caller: &GrantId, target: &GrantId) -> Result<RevokeOutcome, Denial> {
        let (seq, res) = self.revoke_core(caller, target);
        let decision = if res.is_ok() { "allow" } else { "deny" };
        // Spec §4.2 (normative, playbook trap 3): the stated latency bound appears "in the audit
        // record of every revocation" — VERBATIM, and never a stronger claim. It rides in the
        // record's free-form JSON payload slot under a self-describing key (a revocation has no
        // authority payload of its own; flagged in spec §11 chunk-5 deviations).
        let payload = res
            .is_ok()
            .then(|| serde_json::json!({ "revocation_takes_effect": delulu_diag::REVOCATION_BOUND }));
        self.record_op(
            seq,
            "revoke",
            Some(caller.as_str().to_string()),
            Some(target.as_str().to_string()),
            payload,
            decision,
            None,
        );
        res
    }

    /// The record-free revocation core (phase 5b enforcement). Returns the consumed audit seq and the
    /// outcome; `revoke` wraps it to emit one `"revoke"` record. Consumes exactly one audit seq per
    /// call in every branch (ruling 5), identical to chunk-1 accounting.
    fn revoke_core(&mut self, caller: &GrantId, target: &GrantId) -> (u64, Result<RevokeOutcome, Denial>) {
        if !self.nodes.contains_key(caller) {
            let seq = self.take_seq();
            return (seq, Err(Denial::UnknownNode { node: caller.clone() }));
        }
        if !self.nodes.contains_key(target) {
            let seq = self.take_seq();
            return (seq, Err(Denial::UnknownNode { node: target.clone() }));
        }
        if !self.is_self_or_descendant(target, caller) {
            let seq = self.take_seq();
            return (seq, Err(Denial::NotRevocable { caller: caller.clone(), target: target.clone() }));
        }
        let seq = self.take_seq();
        self.epoch += 1;
        let subtree = self.subtree_inclusive(target);
        let mut newly_revoked = Vec::new();
        for id in subtree {
            if let Some(node) = self.nodes.get_mut(&id) {
                if matches!(node.state, State::Live) {
                    node.state = State::Revoked { by_seq: seq };
                    newly_revoked.push(id);
                }
            }
        }
        newly_revoked.sort();
        // P20-R4: revoking an adopted node retires its whole credential. Every certificate in the
        // adopted chain becomes un-re-adoptable, so re-presenting the chain extended by a fresh
        // self-delegation — which has a new leaf fingerprint and would otherwise slip past the leaf
        // single-adoption check — can no longer bring the revoked authority back.
        let retired: Vec<String> = newly_revoked
            .iter()
            .filter_map(|id| self.adopted_chain_fps.get(id))
            .flatten()
            .cloned()
            .collect();
        for fp in retired {
            self.revoked_adoption_fps.insert(fp);
        }
        (seq, Ok(RevokeOutcome { by_seq: seq, newly_revoked, epoch: self.epoch }))
    }

    /// Inspect a node by id (spec §3.2 `inspect`).
    pub fn inspect(&self, id: &GrantId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// All nodes, sorted by id — the `delulu grants list` read surface (spec §3.2, phase 5j).
    /// Read-only; a HUMAN/CLI operation at the top of the tree (a program's lease client never
    /// calls this — invariant 25 is about the lease holder's IPC reach, and the holder-facing
    /// custody client uses only its own node).
    pub fn nodes(&self) -> Vec<&Node> {
        let mut v: Vec<&Node> = self.nodes.values().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// The effective state of a node right now, folding revocation and TTL against the clock.
    pub fn effective_state(&self, id: &GrantId) -> Option<EffState> {
        // Inherited, so what an operator is SHOWN matches what the enforcement path decides. A
        // report that says "live" about a node the broker would refuse is worse than no report.
        self.effective_state_inherited(id, self.now())
    }

    /// The effective state of `id` **including its ancestors**: a node is live only if every node
    /// on the path to the root is live.
    ///
    /// # Why this walk exists (RFC 0001 F4, and a real hole it closes)
    ///
    /// [`effective_state`] judges one node. Revocation is transitive *at write time* (`revoke`
    /// marks the whole subtree), so that path was already covered — but **TTL expiry was not**, and
    /// `attenuate` bounds a child's authority by `⊑` without bounding its *deadline*. A holder
    /// could therefore delegate itself a child with `ttl_millis: None` and keep commanding after
    /// its own lease died.
    ///
    /// Locally that was a latent wrong; under federation it is load-bearing. The uplink lease
    /// (F4) is the only bound that survives a partition, and the party it bounds — the vehicle —
    /// is precisely the party that can mint children. A subtree that can outlive its root is a
    /// federated grant that cannot be timed out.
    ///
    /// Inheriting at *read* time rather than clamping at *write* time is deliberate: a contact
    /// receipt EXTENDS an adopted root's deadline, and the whole subtree must come with it.
    /// Clamping at creation would leave children dying at a deadline their parent no longer has.
    pub(crate) fn effective_state_inherited(&self, id: &GrantId, now: i64) -> Option<EffState> {
        let mut cur = Some(id.clone());
        // Parents always point at an already-existing node and ids are never reused, so a cycle is
        // unreachable; the bound is cheap insurance that fails CLOSED rather than spinning.
        for _ in 0..1024 {
            let Some(cid) = cur else { return Some(EffState::Live) };
            let n = self.nodes.get(&cid)?;
            match effective_state(n, now) {
                EffState::Live => {}
                dead => return Some(dead),
            }
            cur = n.parent.clone();
        }
        Some(EffState::Expired { ttl_millis: 0, now_millis: now })
    }

    /// Number of nodes in the tree (for tests).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    // ----- tree topology helpers -------------------------------------------

    /// Is `node` the same as `ancestor`, or a descendant of it? Walks parent pointers upward.
    fn is_self_or_descendant(&self, node: &GrantId, ancestor: &GrantId) -> bool {
        let mut cur = Some(node.clone());
        while let Some(id) = cur {
            if &id == ancestor {
                return true;
            }
            cur = self.nodes.get(&id).and_then(|n| n.parent.clone());
        }
        false
    }

    /// `target` and every transitive descendant (BFS over parent pointers). Deterministic order.
    fn subtree_inclusive(&self, target: &GrantId) -> Vec<GrantId> {
        let mut out = vec![target.clone()];
        let mut i = 0;
        while i < out.len() {
            let current = out[i].clone();
            let mut children: Vec<GrantId> = self
                .nodes
                .values()
                .filter(|n| n.parent.as_ref() == Some(&current))
                .map(|n| n.id.clone())
                .collect();
            children.sort();
            out.extend(children);
            i += 1;
        }
        out
    }

    // ----- serialization / rendering ---------------------------------------

    /// A **holder-neutral** custody-outcome projection: nodes sorted by id, each rendered as
    /// {id, parent, authority, state, ttl, created, audit_seq} — deliberately EXCLUDING the holder.
    ///
    /// This is what criterion 9 compares: running the same op sequence with `holder.kind` set to
    /// "human"/"process"/"delegate" yields byte-identical output here, because no authority decision
    /// ever depends on the holder. (Including the holder would trivially differ on the varied kind
    /// and prove nothing.) Canonical: sorted keys, sorted node order.
    pub fn outcome_json(&self) -> serde_json::Value {
        let mut ids: Vec<&GrantId> = self.nodes.keys().collect();
        ids.sort();
        let nodes: Vec<serde_json::Value> = ids
            .into_iter()
            .map(|id| {
                let n = &self.nodes[id];
                serde_json::json!({
                    "audit_seq": n.audit_seq,
                    "authority": n.authority.to_json(),
                    "created": n.created_millis,
                    "id": n.id.as_str(),
                    "parent": n.parent.as_ref().map(|p| p.as_str()),
                    "state": state_json(n.state),
                    "ttl": n.ttl_millis,
                })
            })
            .collect();
        serde_json::json!({ "epoch": self.epoch, "nodes": nodes })
    }

    /// A human-facing tree rendering (spec §3.2 `tree`). Includes the holder for display — this is
    /// the one place holder fields are read, and they are DATA, never a decision input.
    pub fn tree(&self) -> String {
        let mut roots: Vec<&GrantId> =
            self.nodes.values().filter(|n| n.parent.is_none()).map(|n| &n.id).collect();
        roots.sort();
        let mut out = String::new();
        for root in roots {
            self.render_node(root, 0, &mut out);
        }
        out
    }

    fn render_node(&self, id: &GrantId, depth: usize, out: &mut String) {
        let Some(n) = self.nodes.get(id) else { return };
        let indent = "  ".repeat(depth);
        let state = match effective_state(n, self.now()) {
            EffState::Live => "live".to_string(),
            EffState::Revoked { by_seq } => format!("revoked@{by_seq}"),
            EffState::Expired { .. } => "expired".to_string(),
        };
        // holder fields are display-only data (never a decision input).
        let holder_desc = &n.holder.desc; // KIND_IS_DATA: desc is descriptive metadata
        let holder_kind = &n.holder.kind; // KIND_IS_DATA: kind is stored/displayed, never switched on
        out.push_str(&format!(
            "{indent}{} [{}] {} ({}) — {}\n",
            n.id.as_str(),
            state,
            n.authority.render_compact(),
            holder_kind,
            holder_desc,
        ));
        let mut children: Vec<&GrantId> =
            self.nodes.values().filter(|c| c.parent.as_ref() == Some(id)).map(|c| &c.id).collect();
        children.sort();
        for child in children {
            self.render_node(child, depth + 1, out);
        }
    }
}

/// Fold a node's stored state and TTL into an effective state at `now`.
pub(crate) fn effective_state(node: &Node, now: i64) -> EffState {
    match node.state {
        State::Revoked { by_seq } => EffState::Revoked { by_seq },
        State::Live => match node.ttl_millis {
            Some(ttl) if now >= ttl => EffState::Expired { ttl_millis: ttl, now_millis: now },
            _ => EffState::Live,
        },
    }
}

fn state_json(state: State) -> serde_json::Value {
    // NOTE: the outcome projection serializes the STORED state (live/revoked); TTL expiry is
    // time-dependent and evaluated only in the check paths, so this stays clock-free and canonical.
    match state {
        State::Live => serde_json::json!("live"),
        State::Revoked { by_seq } => serde_json::json!({ "revoked": { "by_seq": by_seq } }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Scopes;
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use delulu_check::Effect;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
    fn det_broker() -> Broker {
        Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(Rc::new(ManualClock::new(1000))))
    }

    fn holder() -> Holder {
        Holder::new("process", "orchestrator.delulu", "pid:4711")
    }

    #[test]
    fn ids_are_g_prefixed_32_hex() {
        let mut b = Broker::new();
        let id = b.issue(holder(), Authority::default(), None);
        let s = id.as_str();
        assert!(s.starts_with("g_"), "id should start with g_: {s}");
        assert_eq!(s.len(), 34, "g_ + 32 hex");
        assert!(s[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn three_level_tree_revoke_middle_kills_subtree_root_lives() {
        let mut b = det_broker();
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Read", "Net"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }),
            None,
        );
        let mid = b
            .attenuate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }), holder(), None)
            .unwrap();
        let leaf = b
            .attenuate(&mid, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub/deep"]), ..Default::default() }), holder(), None)
            .unwrap();

        let out = b.revoke(&root, &mid).unwrap();
        let mut expected = vec![mid.clone(), leaf.clone()];
        expected.sort();
        assert_eq!(out.newly_revoked, expected, "revoke is transitive over the subtree");

        assert_eq!(b.effective_state(&root), Some(EffState::Live), "root stays live");
        assert!(matches!(b.effective_state(&mid), Some(EffState::Revoked { .. })), "middle revoked");
        assert!(matches!(b.effective_state(&leaf), Some(EffState::Revoked { .. })), "leaf revoked transitively");
    }

    #[test]
    fn attenuate_widening_rejected_with_correct_intersection() {
        let mut b = det_broker();
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }),
            None,
        );
        // Child asks for Net (not held) and a wider path.
        let err = b
            .attenuate(
                &root,
                Authority::new(eff(&["Read", "Net"]), Scopes { fs_read: names(&["./"]), ..Default::default() }),
                holder(),
                None,
            )
            .unwrap_err();
        assert_eq!(err.code(), "DL0802");
        let inter = err.intersection().expect("DL0802 carries an intersection");
        // Never widens: effects meet = {Read}, fs meet keeps only the parent's ./data.
        assert_eq!(inter.effects, eff(&["Read"]));
        assert_eq!(inter.scopes.fs_read, names(&["./data"]));
        assert!(!inter.scopes.net.contains("*"));
    }

    #[test]
    fn revoke_is_idempotent() {
        let mut b = det_broker();
        let root = b.issue(holder(), Authority::new(eff(&["Read"]), Scopes::default()), None);
        let child = b.attenuate(&root, Authority::new(eff(&["Read"]), Scopes::default()), holder(), None).unwrap();

        let first = b.revoke(&root, &child).unwrap();
        assert_eq!(first.newly_revoked, vec![child.clone()]);
        let first_seq = first.by_seq;
        let after_first = b.outcome_json();

        // Re-revoking changes no node state (idempotent) though it consumes a fresh seq/epoch.
        let second = b.revoke(&root, &child).unwrap();
        assert!(second.newly_revoked.is_empty(), "nothing newly revoked on re-revoke");
        // The child keeps its ORIGINAL revoking seq.
        assert!(matches!(b.inspect(&child).unwrap().state, State::Revoked { by_seq } if by_seq == first_seq));
        assert_eq!(after_first["nodes"], b.outcome_json()["nodes"], "node states unchanged");
    }

    #[test]
    fn revoke_of_non_descendant_is_refused() {
        let mut b = det_broker();
        let root = b.issue(holder(), Authority::new(eff(&["Read"]), Scopes::default()), None);
        let a = b.attenuate(&root, Authority::new(eff(&["Read"]), Scopes::default()), holder(), None).unwrap();
        let bnode = b.attenuate(&root, Authority::new(eff(&["Read"]), Scopes::default()), holder(), None).unwrap();
        // `a` cannot revoke its sibling `b` (no lateral reach, spec §3.2).
        let err = b.revoke(&a, &bnode).unwrap_err();
        assert_eq!(err.code(), "DL0904");
        assert!(matches!(err, Denial::NotRevocable { .. }));
        // `b` remains live.
        assert_eq!(b.effective_state(&bnode), Some(EffState::Live));
        // A child CAN revoke itself.
        assert!(b.revoke(&a, &a).is_ok());
    }

    #[test]
    fn cannot_attenuate_under_a_revoked_parent() {
        let mut b = det_broker();
        let root = b.issue(holder(), Authority::new(eff(&["Read"]), Scopes::default()), None);
        let mid = b.attenuate(&root, Authority::new(eff(&["Read"]), Scopes::default()), holder(), None).unwrap();
        b.revoke(&root, &mid).unwrap();
        let err = b
            .attenuate(&mid, Authority::new(eff(&["Read"]), Scopes::default()), holder(), None)
            .unwrap_err();
        assert_eq!(err.code(), "DL1403");
    }

    #[test]
    fn criterion_9_holder_kind_does_not_affect_the_outcome() {
        // The SAME op sequence with three different holder kinds must yield byte-identical custody
        // outcomes. Ids are injected deterministically (SeqIdSource), clock is fixed (ManualClock).
        fn run(kind: &str) -> String {
            let mut b = det_broker();
            let h = |k: &str| Holder::new(k, "desc", "peer");
            let root = b.issue(
                h(kind),
                Authority::new(eff(&["Read", "Net"]), Scopes { fs_read: names(&["./data"]), net: names(&["a.com"]), ..Default::default() }),
                None,
            );
            let c1 = b
                .attenuate(&root, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub"]), ..Default::default() }), h(kind), None)
                .unwrap();
            let _c2 = b
                .attenuate(&root, Authority::new(eff(&["Net"]), Scopes { net: names(&["a.com"]), ..Default::default() }), h(kind), None)
                .unwrap();
            let _c3 = b
                .attenuate(&c1, Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data/sub/x"]), ..Default::default() }), h(kind), None)
                .unwrap();
            // A rejected widening (consumes a seq, same for every kind).
            let _ = b.attenuate(&c1, Authority::new(eff(&["Net"]), Scopes::default()), h(kind), None);
            b.revoke(&root, &c1).unwrap();
            serde_json::to_string(&b.outcome_json()).unwrap()
        }
        let human = run("human");
        let process = run("process");
        let delegate = run("delegate");
        assert_eq!(human, process, "human vs process outcomes must be byte-identical");
        assert_eq!(process, delegate, "process vs delegate outcomes must be byte-identical");
    }
}
