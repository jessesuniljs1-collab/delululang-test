//! Stage 10 phase 10f — dead-man leases, the reference simulator, and the sim-to-hardware gate
//! (Track D, spec §5.2/§5.4, invariants 47 and 48).
//!
//! 10e made the envelope refuse bad commands. This file tests the part that has to work when the
//! program itself stops working, and the part that decides whether a program may touch hardware
//! at all. Four laws, each tested from both sides:
//!
//! 1. **A device grant without a dead-man is not a grant.** `heartbeat_ms`, `ttl_ms` and `fail`
//!    are mandatory, and each missing one is refused BY NAME — an operator who forgot a term is
//!    told which, not handed a syntax summary to diff by eye.
//! 2. **The lease dies on its own.** The watchdog owes the program nothing: a control loop that
//!    stops driving its device loses it, and the fail-state engages whether or not the
//!    interpreter ever runs another instruction. The control case matters just as much — a
//!    dead-man that fires under a healthy program teaches operators to disable it.
//! 3. **Losing the device is not one more bad setpoint.** `LeaseRevoked` is a separate variant
//!    from `Envelope` because the correct reaction differs: you clamp and retry a bad setpoint,
//!    and you STOP when you no longer hold the machine.
//! 4. **The hardware gate says no when it cannot tell.** No sign-off record is not "nothing to
//!    check" — it is a refusal (DL1905). That is the branch this phase exists to get right.

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
    let d = std::env::temp_dir().join(format!("delulu-deadman-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A program that commands, burns a measurable amount of wall clock, then commands again and
/// reports each outcome. The burn is `fib` — the corpus's own recursion — because this runtime
/// has no sleep, and "the program was busy elsewhere" is exactly the situation a dead-man is for.
fn burner_program(burn: u32) -> String {
    format!(
        "module m\n\n\
         type Elbow {{ angle_deg: Float, velocity_dps: Float }}\n\n\
         fn fib(n: Int) -> Int {{\n\
         \x20   if n < 2 {{ n }} else {{ fib(n - 1) + fib(n - 2) }}\n\
         }}\n\n\
         fn say(r: Result[Unit, ActuateErr]) -> Str {{\n\
         \x20   match r {{\n\
         \x20       Ok(u) => \"COMMANDED\",\n\
         \x20       Err(e) => match e {{\n\
         \x20           Envelope(reason) => \"REFUSED: \" + reason,\n\
         \x20           LeaseRevoked(reason) => \"REVOKED: \" + reason,\n\
         \x20           NoDevice => \"NODEVICE\"\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n\n\
         fn main(root: Root) ! {{Write, Actuate}} {{\n\
         \x20   let c = root.console()\n\
         \x20   let a = root.actuator(\"arm0/elbow\")\n\
         \x20   let cmd = Elbow {{ angle_deg: 10.0, velocity_dps: 4.0 }}\n\
         \x20   c.println(say(a.command(cmd)))\n\
         \x20   c.println(\"burned \" + str(fib({burn})))\n\
         \x20   c.println(say(a.command(cmd)))\n\
         }}\n"
    )
}

/// The burn size. Large enough that a tree-walking interpreter spends well over 100 ms on it, so
/// a 25 ms heartbeat is missed by a wide margin rather than by a whisker.
const BURN: u32 = 27;

fn write_prog(tag: &str, name: &str, src: &str) -> PathBuf {
    let dir = scratch(tag);
    let f = dir.join(name);
    std::fs::write(&f, src).unwrap();
    f
}

// ----- law 1: a device grant without a dead-man is not a grant -------------------------------

/// Each missing term is refused, and the refusal names the term. The fourth case is the one a
/// careless operator actually hits: terms present but nonsensical together, where the lease would
/// expire before its first beat was ever due.
#[test]
fn a_grant_missing_a_dead_man_term_is_refused_and_says_which_one() {
    let f = write_prog("no-deadman", "arm.delulu", &burner_program(1));
    let cases: [(&str, &str); 4] = [
        ("actuator=arm0/elbow:angle_deg=0..90,ttl_ms=1000,fail=hold", "heartbeat_ms"),
        ("actuator=arm0/elbow:angle_deg=0..90,heartbeat_ms=100,fail=hold", "ttl_ms"),
        ("actuator=arm0/elbow:angle_deg=0..90,heartbeat_ms=100,ttl_ms=1000", "fail="),
        (
            "actuator=arm0/elbow:angle_deg=0..90,heartbeat_ms=1000,ttl_ms=100,fail=hold",
            "shorter than",
        ),
    ];
    for (grant, expected) in cases {
        let o = delulu(&["run", &f.to_string_lossy(), "--grant", "console", "--grant", grant]);
        assert!(!o.status.success(), "an incomplete device grant must not run: {grant}");
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            err.contains(expected),
            "the refusal for `{grant}` must name `{expected}`, got: {err}"
        );
    }
}

/// An unknown fail-state is refused rather than quietly meaning something. There is no sensible
/// default for "what this machine does when the software stops", so a typo cannot be absorbed.
#[test]
fn an_unknown_fail_state_is_refused_rather_than_interpreted() {
    let f = write_prog("bad-fail", "arm.delulu", &burner_program(1));
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "actuator=arm0/elbow:angle_deg=0..90,heartbeat_ms=100,ttl_ms=1000,fail=stop-i-guess",
    ]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("stop-i-guess") && err.contains("safe-park"),
        "the refusal names the typo AND the legal vocabulary: {err}"
    );
}

