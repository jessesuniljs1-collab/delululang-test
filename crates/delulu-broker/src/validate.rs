//! Phase 5c — the two validation classes + revocation epochs (spec §4).
//!
//! Every primitive-table operation is classed (spec §4.1):
//! - **Synchronous** — validated by a broker round-trip against LIVE tree state per use:
//!   `Declassify`/expose, `FsWrite`, `Net`, foreign bind (plugin-load and Actuate are reserved
//!   variants, unused in v0.5).
//! - **Epoch** — validated locally against a cached [`Snapshot`] the runtime refreshes every
//!   `--epoch-ms` (that refresh wiring is chunk 3): `FsRead`, `Clock`, `Rand`, `Console`.
//!
//! Liveness (spec §4.3): every op checks state ≠ Revoked/Expired. A revoked lease is DL1403 carrying
//! the revoking audit seq; a TTL-expired lease is DL1402 (checked against the pluggable clock).

use crate::authority::Authority;
use crate::diag::Denial;
use crate::tree::{effective_state, Broker, EffState, GrantId};
use delulu_check::Effect;

/// The validation class of an op (spec §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpClass {
    Synchronous,
    Epoch,
}

/// A primitive-table operation, for classification and scope validation (spec §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    // ----- synchronous class -----
    Declassify,
    FsWrite,
    Net,
    ForeignBind,
    /// Reserved variant (Stage 6 plugin loading); unused in v0.5.
    PluginLoad,
    /// Reserved variant (Stage 10 robotics); unused in v0.5.
    Actuate,
    // ----- epoch class -----
    FsRead,
    Clock,
    Rand,
    Console,
}

impl Op {
    pub fn class(self) -> OpClass {
        match self {
            Op::Declassify | Op::FsWrite | Op::Net | Op::ForeignBind | Op::PluginLoad | Op::Actuate => {
                OpClass::Synchronous
            }
            Op::FsRead | Op::Clock | Op::Rand | Op::Console => OpClass::Epoch,
        }
    }

    /// The effect this op requires in the node's authority (`None` for reserved/unused variants).
    fn required_effect(self) -> Option<Effect> {
        match self {
            Op::Declassify => Some(Effect::Declassify),
            Op::FsWrite => Some(Effect::Write),
            Op::Net => Some(Effect::Net),
            Op::ForeignBind => Some(Effect::ForeignCall),
            Op::FsRead => Some(Effect::Read),
            Op::Clock => Some(Effect::Clock),
            Op::Rand => Some(Effect::Rand),
            Op::Console => Some(Effect::Write),
            Op::PluginLoad | Op::Actuate => None,
        }
    }

    /// The scope dimension label this op's argument is checked against (for diagnostics).
    fn scope_dimension(self) -> &'static str {
        match self {
            Op::FsRead => "fs.read",
            Op::FsWrite => "fs.write",
            Op::Net => "net",
            Op::Declassify => "declassify",
            Op::ForeignBind => "foreign.c",
            _ => "-",
        }
    }
}

/// The result of a `check` (spec §4.4). `Allow` carries the consumed audit seq for synchronous-class
/// uses (which produce an audit record — invariant 26); epoch-class uses carry `None` (local, no
/// record).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow { audit_seq: Option<u64> },
    Deny(Denial),
}

impl Decision {
    pub fn is_allow(&self) -> bool {
        matches!(self, Decision::Allow { .. })
    }
    pub fn denial(&self) -> Option<&Denial> {
        match self {
            Decision::Deny(d) => Some(d),
            _ => None,
        }
    }
}

/// Validate an op against a node's authority + effective state. Shared by the live path and the
/// snapshot path; the caller decides the data source (live tree vs frozen snapshot).
fn validate(node_id: &GrantId, eff: EffState, authority: &Authority, op: Op, arg: Option<&str>) -> Result<(), Denial> {
    // 1. Liveness (spec §4.3).
    match eff {
        EffState::Revoked { by_seq } => return Err(Denial::Revoked { node: node_id.clone(), by_seq }),
        EffState::Expired { ttl_millis, now_millis } => {
            return Err(Denial::Expired { node: node_id.clone(), ttl_millis, now_millis })
        }
        EffState::Live => {}
    }
    // 2. Effect present in the node's authority (defense in depth — the checker already bounds kind).
    if let Some(needed) = op.required_effect() {
        if !authority.effects.contains(&needed) {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: "effects",
                arg: needed.name().to_string(),
            });
        }
    }
    // 3. Scope argument within the node's granted scope for this dimension.
    if let Some(arg) = arg {
        let ok = match op {
            Op::FsRead => authority.scopes.fs_read.iter().any(|root| crate::path_is_within(arg, root)),
            Op::FsWrite => authority.scopes.fs_write.iter().any(|root| crate::path_is_within(arg, root)),
            Op::Net => authority.scopes.net.contains(arg),
            Op::Declassify => authority.scopes.declassify.contains(arg),
            Op::ForeignBind => authority.scopes.foreign_c.contains(arg),
            // Clock/Rand/Console/reserved: no scope argument to validate.
            _ => true,
        };
        if !ok {
            return Err(Denial::OutOfScope {
                node: node_id.clone(),
                dimension: op.scope_dimension(),
                arg: arg.to_string(),
            });
        }
    }
    Ok(())
}

