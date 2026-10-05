//! P8-02: a hardware driver whose logic is a signed Verified plugin, end to end.
//!
//! `--adapter-dpx FILE` names the driver's LOGIC — the reference driver is `examples/line_driver`, the
//! line protocol of `delulu_runtime::adapter` written as a Verified plugin — and `--adapter-transport CMD`
//! the process that carries its frames to the device. Here the device is a short script that logs every
//! frame it receives and enforces its own hard stop, as `hw_adapter_cli.rs`'s driver does: its log is
//! what crossed to the machine, seen from the machine's side, and the file it writes when it starts is the
//! witness that a refused driver started nothing at all.
//!
//! Nothing here means hardware moved. No driver for any real device ships in this tree.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const GRANT: &str = "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";

/// Three commands: inside the envelope, inside it but past the device's own hard stop, outside it.
const PROGRAM: &str = "module arm\n\
type Cmd { angle_deg: Float }\n\
fn say(r: Result[Unit, ActuateErr]) -> Str {\n\
\x20 match r {\n\
\x20   Ok(u) => \"COMMANDED\",\n\
\x20   Err(e) => match e {\n\
\x20     Envelope(reason) => \"REFUSED: \" + reason,\n\
\x20     LeaseRevoked(reason) => \"REVOKED\",\n\
\x20     NoDevice => \"NODEVICE\"\n\
\x20   }\n\
\x20 }\n\
}\n\
fn main(root: Root) ! {Write, Actuate} {\n\
\x20 let c = root.console()\n\
\x20 let a = root.actuator(\"arm0/elbow\")\n\
\x20 c.println(say(a.command(Cmd { angle_deg: 12.0 })))\n\
\x20 c.println(say(a.command(Cmd { angle_deg: 70.0 })))\n\
\x20 c.println(say(a.command(Cmd { angle_deg: 999.0 })))\n\
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
    device: String,
}

