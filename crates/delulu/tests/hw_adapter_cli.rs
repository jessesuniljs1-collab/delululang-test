//! The first real hardware adapter, end to end (RFC 0001 dish 3, build-order D23).
//!
//! `Profile::Hw` used to exist "so the artifact-hash gate has something real to gate", with nothing
//! behind it. There is now something behind it: an operator-supplied **subprocess** speaking a line
//! protocol (`delulu_runtime::adapter`). The protocol itself is unit-tested there; this file tests
//! the thing only an end-to-end run can show — **where the envelope is enforced relative to code
//! DeluluLang does not control.**
//!
//! The driver used here is a short PowerShell/sh script that logs every request it receives and
//! enforces its own hard stop. That log is the evidence: it shows exactly which commands crossed
//! the process boundary, which is not observable from inside DeluluLang at all.
//!
//! Nothing here means hardware moved. No driver for any real device ships in this tree, and
//! `STAGE10_AUTONOMY_ADDENDUM.md` §4 still says so.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
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

const GRANT: &str =
    "actuator=arm0/elbow:angle_deg=-30..95,heartbeat_ms=60000,ttl_ms=60000,fail=safe-park";

/// A program that commands one value inside the envelope and one far outside it.
fn arm_program(a: f64, b: f64) -> String {
    format!(
        "module arm\n\
         type Cmd {{ angle_deg: Float }}\n\
         fn say(r: Result[Unit, ActuateErr]) -> Str {{\n\
         \x20 match r {{\n\
         \x20   Ok(u) => \"COMMANDED\",\n\
         \x20   Err(e) => match e {{\n\
         \x20     Envelope(reason) => \"REFUSED: \" + reason,\n\
         \x20     LeaseRevoked(reason) => \"REVOKED\",\n\
         \x20     NoDevice => \"NODEVICE\"\n\
         \x20   }}\n\
         \x20 }}\n\
         }}\n\
         fn main(root: Root) ! {{Write, Actuate}} {{\n\
         \x20 let c = root.console()\n\
         \x20 let a = root.actuator(\"arm0/elbow\")\n\
         \x20 c.println(say(a.command(Cmd {{ angle_deg: {a:?} }})))\n\
         \x20 c.println(say(a.command(Cmd {{ angle_deg: {b:?} }})))\n\
         }}\n"
    )
}

struct Rig {
    dir: PathBuf,
    adapter_cmd: String,
}

