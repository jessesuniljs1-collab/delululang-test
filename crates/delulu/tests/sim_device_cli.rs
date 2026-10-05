//! P8-03: the in-tree simulator run as a device PROCESS, and a Verified driver speaking to it.
//!
//! `--broker-profile sim` drives the simulator inside the host; `delulu device sim` runs the same
//! simulator as a separate process on the `CMD`/`READ` line protocol. Put the two together and the
//! whole P8 stack is exercised across a real process boundary — a control program, the host's
//! envelope and lease, a signed Verified driver computing frames, the host's transport writing them,
//! and a device that answers:
//!
//! ```text
//! program → envelope/lease (host) → driver's logic (a re-proved .dpx) → LineTransport → device process
//! ```
//!
//! **The property that makes it worth building:** the same program, the same seed and the same
//! command sequence read the SAME numbers in-process and across the boundary. One simulator, two
//! callers (`delulu_runtime::sim`), asserted here rather than assumed.
//!
//! Nothing here means hardware moved: the device is a simulator, and it says so in its own `--json`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// What the HOST permits (the grant). The bench's own bounds are narrower, below — that is the
/// hard stop.
const GRANTS: [&str; 3] = [
    "actuator=arm0/elbow:angle_deg=-95..95,heartbeat_ms=60000,ttl_ms=600000,fail=safe-park",
    "sensor=arm0/elbow#angle_deg",
    "sensor=arm0/strain",
];

/// What the BENCH can do: ±60 degrees. A command the grant allows and the bench refuses is the
/// device's own `ERR`, reported as the hardware's refusal.
const BENCH: &str = "arm0/elbow:angle_deg=-60..60,heartbeat_ms=60000,ttl_ms=600000,fail=safe-park";

