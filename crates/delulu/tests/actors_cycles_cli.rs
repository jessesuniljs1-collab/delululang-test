//! Stage 10 phase 10d — the cycle collector, witnessed from the language (Track B1, spec §3).
//!
//! The Stage-1/7 leak note said: cyclic `Rc` graphs leak, documented. This corpus is that note
//! going from "documented" to "collected": a turn that manufactures cyclic garbage past the
//! sweep threshold must show sweeps > 0 and cells > 0 in `--trace-memory` — and the program's
//! results must be untouched, because a collector that changes behavior is a bug with a longer
//! name. The collector's own unit tests (`cycles.rs`) prove actual freeing via `Weak` handles;
//! this file proves the language reaches it.

use std::path::PathBuf;
use std::process::{Command, Output};

fn delulu(args: &[&str], dir: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-cycles-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The leak corpus program: each loop iteration builds `l → Link(l) → l`, a genuine in-language
/// reference cycle, then drops it. 200 of them cross the sweep threshold with room to spare.
const CHURN: &str = "module m\n\ntype Chain = Link(List[Chain]) | Nil\n\nactor Churner {\n  var n: Int\n  new() { self.n = 0 }\n  be churn() {\n    var i = 0\n    while i < 200 {\n      let l: ref List[Chain] = []\n      l.push(Link(l))\n      i = i + 1\n    }\n    self.n = self.n + 200\n  }\n  be report(out: Cap[Console]) ! {Write} { out.println(str(self.n)) }\n}\n\nfn main(root: Root) ! {Async, Write} {\n  let c = spawn Churner()\n  c.churn()\n  c.report(root.console())\n}\n";

/// THE B1 CRITERION: cyclic garbage manufactured inside a turn is COLLECTED between turns —
/// the sweep ran, cells were broken, and the program's observable behavior is untouched.
#[test]
fn a_turn_full_of_cyclic_garbage_is_collected_between_turns() {
    let dir = scratch("churn");
    std::fs::write(dir.join("churn.delulu"), CHURN).unwrap();
    let o = delulu(&["run", "churn.delulu", "--grant", "console", "--trace-memory"], &dir);
    assert!(o.status.success(), "stderr: {}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        "200",
        "collection never changes what the program computes"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    let line = err
        .lines()
        .find(|l| l.contains("cycle collector:"))
        .unwrap_or_else(|| panic!("--trace-memory reports the collector: {err}"));
    let sweeps: u64 = line
        .split("cycle collector: ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let cells: u64 = line
        .split(", ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    assert!(sweeps >= 1, "the sweep ran under pressure: {line}");
    assert!(
        cells >= 200,
        "every manufactured cycle was broken (200 made, {cells} collected): {line}"
    );
}

/// The safety half at the language level: state-reachable cyclic structure SURVIVES collection
/// — the actor stores its cycle in a field, churns enough garbage to force a sweep, and the
/// stored structure still answers afterwards.
#[test]
fn a_state_reachable_cycle_survives_the_sweep() {
    let dir = scratch("live");
    let src = "module m\n\ntype Chain = Link(List[Chain]) | Nil\n\nactor Keeper {\n  var kept: ref List[Chain]\n  new() {\n    let l: ref List[Chain] = []\n    l.push(Link(l))\n    self.kept = l\n  }\n  be churn() {\n    var i = 0\n    while i < 200 {\n      let g: ref List[Chain] = []\n      g.push(Link(g))\n      i = i + 1\n    }\n  }\n  be report(out: Cap[Console]) ! {Write} { out.println(str(self.kept.len())) }\n}\n\nfn main(root: Root) ! {Async, Write} {\n  let k = spawn Keeper()\n  k.churn()\n  k.report(root.console())\n}\n";
    std::fs::write(dir.join("live.delulu"), src).unwrap();
    let o = delulu(&["run", "live.delulu", "--grant", "console", "--trace-memory"], &dir);
    assert!(o.status.success(), "stderr: {}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        "1",
        "the state-reachable cycle is live data — the sweep may not touch it"
    );
}

/// Non-actor programs never register, never sweep: the collector line reads zero even under
/// `--trace-memory`... and for a program with no actors the actor system never starts, so the
/// honest assertion is subtractive — no collector output at all, and identical behavior.
#[test]
fn a_non_actor_program_is_untouched_by_the_collector() {
    let dir = scratch("plain");
    let src = "module m\n\nfn main(root: Root) ! {Write} {\n  let c = root.console()\n  var i = 0\n  var acc = 0\n  while i < 1000 {\n    let l = [i]\n    acc = acc + l.len()\n    i = i + 1\n  }\n  c.println(str(acc))\n}\n";
    std::fs::write(dir.join("plain.delulu"), src).unwrap();
    let o = delulu(&["run", "plain.delulu", "--grant", "console"], &dir);
    assert!(o.status.success());
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "1000");
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("cycle collector"),
        "no actors → no collector surface at all"
    );
}
