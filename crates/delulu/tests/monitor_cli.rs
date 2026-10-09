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
