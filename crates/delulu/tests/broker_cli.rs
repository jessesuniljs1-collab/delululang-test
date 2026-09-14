//! Stage 5 phase 5f/5g — THE daemon lifecycle test (playbook 5f: "the daemon lifecycle test is the
//! one integration test allowed to spawn a real process"). Everything else about the daemon is
//! unit-tested in-process (src/brokerd.rs tests, on the real pipe/socket).
//!
//! The story, end to end through the REAL binary:
//! start → status → `delulu run --broker daemon` (issue-then-run sugar) performs a granted write →
//! broker-held secret exposes through the daemon (bytes only via `expose`, audited with span) →
//! stop → the SAME run now fails **DL1401 and performs nothing** (fail closed, invariant 27 — the
//! written file proves embedded fallback did NOT happen) → embedded custody still works with the
//! broker down (criterion 11).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// Stops the daemon on drop, so a failed assertion never leaves a stray process holding the pipe.
struct DaemonGuard {
    state: PathBuf,
}
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        // Best-effort, and it must never itself panic (a Drop panic masks the real assertion). Run
        // from a directory guaranteed to exist — the test's work dir may already be gone on the
        // success path — since `broker stop` needs only DELULU_STATE_DIR, not the program cwd.
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

/// A state directory whose socket path the kernel cannot hold is refused BY NAME, before a daemon
/// is spawned — not after a five-second wait.
///
/// `broker_transport` has refused such paths legibly for a long time: the byte count, the platform
/// limit, the remedy. But the check ran only inside the detached daemon, whose output goes to
/// `broker.log`. So the first time it fired on a real Mac — CI, 2026-09-14, a test's state dir under
/// macOS's 49-character temp dir making a 106-byte socket path against the 103 macOS allows — the
/// user saw only "broker daemon did not come up within 5s". A refusal written to be legible,
/// delivered illegibly.
///
/// The path here is too long on EVERY Unix (Linux allows 107), so this runs — and can fail — on
/// Linux, the platform this project can run locally, not only on a Mac.
#[cfg(unix)]
#[test]
fn broker_start_names_an_unholdable_socket_path_instead_of_timing_out() {
    let state = std::env::temp_dir().join(format!("dl_sun_{}_{}", std::process::id(), "x".repeat(100)));
    // Created first, as the federation fixture that failed on macOS created its own: the directory's
    // NAME is fine; only the socket path inside it is too long. Without this the old code failed for
    // a different reason (it could not spawn into a missing directory), which proves nothing here.
    std::fs::create_dir_all(&state).expect("create the state dir");
    let started = std::time::Instant::now();
    let o = delulu_in(&std::env::temp_dir(), &state, &["broker", "start"]);
    let took = started.elapsed();
    let _guard = DaemonGuard { state: state.clone() }; // in case a regression DID start one
    let _ = std::fs::remove_dir_all(&state);
    let err = stderr(&o);
    assert!(!o.status.success(), "a broker whose socket cannot be bound must not report started:\n{err}");
    assert!(err.contains("broker socket path is"), "the refusal must name the problem, not time out:\n{err}");
    assert!(err.contains("--state-dir"), "and name the remedy:\n{err}");
    assert!(
        took < std::time::Duration::from_secs(3),
        "refused before spawning, not after waiting {took:?} for a daemon that could never bind"
    );
}

