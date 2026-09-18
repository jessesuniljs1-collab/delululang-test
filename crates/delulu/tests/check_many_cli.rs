//! `delulu check` takes several files, and every other command refuses what it cannot act on.
//!
//! Both halves came out of one measurement. `measurements/agent-loop/RECORD.md` found that a bare
//! process spawn costs 27.2 ms on Windows and 3.7 ms on Linux, while checking a 35-line file costs
//! about 1.1 ms — so the loop an agent actually runs was paying the floor once per file. Looking at
//! whether one process could do several turned up something worse than a missing feature: the
//! argument parser kept the first non-flag argument and **dropped the rest in silence**.
//!
//! `delulu check a.delulu b.delulu` printed `ok: a.delulu checked clean` and exited **0** while
//! `b.delulu` — never opened — held two errors. A shell glob did the same. Reporting success about
//! work that was not done is the failure this toolchain exists to refuse, so the tests that matter
//! most in this file are the ones about arguments, not the ones about speed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-many-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    // One clean module, one with two real errors, one more clean module.
    std::fs::write(d.join("a.delulu"), "module a\n\nfn one() -> Int { 1 }\n").unwrap();
    std::fs::write(
        d.join("bad.delulu"),
        "module bad\n\nfn oops() -> Int { this_does_not_exist() }\n",
    )
    .unwrap();
    std::fs::write(d.join("c.delulu"), "module c\n\nfn two() -> Int { 2 }\n").unwrap();
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

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// **The defect, stated as a test.** A file that was passed and not looked at must never be
/// reported as clean, and the exit code must reflect every file named.
///
/// Against the code before this change, this test fails on its first assertion: exit 0.
#[test]
fn a_second_file_is_checked_and_not_silently_dropped() {
    let d = scratch("drop");
    let o = delulu_in(&d, &["check", "a.delulu", "bad.delulu"]);
    assert_eq!(
        o.status.code(),
        Some(1),
        "a file with errors was passed; exit 0 would be a false clean:\n{}",
        text(&o)
    );

    let t = text(&o);
    assert!(t.contains("DL0301"), "the second file's error is reported:\n{t}");
    assert!(t.contains("bad.delulu"), "and attributed to the file it came from:\n{t}");
    assert!(t.contains("a.delulu"), "the clean file is named too:\n{t}");
}

/// Order does not matter: the failing file first is the same verdict.
#[test]
fn the_verdict_does_not_depend_on_argument_order() {
    let d = scratch("order");
    let first = delulu_in(&d, &["check", "bad.delulu", "a.delulu"]);
    let second = delulu_in(&d, &["check", "a.delulu", "bad.delulu"]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
}

/// Every file is named, including the clean ones — otherwise a reader cannot tell a file that
/// passed from one that was skipped, which is the defect in another costume.
#[test]
fn all_clean_files_are_named_and_the_run_succeeds() {
    let d = scratch("clean");
    let o = delulu_in(&d, &["check", "a.delulu", "c.delulu"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let t = text(&o);
    assert!(t.contains("a.delulu"), "{t}");
    assert!(t.contains("c.delulu"), "{t}");
    assert!(t.contains("2 file(s) checked"), "and a summary line:\n{t}");
}

/// One file must behave exactly as it always did — the multi-file form is additive.
///
/// `core_invariance.rs` pins this across all 108 shipped targets; this is the local statement of
/// the same rule, so a reader of this file sees the constraint the change was written under.
#[test]
fn a_single_file_still_reports_the_way_it_always_did() {
    let d = scratch("single");
    let o = delulu_in(&d, &["check", "a.delulu"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    let t = text(&o);
    assert!(t.contains("ok: a.delulu checked clean"), "{t}");
    assert!(!t.contains("file(s) checked"), "no summary line for one file:\n{t}");
}

/// The machine contract is **one** object, whatever the file count.
#[test]
fn several_files_still_emit_exactly_one_json_object() {
    let d = scratch("json");
    let o = delulu_in(&d, &["check", "a.delulu", "bad.delulu", "c.delulu", "--json"]);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(&out).expect("exactly one JSON object");
    assert_eq!(v["command"], "check");
    // One: `unknown name` alone. The second error was the DL0404 cascade on the checker's own
    // placeholder type, which P1-F1 (D-V2-19) removed.
    assert_eq!(v["summary"]["errors"], 1, "the bad file's error, and no cascade:\n{out}");

    // Each diagnostic still names the file it came from, which is what makes one merged envelope
    // as useful as three separate ones.
    let files: Vec<String> = v["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .map(|d| d["spans"][0]["file"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(files.iter().all(|f| f == "bad.delulu"), "{files:?}");
}

/// A file that cannot be read stops the run rather than being counted clean.
#[test]
fn a_missing_file_among_several_is_refused() {
    let d = scratch("missing");
    let o = delulu_in(&d, &["check", "a.delulu", "not_here.delulu"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("cannot read"), "{}", text(&o));
    assert!(
        !text(&o).contains("checked clean"),
        "nothing may be reported clean when the run did not finish:\n{}",
        text(&o)
    );
}

/// A package is checked as a whole; mixing one with loose files would blur which manifest ceiling
/// applied to what, so it is refused rather than guessed at.
#[test]
fn a_package_directory_may_not_be_mixed_with_loose_files() {
    let d = scratch("mixed");
    std::fs::create_dir_all(d.join("pkg").join("src")).unwrap();
    std::fs::write(
        d.join("pkg").join("delulu.toml"),
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\nkind = \"lib\"\n",
    )
    .unwrap();
    std::fs::write(d.join("pkg").join("src").join("pkg.delulu"), "module pkg\n").unwrap();

    let o = delulu_in(&d, &["check", "pkg", "a.delulu"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("takes one path"), "{}", text(&o));
}

/// **Every command that takes one path says so, instead of using the first and ignoring the rest.**
///
/// This is the half that is not about speed. Each of these silently discarded its extra argument
/// before, so a mistyped command line got a confident answer about the wrong file.
#[test]
fn single_path_commands_refuse_a_second_path() {
    let d = scratch("refuse");
    for args in [
        vec!["authority", "a.delulu", "c.delulu"],
        vec!["run", "a.delulu", "c.delulu"],
        vec!["why", "Write", "a.delulu", "c.delulu"],
        vec!["build", "a.delulu", "c.delulu"],
        vec!["lock", ".", ".."],
    ] {
        let o = delulu_in(&d, &args);
        let t = text(&o);
        assert_eq!(o.status.code(), Some(2), "`{}` must refuse:\n{t}", args.join(" "));
        assert!(t.contains("takes one path"), "`{}`:\n{t}", args.join(" "));
        assert!(
            t.contains("silently ignored"),
            "the refusal says why it is a refusal:\n{t}"
        );
    }
}

/// A refusal keeps the machine contract too — one envelope, reporting that nothing ran.
#[test]
fn a_refusal_still_emits_one_json_envelope() {
    let d = scratch("refusejson");
    let o = delulu_in(&d, &["authority", "a.delulu", "c.delulu", "--json"]);
    assert_eq!(o.status.code(), Some(2));
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(&out).expect("exactly one JSON object");
    assert_eq!(v["command"], "authority");
    assert_eq!(v["summary"]["errors"], 1, "{out}");
}
