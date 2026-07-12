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
            sink: None,
            key: None,
            redeemed: HashSet::new(),
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

    /// The next audit seq that will be assigned (for tests / introspection).
    pub fn next_audit_seq(&self) -> u64 {
        self.audit_seq
    }

    fn take_seq(&mut self) -> u64 {
        let s = self.audit_seq;
        self.audit_seq += 1;
        s
    }

    fn now(&self) -> i64 {
        self.clock.now_millis()
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

    /// Issue a **root** grant (spec §3.2 `issue`, CLI-only human action — nothing programmatic
    /// creates root nodes, Constitution §5.16 law 4). Returns the new node's id.
    pub fn issue(&mut self, holder: Holder, authority: Authority, ttl_millis: Option<i64>) -> GrantId {
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
        let now = self.now();
        let parent_node = match self.nodes.get(parent) {
            Some(n) => n,
            None => {
                let seq = self.take_seq(); // the deny still consumes a seq
                return (seq, Err(Denial::UnknownNode { node: parent.clone() }));
            }
        };
        // Fail closed: no child may be born under a dead parent.
        match effective_state(parent_node, now) {
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
        self.record_op(
            seq,
            "revoke",
            Some(caller.as_str().to_string()),
            Some(target.as_str().to_string()),
            None,
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
        (seq, Ok(RevokeOutcome { by_seq: seq, newly_revoked, epoch: self.epoch }))
    }

    /// Inspect a node by id (spec §3.2 `inspect`).
    pub fn inspect(&self, id: &GrantId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// The effective state of a node right now, folding revocation and TTL against the clock.
    pub fn effective_state(&self, id: &GrantId) -> Option<EffState> {
        self.nodes.get(id).map(|n| effective_state(n, self.now()))
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
