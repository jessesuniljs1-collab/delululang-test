//! P8-01 — a control program in a guest (`V2_P8_DESIGN.md`, D-V2-51, D-V2-53).
//!
//! Until P8-01 a program that named an actuator or a sensor was refused under `--sandbox` ("it uses
//! Actuator. Nothing ran."). Minting already crossed the channel; USING did not: a guest's `command` and
//! `read` reached a host that knew no device. Now the host performs them, through the interpreter's own
//! body (`device::actuate`/`sense`), against the run's device broker — the same envelope, rate, lease,
//! dead-man and e-stop an ordinary run has. The guest holds a handle and nothing else.
//!
//! The witnesses, all against the simulator or a logging driver, on every OS CI runs:
//! 1. the same program, sandboxed and not, says the same thing, and the guest's out-of-envelope command
//!    never reaches the driver process (its own log is the evidence, as in `hw_adapter_cli.rs`);
//! 2. a guest that stops beating loses its actuator on the HOST's clock — told so if it asks again, and
//!    lost all the same if it never asks again; and a guest that keeps talking to its host without
//!    touching the device is not beating it;
//! 3. `delulu grants revoke` of the device's node engages the fail-state while the guest is mid-motion;
//! 4. the DL1905 sign-off gate binds a sandboxed hardware run: unapproved bytes start no driver and no
//!    guest.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
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

/// A scratch directory with a working directory and a state directory of its own. Short, and under `/tmp`
/// off Windows: the broker's socket lives in the state directory, and macOS allows a socket path 104 bytes —
/// its own temporary directory (`/var/folders/…/T/`) spends half of them (`HANDOFF.md` §11.5; this file's
/// e-stop witness went red on a macOS runner at 113).
fn scratch(tag: &str) -> (PathBuf, PathBuf) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = if cfg!(windows) { std::env::temp_dir() } else { PathBuf::from("/tmp") };
    let base = root.join(format!("dsd-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    (cwd, state)
}

const SAY: &str = "\
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
";

/// One command inside the envelope, one far outside it, and the joint read back.
fn arm_program() -> String {
    format!(
        "module arm\n\ntype Cmd {{ angle_deg: Float }}\n\n{SAY}\n\
         fn main(root: Root) ! {{Write, Actuate, Read}} {{\n\
         \x20 let c = root.console()\n\
         \x20 let a = root.actuator(\"arm0/elbow\")\n\
         \x20 c.println(say(a.command(Cmd {{ angle_deg: 12.0 }})))\n\
         \x20 c.println(say(a.command(Cmd {{ angle_deg: 999.0 }})))\n\
         \x20 let s = root.sensor(\"arm0/elbow#angle_deg\")\n\
         \x20 match s.read() {{\n\
         \x20   Ok(v) => c.println(\"READ \" + str(v)),\n\
         \x20   Err(e) => c.println(\"NO READING\")\n\
         \x20 }}\n\
         }}\n"
    )
}

const GRANT: &str = "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";
const SENSOR: &str = "sensor=arm0/elbow#angle_deg";

/// Witness 1: the guest is told exactly what the interpreter would tell it — the command inside the
/// envelope is commanded, the one outside it refused with the envelope's words, the joint reads back the
/// commanded angle — and the refusal is on the report's record, as a DL1904 the host gave.
#[test]
fn a_control_program_says_the_same_thing_in_a_guest_as_it_does_unsandboxed() {
    let (cwd, state) = scratch("same");
    std::fs::write(cwd.join("arm.delulu"), arm_program()).unwrap();
    let base = ["run", "arm.delulu", "--grant", "console", "--grant", GRANT, "--grant", SENSOR, "--broker-profile", "sim", "--no-prompt"];
    let plain = delulu_in(&cwd, &state, &base);
    assert!(plain.status.success(), "the ordinary run: {}", stderr(&plain));
    let mut sandboxed = base.to_vec();
    sandboxed.extend_from_slice(&["--sandbox", "--report-out", "rep.json"]);
    let guest = delulu_in(&cwd, &state, &sandboxed);
    assert!(guest.status.success(), "the sandboxed run must carry a control program now:\n{}", stderr(&guest));
    assert_eq!(stdout(&guest), stdout(&plain), "a guest is told what the interpreter would tell it");
    let lines: Vec<String> = stdout(&guest).lines().map(str::to_string).collect();
    assert_eq!(lines[0], "COMMANDED", "{lines:?}");
    assert!(lines[1].starts_with("REFUSED: ") && lines[1].contains("outside the envelope"), "{lines:?}");
    assert_eq!(lines[2], "READ 12.0", "the simulated joint reads back the angle the guest commanded: {lines:?}");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(cwd.join("rep.json")).expect("the report")).unwrap();
    let denied = report["sandbox"]["denied"].to_string();
    assert!(
        denied.contains("DL1904 arm0/elbow") && denied.contains("999"),
        "the host's refusal is on the report's record: {denied}"
    );
}

