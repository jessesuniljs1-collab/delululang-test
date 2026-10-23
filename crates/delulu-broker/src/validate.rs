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
use crate::tree::{Broker, EffState, GrantId};
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
            Op::Declassify | Op::FsWrite | Op::Net | Op::ForeignBind | Op::PluginLoad | Op::Actuate => OpClass::Synchronous,
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
/// snapshot path; the caller decides the data source (live tree vs frozen snapshot). `url_path` is a
/// network use's request path, when its caller sends one (P9-05, D-V2-169); `method` its HTTP method, `None` a GET
/// (P9-05 (d), D-V2-183).
fn validate(
    node_id: &GrantId,
    eff: EffState,
    authority: &Authority,
    op: Op,
    arg: Option<&str>,
    url_path: Option<&str>,
    method: Option<&str>,
) -> Result<(), Denial> {
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
            return Err(Denial::OutOfScope { node: node_id.clone(), dimension: "effects", arg: needed.name().to_string() });
        }
    }
    // 3. Scope argument within the node's granted scope for this dimension.
    if let Some(arg) = arg {
        let ok = match op {
            Op::FsRead => authority.scopes.fs_read.iter().any(|root| crate::path_is_within(arg, root)),
            Op::FsWrite => authority.scopes.fs_write.iter().any(|root| crate::path_is_within(arg, root)),
            // NET-WILDCARD-1 (routine run 25): by the runtime's own matcher, `*.example.com` included — the exact-string
            // `contains` refused every use of a pattern grant under the daemon that embedded custody allowed.
            // P9-05 (a), D-V2-138: an entry may name a path pattern; the op carries the host, so the host part is compared
            // here and the path is the egress client's to judge on every hop (`netscope::judge`) — under every custody.
            Op::Net => authority.scopes.net.iter().any(|pat| host_matches(arg, crate::netscope::host_of(pat))),
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
            return Err(Denial::OutOfScope { node: node_id.clone(), dimension: op.scope_dimension(), arg: arg.to_string() });
        }
        // P9-05, D-V2-169: a use that names its request path is judged on it too, by the reading the runtime's first check
        // and the egress client share (`netscope::judge_all`) — so the chain never records `allow` for a path the grant
        // does not reach. The host is within some entry (above); an entry for this host that names no pattern reaches
        // every path.
        // P9-05 (d), D-V2-182/183: and by its method — the custody's, or a GET where it names none.
        if let (Op::Net, Some(path)) = (op, url_path) {
            let refused = |why| Denial::NetPath { node: node_id.clone(), host: arg.to_string(), path: path.to_string(), why };
            let method = method.unwrap_or("GET");
            match crate::netscope::judge_all(&authority.scopes.net, method, arg, path) {
                crate::netscope::Verdict::Allow => {}
                crate::netscope::Verdict::Ambiguous(why) => return Err(refused(Some(why))),
                crate::netscope::Verdict::OtherMethod => {
                    return Err(Denial::NetMethod {
                        node: node_id.clone(),
                        host: arg.to_string(),
                        path: path.to_string(),
                        method: crate::netscope::METHODS.iter().find(|m| **m == method).copied().unwrap_or("an unnamed method"),
                    })
                }
                crate::netscope::Verdict::OutsidePath | crate::netscope::Verdict::OtherHost => return Err(refused(None)),
            }
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
        self.check_use_at(node_id, op, arg, None)
    }

    /// [`Broker::check_use`] for a use that names its request path (P9-05, D-V2-169): a network use's `url_path` — the
    /// URL's path, without its query or fragment, `/` when it names none — is judged against the node's `net` entries for
    /// the host and carried in the use's record (`url_path`, beside `op`). `target` stays the host: every reader of a
    /// `use` record reads it so (the OCSF export's `dst_endpoint.hostname`, an egress refusal's record, `audit query`).
    /// Ignored for any other op.
    pub fn check_use_at(&mut self, node_id: &GrantId, op: Op, arg: Option<&str>, url_path: Option<&str>) -> GuardedDecision {
        self.check_use_by(node_id, op, arg, url_path, None)
    }

    /// [`Broker::check_use_at`] by the request's HTTP `method` (P9-05 (d), D-V2-183): `None` is a GET, the request every
    /// use was before a primitive made another. A method that is not a GET is judged against the node's entries and named in
    /// the use's record (`method`, beside `url_path`), so the chain says a POST was a POST.
    pub fn check_use_by(
        &mut self,
        node_id: &GrantId,
        op: Op,
        arg: Option<&str>,
        url_path: Option<&str>,
        method: Option<&str>,
    ) -> GuardedDecision {
        let url_path = url_path.filter(|_| op == Op::Net);
        let method = method.filter(|m| op == Op::Net && *m != "GET");
        let now = self.effective_now();
        // PS-E-06 (D-V2-80): a use's record names its effect, in the payload slot under a self-describing
        // key — as a revocation's carries its bound (spec §11, chunk-5 deviation 3). Without it a record
        // said which node used which argument and not HOW: a path read, a path written and a host asked
        // were the same record, and an export could only guess one from the argument's spelling.
        let payload = || {
            let mut p = serde_json::json!({ "op": op.wire_name() });
            if let Some(path) = url_path {
                p["url_path"] = serde_json::json!(path);
            }
            if let Some(m) = method {
                p["method"] = serde_json::json!(m);
            }
            Some(p)
        };
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
                        payload(),
                        "deny",
                        None,
                    );
                }
                return GuardedDecision::deny(Denial::UnknownNode { node: node_id.clone() });
            }
        };
        // 1. Normal authorization: liveness + effect + scope (unchanged — the guard sits ON TOP).
        if let Err(d) = validate(node_id, eff, &authority, op, arg, url_path, method) {
            if op.class() == OpClass::Synchronous {
                let seq = self.consume_seq();
                self.record_op(
                    seq,
                    "use",
                    Some(node_id.as_str().to_string()),
                    arg.map(|a| a.to_string()),
                    payload(),
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
                    self.record_op(seq, "use", actor(), tgt(), payload(), "allow", None);
                    GuardedDecision::allow(Some(seq))
                }
                OpClass::Epoch => GuardedDecision::allow(None),
            },
            GuardVerdict::PermitUse => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_permit_use", actor(), tgt(), payload(), "allow", None);
                GuardedDecision::allow(Some(seq))
            }
            GuardVerdict::BypassedUse { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_bypassed_use", actor(), tgt(), payload(), "allow", None);
                GuardedDecision { decision: Decision::Allow { audit_seq: Some(seq) }, warn: Some(note) }
            }
            GuardVerdict::Warn { note } => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_warn", actor(), tgt(), payload(), "allow", None);
                GuardedDecision { decision: Decision::Allow { audit_seq: Some(seq) }, warn: Some(note) }
            }
            GuardVerdict::Block(d) => {
                let seq = self.consume_seq();
                self.record_op(seq, "guard_block", actor(), tgt(), payload(), "deny", None);
                GuardedDecision::deny(d)
            }
        }
    }

    /// Record a fetch the run's EGRESS CLIENT refused after this broker allowed the use of its host (RW 4.46,
    /// D-V2-112): one `use` record with decision `deny` on `node`, at `host`, carrying the egress client's
    /// machine-readable reason (`special-use`, `not-allowlisted`, …) and the seq of the `use allow` it
    /// overrides when the run knows it — [`Broker::record_device_refusal`]'s shape for the network. The
    /// broker answers whether this node may use the host; the egress client then checks what the name
    /// resolved to, where a redirect points, and more, and its refusal reached only the run's report and
    /// standard error, so the chain said the fetch was allowed. Decides nothing; refused for a node the
    /// tree does not hold; the reason is bounded.
    pub fn record_egress_refusal(
        &mut self,
        node_id: &GrantId,
        host: &str,
        reason: &str,
        overrides_seq: Option<u64>,
    ) -> Result<u64, Denial> {
        if self.inspect(node_id).is_none() {
            return Err(Denial::UnknownNode { node: node_id.clone() });
        }
        let mut payload = serde_json::json!({
            "op": Op::Net.wire_name(),
            "refused_by": "egress",
            "reason": crate::bounded_text(reason, 128),
        });
        if let Some(s) = overrides_seq {
            payload["overrides_seq"] = serde_json::json!(s);
        }
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "use",
            Some(node_id.as_str().to_string()),
            Some(crate::bounded_text(host, 512)),
            Some(payload),
            "deny",
            None,
        );
        Ok(seq)
    }

    /// Record a device command its ENVELOPE refused (P8-04, routine run 15): one `use` record with
    /// decision `deny` on `node`, at `device`, carrying the refusal's code (DL1904), the reason in the
    /// words the program was told, and the seq of the `use allow` it overrides when the run knows it.
    ///
    /// The broker never sees a command's magnitudes — it answers the device's IDENTITY (may this node
    /// command `arm0/elbow`?), records `allow`, and the run's device broker then judges the envelope
    /// (`device::actuate`). Until this existed that second decision reached only the run's standard
    /// error, so the chain said a refused command was allowed and an out-of-band monitor reading it
    /// could never see a refusal. This does not DECIDE anything — the command is already refused —
    /// it makes the chain say so. Refused for a node the tree does not hold (nothing is recorded about
    /// a node that does not exist); the reason is bounded, because it is a store a person reads later.
    pub fn record_device_refusal(
        &mut self,
        node_id: &GrantId,
        device: &str,
        reason: &str,
        overrides_seq: Option<u64>,
    ) -> Result<u64, Denial> {
        if self.inspect(node_id).is_none() {
            return Err(Denial::UnknownNode { node: node_id.clone() });
        }
        let reason = crate::bounded_text(reason, 512);
        let mut payload = serde_json::json!({
            "op": Op::Actuate.wire_name(),
            "refused_by": "envelope",
            "code": "DL1904",
            "reason": reason,
        });
        if let Some(s) = overrides_seq {
            payload["overrides_seq"] = serde_json::json!(s);
        }
        let seq = self.consume_seq();
        self.record_op(seq, "use", Some(node_id.as_str().to_string()), Some(device.to_string()), Some(payload), "deny", None);
        Ok(seq)
    }

    /// P8-04's `break-glass` rule (D-V2-116): a break-glass run's use, bound to the run's node — one `break-glass-run`
    /// record with decision `allow` on `node`, citing the ticket's fingerprint. The gate (`breakglass::gate`) records the
    /// glass broken before the run has a node; a lease run learns its node only when it redeems, and an out-of-band
    /// monitor judges only records that name a node in its subtree. Records, never decides (the ticket was accepted
    /// already). Refused for a node the tree does not hold; the ticket's words are bounded, as every stored string is.
    pub fn record_break_glass_run(&mut self, node_id: &GrantId, ticket: &str, relax: &str) -> Result<u64, Denial> {
        if self.inspect(node_id).is_none() {
            return Err(Denial::UnknownNode { node: node_id.clone() });
        }
        let seq = self.consume_seq();
        self.record_op(
            seq,
            "break-glass-run",
            Some(node_id.as_str().to_string()),
            Some(crate::bounded_text(ticket, 128)),
            Some(serde_json::json!({ "relax": crate::bounded_text(relax, 64) })),
            "allow",
            None,
        );
        Ok(seq)
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
            .map(|n| {
                // Same inherited rule as the live path, so an epoch-class op cannot survive on a
                // stale view of an ancestry that has since expired.
                let eff =
                    self.effective_state_inherited(&n.id, now).unwrap_or(EffState::Expired { ttl_millis: 0, now_millis: now });
                (n.id.clone(), SnapNode { eff, authority: n.authority.clone() })
            })
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
        let nodes = entries.into_iter().map(|(id, eff, authority)| (id, SnapNode { eff, authority })).collect();
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
        match validate(node_id, sn.eff, &sn.authority, op, arg, None, None) {
            Ok(()) => Decision::Allow { audit_seq: None },
            Err(d) => Decision::Deny(d),
        }
    }
}

