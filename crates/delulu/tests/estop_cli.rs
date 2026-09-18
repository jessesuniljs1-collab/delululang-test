//! Stage 10 phase 10g — the operator e-stop, end to end through the real binary (Track D,
//! spec §5.2, criterion 4's fourth behavior).
//!
//! 10f made a lease die when the program stopped beating. This file tests the case where the
//! program is *fine* — beating, driving in-envelope, doing everything right — and a human decides
//! it must stop anyway. That is a different mechanism and it lives in a different place: the grant
//! tree. `delulu grants revoke` kills the node, and the arm parks.
//!
//! Three properties, and the third is the one worth the process spawn:
//!
//! 1. **`Actuate` is in the node's authority.** Since 10g an actuator grant shows up in
//!    `grants list` — which is how an operator finds the node holding a machine in the first
//!    place. An e-stop you cannot aim is not an e-stop.
//! 2. **Revocation reaches the device without the program's help.** The watchdog probes the grant
//!    tree; a wedged program parks exactly like a cooperative one.
//! 3. **The program is told, and survives to say so.** Losing the arm returns `LeaseRevoked` — a
//!    value. The supervisor keeps running, which is what lets it park anything else it holds.
//!
//! This is the one file in 10g allowed to spawn a real daemon (the 5f rule); everything else about
//! the e-stop is unit-tested in `device.rs` against a probe, including the branch where the broker
//! cannot answer at all.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
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

/// Stops the daemon on drop so a failed assertion never strands a process holding the pipe.
struct DaemonGuard {
    state: PathBuf,
}
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

/// A supervisor that drives its arm in a loop, reporting every outcome, and keeps going after a
/// refusal. `fib` is the burn — this runtime has no sleep — so the loop spans enough wall clock
/// for an operator to act in the middle of it, which is the entire scenario.
const SUPERVISOR: &str = "\
module m

type Elbow { angle_deg: Float, velocity_dps: Float }