/// A driver that appends every request it receives to `adapter-saw.txt` and accepts it. Python on
/// Windows (started once untimed, as `hw_adapter_cli.rs` learned a cold interpreter misses the first
/// exchange's 2,000 ms), `sh` elsewhere.
fn logging_driver(cwd: &Path) -> String {
    #[cfg(windows)]
    {
        let script = "import sys\n\
             log = open('adapter-saw.txt', 'a')\n\
             while True:\n\
             \x20   line = sys.stdin.readline()\n\
             \x20   if not line:\n\
             \x20       break\n\
             \x20   l = line.rstrip('\\r\\n')\n\
             \x20   log.write(l + '\\n')\n\
             \x20   log.flush()\n\
             \x20   print('VAL 21.5' if l.startswith('READ') else 'OK', flush=True)\n";
        std::fs::write(cwd.join("driver.py"), script).unwrap();
        let warm = Command::new("python")
            .args(["-I", "-S", "driver.py"])
            .current_dir(cwd)
            .stdin(Stdio::null())
            .output();
        assert!(warm.as_ref().is_ok_and(|o| o.status.success()), "the test driver could not start: {warm:?}");
        let _ = std::fs::remove_file(cwd.join("adapter-saw.txt"));
        "python -I -S driver.py".to_string()
    }
    #[cfg(not(windows))]
    {
        let script = "#!/bin/sh\n\
             while IFS= read -r l; do\n\
             \x20 echo \"$l\" >> adapter-saw.txt\n\
             \x20 case \"$l\" in\n\
             \x20   READ*) echo 'VAL 21.5' ;;\n\
             \x20   *) echo 'OK' ;;\n\
             \x20 esac\n\
             done\n";
        std::fs::write(cwd.join("driver.sh"), script).unwrap();
        "sh driver.sh".to_string()
    }
}

/// Witness 1, at the process boundary that matters: a guest's out-of-envelope command never reaches the
/// driver at all — not "reaches it and is refused" — while its in-envelope command does. The guest holds
/// no envelope and no driver; the host checks before one byte crosses.
#[test]
fn a_guests_out_of_envelope_command_never_reaches_the_driver_process() {
    let (cwd, state) = scratch("driver");
    std::fs::write(cwd.join("arm.delulu"), arm_program()).unwrap();
    let driver = logging_driver(&cwd);
    // The sign-off DL1905 demands, written by the ordinary run's simulation of these exact bytes.
    let signed = delulu_in(
        &cwd,
        &state,
        &["run", "arm.delulu", "--grant", "console", "--grant", GRANT, "--grant", SENSOR, "--broker-profile", "sim", "--signoff", "signoff.json", "--no-prompt"],
    );
    assert!(signed.status.success(), "the simulation signs off: {}", stderr(&signed));
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", GRANT, "--grant", SENSOR,
            "--broker-profile", "hw:demo-adapter", "--approved", "signoff.json", "--adapter-cmd", &driver, "--no-prompt",
        ],
    );
    assert!(o.status.success(), "a refusal is a value; the guest's run exits 0:\n{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("COMMANDED") && out.contains("REFUSED"), "{out}");
    assert!(out.contains("READ 21.5"), "the driver's reading reached the guest:\n{out}");
    let saw = std::fs::read_to_string(cwd.join("adapter-saw.txt")).unwrap_or_default();
    assert!(saw.contains("angle_deg=12"), "the permitted command DID cross to the driver:\n{saw}");
    assert!(!saw.contains("999"), "THE GUEST'S REFUSED COMMAND MUST NEVER REACH THE DRIVER:\n{saw}");
}