/// Write a driver that appends every request to `adapter-saw.txt` and refuses beyond ±60°.
fn rig(tag: &str, program: &str) -> Rig {
    let dir = std::env::temp_dir().join(format!("delulu_hw_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("arm.delulu"), program).unwrap();

    // Windows used to run this driver as a PowerShell script. The first CI run on a Windows runner
    // (2026-09-14) timed it out: the adapter's first exchange has 2000 ms (`EXCHANGE_TIMEOUT`,
    // deliberately not stretchable), and that budget includes the driver's own start-up — which for
    // Windows PowerShell on a loaded two-core VM is seconds. The property under test is WHERE the
    // envelope is enforced, not how fast PowerShell starts, so the driver is now Python, which the
    // default build already requires on Windows. Same log, same ±60° hard stop, same replies.
    #[cfg(windows)]
    {
        let script = "import re, sys\n\
             log = open('adapter-saw.txt', 'a')\n\
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
        std::fs::write(dir.join("driver.py"), script).unwrap();
        // The same timeout again, on a Windows runner, 2026-09-25 (run 36117814329): the job's first
        // two driver starts, launched together, both missed the 2000 ms. Settled by experiment, not
        // by guessing. CPU starvation did NOT reproduce it (0 of 8 runs failed with a high-priority
        // hog pinned to the test's own CPU). A COLD interpreter did: the installed Python's first
        // start took 4251 ms and its next 177 ms, because a plain `python` imports `site` and walks
        // every package installed beside it. An interpreter with no site-packages started cold in
        // 470-564 ms and warm in 51-58 ms. So the driver now runs isolated from whatever is
        // installed (`-I`, and `-S`: no `site`, since it uses only `re` and `sys`), and it is started
        // once, untimed, before any test times it, the way an operator's driver has been run before
        // it drives anything. `EXCHANGE_TIMEOUT` is untouched: what was too slow was this runner's
        // first launch of an interpreter, which is not the property these tests exist for.
        let warm = Command::new("python")
            .args(["-I", "-S", "driver.py"])
            .current_dir(&dir)
            .stdin(std::process::Stdio::null())
            .output();
        assert!(
            warm.as_ref().is_ok_and(|o| o.status.success()),
            "the test driver could not be started by `python` at all: {warm:?}"
        );
        Rig { dir, adapter_cmd: "python -I -S driver.py".to_string() }
    }
    #[cfg(not(windows))]
    {
        let script = "#!/bin/sh\n\
             while IFS= read -r l; do\n\
             \x20 echo \"$l\" >> adapter-saw.txt\n\
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
        let p = dir.join("driver.sh");
        std::fs::write(&p, script).unwrap();
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        Rig { dir, adapter_cmd: "sh driver.sh".to_string() }
    }
}

impl Rig {
    /// Simulate, producing the sign-off record DL1905 will demand.
    fn signoff(&self, name: &str) {
        let o = delulu(
            &self.dir,
            &["run", "arm.delulu", "--grant", "console", "--grant", GRANT,
              "--broker-profile", "sim", "--signoff", name, "--no-prompt"],
        );
        assert!(o.status.success(), "the simulation must run: {}", stderr(&o));
    }
    fn hw(&self, extra: &[&str]) -> Output {
        let mut a = vec!["run", "arm.delulu", "--grant", "console", "--grant", GRANT,
                         "--broker-profile", "hw:demo-adapter"];
        a.extend_from_slice(extra);
        a.push("--no-prompt");
        delulu(&self.dir, &a)
    }
    fn saw(&self) -> String {
        std::fs::read_to_string(self.dir.join("adapter-saw.txt")).unwrap_or_default()
    }
    fn clear_log(&self) {
        let _ = std::fs::remove_file(self.dir.join("adapter-saw.txt"));
    }
}

/// **The property the whole dish exists to establish.**
///
/// A command outside the granted envelope must never reach the driver *at all*. Not "reach it and
/// be rejected" — never arrive. The adapter's own log is the only way to see that, because from
/// inside DeluluLang the two outcomes look identical.
#[test]
fn an_out_of_envelope_command_never_reaches_the_driver_process() {
    let r = rig("order", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");
    r.clear_log();

    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &r.adapter_cmd]);
    assert!(o.status.success(), "a refusal is a value; the run exits 0: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("COMMANDED"), "12 deg is inside the envelope:\n{out}");
    assert!(out.contains("REFUSED"), "999 deg is outside it:\n{out}");

    let saw = r.saw();
    assert!(saw.contains("angle_deg=12"), "the permitted command DID cross the boundary:\n{saw}");
    assert!(
        !saw.contains("999"),
        "THE REFUSED COMMAND MUST NEVER REACH VENDOR CODE. The driver's log shows it did:\n{saw}"
    );
    assert_eq!(saw.lines().filter(|l| !l.trim().is_empty()).count(), 1, "exactly one request:\n{saw}");
}

/// The other direction, and the reason an adapter is worth having: the hardware may refuse
/// something DeluluLang permitted. A driver can refuse MORE and can never permit more.
#[test]
fn the_driver_may_refuse_a_command_the_envelope_allowed() {
    // 90° is inside the granted −30..95 envelope but outside the driver's own 60° hard stop.
    let r = rig("refuse", &arm_program(12.0, 90.0));
    r.signoff("signoff.json");
    r.clear_log();

    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &r.adapter_cmd]);
    assert!(o.status.success(), "a hardware refusal is a VALUE, not a fault: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("COMMANDED"), "{out}");
    assert!(
        out.contains("hard stop") || out.contains("hardware refused"),
        "the hardware's own reason must reach the program:\n{out}"
    );
    let saw = r.saw();
    assert!(saw.contains("angle_deg=90"), "the driver DID see it — it is allowed to refuse:\n{saw}");
}

/// DL1905 is unchanged by any of this: a `hw:` run still needs a human sign-off for these exact
/// bytes, and the driver is not started without one.
#[test]
fn the_dl1905_gate_still_refuses_unapproved_bytes_and_starts_no_driver() {
    let r = rig("gate", &arm_program(12.0, 999.0));
    r.clear_log();

    // No --approved at all.
    let o = r.hw(&["--adapter-cmd", &r.adapter_cmd]);
    assert!(!o.status.success(), "unapproved hardware must not run:\n{}", stdout(&o));
    assert!(stderr(&o).contains("DL1905"), "by code: {}", stderr(&o));
    assert_eq!(r.saw(), "", "and NO driver process was started for unapproved bytes");

    // An approval for different bytes is equally refused.
    r.signoff("signoff.json");
    std::fs::write(r.dir.join("arm.delulu"), arm_program(13.0, 999.0)).unwrap();
    r.clear_log();
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &r.adapter_cmd]);
    assert!(!o.status.success(), "an approval for OTHER bytes approves nothing:\n{}", stdout(&o));
    assert!(stderr(&o).contains("DL1905"), "{}", stderr(&o));
    assert_eq!(r.saw(), "", "and still no driver was started");
}

