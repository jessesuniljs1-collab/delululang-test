//! Stage 10 phase 10e — `Actuate` activates (Track D, spec §5.1/§5.3, invariants 49 and 50).
//!
//! Three laws meet in this file and each one is tested from both sides:
//!
//! 1. **The envelope is fail-closed.** A command is refused unless every field is numeric, names
//!    a bounded dimension, and lies inside that dimension's inclusive range. The skip branch — the
//!    case where the checker "couldn't tell" — is the one that matters: a field the envelope has
//!    never heard of must be REFUSED, not waved through for want of a bound.
//! 2. **The command dies, not the process.** A refusal is a `Result` VALUE (`ActuateErr`), never a
//!    fault. A robot that panics mid-motion is worse than one that declines a step and keeps its
//!    control loop alive. So every refusal test asserts exit 0.
//! 3. **Refusals are visible.** DL1904 is telemetry, not a diagnostic: the refusal lands in the
//!    effect trace as `command.refused`, next to the `command` record for the attempt itself. A
//!    physical refusal nobody can audit is indistinguishable from a physical refusal that never
//!    happened.
//!
//! Invariant 50 also gets its witness here: the null sensor adapter returns `NoDevice`, never a
//! number. An unbound sensor that answers `0.0` is a fabricated measurement, and a fabricated
//! measurement in a control loop is how things get destroyed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-actuate-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The envelope granted to `arm0/elbow` throughout: two bounded dimensions, one of them with a
/// negative lower bound (a joint that bends both ways is the ordinary case, not the exotic one),
/// plus the dead-man terms 10f made mandatory. The heartbeat is long relative to these tests so
/// nothing here races the watchdog — losing the lease is 10f's subject, not this file's.
const ARM_GRANT: &str = "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,\
                         heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";

/// A program that sends one command and prints what came back. The command record is bound to a
/// `let` first because record literals are deliberately not parsed in a `match` scrutinee.
fn arm_program(decl: &str, lit: &str) -> String {
    format!(
        "module m\n\n\
         {decl}\n\n\
         fn main(root: Root) ! {{Write, Actuate}} {{\n\
         \x20   let c = root.console()\n\
         \x20   let a = root.actuator(\"arm0/elbow\")\n\
         \x20   let cmd = {lit}\n\
         \x20   let r = a.command(cmd)\n\
         \x20   match r {{\n\
         \x20       Ok(u) => c.println(\"COMMANDED\"),\n\
         \x20       Err(e) => match e {{\n\
         \x20           Envelope(reason) => c.println(\"REFUSED: \" + reason),\n\
         \x20           LeaseRevoked(reason) => c.println(\"REVOKED: \" + reason),\n\
         \x20           NoDevice => c.println(\"NODEVICE\")\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    )
}

const ELBOW_DECL: &str = "type Elbow { angle_deg: Float, velocity_dps: Float }";

fn run_arm(tag: &str, decl: &str, lit: &str, grant: &str) -> Output {
    let dir = scratch(tag);
    let f = dir.join("arm.delulu");
    std::fs::write(&f, arm_program(decl, lit)).unwrap();
    delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        grant,
        "--trace-effects",
    ])
}

/// The happy path, and the audit record that proves it happened. Both bounds are inclusive, so
/// this command sits comfortably inside them; the trace carries `Actuate`/`command` and names the
/// device, because a trace that cannot say which actuator moved is not an audit.
#[test]
fn an_in_envelope_command_succeeds_and_traces_the_actuate() {
    let o = run_arm(
        "ok",
        ELBOW_DECL,
        "Elbow { angle_deg: 12.5, velocity_dps: 4.0 }",
        ARM_GRANT,
    );
    assert!(o.status.success(), "an in-envelope command runs: {o:?}");
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "COMMANDED");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("\"effect\":\"Actuate\""), "the effect is traced: {err}");
    assert!(err.contains("\"op\":\"command\""), "the op is traced: {err}");
    assert!(err.contains("arm0/elbow"), "the record names the device: {err}");
    assert!(
        !err.contains("command.refused"),
        "nothing was refused, so nothing claims it was: {err}"
    );
}

