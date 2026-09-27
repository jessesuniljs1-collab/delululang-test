//! C27, as a sweep rather than a list: no command, handed a DIRECTORY where it wanted a file or a path
//! that does not exist, prints the raw OS error.
//!
//! C27 (`HARDENING_CAMPAIGN.md`) found `run <dir>` printing `Access is denied. (os error 5)` on
//! Windows — which reads as a permissions problem and sends the reader hunting for an ACL that was
//! never involved — and `Is a directory (os error 21)` on Linux: two misleading texts for one mistake.
//! It was fixed where it was found, one command at a time. The 2026-09-27 multi-OS test pass found
//! `sign <dir>` still doing it, and 35 sites formatting the OS error themselves. They share one
//! function now (`cli::unreadable`), and this test keeps it that way: the invocations are DERIVED from
//! the binary's own `delulu toolchain --json` usage lines — every positional placeholder filled with
//! the path — so a command added tomorrow is swept without anyone remembering to add it here.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn delulu() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_delulu"));
    c.env("DELULU_NO_FIRST_RUN", "1").env("DELULU_NO_COLOR", "1");
    c
}

fn scratch() -> PathBuf {
    // A counter as well as the clock: macOS's clock resolves to microseconds, so this file's two tests,
    // started together, got ONE directory, and the short one deleted it under the sweep (CI run
    // 36333001114: `spawn: NotFound`).
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-unreadable-{}-{t}-{n}", std::process::id()));
    std::fs::create_dir_all(d.join("state")).unwrap();
    std::fs::create_dir_all(d.join("adir")).unwrap();
    d
}

/// Every invocation a usage line implies, with each positional placeholder replaced by `path`.
/// `atlas node <x> | callers <x>` is two invocations; a line with no placeholder takes no path.
fn invocations(path: &str) -> Vec<Vec<String>> {
    let o = delulu().args(["toolchain", "--json"]).output().expect("the binary runs");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("toolchain --json is JSON");
    let mut out: Vec<Vec<String>> = Vec::new();
    for c in v["toolchain"]["commands"].as_array().expect("a command list") {
        let name = c["name"].as_str().unwrap();
        for line in c["usage"].as_array().unwrap().iter().filter_map(|l| l.as_str()) {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if toks.len() < 2 || toks[0] != "delulu" || toks[1] != name {
                continue;
            }
            // The positional part: up to the first option, bracket or parenthesis.
            let mut words: Vec<String> = Vec::new();
            let mut i = 2;
            while i < toks.len() {
                let t = toks[i];
                if t.starts_with('[') || t.starts_with('-') || t.starts_with('(') {
                    break;
                }
                if t.starts_with('<') {
                    while !toks[i].ends_with('>') && i + 1 < toks.len() {
                        i += 1;
                    }
                    words.push("\u{0}".to_string());
                } else {
                    words.push(t.to_string());
                }
                i += 1;
            }
            // `a <x> | b <x>` — alternatives of one command.
            for alt in words.split(|w| w == "|") {
                if !alt.iter().any(|w| w == "\u{0}") {
                    continue;
                }
                let mut argv = vec![name.to_string()];
                argv.extend(alt.iter().map(|w| if w == "\u{0}" { path.to_string() } else { w.clone() }));
                if !out.contains(&argv) {
                    out.push(argv);
                }
            }
        }
    }
    out
}

/// Run with stdin closed and a deadline, so a command that waits for input cannot hang the suite.
fn run(argv: &[String], cwd: &Path, state: &Path) -> (Option<i32>, String) {
    let mut child = delulu()
        .args(argv)
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let mut so = child.stdout.take().unwrap();
    let mut se = child.stderr.take().unwrap();
    let a = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = so.read_to_string(&mut s);
        s
    });
    let b = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = se.read_to_string(&mut s);
        s
    });
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(st) = child.try_wait().unwrap() {
            break st.code();
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("`delulu {}` did not finish within 60 s with stdin closed", argv.join(" "));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    (status, format!("{}{}", a.join().unwrap(), b.join().unwrap()))
}

#[test]
fn no_command_prints_the_raw_os_error_for_a_directory_or_a_missing_path() {
    let d = scratch();
    let state = d.join("state");
    let mut swept = 0;
    let mut leaks = Vec::new();
    for path in ["adir", "no_such_file.delulu"] {
        let forms = invocations(path);
        assert!(forms.len() >= 25, "the usage lines must yield the file-taking forms: {forms:?}");
        for argv in &forms {
            let (code, text) = run(argv, &d, &state);
            swept += 1;
            assert!(!text.contains("panicked at"), "`delulu {}` panicked:\n{text}", argv.join(" "));
            if text.contains("os error") {
                leaks.push(format!("`delulu {}` (exit {code:?}):\n{}", argv.join(" "), text.trim()));
            }
        }
    }
    assert!(leaks.is_empty(), "{} of {swept} invocations printed a raw OS error:\n\n{}", leaks.len(), leaks.join("\n\n"));
    let _ = std::fs::remove_dir_all(&d);
}

/// The words themselves, for the three cases that have them — and the rule's own witness: `sign` on a
/// directory, the invocation the test pass caught, says what the mistake is.
#[test]
fn a_directory_a_missing_file_and_their_words() {
    let d = scratch();
    let state = d.join("state");
    let (code, text) = run(&["sign".to_string(), "adir".to_string()], &d, &state);
    assert_eq!(code, Some(2), "{text}");
    assert!(text.contains("cannot read `adir`: it is a directory, and a file was expected here"), "{text}");
    let (code, text) = run(&["sign".to_string(), "nothere.dwx".to_string()], &d, &state);
    assert_eq!(code, Some(2), "{text}");
    assert!(text.contains("cannot read `nothere.dwx`: no such file"), "{text}");
    let _ = std::fs::remove_dir_all(&d);
}
