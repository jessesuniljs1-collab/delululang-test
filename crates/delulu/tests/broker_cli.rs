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
    assert!(o.status.success(), "broker start must create a missing state dir, not fail to spawn into it:\n{}", stderr(&o));
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
        &[
            "run",
            "exposer.delulu",
            "--broker",
            "daemon",
            "--grant",
            "declassify",
            "--grant",
            "console",
            "--grant",
            "secret:API_KEY=broker",
            "--no-prompt",
        ],
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

/// The bytes of the broker-held secret below, and the part of them no audit record, stdout or stderr may hold. The
/// canary is NOT hex: every record carries 64-hex-digit `hash`/`prev_hash` and a hex `actor_node`, and the hex canary
/// `8f2c` was spelled by a hash by chance (2026-10-17: macOS CI, then a local loop — no byte of the secret anywhere).
const SECRET: &str = "hunter2-wq7z";
const CANARY: &str = "wq7z";

/// P9-04 (d), D-V2-134: under the daemon a run's secret is a broker HANDLE — its bytes are the broker's — and
/// `get_with_secret` of one now asks the broker for them for the ONE origin the request names: the node's `secret_to`
/// (D-V2-133) must hold `NAME@origin`, and the chain records a `send` naming the pair and the span, never a byte. The
/// request then goes out (the listener sees it connect). A request to another origin, or with a header no request may
/// carry, is refused before the broker is asked — no `send` record, nothing connected; and the run's node keeps the
/// audience its grant line named.
#[test]
fn a_broker_held_secret_is_sent_only_to_its_nodes_audience_and_the_send_is_recorded() {
    let base = std::env::temp_dir().join(format!("delulu_bsec_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    let send = |file: &str, to_port: u16, header: &str| {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"127.0.0.1\"])\n    \
             let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
             match h.get_with_secret(\"https://127.0.0.1:{to_port}/probe\", hs, \"{header}\", tok) {{\n        \
             Ok(b) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    };
    send("send.delulu", port, "Authorization");
    send("elsewhere.delulu", port.wrapping_add(1).max(1), "Authorization");
    // A header name no request may carry (a space is not an HTTP token): judged before the bytes exist.
    send("badheader.delulu", port, "Bad Header");
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_TOKEN", SECRET]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let grant = format!("secret:API_TOKEN@127.0.0.1:{port}=broker");
    let daemon = |file: &str| {
        delulu_in(
            &cwd,
            &state,
            &[
                "run",
                file,
                "--grant",
                "console",
                "--grant",
                "net.special=127.0.0.1",
                "--grant",
                &grant,
                "--no-prompt",
                "--broker",
                "daemon",
            ],
        )
    };
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let records = || -> Vec<serde_json::Value> {
        let o = delulu_in(&cwd, &state, &["audit", "query", "--dir", &audit_dir, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
        v["records"].as_array().expect("records").clone()
    };
    let sends = |rs: &[serde_json::Value]| rs.iter().filter(|r| r["action"] == "send").cloned().collect::<Vec<_>>();
    // Another origin: refused before the broker is asked — no `send`, nothing connected.
    let o = daemon("elsewhere.delulu");
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not delivered\n"), "{}", stderr(&o));
    assert!(sends(&records()).is_empty(), "the broker was asked nothing: {:#?}", records());
    let o = daemon("badheader.delulu");
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not delivered\n"), "{}", stderr(&o));
    assert!(sends(&records()).is_empty(), "nor for a header no request may carry: {:#?}", records());
    // P9-05 (a), D-V2-138: a path outside the capability's path scope is refused at the call (DL0904) before the broker is
    // asked — the bytes never leave it, and no `send` is recorded for a request that was never made.
    let offpath = format!(
        "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"127.0.0.1/probe/**\"])\n    \
         let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
         match h.get_with_secret(\"https://127.0.0.1:{port}/elsewhere\", hs, \"Authorization\", tok) {{\n        \
         Ok(b) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
    );
    std::fs::write(cwd.join("offpath.delulu"), offpath).unwrap();
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run",
            "offpath.delulu",
            "--grant",
            "console",
            "--grant",
            "net.special=127.0.0.1/probe/**",
            "--grant",
            &grant,
            "--no-prompt",
            "--broker",
            "daemon",
        ],
    );
    assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("outside every path"), "{}{}", stdout(&o), stderr(&o));
    assert!(sends(&records()).is_empty(), "the broker was asked nothing for a path outside the scope: {:#?}", records());
    assert!(l.accept().is_err(), "nothing connected");
    // The audience: the broker hands the bytes to the egress client, the request goes out, the chain says so.
    let accepter = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        loop {
            match l.accept() {
                Ok(_) => return true,
                Err(_) if started.elapsed() < std::time::Duration::from_secs(20) => {
                    std::thread::sleep(std::time::Duration::from_millis(10))
                }
                Err(_) => return false,
            }
        }
    });
    let o = daemon("send.delulu");
    assert_eq!(o.status.code(), Some(0), "{}{}", stdout(&o), stderr(&o));
    assert!(accepter.join().unwrap(), "the request went out: {}{}", stdout(&o), stderr(&o));
    let rs = records();
    let sent = sends(&rs);
    assert_eq!(sent.len(), 1, "{rs:#?}");
    assert_eq!(
        (sent[0]["decision"].as_str(), sent[0]["target"].as_str()),
        (Some("allow"), Some(format!("API_TOKEN@127.0.0.1:{port}").as_str()))
    );
    // The calling span, in the form an `expose` record carries it (`FILE:START:END`, the interpreter's own).
    assert!(sent[0]["span"].as_str().is_some_and(|s| s.split(':').count() == 3), "the send carries its span: {:#}", sent[0]);
    // No byte of the secret in the chain, in what the program printed, or in what the run said.
    for entry in std::fs::read_dir(state.join("audit")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
        assert!(!text.contains(CANARY), "the chain holds the bytes");
    }
    assert!(!stdout(&o).contains(CANARY) && !stderr(&o).contains(CANARY), "{}{}", stdout(&o), stderr(&o));
    // D-V2-133: the run's node keeps the audience its grant line named.
    let o = delulu_in(&cwd, &state, &["grants", "list", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("grants list json");
    let node = v["nodes"].as_array().expect("nodes").iter().find(|n| n["secrets"] == serde_json::json!(["API_TOKEN"])).cloned();
    let to = node.as_ref().map(|n| n["secret_to"].clone());
    assert_eq!(to, Some(serde_json::json!([format!("API_TOKEN@127.0.0.1:{port}")])), "the node keeps the audience: {v:#}");
    let o = delulu_in(&cwd, &state, &["audit", "verify", "--dir", &audit_dir]);
    assert!(o.status.success(), "audit verify: {}", stderr(&o));
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-05 (d), D-V2-183: `post_with_secret` of a broker-held secret under the daemon — the bytes are asked of the broker for
/// the one origin, as `get_with_secret`'s are, and the request leaves as a POST. The chain holds the custody's `use` of
/// the host naming `method: POST`, then the `send`. A capability minted without POST refuses it at the call, before the
/// broker is asked for a byte: no `send`, nothing connected.
#[test]
fn a_broker_held_secret_rides_a_post_to_its_audience_and_the_use_names_the_method() {
    let base = std::env::temp_dir().join(format!("delulu_bsec_post_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    for (file, mint) in [("post.delulu", "127.0.0.1:POST"), ("getonly.delulu", "127.0.0.1")] {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"{mint}\"])\n    \
             let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
             match h.post_with_secret(\"https://127.0.0.1:{port}/items\", hs, \"Authorization\", tok, \"{{}}\") {{\n        \
             Ok(b) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    }
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_TOKEN", SECRET]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let secret = format!("secret:API_TOKEN@127.0.0.1:{port}=broker");
    let daemon = |file: &str| {
        let net = "net.special=127.0.0.1:GET,POST";
        delulu_in(
            &cwd,
            &state,
            &["run", file, "--grant", "console", "--grant", net, "--grant", &secret, "--no-prompt", "--broker", "daemon"],
        )
    };
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let records = || -> Vec<serde_json::Value> {
        let o = delulu_in(&cwd, &state, &["audit", "query", "--dir", &audit_dir, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
        v["records"].as_array().expect("records").clone()
    };
    let sends = |rs: &[serde_json::Value]| rs.iter().filter(|r| r["action"] == "send").count();
    // Minted for GET alone: the POST is refused at the call, and the broker is asked for nothing.
    let o = daemon("getonly.delulu");
    assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("a `POST` to"), "{}{}", stdout(&o), stderr(&o));
    assert_eq!(sends(&records()), 0, "{:#?}", records());
    assert!(l.accept().is_err(), "nothing connected");
    let accepter = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        loop {
            match l.accept() {
                Ok(_) => return true,
                Err(_) if started.elapsed() < std::time::Duration::from_secs(20) => {
                    std::thread::sleep(std::time::Duration::from_millis(10))
                }
                Err(_) => return false,
            }
        }
    });
    let o = daemon("post.delulu");
    assert_eq!(o.status.code(), Some(0), "{}{}", stdout(&o), stderr(&o));
    assert!(accepter.join().unwrap(), "the POST went out: {}{}", stdout(&o), stderr(&o));
    let rs = records();
    assert_eq!(sends(&rs), 1, "{rs:#?}");
    let used = rs.iter().find(|r| r["action"] == "use" && r["target"] == "127.0.0.1" && r["decision"] == "allow");
    assert_eq!(used.map(|r| r["authority"]["method"].clone()), Some(serde_json::json!("POST")), "{rs:#?}");
    for entry in std::fs::read_dir(state.join("audit")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
        assert!(!text.contains(CANARY), "the chain holds the bytes");
    }
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// D-V2-174: on the WASM engine under the daemon, a run's secret is the broker's handle, as on the interpreter — the
/// program holds an index to it, the host holds the handle — and `get_with_secret` asks the broker for the bytes for the
/// one origin by the interpreter's own code. The same program run on each engine writes the same chain, record for
/// record (the custody's `use` of the host, then the `send`), each request goes out, and no byte of the secret is in the
/// chain or in anything a run said.
#[test]
fn a_wasm_runs_broker_held_secret_is_sent_through_the_broker_as_the_interpreters_is() {
    let base = std::env::temp_dir().join(format!("delulu_bsec_wasm_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    let program = format!(
        "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"127.0.0.1\"])\n    \
         let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
         match h.get_with_secret(\"https://127.0.0.1:{port}/probe\", hs, \"Authorization\", tok) {{\n        \
         Ok(b) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
    );
    std::fs::write(cwd.join("send.delulu"), program).unwrap();
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_TOKEN", SECRET]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    l.set_nonblocking(true).unwrap();
    let l = std::sync::Arc::new(l);
    let accepted = |l: std::sync::Arc<std::net::TcpListener>| {
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            loop {
                match l.accept() {
                    Ok(_) => return true,
                    Err(_) if started.elapsed() < std::time::Duration::from_secs(20) => {
                        std::thread::sleep(std::time::Duration::from_millis(10))
                    }
                    Err(_) => return false,
                }
            }
        })
    };
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let records = || -> Vec<serde_json::Value> {
        let o = delulu_in(&cwd, &state, &["audit", "query", "--dir", &audit_dir, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
        v["records"].as_array().expect("records").clone()
    };
    let grant = format!("secret:API_TOKEN@127.0.0.1:{port}=broker");
    // Each engine's run, and the records it added: its action, decision and — for a use and a send — its target (the
    // others name the run's own node, which differs).
    let mut chains = Vec::new();
    for engine in ["interp", "wasm"] {
        let before = records().len();
        let went = accepted(l.clone());
        let mut args = vec![
            "run",
            "send.delulu",
            "--grant",
            "console",
            "--grant",
            "net.special=127.0.0.1",
            "--grant",
            &grant,
            "--no-prompt",
            "--broker",
            "daemon",
        ];
        if engine == "wasm" {
            args.extend(["--engine", "wasm"]);
        }
        let o = delulu_in(&cwd, &state, &args);
        assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not delivered\n"), "{engine}: {}", stderr(&o));
        assert!(went.join().unwrap(), "{engine}: the request went out: {}", stderr(&o));
        assert!(!stdout(&o).contains(CANARY) && !stderr(&o).contains(CANARY), "{engine}: {}{}", stdout(&o), stderr(&o));
        let added: Vec<String> = records()[before..]
            .iter()
            .map(|r| {
                let target = if r["action"] == "use" || r["action"] == "send" { r["target"].to_string() } else { String::new() };
                format!("{} {} {target}", r["action"], r["decision"])
            })
            .collect();
        let sends = added.iter().filter(|a| a.starts_with("\"send\" \"allow\"")).count();
        assert_eq!(sends, 1, "{engine}: the broker was asked once, and gave: {added:#?}");
        assert!(added.iter().any(|a| a.contains(&format!("API_TOKEN@127.0.0.1:{port}"))), "{engine}: {added:#?}");
        chains.push(added);
    }
    assert_eq!(chains[1], chains[0], "the WASM run's chain is the interpreter's");
    for entry in std::fs::read_dir(state.join("audit")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
        assert!(!text.contains(CANARY), "the chain holds the bytes");
    }
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-04 (d), D-V2-134: a LEASED run's send is the Guard's, as a declassification is — the bytes leave the broker — and
/// its permit names the destination: a delegated node's send is DL1410 until the owner approves `declassify:NAME@origin`
/// (one for another origin does not buy it; one on the NAME would, D-V2-137); a child delegated a narrower audience cannot send to its parent's
/// other origin (refused before the broker is asked). The run then reaches its egress client with the bytes; this host's
/// loopback is a special-use address a lease does not reach (PS-B-02), so the client refuses it — after the `send`.
#[test]
fn a_leased_send_is_guarded_on_its_destination_and_a_narrower_child_cannot_reach_its_parents_other_origin() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    // Short: a broker's socket lives in the state directory (macOS allows 103 bytes).
    let base = std::env::temp_dir().join(format!("dls-{}-{}", std::process::id(), t % 1_000_000_000));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let (a, b) = (44301u16, 44302u16);
    for (file, port) in [("a.delulu", a), ("b.delulu", b)] {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"127.0.0.1\"])\n    \
             let tok = root.secret(\"API_TOKEN\")\n    let hs = Map()\n    \
             match h.get_with_secret(\"https://127.0.0.1:{port}/probe\", hs, \"Authorization\", tok) {{\n        \
             Ok(x) => out.println(\"delivered\")\n        Err(e) => out.println(\"not delivered\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    }
    let s = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(s.status.success(), "broker start: {}", stderr(&s));
    let guard = DaemonGuard { state: state.clone() };
    let text = format!("{}{}", stdout(&s), stderr(&s));
    let owner = {
        let i = text.find("gow1_").expect("the owner code is printed once at start");
        let tail = &text[i..];
        tail[..tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len())].to_string()
    };
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_TOKEN", SECRET]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    let delegate = |extra: &[&str]| -> (String, String) {
        let mut args = vec![
            "grants",
            "delegate",
            "--effects",
            "Write,Net",
            "--net",
            "127.0.0.1",
            "--secret",
            "API_TOKEN",
            "--multi",
            "--json",
        ];
        args.extend_from_slice(extra);
        let o = delulu_in(&cwd, &state, &args);
        assert!(o.status.success(), "delegate {extra:?}: {}{}", stdout(&o), stderr(&o));
        let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
        (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string())
    };
    let (to_a, to_b) = (format!("API_TOKEN@127.0.0.1:{a}"), format!("API_TOKEN@127.0.0.1:{b}"));
    let (node, token) = delegate(&["--secret-to", &to_a, "--secret-to", &to_b]);
    let run = |file: &str, token: &str| delulu_in(&cwd, &state, &["run", file, "--lease", token, "--no-prompt"]);
    let approve = |use_: &str| {
        let q = delulu_in(&cwd, &state, &["guard", "request", &node, "--use", use_, "--why", "send"]);
        assert!(q.status.success(), "request: {}", stderr(&q));
        let req = stdout(&q).trim().to_string();
        let ap = delulu_in(&cwd, &state, &["guard", "approve", &req, "--owner", &owner]);
        assert!(ap.status.success(), "approve: {}", stderr(&ap));
    };
    // Guarded by default; a permit for the secret's other destination does not buy this one (D-V2-137: one on its NAME would).
    let o = run("a.delulu", &token);
    assert!(stderr(&o).contains("DL1410"), "a leased send is guarded: {}{}", stdout(&o), stderr(&o));
    approve(&format!("declassify:{to_b}"));
    let o = run("a.delulu", &token);
    assert!(stderr(&o).contains("DL1410"), "another destination's permit does not buy this send: {}{}", stdout(&o), stderr(&o));
    approve(&format!("declassify:{to_a}"));
    let o = run("a.delulu", &token);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not delivered\n"), "{}", stderr(&o));
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let q = delulu_in(&cwd, &state, &["audit", "query", "--action", "send", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&q)).expect("audit query json");
    let allowed: Vec<&serde_json::Value> = v["records"].as_array().unwrap().iter().filter(|r| r["decision"] == "allow").collect();
    assert_eq!(allowed.len(), 1, "one send, after the permit: {v:#}");
    assert_eq!(allowed[0]["target"].as_str(), Some(to_a.as_str()));
    // A child with the narrower audience cannot send to the parent's other origin — refused before the broker is asked.
    let (_child, child_token) = delegate(&["--parent", &node, "--secret-to", &to_b]);
    let before = v["records"].as_array().unwrap().len();
    let o = run("a.delulu", &child_token);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not delivered\n"), "{}", stderr(&o));
    assert!(stderr(&o).contains("audience"), "said to the operator: {}", stderr(&o));
    let q = delulu_in(&cwd, &state, &["audit", "query", "--action", "send", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&q)).expect("audit query json");
    assert_eq!(v["records"].as_array().unwrap().len(), before, "the broker was asked nothing: {v:#}");
    for entry in std::fs::read_dir(state.join("audit")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap_or_default();
        assert!(!text.contains(CANARY), "the chain holds the bytes");
    }
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// NET-WILDCARD-1 (routine run 25): a pattern `net` grant means the same under the daemon as embedded. A run granted
/// `net.special=*.example.invalid` (`.invalid` never resolves — RFC 6761 — so nothing leaves the machine either way) was
/// allowed its request embedded and refused DL0904 under `--broker daemon`, the broker comparing the host by exact string.
/// Now both reach the egress client (which answers that the name resolves to nothing), and the chain holds `use allow`.
#[test]
fn a_pattern_net_grant_is_honored_under_the_daemon_as_embedded() {
    let base = std::env::temp_dir().join(format!("delulu_bwild_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let program = "module m\n\nfn main(root: Root) ! {Net, Write} {\n    let out = root.console()\n    let h = root.http([\"*.example.invalid\"])\n    \
                   match h.get(\"https://api.example.invalid/\") {\n        Ok(b) => out.println(\"fetched\")\n        \
                   Err(e) => out.println(\"not fetched\")\n    }\n}\n";
    std::fs::write(cwd.join("wild.delulu"), program).unwrap();
    let args = ["run", "wild.delulu", "--grant", "console", "--grant", "net.special=*.example.invalid", "--no-prompt"];
    let o = delulu_in(&cwd, &state, &args);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not fetched\n"), "embedded: {}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let mut daemon = args.to_vec();
    daemon.extend(["--broker", "daemon"]);
    let o = delulu_in(&cwd, &state, &daemon);
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not fetched\n"), "under the daemon: {}", stderr(&o));
    assert!(!stderr(&o).contains("DL0904"), "{}", stderr(&o));
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "query", "--action", "use", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
    let uses: Vec<(String, String)> = v["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["target"] == "api.example.invalid")
        .map(|r| (r["decision"].as_str().unwrap_or("").to_string(), r["target"].as_str().unwrap_or("").to_string()))
        .collect();
    assert!(uses.first().is_some_and(|(d, _)| d == "allow"), "the host's use is allowed and recorded: {v:#}");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-05 (d), D-V2-183: a POST is judged by its method under the daemon as embedded. Granted
/// `net.special=*.example.invalid:POST/v1/**` (`.invalid` never resolves, so nothing leaves the machine), a POST under
/// `/v1` reaches the egress client under both custodies and from a sandboxed guest; a GET there is refused DL0904 at the
/// call, in words about its method. The daemon's chain names the POST's `method` beside its `url_path`, allowed; the
/// GET's use names none and is refused.
#[test]
fn a_post_is_judged_by_its_method_under_the_daemon_as_embedded() {
    let base = std::env::temp_dir().join(format!("delulu_bpost_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let scope = "*.example.invalid:POST/v1/**";
    for (file, call) in [
        ("post.delulu", "post(\"https://api.example.invalid/v1/items\", Map(), \"{}\")"),
        ("get.delulu", "get(\"https://api.example.invalid/v1/items\")"),
    ] {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"{scope}\"])\n    \
             match h.{call} {{\n        Ok(b) => out.println(\"answered\")\n        Err(e) => out.println(\"not answered\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    }
    let grant = format!("net.special={scope}");
    let run = |file: &str, extra: &[&str]| {
        let mut args = vec!["run", file, "--grant", "console", "--grant", grant.as_str(), "--no-prompt"];
        args.extend_from_slice(extra);
        delulu_in(&cwd, &state, &args)
    };
    let check = |extra: &[&str]| {
        let o = run("post.delulu", extra);
        assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not answered\n"), "{extra:?}: {}", stderr(&o));
        assert!(!stderr(&o).contains("DL0904"), "{extra:?}: {}", stderr(&o));
        let o = run("get.delulu", extra);
        assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("outside the methods"), "{extra:?}: {}", stderr(&o));
        assert!(!stdout(&o).contains("answered"), "{extra:?}: the program never got an answer: {}", stdout(&o));
    };
    check(&[]);
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    check(&["--broker", "daemon"]);
    check(&["--broker", "daemon", "--sandbox"]);
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "query", "--action", "use", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
    let uses: Vec<(Option<String>, String)> = v["records"]
        .as_array()
        .unwrap()
        .iter()
        // The broker's own judgement of each use names its path (the egress refusal's record after a refused one does not).
        .filter(|r| r["target"] == "api.example.invalid" && r["authority"].get("url_path").is_some())
        .map(|r| (r["authority"]["method"].as_str().map(str::to_string), r["decision"].as_str().unwrap().to_string()))
        .collect();
    let (post, get) = ((Some("POST".to_string()), "allow".to_string()), (None, "deny".to_string()));
    assert_eq!(uses, [post.clone(), get.clone(), post, get], "the interpreter's two runs, then the guest's: {v:#}");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-05 (a), D-V2-138: a path-scoped `net` grant means the same under the daemon as embedded — NET-WILDCARD-1's question
/// asked of a path. Granted `net.special=*.example.invalid/v1/**` (`.invalid` never resolves, so nothing leaves the
/// machine), a request under `/v1` reaches the egress client under both custodies, and the daemon's chain holds the host's
/// `use allow`; a request for another path, or for a path spelled two ways, is refused DL0904 at the call under both.
/// D-V2-169: under the daemon each use names its path (never its query), judged by the broker — from both engines and a
/// sandboxed guest.
#[test]
fn a_path_scoped_net_grant_is_honored_under_the_daemon_as_embedded() {
    let base = std::env::temp_dir().join(format!("delulu_bpath_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("work"), base.join("state"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(state.join("audit")).unwrap();
    let scope = "*.example.invalid/v1/**";
    for (file, path) in [("in.delulu", "/v1/items?page=/v2"), ("out.delulu", "/v2/items"), ("twisted.delulu", "/v1/../v2/items")]
    {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    let h = root.http([\"{scope}\"])\n    \
             match h.get(\"https://api.example.invalid{path}\") {{\n        Ok(b) => out.println(\"fetched\")\n        \
             Err(e) => out.println(\"not fetched\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    }
    let grant = format!("net.special={scope}");
    let run = |file: &str, daemon: bool| {
        let mut args = vec!["run", file, "--grant", "console", "--grant", grant.as_str(), "--no-prompt"];
        if daemon {
            args.extend(["--broker", "daemon"]);
        }
        delulu_in(&cwd, &state, &args)
    };
    let check = |daemon: bool| {
        let o = run("in.delulu", daemon);
        assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not fetched\n"), "daemon={daemon}: {}", stderr(&o));
        assert!(!stderr(&o).contains("DL0904"), "daemon={daemon}: {}", stderr(&o));
        for (file, words) in
            [("out.delulu", "outside every path the grant names"), ("twisted.delulu", "refused under a path scope")]
        {
            let o = run(file, daemon);
            assert!(stderr(&o).contains("DL0904") && stderr(&o).contains(words), "daemon={daemon}, {file}: {}", stderr(&o));
            assert!(!stdout(&o).contains("fetched"), "daemon={daemon}, {file}: the program never got an answer: {}", stdout(&o));
        }
    };
    check(false);
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    check(true);
    // The WASM host's gate asks its custody with the path as the interpreter's does (`custody_allows_at`), and so does a
    // sandboxed guest's host answering its request (`channel.rs`).
    for (route, flag) in [("wasm", ["--engine", "wasm"].as_slice()), ("sandbox", ["--sandbox"].as_slice())] {
        for (file, refused) in [("in.delulu", false), ("out.delulu", true)] {
            let mut args =
                vec!["run", file, "--grant", "console", "--grant", grant.as_str(), "--no-prompt", "--broker", "daemon"];
            args.extend_from_slice(flag);
            let o = delulu_in(&cwd, &state, &args);
            assert_eq!(stderr(&o).contains("DL0904"), refused, "{route}, {file}: {}", stderr(&o));
            if refused {
                assert!(!stdout(&o).contains("fetched"), "{route}, {file}: the program never got an answer: {}", stdout(&o));
            } else {
                assert_eq!(stdout(&o), "not fetched\n", "{route}, {file}: reached the egress client: {}", stderr(&o));
            }
        }
    }
    let audit_dir = state.join("audit").to_string_lossy().to_string();
    let o = delulu_in(&cwd, &state, &["audit", "query", "--action", "use", "--dir", &audit_dir, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("audit query json");
    let allowed =
        v["records"].as_array().unwrap().iter().any(|r| r["target"] == "api.example.invalid" && r["decision"] == "allow");
    assert!(allowed, "the host's use is allowed and recorded under the daemon: {v:#}");
    // P9-05, D-V2-169: each use names the path it asked of the host — never its query — and the broker judges it, so the
    // chain records `deny` for the two requests the call refused, where it had recorded the host's `allow` for each.
    let uses: Vec<(String, String)> = v["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["target"] == "api.example.invalid" && r["authority"].get("url_path").is_some())
        .map(|r| (r["authority"]["url_path"].as_str().unwrap().to_string(), r["decision"].as_str().unwrap().to_string()))
        .collect();
    // The interpreter's three runs, then the WASM engine's two and the sandboxed guest's two.
    let expected = [
        ("/v1/items", "allow"),
        ("/v2/items", "deny"),
        ("/v1/../v2/items", "deny"),
        ("/v1/items", "allow"),
        ("/v2/items", "deny"),
        ("/v1/items", "allow"),
        ("/v2/items", "deny"),
    ];
    assert_eq!(uses, expected.map(|(p, d)| (p.to_string(), d.to_string())), "{v:#}");
    let o = delulu_in(&cwd, &state, &["audit", "query", "--action", "use", "--dir", &audit_dir]);
    let text = stdout(&o);
    assert!(text.contains("target=api.example.invalid  path=/v2/items"), "the text form names the path too: {text}");
    assert!(!text.contains("page="), "never the query: {text}");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// P9-05 (b), D-V2-139: a delegation may narrow a host to its paths and never widens one. Under a node granted
/// `api.example.invalid`, a child may hold `api.example.invalid/v1/**`; under that child, the whole host is refused
/// DL0802 with the narrower entry as the repair. A `--net` the grammar refuses is refused in words before the broker is
/// asked. A leased run holds the narrow scope: its request for another path is refused at the call (DL0904), one under it
/// reaches the egress client (`.invalid` resolves to nothing — nothing leaves the machine).
#[test]
fn a_delegation_narrows_a_host_to_its_paths_and_never_widens_one() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    // Short: a broker's socket lives in the state directory (macOS allows 103 bytes).
    let base = std::env::temp_dir().join(format!("dlp-{}-{}", std::process::id(), t % 1_000_000_000));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    for (file, path) in [("in.delulu", "/v1/items"), ("out.delulu", "/v2/items")] {
        let program = format!(
            "module m\n\nfn main(root: Root) ! {{Net, Write}} {{\n    let out = root.console()\n    \
             let h = root.http([\"api.example.invalid/v1/**\"])\n    \
             match h.get(\"https://api.example.invalid{path}\") {{\n        Ok(b) => out.println(\"fetched\")\n        \
             Err(e) => out.println(\"not fetched\")\n    }}\n}}\n"
        );
        std::fs::write(cwd.join(file), program).unwrap();
    }
    let s = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(s.status.success(), "broker start: {}", stderr(&s));
    let guard = DaemonGuard { state: state.clone() };
    let delegate = |extra: &[&str]| {
        let mut args = vec!["grants", "delegate", "--effects", "Write,Net", "--multi", "--json"];
        args.extend_from_slice(extra);
        delulu_in(&cwd, &state, &args)
    };
    let node_token = |o: &Output| -> (String, String) {
        assert!(o.status.success(), "delegate: {}{}", stdout(o), stderr(o));
        let v: serde_json::Value = serde_json::from_str(&stdout(o)).expect("delegate --json");
        (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string())
    };
    let (parent, _) = node_token(&delegate(&["--net", "api.example.invalid"]));
    let (child, token) = node_token(&delegate(&["--parent", &parent, "--net", "api.example.invalid/v1/**"]));
    let o = delegate(&["--parent", &child, "--net", "api.example.invalid"]);
    assert!(!o.status.success(), "the whole host is wider than the child's path: {}", stdout(&o));
    let said = format!("{}{}", stdout(&o), stderr(&o));
    assert!(said.contains("DL0802") && said.contains("api.example.invalid/v1/**"), "the repair is the narrower entry: {said}");
    let o = delegate(&["--net", "api.example.invalid/v1/../x"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(stderr(&o).contains("is refused") && stderr(&o).contains(".."), "{}", stderr(&o));
    let run = |file: &str| delulu_in(&cwd, &state, &["run", file, "--lease", &token, "--no-prompt"]);
    let o = run("in.delulu");
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "not fetched\n"), "{}", stderr(&o));
    let o = run("out.delulu");
    assert!(stderr(&o).contains("DL0904") && stderr(&o).contains("outside every path"), "{}{}", stdout(&o), stderr(&o));
    drop(guard);
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
    std::fs::write(
        cwd.join("mapped.delulu"),
        format!("{head}    let m = s.map(fn(x: Str) -> Str {{ x + \"!\" }})\n    out.println(m.expose(d))\n}}\n"),
    )
    .unwrap();
    std::fs::write(cwd.join("exposed.delulu"), format!("{head}    out.println(s.expose(d) + \"!\")\n}}\n")).unwrap();
    let o = delulu_in(&cwd, &state, &["secrets", "set", "API_KEY", "hunter2"]);
    assert!(o.status.success(), "secrets set: {}", stderr(&o));
    // The control: embedded custody maps over the real bytes.
    let o = delulu_in(
        &cwd,
        &state,
        &[
            "run",
            "mapped.delulu",
            "--grant",
            "console",
            "--grant",
            "declassify",
            "--grant",
            "secret:API_KEY=hunter2",
            "--no-prompt",
        ],
    );
    assert_eq!((o.status.code(), stdout(&o).as_str()), (Some(0), "hunter2!\n"), "{}", stderr(&o));
    let o = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(o.status.success(), "broker start: {}", stderr(&o));
    let guard = DaemonGuard { state: state.clone() };
    let daemon = |file: &str| {
        delulu_in(
            &cwd,
            &state,
            &[
                "run",
                file,
                "--broker",
                "daemon",
                "--grant",
                "console",
                "--grant",
                "declassify",
                "--grant",
                "secret:API_KEY=broker",
                "--no-prompt",
            ],
        )
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

/// The daemon's endpoint, opened as a client opens it: the socket in the state directory, or on Windows the pipe named
/// from the canonical state directory (`broker_transport::state_hash`: FNV-1a over its lower-cased spelling).
#[cfg(unix)]
fn raw_connection(state: &Path) -> std::os::unix::net::UnixStream {
    std::os::unix::net::UnixStream::connect(state.join("broker.sock")).expect("connect to the broker's socket")
}
#[cfg(windows)]
fn raw_connection(state: &Path) -> std::fs::File {
    let canon = std::fs::canonicalize(state).unwrap_or_else(|_| state.to_path_buf());
    let mut h: u64 = 0xcbf29ce484222325;
    for b in canon.to_string_lossy().to_lowercase().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    let pipe = format!(r"\\.\pipe\delulu-broker-{h:016x}");
    std::fs::OpenOptions::new().read(true).write(true).open(&pipe).unwrap_or_else(|e| panic!("open {pipe}: {e}"))
}

/// RW 7.17: an answer the daemon gives later than half a device probe's 1 s is named in `broker.log`, with how long it
/// waited for the requests ahead of it and how long it took itself. A run's line says `operator-revoke` for a daemon too
/// slow for its probe as for a revoke (D-V2-99); the log is where the operator, or the next red, finds which it was. The
/// handler is held as RW 4.35's red team held it: a client asks for a large answer — the pending requests, each with a
/// long justification — and reads it slowly, so the daemon's one handler writes that reply until its 5 s bound, and a
/// `broker status` behind it is answered seconds late.
#[test]
fn an_answer_given_late_is_named_in_the_broker_log() {
    use std::io::{Read as _, Write as _};
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    // Short: a broker's socket lives in the state directory (macOS allows 103 bytes).
    let base = std::env::temp_dir().join(format!("dll-{}-{}", std::process::id(), t % 1_000_000_000));
    let _ = std::fs::remove_dir_all(&base);
    let (cwd, state) = (base.join("w"), base.join("s"));
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let s = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(s.status.success(), "broker start: {}", stderr(&s));
    let guard = DaemonGuard { state: state.clone() };
    let text = format!("{}{}", stdout(&s), stderr(&s));
    let owner = {
        let i = text.find("gow1_").expect("the owner code is printed once at start");
        let tail = &text[i..];
        tail[..tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len())].to_string()
    };
    let log = || std::fs::read_to_string(state.join("broker.log")).unwrap_or_default();
    let o = delulu_in(&cwd, &state, &["broker", "status"]);
    assert!(o.status.success(), "broker status: {}", stderr(&o));
    assert!(!log().contains("a Status request was answered"), "a prompt answer is not named:\n{}", log());

    let o = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--effects", "Declassify", "--declassify", "S", "--owner", &owner, "--json"],
    );
    assert!(o.status.success(), "delegate: {}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("delegate --json");
    let node = v["node"].as_str().expect("the node").to_string();
    // Twelve long justifications: an answer several times past what the transport buffers (a Windows pipe 64 KiB, a Linux
    // socket about 208 KiB), each argument inside its platform's command line (32,767 characters on Windows). Each asks
    // for its own pattern: a request the same as one pending is folded into it (`deduped`), not queued.
    let why = "w".repeat(if cfg!(windows) { 30_000 } else { 100_000 });
    for i in 0..12 {
        let o = delulu_in(&cwd, &state, &["guard", "request", &node, "--use", &format!("declassify:p{i}"), "--why", &why]);
        assert!(o.status.success(), "guard request: {}", stderr(&o));
    }

    let mut holder = raw_connection(&state);
    let mut frame = vec![0u8; 4];
    let ask = ciborium::Value::Map(vec![
        (ciborium::Value::Text("version".into()), ciborium::Value::Text("broker/1".into())),
        (ciborium::Value::Text("body".into()), ciborium::Value::Text("GuardPending".into())),
    ]);
    ciborium::into_writer(&ask, &mut frame).unwrap();
    let len = u32::try_from(frame.len() - 4).unwrap();
    frame[..4].copy_from_slice(&len.to_le_bytes());
    holder.write_all(&frame).expect("the holder asks");
    let reading = std::thread::spawn(move || {
        // 32 KiB every quarter second: each of the daemon's writes makes progress within its own bound, and a Windows
        // pipe's 64 KiB drains inside the 1 s its flush is given — slower, and the daemon drops the holder after a second
        // (`the reader did not take its answer`); the whole answer still takes seconds.
        let mut chunk = vec![0u8; 32 * 1024];
        let t = std::time::Instant::now();
        while t.elapsed() < std::time::Duration::from_secs(8) {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if !matches!(holder.read(&mut chunk), Ok(n) if n > 0) {
                break;
            }
        }
    });
    // Let the handler take the holder's request and start writing before the operator asks.
    std::thread::sleep(std::time::Duration::from_millis(500));
    let asked = std::time::Instant::now();
    let o = delulu_in(&cwd, &state, &["broker", "status"]);
    let waited = asked.elapsed().as_millis();
    assert!(o.status.success(), "broker status behind the holder: {}", stderr(&o));
    reading.join().unwrap();

    let log = log();
    let line = log
        .lines()
        .find(|l| l.contains("a Status request was answered"))
        .unwrap_or_else(|| panic!("the late answer is named (the client waited {waited} ms):\n{log}"));
    let ms = |after: &str| -> u128 {
        let rest = &line[line.find(after).unwrap_or_else(|| panic!("`{after}` in {line}")) + after.len()..];
        rest[..rest.find(' ').unwrap_or(rest.len())].parse().unwrap_or_else(|e| panic!("{e}: {line}"))
    };
    let (late, queued) = (ms("was answered "), ms("after it arrived: "));
    assert!(line.contains("ms waiting for the requests ahead of it"), "{line}");
    assert!(late >= 500 && late <= waited, "late by {late} ms, the client waited {waited} ms: {line}");
    assert!(queued >= 500 && queued <= late, "the wait was behind the holder's answer, not the Status itself: {line}");
    drop(guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// RW 7.19: `broker stop` returns once the daemon has exited, so what it held is free — on Windows a running daemon holds
/// its working directory, the state directory, and `broker_cli`'s secret tests left theirs behind on most runs: `stop`
/// returned when the daemon ANSWERED, and the directory was removed while it was still exiting. Five times over, the
/// state directory is removed the moment `stop` returns.
#[test]
fn a_stopped_broker_holds_nothing_its_state_directory_can_be_removed_at_once() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    for i in 0..5 {
        // Short: a broker's socket lives in the state directory (macOS allows 103 bytes).
        let base = std::env::temp_dir().join(format!("dlx-{}-{}-{i}", std::process::id(), t % 1_000_000_000));
        let _ = std::fs::remove_dir_all(&base);
        let (cwd, state) = (base.join("w"), base.join("s"));
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        let s = delulu_in(&cwd, &state, &["broker", "start"]);
        assert!(s.status.success(), "broker start: {}", stderr(&s));
        let guard = DaemonGuard { state: state.clone() };
        let o = delulu_in(&std::env::temp_dir(), &state, &["broker", "stop"]);
        assert!(o.status.success(), "broker stop: {}", stderr(&o));
        assert!(stderr(&o).contains("ok: broker stopped"), "stop waits for the daemon to be gone: {}", stderr(&o));
        let removed = std::fs::remove_dir_all(&base);
        assert!(removed.is_ok() && !base.exists(), "run {i}: the state directory is held after `stop`: {removed:?}");
        std::mem::forget(guard);
    }
}
