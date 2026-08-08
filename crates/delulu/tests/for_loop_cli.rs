//! `for` / `break` / `continue` executed end-to-end (Tier 1, owner-directed 2026-08-08).
//!
//! Conformance pins that a for-loop *checks clean* and that DL0411/DL0412 *fire*; the core-invariance
//! snapshot pins deterministic analysis surfaces. Neither runs a loop and reads what it printed —
//! and the loop's whole point is its runtime behavior. These tests drive the real binary and assert
//! the output, because "checks clean" and "iterates correctly" are different claims.

use std::path::PathBuf;
use std::process::Output;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu_bin() -> PathBuf {
    // The test binary lives in target/<profile>/deps; the CLI is two levels up.
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.push(if cfg!(windows) { "delulu.exe" } else { "delulu" });
    p
}

fn run_program(src: &str, grants: &[&str]) -> Output {
    let dir = std::env::temp_dir().join(format!("delulu-forloop-{}-{}", std::process::id(), rand_tag()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.delulu");
    std::fs::write(&file, src).unwrap();
    let mut args = vec!["run".to_string(), file.to_string_lossy().to_string()];
    for g in grants {
        args.push("--grant".to_string());
        args.push((*g).to_string());
    }
    let out = std::process::Command::new(delulu_bin())
        .args(&args)
        .current_dir(repo_root())
        .output()
        .expect("run the delulu binary");
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn check_program(src: &str) -> String {
    let dir = std::env::temp_dir().join(format!("delulu-forcheck-{}-{}", std::process::id(), rand_tag()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.delulu");
    std::fs::write(&file, src).unwrap();
    let out = std::process::Command::new(delulu_bin())
        .args(["check", &file.to_string_lossy()])
        .current_dir(repo_root())
        .output()
        .expect("check the delulu binary");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)
}

fn rand_tag() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// `break` ends the loop and `continue` skips one element — asserted by what the program prints,
/// not by the fact that it exits 0.
#[test]
fn break_stops_and_continue_skips() {
    let src = r#"module m
fn main(root: Root) ! {Write} {
    let out = root.console()
    var total = 0
    for x in [1, 2, 3, 4, 5] {
        if x == 4 { break }
        if x == 2 { continue }
        total = total + x
    }
    out.println("total=" + str(total))
}
"#;
    let out = run_program(src, &["console"]);
    // 1 kept, 2 skipped (continue), 3 kept, 4 stops (break) => 1 + 3 = 4.
    assert!(stdout(&out).contains("total=4"), "expected total=4, got: {}", stdout(&out));
}

/// The loop iterates a SNAPSHOT: pushing to the list inside the body must not lengthen the loop.
/// Without the snapshot this either loops forever or iterates the growing list.
#[test]
fn the_loop_iterates_a_snapshot_not_the_live_list() {
    let src = r#"module m
fn main(root: Root) ! {Write} {
    let out = root.console()
    let xs = [1, 2, 3]
    var n = 0
    for x in xs {
        let _ = x
        xs.push(99)
        n = n + 1
    }
    out.println("iterations=" + str(n))
}
"#;
    let out = run_program(src, &["console"]);
    assert!(
        stdout(&out).contains("iterations=3"),
        "the loop must run exactly 3 times over the original list, got: {}",
        stdout(&out)
    );
}

/// A nested loop's `break` escapes only the inner loop — the outer one keeps going.
#[test]
fn break_escapes_only_the_innermost_loop() {
    let src = r#"module m
fn main(root: Root) ! {Write} {
    let out = root.console()
    var seen = 0
    for a in [1, 2] {
        for b in [10, 20, 30] {
            if b == 20 { break }
            seen = seen + 1
        }
    }
    out.println("seen=" + str(seen))
}
"#;
    let out = run_program(src, &["console"]);
    // Each outer iteration: inner runs for b=10 (seen++), breaks at b=20. 2 outer * 1 = 2.
    assert!(stdout(&out).contains("seen=2"), "expected seen=2, got: {}", stdout(&out));
}

/// `break` outside any loop is refused at compile time (DL0412), not accepted and mishandled.
#[test]
fn break_outside_a_loop_is_dl0412() {
    let out = check_program("module m\nfn f() {\n    break\n}\n");
    assert!(out.contains("DL0412"), "expected DL0412 for break outside a loop, got: {out}");
}

/// `for` over a non-list is refused (DL0411).
#[test]
fn for_over_a_non_list_is_dl0411() {
    let out = check_program("module m\nfn f() {\n    for x in 5 {\n        let _ = x\n    }\n}\n");
    assert!(out.contains("DL0411"), "expected DL0411 for a non-list iterable, got: {out}");
}

/// An actor constructor that assigns `self.field` ONLY inside a `for` loop must check clean. The
/// definite-assignment scan (`collect_self_field_assigns`) recursed into `while`/`if`/blocks but not
/// `for`, so it missed the assignment and falsely reported DL0405 "constructor never assigns field".
/// This pins the fix: a valid actor is not rejected because its assignment happens to be in a loop.
#[test]
fn an_actor_field_assigned_only_in_a_for_loop_is_not_falsely_unassigned() {
    let src = "module m\n\
actor Acc {\n\
    var seen: Int\n\
    new(xs: List[Int]) {\n\
        for x in xs {\n\
            self.seen = x\n\
        }\n\
    }\n\
    be get(out: Cap[Console]) ! {Write} {\n\
        out.println(str(self.seen))\n\
    }\n\
}\n";
    let out = check_program(src);
    assert!(
        !out.contains("DL0405") && out.contains("checked clean"),
        "an actor field assigned in a for-loop must not be reported unassigned, got: {out}"
    );
}