/// Out of range on a dimension the envelope DOES bound. The value is a `Result`, the process
/// survives (exit 0), the program handled it, and DL1904 is in the trace as `command.refused` —
/// alongside the ordinary `command` record for the attempt, in that order. Both records exist on
/// purpose: an attempt that was refused is still an attempt, and hiding it would hide intent.
#[test]
fn an_out_of_envelope_command_is_refused_as_a_value_and_traced_dl1904() {
    let o = run_arm(
        "range",
        ELBOW_DECL,
        "Elbow { angle_deg: 200.0, velocity_dps: 4.0 }",
        ARM_GRANT,
    );
    assert!(
        o.status.success(),
        "the command dies, not the process — a refusal is never a fault: {o:?}"
    );
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("REFUSED"), "the program saw an Envelope error: {out}");
    assert!(out.contains("200"), "the refusal names the offending value: {out}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL1904"), "the refusal is telemetry, not silence: {err}");
    assert!(err.contains("command.refused"), "the refusal op is named: {err}");
    assert!(err.contains("arm0/elbow"), "the refusal names the device: {err}");
    let attempt = err.find("\"op\":\"command\"").expect("the attempt is recorded");
    let refusal = err.find("command.refused").expect("the refusal is recorded");
    assert!(attempt < refusal, "attempt first, then refusal — the order is the story");
}

/// THE SKIP BRANCH, and the reason this phase has a fail-closed law at all. `torque_nm` is a
/// perfectly sensible dimension that this grant simply never mentioned. The tempting bug is to
/// check only the fields the envelope knows and let the rest through — which would mean an
/// envelope grants everything it forgot to mention. Unbounded means REFUSED.
#[test]
fn a_dimension_the_envelope_never_bounded_is_refused_not_waved_through() {
    let o = run_arm(
        "unknown-dim",
        "type Torque { torque_nm: Float }",
        "Torque { torque_nm: 0.5 }",
        ARM_GRANT,
    );
    assert!(o.status.success(), "still a value, still not a fault: {o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("REFUSED") && out.contains("torque_nm"),
        "an unbounded dimension is refused BY NAME — 0.5 is inside no range at all: {out}"
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("DL1904"),
        "and the refusal is auditable like any other"
    );
}

/// A non-numeric field cannot be bounded by a numeric envelope, so it is refused rather than
/// ignored. Same law, different shape: the envelope's answer to "I cannot tell" is always no.
#[test]
fn a_non_numeric_field_is_refused_because_no_range_can_bound_it() {
    let o = run_arm(
        "non-numeric",
        "type Mode { angle_deg: Str }",
        "Mode { angle_deg: \"fast\" }",
        ARM_GRANT,
    );
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("REFUSED") && out.contains("angle_deg"),
        "a bounded dimension name does not save a value the range cannot compare: {out}"
    );
}

/// Deny by default, twice over. Minting `arm0/elbow` against a grant for a DIFFERENT device is
/// DL0703 at the mint — this is the sharper case than a bare zero-grant run, because the
/// pre-flight sees an actuator grant and lets the program start. The refusal has to come from the
/// mint itself, matching the device name, or an authority to move one machine would silently
/// become an authority to move another.
#[test]
fn minting_a_device_the_grant_never_named_is_dl0703() {
    let o = run_arm(
        "wrong-device",
        ELBOW_DECL,
        "Elbow { angle_deg: 12.5, velocity_dps: 4.0 }",
        "actuator=arm1/gripper:width_mm=0..80,heartbeat_ms=60000,ttl_ms=60000,fail=hold",
    );
    assert!(
        !o.status.success(),
        "an ungranted device is a fault at the mint, not a refusal at the command"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL0703"), "ungranted authority is DL0703: {err}");
    assert!(err.contains("arm0/elbow"), "the refusal names what was asked for: {err}");
}

/// The zero-grant case is caught earlier still — at the pre-flight, before a line runs. A program
/// whose row contains `Actuate` with no actuator granted anywhere never starts.
#[test]
fn an_actuate_program_with_no_actuator_grant_refuses_at_the_pre_flight() {
    let dir = scratch("no-grant");
    let f = dir.join("arm.delulu");
    std::fs::write(
        &f,
        arm_program(ELBOW_DECL, "Elbow { angle_deg: 12.5, velocity_dps: 4.0 }"),
    )
    .unwrap();
    let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console"]);
    assert!(!o.status.success(), "no grant, no run");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        "",
        "the program never reached its first println"
    );
}

