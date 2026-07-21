//! RFC 0001 phase F1 — the **device** scope dimension and its lattice (build-order D12e).
//!
//! Before this module, `delulu_broker::Scopes` had dimensions for files, network, secrets and
//! foreign libraries and **none for a device**. A delegating party could say "you may actuate" and
//! could not say "you may fly this corridor only", so the UAS two-grant lost-link pattern
//! (`STAGE10_AUTONOMY_ADDENDUM.md` §2.2) was inexpressible and `run --lease` refused a local device
//! grant by name rather than silently dropping it.
//!
//! # The subset relation, and why it is the opposite of the obvious guess
//!
//! An envelope's dimension list is a **whitelist**, not a set of constraints on an otherwise-free
//! command. `delulu_runtime::device::envelope_check` iterates the *command's* fields and refuses any
//! field the envelope does not name — "the envelope cannot vouch for what it never bounded"
//! (`value.rs`). Therefore:
//!
//! - **More dimensions is WIDER.** Each added dimension admits a command shape that was refused
//!   before. A child naming a dimension its parent lacks is a **widening** and is refused.
//! - **Fewer dimensions is NARROWER.** A child may drop dimensions freely; dropping one means every
//!   command naming it is refused.
//!
//! Getting this backwards would have been a silent widening of exactly the shape this project has
//! been bitten by three times, which is why the rule is stated here next to the code that enforces
//! it rather than left to a reader's intuition.
//!
//! The remaining terms follow the same "narrower or equal" discipline:
//!
//! | Term | `child ⊑ parent` requires | Why |
//! |---|---|---|
//! | `dims` | every child dim in parent, child range ⊆ parent range | whitelist, above |
//! | `rate_hz` | parent unbounded → anything; parent bounded → child bounded and `≤` | an unbounded child under a bounded parent is a widening |
//! | `heartbeat_ms` | child `≤` parent | a *smaller* heartbeat is stricter: prove liveness more often |
//! | `ttl_ms` | child `≤` parent | a shorter loan is a smaller loan |
//! | `fail` | exact match | `hold`/`coast`/`safe-park` have no safety order — which is safer is device-dependent, so conservative is sound |
//!
//! # Non-finite bounds are refused at parse, and that is load-bearing
//!
//! `"NaN".parse::<f64>()` succeeds in Rust. A NaN bound would make every comparison in this module
//! meaningless and would break the reflexivity that [`DeviceScope`]'s `Eq` promises. [`parse`]
//! therefore refuses any non-finite bound. Downstream that refusal is what makes `impl Eq` sound.

use std::collections::{BTreeMap, BTreeSet};

/// One device's granted envelope: the authority-side twin of the runtime's `ActuatorEnvelope`.
///
/// Deliberately **not** a re-export of the runtime type: `delulu-broker` depends only on
/// `delulu-diag` and `delulu-check` (crate ruling 1), and the broker "deals in dimensions and
/// magnitudes, never in language values." The two parsers are pinned against each other by a
/// cross-check test in the `delulu` crate, which is where both are visible.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceScope {
    pub device: String,
    /// `dim -> (lo, hi)`, inclusive. `BTreeMap` so serialization is canonical (sorted) regardless
    /// of insertion order, matching every other dimension in [`crate::authority::Scopes`].
    pub dims: BTreeMap<String, (f64, f64)>,
    /// `None` = no rate bound. A bounded parent may not delegate to an unbounded child.
    pub rate_hz: Option<u32>,
    pub heartbeat_ms: u64,
    pub ttl_ms: u64,
    pub fail: String,
}

/// Sound because [`parse`] refuses non-finite bounds, so `x == x` holds for every value that can
/// reach this type. If that refusal is ever removed, this impl becomes unsound — the two are one
/// decision, not two.
impl Eq for DeviceScope {}

impl DeviceScope {
    /// Render back to the canonical grant form. Round-trips with [`parse`]; dimensions emit in
    /// sorted order, then the fixed terms, so the string is byte-canonical.
    pub fn to_grant_string(&self) -> String {
        let mut parts: Vec<String> =
            self.dims.iter().map(|(d, (lo, hi))| format!("{d}={}..{}", fmt_f64(*lo), fmt_f64(*hi))).collect();
        if let Some(hz) = self.rate_hz {
            parts.push(format!("rate_hz={hz}"));
        }
        parts.push(format!("heartbeat_ms={}", self.heartbeat_ms));
        parts.push(format!("ttl_ms={}", self.ttl_ms));
        parts.push(format!("fail={}", self.fail));
        format!("{}:{}", self.device, parts.join(","))
    }
}

