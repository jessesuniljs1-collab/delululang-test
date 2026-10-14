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

/// `broker start` with a RELATIVE state directory (`DELULU_STATE_DIR=st`, or `--state-dir st`) starts the
/// daemon there — the directory the caller named — and the caller finds it.
///
/// The daemon is spawned with the state directory as its working directory, and it was handed the same
/// relative words: it served at `st/st`, one level down, while the caller waited at `st` and said "did not
/// come up within 5s" — leaving a daemon running that nothing would stop (found by the red-team pass on
/// FRAME-DRIP-1, 2026-09-30; red on `68b8888`). Resolved at the edge now: the daemon is given the absolute
/// path.
#[test]
fn broker_start_with_a_relative_state_dir_serves_where_the_caller_looks() {
    let base = std::env::temp_dir().join(format!("dl_rel_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    // Whatever happens, stop a daemon at either place before the directory goes.
    struct Both(PathBuf);
    impl Drop for Both {
        fn drop(&mut self) {
            for s in [self.0.join("st"), self.0.join("st").join("st")] {
                let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
                    .current_dir(std::env::temp_dir())
                    .env("DELULU_STATE_DIR", &s)
                    .args(["broker", "stop"])
                    .output();
            }
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _both = Both(base.clone());
    let o = delulu_in(&base, Path::new("st"), &["broker", "start"]);
    assert!(o.status.success(), "broker start with a relative state dir: {}", stderr(&o));
    let o = delulu_in(&base, Path::new("st"), &["broker", "status"]);
    assert!(o.status.success(), "the caller finds the daemon where it named it: {}", stderr(&o));
    assert!(!base.join("st").join("st").exists(), "and no second state directory was made inside the first");
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

/// P9-04 (b)/(c), D-V2-131: under the daemon a run's secret is a broker HANDLE — its bytes are the broker's, and the
/// broker does not yet judge an audience (P9-04 (d)). So `get_with_secret` of one is refused in words (DL0904,
/// naming the way out), nothing connects, and the chain holds no `use` of the host — a use for a request that was
/// never made would be a record of nothing (HTTP-SCHEME-1's rule). The way out it names is taken: the same program
/// with embedded custody and the secret granted with its audience sends it.
#[test]
fn a_broker_held_secret_is_not_sent_and_no_use_is_recorded() {
    let base = std::env::temp_dir().join(format!("delulu_bsec_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    let program = format!(
        "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"127.0.0.1\"])\n    \
         let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
         match h.get_with_secret(\"https://127.0.0.1:{port}/probe\", hs, \"Authorization\", tok) {{\n        \
         Ok(b) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
    );
    std::fs::write(cwd.join("send.delulu"), program).unwrap();
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_TOKEN", "hunter2-8f2c"]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let grant = format!("secret:API_TOKEN@127.0.0.1:{port}=broker");
    let args = ["run", "send.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--grant", &grant, "--no-prompt"];
    let mut daemon_args = args.to_vec();
    daemon_args.extend(["--broker", "daemon"]);
    let o = delulu_in(&cwd, &state, &daemon_args);
    assert_eq!(o.status.code(), Some(1), "{}{}", stdout(&o), stderr(&o));
    assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("broker-held secret"), "{}", stderr(&o));
    assert!(stderr(&o).contains("without `--broker daemon`"), "the refusal names its way out: {}", stderr(&o));
    assert!(!stdout(&o).contains("delivered") && !stderr(&o).contains("8f2c"), "{}{}", stdout(&o), stderr(&o));
    // The chain: the run's node was issued (so the chain was read), and no use of the host was recorded.
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "query", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
    let records = v["records"].as_array().expect("records");
    assert!(records.iter().any(|r| r["action"] == "issue"), "the run's issue is in the chain: {v}");
    assert!(!records.iter().any(|r| r["action"] == "use"), "no use for a request never made: {v}");
    drop(guard);
    assert!(l.accept().is_err(), "nothing connected");
    // The way out the refusal names: embedded custody, the secret granted to the run with its audience.
    // Accepted and hung up at once, so the client is told so rather than waiting out its deadline.
    let accepter = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        loop {
            match l.accept() {
                Ok(_) => return true,
                Err(_) if started.elapsed() < std::time::Duration::from_secs(20) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(_) => return false,
            }
        }
    });
    let grant = format!("secret:API_TOKEN@127.0.0.1:{port}=embedded-8f2c");
    let o = delulu_in(&cwd, &state, &["run", "send.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--grant", &grant, "--no-prompt"]);
    assert_eq!(o.status.code(), Some(0), "{}{}", stdout(&o), stderr(&o));
    assert!(accepter.join().unwrap(), "the way out sends the request: {}{}", stdout(&o), stderr(&o));
    let _ = std::fs::remove_dir_all(&base);
}


/// MAP-HANDLE-1 (routine run 24): under the daemon a run's secret is a broker HANDLE, with no bytes in the program's process —
/// and `Secret.map`'s function runs in that process. It was handed the empty string, and `s.map(f).expose(d)` printed `f("")`
/// (here `!`) where embedded custody prints `f(bytes)` (`hunter2!`): a fabricated value, then exposed with no broker gate and no
/// record. Now refused in words (DL0904) before the function runs — and the way out it names is taken: `expose` under
/// `Declassify`, then the transform, prints the real value through the broker.
#[test]
fn a_broker_held_secret_is_never_mapped_over_bytes_it_does_not_hold() {
    let base = std::env::temp_dir().join(format!("delulu_bmap_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let head = "module m\n\nfn main(root: Root) ! {Declassify, Write} {\n    let out = root.console()\n    let s = root.secret(\"API_KEY\")\n    let d = root.declassify()\n";
    std::fs::write(cwd.join("mapped.delulu"), format!("{head}    let m = s.map(fn(x: Str) -> Str {{ x + \"!\" }})\n    out.println(m.expose(d))\n}}\n")).unwrap();
    std::fs::write(cwd.join("exposed.delulu"), format!("{head}    out.println(s.expose(d) + \"!\")\n}}\n")).unwrap();
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_KEY", "hunter2"]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    // The control: embedded custody maps over the real bytes.
    let o = delulu_in(&cwd, &state, &["run", "mapped.delulu", "--grant", "console", "--grant", "declassify", "--grant", "secret:API_KEY=hunter2", "--no-prompt"]);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "hunter2!\n"), "{}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let daemon = |file: &str| {
        delulu_in(&cwd, &state, &["run", file, "--broker", "daemon", "--grant", "console", "--grant", "declassify", "--grant", "secret:API_KEY=broker", "--no-prompt"])
    };
    let o = daemon("mapped.delulu");
    assert_eq!(o.status.code(), Some(1), "refused, never a fabricated value: stdout {:?}\n{}", stdout(&o), stderr(&o));
    assert!(stdout(&o).is_empty(), "nothing printed — `!` was f(\"\"): {:?}", stdout(&o));
    assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("broker-held secret"), "{}", stderr(&o));
    // The way out it names: expose under Declassify through the broker, then transform.
    let o = daemon("exposed.delulu");
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "hunter2!\n"), "{}", stderr(&o));
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}