impl Rig {
    fn new(tag: &str) -> Rig {
        let dir = std::env::temp_dir().join(format!("delulu_dpx_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(dir.join("arm.delulu"), PROGRAM).unwrap();
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/line_driver");
        copy_dir(&examples, &dir.join("line_driver"));
        let device = device(&dir);
        let rig = Rig { dir, device };
        let o = rig.delulu(&["run", "arm.delulu", "--grant", "console", "--grant", GRANT, "--broker-profile", "sim",
            "--signoff", "signoff.json", "--no-prompt"]);
        assert!(o.status.success(), "the simulation signs the program off: {}", text(&o));
        rig
    }

    fn delulu(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(&self.dir)
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_HOME", self.dir.join("home"))
            .env("DELULU_STATE_DIR", self.dir.join("state"))
            .args(args)
            .output()
            .expect("run delulu")
    }

    /// `delulu keygen`; returns (the private key's path, the public key's hex).
    fn keygen(&self, name: &str) -> (String, String) {
        let o = self.delulu(&["keygen", "--name", name, "--json"]);
        assert!(o.status.success(), "keygen: {}", text(&o));
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("keygen --json");
        (v["key"].as_str().unwrap().to_string(), v["public_key"].as_str().unwrap().to_string())
    }

    /// Build the driver package, signed by `key` if one is given.
    fn build(&self, out: &str, key: Option<&str>) {
        let mut a = vec!["plugin", "build", "line_driver", "-o", out];
        if let Some(k) = key {
            a.extend_from_slice(&["--sign", k]);
        }
        let o = self.delulu(&a);
        assert!(o.status.success(), "plugin build: {}", text(&o));
    }

    /// A hardware run with this driver and the device as its transport; `extra` is appended.
    fn hw(&self, dpx: &str, signer: &str, extra: &[&str]) -> Output {
        self.hw_through(&self.device, dpx, signer, extra)
    }

    /// The same with another transport. A transport that does not exist makes the ORDER visible: a driver
    /// refused before its transport is looked up says why; one whose transport came first says "could not
    /// be started" — deterministic where watching a started process die first is a race.
    fn hw_through(&self, transport: &str, dpx: &str, signer: &str, extra: &[&str]) -> Output {
        let mut a = vec!["run", "arm.delulu", "--grant", "console", "--grant", GRANT, "--broker-profile", "hw:arm",
            "--approved", "signoff.json", "--adapter-dpx", dpx, "--adapter-transport", transport,
            "--adapter-signer", signer];
        a.extend_from_slice(extra);
        a.push("--no-prompt");
        self.delulu(&a)
    }

    /// The frames the device received, one per line.
    fn saw(&self) -> Vec<String> {
        std::fs::read_to_string(self.dir.join("device-saw.txt"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn device_started(&self) -> bool {
        self.dir.join("device-started.txt").exists()
    }
}

/// The device: writes `device-started.txt` when it starts, logs each frame, answers `OK` or its own hard
/// stop past 60 degrees, `VAL 21.5` to a read. Python on Windows (as `hw_adapter_cli.rs`, for the same
/// start-up reason), `sh` elsewhere.
fn device(dir: &Path) -> String {
    #[cfg(windows)]
    {
        let script = "import re, sys\n\
             open('device-started.txt', 'w').close()\n\
             log = open('device-saw.txt', 'a')\n\
             while True:\n\
             \x20   line = sys.stdin.readline()\n\
             \x20   if not line:\n\
             \x20       break\n\
             \x20   l = line.rstrip('\\r\\n')\n\
             \x20   log.write(l + '\\n')\n\
             \x20   log.flush()\n\
             \x20   m = re.match(r'^CMD .* angle_deg=([-0-9.]+)', l)\n\
             \x20   if m:\n\
             \x20       reply = 'ERR joint hard stop at 60 deg' if abs(float(m.group(1))) > 60 else 'OK'\n\
             \x20   elif l.startswith('READ'):\n\
             \x20       reply = 'VAL 21.5'\n\
             \x20   else:\n\
             \x20       reply = 'ERR unknown request'\n\
             \x20   print(reply, flush=True)\n";
        std::fs::write(dir.join("device.py"), script).unwrap();
        let warm = Command::new("python")
            .args(["-I", "-S", "device.py"])
            .current_dir(dir)
            .stdin(std::process::Stdio::null())
            .output();
        assert!(warm.as_ref().is_ok_and(|o| o.status.success()), "python could not run the device: {warm:?}");
        let _ = std::fs::remove_file(dir.join("device-started.txt"));
        "python -I -S device.py".to_string()
    }
    #[cfg(not(windows))]
    {
        let script = "#!/bin/sh\n\
             : > device-started.txt\n\
             while IFS= read -r l; do\n\
             \x20 echo \"$l\" >> device-saw.txt\n\
             \x20 case \"$l\" in\n\
             \x20   CMD*angle_deg=*)\n\
             \x20     v=$(echo \"$l\" | sed 's/.*angle_deg=//' | sed 's/,.*//')\n\
             \x20     m=$(echo \"$v\" | tr -d '-')\n\
             \x20     if [ \"$(echo \"$m > 60\" | bc -l 2>/dev/null || echo 0)\" = \"1\" ]; then\n\
             \x20       echo 'ERR joint hard stop at 60 deg'\n\
             \x20     else echo 'OK'; fi ;;\n\
             \x20   READ*) echo 'VAL 21.5' ;;\n\
             \x20   *) echo 'ERR unknown request' ;;\n\
             \x20 esac\n\
             done\n";
        std::fs::write(dir.join("device.sh"), script).unwrap();
        "sh device.sh".to_string()
    }
}

/// **The slice's property.** A signed driver, pinned, drives the device: the frames its re-proved logic
/// computed are the frames the device received; the device's own hard stop comes back as a refusal; and
/// a command outside the envelope never reaches the device at all.
#[test]
fn a_pinned_verified_driver_drives_the_device_and_the_envelope_holds_before_it() {
    let r = Rig::new("drives");
    let (key, public) = r.keygen("driver-signer");
    r.build("driver.dpx", Some(&key));
    let o = r.hw("driver.dpx", &public, &[]);
    let out = text(&o);
    assert!(o.status.success(), "a refusal is a value; the run exits 0:\n{out}");
    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
    let said: Vec<&str> = stdout.lines().collect();
    assert_eq!(said.first().copied(), Some("COMMANDED"), "12 degrees is commanded:\n{out}");
    assert!(said.get(1).is_some_and(|l| l.contains("REFUSED") && l.contains("hard stop")), "the device refuses 70:\n{out}");
    assert!(said.get(2).is_some_and(|l| l.contains("REFUSED") && !l.contains("hard stop")), "the envelope refuses 999:\n{out}");
    assert_eq!(
        r.saw(),
        ["CMD arm0/elbow angle_deg=12.0", "CMD arm0/elbow angle_deg=70.0"],
        "the device received the plugin's frames, and never the command outside the envelope:\n{out}"
    );
    assert!(out.contains("pinned key"), "the run says the driver's signer was the pinned one:\n{out}");
}

/// The same driver under `--sandbox`: the guest holds the actuator handle, and the host runs the driver.
#[test]
fn a_sandboxed_control_program_drives_the_device_through_the_hosts_verified_driver() {
    let r = Rig::new("sandboxed");
    let (key, public) = r.keygen("driver-signer");
    r.build("driver.dpx", Some(&key));
    let o = r.hw("driver.dpx", &public, &["--sandbox"]);
    let out = text(&o);
    assert!(o.status.success(), "{out}");
    assert!(String::from_utf8_lossy(&o.stdout).starts_with("COMMANDED"), "{out}");
    assert_eq!(r.saw(), ["CMD arm0/elbow angle_deg=12.0", "CMD arm0/elbow angle_deg=70.0"], "{out}");
}

/// A driver nobody the operator named vouched for is never interpreted, and its transport never starts.
#[test]
fn a_driver_the_operator_did_not_pin_starts_nothing() {
    let r = Rig::new("unpinned");
    let (key, public) = r.keygen("driver-signer");
    let (stranger, _) = r.keygen("stranger");
    r.build("by-stranger.dpx", Some(&stranger));
    r.build("unsigned.dpx", None);
    r.build("driver.dpx", Some(&key));
    let mut tampered = std::fs::read(r.dir.join("driver.dpx")).unwrap();
    let at = tampered.windows(5).position(|w| w == b"READ ").expect("a frame literal in the DIR");
    tampered[at] = b'X';
    std::fs::write(r.dir.join("tampered.dpx"), &tampered).unwrap();
    for (dpx, codes) in [
        ("by-stranger.dpx", &["DL1510"][..]),
        ("unsigned.dpx", &["DL1511"][..]),
        ("tampered.dpx", &["DL1504", "DL1510"][..]),
    ] {
        let o = r.hw(dpx, &public, &["--adapter-record", "audit"]);
        let out = text(&o);
        assert_eq!(o.status.code(), Some(1), "{dpx}:\n{out}");
        assert!(codes.iter().any(|c| out.contains(c)), "{dpx}: expected one of {codes:?}:\n{out}");
        assert!(!out.contains("COMMANDED"), "{dpx}: nothing ran:\n{out}");
        assert!(!r.device_started(), "{dpx}: the transport was started for a driver that was refused:\n{out}");
        assert!(out.contains("provenance recorded"), "{dpx}: the refusal is recorded (C60):\n{out}");
        // Refused before the transport was looked up: a missing one is never reached.
        let o = r.hw_through("no-such-transport-p8", dpx, &public, &[]);
        let out = text(&o);
        assert!(codes.iter().any(|c| out.contains(c)), "{dpx}: expected one of {codes:?}:\n{out}");
        assert!(!out.contains("could not be started"), "{dpx}: the transport was reached first:\n{out}");
    }
}

/// A plugin that is not a device driver — an export missing, or one that names an effect — is refused
/// before the transport starts.
#[test]
fn a_plugin_that_is_not_a_driver_is_refused_before_the_transport_starts() {
    let r = Rig::new("notdriver");
    let (key, public) = r.keygen("driver-signer");
    // An export missing: `decode_read` taken out of the code and the manifest alike, so it builds.
    let lib = r.dir.join("line_driver/src/lib.delulu");
    let code = std::fs::read_to_string(&lib).unwrap();
    std::fs::write(&lib, &code[..code.find("pub fn decode_read").unwrap()]).unwrap();
    let toml = r.dir.join("line_driver/delulu.toml");
    let manifest = std::fs::read_to_string(&toml).unwrap();
    let kept: Vec<&str> = manifest.lines().filter(|l| !l.starts_with("decode_read")).collect();
    std::fs::write(&toml, kept.join("\n") + "\n").unwrap();
    r.build("missing.dpx", Some(&key));
    let o = r.hw("missing.dpx", &public, &[]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("DL1502") && out.contains("not a device driver") && out.contains("decode_read"), "{out}");
    assert!(!r.device_started(), "the transport was started for a plugin that is not a driver:\n{out}");
    // The order, deterministically: with a transport that does not exist, the interface still answers first.
    let o = r.hw_through("no-such-transport-p8", "missing.dpx", &public, &[]);
    let out = text(&o);
    assert!(out.contains("not a device driver"), "{out}");
    assert!(!out.contains("could not be started"), "the transport was looked up before the interface was checked:\n{out}");
}

/// A Verified driver's flags make sense only together and only on a hardware run; each refusal is exit 2
/// and starts nothing.
#[test]
fn a_verified_drivers_flags_are_refused_where_they_would_be_dropped() {
    let r = Rig::new("flags");
    let (key, public) = r.keygen("driver-signer");
    r.build("driver.dpx", Some(&key));
    let base = ["run", "arm.delulu", "--grant", "console", "--grant", GRANT];
    let hw = ["--broker-profile", "hw:arm", "--approved", "signoff.json"];
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("no signer", [&hw[..], &["--adapter-dpx", "driver.dpx", "--adapter-transport", &r.device]].concat()),
        ("no transport", [&hw[..], &["--adapter-dpx", "driver.dpx", "--adapter-signer", &public]].concat()),
        (
            "beside --adapter-cmd",
            [&hw[..], &["--adapter-dpx", "driver.dpx", "--adapter-transport", &r.device, "--adapter-signer", &public,
                "--adapter-cmd", &r.device]]
                .concat(),
        ),
        ("a transport alone", [&hw[..], &["--adapter-cmd", &r.device, "--adapter-transport", &r.device]].concat()),
        // Without `--adapter-signer`, so the two new flags alone must be what refuses it.
        ("not a hardware run", vec!["--broker-profile", "sim", "--adapter-dpx", "driver.dpx", "--adapter-transport", &r.device]),
    ];
    for (what, extra) in cases {
        let mut a: Vec<&str> = base.to_vec();
        a.extend(extra);
        a.push("--no-prompt");
        let o = r.delulu(&a);
        let out = text(&o);
        assert_eq!(o.status.code(), Some(2), "{what}:\n{out}");
        assert!(!out.contains("COMMANDED"), "{what}:\n{out}");
        assert!(!r.device_started(), "{what}: the device was started:\n{out}");
    }
}
