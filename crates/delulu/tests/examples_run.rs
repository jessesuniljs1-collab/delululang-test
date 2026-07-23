//! Every shipped example checks clean — and every one that can run, runs.
//!
//! The Book's samples already had the first half (`book.rs`, criterion 7), and its reasoning is
//! right: a tutorial whose examples do not compile teaches people something false. But checking
//! is only half of it. `examples/guide/04_effects.delulu` checked clean and then faulted at
//! runtime with "unbound name `double`", because passing a NAMED function as a value was
//! accepted by the checker and unimplemented in the interpreter (HARDENING_CAMPAIGN C13). The
//! reference sample for row polymorphism has that exact shape and nobody noticed, because
//! nothing ever ran it.
//!
//! So this file adds the other half. The running assertion is deliberately narrow: a program
//! that checks clean must never fail with **DL0907**, the code the runtime raises when the
//! checker let something through. Every other runtime outcome is legitimate — an example may
//! want a file that is not there, or a grant this test did not hand it — and the test says
//! nothing about those.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// Every `.delulu` file directly under `examples/` or `examples/guide/`. Package directories
/// (`greeter`, `plugin_shout`) are checked as packages by `tooling.rs` and are skipped here.
fn example_files() -> Vec<PathBuf> {
    let mut v = Vec::new();
    for dir in ["examples", "examples/guide"] {
        let d = root().join(dir);
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

#[test]
fn every_example_checks_clean() {
    let files = example_files();
    assert!(files.len() >= 7, "expected the example set, found {}", files.len());
    for f in &files {
        let o = delulu(&["check", &rel(f)]);
        assert!(
            o.status.success(),
            "{} does not check clean:\n{}",
            rel(f),
            text(&o)
        );
    }
}

#[test]
fn no_example_that_checks_clean_fails_with_a_checker_bug_code() {
    // A generous grant set: the point is to reach as much of each program as possible, not to
    // model least privilege. Examples that still refuse for want of a grant or a file are fine.
    let grants = [
        "--grant", "console",
        "--grant", "clock",
        "--grant", "rand",
        "--grant", "fs.read=.",
        "--grant", "fs.write=.",
    ];
    for f in &example_files() {
        let src = std::fs::read_to_string(f).expect("read example");
        if !src.contains("fn main(") {
            continue; // nothing to run — `check` already covered it
        }
        let path = rel(f);
        let mut args: Vec<&str> = vec!["run", &path, "--no-prompt"];
        args.extend_from_slice(&grants);
        let o = delulu(&args);
        let out = text(&o);
        assert!(
            !out.contains("DL0907"),
            "{path} checks clean but fails at runtime with DL0907 — the checker let something \
             through. This is a compiler bug, not the example's fault:\n{out}"
        );
    }
}