/// Command the joint, read it back through the mirror, then read the synthetic sensor twice — the
/// same program `dead_man_cli.rs` uses for the in-process simulator.
const PROGRAM: &str = "module m\n\n\
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
fn say(r: Result[Unit, ActuateErr]) -> Str {\n\
\x20   match r {\n\
\x20       Ok(u) => \"COMMANDED\",\n\
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
\x20   c.println(say(a.command(Elbow { angle_deg: 42.5 })))\n\
\x20   c.println(show(mirror.read()))\n\
\x20   c.println(show(strain.read()))\n\
\x20   c.println(show(strain.read()))\n\
\x20   c.println(say(a.command(Elbow { angle_deg: 80.0 })))\n\
}\n";

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

struct Rig {
    dir: PathBuf,
    bin: String,
}

impl Rig {
    fn new(tag: &str) -> Rig {
        let dir = std::env::temp_dir().join(format!("delulu_simdev_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(dir.join("m.delulu"), PROGRAM).unwrap();
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/line_driver");
        copy_dir(&examples, &dir.join("line_driver"));
        Rig { dir, bin: env!("CARGO_BIN_EXE_delulu").to_string() }
    }

    fn delulu(&self, args: &[&str]) -> Output {
        Command::new(&self.bin)
            .current_dir(&self.dir)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_HOME", self.dir.join("home"))
            .env("DELULU_STATE_DIR", self.dir.join("state"))
            .args(args)
            .output()
            .expect("run delulu")
    }

    fn keygen(&self) -> (String, String) {
        let o = self.delulu(&["keygen", "--name", "bench", "--json"]);
        assert!(o.status.success(), "keygen: {}", text(&o));
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("keygen --json");
        (v["key"].as_str().unwrap().to_string(), v["public_key"].as_str().unwrap().to_string())
    }

    /// The run's arguments, with the grants and this program.
    fn args(&self, extra: &[&str]) -> Vec<String> {
        let mut a: Vec<String> = vec!["run".into(), "m.delulu".into(), "--grant".into(), "console".into()];
        for g in GRANTS {
            a.push("--grant".into());
            a.push(g.into());
        }
        for e in extra {
            a.push((*e).to_string());
        }
        a.push("--no-prompt".into());
        a
    }

    fn run(&self, extra: &[&str]) -> Output {
        let a = self.args(extra);
        let refs: Vec<&str> = a.iter().map(String::as_str).collect();
        self.delulu(&refs)
    }
}

/// **The slice's property.** The same program, seed and command sequence read the same numbers
/// in-process and through a device process — and the bench's own limit refuses what the grant allowed.
#[test]
fn the_same_program_reads_the_same_numbers_in_process_and_across_a_device_process() {
    let r = Rig::new("parity");
    let (key, public) = r.keygen();
    let o = r.delulu(&["plugin", "build", "line_driver", "-o", "driver.dpx", "--sign", &key]);
    assert!(o.status.success(), "plugin build: {}", text(&o));

    // In-process: the simulator the host drives itself.
    let inproc = r.run(&["--broker-profile", "sim", "--seed", "7", "--signoff", "signoff.json"]);
    assert!(inproc.status.success(), "{}", text(&inproc));
    let inproc_lines: Vec<String> =
        String::from_utf8_lossy(&inproc.stdout).lines().map(|l| l.trim().to_string()).collect();
    assert_eq!(inproc_lines[0], "COMMANDED");
    assert_eq!(inproc_lines[1], "READ 42.5", "the mirror reads the commanded pose: {inproc_lines:?}");
    assert!(inproc_lines[2].starts_with("READ "), "{inproc_lines:?}");

    // Across the boundary: the same simulator as a device process, a signed Verified driver
    // computing the frames, and the bench's own ±60 limit.
    let transport = format!("{} device sim --seed 7 --actuator {BENCH} --sensor arm0/strain", r.bin);
    let across = r.run(&[
        "--broker-profile", "hw:bench", "--approved", "signoff.json",
        "--adapter-dpx", "driver.dpx", "--adapter-signer", &public,
        "--adapter-transport", &transport,
    ]);
    let out = text(&across);
    assert!(across.status.success(), "{out}");
    let lines: Vec<String> =
        String::from_utf8_lossy(&across.stdout).lines().map(|l| l.trim().to_string()).collect();
    assert_eq!(lines[0], "COMMANDED", "{out}");
    assert_eq!(lines[1], "READ 42.5", "the device process mirrors the commanded pose: {out}");
    assert_eq!(
        (&lines[1], &lines[2], &lines[3]),
        (&inproc_lines[1], &inproc_lines[2], &inproc_lines[3]),
        "ONE simulator, two callers: the same seed and sequence read the same numbers\nin-process: {inproc_lines:?}\nacross:    {lines:?}"
    );
    // The bench's own hard stop, which the grant permitted: the device refused it, not the host.
    assert!(
        lines[4].contains("REFUSED") && lines[4].contains("the bench refuses it"),
        "the device's own limit is reported as the hardware's refusal: {out}"
    );
}

/// A command outside the GRANT never reaches the device process at all — the host refuses first,
/// and the device's own log is where that absence is visible.
#[test]
fn a_command_outside_the_grant_never_reaches_the_device_process() {
    let r = Rig::new("envelope");
    let (key, public) = r.keygen();
    assert!(r.delulu(&["plugin", "build", "line_driver", "-o", "driver.dpx", "--sign", &key]).status.success());
    assert!(r.run(&["--broker-profile", "sim", "--seed", "7", "--signoff", "signoff.json"]).status.success());

    // The bench is WIDER than the grant here, so anything the device sees it would accept: a refusal
    // can only have come from the host. The grant stops at 95; the program asks for 200.
    std::fs::write(
        r.dir.join("over.delulu"),
        PROGRAM.replace("angle_deg: 80.0", "angle_deg: 200.0"),
    )
    .unwrap();
    let transport = format!(
        "{} device sim --seed 7 --actuator arm0/elbow:angle_deg=-400..400,heartbeat_ms=60000,ttl_ms=600000,fail=safe-park --sensor arm0/strain",
        r.bin
    );
    let mut a = r.args(&[
        "--broker-profile", "hw:bench", "--approved", "signoff.json",
        "--adapter-dpx", "driver.dpx", "--adapter-signer", &public,
        "--adapter-transport", &transport,
    ]);
    a[1] = "over.delulu".into();
    // The sign-off binds the artifact's bytes, so the changed program needs its own simulation.
    let mut sim = a.clone();
    sim.retain(|x| !["hw:bench", "--broker-profile", "--approved", "signoff.json", "--adapter-dpx", "driver.dpx", "--adapter-signer", &public, "--adapter-transport", &transport].contains(&x.as_str()));
    sim.extend(["--broker-profile".to_string(), "sim".to_string(), "--seed".to_string(), "7".to_string(),
                "--signoff".to_string(), "over-signoff.json".to_string()]);
    let refs: Vec<&str> = sim.iter().map(String::as_str).collect();
    let o = r.delulu(&refs);
    assert!(o.status.success(), "the changed program must simulate: {}", text(&o));
    let i = a.iter().position(|x| x == "signoff.json").expect("the approved record");
    a[i] = "over-signoff.json".into();

    let refs: Vec<&str> = a.iter().map(String::as_str).collect();
    let o = r.delulu(&refs);
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    let lines: Vec<String> = String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).collect();
    assert!(
        lines[4].contains("REFUSED") && lines[4].contains("outside the envelope"),
        "the HOST refuses it, naming the envelope: {out}"
    );
    assert!(
        !lines[4].contains("the bench refuses it"),
        "the bench would have accepted it — so a device refusal here means the host let it through: {out}"
    );
}

/// `delulu device sim` refuses what it cannot do, and describes itself without serving.
#[test]
fn the_device_verb_refuses_a_bench_it_cannot_be_and_describes_the_one_it_can() {
    let r = Rig::new("verb");
    for bad in [
        vec!["device"],
        vec!["device", "simulate"],
        vec!["device", "sim"],
        vec!["device", "sim", "--seed"],
        vec!["device", "sim", "--seed", "x", "--sensor", "s"],
        vec!["device", "sim", "--actuator", "arm0/elbow"],
        vec!["device", "sim", "--sensor", "arm0/strain", "--whatever"],
    ] {
        let o = r.delulu(&bad);
        let out = text(&o);
        assert_eq!(o.status.code(), Some(2), "`{bad:?}` must be refused:\n{out}");
        assert!(out.starts_with("error:"), "`{bad:?}`:\n{out}");
    }
    // `--json` describes the bench and serves nothing: it answers with no input at all.
    let o = r.delulu(&["device", "sim", "--seed", "3", "--actuator", BENCH, "--sensor", "arm0/strain", "--json"]);
    assert!(o.status.success(), "{}", text(&o));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("one envelope");
    assert_eq!(v["seed"], 3);
    assert_eq!(v["simulated"], true, "a simulator says so: {v}");
    assert_eq!(v["actuators"][0]["device"], "arm0/elbow");
    assert_eq!(v["actuators"][0]["dims"][0]["hi"], 60.0, "the bench's own bound: {v}");
    assert_eq!(v["sensors"][0], "arm0/strain");
}
