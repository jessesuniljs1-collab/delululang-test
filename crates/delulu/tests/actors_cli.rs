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
    Command::new(env!("CARGO_BIN_EXE_delulu")).current_dir(cwd).args(args).output().expect("failed to run delulu")
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

/// MSG-CYCLE-1 (RW 2.16, D-V2-165): a graph that holds itself, built in `recover` and moved to an actor, arrives holding
/// itself — and under `--debug-rcaps` is not reported as a compiler bug. Before, the move walked the graph as a tree until
/// the stack overflowed and the process died; the debug walk counted the graph's own back-reference as an alias (DL1610).
#[test]
fn a_graph_that_holds_itself_moves_to_an_actor_whole() {
    let dir = scratch("cycle");
    let file = dir.join("app.delulu");
    let src = "module app\n\
        type Node {\n  v: Int,\n  next: Option[Node],\n}\n\
        actor Sink {\n\
        var out: Cap[Console]\n\
        new(out: Cap[Console]) { self.out = out }\n\
        be take(n: iso Node) ! {Write} { self.out.println(str(n)) }\n\
        }\n\
        fn main(root: Root) ! {Async, Write} {\n\
        let s = spawn Sink(root.console())\n\
        let n: iso Node = recover {\n\
        var k = Node { v: 7, next: None }\n\
        k.next = Some(k)\n\
        k\n\
        }\n\
        s.take(consume n)\n\
        }\n";
    std::fs::write(&file, src).unwrap();
    for flags in [&[][..], &["--debug-rcaps"][..]] {
        let mut args = vec!["run", file.to_str().unwrap(), "--grant", "console", "--no-prompt"];
        args.extend_from_slice(flags);
        let o = delulu_in(&dir, &args);
        let out = combined(&o);
        assert!(o.status.success(), "{flags:?}: must exit 0: {out}");
        assert!(!out.contains("DL1610"), "{flags:?}: a legal graph is not a compiler bug: {out}");
        assert!(out.contains("Node { v: 7, next: Some(Node { ... }) }"), "{flags:?}: it arrives holding itself: {out}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Every part of a moved value lands where it was: a closure's captured values each under its own name, a map's values
/// each under its own key. DATA-DEPTH-2 made both walks iterative, a part at a time through the cell that holds it; no
/// test had sent a closure capturing more than one value or a map of more than one key, so a part misplaced was unseen.
#[test]
fn a_closure_and_a_map_cross_to_an_actor_with_every_part_in_place() {
    let dir = scratch("parts");
    let file = dir.join("app.delulu");
    let src = "module app\n\
        actor Calc {\n\
        var n: Int\n\
        new() { self.n = 0 }\n\
        be apply(f: val fn(Int) -> Int, out: Cap[Console]) ! {Write} { out.println(str(f(0))) }\n\
        be keep(m: iso Map[Str, Int], out: Cap[Console]) ! {Write} { out.println(str(m)) }\n\
        }\n\
        fn main(root: Root) ! {Async, Write} {\n\
        let a = 10\n\
        let b = 3\n\
        let c = spawn Calc()\n\
        c.apply(fn(x: Int) -> Int { x + a - b * 2 }, root.console())\n\
        let m: iso Map[Str, Int] = recover {\n\
        let w: ref Map[Str, Int] = Map()\n\
        w.insert(\"one\", 1)\n\
        w.insert(\"two\", 2)\n\
        w.insert(\"three\", 3)\n\
        w\n\
        }\n\
        c.keep(consume m, root.console())\n\
        }\n";
    std::fs::write(&file, src).unwrap();
    let o = delulu_in(&dir, &["run", file.to_str().unwrap(), "--grant", "console", "--no-prompt"]);
    let out = combined(&o);
    assert!(o.status.success(), "must exit 0: {out}");
    assert_eq!(String::from_utf8_lossy(&o.stdout), "4\n{one: 1, three: 3, two: 2}\n", "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// CLOSURE-SEND-1: a closure bound by `let` crosses to an actor carrying what its body reads — through a closure inside it
/// too, each name as the innermost scope binds it — and nothing else. Every binding of its scopes crossed, itself among
/// them, so the walk never ended: a stack overflow, then (once the walk was iterative) the memory budget; and an `iso` map
/// beside it crossed with it.
#[test]
fn a_let_bound_closure_crosses_to_an_actor_carrying_only_what_it_reads() {
    let dir = scratch("letfn");
    let file = dir.join("app.delulu");
    let src = "module app\n\
        actor Calc {\n\
        var n: Int\n\
        new() { self.n = 0 }\n\
        be apply(f: val fn(Int) -> Int, out: Cap[Console]) ! {Write} { out.println(str(f(1))) }\n\
        be keep(m: iso Map[Str, Int], out: Cap[Console]) ! {Write} { out.println(str(m)) }\n\
        }\n\
        fn main(root: Root) ! {Async, Write} {\n\
        let a = 10\n\
        let k = 100\n\
        let f: val fn(Int) -> Int = if a > 0 {\n\
        let k = 5\n\
        fn(x: Int) -> Int {\n\
        let g = fn(y: Int) -> Int { y + k }\n\
        g(x) + a\n\
        }\n\
        } else {\n\
        fn(x: Int) -> Int { x }\n\
        }\n\
        let m: iso Map[Str, Int] = recover {\n\
        let w: ref Map[Str, Int] = Map()\n\
        w.insert(\"one\", 1)\n\
        w\n\
        }\n\
        let c = spawn Calc()\n\
        c.apply(f, root.console())\n\
        c.keep(consume m, root.console())\n\
        }\n";
    std::fs::write(&file, src).unwrap();
    let o = delulu_in(&dir, &["run", file.to_str().unwrap(), "--grant", "console", "--no-prompt"]);
    let out = combined(&o);
    assert!(o.status.success(), "must exit 0: {out}");
    assert_eq!(String::from_utf8_lossy(&o.stdout), "16\n{one: 1}\n", "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Two actors spawned by `main`, each spawning a child, each drawing twice per turn (`a` for two turns); and one
/// reading the clock.
const DRAWS: &str = "module app\n\
actor Roller {\n\
  let out: Cap[Console]\n\
  let r: Cap[Rand]\n\
  let name: Str\n\
  new(out: Cap[Console], r: Cap[Rand], name: Str) {\n\
    self.out = out\n\
    self.r = r\n\
    self.name = name\n\
  }\n\
  be roll(child: Bool) ! {Write, Rand, Async} {\n\
    self.out.println(self.name + \" \" + str(self.r.int(0, 1000000000)) + \" \" + str(self.r.int(0, 1000000000)))\n\
    if child {\n\
      let c = spawn Roller(self.out, self.r, self.name + \".child\")\n\
      c.roll(false)\n\
    }\n\
  }\n\
}\n\
actor Watch {\n\
  let out: Cap[Console]\n\
  let c: Cap[Clock]\n\
  new(out: Cap[Console], c: Cap[Clock]) {\n\
    self.out = out\n\
    self.c = c\n\
  }\n\
  be tell() ! {Write, Clock} { self.out.println(\"clock \" + str(self.c.now_ms())) }\n\
}\n\
fn main(root: Root) ! {Write, Rand, Clock, Async} {\n\
  let out = root.console()\n\
  let r = root.rand()\n\
  let a = spawn Roller(out, r, \"a\")\n\
  let b = spawn Roller(out, r, \"b\")\n\
  a.roll(true)\n\
  b.roll(true)\n\
  a.roll(false)\n\
  let w = spawn Watch(out, root.clock())\n\
  w.tell()\n\
}\n";

/// ACTOR-DETERMINISM-1 (2026-10-27, D-V2-192): `--seed` and `--clock fixed:` were set on the main thread only, and an
/// actor runs on a worker thread, so an actor's draws and clock reads took the worker's wall-seeded generator and the
/// wall clock: every run differed, even on one worker. Each actor now draws from a stream of its own, keyed by who
/// spawned it and when, so its draws do not depend on which worker runs it; every worker reads the fixed clock. The
/// lines are compared sorted: the order in which actors print is not promised (per-sender FIFO only, Stage 7).
#[test]
fn seed_and_fixed_clock_reach_every_actor_on_any_number_of_workers() {
    let dir = scratch("determinism");
    let file = dir.join("app.delulu");
    std::fs::write(&file, DRAWS).unwrap();
    let f = file.to_str().unwrap();
    let run = |seed: &str, threads: &str| {
        let grants = ["--grant", "console", "--grant", "rand", "--grant", "clock"];
        let mut args = vec!["run", f];
        args.extend(grants);
        args.extend(["--seed", seed, "--clock", "fixed:1000", "--actors-threads", threads]);
        let o = delulu_in(&dir, &args);
        assert!(o.status.success(), "{}", combined(&o));
        let mut lines: Vec<String> = String::from_utf8_lossy(&o.stdout).lines().map(str::to_string).collect();
        lines.sort();
        lines
    };
    let first = run("7", "1");
    assert_eq!(first.len(), 6, "{first:?}");
    assert!(first.contains(&"clock 1000".to_string()), "an actor reads the fixed clock: {first:?}");
    let a_turns: Vec<&String> = first.iter().filter(|l| l.starts_with("a ")).collect();
    assert!(a_turns.len() == 2 && a_turns[0] != a_turns[1], "an actor's second turn draws on from its first: {first:?}");
    assert_eq!(run("7", "1"), first, "the same seed again, on one worker");
    assert_eq!(run("7", "4"), first, "the same seed on four workers: each actor draws its own stream");
    let other = run("8", "4");
    assert_ne!(other, first, "another seed draws otherwise");
    assert!(other.contains(&"clock 1000".to_string()), "{other:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_non_actor_program_run_is_untouched_by_stage7_flags_absence() {
    // House rule: a module with no actors takes the identical v0.6 path — smoke it.
    let dir = scratch("plain");
    let file = dir.join("plain.delulu");
    std::fs::write(&file, "module plain\nfn main(root: Root) ! {Write} { root.console().println(\"hi\") }\n").unwrap();
    let o = delulu_in(&dir, &["run", file.to_str().unwrap(), "--grant", "console", "--no-prompt"]);
    let out = combined(&o);
    assert!(o.status.success(), "{out}");
    assert!(out.contains("hi"), "{out}");
    assert!(!out.contains("quiesce"), "no actor machinery output for a plain program: {out}");
    let _ = std::fs::remove_dir_all(&dir);
}
