//! Stage 8, phase 8d: `delulu fmt` through the real binary — the canonical formatter's
//! CLI surface (criterion 4's `--check`/`--stdin` halves; the ≥100k law gate lives in
//! `delulu-syntax::fmt` beside the printer).

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_fmt_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn delulu_in(dir: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env_remove("DELULU_LOCALE")
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("run delulu")
}

const MESSY: &str = "module m\nfn add(a:Int,b:Int)->Int !{} {a+b}\n";
const CANON: &str = "module m\n\nfn add(a: Int, b: Int) -> Int ! {} {\n    a + b\n}\n";

#[test]
fn fmt_rewrites_in_place_then_check_goes_green() {
    let dir = tmp("write");
    let f = dir.join("m.delulu");
    std::fs::write(&f, MESSY).unwrap();

    // `--check` on a messy file exits 1 and names it, touching nothing.
    let o = delulu_in(&dir, &["fmt", "m.delulu", "--check"]);
    assert_eq!(o.status.code(), Some(1), "--check must exit 1 on an unformatted file");
    assert!(String::from_utf8_lossy(&o.stdout).contains("would reformat: m.delulu"));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), MESSY, "--check never writes");

    // Formatting rewrites to the canonical style…
    let o2 = delulu_in(&dir, &["fmt", "m.delulu"]);
    assert_eq!(o2.status.code(), Some(0), "{}", String::from_utf8_lossy(&o2.stderr));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), CANON);

    // …after which `--check` is green and a second format is a no-op (idempotence,
    // observed at the file level).
    let o3 = delulu_in(&dir, &["fmt", "m.delulu", "--check"]);
    assert_eq!(o3.status.code(), Some(0));
    let o4 = delulu_in(&dir, &["fmt", "m.delulu"]);
    assert!(String::from_utf8_lossy(&o4.stdout).contains("1 already canonical"));
}

#[test]
fn fmt_stdin_formats_to_stdout() {
    let dir = tmp("stdin");
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(&dir)
        .args(["fmt", "--stdin"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(MESSY.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), CANON);
}

#[test]
fn fmt_refuses_unparseable_files_and_exits_1() {
    let dir = tmp("refuse");
    let f = dir.join("bad.delulu");
    std::fs::write(&f, "module m\nfn broken( {\n").unwrap();
    let o = delulu_in(&dir, &["fmt", "bad.delulu"]);
    assert_eq!(o.status.code(), Some(1), "refusal is an error exit");
    assert_eq!(
        std::fs::read_to_string(&f).unwrap(),
        "module m\nfn broken( {\n",
        "an unparseable file is NEVER rewritten"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("DL02") || err.contains("DL01"), "the real parse diagnostics surface: {err}");
}

#[test]
fn migrate_07_still_works_unchanged() {
    let dir = tmp("migrate");
    let f = dir.join("old.delulu");
    std::fs::write(&f, "module m\nfn f() -> Int {\n    let consume = 1\n    consume\n}\n").unwrap();
    let o = delulu_in(&dir, &["fmt", "--migrate", "0.7", "old.delulu"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let migrated = std::fs::read_to_string(&f).unwrap();
    assert!(migrated.contains("consume_"), "{migrated}");
}
