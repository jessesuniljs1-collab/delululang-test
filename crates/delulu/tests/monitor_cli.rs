//! P8-04 — what an out-of-band monitor reads, end to end through the real binary (P8-04's one
//! process-spawning test, the per-phase 5f rule).
//!
//! A monitor (`V2_P8_DESIGN.md` P8-04, D-V2-54) watches the hash-chained audit a run's broker writes
//! and quarantines a run by revoking it. Its design said everything it would read already existed:
//! "a refused command is a `deny` record on the device's own node". Routine run 15 measured it before
//! building on it, and it was not so: a program that commanded its arm once inside the envelope and
//! three times outside it left FOUR `use allow` records in the chain and no `deny` at all — the
//! custody check (the authority: may this node command `arm0/elbow`?) is decided by the broker and
//! recorded, and the envelope (how far?) is decided afterwards by the run's device broker and was
//! journaled only on standard error. An investigator reading the chain saw four allowed uses of an arm
//! that refused three of them; a monitor reading it could never see a refusal.
//!
//! The witnesses here hold the chain to what happened.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// Stops the daemon on drop so a failed assertion never strands a process holding the socket.
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

/// A program that commands its arm once inside the envelope, then three times outside it, saying
/// what it was told each time. The in-envelope command first is the control: it must leave no `deny`.
const PROBER: &str = "\
module m

type Elbow { angle_deg: Float, velocity_dps: Float }

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

fn probe(c: Cap[Console], a: Cap[Actuator], n: Int) -> Int ! {Write, Actuate} {
  if n <= 0 {
    0
  } else {
    c.println(say(a.command(Elbow { angle_deg: 150.0, velocity_dps: 4.0 })))
    probe(c, a, n - 1)
  }
}

