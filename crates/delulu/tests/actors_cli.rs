//! Stage 7 close-out witnesses through the REAL binary:
//! - criterion 5: `delulu why Write` shows the causal chain `main → send → behavior →
//!   primitive op` ACROSS the actor boundary;
//! - criterion 6 (native half): an actor program runs clean under
//!   `--assert-trace --debug-rcaps` — zero violations of the causal trace ⊆ row law and
//!   zero uniqueness violations at iso moves;
//! - `--on-quiesce report` labels the exit.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn combined(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_actors_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const LOGGER: &str = "module app\n\
actor Logger {\n\
  var out: Cap[Console]\n\
  new(out: Cap[Console]) { self.out = out }\n\
  be log(msg: Str) ! {Write} { self.out.println(msg) }\n\
}\n\
fn main(root: Root) ! {Async, Write} {\n\
  let l = spawn Logger(root.console())\n\
  l.log(\"across the boundary\")\n\
}\n";

#[test]
fn criterion5_why_write_crosses_the_actor_boundary() {
    let dir = scratch("why");
    let file = dir.join("app.delulu");
    std::fs::write(&file, LOGGER).unwrap();
    let o = delulu_in(&dir, &["why", "Write", file.to_str().unwrap()]);
    let out = combined(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("main"), "the chain starts at main: {out}");
    assert!(out.contains("Logger.log"), "the chain crosses into the behavior: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn criterion6_native_actor_run_is_clean_under_assert_trace_and_debug_rcaps() {
    let dir = scratch("c6native");
    let file = dir.join("app.delulu");
    // Includes an iso move so --debug-rcaps has something real to verify.
    let src = "module app\n\
        actor Sink {\n\
        var total: Int\n\
        new() { self.total = 0 }\n\
        be take(vs: iso List[Int]) ! {} { self.total = self.total + vs.len() }\n\
        }\n\
        actor Logger {\n\
        var out: Cap[Console]\n\
        new(out: Cap[Console]) { self.out = out }\n\
        be log(msg: Str) ! {Write} { self.out.println(msg) }\n\
        }\n\
        fn main(root: Root) ! {Async, Write} {\n\
        let s = spawn Sink()\n\
        let xs: iso List[Int] = recover { [1, 2, 3] }\n\
        s.take(consume xs)\n\
        let l = spawn Logger(root.console())\n\
        l.log(\"done\")\n\
        }\n";
    std::fs::write(&file, src).unwrap();
    let o = delulu_in(
        &dir,
        &[
            "run",
            file.to_str().unwrap(),
            "--grant",
            "console",
            "--no-prompt",
            "--assert-trace",
            "--debug-rcaps",
            "--on-quiesce",
            "report",
        ],
    );
    let out = combined(&o);
    assert!(o.status.success(), "must exit 0 (assert-trace violations exit 3): {out}");
    assert!(!out.contains("DL1101"), "zero causal-law violations: {out}");
    assert!(!out.contains("DL1610"), "zero uniqueness violations: {out}");
    assert!(out.contains("quiesce:"), "the quiescence report prints: {out}");
    assert!(out.contains("2 surviving actor(s)"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_non_actor_program_run_is_untouched_by_stage7_flags_absence() {
    // House rule: a module with no actors takes the identical v0.6 path — smoke it.
    let dir = scratch("plain");
    let file = dir.join("plain.delulu");
    std::fs::write(
        &file,
        "module plain\nfn main(root: Root) ! {Write} { root.console().println(\"hi\") }\n",
    )
    .unwrap();
    let o = delulu_in(&dir, &["run", file.to_str().unwrap(), "--grant", "console", "--no-prompt"]);
    let out = combined(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("hi"), "{out}");
    assert!(!out.contains("quiesce"), "no actor machinery output for a plain program: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}