/// A `hw:` profile with no driver must REFUSE, never quietly command nothing. A program told
/// "COMMANDED" while the machine never moved is the worst failure mode available here.
#[test]
fn a_hardware_profile_with_no_driver_refuses_rather_than_pretending() {
    let r = rig("nodriver", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let o = r.hw(&["--approved", "signoff.json"]);
    assert_eq!(o.status.code(), Some(2), "a hw run with no driver is a usage error");
    let err = stderr(&o);
    assert!(err.contains("--adapter-cmd"), "the refusal names what is missing: {err}");
    assert!(
        err.contains("would be a lie"),
        "…and says why silence is not an option here: {err}"
    );
    assert!(
        !stdout(&o).contains("COMMANDED"),
        "nothing may report success when no driver exists:\n{}",
        stdout(&o)
    );
}

/// A driver that cannot be started fails the run BEFORE `main`, not partway through a motion.
#[test]
fn a_driver_that_will_not_start_fails_the_run_before_main() {
    let r = rig("badexe", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", "definitely-not-a-program-xyzzy"]);
    assert!(!o.status.success(), "an unstartable driver must not run the program");
    assert!(
        stderr(&o).contains("adapter could not be started"),
        "the refusal says what failed: {}",
        stderr(&o)
    );
    assert!(
        !stdout(&o).contains("COMMANDED"),
        "and `main` never ran:\n{}",
        stdout(&o)
    );
}

// ----- adapter provenance (D52, closing the gap D23 named) --------------------------------------

/// **A hardware driver's provenance is checked before it is spawned, and "cannot tell" never means
/// "yes".** D23 shipped the adapter as an operator-supplied subprocess with no signature check at
/// all — named honestly as a gap, but a gap: the envelope bounds what a driver may be *asked* to do
/// and says nothing about where the driver came from.
///
/// Four branches, and the asymmetry between them is the ruling:
///
/// 1. signature present and INVALID  → refused, **regardless of the policy flag**
/// 2. signature absent, no flag      → allowed, disclosed loudly
/// 3. signature absent, flag given   → refused (DL1511)
/// 4. first token is not a file      → not verifiable; disclosed, and refused under the flag
///
/// Branch 1 is the one a "not required, so don't check" reading would skip. Branch 4 is the one a
/// careless implementation gets wrong: `--adapter-cmd "powershell -File drive.ps1"` names the
/// INTERPRETER, so verifying the first token would vouch for the wrong bytes — and passing silently
/// there would be worse than not checking at all, because it would look checked.
#[test]
fn an_adapter_with_a_bad_signature_is_refused_whatever_the_policy_says() {
    let r = rig("sig", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    // A real file to stand in for a driver binary, with a signature that is present and wrong.
    let drv = r.dir.join("driver.bin");
    std::fs::write(&drv, b"not a real driver, but real bytes").unwrap();
    std::fs::write(r.dir.join("driver.bin.sig"), vec![0u8; 96]).unwrap();
    let drv_s = drv.display().to_string();

    // 1. Present-but-invalid refuses even WITHOUT the flag — the branch that matters.
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s]);
    assert!(!o.status.success(), "a signature that does not verify must refuse: {}", stdout(&o));
    let err = stderr(&o);
    assert!(
        err.contains("DL1510") && err.contains("not the bytes that were signed"),
        "the refusal names the fault as provenance, not policy: {err}"
    );

    // 2. No signature, no flag: allowed, but the run SAYS SO.
    std::fs::remove_file(r.dir.join("driver.bin.sig")).unwrap();
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s]);
    let err = stderr(&o);
    assert!(err.contains("UNSIGNED"), "an unsigned driver is disclosed loudly: {err}");

    // 3. No signature WITH the flag: refused.
    let o = r.hw(&["--approved", "signoff.json", "--require-signed-adapter", "--adapter-cmd", &drv_s]);
    assert!(!o.status.success(), "--require-signed-adapter must refuse an unsigned driver");
    assert!(stderr(&o).contains("DL1511"), "{}", stderr(&o));

    // 4. An interpreter-hosted driver is NOT silently treated as verified.
    let o = r.hw(&["--approved", "signoff.json", "--require-signed-adapter", "--adapter-cmd", &r.adapter_cmd]);
    assert!(
        !o.status.success(),
        "a command whose first token is not a file cannot be verified, and under the flag that must \
         refuse rather than pass: {}",
        stdout(&o)
    );
    assert!(
        stderr(&o).contains("nothing to verify"),
        "and it must say WHY it could not check, not merely that it refused: {}",
        stderr(&o)
    );
}