fn main(root: Root) ! {Write, Actuate} {
  let c = root.console()
  let a = root.actuator(\"arm0/elbow\")
  c.println(say(a.command(Elbow { angle_deg: 12.0, velocity_dps: 4.0 })))
  let done = probe(c, a, 3)
  c.println(\"PROBER DOWN\")
}
";

/// A heartbeat and TTL far longer than the test runs, so nothing but the envelope refuses anything.
const GRANT: &str = "actuator=arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,\
heartbeat_ms=600000,ttl_ms=600000,fail=safe-park";

struct Fixture {
    cwd: PathBuf,
    state: PathBuf,
}

/// The state directory is kept SHORT: the broker's socket lives in it, and a Unix socket path is
/// limited to 104 bytes on macOS (108 on Linux).
fn setup(tag: &str) -> Fixture {
    let base = std::env::temp_dir().join(format!("dlmon-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("w");
    let state = base.join("s");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    std::fs::write(cwd.join("prober.delulu"), PROBER).unwrap();
    Fixture { cwd, state }
}

fn use_records(f: &Fixture) -> Vec<serde_json::Value> {
    let o = delulu_in(&f.cwd, &f.state, &["audit", "query", "--action", "use", "--json"]);
    assert!(o.status.success(), "audit query: {}\n{}", stdout(&o), stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query --json is one object");
    assert_eq!(v["chain_verified"], true, "the chain must verify: {v}");
    v["records"].as_array().expect("records").clone()
}

/// **The chain records an envelope refusal as a refusal.** Each command the envelope refused is a
/// `deny` record — on the run's node, at the device, naming DL1904 and the reason in the words the
/// program was told — and it cites the `use allow` record it overrides, so the pair reads as what
/// happened: the authority allowed this node to command the arm, and the envelope refused this
/// command. The in-envelope command leaves its `allow` and nothing after it. `extra` runs the same
/// program another way (`--sandbox`: the guest's command reaches the same function through its host).
fn refusals_are_recorded(tag: &str, extra: &[&str]) {
    let f = setup(tag);
    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let mut args = vec![
        "run",
        "prober.delulu",
        "--broker",
        "daemon",
        "--grant",
        "console",
        "--grant",
        GRANT,
        "--broker-profile",
        "sim",
        "--no-prompt",
    ];
    args.extend_from_slice(extra);
    let o = delulu_in(&f.cwd, &f.state, &args);
    let so = stdout(&o);
    let se = stderr(&o);
    assert!(o.status.success(), "the run must succeed — a refused command is a value:\n{so}\n{se}");
    // The baseline: the program was told exactly what the chain must now say.
    assert_eq!(so.matches("COMMANDED").count(), 1, "{so}\n{se}");
    assert_eq!(so.matches("REFUSED: `angle_deg` = 150 is outside the envelope").count(), 3, "{so}\n{se}");
    assert!(!se.contains("did not record a refused command"), "every refusal reached the chain:\n{se}");

    let uses = use_records(&f);
    let allows: Vec<&serde_json::Value> = uses.iter().filter(|r| r["decision"] == "allow").collect();
    let denies: Vec<&serde_json::Value> = uses.iter().filter(|r| r["decision"] == "deny").collect();
    assert_eq!(allows.len(), 4, "the authority allowed all four commands to reach the envelope: {uses:#?}");
    assert_eq!(
        denies.len(),
        3,
        "each of the three commands the envelope refused must be a `deny` record in the chain — \
         without one, the chain says four uses were allowed and a monitor can never see a refusal:\n{uses:#?}"
    );
    let first_allow = allows[0]["seq"].as_u64().unwrap();
    for d in &denies {
        assert_eq!(d["target"], "arm0/elbow", "{d}");
        assert_eq!(d["actor_node"], allows[0]["actor_node"], "the refusal is on the node that used the arm: {d}");
        let a = &d["authority"];
        assert_eq!(a["op"], "Actuate", "{d}");
        assert_eq!(a["refused_by"], "envelope", "{d}");
        assert_eq!(a["code"], "DL1904", "{d}");
        assert!(
            a["reason"].as_str().unwrap_or("").contains("`angle_deg` = 150 is outside the envelope [-30, 95]"),
            "the reason in the words the program was told: {d}"
        );
        // The pair: the `use allow` this refusal overrides, before it in the chain.
        let cited = a["overrides_seq"].as_u64().expect("a refusal cites the use it overrides");
        let seq = d["seq"].as_u64().unwrap();
        assert!(cited < seq, "the cited use precedes its refusal: {d}");
        assert!(
            allows.iter().any(|r| r["seq"].as_u64() == Some(cited)),
            "the cited seq is a `use allow` of the arm: {d}\n{uses:#?}"
        );
    }
    // Three refusals, three different commands: no two cite the same `use`.
    let mut cited: Vec<u64> = denies.iter().filter_map(|d| d["authority"]["overrides_seq"].as_u64()).collect();
    cited.sort_unstable();
    cited.dedup();
    assert_eq!(cited.len(), 3, "each refusal cites its own command's use:\n{uses:#?}");
    // The control: the first, in-envelope command is overridden by nothing.
    assert!(
        denies.iter().all(|d| d["authority"]["overrides_seq"].as_u64() != Some(first_allow)),
        "the in-envelope command was refused by nothing, and no record may say otherwise:\n{uses:#?}"
    );
}

#[test]
fn an_envelope_refusal_is_a_deny_record_citing_the_use_it_overrides() {
    refusals_are_recorded("refusal", &[]);
}

/// The same, for a sandboxed guest: its command crosses the channel to its host, which performs it
/// through the same `device::actuate` (P8-01) and must say the same thing to the chain.
#[test]
fn a_sandboxed_guests_envelope_refusal_is_recorded_the_same_way() {
    refusals_are_recorded("sbx", &["--sandbox"]);
}

// ----- the monitor (P8-04 step 3 (b)) ----------------------------------------------------------------

/// The device envelope a delegation carries (RFC 0001 F1), with a heartbeat and TTL far longer than
/// any test: nothing but the monitor can take the arm.
const DEVICE: &str = "arm0/elbow:angle_deg=-30..95,velocity_dps=0..40,heartbeat_ms=600000,ttl_ms=600000,fail=safe-park";

/// A run that keeps probing: one command inside the envelope, then `n` outside it, burning between
/// them so the wall clock has room for a monitor to act mid-run. It reports every answer.
fn persistent_prober(n: u32) -> String {
    format!(
        "\
module m

type Elbow {{ angle_deg: Float, velocity_dps: Float }}

fn fib(n: Int) -> Int {{
  if n < 2 {{ n }} else {{ fib(n - 1) + fib(n - 2) }}
}}

fn say(r: Result[Unit, ActuateErr]) -> Str {{
  match r {{
    Ok(u) => \"COMMANDED\",
    Err(e) => match e {{
      Envelope(reason) => \"REFUSED: \" + reason,
      LeaseRevoked(reason) => \"REVOKED: \" + reason,
      NoDevice => \"NODEVICE\"
    }}
  }}
}}

fn probe(c: Cap[Console], a: Cap[Actuator], n: Int) -> Int ! {{Write, Actuate}} {{
  if n <= 0 {{
    0
  }} else {{
    c.println(say(a.command(Elbow {{ angle_deg: 150.0, velocity_dps: 4.0 }})))
    c.println(\"burn \" + str(fib(21)))
    probe(c, a, n - 1)
  }}
}}

fn main(root: Root) ! {{Write, Actuate}} {{
  let c = root.console()
  let a = root.actuator(\"arm0/elbow\")
  c.println(say(a.command(Elbow {{ angle_deg: 12.0, velocity_dps: 4.0 }})))
  let done = probe(c, a, {n})
  c.println(\"PROBER DOWN\")
}}
"
    )
}

/// `grants delegate … --json` → (node, token).
fn delegate(f: &Fixture, extra: &[&str]) -> (String, String) {
    let mut args = vec!["grants", "delegate", "--effects", "Actuate,Write", "--device", DEVICE, "--multi", "--json"];
    args.extend_from_slice(extra);
    let o = delulu_in(&f.cwd, &f.state, &args);
    assert!(o.status.success(), "grants delegate {extra:?}: {}\n{}", stdout(&o), stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string())
}

/// A monitor started in the background, its standard error read line by line so the test can wait
/// for "watching" before the run begins — a monitor watches from the chain's head at its start, and a
/// run that began first would be outside what it was asked to judge.
struct Monitor {
    child: std::process::Child,
    lines: std::sync::mpsc::Receiver<String>,
    stderr_seen: Vec<String>,
}

fn start_monitor(f: &Fixture, node: &str, rule: &str) -> Monitor {
    use std::io::BufRead;
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&f.cwd)
        .env("DELULU_STATE_DIR", &f.state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(["monitor", "watch", "--node", node, "--rule", rule, "--poll", "25", "--for", "120000", "--json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn the monitor");
    let err = child.stderr.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut m = Monitor { child, lines: rx, stderr_seen: Vec::new() };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        match m.lines.recv_timeout(left) {
            Ok(l) => {
                let ready = l.starts_with("monitor: watching");
                m.stderr_seen.push(l);
                if ready {
                    return m;
                }
            }
            Err(_) => panic!("the monitor never said it was watching: {:?}", m.stderr_seen),
        }
    }
}

/// End a monitor the way its operator would — revoke the node it watches — and read its one report.
fn stop_monitor(f: &Fixture, node: &str, mut m: Monitor) -> (serde_json::Value, Vec<String>) {
    use std::io::Read;
    let o = delulu_in(&f.cwd, &f.state, &["grants", "revoke", node]);
    assert!(o.status.success(), "grants revoke {node}: {}", stderr(&o));
    let mut out = String::new();
    m.child.stdout.take().unwrap().read_to_string(&mut out).unwrap();
    let status = m.child.wait().unwrap();
    while let Ok(l) = m.lines.recv_timeout(std::time::Duration::from_secs(2)) {
        m.stderr_seen.push(l);
    }
    assert!(status.success(), "the monitor's exit: {status:?}\n{out}\n{:?}", m.stderr_seen);
    let v: serde_json::Value = serde_json::from_str(out.trim())
        .unwrap_or_else(|e| panic!("the monitor's --json is one object ({e}):\n{out}\n{:?}", m.stderr_seen));
    (v, m.stderr_seen)
}

fn all_records(f: &Fixture) -> Vec<serde_json::Value> {
    let o = delulu_in(&f.cwd, &f.state, &["audit", "query", "--json"]);
    assert!(o.status.success(), "audit query: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["chain_verified"], true, "{v}");
    v["records"].as_array().unwrap().clone()
}

/// **A monitor quarantines the run that keeps probing, and the chain says why.** The operator mints a
/// monitor node `g_M`, then the run's lease UNDER it. A program commands its arm out of the envelope
/// again and again; the monitor, holding `denies=3/60000`, revokes the RUN's node — acting as `g_M`,
/// the only standing the broker gives it — after the third refusal. The program is told
/// `LeaseRevoked` and never commands again. The revocation's own record names the rule, the count and
/// the evidence, and each piece of evidence is a refusal of THIS run.
#[test]
fn a_monitor_quarantines_the_run_that_keeps_probing_and_the_chain_says_why() {
    let f = setup("quar");
    std::fs::write(f.cwd.join("persist.delulu"), persistent_prober(60)).unwrap();
    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let (m, _) = delegate(&f, &["--holder-desc", "monitor"]);
    let (run, token) = delegate(&f, &["--parent", &m, "--holder-desc", "the run"]);
    let mon = start_monitor(&f, &m, "denies=3/60000");

    let o = delulu_in(
        &f.cwd,
        &f.state,
        &["run", "persist.delulu", "--lease", &token, "--broker-profile", "sim", "--trace-effects", "--no-prompt"],
    );
    let so = stdout(&o);
    let se = stderr(&o);
    let (report, mon_err) = stop_monitor(&f, &m, mon);
    let all = format!("stdout:\n{so}\nstderr:\n{se}\nmonitor: {report}\n{mon_err:?}");

    // The program: it probed, was refused at least three times, and was ENDED — a quarantine revokes
    // the run's node, so the arm parks (its node is a descendant) and the program's next use of what it
    // held faults, as revoking a run's parent does in `estop_cli.rs`. It never finished.
    assert_eq!(so.matches("COMMANDED").count(), 1, "the in-envelope command, and nothing after: {all}");
    assert!(so.matches("REFUSED:").count() >= 3, "three refusals came first: {all}");
    assert!(!so.contains("PROBER DOWN"), "the quarantine ended the run before it finished its 60 probes: {all}");
    assert_eq!(o.status.code(), Some(1), "{all}");
    let q = report["quarantines"].as_array().expect("quarantines");
    assert_eq!(q.len(), 1, "one run, one quarantine: {all}");
    let by_seq = q[0]["revoked_by_seq"].as_u64().expect("the quarantine's revoke seq");
    assert!(
        se.contains("DL1403") && se.contains(&format!("revoked by audit seq {by_seq}")),
        "the program died of the monitor's revocation, and says which: {all}"
    );
    assert!(
        se.contains("lease.revoked") && se.contains(&format!("(by audit seq {by_seq})")) && se.contains("failstate.engaged"),
        "the arm's lease died of the same revocation and its declared fail-state engaged: {all}"
    );

    // The monitor's report: one quarantine, of the RUN, by the rule, on three refusals.
    let q = &q[0];
    assert_eq!(q["target"], run.as_str(), "the run's node, not the monitor's own: {report}");
    assert_eq!(q["rule"], "denies=3/60000", "{report}");
    assert_eq!(q["count"], 3, "{report}");
    let revoked: Vec<&str> = q["revoked"].as_array().unwrap().iter().filter_map(|v| v.as_str()).collect();
    assert!(revoked.contains(&run.as_str()) && !revoked.contains(&m.as_str()), "the run's subtree only: {report}");
    let after = q["ms_after_last_evidence"].as_i64().unwrap();
    eprintln!("measured: the quarantine landed {after} ms after the third refusal was recorded (poll 25 ms)");
    assert!(after < 10_000, "a sanity bound, not a published figure: {after} ms");
    assert!(report["ended"].as_str().unwrap_or("").contains("nothing left to watch"), "{report}");

    // The chain: the evidence is three envelope refusals of this run, and the revocation says why.
    let recs = all_records(&f);
    for e in q["evidence"].as_array().unwrap() {
        let r = recs.iter().find(|r| r["hash"] == e["hash"]).unwrap_or_else(|| panic!("evidence {e} is in the chain"));
        assert_eq!((r["action"].as_str(), r["decision"].as_str()), (Some("use"), Some("deny")), "{r}");
        assert_eq!(r["authority"]["refused_by"], "envelope", "{r}");
        assert_eq!(r["actor_node"], run.as_str(), "{r}");
    }
    let rev = recs
        .iter()
        .find(|r| r["action"] == "revoke" && r["target"] == run.as_str())
        .unwrap_or_else(|| panic!("the quarantine is a revoke record: {recs:#?}"));
    assert_eq!(rev["actor_node"], m.as_str(), "the monitor acted as its own node: {rev}");
    assert_eq!(rev["decision"], "allow", "{rev}");
    let why = rev["authority"]["why"].as_str().unwrap_or("");
    assert!(
        why.contains("rule `denies=3/60000`") && why.contains("3 deny record(s)"),
        "the revocation's own record says what the monitor saw: {rev}"
    );
}

/// **A monitor sees nothing outside its own subtree.** The same probing run, under `g_M` — and a
/// monitor holding a SIBLING node, with the strictest rule there is. Every refusal of the run is in
/// the chain (the baseline), and the monitor quarantines nothing: the run's nodes are not under it,
/// and the broker would not let it revoke them if they were named (`tree.rs`'s `NotRevocable`).
#[test]
fn a_monitor_sees_nothing_outside_its_own_subtree() {
    let f = setup("sib");
    std::fs::write(f.cwd.join("persist.delulu"), persistent_prober(5)).unwrap();
    let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: f.state.clone() };

    let (m, _) = delegate(&f, &["--holder-desc", "the run's monitor"]);
    let (run, token) = delegate(&f, &["--parent", &m, "--holder-desc", "the run"]);
    let (sibling, _) = delegate(&f, &["--holder-desc", "someone else's monitor"]);
    let mon = start_monitor(&f, &sibling, "envelope");

    let o = delulu_in(&f.cwd, &f.state, &["run", "persist.delulu", "--lease", &token, "--broker-profile", "sim", "--no-prompt"]);
    let so = stdout(&o);
    // Let the monitor read past the run's last record before it is stopped.
    std::thread::sleep(std::time::Duration::from_millis(300));
    let (report, _) = stop_monitor(&f, &sibling, mon);

    assert!(o.status.success(), "{so}\n{}", stderr(&o));
    assert_eq!(so.matches("REFUSED:").count(), 5, "the baseline — five refusals happened:\n{so}");
    assert!(!so.contains("REVOKED:"), "nothing took the arm:\n{so}");
    let refusals = all_records(&f)
        .into_iter()
        .filter(|r| r["actor_node"] == run.as_str() && r["authority"]["refused_by"] == "envelope")
        .count();
    assert_eq!(refusals, 5, "and every one is in the chain, where the monitor reads");
    // Not vacuous: the monitor READ the run's records — its six commands' `use allow`s and five
    // refusals at least — and judged them outside its subtree.
    assert!(
        report["records_read"].as_u64().unwrap_or(0) >= 11,
        "the monitor must have read the run's records before it was stopped, or 'nothing' proves nothing: {report}"
    );
    assert_eq!(report["quarantines"], serde_json::json!([]), "a sibling's monitor quarantines nothing: {report}");
}

// ----- the chain's numbering (AUDIT-SEQ-1, RW 4.44) — what a reader of the chain resumes from ----------

/// **The chain's seq is one numbering — across a daemon restart, and with a sandboxed run's host
/// writing beside the daemon.** Before the fix a daemon started its count at 1 on every start, and a
/// sandboxed run's host numbered its own records "last + 1": one verified chain read
/// `1, 2, 3, 1, 2, 3`, and `audit export --since SEQ` — the documented way to feed a SIEM
/// incrementally — returned some records twice and dropped others.
#[test]
fn the_chains_seq_is_one_numbering_across_a_restart_and_a_sandboxed_run() {
    let f = setup("seq");
    let start = |f: &Fixture| {
        let o = delulu_in(&f.cwd, &f.state, &["broker", "start"]);
        assert!(o.status.success(), "broker start: {}", stderr(&o));
    };
    let stop = |f: &Fixture| {
        let o = delulu_in(&f.cwd, &f.state, &["broker", "stop"]);
        assert!(o.status.success(), "broker stop: {}", stderr(&o));
        // `stop` asks; wait until the socket no longer answers, so the next start is a new daemon.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while delulu_in(&f.cwd, &f.state, &["broker", "status"]).status.success() {
            assert!(std::time::Instant::now() < deadline, "the daemon did not stop");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    };
    start(&f);
    let _guard = DaemonGuard { state: f.state.clone() };
    let (_, _) = delegate(&f, &[]);
    stop(&f);
    start(&f); // a second daemon on the same chain
    let o = delulu_in(
        &f.cwd,
        &f.state,
        &["run", "prober.delulu", "--sandbox", "--broker", "daemon", "--grant", "console", "--grant", GRANT, "--broker-profile", "sim", "--no-prompt"],
    );
    assert!(o.status.success(), "the sandboxed run: {}\n{}", stdout(&o), stderr(&o));
    let (late, _) = delegate(&f, &[]);
    let o = delulu_in(&f.cwd, &f.state, &["grants", "revoke", &late]);
    assert!(o.status.success(), "grants revoke: {}", stderr(&o));

    // The daemon's own references name the record that is in the chain: a revoked node's
    // `revoked_by_seq` (what DL1403 tells a program) is the seq of the revocation's record. A daemon
    // whose count restarted at 1 would have its record renumbered under the lock and say another number.
    let o = delulu_in(&f.cwd, &f.state, &["grants", "list", "--json"]);
    let listed: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("grants list --json");
    let by_seq = listed["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == late.as_str())
        .and_then(|n| n["by_seq"].as_u64())
        .unwrap_or_else(|| panic!("the revoked node names the seq that revoked it: {listed}"));

    let recs = all_records(&f);
    let revoke = recs
        .iter()
        .find(|r| r["action"] == "revoke" && r["target"] == late.as_str())
        .expect("the revocation is in the chain");
    assert_eq!(
        revoke["seq"].as_u64(),
        Some(by_seq),
        "the node says it was revoked by seq {by_seq}; the chain's revocation of it is {revoke}"
    );
    let seqs: Vec<u64> = recs.iter().map(|r| r["seq"].as_u64().unwrap()).collect();
    // The baseline: two daemons and a sandboxed host all wrote here.
    assert!(recs.iter().any(|r| r["action"] == "sandbox-launch"), "the host wrote beside the daemon: {seqs:?}");
    assert_eq!(recs.iter().filter(|r| r["action"] == "root-policy-mode").count(), 2, "two daemons wrote: {seqs:?}");
    assert!(
        seqs.windows(2).all(|w| w[1] > w[0]),
        "seq must be unique and strictly increasing along the chain, whoever wrote: {:?}",
        recs.iter().map(|r| (r["seq"].as_u64().unwrap(), r["action"].as_str().unwrap().to_string())).collect::<Vec<_>>()
    );
    // And so an incremental export from any record's seq returns exactly that record and every one after it.
    let mid = recs.len() / 2;
    let since = seqs[mid].to_string();
    let o = delulu_in(&f.cwd, &f.state, &["audit", "export", "--format", "ocsf", "--since", &since]);
    assert!(o.status.success(), "{}", stderr(&o));
    let exported: Vec<String> = stdout(&o)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["metadata"]["uid"].as_str().unwrap().to_string())
        .collect();
    let expected: Vec<String> = recs[mid..].iter().map(|r| r["hash"].as_str().unwrap().to_string()).collect();
    assert_eq!(exported, expected, "`--since {since}` must be every record from that one on, none twice, none dropped");
}
