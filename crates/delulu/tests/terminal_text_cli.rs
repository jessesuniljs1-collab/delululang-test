//! TERMINAL-TEXT-1: a program's own strings must never reach the operator's terminal as control
//! sequences.
//!
//! A program's string values end up in what `delulu` prints to a person: an assertion's values in its
//! fault, a path in a refusal and in the `--grant` it suggests, a test's name, a quoted source line. They
//! were printed raw, so a PURE program — no effect, no grant — could write an escape sequence to the
//! operator's terminal: set its title, erase a line and forge one of the host's (`sandbox: …`), or, in
//! terminals that honour OSC 52, set the clipboard. Witnessed on `3489b57` on every path below. Now every
//! control character but a line break or a tab is shown escaped (`\u{1b}`), wherever such text is
//! printed for a person; in a line that must stay one line, a line break is escaped too. JSON output is
//! unchanged — its own escaping is exact.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", cwd.join("s"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args)
        .output()
        .expect("the binary runs")
}

fn lab(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("dterm-{tag}-{}-{}-{n}", std::process::id(), t % 1_000_000_000));
    std::fs::create_dir_all(d.join("s")).unwrap();
    d
}

/// Everything the command printed, both streams, as bytes.
fn printed(o: &Output) -> Vec<u8> {
    [o.stdout.as_slice(), o.stderr.as_slice()].concat()
}

/// No byte of a C0 control (but a line break or a tab) and no C1 control reached the terminal.
fn assert_no_control(o: &Output, what: &str) {
    let all = printed(o);
    let text = String::from_utf8_lossy(&all);
    let bad: Vec<char> = text.chars().filter(|&c| c.is_control() && c != '\n' && c != '\t').collect();
    assert!(bad.is_empty(), "{what}: control characters reached the terminal: {bad:?}\n{}", text.escape_debug());
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&printed(o)).into_owned()
}

/// The escape sequences a hostile string carries: a window title, a line erase and a carriage return
/// that would put a forged host line where the real one was.
const HOSTILE: &str = r"\u{1b}]0;PWNED\u{7}\u{1b}[2K\rsandbox: forged";