impl Broker {
    /// Validate an op against LIVE tree state (spec §4.4). Intended for the **synchronous** class
    /// (re-validated every call); it also accepts epoch-class ops against live state for uniformity.
    /// A synchronous-class use consumes one audit seq (invariant 26) whether allowed or denied;
    /// epoch-class uses consume none.
    pub fn check(&mut self, node_id: &GrantId, op: Op, arg: Option<&str>) -> Decision {
        let now = self.effective_now();
        let (eff, authority) = match self.node_view(node_id, now) {
            Some(v) => v,
            None => {
                if op.class() == OpClass::Synchronous {
                    self.consume_seq();
                }
                return Decision::Deny(Denial::UnknownNode { node: node_id.clone() });
            }
        };
        let result = validate(node_id, eff, &authority, op, arg);
        match op.class() {
            OpClass::Synchronous => {
                let seq = self.consume_seq(); // one audit record per synchronous use
                match result {
                    Ok(()) => Decision::Allow { audit_seq: Some(seq) },
                    Err(d) => Decision::Deny(d),
                }
            }
            OpClass::Epoch => match result {
                Ok(()) => Decision::Allow { audit_seq: None },
                Err(d) => Decision::Deny(d),
            },
        }
    }

    /// Freeze a cheap point-in-time [`Snapshot`] of node states + authorities + the epoch counter
    /// (spec §4). The runtime refreshes this every `--epoch-ms` (chunk 3); epoch-class ops validate
    /// against it, so revocation takes effect for them within one refresh interval — never claimed
    /// as "immediate" (spec §4.2).
    pub fn snapshot(&self) -> Snapshot {
        let now = self.effective_now();
        let nodes = self
            .iter_nodes()
            .map(|n| (n.id.clone(), SnapNode { eff: effective_state(n, now), authority: n.authority.clone() }))
            .collect();
        Snapshot { epoch: self.epoch(), nodes }
    }
}

/// A frozen, read-only view of the tree for epoch-class validation (spec §4).
pub struct Snapshot {
    epoch: u64,
    nodes: std::collections::HashMap<GrantId, SnapNode>,
}

struct SnapNode {
    eff: EffState,
    authority: Authority,
}

impl Snapshot {
    /// The revocation epoch this snapshot was taken at (staleness detection).
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Validate an **epoch-class** op against this frozen view (spec §4.4). No audit seq: epoch-class
    /// uses produce no audit record. A synchronous-class op should go through [`Broker::check`]
    /// against live state instead — passing one here is still validated (fail-closed) but callers
    /// must not rely on the snapshot for synchronous freshness.
    pub fn check(&self, node_id: &GrantId, op: Op, arg: Option<&str>) -> Decision {
        let Some(sn) = self.nodes.get(node_id) else {
            return Decision::Deny(Denial::UnknownNode { node: node_id.clone() });
        };
        match validate(node_id, sn.eff, &sn.authority, op, arg) {
            Ok(()) => Decision::Allow { audit_seq: None },
            Err(d) => Decision::Deny(d),
        }
    }
}

// ----- private broker accessor used above (kept here so `tree.rs` stays focused) ---------------

