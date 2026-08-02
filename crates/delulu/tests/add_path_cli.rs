//! `delulu add --path` — declaring a dependency without hand-writing its authority pin.
//!
//! There is no hosted registry, so a real dependency today is a directory beside yours. Declaring
//! one meant hand-writing `{ path = …, authority = { effects = […] } }` and *guessing* the pin —
//! then learning the right value by reading DL1001. The toolchain already knows it.
//!
//! The tests that matter here are the refusals. Adding a dependency is the moment a supply chain
//! acquires new authority, and a tool that quietly widened a manifest at that moment would be
//! doing the one thing this language exists to prevent.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-addpath-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn delulu_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .current_dir(cwd)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// A workspace with an app, a pure library, and a library that performs `Write`.
fn workspace(tag: &str) -> PathBuf {
    let w = scratch(tag);
    for args in [
        vec!["new", "app"],
        vec!["new", "purelib", "--lib"],
        vec!["new", "writerlib", "--lib"],
    ] {
        let o = delulu_in(&w, &args);
        assert_eq!(o.status.code(), Some(0), "scaffolding failed: {}", stderr(&o));
    }
    // Give `writerlib` something to do, and the ceiling to do it under.
    std::fs::write(
        w.join("writerlib").join("delulu.toml"),
        "[package]\nname = \"writerlib\"\nversion = \"0.1.0\"\nkind = \"lib\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();
    std::fs::write(
        w.join("writerlib").join("src").join("writerlib.delulu"),
        "module writerlib\n\npub fn shout(out: Cap[Console], n: Str) ! {Write} {\n    out.println(n)\n}\n",
    )
    .unwrap();
    w
}

fn manifest(w: &Path) -> String {
    std::fs::read_to_string(w.join("app").join("delulu.toml")).unwrap()
}

/// A dependency that cannot do anything needs no decision, so it is simply added.
#[test]
fn a_pure_dependency_is_added_with_an_empty_pin() {
    let w = workspace("pure");
    let o = delulu_in(&w.join("app"), &["add", "--path", "../purelib"]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));

    let m = manifest(&w);
    assert!(m.contains("[dependencies]"), "the section is created:\n{m}");
    assert!(
        m.contains("purelib = { path = \"../purelib\", authority = { effects = [] } }"),
        "pinned at nothing, because it needs nothing:\n{m}"
    );
    assert!(stdout(&o).contains("provably pure"), "{}", stdout(&o));

    // And the result is a package that checks.
    let chk = delulu_in(&w.join("app"), &["check", "."]);
    assert_eq!(chk.status.code(), Some(0), "the edited manifest checks:\n{}", stderr(&chk));
}

/// **A dependency that needs authority is shown and refused until you name the decision.**
///
/// This is the line worth keeping if every other test here were deleted. `delulu add` is the kind
/// of command people run without reading the output, and the thing it would be granting is not a
/// version range — it is what the program may do to the machine it runs on.
#[test]
fn a_dependency_that_needs_authority_is_refused_until_it_is_accepted() {
    let w = workspace("refuse");
    let before = manifest(&w);

    let o = delulu_in(&w.join("app"), &["add", "--path", "../writerlib"]);
    assert_eq!(o.status.code(), Some(1), "it must refuse:\n{}", stderr(&o));
    assert_eq!(manifest(&w), before, "and write nothing at all");

    let err = stderr(&o);
    assert!(err.contains("Write"), "it names what the dependency can do:\n{err}");
    assert!(err.contains("nothing was written"), "and says it did not act:\n{err}");
    // A refusal a reader cannot act on is an obstacle, not a safeguard.
    assert!(
        err.contains("--accept-authority"),
        "and gives the exact command to accept:\n{err}"
    );

    // Named, it goes through — and pins exactly what the dependency needs, no more.
    let ok = delulu_in(&w.join("app"), &["add", "--path", "../writerlib", "--accept-authority"]);
    assert_eq!(ok.status.code(), Some(0), "{}", stderr(&ok));
    let m = manifest(&w);
    assert!(
        m.contains("writerlib = { path = \"../writerlib\", authority = { effects = [\"Write\"] } }"),
        "the pin is exactly the dependency's authority:\n{m}"
    );
    let chk = delulu_in(&w.join("app"), &["check", "."]);
    assert_eq!(chk.status.code(), Some(0), "and it checks clean:\n{}", stderr(&chk));
}