/// Format a bound so `parse(to_grant_string(x)) == x`. `{}` on `f64` already round-trips in Rust
/// (shortest representation that reparses exactly), so no precision is lost here.
fn fmt_f64(x: f64) -> String {
    format!("{x}")
}

/// Parse the canonical grant form `DEVICE:dim=lo..hi[,...][,rate_hz=N],heartbeat_ms=N,ttl_ms=N,fail=S`.
///
/// Fail-closed at every branch: an unparseable part is an error, never a silently-unbounded
/// dimension and never a defaulted dead-man. Mirrors `ActuatorEnvelope::parse`'s contract because a
/// grant and the capability value minted from it must not be able to disagree.
pub fn parse(spec: &str) -> Result<DeviceScope, String> {
    let (device, rest) = spec.split_once(':').ok_or("missing `:` (use DEVICE:dim=lo..hi,...)")?;
    let device = device.trim();
    if device.is_empty() {
        return Err("empty device name".into());
    }
    let mut dims = BTreeMap::new();
    let mut rate_hz = None;
    let mut heartbeat_ms = None;
    let mut ttl_ms = None;
    let mut fail = None;
    for part in rest.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part.split_once('=').ok_or_else(|| format!("bad envelope part `{part}`"))?;
        let (k, v) = (k.trim(), v.trim());
        match k {
            "rate_hz" => rate_hz = Some(v.parse::<u32>().map_err(|_| format!("bad rate_hz `{v}`"))?),
            "heartbeat_ms" => {
                let n = v.parse::<u64>().map_err(|_| format!("bad heartbeat_ms `{v}`"))?;
                if n == 0 {
                    return Err("heartbeat_ms=0 would revoke the lease before the first command".into());
                }
                heartbeat_ms = Some(n);
            }
            "ttl_ms" => ttl_ms = Some(v.parse::<u64>().map_err(|_| format!("bad ttl_ms `{v}`"))?),
            "fail" => fail = Some(v.to_string()),
            _ => {
                let (lo, hi) = v.split_once("..").ok_or_else(|| format!("bad range `{v}` for `{k}`"))?;
                let lo: f64 = lo.trim().parse().map_err(|_| format!("bad low bound `{lo}` for `{k}`"))?;
                let hi: f64 = hi.trim().parse().map_err(|_| format!("bad high bound `{hi}` for `{k}`"))?;
                // Non-finite bounds are refused: they would make every comparison in this module
                // meaningless and would break `Eq`'s reflexivity. `"NaN".parse::<f64>()` succeeds,
                // so this branch is reachable from a grant string and must be closed.
                if !lo.is_finite() || !hi.is_finite() {
                    return Err(format!("non-finite bound in `{k}={v}` — an envelope must be a real interval"));
                }
                if lo > hi {
                    return Err(format!("inverted range `{k}={v}` (lo > hi bounds nothing)"));
                }
                dims.insert(k.to_string(), (lo, hi));
            }
        }
    }
    if dims.is_empty() {
        return Err("no bounded dimensions — a device grant that bounds nothing is not a grant".into());
    }
    let heartbeat_ms = heartbeat_ms
        .ok_or("no `heartbeat_ms` — a device grant with no dead-man is not a grant (spec §5.2)")?;
    let ttl_ms = ttl_ms.ok_or("no `ttl_ms` — a device grant with no loan end is not a grant (spec §5.2)")?;
    if ttl_ms < heartbeat_ms {
        return Err(format!(
            "ttl_ms={ttl_ms} is shorter than heartbeat_ms={heartbeat_ms} — the lease would expire \
             before its first beat was ever due"
        ));
    }
    let fail = fail.ok_or("no `fail` — a device grant must say what the machine does when authority ends")?;
    Ok(DeviceScope { device: device.to_string(), dims, rate_hz, heartbeat_ms, ttl_ms, fail })
}

