//! PS-A-03: the sandbox guest, end to end against the real binary.
//!
//! The test plays the HOST: it spawns `delulu __guest`, sends the hello, and serves the channel with
//! a `HostChannel` holding the only root. What it proves is the arrangement itself — the guest runs
//! a program and performs nothing; every effect happens on this side, under the scope this side
//! granted.

use std::io::Write;
use std::process::{Command, Stdio};
use std::rc::Rc;

use delulu_runtime::channel::{write_frame, Hello, HostChannel, CHANNEL_VERSION};
use delulu_runtime::sink::LocalSink;
use delulu_runtime::value::RootVal;

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-guest-{}-{}-{}", name, std::process::id(), tag()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a temp directory");
    d
}

/// Unique per call: the clock alone collides when tests run in parallel (the lesson from CI run
/// 35347357572).
fn tag() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("{t}-{n}")
}

/// Run `program` in a real guest child, serving it from here with `root`. Returns the guest's exit.
fn host_a_guest(program: &str, root: RootVal) -> i32 {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("__guest")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .env("DELULU_NO_FIRST_RUN", "1")
        .spawn()
        .expect("the guest starts");
    let mut to_guest = child.stdin.take().unwrap();
    let mut from_guest = child.stdout.take().unwrap();
    let hello = Hello {
        version: CHANNEL_VERSION.to_string(),
        program: program.to_string(),
        hash: blake3::hash(program.as_bytes()).to_hex().to_string(),
        seed: 1,
        fixed_clock_ms: Some(7),
    };
    write_frame(&mut to_guest, &hello).expect("the hello is sent");
    let mut host = HostChannel::new(LocalSink).with_root(Rc::new(root));
    let exit = host.serve(&mut from_guest, &mut to_guest).expect("the guest talks until it is done");
    let _ = child.wait();
    exit
}

/// The whole arrangement in one run: the guest holds nothing, asks for a writer, and the file
/// appears — written by the host, inside the scope the host granted.
#[test]
fn a_guest_program_runs_and_the_host_performs_its_effects() {
    let dir = tmp("write");
    std::fs::create_dir_all(dir.join("out")).unwrap();
    // The host performs the effect, so the path is the HOST's: the program names the same directory
    // the host was granted. Forward slashes keep the program source identical on every platform.
    let out = dir.join("out").display().to_string().replace('\\', "/");
    let program = format!(
        "module g\n\nfn main(root: Root) ! {{Write}} {{\n    \
         let w = root.fs_write(\"{out}\")\n    \
         let _ = w.write_text(\"made.txt\", \"by the host\")\n}}\n"
    );
    let program = program.as_str();
    let root = RootVal { fs_write: vec![dir.join("out")], ..Default::default() };
    let exit = host_a_guest(program, root);
    assert_eq!(exit, 0, "the guest program ran");
    let made = std::fs::read_to_string(dir.join("out").join("made.txt")).expect("the host wrote the file");
    assert_eq!(made, "by the host");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The guest's own root grants nothing: a program asking for something the HOST was not granted is
/// refused, and the refusal is the host's decision, made with the host's checks.
#[test]
fn a_guest_cannot_reach_what_the_host_was_not_granted() {
    let dir = tmp("refuse");
    let program = "module g\n\nfn main(root: Root) ! {Write} {\n    \
                   let w = root.fs_write(\"./out\")\n    \
                   let _ = w.write_text(\"made.txt\", \"nope\")\n}\n";
    // The host holds no write grant at all.
    let exit = host_a_guest(program, RootVal::default());
    assert_ne!(exit, 0, "a program the host cannot serve must not report success");
    assert!(!dir.join("out").join("made.txt").exists(), "nothing was written");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Started by hand, with no host and no hello, the guest does nothing at all.
#[test]
fn a_guest_started_without_a_hello_refuses_to_run() {
    let out = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("__guest")
        .stdin(Stdio::null())
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("it runs");
    assert_eq!(out.status.code(), Some(2), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("without a hello"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The hash pins what runs: a program swapped for another between the host's decision and the
/// guest's execution is refused rather than executed.
#[test]
fn a_program_that_does_not_match_its_hash_is_refused() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("__guest")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("DELULU_NO_FIRST_RUN", "1")
        .spawn()
        .expect("the guest starts");
    let mut to_guest = child.stdin.take().unwrap();
    let hello = Hello {
        version: CHANNEL_VERSION.to_string(),
        program: "module g\n\nfn main(root: Root) {\n}\n".to_string(),
        hash: blake3::hash(b"a different program").to_hex().to_string(),
        seed: 1,
        fixed_clock_ms: None,
    };
    write_frame(&mut to_guest, &hello).unwrap();
    to_guest.flush().unwrap();
    let out = child.wait_with_output().expect("it exits");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("does not match the hash"));
}