/// `broker start` into a state directory that does not exist yet must create it and start.
///
/// The daemon is spawned with the state dir as its working directory, and nothing on the default
/// path created that directory first — only `--guard-policy` and `--require-anchored-roots` happened
/// to — so a fresh `DELULU_STATE_DIR` (a new deployment account, a CI job) failed with "cannot spawn
/// the broker daemon" and the OS's word for a missing directory. Found on 2026-09-14 while witnessing
/// the socket-path fix above: the first version of that test forgot to create its directory, and
/// the failure it hit was this one.
#[test]
fn broker_start_creates_a_state_dir_that_does_not_exist_yet() {
    let state = std::env::temp_dir().join(format!("dl_fresh_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&state);
    assert!(!state.exists(), "precondition: the state dir must not exist yet");
    let o = delulu_in(&std::env::temp_dir(), &state, &["broker", "start"]);
    let guard = DaemonGuard { state: state.clone() };
    assert!(
        o.status.success(),
        "broker start must create a missing state dir, not fail to spawn into it:\n{}",
        stderr(&o)
    );
    assert!(state.is_dir(), "and the directory exists afterwards");
    drop(guard);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn daemon_lifecycle_run_expose_stop_fail_closed() {
    // ----- arrange: temp workspace + programs + broker-resident secret ------------------------
    let base = std::env::temp_dir().join(format!("delulu_broker_cli_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cwd = base.join("work");
    let state = base.join("state");
    std::fs::create_dir_all(cwd.join("out")).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let writer = "module m\nfn main(root: Root) ! {Write} { let fs = root.fs_write(\"./out\")\n match fs.write_text(\"a.txt\", \"hello\") { Ok(_) => {}, Err(_) => {} } }\n";
    std::fs::write(cwd.join("writer.delulu"), writer).unwrap();
    let exposer = "module m\nfn main(root: Root) ! {Declassify, Write} { let s = root.secret(\"API_KEY\")\n let d = root.declassify()\n let v = s.expose(d)\n let c = root.console()\n c.println(v) }\n";
    std::fs::write(cwd.join("exposer.delulu"), exposer).unwrap();

    // The secret is set BEFORE the daemon starts (the daemon loads the store at startup — v0.5).
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_KEY", "hunter2"]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));

    // ----- start + status ----------------------------------------------------------------------
    let o = delulu_in(&cwd, &state, &["broker", "status"]);
    assert_eq!(o.status.code(), Some(1), "no daemon yet");

    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let _guard = DaemonGuard { state: state.clone() };

    let o = delulu_in(&cwd, &state, &["broker", "status"]);
    assert!(o.status.success(), "status after start: {}", stderr(&o));
    assert!(stdout(&o).contains("custody: daemon"), "{}", stdout(&o));

    // ----- a daemon-mode run performs a granted write (issue-then-run sugar) --------------------
    let o = delulu_in(&cwd, &state, &["run", "writer.delulu", "--broker", "daemon", "--grant", "fs.write=./out", "--no-prompt"]);
    assert!(o.status.success(), "daemon-mode run failed: {}", stderr(&o));
    assert!(stderr(&o).contains("custody: daemon"), "custody label printed: {}", stderr(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("out/a.txt")).unwrap(), "hello", "the granted write happened");

    // ----- a broker-held secret exposes through the daemon (phase 5g) --------------------------
    let o = delulu_in(
        &cwd,
        &state,
        &["run", "exposer.delulu", "--broker", "daemon", "--grant", "declassify", "--grant", "console", "--grant", "secret:API_KEY=broker", "--no-prompt"],
    );
    assert!(o.status.success(), "expose run failed: {}", stderr(&o));
    assert_eq!(stdout(&o), "hunter2\n", "bytes entered the program only via the broker expose");

    // The daemon audited the expose (with a span) and its whole chain verifies.
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "verify", "--dir", &audit_dir]);
    assert!(o.status.success(), "audit verify: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["audit", "query", "--action", "expose", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
    assert!(v["count"].as_u64().unwrap_or(0) >= 1, "the expose was audited: {v}");
    assert!(
        v["records"].as_array().unwrap().iter().any(|r| r["span"].is_string()),
        "the expose record carries the calling span: {v}"
    );

    // ----- stop, then FAIL CLOSED (invariant 27 / playbook trap 4) ------------------------------
    let o = delulu_in(&cwd, &state, &["broker", "stop"]);
    assert!(o.status.success(), "broker stop: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "status"]);
    assert_eq!(o.status.code(), Some(1), "daemon gone after stop");

    // The SAME daemon-mode run now fails DL1401 — and the write did NOT happen (the sentinel file
    // stays absent), which is the executable proof there was no silent embedded fallback.
    std::fs::remove_file(cwd.join("out/a.txt")).unwrap();
    let o = delulu_in(&cwd, &state, &["run", "writer.delulu", "--broker", "daemon", "--grant", "fs.write=./out", "--no-prompt"]);
    assert_eq!(o.status.code(), Some(1), "daemon-mode run must fail with the broker down");
    assert!(stderr(&o).contains("DL1401"), "fail-closed is DL1401: {}", stderr(&o));
    assert!(stderr(&o).contains("delulu broker start"), "the exact start command is in the message: {}", stderr(&o));
    assert!(!cwd.join("out/a.txt").exists(), "invariant 27: no write, no embedded fallback");

    // ----- criterion 11: embedded custody is untouched by all of this ---------------------------
    let o = delulu_in(&cwd, &state, &["run", "writer.delulu", "--grant", "fs.write=./out", "--no-prompt"]);
    assert!(o.status.success(), "embedded run must still work: {}", stderr(&o));
    assert_eq!(std::fs::read_to_string(cwd.join("out/a.txt")).unwrap(), "hello");

    let _ = std::fs::remove_dir_all(&base);
}