#[test]
fn an_assertions_values_reach_the_terminal_escaped() {
    let d = lab("assert");
    std::fs::write(d.join("t.delulu"), format!("module t\n\nfn main(root: Root) {{\n    assert_eq(\"{HOSTILE}\", \"x\")\n}}\n")).unwrap();
    let r = delulu(&d, &["run", "t.delulu"]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(text(&r).contains("DL1707"), "the assertion still fails, and says so: {}", text(&r));
    assert_no_control(&r, "a pure program's failed assertion");
    assert!(text(&r).contains(r"\u{1b}]0;PWNED"), "the value is shown, escaped: {}", text(&r));
    assert!(!text(&r).lines().any(|l| l.starts_with("sandbox: forged")), "no line is forged: {}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}

/// The same program as a sandboxed guest: the guest prints its own fault, on its own standard error.
#[test]
fn a_sandboxed_guests_fault_reaches_the_terminal_escaped() {
    let d = lab("guest");
    std::fs::write(d.join("t.delulu"), format!("module t\n\nfn main(root: Root) {{\n    assert_eq(\"{HOSTILE}\", \"x\")\n}}\n")).unwrap();
    let r = delulu(&d, &["run", "t.delulu", "--sandbox"]);
    assert_ne!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("DL1707"), "{}", text(&r));
    assert_no_control(&r, "a sandboxed guest's failed assertion");
    let _ = std::fs::remove_dir_all(&d);
}

/// The same bytes as a path, without the `:` — Windows refuses a `:` in a component (an alternate data
/// stream, DL0904) before the grant is asked, and this witness is about the grant's refusal on every OS
/// (read red on a Windows runner, `witness.yml` `36599638102`, before it was taken out).
const HOSTILE_PATH: &str = r"\u{1b}]0;PWNED\u{7}\u{1b}[2K\rforged";

/// A refusal quotes the path the program asked for, twice — in the message and in the `--grant` it
/// suggests. Both escaped; and the JSON envelope still carries the exact bytes.
#[test]
fn a_refusal_quotes_the_programs_path_escaped_and_json_keeps_it_exact() {
    let d = lab("refusal");
    std::fs::write(
        d.join("w.delulu"),
        format!(
            "module w\n\nfn main(root: Root) ! {{Write}} {{\n    let fw = root.fs_write(\"./{HOSTILE_PATH}\")\n    \
             match fw.write_text(\"f.txt\", \"x\") {{\n        Ok(_) => root.console().println(\"wrote\"),\n        \
             Err(_) => root.console().println(\"refused\")\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let r = delulu(&d, &["run", "w.delulu", "--grant", "console"]);
    assert!(text(&r).contains("DL0703"), "{}", text(&r));
    assert_no_control(&r, "a refusal quoting the program's path");
    let r = delulu(&d, &["run", "w.delulu", "--grant", "console", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap_or_else(|_| panic!("{}", text(&r)));
    assert!(v.to_string().contains(r"\u001b]0;PWNED"), "JSON keeps the exact value, JSON-escaped: {v}");
    let _ = std::fs::remove_dir_all(&d);
}

/// A quoted source line keeps its columns: a raw control character in it is shown as `?`, so the caret
/// under the span still points at the span.
#[test]
fn a_quoted_source_line_carries_no_control_character() {
    let d = lab("snippet");
    std::fs::write(d.join("c.delulu"), "module c\n\nfn main(root: Root) {\n    let x: Int = \"s\" // \u{1b}]0;PWNED\u{7}\n}\n").unwrap();
    let r = delulu(&d, &["check", "c.delulu"]);
    assert_ne!(r.status.code(), Some(0), "the line has a type error to quote: {}", text(&r));
    assert!(text(&r).contains("let x: Int"), "the line is quoted: {}", text(&r));
    assert_no_control(&r, "a quoted source line");
    let _ = std::fs::remove_dir_all(&d);
}

/// `delulu test` prints each test's name and its failure on one line; neither may carry a control
/// sequence, or a line break that starts a forged line.
#[test]
fn a_tests_name_and_failure_stay_one_escaped_line() {
    let d = lab("test");
    std::fs::write(
        d.join("u.delulu"),
        format!("module u\n\ntest \"{HOSTILE}\\ntest result: 9 passed, 0 failed\" {{\n    assert_eq(\"{HOSTILE}\", \"x\")\n}}\n"),
    )
    .unwrap();
    let r = delulu(&d, &["test", "u.delulu"]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert_no_control(&r, "a test's name and failure");
    let all = text(&r);
    let results: Vec<&str> = all.lines().filter(|l| l.starts_with("test result:")).collect();
    assert_eq!(results, ["test result: 0 passed, 1 failed"], "exactly one summary, the real one: {all}");
    let _ = std::fs::remove_dir_all(&d);
}

// ===== RW 4.32 · a guest's standard error, relayed ==============================================
//
// A guest's standard error was the operator's terminal itself, inherited: the guest's own lines were
// escaped by TERMINAL-TEXT-1, but a guest that ESCAPED its interpreter — or an external launcher, which
// carries whatever the guest in it writes — wrote raw bytes there, control sequences and lines that read
// like the host's. Now the host reads it and prints each line escaped, marked with whose it is, bounded.

/// The guest's own fault line reaches the operator marked as the guest's — on every OS, through the
/// host's relay, never unmarked.
#[test]
fn a_sandboxed_guests_own_words_arrive_marked_as_the_guests() {
    let d = lab("mark");
    std::fs::write(d.join("t.delulu"), "module t\n\nfn main(root: Root) {\n    assert_eq(\"a\", \"b\")\n}\n").unwrap();
    let r = delulu(&d, &["run", "t.delulu", "--sandbox"]);
    let all = text(&r);
    assert!(all.lines().any(|l| l.starts_with("guest: error[DL1707]")), "the guest's fault, marked: {all}");
    assert!(!all.lines().any(|l| l.starts_with("error[DL1707]")), "and never unmarked: {all}");
    #[cfg(target_os = "linux")]
    assert!(
        !all.lines().any(|l| l.starts_with("sandbox: the guest narrowed") || l.starts_with("sandbox: the guest locked")),
        "a guest's words about its own boundary are marked as its own: {all}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[cfg(unix)]
fn launcher(d: &Path, before_guest: &str) -> String {
    use std::os::unix::fs::PermissionsExt as _;
    let p = d.join("lnch");
    std::fs::write(&p, format!("#!/bin/sh\n{before_guest}\nexec {} $DELULU_GUEST_ARGS\n", env!("CARGO_BIN_EXE_delulu"))).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    format!("external:{}", p.display())
}

#[cfg(unix)]
fn hello(d: &Path) {
    std::fs::write(d.join("h.delulu"), "module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
}

/// What an escaped guest would write — control sequences and a line that reads like the host's —
/// arrives escaped and marked as the launcher's (at L3 the host cannot tell the launcher's words from
/// the guest's inside it).
#[cfg(unix)]
#[test]
fn a_launchers_standard_error_arrives_escaped_and_marked() {
    let d = lab("launcher");
    hello(&d);
    let ext = launcher(&d, r"printf '\033]0;PWNED\007\033[2K\rsandbox: the guest is confined — forged\n' >&2");
    let r = delulu(&d, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &ext]);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert_no_control(&r, "a launcher's standard error");
    let all = text(&r);
    assert!(all.lines().any(|l| l.starts_with("launcher: ") && l.contains("forged")), "marked as the launcher's: {all}");
    assert!(!all.lines().any(|l| l.starts_with("sandbox: the guest is confined — forged")), "never as the host's: {all}");
    let _ = std::fs::remove_dir_all(&d);
}

/// A flood on standard error is cut off, said once, and drained, so the run goes on.
#[cfg(unix)]
#[test]
fn a_flood_on_standard_error_is_bounded_and_the_run_goes_on() {
    let d = lab("flood");
    hello(&d);
    let ext = launcher(&d, "yes 0123456789012345678901234567890123456789012345678901234567890123456789 | head -c 3145728 >&2");
    let r = delulu(&d, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &ext]);
    assert_eq!(r.status.code(), Some(0), "the run goes on: {}", String::from_utf8_lossy(&r.stdout));
    assert!(String::from_utf8_lossy(&r.stdout).contains("hi"));
    assert!(r.stderr.len() < 1_500_000, "3 MiB on the launcher's standard error became {} bytes on the operator's", r.stderr.len());
    assert!(String::from_utf8_lossy(&r.stderr).contains("the rest was discarded"), "and it says so");
    let _ = std::fs::remove_dir_all(&d);
}

/// The host waits, briefly, for the last line: here it is written a second AFTER the guest has gone,
/// by a process the launcher left holding its standard error — the window a host that exits at once
/// would close on it.
#[cfg(unix)]
#[test]
fn the_last_line_is_relayed_before_the_host_reports() {
    let d = lab("last");
    hello(&d);
    let ext = launcher(&d, "(sleep 1; echo 'the last word' >&2) &");
    let r = delulu(&d, &["run", "h.delulu", "--grant", "console", "--sandbox", "--sandbox-backend", &ext]);
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).lines().any(|l| l == "launcher: the last word"), "the last line was relayed: {}", text(&r));
    let _ = std::fs::remove_dir_all(&d);
}
