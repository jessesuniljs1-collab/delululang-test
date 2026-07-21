//! Phase 5c — the two validation classes + revocation epochs (spec §4).
//!
//! Every primitive-table operation is classed (spec §4.1):
//! - **Synchronous** — validated by a broker round-trip against LIVE tree state per use:
//!   `Declassify`/expose, `FsWrite`, `Net`, foreign bind, and — since Stage 10 phase 10g —
//!   `Actuate`, which was reserved here in Stage 5 and now carries real commands to real
//!   machines (plugin-load remains a reserved variant, unused).
//! - **Epoch** — validated locally against a cached [`Snapshot`] the runtime refreshes every
//!   `--epoch-ms` (that refresh wiring is chunk 3): `FsRead`, `Clock`, `Rand`, `Console`.
//!
//! Liveness (spec §4.3): every op checks state ≠ Revoked/Expired. A revoked lease is DL1403 carrying
//! the revoking audit seq; a TTL-expired lease is DL1402 (checked against the pluggable clock).

use crate::authority::Authority;
use crate::diag::Denial;
use crate::guard::GuardVerdict;
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
    /// Commanding a physical device (Stage 10 Track D). Reserved in Stage 5, ACTIVE since phase
    /// 10g: `Cap[Actuator].command` round-trips here per command, which is what makes an operator
    /// e-stop — `delulu grants revoke` on the holding node — reach a running arm at all.
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
    pub fn required_effect(self) -> Option<Effect> {
        match self {
            Op::Declassify => Some(Effect::Declassify),
            Op::FsWrite => Some(Effect::Write),
            Op::Net => Some(Effect::Net),
            Op::ForeignBind => Some(Effect::ForeignCall),
            Op::FsRead => Some(Effect::Read),
            Op::Clock => Some(Effect::Clock),
            Op::Rand => Some(Effect::Rand),
            Op::Console => Some(Effect::Write),
            // A node with no `Actuate` in its authority commands nothing, whatever envelope its
            // capability value happens to carry — the grant tree is the authority, the value is a
            // copy of it (10e's ordering law, now enforced one layer further out).
            Op::Actuate => Some(Effect::Actuate),
            Op::PluginLoad => None,
        }
    }

    /// The stable wire name for this op (chunk 3 IPC: the `Check`/`use` request carries it as a
    /// string so the enum encoding is deterministic and version-stable). Round-trips with
    /// [`Op::from_wire_name`].
    pub fn wire_name(self) -> &'static str {
        match self {
            Op::Declassify => "Declassify",
            Op::FsWrite => "FsWrite",
            Op::Net => "Net",
            Op::ForeignBind => "ForeignBind",
            Op::PluginLoad => "PluginLoad",
            Op::Actuate => "Actuate",
            Op::FsRead => "FsRead",
            Op::Clock => "Clock",
            Op::Rand => "Rand",
            Op::Console => "Console",
        }
    }

    /// Parse an [`Op`] from its [`Op::wire_name`] (chunk 3 IPC). `None` for an unknown name
    /// (fail-closed: an unrecognized op is refused by the caller, never silently allowed).
    pub fn from_wire_name(s: &str) -> Option<Op> {
        Some(match s {
            "Declassify" => Op::Declassify,
            "FsWrite" => Op::FsWrite,
            "Net" => Op::Net,
            "ForeignBind" => Op::ForeignBind,
            "PluginLoad" => Op::PluginLoad,
            "Actuate" => Op::Actuate,
            "FsRead" => Op::FsRead,
            "Clock" => Op::Clock,
            "Rand" => Op::Rand,
            "Console" => Op::Console,
            _ => return None,
        })
    }

    /// The scope dimension label this op's argument is checked against (for diagnostics).
    fn scope_dimension(self) -> &'static str {
        match self {
            Op::FsRead => "fs.read",
            Op::FsWrite => "fs.write",
            Op::Net => "net",
            Op::Declassify => "declassify",
            Op::ForeignBind => "foreign.c",
            Op::Actuate => "device",
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

/// A guard-aware [`Broker::check_use`] result (Stage 5 chunk 6). Carries the normal [`Decision`] plus
/// an optional agent-side `warn` note — a `warn`-tier use or a bypassed-guarded use proceeds but
/// carries a one-line note the custody client surfaces once per rule per run (addendum §2.6/§2.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardedDecision {
    pub decision: Decision,
    pub warn: Option<String>,
}