/// Invariant 50: no fabricated measurements. The null adapter — what a run gets when it binds no
/// `--broker-profile` — answers `NoDevice`, never a plausible-looking number. A control loop that
/// receives 0.0 from a sensor that isn't there will act on it. 10f gave sensors a simulator to
/// read from and left this path untouched: absence still reads as absence.
#[test]
fn an_unbound_sensor_reads_no_device_never_a_fabricated_number() {
    let dir = scratch("sensor");
    let f = dir.join("sense.delulu");
    std::fs::write(
        &f,
        "module m\n\n\
         fn main(root: Root) ! {Write, Read} {\n\
         \x20   let c = root.console()\n\
         \x20   let s = root.sensor(\"arm0/angle\")\n\
         \x20   let r = s.read()\n\
         \x20   match r {\n\
         \x20       Ok(v) => c.println(\"READ \" + str(v)),\n\
         \x20       Err(e) => match e {\n\
         \x20           Envelope(reason) => c.println(\"REFUSED: \" + reason),\n\
         \x20           LeaseRevoked(reason) => c.println(\"REVOKED: \" + reason),\n\
         \x20           NoDevice => c.println(\"NODEVICE\")\n\
         \x20       }\n\
         \x20   }\n\
         }\n",
    )
    .unwrap();
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "sensor=arm0/angle",
        "--trace-effects",
    ]);
    assert!(o.status.success(), "an honest `no device` is not an error: {o:?}");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        "NODEVICE",
        "the granted-but-unbound sensor reports absence, not a value"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("\"op\":\"read\"") && err.contains("\"effect\":\"Read\""),
        "a sensor read is plain `Read` — observation is observation, not a new effect: {err}"
    );
    assert!(err.contains("arm0/angle"), "the read names its device: {err}");
}

// ===================================================================================================
// RFC 0001 phase F1 (build-order D12e) — the device scope dimension crosses the crate boundary.
//
// `delulu-broker` depends on neither `delulu-runtime` nor vice versa (crate ruling 1), so the
// canonical device-grant STRING is the contract between them and each side parses it with its own
// parser. Two parsers for one grammar is a drift risk; these tests are the mechanical pin, and this
// crate is where both parsers are visible at once.
// ===================================================================================================