/// A command, a burn longer than the heartbeat, then `tail`.
fn burner(burn: u32, tail: &str) -> String {
    format!(
        "module m\n\ntype Elbow {{ angle_deg: Float }}\n\n{SAY}\n\
         fn fib(n: Int) -> Int {{\n  if n < 2 {{ n }} else {{ fib(n - 1) + fib(n - 2) }}\n}}\n\n\
         fn burst(c: Cap[Console], n: Int) -> Int ! {{Write}} {{\n  if n <= 0 {{ 0 }} else {{ c.println(\".\")\n  burst(c, n - 1) }}\n}}\n\n\
         fn chat(c: Cap[Console], k: Cap[Clock], until: Int) -> Int ! {{Write, Clock}} {{\n  if k.now_ms() >= until {{ 0 }} else {{ let b = burst(c, 50)\n  chat(c, k, until) }}\n}}\n\n\
         fn main(root: Root) ! {{Write, Actuate, Clock}} {{\n\
         \x20 let c = root.console()\n\
         \x20 let a = root.actuator(\"arm0/elbow\")\n\
         \x20 c.println(say(a.command(Elbow {{ angle_deg: 10.0 }})))\n\
         \x20 c.println(\"burned \" + str(fib({burn})))\n\
         {tail}\
         }}\n"
    )
}

/// 150 ms: far more than a slow runner's guest needs between being sent its program and its first command (the
/// broker starts at the send, so the guest's own check of the program counts), far less than each burn below.
const SHORT_BEAT: &str = "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=150,ttl_ms=600000,fail=safe-park";

/// The beat overdue figure the host journaled — measured on the host's clock, never the guest's.
fn overdue_us(err: &str) -> u64 {
    let at = err.find("beat overdue by ").expect("the host journaled how late the beat was");
    err[at + "beat overdue by ".len()..].split(' ').next().unwrap().parse().expect("a number of µs")
}

/// Witness 2: the guest commands, then burns far past its 150 ms heartbeat without beating; the host's
/// watchdog revokes the lease on its own tick, the guest's next command is told `LeaseRevoked` with the
/// cause and the fail-state, and the host says which device it lost and how late the beat was.
#[test]
fn a_guest_that_stops_beating_loses_its_actuator_on_the_hosts_clock() {
    let (cwd, state) = scratch("beat");
    std::fs::write(cwd.join("arm.delulu"), burner(31, "  c.println(say(a.command(Elbow { angle_deg: 10.0 })))\n")).unwrap();
    let o = delulu_in(
        &cwd,
        &state,
        &["run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", SHORT_BEAT, "--broker-profile", "sim", "--no-prompt"],
    );
    assert!(o.status.success(), "losing a device kills the command, never the run:\n{}", stderr(&o));
    let out = stdout(&o);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "COMMANDED", "the first command was inside the heartbeat: {out}");
    assert!(
        lines[2].starts_with("REVOKED") && lines[2].contains("missed-heartbeat") && lines[2].contains("safe-park"),
        "the guest is told it lost the device, why, and what the machine does now: {out}\n{}",
        stderr(&o)
    );
    let err = stderr(&o);
    assert!(err.contains("devices: arm0/elbow: lease revoked (missed-heartbeat)"), "{err}");
}

/// Witness 2, with no cooperation at all: the guest commands once and then never asks its host for
/// anything again — it is ended by its wall-clock ceiling while still computing. The lease is lost all
/// the same, on the host's watchdog, within a bound of the heartbeat.
#[test]
fn a_wedged_guest_that_never_asks_again_still_loses_its_actuator() {
    let (cwd, state) = scratch("wedged");
    std::fs::write(cwd.join("arm.delulu"), burner(40, "")).unwrap();
    let started = Instant::now();
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", SHORT_BEAT, "--broker-profile", "sim",
            "--limits", "wall=3", "--no-prompt",
        ],
    );
    let took = started.elapsed();
    let err = stderr(&o);
    assert!(stdout(&o).starts_with("COMMANDED"), "the guest commanded before it wedged:\n{}\n{err}", stdout(&o));
    assert!(!stdout(&o).contains("burned"), "the guest never finished its burn — it was wedged:\n{}", stdout(&o));
    assert!(
        err.contains("devices: arm0/elbow: lease revoked (missed-heartbeat)"),
        "a guest that never speaks again loses its device on the host's clock:\n{err}"
    );
    // The watchdog's tick is a quarter of the heartbeat; a second is a bound a busy runner keeps.
    let late = overdue_us(&err);
    assert!(late < 1_000_000, "the lease died {late} µs after its beat was due — the watchdog was not watching:\n{err}");
    assert!(took < Duration::from_secs(60), "the wall ceiling ended the wedged guest: {took:?}");
}