fn fib(n: Int) -> Int {
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn say(r: Result[Unit, ActuateErr]) -> Str {
  match r {
    Ok(u) => \"COMMANDED\",
    Err(e) => match e {
      Envelope(reason) => \"REFUSED: \" + reason,
      LeaseRevoked(reason) => \"REVOKED: \" + reason,
      NoDevice => \"NODEVICE\"
    }
  }
}

fn drive(c: Cap[Console], a: Cap[Actuator], n: Int) -> Int ! {Write, Actuate} {
  if n <= 0 {
    0
  } else {
    c.println(say(a.command(Elbow { angle_deg: 12.0, velocity_dps: 4.0 })))
    c.println(\"burn \" + str(fib(24)))
    drive(c, a, n - 1)
  }
}

fn main(root: Root) ! {Write, Actuate} {
  let c = root.console()
  let a = root.actuator(\"arm0/elbow\")
  c.println(\"SUPERVISOR UP\")
  let done = drive(c, a, 14)
  c.println(\"SUPERVISOR DOWN\")
}
";

/// A heartbeat and TTL far longer than the test can possibly run. This is what makes the test
/// prove what it claims: if the arm is lost, the ONLY mechanism that could have taken it is the
/// operator's revoke. Without this the dead-man from 10f would happily produce a green test for
/// the wrong reason.
const GRANT: &str = "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,\
heartbeat_ms=600000,ttl_ms=600000,fail=safe-park";

struct Fixture {
    cwd: PathBuf,
    state: PathBuf,
}

fn setup(tag: &str) -> Fixture {
    let base = std::env::temp_dir().join(format!("delulu-estop-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(cwd.join("supervisor.delulu"), SUPERVISOR).unwrap();
    Fixture { cwd, state }
}

/// The node holding one named device, found the way an operator finds it: by listing the grant
/// tree and looking for the child whose holder IS that device. Returns `(id, parent)`, or `None`
/// until the run has minted it.
///
/// Aiming at the device's own node rather than the run's is the whole point of 10g's subtree.
/// The run's node carries `Actuate` too — it must, or it could not have delegated it — so an
/// operator who revokes *that* takes the program's console with the arm. Both are legitimate
/// e-stops; they stop different amounts of machine, and the tree says which is which.
fn device_node(cwd: &Path, state: &Path, device: &str) -> Option<(String, String)> {
    let o = delulu_in(cwd, state, &["grants", "list", "--json"]);
    if !o.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).ok()?;
    v["nodes"].as_array()?.iter().find_map(|n| {
        let is_device = n["holder_kind"].as_str() == Some("device")
            && n["holder_desc"].as_str() == Some(device)
            && n["state"].as_str() == Some("live");
        if !is_device {
            return None;
        }
        // A device node carries `Actuate` and nothing else: it is an attenuation of the run's
        // authority, not a second grant beside it.
        let effects: Vec<&str> = n["effects"].as_array()?.iter().filter_map(|e| e.as_str()).collect();
        assert_eq!(effects, vec!["Actuate"], "a device node carries exactly {{Actuate}}");
        Some((n["id"].as_str()?.to_string(), n["parent"].as_str()?.to_string()))
    })
}

#[test]
fn an_operator_revoke_stops_a_healthy_arm_and_the_supervisor_lives_to_report_it() {
    let f = setup("revoke");

    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    // ----- the supervisor comes up and starts driving -----------------------------------------
    let child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&f.cwd)
        .env("DELULU_STATE_DIR", &f.state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args([
            "run",
            "supervisor.delulu",
            "--broker",
            "daemon",
            "--grant",
            "console",
            "--grant",
            GRANT,
            "--broker-profile",
            "sim",
            "--trace-effects",
            "--no-prompt",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the supervisor");

    // ----- the operator finds the node holding the machine -------------------------------------
    // Property 1: the device is visible in the tree, by name, as a child of the run. Before 10g
    // this search could not have succeeded — an actuator grant left no trace in the grant tree at
    // all, so there was nothing for an operator to aim at.
    let deadline = Instant::now() + Duration::from_secs(20);
    let (node, parent) = loop {
        if let Some(n) = device_node(&f.cwd, &f.state, "arm0/elbow") {
            break n;
        }
        assert!(Instant::now() < deadline, "the run never published a node for arm0/elbow");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_ne!(node, parent, "the device's node is a CHILD, not the run's own");

    // ----- e-stop ------------------------------------------------------------------------------
    let pulled = Instant::now();
    let o = delulu_in(&f.cwd, &f.state, &["grants", "revoke", &node]);
    assert!(o.status.success(), "grants revoke {node}: {}", stderr(&o));
    let revoke_returned = pulled.elapsed();

    let out = child.wait_with_output().expect("the supervisor exits");
    let so = String::from_utf8_lossy(&out.stdout).to_string();
    let se = String::from_utf8_lossy(&out.stderr).to_string();

    // Property 3: the program was told, in the language, and kept running afterwards.
    assert!(so.contains("SUPERVISOR UP"), "the run must have started: {so}\n{se}");
    assert!(
        so.contains("REVOKED:"),
        "losing the arm must reach the program as LeaseRevoked:\nstdout:\n{so}\nstderr:\n{se}"
    );
    assert!(
        so.contains("SUPERVISOR DOWN"),
        "the supervisor must SURVIVE losing its arm — a revoked device is not a crashed process:\n{so}"
    );
    // It commanded successfully before the revoke and never again after. A run that only ever
    // printed REVOKED would pass the assertion above while proving nothing about the e-stop.
    let lines: Vec<&str> = so.lines().collect();
    let first_revoked = lines.iter().position(|l| l.starts_with("REVOKED:")).expect("a REVOKED line");
    assert!(
        lines[..first_revoked].iter().any(|l| l.trim() == "COMMANDED"),
        "the arm must have been driving BEFORE the e-stop, or the test proves nothing:\n{so}"
    );
    assert!(
        !lines[first_revoked..].iter().any(|l| l.trim() == "COMMANDED"),
        "no command may succeed after the revoke:\n{so}"
    );

    // Property 2: the fail-state engaged, journaled with the operator as the cause and the audit
    // seq that killed the node — not a fabricated missed heartbeat.
    assert!(
        se.contains("lease.revoked") && se.contains("operator-revoke"),
        "the trace must record the operator revocation:\n{se}"
    );
    assert!(
        se.contains("failstate.engaged"),
        "the declared fail-state must actually engage:\n{se}"
    );
    assert!(
        !se.contains("missed-heartbeat"),
        "nothing here missed a beat — a 600 s heartbeat cannot expire in this test:\n{se}"
    );
    // A sanity bound, not a published latency figure: the measured numbers live in
    // measurements/robotics-demo/RECORD.md, taken over repeated runs. This only catches an e-stop
    // that has silently become unbounded.
    assert!(
        revoke_returned < Duration::from_secs(10),
        "`grants revoke` itself took {revoke_returned:?}"
    );
}

/// The control case. The identical program, the identical grant, the identical daemon — and
/// nobody revokes anything. Every command succeeds and the arm is still held at exit.
///
/// This is the test that makes the one above mean something. An implementation that parked the
/// arm on a timer, on the first command, or on any daemon round-trip at all would sail through
/// the e-stop test and die here.
#[test]
fn with_nobody_revoking_anything_the_same_supervisor_keeps_its_arm() {
    let f = setup("control");

    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let o = delulu_in(
        &f.cwd,
        &f.state,
        &[
            "run",
            "supervisor.delulu",
            "--broker",
            "daemon",
            "--grant",
            "console",
            "--grant",
            GRANT,
            "--broker-profile",
            "sim",
            "--trace-effects",
            "--no-prompt",
        ],
    );
    let so = stdout(&o);
    let se = stderr(&o);
    assert!(o.status.success(), "the control run must succeed:\n{so}\n{se}");
    assert!(so.contains("SUPERVISOR DOWN"), "{so}");
    assert!(
        !so.contains("REVOKED:") && !so.contains("REFUSED:"),
        "an unrevoked, in-envelope run loses nothing:\n{so}"
    );
    assert!(
        !se.contains("lease.revoked"),
        "no lease may be revoked when no operator revoked one:\n{se}"
    );
    assert_eq!(
        so.matches("COMMANDED").count(),
        14,
        "every one of the 14 commands must land:\n{so}"
    );
}

/// The other e-stop, one level up: revoking the RUN's node — the parent — must also stop the arm,
/// transitively, without anyone naming the device. This is the fleet-level stop from addendum §2.4
/// ("revoking the fleet kills everything"), and it is the broker's transitive revocation doing the
/// work, not a second mechanism.
///
/// The cost is real and is the reason both exist: this stop takes the console with it, so the
/// program cannot report anything afterwards. An operator chooses which blast radius they want.
#[test]
fn revoking_the_parent_stops_the_arm_too_and_takes_the_program_with_it() {
    let f = setup("parent");

    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&f.cwd)
        .env("DELULU_STATE_DIR", &f.state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args([
            "run",
            "supervisor.delulu",
            "--broker",
            "daemon",
            "--grant",
            "console",
            "--grant",
            GRANT,
            "--broker-profile",
            "sim",
            "--trace-effects",
            "--no-prompt",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the supervisor");

    let deadline = Instant::now() + Duration::from_secs(20);
    let parent = loop {
        if let Some((_, p)) = device_node(&f.cwd, &f.state, "arm0/elbow") {
            break p;
        }
        assert!(Instant::now() < deadline, "the run never published a node for arm0/elbow");
        std::thread::sleep(Duration::from_millis(20));
    };

    let o = delulu_in(&f.cwd, &f.state, &["grants", "revoke", &parent]);
    assert!(o.status.success(), "grants revoke {parent}: {}", stderr(&o));

    let out = child.wait_with_output().expect("the supervisor exits");
    let se = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        se.contains("lease.revoked") && se.contains("operator-revoke"),
        "revoking the parent must reach the device through the tree:\n{se}"
    );
    assert!(se.contains("failstate.engaged"), "the fail-state must engage:\n{se}");
    // And the program itself loses everything, console included — DL1403 on its next effect. This
    // assertion documents the blast radius rather than treating it as an accident.
    assert!(
        se.contains("DL1403"),
        "revoking the run's own node ends the run — say so plainly:\n{se}"
    );
}

/// A device's grant node must not outlive the run that minted it.
///
/// This was found by the measurement harness, not by review: device nodes were staying `[live]`
/// after their process was long gone, so `delulu grants list` offered an operator several arms and
/// no way to tell which one anybody held. Revoking a ghost prints `ok: revoked 1 node(s)` and
/// stops nothing — a successful-looking e-stop is worse than a missing one, because it ends the
/// search for the real one.
///
/// The `(process)` node still outlives the run; that is Stage 5 behaviour shared with every other
/// run and relied on by `run --lease`. The assertion below is deliberately about `(device)` nodes
/// only, which is what an e-stop is aimed at.
#[test]
fn a_device_node_does_not_outlive_the_run_that_minted_it() {
    let f = setup("ghosts");
    let quick = "\
module m
type Elbow { angle_deg: Float, velocity_dps: Float }
fn main(root: Root) ! {Write, Actuate} {
  let c = root.console()
  let a = root.actuator(\"arm0/elbow\")
  match a.command(Elbow { angle_deg: 10.0, velocity_dps: 4.0 }) {
    Ok(u) => c.println(\"ok\"),
    Err(e) => c.println(\"err\")
  }
}
";
    std::fs::write(f.cwd.join("quick.delulu"), quick).unwrap();

    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    for _ in 0..3 {
        let o = delulu_in(
            &f.cwd,
            &f.state,
            &[
                "run", "quick.delulu", "--broker", "daemon", "--grant", "console", "--grant",
                GRANT, "--broker-profile", "sim", "--no-prompt",
            ],
        );
        assert!(o.status.success(), "{}", stderr(&o));
    }

    let o = delulu_in(&f.cwd, &f.state, &["grants", "list"]);
    assert!(o.status.success(), "grants list: {}", stderr(&o));
    let listing = stdout(&o);
    let ghosts: Vec<&str> = listing
        .lines()
        .filter(|l| l.contains("(device)") && l.contains("[live]"))
        .collect();
    assert!(
        ghosts.is_empty(),
        "three finished runs left {} live device node(s) for an operator to aim at:\n{listing}",
        ghosts.len()
    );
    // The nodes are still THERE — revoked, not deleted. The audit chain is a record of what was
    // held and when, and erasing it to tidy a listing would trade evidence for cosmetics.
    assert!(
        listing.lines().filter(|l| l.contains("(device)")).count() >= 3,
        "the revoked device nodes must remain visible in the audit record:\n{listing}"
    );
}

/// The grant tree is how the e-stop is aimed, so what it *shows* is part of the mechanism.
/// `Actuate` appears in the authority of a node granted an actuator, and does not appear in one
/// granted only a sensor — observation is not command, and the tree must not blur them.
///
/// `authority` answers from the PROGRAM, not from grants: these calls used to pass `--grant`
/// flags it never read (P1-F3 now refuses them), so what is asserted is the program's own answer.
#[test]
fn the_grant_tree_shows_actuate_for_an_actuator_and_not_for_a_sensor() {
    let f = setup("authority");
    let prog = "module m\nfn main(root: Root) ! {Write} { let c = root.console()\n c.println(\"hi\") }\n";
    std::fs::write(f.cwd.join("quiet.delulu"), prog).unwrap();

    let o = delulu_in(
        &f.cwd,
        &f.state,
        &["authority", "supervisor.delulu", "--json"],
    );
    assert!(o.status.success(), "authority: {}", stderr(&o));
    assert!(
        stdout(&o).contains("Actuate"),
        "an actuator grant must show Actuate in the authority answer:\n{}",
        stdout(&o)
    );

    // A sensor grant alone confers no Actuate — and no `Read` either. A sensor read is `Read` with
    // a SENSOR scope (spec §5.1); minting a bare `Read` effect here would hand the node the power
    // to read files, which nobody granted. The absence below is the whole point.
    let o = delulu_in(
        &f.cwd,
        &f.state,
        &["authority", "quiet.delulu", "--json"],
    );
    assert!(o.status.success(), "authority: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("json");
    let text = v.to_string();
    assert!(!text.contains("Actuate"), "a sensor grant is not a licence to command:\n{text}");
}