/// D53. **A signature that verifies is not a signature you trust**, and D52 shipped a gate that
/// could not tell the difference.
///
/// The `.sig` lives beside the driver and CARRIES ITS OWN PUBLIC KEY — `verify_detached` reads the
/// key out of the first 32 bytes of the file it is checking. So an attacker who can overwrite
/// `driver.bin` can overwrite `driver.bin.sig` too, signing the replacement with a key they
/// generated a second ago. Against that attacker, `--require-signed-adapter` bought nothing: the
/// signature verifies, the gate says "verifies (signer …)", and the driver spawns.
///
/// This test plays that attack out, and then pins the key.
#[test]
fn a_driver_resigned_by_an_attackers_key_verifies_and_is_still_refused_when_the_key_is_pinned() {
    let r = rig("pin", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let drv = r.dir.join("driver.bin");
    let sig = r.dir.join("driver.bin.sig");
    let drv_s = drv.display().to_string();

    // The operator's key signs the driver they built.
    let operator_seed = [7u8; 32];
    let operator_key = delulu_runtime::plugin::public_key_hex(&operator_seed);
    std::fs::write(&drv, b"the driver the operator built and reviewed").unwrap();
    std::fs::write(&sig, delulu_runtime::plugin::sign_detached(&operator_seed, &std::fs::read(&drv).unwrap())).unwrap();

    // Pinned to the operator's key: accepted, and the run gets as far as the driver.
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s,
                   "--adapter-signer", &operator_key]);
    let err = stderr(&o);
    assert!(
        err.contains("verifies under the pinned key"),
        "the operator's own signed driver must pass its own pin: {err}"
    );

    // Now the attack. Both files are replaced; the new signature is real and self-consistent.
    let attacker_seed = [66u8; 32];
    let attacker_key = delulu_runtime::plugin::public_key_hex(&attacker_seed);
    assert_ne!(attacker_key, operator_key);
    std::fs::write(&drv, b"a driver that does something else entirely").unwrap();
    std::fs::write(&sig, delulu_runtime::plugin::sign_detached(&attacker_seed, &std::fs::read(&drv).unwrap())).unwrap();

    // WITHOUT a pin — the D52 gate, and the whole point of this test. The strongest flag D52
    // offered ACCEPTS the attacker's driver, because "signed" was all it ever asked.
    let o = r.hw(&["--approved", "signoff.json", "--require-signed-adapter", "--adapter-cmd", &drv_s]);
    let err = stderr(&o);
    assert!(
        err.contains("signature verifies"),
        "unpinned, the attacker's own signature satisfies the gate — this is the hole: {err}"
    );
    // And the run must SAY that this is not an assurance, rather than reporting a bare success.
    assert!(
        err.contains("NOT that the key is trusted"),
        "an unpinned verify must not read as a trust decision: {err}"
    );

    // WITH the pin: refused. Same bytes, same valid signature, different answer — because the
    // question changed from "did anyone sign this?" to "did YOU sign this?".
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s,
                   "--adapter-signer", &operator_key]);
    assert!(!o.status.success(), "a pinned run must refuse another signer: {}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("DL1510"), "the wrong-key case is DL1510: {err}");
    assert!(
        err.contains(&attacker_key) && err.contains(&operator_key),
        "the refusal must name BOTH keys, or an operator cannot tell what happened: {err}"
    );
}