/// Is `child ⊑ parent` for one device? See the module docs for each term's direction.
pub fn within(child: &DeviceScope, parent: &DeviceScope) -> bool {
    if child.device != parent.device || child.fail != parent.fail {
        return false;
    }
    // The whitelist rule: every dimension the child admits must be one the parent already admits,
    // and no wider. A child dim absent from the parent is a WIDENING (it admits a command shape the
    // parent refuses), which is why this is `else { return false }` and not `else { continue }`.
    for (dim, (clo, chi)) in &child.dims {
        let Some((plo, phi)) = parent.dims.get(dim) else { return false };
        if clo < plo || chi > phi {
            return false;
        }
    }
    // An unbounded child under a bounded parent is a widening.
    match (child.rate_hz, parent.rate_hz) {
        (_, None) => {}
        (None, Some(_)) => return false,
        (Some(c), Some(p)) => {
            if c > p {
                return false;
            }
        }
    }
    child.heartbeat_ms <= parent.heartbeat_ms && child.ttl_ms <= parent.ttl_ms
}

/// The **meet** (`⊓`) of two device scopes, or `None` when they have no common lower bound.
///
/// `None` means the device drops out of the intersection entirely — the narrowest possible answer,
/// so the meet still never widens. Different devices and different fail-states both land here:
/// there is no safety order on `hold`/`coast`/`safe-park` to take a minimum of.
pub fn meet(a: &DeviceScope, b: &DeviceScope) -> Option<DeviceScope> {
    if a.device != b.device || a.fail != b.fail {
        return None;
    }
    // Shared dimensions only, each narrowed to the overlap. A dimension present in one side and not
    // the other is DROPPED (narrower: commands naming it are refused), and so is a shared dimension
    // whose ranges do not overlap.
    let mut dims = BTreeMap::new();
    for (dim, (alo, ahi)) in &a.dims {
        if let Some((blo, bhi)) = b.dims.get(dim) {
            let lo = alo.max(*blo);
            let hi = ahi.min(*bhi);
            if lo <= hi {
                dims.insert(dim.clone(), (lo, hi));
            }
        }
    }
    let rate_hz = match (a.rate_hz, b.rate_hz) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    };
    let heartbeat_ms = a.heartbeat_ms.min(b.heartbeat_ms);
    let ttl_ms = a.ttl_ms.min(b.ttl_ms);
    // `min(ttl) >= min(heartbeat)` holds whenever both inputs satisfy `ttl >= heartbeat` (take the
    // side owning the smaller ttl: its own heartbeat is ≤ it, and the meet's heartbeat is ≤ that).
    // Checked rather than assumed: if it were ever violated the device drops out, fail-closed.
    if ttl_ms < heartbeat_ms {
        return None;
    }
    Some(DeviceScope { device: a.device.clone(), dims, rate_hz, heartbeat_ms, ttl_ms, fail: a.fail.clone() })
}

/// Is every device scope in `child` within some scope in `parent`? A child device the parent does
/// not grant at all is a widening — refused, never skipped.
pub fn all_within(child: &BTreeMap<String, DeviceScope>, parent: &BTreeMap<String, DeviceScope>) -> bool {
    child.iter().all(|(dev, cs)| parent.get(dev).is_some_and(|ps| within(cs, ps)))
}

/// Per-device meet across two sets. Devices present on only one side drop out.
pub fn intersect_device_sets(
    a: &BTreeMap<String, DeviceScope>,
    b: &BTreeMap<String, DeviceScope>,
) -> BTreeMap<String, DeviceScope> {
    let mut out = BTreeMap::new();
    for (dev, av) in a {
        if let Some(bv) = b.get(dev) {
            if let Some(m) = meet(av, bv) {
                out.insert(dev.clone(), m);
            }
        }
    }
    out
}

/// Does `arg` — a device path arriving with an `Actuate` check — fall inside any granted scope?
///
/// This is the function `validate.rs` calls in the arm where `Op::Actuate` used to fall through to
/// `_ => true`. It answers the device *identity* question only; the numeric envelope is enforced
/// against the command's fields in the runtime's `envelope_check`, which sees the magnitudes this
/// layer never receives.
pub fn grants_device(granted: &BTreeMap<String, DeviceScope>, arg: &str) -> bool {
    granted.contains_key(arg)
}