impl Broker {
    fn node_view(&self, id: &GrantId, now: i64) -> Option<(EffState, Authority)> {
        self.inspect(id).map(|n| (effective_state(n, now), n.authority.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::Scopes;
    use crate::ids::SeqIdSource;
    use crate::time::ManualClock;
    use crate::tree::Holder;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    fn eff(names: &[&str]) -> BTreeSet<Effect> {
        names.iter().map(|n| Effect::core_from_name(n).unwrap()).collect()
    }
    fn names(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }
    fn holder() -> Holder {
        Holder::new("process", "agent", "pid:1")
    }
    fn broker_with_clock(clock: Rc<ManualClock>) -> Broker {
        Broker::with_sources(Box::new(SeqIdSource::new()), Box::new(clock))
    }

    #[test]
    fn op_classification_matches_spec_4_1() {
        for op in [Op::Declassify, Op::FsWrite, Op::Net, Op::ForeignBind, Op::PluginLoad, Op::Actuate] {
            assert_eq!(op.class(), OpClass::Synchronous, "{op:?} is synchronous");
        }
        for op in [Op::FsRead, Op::Clock, Op::Rand, Op::Console] {
            assert_eq!(op.class(), OpClass::Epoch, "{op:?} is epoch");
        }
    }

    #[test]
    fn synchronous_op_fails_on_the_very_next_call_after_revoke() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
            None,
        );
        // Before revoke: an FsWrite within scope is allowed and consumes a seq.
        let d = b.check(&root, Op::FsWrite, Some("./out/log.txt"));
        assert!(d.is_allow());
        assert!(matches!(d, Decision::Allow { audit_seq: Some(_) }), "synchronous use records a seq");

        b.revoke(&root, &root).unwrap();

        // The very next synchronous call fails DL1403 with the revoking seq.
        let d = b.check(&root, Op::FsWrite, Some("./out/log.txt"));
        let denial = d.denial().expect("revoked lease denies");
        assert_eq!(denial.code(), "DL1403");
        assert!(denial.revoking_seq().is_some(), "DL1403 carries the revoking audit seq");
    }

    #[test]
    fn epoch_op_passes_on_stale_snapshot_then_fails_after_refresh() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }),
            None,
        );
        // Snapshot taken BEFORE revocation: the child is Live in this frozen view.
        let stale = b.snapshot();
        assert!(stale.check(&root, Op::FsRead, Some("./data/x")).is_allow());

        b.revoke(&root, &root).unwrap();

        // The stale snapshot still allows the epoch-class op (revocation not yet observed) — this is
        // the honest ≤one-interval bound, never "immediate" (spec §4.2). No sleep involved.
        assert!(stale.check(&root, Op::FsRead, Some("./data/x")).is_allow(), "stale snapshot still Live");
        assert_eq!(stale.epoch(), 0);

        // After a refresh the new snapshot reflects the revocation and the epoch-class op fails.
        let fresh = b.snapshot();
        assert_eq!(fresh.epoch(), 1, "epoch bumped by the revocation");
        let d = fresh.check(&root, Op::FsRead, Some("./data/x"));
        assert_eq!(d.denial().unwrap().code(), "DL1403");
    }

    #[test]
    fn ttl_expiry_is_dl1402_against_the_pluggable_clock() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock.clone());
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Write"]), Scopes { fs_write: names(&["./out"]), ..Default::default() }),
            Some(5000), // TTL deadline at 5000 ms
        );
        // Before the deadline: allowed.
        assert!(b.check(&root, Op::FsWrite, Some("./out/a")).is_allow());
        // Advance the fake clock past the TTL — no sleep.
        clock.set(6000);
        let d = b.check(&root, Op::FsWrite, Some("./out/a"));
        assert_eq!(d.denial().unwrap().code(), "DL1402");
        // The epoch path sees it too, via a fresh snapshot at the new time.
        let snap = b.snapshot();
        assert_eq!(snap.check(&root, Op::FsRead, None).denial().map(|d| d.code()), Some("DL1402"))
    }

    #[test]
    fn out_of_scope_argument_is_dl0904() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let root = b.issue(
            holder(),
            Authority::new(eff(&["Write", "Net"]), Scopes { fs_write: names(&["./out"]), net: names(&["a.com"]), ..Default::default() }),
            None,
        );
        // A write outside the granted subtree.
        assert_eq!(b.check(&root, Op::FsWrite, Some("./secret")).denial().unwrap().code(), "DL0904");
        // A net host not in the allowlist.
        assert_eq!(b.check(&root, Op::Net, Some("evil.com")).denial().unwrap().code(), "DL0904");
        // A sibling-prefix path is NOT within (no naive string-prefix bug).
        assert_eq!(b.check(&root, Op::FsWrite, Some("./output")).denial().unwrap().code(), "DL0904");
        // In-scope succeeds.
        assert!(b.check(&root, Op::FsWrite, Some("./out/ok")).is_allow());
    }

    #[test]
    fn epoch_ops_do_not_consume_audit_seqs() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let root = b.issue(holder(), Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), None);
        let before = b.next_audit_seq();
        // Epoch-class op against live state: allowed, but no record/seq consumed.
        let d = b.check(&root, Op::FsRead, Some("./data/x"));
        assert!(matches!(d, Decision::Allow { audit_seq: None }));
        assert_eq!(b.next_audit_seq(), before, "epoch-class use consumes no audit seq");
    }
}
