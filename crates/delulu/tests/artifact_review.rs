//! Reviewing what you actually ship (campaign findings C65 and C66, ruling D59).
//!
//! A `.dwx` is the **distribution** format, and the Book's claim for it is "authority that travels
//! with the code". So the `.dwx` is the artifact a reviewer is handed — and `delulu authority`, the
//! command whose entire job is "everything this program can do", could not read one. It fell through
//! to the source loader and died with `stream did not contain valid UTF-8`, while `delulu run` on the
//! same file happily verified the same embedded manifest and printed the declared effects. The
//! manifest was always there. Nothing asked it.
//!
//! The second finding is smaller and the same family as C26: `delulu fmt notes.txt` reported
//! "reformatted 0 file(s)" and exited **0** — nothing done, success claimed, on a path the user chose
//! deliberately.

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

/// A program inside the WASM fragment, so it can actually become a `.dwx`: Int arithmetic,
/// recursion, `match` on a sum, `Str` concat, console. No loop, no float, no record, no list —
/// see the Book's Chapter 9 for why that list is what it is.
const FRAGMENT_PROGRAM: &str = "module frag\n\
     type Verdict = Yes | No\n\
     fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\n\
     fn decide(n: Int) -> Verdict { if n > 100 { Yes } else { No } }\n\
     fn say(v: Verdict) -> Str { match v { Yes => \"big\", No => \"small\" } }\n\
     fn main(root: Root) ! {Write} {\n\
     \x20 let o = root.console()\n\
     \x20 o.println(\"fib(20)=\" + str(fib(20)) + \" is \" + say(decide(fib(20))))\n\
     }\n";

fn rig(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu-artifact-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("frag.delulu"), FRAGMENT_PROGRAM).unwrap();
    let o = delulu(&dir, &["build", "frag.delulu", "--target", "wasm", "-o", "frag.dwx"]);
    assert!(o.status.success(), "the fragment program must compile: {}", stderr(&o));
    dir
}

/// C65. The review surface can review the shipped artifact.
#[test]
fn authority_reports_a_compiled_artifacts_own_embedded_manifest() {
    let dir = rig("auth");

    let o = delulu(&dir, &["authority", "frag.dwx"]);
    assert!(o.status.success(), "authority on a .dwx must work: {}", stderr(&o));
    let s = stdout(&o);
    assert!(s.contains("Write"), "the declared effect must be reported:\n{s}");
    // It must not be mistaken for a source report. A `.dwx` has no code in it, so the things that
    // need code are absent, and saying so is the difference between a report and a claim.
    assert!(
        s.contains("COMPILED ARTIFACT") && s.contains("no source"),
        "the report must say what it is and what it therefore cannot tell you:\n{s}"
    );

    let o = delulu(&dir, &["authority", "frag.dwx", "--json"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("--json must emit one object");
    assert_eq!(v["kind"], "dwx");
    assert_eq!(v["verified"], true);
    assert!(v["authority"].is_object(), "the embedded manifest must be in the envelope: {v}");
}

/// C65, and the property that makes it worth having: **the review surface must not vouch for bytes
/// the runtime would refuse.** Both go through the same `read_and_verify`, so a tampered artifact is
/// the same code in both places — not "reported clean here, refused there", which would be a worse
/// state than not being able to read it at all.
#[test]
fn a_tampered_artifact_is_refused_identically_by_authority_and_by_run() {
    let dir = rig("tamper");
    let mut bytes = std::fs::read(dir.join("frag.dwx")).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    std::fs::write(dir.join("frag.dwx"), &bytes).unwrap();

    let a = delulu(&dir, &["authority", "frag.dwx"]);
    let r = delulu(&dir, &["run", "frag.dwx", "--grant", "console", "--no-prompt"]);
    assert!(!a.status.success(), "authority must refuse a tampered artifact: {}", stdout(&a));
    assert!(!r.status.success(), "run must refuse a tampered artifact");
    let (ae, re) = (stderr(&a) + &stdout(&a), stderr(&r) + &stdout(&r));
    assert!(ae.contains("DL1202"), "authority's refusal should be the artifact-integrity code: {ae}");
    assert!(re.contains("DL1202"), "run's refusal should be the same code: {re}");
}

/// C65's other half: a source command handed an artifact must say what it is, not leak an encoding
/// error. `stream did not contain valid UTF-8` tells the reader nothing about what they did.
#[test]
fn a_source_command_handed_an_artifact_says_so_instead_of_leaking_an_encoding_error() {
    let dir = rig("src");
    for cmd in [
        vec!["check", "frag.dwx"],
        vec!["why", "Write", "frag.dwx"],
        vec!["atlas", "frag.dwx"],
    ] {
        let o = delulu(&dir, &cmd);
        assert!(!o.status.success(), "{cmd:?} must refuse an artifact");
        let e = stderr(&o);
        assert!(
            e.contains("compiled `.dwx` artifact") && e.contains("not DeluluLang source"),
            "{cmd:?} must name what the file is: {e}"
        );
        assert!(
            e.contains("delulu authority") && e.contains("delulu run"),
            "{cmd:?} must point at the two commands that CAN read it: {e}"
        );
        assert!(!e.contains("valid UTF-8"), "{cmd:?} still leaks the encoding error: {e}");
    }

    // Detection is by CONTENT, not extension: the same mistake with the file renamed gets the same
    // answer, because an extension is not evidence.
    std::fs::copy(dir.join("frag.dwx"), dir.join("renamed.delulu")).unwrap();
    let o = delulu(&dir, &["check", "renamed.delulu"]);
    assert!(!o.status.success());
    assert!(
        stderr(&o).contains("compiled `.dwx` artifact"),
        "a renamed artifact is still an artifact: {}",
        stderr(&o)
    );
}

/// C66. `fmt` on a file it will not format must not report success.
#[test]
fn fmt_refuses_a_file_it_cannot_format_instead_of_reporting_zero_work_done() {
    let dir = rig("fmt");
    std::fs::write(dir.join("notes.txt"), "this is not delulu source\n").unwrap();

    for f in ["notes.txt", "frag.dwx"] {
        let o = delulu(&dir, &["fmt", f]);
        assert!(
            !o.status.success(),
            "`fmt {f}` reported success having done nothing — the C26 shape: {}",
            stdout(&o)
        );
        assert!(
            stderr(&o).contains("not a `.delulu` source file"),
            "`fmt {f}` must say why: {}",
            stderr(&o)
        );
    }

    // And the behaviour that must NOT change: a DIRECTORY is filtered, because filtering is the
    // entire point of walking one. A tree containing no `.delulu` is not a user mistake.
    let o = delulu(&dir, &["fmt", "."]);
    assert!(o.status.success(), "formatting a directory must still work: {}", stderr(&o));
    assert!(stdout(&o).contains("fmt:"), "and still report: {}", stdout(&o));
}