// ===== C60 · the provenance decision is RECORDED, not only printed ==========================
//
// D53 made the decision correct and wrote it to stderr. Afterwards there was no durable, queryable
// evidence of which key signed the driver that moved the machine — the one question an incident
// asks. The record goes into the broker's existing hash-chained audit log rather than a second
// format, so `delulu audit verify|tail|query` already reads it.

/// Everything the audit directory holds, as one string. Read from the files rather than through the
/// library so the test is checking what an auditor would actually find on disk.
fn audit_text(dir: &std::path::Path) -> String {
    let mut out = String::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            out.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
        }
    }
    out
}

#[test]
fn a_verified_driver_leaves_a_durable_record_of_which_key_signed_it() {
    let r = rig("record", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let drv = r.dir.join("driver.bin");
    let drv_s = drv.display().to_string();
    let seed = [7u8; 32];
    let key = delulu_runtime::plugin::public_key_hex(&seed);
    std::fs::write(&drv, b"the driver the operator built and reviewed").unwrap();
    std::fs::write(
        r.dir.join("driver.bin.sig"),
        delulu_runtime::plugin::sign_detached(&seed, &std::fs::read(&drv).unwrap()),
    )
    .unwrap();

    let rec_dir = r.dir.join("provenance");
    let rec_s = rec_dir.display().to_string();
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s,
                   "--adapter-signer", &key, "--adapter-record", &rec_s]);
    assert!(
        stderr(&o).contains("verifies under the pinned key"),
        "the operator's own driver must pass its own pin: {}",
        stderr(&o)
    );

    let text = audit_text(&rec_dir);
    assert!(text.contains("adapter.provenance"), "the decision must be recorded: {text}");
    assert!(text.contains("verified-pinned"), "the record must carry the verdict: {text}");
    assert!(text.contains(&key), "and WHICH KEY signed it — the whole point of C60: {text}");
    // It is a chain, not a log: every record links to the one before it.
    assert!(text.contains("prev_hash"), "the record must be hash-chained: {text}");
}

/// **The load-bearing half.** A run stopped because the driver was signed by the wrong key is
/// exactly the event worth keeping, and a recorder that only writes successes keeps the one case
/// nobody needed to look up.
#[test]
fn a_refused_driver_is_recorded_too() {
    let r = rig("recordrefuse", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let drv = r.dir.join("driver.bin");
    let drv_s = drv.display().to_string();
    let attacker = [66u8; 32];
    let operator_key = delulu_runtime::plugin::public_key_hex(&[7u8; 32]);
    let attacker_key = delulu_runtime::plugin::public_key_hex(&attacker);
    std::fs::write(&drv, b"a driver that does something else entirely").unwrap();
    std::fs::write(
        r.dir.join("driver.bin.sig"),
        delulu_runtime::plugin::sign_detached(&attacker, &std::fs::read(&drv).unwrap()),
    )
    .unwrap();

    let rec_dir = r.dir.join("provenance");
    let rec_s = rec_dir.display().to_string();
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s,
                   "--adapter-signer", &operator_key, "--adapter-record", &rec_s]);
    assert!(!o.status.success(), "a pinned run must refuse another signer: {}", stdout(&o));

    let text = audit_text(&rec_dir);
    assert!(text.contains("refused-wrong-signer"), "the refusal must be recorded: {text}");
    assert!(
        text.contains(&attacker_key),
        "and it must name the key that was actually presented, which is the forensic value: {text}"
    );
}