// ----- law 2 and 3: the lease dies on its own, and losing it is its own error ------------------

/// The phase's headline. Same program, same command, twice: the first is honored, then the
/// program goes and does something else for longer than its heartbeat allows, and the second
/// finds the device gone. Nothing in the program asked for this — the watchdog acted alone.
#[test]
fn a_program_that_stops_beating_loses_its_device_mid_run() {
    let f = write_prog("revoke", "arm.delulu", &burner_program(BURN));
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=25,ttl_ms=600000,fail=safe-park",
        "--broker-profile",
        "sim",
        "--trace-effects",
    ]);
    assert!(
        o.status.success(),
        "losing a device kills the COMMAND, never the process — the control program must stay \
         alive to react: {o:?}"
    );
    let out = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "COMMANDED", "the first command was inside the heartbeat");
    assert!(
        lines[2].starts_with("REVOKED"),
        "the second command found the lease dead, and said so as LeaseRevoked rather than as one \
         more envelope complaint: {out}"
    );
    assert!(
        lines[2].contains("missed-heartbeat") && lines[2].contains("safe-park"),
        "the program is told WHY it lost the device and what the machine is doing now: {out}"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("lease.revoked"), "the revocation is traced: {err}");
    assert!(err.contains("failstate.engaged"), "so is the fail-state: {err}");
    assert!(
        err.contains("arm0/elbow"),
        "both name the device — an audit that cannot say which actuator was lost is not an audit"
    );
    // The refusal is NOT a DL1904: an auditor counting envelope refusals must not find a lost
    // device in the tally.
    let revoked = err.find("command.revoked").expect("the refused command is traced");
    assert!(
        !err[revoked..].contains("DL1904"),
        "a dead lease is not an envelope refusal and must not borrow its code: {err}"
    );
}

/// The control, and the test that makes the one above mean anything. Identical program, identical
/// work, only the granted heartbeat differs — and the device survives. Without this, "the lease
/// was revoked" could just as well be "this phase revokes leases".
#[test]
fn the_same_program_keeps_its_device_when_the_heartbeat_allows_the_work() {
    let f = write_prog("keep", "arm.delulu", &burner_program(BURN));
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=600000,ttl_ms=600000,fail=safe-park",
        "--broker-profile",
        "sim",
    ]);
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "COMMANDED");
    assert_eq!(
        lines[2], "COMMANDED",
        "a generous heartbeat over the same work keeps the device: the revocation above was \
         caused by the heartbeat, not by the phase: {out}"
    );
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("lease.revoked"),
        "nothing was revoked, so nothing claims it was"
    );
}

/// `rate_hz` stops being decorative — build-order D10f named this gap in 10e and said it would
/// close here. Two commands back to back against a 1 Hz grant cannot both be honored.
#[test]
fn rate_hz_is_enforced_now_that_the_lease_machinery_exists() {
    let f = write_prog("rate", "arm.delulu", &burner_program(1));
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,rate_hz=1,heartbeat_ms=60000,ttl_ms=600000,fail=hold",
        "--broker-profile",
        "sim",
    ]);
    assert!(o.status.success(), "a rate refusal is a value like any other: {o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "COMMANDED");
    assert!(
        lines[2].starts_with("REFUSED") && lines[2].contains("1 Hz"),
        "the second command exceeded the granted rate and the refusal names it: {out}"
    );
}

// ----- the reference simulator (spec §5.4) ----------------------------------------------------