/// Every envelope the runtime can parse must render to a string the broker parses back to the SAME
/// authority. If the grammars ever diverge, this fails — rather than a device quietly losing a bound
/// on its way into the grant tree.
#[test]
fn device_grant_strings_round_trip_between_the_runtime_and_broker_parsers() {
    // Spanning the grammar: negative bounds, fractional bounds, an optional rate, several
    // dimensions in deliberately unsorted order, and each fail-state.
    let specs = [
        "arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=200,ttl_ms=60000,fail=hold",
        "arm0/wrist:torque_nm=0..2.5,angle_deg=-1.5..1.5,rate_hz=50,heartbeat_ms=100,ttl_ms=1000,fail=coast",
        "sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=1000,ttl_ms=600000,fail=safe-park",
        "battery0/bms:charge_a=0..12,soc_pct=15..90,discharge_a=0..40,heartbeat_ms=5000,ttl_ms=600000,fail=hold",
    ];
    for spec in specs {
        let rt = delulu_runtime::value::ActuatorEnvelope::parse(spec)
            .unwrap_or_else(|e| panic!("runtime parse of `{spec}` failed: {e}"));
        let br = delulu_broker::device_scope::parse(spec)
            .unwrap_or_else(|e| panic!("broker parse of `{spec}` failed: {e}"));

        // The two parsers agree on every field they both carry.
        assert_eq!(rt.device, br.device, "device name: {spec}");
        assert_eq!(rt.heartbeat_ms, br.heartbeat_ms, "heartbeat_ms: {spec}");
        assert_eq!(rt.ttl_ms, br.ttl_ms, "ttl_ms: {spec}");
        assert_eq!(rt.rate_hz, br.rate_hz, "rate_hz: {spec}");
        assert_eq!(rt.fail_state.name(), br.fail, "fail-state: {spec}");
        assert_eq!(rt.dims.len(), br.dims.len(), "dimension count: {spec}");
        for (d, lo, hi) in &rt.dims {
            assert_eq!(
                br.dims.get(d),
                Some(&(*lo, *hi)),
                "dimension `{d}` disagrees between the parsers: {spec}"
            );
        }
        // And the broker's canonical rendering re-parses to the same thing on BOTH sides, so the
        // string that actually travels to the grant tree is a fixed point.
        let canon = br.to_grant_string();
        assert_eq!(delulu_broker::device_scope::parse(&canon).unwrap(), br, "broker round-trip: {canon}");
        let rt2 = delulu_runtime::value::ActuatorEnvelope::parse(&canon)
            .unwrap_or_else(|e| panic!("runtime cannot re-read the canonical form `{canon}`: {e}"));
        assert_eq!(rt2.device, rt.device);
        assert_eq!((rt2.heartbeat_ms, rt2.ttl_ms), (rt.heartbeat_ms, rt.ttl_ms));
        assert_eq!(rt2.dims.len(), rt.dims.len(), "canonical form keeps every dimension: {canon}");
    }
}

