//! PS-D-01 (L3): `--sandbox-backend external:CMD` — the operator's launcher runs the guest in THEIR
//! environment (Docker + gVisor, Kata, a cloud sandbox, `ssh`) and carries the channel on its standard
//! input and output. These tests use the `delulu` binary itself as the launcher
//! (`external:<delulu> __guest --stdio-pipes`), which is the whole protocol with no container in the way.
//!
//! What must hold, whatever launcher is used:
//! - the guest still holds no authority: every effect is decided and performed by the host, under the
//!   grants — an ungranted one is refused, a granted one happens;
//! - the report claims NOTHING DeluluLang did not measure: level 3, backend `external`, no host guarantee,
//!   `fully_enforced: false`, and the launcher's program named (never its arguments);
//! - a launcher that dies, or a command that names no launcher, fails legibly rather than hanging.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args)
        .output()
        .expect("the binary runs")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn lab(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("dext-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    std::fs::create_dir_all(d.join("out")).unwrap();
    std::fs::create_dir_all(d.join("s")).unwrap();
    d
}

/// The binary as a launcher: its own path, then the guest's words.
fn self_launcher() -> String {
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    assert!(!exe.contains(' '), "the launcher is split on whitespace; this checkout's path has a space: {exe}");
    format!("external:{exe} __guest --stdio-pipes")
}

#[test]
fn an_external_guest_runs_under_the_hosts_grants_and_the_report_claims_nothing_unmeasured() {
    let d = lab("run");
    let out = d.join("out").display().to_string().replace('\\', "/");
    std::fs::write(
        d.join("w.delulu"),
        format!(
            "module w\n\nfn main(root: Root) ! {{Write}} {{\n    let o = root.console()\n    let fw = root.fs_write(\"{out}\")\n    \
             match fw.write_text(\"made.txt\", \"by the host\") {{\n        Ok(_) => o.println(\"wrote\"),\n        Err(_) => o.println(\"refused\")\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let launcher = self_launcher();
    let report = d.join("r.json");
    let args = [
        "run", "w.delulu", "--grant", "console", "--grant", &format!("fs.write={out}"), "--sandbox",
        "--sandbox-backend", &launcher, "--report-out", report.to_str().unwrap(),
    ];
    let r = delulu(&d, &d.join("s"), &args);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("wrote"), "{}", text(&r));
    assert!(text(&r).contains("DeluluLang measured none of it"), "the run says whose boundary it is: {}", text(&r));
    assert_eq!(std::fs::read_to_string(d.join("out").join("made.txt")).unwrap(), "by the host");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    let s = &v["sandbox"];
    assert_eq!(s["backend"], "external", "{s}");
    assert_eq!(s["level"], 3, "{s}");
    assert_eq!(s["requested_level"], 3, "{s}");
    assert_eq!(s["fully_enforced"], false, "an external boundary is never reported as enforced: {s}");
    // `granted` is the profile only when something was MEASURED: on Linux the guest confines itself
    // (Landlock, seccomp) and reports it in the host's known words (RW 4.23); on Windows it applies
    // nothing to itself, and the answer is "none".
    let measured = !s["host_guarantees"].as_array().unwrap().is_empty();
    assert_eq!(s["granted"] == "none", !measured, "{s}");
    assert_eq!(s["host_guarantees"].as_array().unwrap().iter().filter(|g| g.as_str().is_some_and(|g| !g.starts_with("no ") && !g.starts_with("reads only"))).count(), 0,
        "no HOST guarantee is claimed (only what the guest measured of itself, if anything): {s}");
    let launcher_named = s["launcher"].as_str().unwrap();
    assert!(launcher_named.ends_with("delulu") || launcher_named.ends_with("delulu.exe"), "{s}");
    assert!(!launcher_named.contains("__guest"), "the launcher's ARGUMENTS are never recorded: {s}");

    // The host still decides: an effect the run did not grant is refused, and nothing happens.
    std::fs::write(d.join("n.delulu"), "module n\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"never\")\n}\n").unwrap();
    let r = delulu(&d, &d.join("s"), &["run", "n.delulu", "--sandbox", "--sandbox-backend", &launcher]);
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("DL0703"), "an ungranted effect is refused by the host: {}", text(&r));
    assert!(!String::from_utf8_lossy(&r.stdout).contains("never"), "{}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_backend_that_is_not_one_or_a_launcher_that_dies_fails_legibly() {
    let d = lab("refuse");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let st = d.join("s");
    for (args, says) in [
        (vec!["--sandbox-backend", "external:x"], "describes a sandboxed run"),
        (vec!["--sandbox", "--sandbox-backend", "docker"], "is not a backend this command knows"),
        (vec!["--sandbox", "--sandbox-backend", "external:"], "needs the launcher's command"),
        (vec!["--sandbox", "--sandbox-backend", "external:x", "--isolation", "microvm"], "two different boundaries"),
    ] {
        let mut a = vec!["run", "h.delulu", "--grant", "console"];
        a.extend(args.iter().copied());
        let r = delulu(&d, &st, &a);
        assert_eq!(r.status.code(), Some(2), "{args:?}: {}", text(&r));
        assert!(text(&r).contains(says), "{args:?}: {}", text(&r));
        assert!(!text(&r).contains("hi\n"), "{args:?}: nothing ran");
    }
    // A launcher that does not exist, and one that exits without ever running a guest: both end the run
    // with a reason, promptly — never a hang, never a success.
    let exe = env!("CARGO_BIN_EXE_delulu").replace('\\', "/");
    for launcher in ["external:no-such-launcher-anywhere".to_string(), format!("external:{exe} --version")] {
        let t = std::time::Instant::now();
        let r = delulu(&d, &st, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &launcher]);
        assert_ne!(r.status.code(), Some(0), "{launcher}: {}", text(&r));
        assert!(t.elapsed() < std::time::Duration::from_secs(50), "{launcher}: refused promptly, not at the channel deadline");
        assert!(!text(&r).contains("os error"), "{launcher}: in words: {}", text(&r));
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// The launcher is told the guest's words and the limits the run asked for, so a recipe can map them
/// (`docker --memory`); enforcing them is the launcher's.
#[cfg(unix)]
#[test]
fn the_launcher_is_told_the_guest_words_and_the_limits() {
    use std::os::unix::fs::PermissionsExt as _;
    let d = lab("env");
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let script = d.join("launch.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$DELULU_GUEST_ARGS|$DELULU_LIMIT_MEMORY_BYTES|$DELULU_LIMIT_CPU_SECONDS|$DELULU_LIMIT_WALL_SECONDS\" > {}/env.txt\nexec {} $DELULU_GUEST_ARGS\n",
            d.display(),
            env!("CARGO_BIN_EXE_delulu")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let r = delulu(
        &d,
        &d.join("s"),
        &["run", "h.delulu", "--grant", "console", "--sandbox", "--limits", "mem=268435456,cpu=7,wall=30", "--sandbox-backend", &format!("external:{}", script.display())],
    );
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert_eq!(std::fs::read_to_string(d.join("env.txt")).unwrap().trim(), "__guest --stdio-pipes|268435456|7|30");
    let _ = std::fs::remove_dir_all(&d);
}
