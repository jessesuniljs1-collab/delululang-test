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

    #[cfg(windows)]
    {
        let script = "$log = \"adapter-saw.txt\"\n\
             while ($l = [Console]::In.ReadLine()) {\n\
             \x20 Add-Content -Path $log -Value $l\n\
             \x20 if ($l -match '^CMD .* angle_deg=([-0-9.]+)') {\n\
             \x20   $a = [double]$Matches[1]\n\
             \x20   if ([Math]::Abs($a) -gt 60) { Write-Output \"ERR joint hard stop at 60 deg\" }\n\
             \x20   else { Write-Output \"OK\" }\n\
             \x20 } elseif ($l -match '^READ') { Write-Output \"VAL 21.5\" }\n\
             \x20 else { Write-Output \"ERR unknown request\" }\n\
             }\n";
        std::fs::write(dir.join("driver.ps1"), script).unwrap();
        Rig { dir, adapter_cmd: "powershell -NoProfile -File driver.ps1".to_string() }
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