/// The skip branch. A record the operator ASKED for and did not get is worse than none, because
/// they would believe they had it — so an unwritable named sink refuses the run rather than
/// warning past it.
#[test]
fn a_named_record_sink_that_cannot_be_written_refuses_the_run() {
    let r = rig("recordfail", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    let drv = r.dir.join("driver.bin");
    let drv_s = drv.display().to_string();
    std::fs::write(&drv, b"a driver").unwrap();

    // A regular FILE where the directory would have to be: `create_dir_all` cannot succeed here on
    // any platform, which is a portable way to make the sink genuinely unwritable.
    let blocker = r.dir.join("not-a-dir");
    std::fs::write(&blocker, b"x").unwrap();
    let rec_s = blocker.join("inner").display().to_string();

    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &drv_s, "--adapter-record", &rec_s]);
    assert!(!o.status.success(), "an unwritable named sink must stop the run: {}", stdout(&o));
    let err = stderr(&o);
    assert!(err.contains("DL1511"), "refused under the adapter-provenance code: {err}");
    assert!(
        err.contains("worse than none"),
        "and it must say why it refused rather than warning: {err}"
    );
}

/// D53. `--adapter-artifact` exists because the commonest driver shape could not be checked at all.
///
/// `powershell -File driver.ps1` names the INTERPRETER. D52 was right to refuse rather than verify
/// the wrong bytes — but that left `--require-signed-adapter` unusable for every script-hosted
/// driver, which is a control nobody can turn on. Naming the artifact separates "which command runs"
/// from "which bytes were signed".
#[test]
fn the_signed_bytes_can_be_named_apart_from_the_command_that_runs() {
    let r = rig("artifact", &arm_program(12.0, 999.0));
    r.signoff("signoff.json");

    // The real driver is the script; the command names the interpreter.
    let script = if cfg!(windows) { "driver.py" } else { "driver.sh" };
    let seed = [11u8; 32];
    let key = delulu_runtime::plugin::public_key_hex(&seed);
    let bytes = std::fs::read(r.dir.join(script)).unwrap();
    std::fs::write(r.dir.join(format!("{script}.sig")), delulu_runtime::plugin::sign_detached(&seed, &bytes)).unwrap();

    // Without naming it, the run still cannot verify — and still refuses under the flag.
    let o = r.hw(&["--approved", "signoff.json", "--require-signed-adapter",
                   "--adapter-cmd", &r.adapter_cmd]);
    assert!(!o.status.success(), "unchanged from D52: the interpreter is not the driver");
    assert!(stderr(&o).contains("--adapter-artifact"), "and it now names the fix: {}", stderr(&o));

    // Naming it verifies the script's own bytes, under the operator's own key.
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &r.adapter_cmd,
                   "--adapter-artifact", script, "--adapter-signer", &key]);
    let err = stderr(&o);
    assert!(err.contains("verifies under the pinned key"), "the script itself is what got signed: {err}");
    assert!(o.status.success(), "and the run proceeds: {} {}", stdout(&o), err);

    // Tamper with the script the interpreter will actually execute: refused, because the bytes
    // named are the bytes checked.
    std::fs::write(r.dir.join(script), "# replaced\n").unwrap();
    let o = r.hw(&["--approved", "signoff.json", "--adapter-cmd", &r.adapter_cmd,
                   "--adapter-artifact", script, "--adapter-signer", &key]);
    assert!(!o.status.success(), "a tampered driver must not run");
    assert!(stderr(&o).contains("DL1510"), "{}", stderr(&o));
}
