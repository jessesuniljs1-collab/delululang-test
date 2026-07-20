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
