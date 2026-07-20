//! Stage 10 phase 10g — the arm demonstration, run as a test (spec §5.5, criterion 4).
//!
//! Criterion 4 requires the demonstration to "reproduce from a clean checkout". A shell script
//! nobody runs is not a reproduction — it is a script that used to work. So the demonstration's
//! actual committed programs are executed here, on every `cargo test`, against the same CLI a
//! reader would type.
//!
//! Behaviors 1–3 and the DL1905 gate live here, in embedded custody, because they need no broker.
//! Behavior 4 (the operator e-stop) needs a real daemon and lives in `estop_cli.rs`, which is the
//! one file in this phase allowed to spawn one.
//!
//! These tests assert the demonstration's *claims*, not its formatting. The numbers themselves are
//! measured separately and published in `measurements/robotics-demo/RECORD.md`; a test that pinned
//! a latency would fail on a slow CI machine and teach everyone to ignore it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn demo(file: &str) -> String {
    repo_root().join("measurements").join("robotics-demo").join(file).to_string_lossy().to_string()
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(repo_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

const DIMS: &str = "angle_deg=-30..95,velocity_dps=0..40,torque_nm=0..2.5";

/// The demonstration's grant, with a heartbeat far longer than any of these runs — so behaviors 1
/// and 2 cannot accidentally be demonstrating the dead-man instead of the envelope.
fn patient_grant() -> String {
    format!("actuator=arm0/elbow:{DIMS},heartbeat_ms=600000,ttl_ms=600000,fail=safe-park")
}

/// The 250 ms heartbeat behavior 3 is built to miss.
fn nominal_grant() -> String {
    format!("actuator=arm0/elbow:{DIMS},heartbeat_ms=250,ttl_ms=600000,fail=safe-park")
}

/// Behavior 1: the correct edit takes effect. Every one of the twenty in-envelope commands lands,
/// and nothing is refused or revoked along the way.
#[test]
fn behavior_1_the_nominal_sweep_lands_every_command() {
    let o = delulu(&[
        "run", &demo("arm.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--no-prompt",
    ]);
    let out = stdout(&o);
    assert!(o.status.success(), "the nominal sweep must succeed:\n{out}\n{}", stderr(&o));
    assert_eq!(
        out.lines().filter(|l| l.trim() == "COMMANDED").count(),
        20,
        "all twenty in-envelope commands must land:\n{out}"
    );
    assert!(!out.contains("REFUSED"), "nothing in the envelope may be refused:\n{out}");
    assert!(!out.contains("REVOKED"), "nothing may lose the arm here:\n{out}");
    assert!(out.contains("DONE"), "{out}");
}

/// Behavior 2: the agent's plausible tuning edit — 5.0 N·m against a 2.5 N·m envelope — is refused
/// per command, by name, and the controller survives to keep driving.
///
/// The surviving part is the half that is easy to lose: a refusal implemented as a fault would
/// satisfy "the command did not happen" while taking the control loop down with it.
#[test]
fn behavior_2_the_over_torque_edit_is_refused_and_the_controller_survives() {
    let o = delulu(&[
        "run", &demo("arm-overtorque.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--trace-effects", "--no-prompt",
    ]);
    let out = stdout(&o);
    let err = stderr(&o);
    assert!(o.status.success(), "a refused command is a value — the process exits 0:\n{out}\n{err}");
    assert_eq!(
        out.lines().filter(|l| l.starts_with("REFUSED:")).count(),
        1,
        "exactly the over-torque command is refused:\n{out}"
    );
    assert!(
        out.contains("torque_nm") && out.contains("2.5"),
        "the refusal names the dimension and the bound it exceeded:\n{out}"
    );
    assert_eq!(
        out.lines().filter(|l| l.trim() == "COMMANDED").count(),
        2,
        "the commands either side of the refusal must land — the controller kept the arm:\n{out}"
    );
    assert!(err.contains("DL1904"), "the refusal is on the record:\n{err}");
}

/// Behavior 3: the controller goes away for longer than its heartbeat and comes back to find the
/// arm gone — parked by a watchdog that asked it nothing.
///
/// `LeaseRevoked`, not `Envelope`: the program is told it no longer holds the machine, which is a
/// different fact from having asked for the wrong motion, and the two must not share a variant.
#[test]
fn behavior_3_a_wedged_controller_loses_the_arm_to_safe_park() {
    let o = delulu(&[
        "run", &demo("arm-wedged.delulu"), "--grant", "console", "--grant", &nominal_grant(),
        "--broker-profile", "sim", "--trace-effects", "--no-prompt",
    ]);
    let out = stdout(&o);
    let err = stderr(&o);
    assert!(o.status.success(), "losing a device is not a crash:\n{out}\n{err}");
    assert!(
        out.contains("REVOKED:") && out.contains("missed-heartbeat"),
        "the wedged controller must be told it lost the arm, and why:\n{out}"
    );
    assert!(
        err.contains("lease.revoked") && err.contains("failstate.engaged"),
        "both halves must be journaled — revoked, then the declared fail-state engaged:\n{err}"
    );
    assert!(
        err.contains("safe-park"),
        "the fail-state that engages is the one the operator declared:\n{err}"
    );
    // The first command landed. Without this the test would pass for a program that never reached
    // the arm at all.
    assert!(
        out.lines().any(|l| l.trim() == "COMMANDED"),
        "the controller must have been driving before it wedged:\n{out}"
    );
}

/// The control for behavior 3: the identical wedged program, granted a heartbeat long enough to
/// cover its thinking, keeps the arm. The dead-man fires because the beat was missed — not
/// because the program was slow, and not because a device grant expires on principle.
#[test]
fn behavior_3_control_a_generous_heartbeat_survives_the_same_long_think() {
    let o = delulu(&[
        "run", &demo("arm-wedged.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--trace-effects", "--no-prompt",
    ]);
    let out = stdout(&o);
    assert!(o.status.success(), "{out}\n{}", stderr(&o));
    assert!(
        !out.contains("REVOKED:"),
        "the same long think under a 600 s heartbeat must NOT lose the arm:\n{out}"
    );
    assert_eq!(
        out.lines().filter(|l| l.trim() == "COMMANDED").count(),
        2,
        "both commands land when the heartbeat covers the think:\n{out}"
    );
}

/// The staged-deploy gate: the demonstration's own program, pointed at a hardware profile with no
/// sign-off record, is refused (DL1905). No record is not "nothing to check".
#[test]
fn the_staged_deploy_gate_refuses_hardware_with_no_signoff() {
    let o = delulu(&[
        "run", &demo("arm.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "hw:demo-adapter", "--no-prompt",
    ]);
    let err = stderr(&o);
    assert!(!o.status.success(), "hardware with no approval must not run:\n{}", stdout(&o));
    assert!(err.contains("DL1905"), "the refusal is the artifact-hash gate:\n{err}");
}

/// The benchmark programs are part of the published method, so they are exercised too: the
/// baseline must do no actuation at all, and the two loops must land exactly what they claim.
/// A drift here would silently corrupt every number in RECORD.md.
#[test]
fn the_benchmark_programs_still_measure_what_the_record_says_they_do() {
    let o = delulu(&[
        "run", &demo("bench-none.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--trace-effects", "--no-prompt",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("baseline 2000"), "{}", stdout(&o));
    assert!(
        !stderr(&o).contains("\"effect\":\"Actuate\""),
        "the differential baseline must perform NO actuation, or it is not a baseline:\n{}",
        stderr(&o)
    );

    let o = delulu(&[
        "run", &demo("bench-ok.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--no-prompt",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("accepted 2000"), "every command must land:\n{}", stdout(&o));

    let o = delulu(&[
        "run", &demo("bench-refused.delulu"), "--grant", "console", "--grant", &patient_grant(),
        "--broker-profile", "sim", "--no-prompt",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(
        stdout(&o).contains("accepted 0"),
        "every command must be refused, or the refusal cost is measured against the wrong thing:\n{}",
        stdout(&o)
    );
}