fn mirror_program() -> String {
    "module m\n\n\
     type Elbow { angle_deg: Float }\n\n\
     fn show(r: Result[Float, ActuateErr]) -> Str {\n\
     \x20   match r {\n\
     \x20       Ok(v) => \"READ \" + str(v),\n\
     \x20       Err(e) => match e {\n\
     \x20           Envelope(reason) => \"REFUSED: \" + reason,\n\
     \x20           LeaseRevoked(reason) => \"REVOKED: \" + reason,\n\
     \x20           NoDevice => \"NODEVICE\"\n\
     \x20       }\n\
     \x20   }\n\
     }\n\n\
     fn main(root: Root) ! {Write, Read, Actuate} {\n\
     \x20   let c = root.console()\n\
     \x20   let a = root.actuator(\"arm0/elbow\")\n\
     \x20   let mirror = root.sensor(\"arm0/elbow#angle_deg\")\n\
     \x20   let strain = root.sensor(\"arm0/strain\")\n\
     \x20   let r = a.command(Elbow { angle_deg: 42.5 })\n\
     \x20   c.println(show(mirror.read()))\n\
     \x20   c.println(show(strain.read()))\n\
     \x20   c.println(show(strain.read()))\n\
     }\n"
        .to_string()
}

const MIRROR_GRANTS: [&str; 3] = [
    "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=600000,fail=hold",
    "sensor=arm0/elbow#angle_deg",
    "sensor=arm0/strain",
];

fn run_mirror(tag: &str, extra: &[&str]) -> Output {
    let f = write_prog(tag, "mirror.delulu", &mirror_program());
    let mut args: Vec<String> =
        vec!["run".into(), f.to_string_lossy().into_owned(), "--grant".into(), "console".into()];
    for g in MIRROR_GRANTS {
        args.push("--grant".into());
        args.push(g.to_string());
    }
    for e in extra {
        args.push((*e).to_string());
    }
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    delulu(&refs)
}

/// The loop closes: command the joint, read the joint back. This is the only reading the
/// simulator offers with a physical meaning, and it must track the command exactly rather than
/// approximately — an arm that reports where it was told to go is the least a sim can do.
#[test]
fn the_simulator_reads_back_the_position_that_was_commanded() {
    let o = run_mirror("mirror", &["--broker-profile", "sim", "--seed", "7"]);
    assert!(o.status.success(), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "READ 42.5", "the mirror sensor reports the commanded angle: {out}");
}

/// Determinism under `--seed` (spec §5.4), and its converse. A seed that changes nothing is not a
/// seed, so the same run under a different seed must actually differ.
#[test]
fn the_simulator_replays_identically_under_a_seed_and_differs_across_seeds() {
    let a = run_mirror("seed-a", &["--broker-profile", "sim", "--seed", "7"]);
    let b = run_mirror("seed-b", &["--broker-profile", "sim", "--seed", "7"]);
    let c = run_mirror("seed-c", &["--broker-profile", "sim", "--seed", "8"]);
    assert_eq!(a.stdout, b.stdout, "same seed, same run, byte for byte");
    assert_ne!(a.stdout, c.stdout, "a different seed must produce a different run");
}

/// Invariant 50 survives the arrival of an adapter, which is when it was most at risk. Without a
/// simulator bound, every sensor still answers absence — the null path is not a leftover, it is
/// the answer whenever nothing is modelling the thing being asked about.
#[test]
fn without_a_simulator_every_sensor_still_reads_no_device() {
    let o = run_mirror("null-sensor", &[]);
    assert!(o.status.success(), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    for line in out.lines() {
        assert_eq!(line, "NODEVICE", "no adapter, no number, on every sensor: {out}");
    }
}

// ----- the sim-to-hardware artifact gate (spec §5.4, invariant 48, DL1905) ---------------------

const GATE_GRANT: &str =
    "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=60000,ttl_ms=600000,fail=safe-park";

fn sim_signoff(tag: &str) -> (PathBuf, PathBuf) {
    let f = write_prog(tag, "arm.delulu", &burner_program(1));
    let approval = f.parent().unwrap().join("approved.json");
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        GATE_GRANT,
        "--broker-profile",
        "sim",
        "--signoff",
        &approval.to_string_lossy(),
    ]);
    assert!(o.status.success(), "the sim run itself must be clean: {o:?}");
    assert!(approval.exists(), "a clean sim run writes its sign-off: {o:?}");
    (f, approval)
}

/// THE SKIP BRANCH, and the reason this gate exists at all. A hardware profile with no sign-off
/// record is not "nothing to check" — the gate cannot tell whether these bytes were ever
/// simulated, and when it cannot tell it says no. A gate that opens for want of evidence is not
/// a gate.
#[test]
fn a_hardware_profile_with_no_signoff_record_is_refused_dl1905() {
    let f = write_prog("hw-nosignoff", "arm.delulu", &burner_program(1));
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        GATE_GRANT,
        "--broker-profile",
        "hw:acme-arm",
    ]);
    assert!(!o.status.success(), "no approval, no hardware");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL1905"), "the refusal is the allocated code: {err}");
    assert!(
        String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        "the program never ran a line: {o:?}"
    );
}

