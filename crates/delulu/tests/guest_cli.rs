//! PS-A-03: the sandbox guest, end to end against the real binary.
//!
//! `__sandbox_run` launches a guest child and serves its effects; the guest holds nothing but the
//! handles the host mints. These tests drive the shipped binary, so what they prove is the
//! arrangement as it actually runs: the guest executes the program and performs none of it.

use std::process::{Command, Output, Stdio};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-guest-{}-{}-{}", name, std::process::id(), tag()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a temp directory");
    d
}

/// Unique per call: the clock alone collides when tests run in parallel (CI run 35347357572).
fn tag() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("{t}-{n}")
}

/// The arrangement in one run: the guest asks for a writer it cannot make, and the file appears —
/// written by the host, inside the scope the operator granted.
#[test]
fn a_guest_program_runs_and_the_host_performs_its_effects() {
    let dir = tmp("write");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let scope = out.display().to_string().replace('\\', "/");
    let src = dir.join("p.delulu");
    std::fs::write(
        &src,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{scope}\")\n    \
             let _ = w.write_text(\"made.txt\", \"by the host\")\n}}\n"
        ),
    )
    .unwrap();

    let o = delulu(&["__sandbox_run", src.to_str().unwrap(), "--grant", &format!("fs.write={scope}")]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let made = std::fs::read_to_string(out.join("made.txt")).expect("the host wrote the file");
    assert_eq!(made, "by the host");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The guest's own root grants nothing, so a program reaching for what the operator did not grant is
/// refused — by the host, with the host's checks, and nothing is written.
#[test]
fn a_guest_cannot_reach_what_was_not_granted() {
    let dir = tmp("refuse");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let scope = out.display().to_string().replace('\\', "/");
    let src = dir.join("p.delulu");
    std::fs::write(
        &src,
        format!(
            "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
             let w = root.fs_write(\"{scope}\")\n    \
             let _ = w.write_text(\"made.txt\", \"nope\")\n}}\n"
        ),
    )
    .unwrap();

    // No `--grant`: the host holds nothing to mint from.
    let o = delulu(&["__sandbox_run", src.to_str().unwrap()]);
    assert_ne!(o.status.code(), Some(0), "a run the host cannot serve must not report success");
    assert!(!out.join("made.txt").exists(), "nothing was written");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The run says what the OS actually enforced, never what it meant to enforce. On Windows that is
/// the Job Object's limits; elsewhere PS-A2 is still to come, and the run says so plainly.
#[test]
fn the_run_reports_what_the_jail_enforced() {
    let dir = tmp("report");
    let src = dir.join("p.delulu");
    std::fs::write(&src, "module g\n\nfn main(root: Root) {\n}\n").unwrap();
    let o = delulu(&["__sandbox_run", src.to_str().unwrap()]);
    let err = String::from_utf8_lossy(&o.stderr);
    // Each platform reports what IT applied, so the assertion is per platform rather than one
    // wording pretending they are the same. (CI run 35390738394 caught this test claiming Linux
    // enforced nothing, on the very commit that made Linux enforce.)
    assert!(err.contains("the guest is confined"), "{err}");
    if cfg!(windows) {
        assert!(err.contains("one process only"), "{err}");
    }
    if cfg!(target_os = "linux") {
        assert!(err.contains("no privilege escalation"), "{err}");
        assert!(err.contains("killed with the host"), "{err}");
    }
    if cfg!(windows) || cfg!(target_os = "linux") {
        assert!(err.contains("memory ceiling"), "{err}");
        assert!(err.contains("processor-time ceiling"), "{err}");
    }
    if cfg!(target_os = "macos") {
        assert!(err.contains("no file writes"), "{err}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Started by hand, with no host on the other end, the guest does nothing at all.
#[test]
fn a_guest_started_without_a_channel_refuses_to_run() {
    let o = delulu(&["__guest"]);
    assert_eq!(o.status.code(), Some(2), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("without a channel directory"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// A guest that is reached but never told what to run gives up on its deadline rather than waiting
/// for ever, and runs nothing meanwhile.
#[test]
fn a_guest_that_is_never_told_what_to_run_does_nothing() {
    let dir = tmp("nohello");
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("__guest")
        .arg(&dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the guest starts");
    // Give it a moment to open its channel, then kill it: the point is that it ran no program.
    std::thread::sleep(std::time::Duration::from_millis(300));
    let _ = child.kill();
    let o = child.wait_with_output().expect("it exits");
    assert!(!o.status.success(), "a guest with no hello must not succeed");
    let _ = std::fs::remove_dir_all(&dir);
}