/// The pin is computed from the dependency, not defaulted to something permissive.
///
/// The tempting shortcut is a pin that covers everything so the first `check` passes. That pin
/// would never be tightened, and `authority --diff` would have nothing to notice when the
/// dependency later grew.
#[test]
fn the_pin_is_the_dependencys_own_authority_and_nothing_more() {
    let w = workspace("pin");
    delulu_in(&w.join("app"), &["add", "--path", "../writerlib", "--accept-authority"]);
    let m = manifest(&w);
    let line = m.lines().find(|l| l.starts_with("writerlib =")).expect("the pin");
    assert!(line.contains("[\"Write\"]"), "{line}");
    for wider in ["Read", "Net", "Clock", "ForeignCall", "Actuate"] {
        assert!(!line.contains(wider), "the pin must not include {wider}: {line}");
    }
}

/// An existing pin is never rewritten — that line is the one a reviewer reads.
#[test]
fn an_existing_dependency_is_never_rewritten() {
    let w = workspace("dup");
    delulu_in(&w.join("app"), &["add", "--path", "../purelib"]);
    let after_first = manifest(&w);

    let o = delulu_in(&w.join("app"), &["add", "--path", "../purelib"]);
    assert_eq!(o.status.code(), Some(2), "a second add is refused:\n{}", stderr(&o));
    assert_eq!(manifest(&w), after_first, "and the manifest is untouched");
    assert!(stderr(&o).contains("already a dependency"), "{}", stderr(&o));
}

/// A dependency whose authority cannot be computed is not pinned at a value nobody can verify.
#[test]
fn a_dependency_that_does_not_check_is_refused() {
    let w = workspace("broken");
    std::fs::write(
        w.join("purelib").join("src").join("purelib.delulu"),
        "module purelib\n\npub fn broken() -> Int { this_does_not_exist() }\n",
    )
    .unwrap();
    let before = manifest(&w);

    let o = delulu_in(&w.join("app"), &["add", "--path", "../purelib"]);
    assert_eq!(o.status.code(), Some(1), "refused:\n{}", stderr(&o));
    assert_eq!(manifest(&w), before, "nothing written");
    assert!(stderr(&o).contains("does not check clean"), "{}", stderr(&o));
}

/// Run outside a package, or pointed at something that is not one, it says so plainly.
#[test]
fn it_refuses_what_is_not_a_package() {
    let w = workspace("notapkg");

    // No manifest in the current directory.
    let o = delulu_in(&w, &["add", "--path", "purelib"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(stderr(&o).contains("no `delulu.toml` here"), "{}", stderr(&o));

    // The target is not a package.
    std::fs::create_dir_all(w.join("empty")).unwrap();
    let o2 = delulu_in(&w.join("app"), &["add", "--path", "../empty"]);
    assert_eq!(o2.status.code(), Some(2), "{}", stderr(&o2));
    assert!(stderr(&o2).contains("not a DeluluLang package"), "{}", stderr(&o2));
}

/// The machine surface reports what was granted, and whether anything was written.
#[test]
fn the_json_envelope_says_what_was_granted() {
    let w = workspace("json");

    let refused = delulu_in(&w.join("app"), &["add", "--path", "../writerlib", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&refused)).expect("one object");
    assert_eq!(v["command"], "add");
    assert_eq!(v["written"], false, "a refusal says so on the machine channel too");
    assert_eq!(v["authority"]["effects"][0], "Write");

    let ok = delulu_in(&w.join("app"), &["add", "--path", "../purelib", "--json"]);
    let v2: serde_json::Value = serde_json::from_str(&stdout(&ok)).expect("one object");
    assert_eq!(v2["written"], true);
    assert_eq!(v2["name"], "purelib");
    assert!(v2["authority"]["effects"].as_array().unwrap().is_empty());
}