/// An edit after sign-off is a different program at the end of a wire that moves something. The
/// edit here is a comment — deliberately the most innocuous change available, because the gate
/// compares bytes and not intentions.
#[test]
fn an_artifact_edited_after_signoff_is_refused_dl1905() {
    let (f, approval) = sim_signoff("hw-edited");
    let mut src = std::fs::read_to_string(&f).unwrap();
    src.push_str("\n// a comment added after the simulation signed this off\n");
    std::fs::write(&f, src).unwrap();
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        GATE_GRANT,
        "--broker-profile",
        "hw:acme-arm",
        "--approved",
        &approval.to_string_lossy(),
    ]);
    assert!(!o.status.success(), "changed bytes, changed program");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL1905"), "{err}");
    assert!(err.contains("blake3:"), "the refusal shows both hashes: {err}");
}

/// The passing branch, which has to exist or the two refusals above prove only that the gate is
/// shut. A matching sign-off gets THROUGH the gate — and then the run stops for a completely
/// different and honestly-stated reason.
///
/// **Updated by RFC 0001 dish 3 (build-order D23).** That reason used to be "no hardware adapter
/// ships in this tree", which was true when `Profile::Hw` had nothing behind it. There is now a
/// real adapter — an operator-supplied subprocess — so the honest wall moved: a `hw:` run stops
/// because no **driver was named**, and the fix is `--adapter-cmd`. The shape of the assertion is
/// deliberately unchanged: the gate is seen to PASS, and the run still refuses rather than
/// pretending to have commanded a machine. `hw_adapter_cli.rs` covers the case where a driver IS
/// supplied.
#[test]
fn a_matching_signoff_passes_the_gate_and_then_stops_for_want_of_an_adapter() {
    let (f, approval) = sim_signoff("hw-match");
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        GATE_GRANT,
        "--broker-profile",
        "hw:acme-arm",
        "--approved",
        &approval.to_string_lossy(),
    ]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("device sign-off: OK"),
        "the gate must be seen to PASS, or its refusals prove nothing: {err}"
    );
    assert!(!err.contains("DL1905"), "a matching artifact is not a DL1905: {err}");
    assert!(
        err.contains("--adapter-cmd") && err.contains("hw:acme-arm"),
        "and then the honest wall, naming the profile and what it lacks: {err}"
    );
    assert!(
        err.contains("would be a lie"),
        "…and why silence is not an option: a hw run that commanded nothing while reporting \
         success is the worst failure mode available here: {err}"
    );
    assert!(!o.status.success(), "a hardware profile with no driver is still a refusal");
}

/// Sign-off is evidence of a clean simulation, so a faulted run must not produce one. Approving
/// an artifact whose sim run fell over would make the gate certify the thing it exists to catch.
#[test]
fn a_faulted_sim_run_writes_no_signoff() {
    // The device is granted but the program mints one that was never granted — DL0703 at the
    // mint, so the run faults after the broker exists but before it finishes.
    let f = write_prog("signoff-fault", "arm.delulu", &burner_program(1));
    let approval = f.parent().unwrap().join("approved.json");
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        "actuator=arm9/other:angle_deg=0..1,heartbeat_ms=60000,ttl_ms=600000,fail=hold",
        "--broker-profile",
        "sim",
        "--signoff",
        &approval.to_string_lossy(),
    ]);
    assert!(!o.status.success(), "the run faults at the mint");
    assert!(
        !approval.exists(),
        "a run that did not complete cleanly approves nothing for hardware"
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("sign-off withheld"),
        "and the omission is stated, not silent: {o:?}"
    );
}

/// A sign-off can only come from a simulation. A null-adapter run commanded nothing and observed
/// nothing; letting it sign would make the gate a formality dressed as evidence.
#[test]
fn a_run_with_no_simulator_cannot_sign_an_artifact_off() {
    let f = write_prog("signoff-null", "arm.delulu", &burner_program(1));
    let approval = f.parent().unwrap().join("approved.json");
    let o = delulu(&[
        "run",
        &f.to_string_lossy(),
        "--grant",
        "console",
        "--grant",
        GATE_GRANT,
        "--signoff",
        &approval.to_string_lossy(),
    ]);
    assert!(!approval.exists(), "no simulator, no approval");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("--broker-profile sim"),
        "the refusal says what was missing: {o:?}"
    );
}