/// Witness 2's other half: a guest that keeps its host busy — console and clock requests, 50 at a time, for
/// a second of its own clock (several heartbeats) — but never touches the device is NOT beating it. The beat
/// rides device activity, not channel activity, so a guest cannot keep a machine by chatting.
#[test]
fn a_guest_that_talks_to_its_host_without_touching_the_device_is_not_beating_it() {
    let (cwd, state) = scratch("chat");
    let tail = "  let k = root.clock()\n  let n = chat(c, k, k.now_ms() + 1000)\n  c.println(say(a.command(Elbow { angle_deg: 10.0 })))\n";
    std::fs::write(cwd.join("arm.delulu"), burner(2, tail)).unwrap();
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", "clock", "--grant", SHORT_BEAT,
            "--broker-profile", "sim", "--no-prompt",
        ],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.lines().filter(|l| *l == ".").count() >= 50, "the guest talked to its host: {}", out.len());
    let last = out.lines().last().unwrap_or_default();
    assert!(
        last.starts_with("REVOKED") && last.contains("missed-heartbeat"),
        "a second of console and clock requests over the channel did not beat the lease: {last}\n{}",
        stderr(&o)
    );
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

/// A supervisor that drives its arm in a loop and keeps going after a refusal (`estop_cli.rs`'s).
const SUPERVISOR: &str = "\
fn fib(n: Int) -> Int {
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn drive(c: Cap[Console], a: Cap[Actuator], n: Int) -> Int ! {Write, Actuate} {
  if n <= 0 {
    0
  } else {
    c.println(say(a.command(Elbow { angle_deg: 12.0 })))
    c.println(\"burn \" + str(fib(24)))
    drive(c, a, n - 1)
  }
}

fn main(root: Root) ! {Write, Actuate} {
  let c = root.console()
  let a = root.actuator(\"arm0/elbow\")
  c.println(\"SUPERVISOR UP\")
  let done = drive(c, a, 30)
  c.println(\"SUPERVISOR DOWN\")
}
";

/// The node holding one named device, found as an operator finds it: in the grant tree.
fn device_node(cwd: &Path, state: &Path, device: &str) -> Option<String> {
    let o = delulu_in(cwd, state, &["grants", "list", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).ok()?;
    v["nodes"].as_array()?.iter().find_map(|n| {
        let hit = n["holder_kind"].as_str() == Some("device")
            && n["holder_desc"].as_str() == Some(device)
            && n["state"].as_str() == Some("live");
        hit.then(|| n["id"].as_str().map(str::to_string)).flatten()
    })
}

/// Witness 3: the operator's e-stop reaches a guest's device mid-motion. A heartbeat and TTL far longer
/// than the test, so the only thing that can take the arm is `grants revoke` of the device's own node;
/// the guest is told `LeaseRevoked`, keeps running, and the host names the operator as the cause.
#[test]
fn an_operator_revoke_stops_a_guests_arm_mid_motion() {
    let (cwd, state) = scratch("estop");
    std::fs::write(cwd.join("sup.delulu"), format!("module m\n\ntype Elbow {{ angle_deg: Float }}\n\n{SAY}\n{SUPERVISOR}")).unwrap();
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };
    let child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&cwd)
        .env("DELULU_STATE_DIR", &state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args([
            "run", "sup.delulu", "--sandbox", "--broker", "daemon", "--grant", "console", "--grant",
            "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=600000,ttl_ms=600000,fail=safe-park",
            "--broker-profile", "sim", "--no-prompt",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the sandboxed supervisor");
    let deadline = Instant::now() + Duration::from_secs(30);
    let node = loop {
        if let Some(n) = device_node(&cwd, &state, "arm0/elbow") {
            break n;
        }
        assert!(Instant::now() < deadline, "the sandboxed run never published a node for arm0/elbow");
        std::thread::sleep(Duration::from_millis(20));
    };
    // Mid-motion: once the guest has commanded at least once.
    std::thread::sleep(Duration::from_millis(300));
    let o = delulu_in(&cwd, &state, &["grants", "revoke", &node]);
    assert!(o.status.success(), "grants revoke {node}: {}", stderr(&o));
    let out = child.wait_with_output().expect("the supervisor exits");
    let (so, se) = (String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string());
    let lines: Vec<&str> = so.lines().collect();
    let first = lines.iter().position(|l| l.starts_with("REVOKED:")).unwrap_or_else(|| panic!("a REVOKED line:\n{so}\n{se}"));
    assert!(lines[..first].contains(&"COMMANDED"), "the arm was driving before the e-stop:\n{so}");
    assert!(!lines[first..].contains(&"COMMANDED"), "no command succeeds after the revoke:\n{so}");
    assert!(lines[first].contains("operator-revoke") && lines[first].contains("safe-park"), "{so}");
    assert!(so.contains("SUPERVISOR DOWN"), "the guest survives losing its arm:\n{so}\n{se}");
    assert!(se.contains("devices: arm0/elbow: lease revoked (operator-revoke)"), "{se}");
    assert!(!se.contains("missed-heartbeat"), "nothing here missed a beat:\n{se}");
}

/// Witness 4: the DL1905 sign-off gate binds a sandboxed hardware run as it binds an ordinary one —
/// before a driver is started and before a guest exists.
#[test]
fn the_sign_off_gate_binds_a_sandboxed_hardware_run_and_starts_no_driver_and_no_guest() {
    let (cwd, state) = scratch("gate");
    std::fs::write(cwd.join("arm.delulu"), arm_program()).unwrap();
    let driver = logging_driver(&cwd);
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", GRANT, "--grant", SENSOR,
            "--broker-profile", "hw:demo-adapter", "--adapter-cmd", &driver, "--no-prompt",
        ],
    );
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "unapproved hardware must not run:\n{err}");
    assert!(err.contains("DL1905"), "by its code: {err}");
    assert!(!err.contains("sandbox: the guest"), "no guest was launched for unapproved bytes:\n{err}");
    assert!(stdout(&o).is_empty(), "nothing ran: {}", stdout(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("adapter-saw.txt")).unwrap_or_default(), "", "and no driver started");
}

/// A hardware run's own flags without a hardware profile are refused, sandboxed or not (found by this
/// slice's suite: once the sandboxed run applied the device flags, `--approved` beside a simulator run was
/// accepted there and did nothing — as the ordinary run had always done). An operator who passed them
/// believes a sign-off or a driver is in force.
#[test]
fn a_hardware_flag_without_a_hardware_profile_is_refused_sandboxed_or_not() {
    let (cwd, state) = scratch("hwflag");
    std::fs::write(cwd.join("arm.delulu"), arm_program()).unwrap();
    for sandbox in [false, true] {
        for flag in [vec!["--approved", "signoff.json"], vec!["--adapter-cmd", "drv"], vec!["--require-signed-adapter"]] {
            let mut args = vec!["run", "arm.delulu", "--grant", "console", "--grant", GRANT, "--grant", SENSOR, "--broker-profile", "sim"];
            if sandbox {
                args.push("--sandbox");
            }
            args.extend(flag.iter().copied());
            args.push("--no-prompt");
            let o = delulu_in(&cwd, &state, &args);
            let err = stderr(&o);
            assert_eq!(o.status.code(), Some(2), "sandbox={sandbox} {flag:?} was accepted and not applied:\n{err}");
            assert!(err.contains(&format!("`{}`", flag[0])) && err.contains("Nothing ran"), "{err}");
            assert!(stdout(&o).is_empty(), "nothing ran: {}", stdout(&o));
        }
    }
}

/// One command, seven out-of-envelope attempts — each an interaction that advances a stepped clock and
/// beats nothing (C39) — then one more command.
fn refused_in_between() -> String {
    format!(
        "module m\n\ntype Elbow {{ angle_deg: Float }}\n\n{SAY}\n\
         fn miss(c: Cap[Console], a: Cap[Actuator], n: Int) -> Int ! {{Write, Actuate}} {{\n\
         \x20 if n <= 0 {{ 0 }} else {{ c.println(say(a.command(Elbow {{ angle_deg: 999.0 }})))\n  miss(c, a, n - 1) }}\n}}\n\n\
         fn main(root: Root) ! {{Write, Actuate}} {{\n\
         \x20 let c = root.console()\n\
         \x20 let a = root.actuator(\"arm0/elbow\")\n\
         \x20 c.println(say(a.command(Elbow {{ angle_deg: 10.0 }})))\n\
         \x20 let n = miss(c, a, 7)\n\
         \x20 c.println(say(a.command(Elbow {{ angle_deg: 10.0 }})))\n\
         }}\n"
    )
}

/// The simulator's stepped clock is the HOST's, so a sandboxed simulation keeps it (D20): with each
/// interaction worth 1,000 simulated ms and a 5,000 ms heartbeat that the wall clock never reaches in this
/// run, the refused attempts outlast the heartbeat and the lease dies at the same attempt in a guest as in
/// the ordinary run — a function of the command sequence, not of how fast either ran. On the wall clock
/// every attempt would be an envelope refusal and the last command would be commanded.
#[test]
fn a_sandboxed_simulation_keeps_the_stepped_clock_and_loses_the_lease_on_the_same_command() {
    let (cwd, state) = scratch("stepped");
    std::fs::write(cwd.join("arm.delulu"), refused_in_between()).unwrap();
    let base = [
        "run", "arm.delulu", "--grant", "console", "--grant",
        "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=5000,ttl_ms=600000,fail=hold", "--broker-profile", "sim",
        "--sim-step", "1000", "--no-prompt",
    ];
    let plain = delulu_in(&cwd, &state, &base);
    assert!(plain.status.success(), "{}", stderr(&plain));
    let mut args = base.to_vec();
    args.push("--sandbox");
    let guest = delulu_in(&cwd, &state, &args);
    assert!(guest.status.success(), "{}", stderr(&guest));
    assert_eq!(stdout(&guest), stdout(&plain), "the same command loses the lease, sandboxed or not");
    let lines: Vec<String> = stdout(&guest).lines().map(str::to_string).collect();
    assert_eq!(lines[0], "COMMANDED", "{lines:?}");
    assert!(
        lines.last().is_some_and(|l| l.starts_with("REVOKED") && l.contains("missed-heartbeat")),
        "eight simulated seconds outlast a five-second heartbeat that the wall clock never reached: {lines:?}"
    );
    assert!(
        lines[1].starts_with("REFUSED") && lines.iter().filter(|l| l.starts_with("REVOKED")).count() >= 2,
        "refused while the lease lived, then revoked, at a point set by the sequence: {lines:?}"
    );
}

/// A clean sandboxed simulation signs the artifact off, as the ordinary one does — and that sign-off opens
/// the DL1905 gate for a sandboxed hardware run of the same bytes. A simulation whose guest faulted signs
/// nothing.
#[test]
fn a_clean_sandboxed_simulation_signs_off_and_a_faulted_one_does_not() {
    let (cwd, state) = scratch("signoff");
    std::fs::write(cwd.join("arm.delulu"), arm_program()).unwrap();
    let sim = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", GRANT, "--grant", SENSOR,
            "--broker-profile", "sim", "--signoff", "signoff.json", "--no-prompt",
        ],
    );
    assert!(sim.status.success(), "{}", stderr(&sim));
    assert!(cwd.join("signoff.json").is_file(), "a clean sandboxed simulation signs off:\n{}", stderr(&sim));
    let driver = logging_driver(&cwd);
    let hw = delulu_in(
        &cwd,
        &state,
        &[
            "run", "arm.delulu", "--sandbox", "--grant", "console", "--grant", GRANT, "--grant", SENSOR,
            "--broker-profile", "hw:demo-adapter", "--approved", "signoff.json", "--adapter-cmd", &driver, "--no-prompt",
        ],
    );
    assert!(hw.status.success(), "the sandboxed sign-off opens the gate:\n{}", stderr(&hw));
    assert!(stdout(&hw).starts_with("COMMANDED"), "{}", stdout(&hw));

    // A guest that faults: its simulation approves nothing.
    std::fs::write(
        cwd.join("bad.delulu"),
        "module m\n\nfn main(root: Root) ! {Write, Actuate} {\n  let a = root.actuator(\"arm0/elbow\")\n  assert_eq(1, 2)\n}\n",
    )
    .unwrap();
    let faulted = delulu_in(
        &cwd,
        &state,
        &[
            "run", "bad.delulu", "--sandbox", "--grant", GRANT, "--broker-profile", "sim", "--signoff", "bad.json",
            "--no-prompt",
        ],
    );
    assert!(!faulted.status.success(), "the guest faulted: {}", stderr(&faulted));
    assert!(!cwd.join("bad.json").exists(), "a faulted simulation signs nothing off:\n{}", stderr(&faulted));
    assert!(stderr(&faulted).contains("sign-off withheld"), "{}", stderr(&faulted));
}