/// Every spec in this corpus, and whether a device grant may say it. Shared by the bidirectional
/// agreement law below and by the per-shape refusals, so a shape can never be tested on one side of
/// the grammar and forgotten on the other.
///
/// The hostile half exists because the round-trip law above could not see any of it: that law says
/// "every envelope the RUNTIME can parse must render to a string the BROKER parses back the same",
/// which is one-directional and quantified over four hand-picked good specs. Three real divergences
/// lived underneath it (`HARDENING_CAMPAIGN.md` C40/C41/C42) — a law that proves less than it claims
/// is the same defect P10 found in the conformance coverage gate, in a different subsystem.
const ENVELOPE_CORPUS: &[(&str, bool, &str)] = &[
    // ----- legal: the grammar's full span --------------------------------------------------------
    ("arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=200,ttl_ms=60000,fail=hold", true, "several dims, negative bound"),
    ("arm0/wrist:torque_nm=0..2.5,rate_hz=50,heartbeat_ms=100,ttl_ms=1000,fail=coast", true, "fractional bound, rate"),
    ("sat0/wheels:slew_deg=-0.5..0.5,heartbeat_ms=1000,ttl_ms=600000,fail=safe-park", true, "safe-park"),
    ("d0:x=0..0,heartbeat_ms=1,ttl_ms=1,fail=hold", true, "a single-point interval is a real interval"),
    // ----- C41: a bound that is not a real number -------------------------------------------------
    ("d0:x=0..inf,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C41 infinite upper bound"),
    ("d0:x=-inf..inf,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C41 infinite both ways"),
    ("d0:x=-infinity..infinity,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C41 spelled out"),
    ("d0:x=NaN..1,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C41 NaN lower bound"),
    ("d0:x=0..NaN,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C41 NaN upper bound"),
    // ----- C40: a term stated twice ---------------------------------------------------------------
    ("d0:x=-30..95,x=-1..1,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C40 dim twice, tighter second"),
    ("d0:x=-1..1,x=-30..95,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C40 dim twice, wider second"),
    ("d0:x=0..1,heartbeat_ms=1,heartbeat_ms=60000,ttl_ms=60000,fail=hold", false, "C40 heartbeat twice"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,ttl_ms=60000,fail=hold", false, "C40 ttl twice"),
    ("d0:x=0..1,rate_hz=1,rate_hz=1000,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "C40 rate twice"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=hold,fail=coast", false, "C40 fail twice"),
    // ----- C42: a fail-state that is not one of the three -----------------------------------------
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=hodl", false, "C42 typo"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=", false, "C42 empty"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=safe_park", false, "C42 underscore, not hyphen"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=Hold", false, "C42 wrong case"),
    // ----- already refused by both before this pass; kept so the corpus is the whole contract -----
    ("d0:x=0..1,ttl_ms=1,fail=hold", false, "no heartbeat"),
    ("d0:x=0..1,heartbeat_ms=1,fail=hold", false, "no ttl"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1", false, "no fail-state"),
    ("d0:heartbeat_ms=1,ttl_ms=1,fail=hold", false, "bounds nothing"),
    ("d0:x=0..1,heartbeat_ms=0,ttl_ms=1,fail=hold", false, "zero heartbeat"),
    ("d0:x=0..1,heartbeat_ms=10,ttl_ms=5,fail=hold", false, "ttl < heartbeat"),
    ("d0:x=5..1,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "inverted"),
    (":x=0..1,heartbeat_ms=1,ttl_ms=1,fail=hold", false, "no device"),
    ("d0:x=0..1,heartbeat_ms=1,ttl_ms=1,fail=hold,bogus", false, "a part with no `=`"),
];

/// **THE LAW, in the direction that matters.** The two parsers must accept and refuse the SAME
/// strings — not merely agree on the ones they both accept.
///
/// A grant string is read twice: `delulu_broker::device_scope::parse` builds the authority that is
/// recorded, delegated, attenuated and audited, and `ActuatorEnvelope::parse` builds the capability
/// value the runtime enforces against a command. When the two disagree about what a string MEANS,
/// the record and the machine part company; when they disagree about whether it is legal at all, an
/// operator gets a grant no program can mint, or a program enforces a bound the authority never
/// recorded. Neither is discoverable from one side, which is why this test lives in the `delulu`
/// crate — the only place both parsers are visible at once.
#[test]
fn the_two_envelope_parsers_accept_and_refuse_exactly_the_same_strings() {
    for (spec, legal, why) in ENVELOPE_CORPUS {
        let rt = delulu_runtime::value::ActuatorEnvelope::parse(spec);
        let br = delulu_broker::device_scope::parse(spec);
        assert_eq!(
            rt.is_ok(),
            br.is_ok(),
            "the parsers disagree about whether this is a legal envelope ({why}): `{spec}`\n  \
             runtime: {}\n  broker:  {}",
            rt.as_ref().map(|_| "accepted".to_string()).unwrap_or_else(|e| format!("refused — {e}")),
            br.as_ref().map(|_| "accepted".to_string()).unwrap_or_else(|e| format!("refused — {e}")),
        );
        assert_eq!(
            rt.is_ok(),
            *legal,
            "this corpus entry says `{spec}` should be {} ({why}), and both parsers say otherwise",
            if *legal { "legal" } else { "refused" }
        );
        // And where both accept, every field they both carry must agree — the original law, kept.
        if let (Ok(rt), Ok(br)) = (rt, br) {
            assert_eq!(rt.device, br.device, "{spec}");
            assert_eq!((rt.heartbeat_ms, rt.ttl_ms, rt.rate_hz), (br.heartbeat_ms, br.ttl_ms, br.rate_hz), "{spec}");
            assert_eq!(rt.fail_state.name(), br.fail, "{spec}");
            assert_eq!(rt.dims.len(), br.dims.len(), "dimension count: {spec}");
            for (d, lo, hi) in &rt.dims {
                assert_eq!(br.dims.get(d), Some(&(*lo, *hi)), "dimension `{d}`: {spec}");
            }
        }
    }
}

/// The fail-state list is ONE list (`device_scope::FAIL_STATES`), and the runtime's `FailState` enum
/// must round-trip exactly it — no more, no fewer. Rust has no reflection over enum variants, so
/// this is the mechanical pin that stops a fourth state from being added to one side only.
#[test]
fn the_canonical_fail_state_list_is_exactly_what_the_runtime_implements() {
    use delulu_runtime::FailState;
    for name in delulu_broker::device_scope::FAIL_STATES {
        let parsed = FailState::parse(name).unwrap_or_else(|| panic!("runtime cannot parse canonical fail-state `{name}`"));
        assert_eq!(parsed.name(), *name, "`{name}` must round-trip through the runtime enum");
    }
    // The other direction: nothing outside the list parses. An exhaustive scan is impossible, so
    // this covers the shapes a drifting implementation would actually produce.
    for outside in ["", "hodl", "safe_park", "Hold", "park", "brake", "hold "] {
        assert!(
            FailState::parse(outside).is_none(),
            "`{outside}` is not in FAIL_STATES but the runtime accepted it — the two lists have drifted"
        );
    }
}

/// The skip branch for `spec_to_authority`'s `filter_map`: an unparseable device grant string is
/// DROPPED rather than failing the conversion. Dropping must be the *safe* direction — a device
/// absent from the map is a device nobody granted — so this pins that a garbled envelope produces a
/// refusal, never a widening.
#[test]
fn a_malformed_device_grant_string_refuses_rather_than_widening() {
    use delulu_broker::{Authority, Op, Scopes};
    // Every one of these is rejected by the broker parser; none may become "unbounded".
    let malformed = [
        "arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000",   // no fail-state
        "arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,fail=hold",      // no ttl
        "arm0/elbow:heartbeat_ms=200,ttl_ms=60000,fail=hold",           // bounds nothing
        "arm0/elbow:angle_deg=NaN..95,heartbeat_ms=200,ttl_ms=1,fail=hold", // non-finite bound
        "arm0/elbow:angle_deg=95..-30,heartbeat_ms=200,ttl_ms=60000,fail=hold", // inverted
        "arm0/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=200,fail=hold", // ttl < heartbeat
    ];
    for spec in malformed {
        assert!(
            delulu_broker::device_scope::parse(spec).is_err(),
            "`{spec}` must not parse — if it ever does, this test is guarding nothing"
        );
        // Exactly what `spec_to_authority` does with an unparseable entry: drop it.
        let device: std::collections::BTreeMap<String, delulu_broker::DeviceScope> =
            [spec].iter().filter_map(|s| delulu_broker::device_scope::parse(s).ok()).map(|d| (d.device.clone(), d)).collect();
        assert!(device.is_empty(), "the drop happened: {spec}");

        // …and the consequence is a REFUSAL at the first command, not an allow.
        let mut b = delulu_broker::Broker::new();
        let effects = [delulu_check::Effect::core_from_name("Actuate").unwrap()];
        let node = b.issue(
            delulu_broker::Holder::new("process", "device program", "pid:1"),
            Authority::new(effects, Scopes { device, ..Default::default() }),
            None,
        );
        let d = b.check(&node, Op::Actuate, Some("arm0/elbow"));
        assert!(
            d.denial().is_some(),
            "a dropped envelope must refuse the device, never leave it unbounded: {spec}"
        );
        assert_eq!(d.denial().unwrap().code(), "DL0904");
    }
}