impl GuardedDecision {
    fn allow(seq: Option<u64>) -> GuardedDecision {
        GuardedDecision { decision: Decision::Allow { audit_seq: seq }, warn: None }
    }
    fn deny(d: Denial) -> GuardedDecision {
        GuardedDecision { decision: Decision::Deny(d), warn: None }
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
            // RFC 0001 F1 (D12e). Until this arm existed, `Op::Actuate` fell through to `_ => true`
            // below — the device path has been arriving here on EVERY actuator command since 10g
            // activated Actuate (`interp.rs` sends `env.device` as the arg), and it was accepted
            // unconditionally by an arm whose comment still said "reserved". That was not a live
            // fail-open, because `Scopes` carried no device dimension to check against and the
            // numeric envelope is genuinely enforced runtime-side. It was, exactly, a rule waiting
            // to die in a fall-through. This arm answers the device IDENTITY question; the
            // magnitudes never reach this layer, and `envelope_check` bounds those where they do.
            Op::Actuate => crate::device_scope::grants_device(&authority.scopes.device, arg),
            // Clock/Rand/Console and the reserved PluginLoad variant take no scope argument.
            Op::Clock | Op::Rand | Op::Console | Op::PluginLoad => true,
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
    /// Validate an op against LIVE tree state (spec §4.4), then apply the Guard (Stage 5 chunk 6).
    /// A synchronous-class use consumes one audit seq (invariant 26) whether allowed or denied;
    /// an ungated epoch-class use consumes none. A GUARDED op always records synchronously (addendum
    /// §2.4.2 / criterion 11) — a cached snapshot cannot consult permits. Returns the [`Decision`]
    /// plus an optional agent-side `warn` note.
    pub fn check_use(&mut self, node_id: &GrantId, op: Op, arg: Option<&str>) -> GuardedDecision {
        let now = self.effective_now();
        let (eff, authority) = match self.node_view(node_id, now) {
            Some(v) => v,
            None => {
                if op.class() == OpClass::Synchronous {
                    // A synchronous use still consumes one seq and emits one record, even for an
                    // unknown lease (invariant 26 — the deny is audited).
                    let seq = self.consume_seq();
                    self.record_op(
                        seq,
                        "use",
                        Some(node_id.as_str().to_string()),
                        arg.map(|a| a.to_string()),
                        None,
                        "deny",
                        None,
                    );
                }
                return GuardedDecision::deny(Denial::UnknownNode { node: node_id.clone() });
            }
        };
        // 1. Normal authorization: liveness + effect + scope (unchanged — the guard sits ON TOP).
        if let Err(d) = validate(node_id, eff, &authority, op, arg) {
            if op.class() == OpClass::Synchronous {
                let seq = self.consume_seq();
                self.record_op(
                    seq,
                    "use",
                    Some(node_id.as_str().to_string()),
                    arg.map(|a| a.to_string()),
                    None,
                    "deny",
                    None,
                );
            }
            return GuardedDecision::deny(d);
        }
        // 2. The op is authorized. Apply the Guard for delegated nodes; a guarded verdict records ONE
        //    guard event synchronously (guarded ops never ride the epoch snapshot — criterion 11).
        let actor = || Some(node_id.as_str().to_string());
        let tgt = || arg.map(|a| a.to_string());
        match self.guard_verdict_use(node_id, op, arg) {
            GuardVerdict::Ungated => match op.class() {
                OpClass::Synchronous => {
                    let seq = self.consume_seq();
                    self.record_op(seq, "use", actor(), tgt(), None, "allow", None);
                    GuardedDecision::allow(Some(seq))
                }
                OpClass::Epoch => GuardedDecision::allow(None),
            },
            GuardVerdict::PermitUse => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_permit_use", actor(), tgt(), None, "allow", None);
                GuardedDecision::allow(Some(seq))
            }
            GuardVerdict::BypassedUse { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_bypassed_use", actor(), tgt(), None, "allow", None);
                GuardedDecision { decision: Decision::Allow { audit_seq: Some(seq) }, warn: Some(note) }
            }
            GuardVerdict::Warn { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_warn", actor(), tgt(), None, "allow", None);
                GuardedDecision { decision: Decision::Allow { audit_seq: Some(seq) }, warn: Some(note) }
            }
            GuardVerdict::Block(d) => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_block", actor(), tgt(), None, "deny", None);
                GuardedDecision::deny(d)
            }
        }
    }

    /// Validate an op against LIVE tree state (spec §4.4), discarding any guard warn note. The
    /// guard-aware form is [`Broker::check_use`]; both share one implementation.
    pub fn check(&mut self, node_id: &GrantId, op: Op, arg: Option<&str>) -> Decision {
        self.check_use(node_id, op, arg).decision
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
    /// Rebuild a snapshot from explicit `(id, effective-state, authority)` entries and an epoch
    /// (chunk 3 IPC: the client caches its node's authority — known from its own `issue` — and
    /// refreshes only the effective state + epoch from the daemon every `--epoch-ms`, then validates
    /// epoch-class ops against THIS reconstructed snapshot). It calls the exact same [`validate`]
    /// path the live broker uses, so the client duplicates no policy logic (playbook §1).
    pub fn from_entries(epoch: u64, entries: Vec<(GrantId, EffState, Authority)>) -> Snapshot {
        let nodes = entries
            .into_iter()
            .map(|(id, eff, authority)| (id, SnapNode { eff, authority }))
            .collect();
        Snapshot { epoch, nodes }
    }

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

    /// Build an authority holding `Actuate` over exactly the named device grants.
    fn device_auth(specs: &[&str]) -> Authority {
        let device = specs
            .iter()
            .map(|s| crate::device_scope::parse(s).expect("test grant parses"))
            .map(|d| (d.device.clone(), d))
            .collect();
        Authority::new(eff(&["Actuate"]), Scopes { device, ..Default::default() })
    }

    /// **The regression witness for RFC 0001 F1 / D12e.**
    ///
    /// This test FAILS against the previous code, where `Op::Actuate` fell through to `_ => true`
    /// and every device path that arrived was accepted. It is the whole reason the arm exists, and
    /// it is written as a `check` on live tree state because that is the path a real command takes
    /// (`Actuate` is synchronous-class: one broker round-trip per command).
    #[test]
    fn actuate_on_a_device_the_node_was_never_granted_is_refused() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let node = b.issue(
            holder(),
            device_auth(&["arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold"]),
            None,
        );
        assert!(b.check(&node, Op::Actuate, Some("arm0/elbow")).is_allow(), "the granted device commands");

        let d = b.check(&node, Op::Actuate, Some("arm0/wrist"));
        let denial = d.denial().expect("a device nobody granted must be refused, not defaulted to allow");
        assert_eq!(denial.code(), "DL0904");
        match denial {
            Denial::OutOfScope { dimension, arg, .. } => {
                assert_eq!(*dimension, "device", "the refusal names the dimension it failed in");
                assert_eq!(arg, "arm0/wrist");
            }
            other => panic!("expected an out-of-scope denial, got {other:?}"),
        }
    }

    /// Holding `Effect::Actuate` is necessary but no longer sufficient: the effect answers "may this
    /// node command machines at all", the scope answers "which one". Before F1 only the first
    /// question existed, so a node with the effect could command anything the runtime handed it.
    #[test]
    fn the_actuate_effect_alone_no_longer_commands_every_device() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let node = b.issue(holder(), Authority::new(eff(&["Actuate"]), Scopes::default()), None);
        assert!(
            b.check(&node, Op::Actuate, Some("arm0/elbow")).denial().is_some(),
            "Actuate with an empty device scope grants no device"
        );
    }

    /// Attenuation carries the envelope now, so a delegated child is bounded where its parent was.
    /// The `⊑` failure returns the never-widening intersection, exactly as every other dimension does.
    #[test]
    fn a_child_cannot_widen_the_envelope_it_was_delegated() {
        let clock = Rc::new(ManualClock::new(1000));
        let mut b = broker_with_clock(clock);
        let parent = b.issue(
            holder(),
            device_auth(&["sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]),
            None,
        );
        // The autonomy-grant shape from the satellite profile: a child may narrow the box…
        let narrower =
            device_auth(&["sat0/wheels:slew_deg=-0.2..0.2,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]);
        assert!(b.attenuate(&parent, narrower, holder(), None).is_ok(), "narrowing the box attenuates");

        // …and may not widen it, however interesting circumstances become.
        let wider =
            device_auth(&["sat0/wheels:slew_deg=-40..40,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]);
        let err = b.attenuate(&parent, wider, holder(), None).unwrap_err();
        assert_eq!(err.code(), "DL0802");
        let inter = err.intersection().expect("DL0802 carries the intersection");
        assert_eq!(
            inter.scopes.device.get("sat0/wheels").map(|d| d.dims.get("slew_deg").copied()),
            Some(Some((-0.5, 0.5))),
            "the repair narrows to the parent's box, never the child's request"
        );
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