/// The set of device names granted, for diagnostics and the `--json` authority report.
pub fn device_names(granted: &BTreeMap<String, DeviceScope>) -> BTreeSet<String> {
    granted.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(s: &str) -> DeviceScope {
        parse(s).unwrap_or_else(|e| panic!("parse `{s}` failed: {e}"))
    }
    fn set(items: &[&str]) -> BTreeMap<String, DeviceScope> {
        items.iter().map(|s| ds(s)).map(|d| (d.device.clone(), d)).collect()
    }

    const P: &str = "arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=60000,fail=hold";

    #[test]
    fn identical_scopes_attenuate() {
        assert!(within(&ds(P), &ds(P)));
    }

    #[test]
    fn narrowing_a_range_attenuates_widening_does_not() {
        let narrow = ds("arm0/elbow:angle_deg=-10..10,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=60000,fail=hold");
        assert!(within(&narrow, &ds(P)), "a narrower interval is an attenuation");
        assert!(!within(&ds(P), &narrow), "and the reverse is a widening");
        let wider = ds("arm0/elbow:angle_deg=-90..95,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=60000,fail=hold");
        assert!(!within(&wider, &ds(P)), "a wider low bound is a widening");
    }

    /// The rule that is the opposite of the obvious guess, pinned as its own test.
    #[test]
    fn dropping_a_dimension_narrows_and_adding_one_widens() {
        let fewer = ds("arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold");
        assert!(within(&fewer, &ds(P)), "dropping a dimension refuses more commands — narrower");
        let more = ds(
            "arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,velocity_dps=0..40,heartbeat_ms=200,ttl_ms=60000,fail=hold",
        );
        assert!(
            !within(&more, &ds(P)),
            "a dimension the parent never bounded admits a command shape the parent refuses"
        );
    }

    #[test]
    fn dead_man_terms_only_tighten() {
        let faster = ds("arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=100,ttl_ms=60000,fail=hold");
        assert!(within(&faster, &ds(P)), "a smaller heartbeat is stricter");
        let slower = ds("arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=5000,ttl_ms=60000,fail=hold");
        assert!(!within(&slower, &ds(P)), "a larger heartbeat proves liveness less often — wider");
        let longer = ds("arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=600000,fail=hold");
        assert!(!within(&longer, &ds(P)), "a longer loan is a bigger loan");
    }

    #[test]
    fn an_unbounded_rate_under_a_bounded_parent_is_a_widening() {
        let bounded = ds("d0:x=0..1,rate_hz=50,heartbeat_ms=200,ttl_ms=1000,fail=hold");
        let unbounded = ds("d0:x=0..1,heartbeat_ms=200,ttl_ms=1000,fail=hold");
        let slower = ds("d0:x=0..1,rate_hz=10,heartbeat_ms=200,ttl_ms=1000,fail=hold");
        assert!(!within(&unbounded, &bounded), "no rate bound under a rate bound is a widening");
        assert!(within(&slower, &bounded), "a lower rate ceiling attenuates");
        assert!(within(&bounded, &unbounded), "any bound attenuates an unbounded parent");
    }

    #[test]
    fn a_different_device_or_fail_state_never_attenuates() {
        let other = ds("arm0/wrist:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=60000,fail=hold");
        assert!(!within(&other, &ds(P)), "a sibling device is not a descendant of this one");
        let coast = ds("arm0/elbow:angle_deg=-30..95,torque_nm=0..2.5,heartbeat_ms=200,ttl_ms=60000,fail=coast");
        assert!(!within(&coast, &ds(P)), "fail-states have no safety order — exact match or nothing");
        assert!(meet(&coast, &ds(P)).is_none(), "and they have no meet either");
    }

    #[test]
    fn the_meet_is_never_wider_than_either_side() {
        let a = ds("d0:x=0..10,y=0..10,rate_hz=50,heartbeat_ms=200,ttl_ms=9000,fail=hold");
        let b = ds("d0:x=5..20,z=0..1,rate_hz=10,heartbeat_ms=500,ttl_ms=1000,fail=hold");
        let m = meet(&a, &b).expect("same device, same fail-state");
        assert!(within(&m, &a), "meet ⊑ a");
        assert!(within(&m, &b), "meet ⊑ b");
        assert_eq!(m.dims.get("x"), Some(&(5.0, 10.0)), "shared dim narrows to the overlap");
        assert!(!m.dims.contains_key("y"), "a dim only one side bounded drops out");
        assert!(!m.dims.contains_key("z"), "…in both directions");
        assert_eq!(m.rate_hz, Some(10));
        assert_eq!((m.heartbeat_ms, m.ttl_ms), (200, 1000));
    }

    #[test]
    fn meet_is_symmetric() {
        let a = ds("d0:x=0..10,rate_hz=50,heartbeat_ms=200,ttl_ms=9000,fail=hold");
        let b = ds("d0:x=5..20,heartbeat_ms=500,ttl_ms=1000,fail=hold");
        assert_eq!(meet(&a, &b), meet(&b, &a));
    }

    #[test]
    fn disjoint_ranges_drop_the_dimension_rather_than_widening_it() {
        let a = ds("d0:x=0..1,y=0..5,heartbeat_ms=200,ttl_ms=1000,fail=hold");
        let b = ds("d0:x=5..6,y=0..5,heartbeat_ms=200,ttl_ms=1000,fail=hold");
        let m = meet(&a, &b).unwrap();
        assert!(!m.dims.contains_key("x"), "no overlap means no authority over that dimension");
        assert!(within(&m, &a) && within(&m, &b));
    }

    #[test]
    fn the_ttl_heartbeat_invariant_survives_the_meet() {
        // Both sides satisfy ttl >= heartbeat; so must the meet, for every pairing.
        for (at, ah, bt, bh) in [(10u64, 5u64, 100u64, 50u64), (10, 10, 1000, 5), (5, 5, 1000, 1000)] {
            let a = ds(&format!("d0:x=0..1,heartbeat_ms={ah},ttl_ms={at},fail=hold"));
            let b = ds(&format!("d0:x=0..1,heartbeat_ms={bh},ttl_ms={bt},fail=hold"));
            let m = meet(&a, &b).expect("meet exists");
            assert!(m.ttl_ms >= m.heartbeat_ms, "meet must not expire before its first beat");
        }
    }

    #[test]
    fn set_operations_refuse_a_device_the_parent_never_granted() {
        let parent = set(&[P]);
        let child = set(&["arm0/wrist:angle_deg=0..1,heartbeat_ms=200,ttl_ms=1000,fail=hold"]);
        assert!(!all_within(&child, &parent), "a device absent from the parent is a widening");
        assert!(intersect_device_sets(&child, &parent).is_empty());
        assert!(all_within(&BTreeMap::new(), &parent), "the empty set attenuates anything");
    }

    #[test]
    fn grants_device_answers_identity_only() {
        let g = set(&[P]);
        assert!(grants_device(&g, "arm0/elbow"));
        assert!(!grants_device(&g, "arm0/wrist"), "an ungranted device is refused, not defaulted");
        assert!(!grants_device(&BTreeMap::new(), "arm0/elbow"), "no device grants, no devices");
    }

    #[test]
    fn non_finite_and_inverted_bounds_are_refused_at_parse() {
        // `"NaN".parse::<f64>()` succeeds, so this branch is reachable from a grant string.
        assert!(parse("d0:x=NaN..1,heartbeat_ms=1,ttl_ms=1,fail=hold").is_err());
        assert!(parse("d0:x=0..inf,heartbeat_ms=1,ttl_ms=1,fail=hold").is_err());
        assert!(parse("d0:x=5..1,heartbeat_ms=1,ttl_ms=1,fail=hold").is_err(), "lo > hi bounds nothing");
    }

    #[test]
    fn every_mandatory_term_is_mandatory() {
        assert!(parse("d0:heartbeat_ms=1,ttl_ms=1,fail=hold").is_err(), "no dimensions");
        assert!(parse("d0:x=0..1,ttl_ms=1,fail=hold").is_err(), "no heartbeat");
        assert!(parse("d0:x=0..1,heartbeat_ms=1,fail=hold").is_err(), "no ttl");
        assert!(parse("d0:x=0..1,heartbeat_ms=1,ttl_ms=1").is_err(), "no fail-state");
        assert!(parse("d0:x=0..1,heartbeat_ms=0,ttl_ms=1,fail=hold").is_err(), "zero heartbeat");
        assert!(parse("d0:x=0..1,heartbeat_ms=10,ttl_ms=5,fail=hold").is_err(), "ttl < heartbeat");
        assert!(parse("x=0..1,heartbeat_ms=1,ttl_ms=1,fail=hold").is_err(), "no device");
    }

    #[test]
    fn the_grant_string_round_trips_canonically() {
        let d = ds(P);
        let s = d.to_grant_string();
        assert_eq!(parse(&s).unwrap(), d, "parse ∘ render is the identity");
        assert_eq!(parse(&s).unwrap().to_grant_string(), s, "and render is canonical");
        // Dimension order in the input must not survive into the output.
        let shuffled = ds("arm0/elbow:torque_nm=0..2.5,angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold");
        assert_eq!(shuffled.to_grant_string(), s, "sorted dims → byte-canonical regardless of input order");
    }
}