// ----- private broker accessor used above (kept here so `tree.rs` stays focused) ---------------

impl Broker {
    fn node_view(&self, id: &GrantId, now: i64) -> Option<(EffState, Authority)> {
        // INHERITED, not per-node: a node is live only if its whole ancestry is. See
        // `Broker::effective_state_inherited` — a subtree that could outlive its root would make a
        // federated uplink lease (RFC 0001 F4) impossible to time out.
        let eff = self.effective_state_inherited(id, now)?;
        self.inspect(id).map(|n| (eff, n.authority.clone()))
    }
}

/// Does the host `host` fall under the `net` pattern `pat` — the one host matcher, used by the broker's use check and by
/// the runtime's allowlist and egress client alike (`delulu_runtime::prim::host_matches` calls it), so a grant means the
/// same thing under every custody (NET-WILDCARD-1). A pattern is a host exactly, or `*.SUFFIX`: any host strictly below
/// `SUFFIX` at a dot boundary — campaign finding C85: a bare `ends_with` let `*.example.com` match `evilexample.com`,
/// another registrable domain. The apex (`example.com`) is not matched by `*.example.com`; narrowing is the safe direction.
pub fn host_matches(host: &str, pat: &str) -> bool {
    if let Some(suffix) = pat.strip_prefix("*.") {
        host.len() > suffix.len() && host.ends_with(suffix) && host.as_bytes()[host.len() - suffix.len() - 1] == b'.'
    } else {
        host == pat
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

    /// NET-WILDCARD-1 (routine run 25): a node granted `*.example.com` may use a host beneath it — the runtime's reading of
    /// the same grant — and nothing else: not the apex, not another registrable domain ending in the same letters (C85),
    /// not a host the node never named. Before, the broker compared the host by exact string and refused every use.
    #[test]
    fn a_pattern_net_grant_is_read_by_the_runtimes_own_matcher() {
        let mut b = broker_with_clock(Rc::new(ManualClock::new(1000)));
        let node = b.issue(
            holder(),
            Authority::new(eff(&["Net"]), Scopes { net: names(&["*.example.com", "plain.org"]), ..Default::default() }),
            None,
        );
        for (host, allowed) in [
            ("api.example.com", true),
            ("a.b.example.com", true),
            ("plain.org", true),
            ("example.com", false),
            ("evilexample.com", false),
            ("api.plain.org", false),
            ("other.net", false),
        ] {
            assert_eq!(b.check(&node, Op::Net, Some(host)).is_allow(), allowed, "{host}");
        }
    }

    /// P9-05 (a), D-V2-138: a node granted a host with a path pattern may use that host — the op carries the host, and the
    /// path is the egress client's to judge — exactly as embedded custody reads the same grant. Comparing the whole entry
    /// (`api.example.com/v1/**`) with the host would refuse every use of it under the daemon: NET-WILDCARD-1's shape again.
    #[test]
    fn a_path_scoped_net_grant_is_compared_by_its_host_part() {
        let mut b = broker_with_clock(Rc::new(ManualClock::new(1000)));
        let node = b.issue(
            holder(),
            Authority::new(
                eff(&["Net"]),
                Scopes { net: names(&["api.example.com/v1/**", "*.cdn.org/assets/*"]), ..Default::default() },
            ),
            None,
        );
        for (host, allowed) in
            [("api.example.com", true), ("x.cdn.org", true), ("example.com", false), ("cdn.org", false), ("evil.com", false)]
        {
            assert_eq!(b.check(&node, Op::Net, Some(host)).is_allow(), allowed, "{host}");
        }
    }

    /// P9-05, D-V2-169: a network use that names its request path is judged on it, by the reading the runtime shares, and
    /// its record names it as `url_path` beside `op` — allowed or refused — while `target` stays the host. Before, the
    /// broker compared the host alone, so the chain recorded `allow` for a request whose path the grant does not reach, and
    /// no record said which path a run asked of a host. A use that names no path is judged on its host, as before; any other
    /// op's path is ignored.
    #[test]
    fn a_net_use_naming_its_path_is_judged_on_it_and_recorded_with_it() {
        let sink = crate::audit::MemSink::new();
        let mut b = broker_with_clock(Rc::new(ManualClock::new(1000))).with_sink(Box::new(sink.clone()));
        let node = b.issue(
            holder(),
            Authority::new(
                eff(&["Net", "Write"]),
                Scopes {
                    net: names(&["api.example.com/v1/**", "open.example.org"]),
                    fs_write: names(&["./out"]),
                    ..Default::default()
                },
            ),
            None,
        );
        for (host, path, allowed) in [
            ("api.example.com", "/v1/items", true),
            ("api.example.com", "/v1", true),
            ("api.example.com", "/v10/items", false),
            ("api.example.com", "/v2/items", false),
            ("api.example.com", "/v1/../v2/items", false),
            ("open.example.org", "/anything/at/all", true),
            ("evil.example.net", "/v1/items", false),
        ] {
            let d = b.check_use_at(&node, Op::Net, Some(host), Some(path)).decision;
            assert_eq!(d.is_allow(), allowed, "{host}{path}: {d:?}");
            let r = sink.records().pop().unwrap();
            assert_eq!(r.target.as_deref(), Some(host), "the target is the host, never the path");
            let payload = r.authority.as_ref().unwrap();
            assert_eq!((payload["op"].as_str(), payload["url_path"].as_str()), (Some("Net"), Some(path)), "{host}{path}");
            assert_eq!(r.decision, if allowed { "allow" } else { "deny" }, "{host}{path}");
        }
        let mut denial = |path| match b.check_use_at(&node, Op::Net, Some("api.example.com"), Some(path)).decision {
            Decision::Deny(d) => (d.code(), d.to_diagnostic().message),
            other => panic!("{path}: {other:?}"),
        };
        let (code, words) = denial("/v2/items");
        assert_eq!(code, "DL0904");
        assert!(words.contains("outside every path the grant names for its host"), "{words}");
        let (code, words) = denial("/v1/../v2");
        assert_eq!(code, "DL0904");
        assert!(words.contains("refused under a path scope"), "{words}");

        // Named no path: judged on the host, as every use was before.
        assert!(b.check_use(&node, Op::Net, Some("api.example.com")).decision.is_allow());
        assert!(sink.records().pop().unwrap().authority.unwrap().get("url_path").is_none());
        // Another op's path is not a request path.
        assert!(b.check_use_at(&node, Op::FsWrite, Some("./out/a"), Some("/v2")).decision.is_allow());
        assert!(sink.records().pop().unwrap().authority.unwrap().get("url_path").is_none());
    }

    /// P9-05 (d), D-V2-183: a network use names its method when it is not a GET, and is judged by it against the node's
    /// entries — a POST under an entry naming no methods (GET alone) is refused in words about its method, and taken under
    /// one naming POST — and its record names it (`method`, beside `url_path`), allowed or refused. A GET's record is
    /// what it always was.
    #[test]
    fn a_net_use_is_judged_by_its_method_and_a_post_is_recorded_as_one() {
        let sink = crate::audit::MemSink::new();
        let mut b = broker_with_clock(Rc::new(ManualClock::new(1000))).with_sink(Box::new(sink.clone()));
        let node = b.issue(
            holder(),
            Authority::new(
                eff(&["Net"]),
                Scopes { net: names(&["api.example.com/v1/**", "hooks.example.org:POST"]), ..Default::default() },
            ),
            None,
        );
        for (host, method, allowed) in [
            ("api.example.com", None, true),
            ("api.example.com", Some("POST"), false),
            ("hooks.example.org", Some("POST"), true),
            ("hooks.example.org", None, false),
            ("hooks.example.org", Some("PUT"), false),
        ] {
            let d = b.check_use_by(&node, Op::Net, Some(host), Some("/v1/x"), method).decision;
            assert_eq!(d.is_allow(), allowed, "{method:?} {host}: {d:?}");
            let payload = sink.records().pop().unwrap().authority.unwrap();
            assert_eq!(payload.get("method").and_then(|m| m.as_str()), method, "{method:?} {host}");
            if let (false, Decision::Deny(d)) = (allowed, d) {
                assert_eq!(d.code(), "DL0904");
                assert!(d.to_diagnostic().message.contains("outside the methods the grant names"), "{d:?}");
            }
        }
        // `GET` named outright is a GET: recorded as one, with no `method`.
        assert!(b.check_use_by(&node, Op::Net, Some("api.example.com"), Some("/v1/x"), Some("GET")).decision.is_allow());
        assert!(sink.records().pop().unwrap().authority.unwrap().get("method").is_none());
    }

    /// P8-04: a device refusal is recorded only for a node the tree holds, and its reason is bounded
    /// on a character boundary — it is a store a person reads later, and a guest chose part of it.
    #[test]
    fn a_device_refusal_is_recorded_for_a_held_node_only_with_its_reason_bounded() {
        let sink = crate::audit::MemSink::new();
        let mut b = broker_with_clock(Rc::new(ManualClock::new(1000))).with_sink(Box::new(sink.clone()));
        let ghost = GrantId::from_trusted("g_nobody".to_string());
        assert!(matches!(b.record_device_refusal(&ghost, "arm0/elbow", "out", None), Err(Denial::UnknownNode { .. })));
        assert!(sink.is_empty(), "nothing is recorded about a node that does not exist");

        let node = b.issue(holder(), Authority::new(eff(&["Actuate"]), Scopes::default()), None);
        let before = sink.len();
        let long = "é".repeat(400); // 800 bytes, two per character
        let seq = b.record_device_refusal(&node, "arm0/elbow", &long, Some(7)).unwrap();
        let recs = sink.records();
        assert_eq!(recs.len(), before + 1);
        let r = recs.last().unwrap();
        assert_eq!((r.seq, r.action.as_str(), r.decision.as_str()), (seq, "use", "deny"));
        assert_eq!(r.actor_node.as_deref(), Some(node.as_str()));
        assert_eq!(r.target.as_deref(), Some("arm0/elbow"));
        let a = r.authority.as_ref().unwrap();
        assert_eq!(a["refused_by"], "envelope");
        assert_eq!(a["overrides_seq"], 7);
        let reason = a["reason"].as_str().unwrap();
        assert!(reason.len() <= 512 + '…'.len_utf8(), "bounded: {} bytes", reason.len());
        assert!(reason.ends_with('…') && reason.starts_with("éé"), "cut on a character, and said to be cut");
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
        let node =
            b.issue(holder(), device_auth(&["arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold"]), None);
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
        let parent =
            b.issue(holder(), device_auth(&["sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]), None);
        // The autonomy-grant shape from the satellite profile: a child may narrow the box…
        let narrower = device_auth(&["sat0/wheels:slew_deg=-0.2..0.2,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]);
        assert!(b.attenuate(&parent, narrower, holder(), None).is_ok(), "narrowing the box attenuates");

        // …and may not widen it, however interesting circumstances become.
        let wider = device_auth(&["sat0/wheels:slew_deg=-40..40,heartbeat_ms=1000,ttl_ms=600000,fail=hold"]);
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
        let root =
            b.issue(holder(), Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), None);
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
            Authority::new(
                eff(&["Write", "Net"]),
                Scopes { fs_write: names(&["./out"]), net: names(&["a.com"]), ..Default::default() },
            ),
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
        let root =
            b.issue(holder(), Authority::new(eff(&["Read"]), Scopes { fs_read: names(&["./data"]), ..Default::default() }), None);
        let before = b.next_audit_seq();
        // Epoch-class op against live state: allowed, but no record/seq consumed.
        let d = b.check(&root, Op::FsRead, Some("./data/x"));
        assert!(matches!(d, Decision::Allow { audit_seq: None }));
        assert_eq!(b.next_audit_seq(), before, "epoch-class use consumes no audit seq");
    }
}
