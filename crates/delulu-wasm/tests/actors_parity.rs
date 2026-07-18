//! Stage 7 phase 7h — the WASM engine's cooperative single-threaded actor scheduler (spec §6.5).
//!
//! Two-engine parity for actors, in keeping with the whole backend's subset discipline: an
//! Int-only actor program (fields + params are `Int` / actor references) runs on BOTH engines and
//! must reach quiescence with **identical observable results** — the same turn count, the same
//! surviving/dead actor counts, the same dropped-send count (criterion 6, spec §9). The
//! interpreter + native `ActorSystem` (at `--actors-threads 1`) is the reference engine; the WASM
//! cooperative scheduler must match it. The semantics never promised timing, only the messages.
//!
//! Also proves the subset boundary stays honest: an actor construct outside the Int/actor-ref
//! fragment does not miscompile — it refuses as DL1201 so the program runs on the interpreter.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::actors::{ActorSystem, QuiesceReport};
use delulu_runtime::{Interp, RootVal, Value};
use delulu_wasm::{actor_table, compile_module, run_main_actors, ActorReport, HostConfig};

/// Run an actor `main` on the reference engine (interpreter + native `ActorSystem`, one worker —
/// the deterministic baseline) and return its quiescence report.
fn native_report(src: &str) -> QuiesceReport {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "witness must check clean: {:?}", checked.diagnostics);
    let system = ActorSystem::start(&checked.module, 1, false);
    let interp = Interp::new(&checked.module).with_actors(system.host());
    interp.run_main(Value::Root(Rc::new(RootVal::default()))).expect("main runs clean");
    system.finish()
}

/// Run the same actor `main` on the WASM engine's cooperative scheduler and return
/// `(console output, quiescence report)`.
fn wasm_report(src: &str) -> (String, ActorReport) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "witness must check clean: {:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).expect("the actor witness compiles to WASM");
    let table = actor_table(&checked.module);
    run_main_actors(&wasm, &HostConfig::default(), &table).expect("wasm actor run reaches quiescence")
}

/// Assert both engines reach quiescence with identical accounting. Returns the WASM console output.
fn assert_actor_parity(src: &str) -> String {
    let n = native_report(src);
    let (out, w) = wasm_report(src);
    assert_eq!(w.total_turns, n.total_turns, "turn count differs (wasm {} vs native {})", w.total_turns, n.total_turns);
    assert_eq!(w.surviving_actors, n.surviving_actors, "surviving-actor count differs");
    assert_eq!(w.dead_actors, n.dead_actors, "dead-actor count differs");
    assert_eq!(w.dropped_sends, n.dropped_sends, "dropped-send count differs");
    out
}

/// A self-sending counter: `spawn` + one send, then N self-sends decrementing to 0. Exercises Int
/// fields, self-field read/write, a guarded self-send, and quiescence.
fn counter_src(start: u64) -> String {
    format!(
        "module cw\n\
         actor Counter {{\n\
         var n: Int\n\
         new() {{ self.n = 0 }}\n\
         be bump(remaining: Int) ! {{Async}} {{\n\
         self.n = self.n + 1\n\
         if remaining > 0 {{ self.bump(remaining - 1) }}\n\
         }}\n\
         }}\n\
         fn main(root: Root) ! {{Async}} {{\n\
         let c = spawn Counter()\n\
         c.bump({start})\n\
         }}\n"
    )
}

/// A single ping-pong pair: two actors trading a bounded number of messages, each send carrying
/// the sender's own reference (`from.pong(self)`). Exercises `spawn` with an arg, actor-reference
/// params, and passing `self` as a message argument.
fn pingpong_src(rounds: u64) -> String {
    format!(
        "module pp\n\
         actor Pong {{\n\
         var served: Int\n\
         new() {{ self.served = 0 }}\n\
         be pong(from: Ping) ! {{Async}} {{\n\
         self.served = self.served + 1\n\
         from.ping(self)\n\
         }}\n\
         }}\n\
         actor Ping {{\n\
         var left: Int\n\
         new(n: Int) {{ self.left = n }}\n\
         be ping(from: Pong) ! {{Async}} {{\n\
         if self.left > 0 {{\n\
         self.left = self.left - 1\n\
         from.pong(self)\n\
         }}\n\
         }}\n\
         }}\n\
         fn main(root: Root) ! {{Async}} {{\n\
         let pg = spawn Pong()\n\
         let pi = spawn Ping({rounds})\n\
         pi.ping(pg)\n\
         }}\n"
    )
}

#[test]
fn criterion6_counter_selfsend_parity() {
    // 1 create + (start+1) bumps (the last finds remaining == 0 and goes quiet — quiescence, not
    // a sentinel). For start = 9 that is 11 turns; the two engines agree independently.
    let out = assert_actor_parity(&counter_src(9));
    assert!(out.is_empty(), "an Int-only actor prints nothing");
    let (_, w) = wasm_report(&counter_src(9));
    assert_eq!(w.total_turns, 11, "1 create + 10 bumps");
    assert_eq!(w.surviving_actors, 1);
    assert_eq!(w.dead_actors, 0);
    assert_eq!(w.dropped_sends, 0);
}

#[test]
fn criterion6_counter_selfsend_parity_deep() {
    // A deeper self-send chain — the cooperative drain must run every queued turn to quiescence.
    let src = counter_src(1000);
    assert_actor_parity(&src);
    let (_, w) = wasm_report(&src);
    assert_eq!(w.total_turns, 1002); // 1 create + 1001 bumps
    assert_eq!(w.surviving_actors, 1);
}

#[test]
fn criterion6_pingpong_pair_parity() {
    // 2 creates + (R+1) pings + R pongs. For R = 5 that is 13 turns; both engines agree.
    assert_actor_parity(&pingpong_src(5));
    let (_, w) = wasm_report(&pingpong_src(5));
    assert_eq!(w.total_turns, 13);
    assert_eq!(w.surviving_actors, 2);
    assert_eq!(w.dead_actors, 0);
    assert_eq!(w.dropped_sends, 0);
}

#[test]
fn criterion6_pingpong_pair_parity_deep() {
    let src = pingpong_src(2000);
    assert_actor_parity(&src);
    let (_, w) = wasm_report(&src);
    assert_eq!(w.total_turns, 2 + (2000 + 1) + 2000); // 2 creates + (R+1) pings + R pongs
    assert_eq!(w.surviving_actors, 2);
}

#[test]
fn a_faulting_behavior_poisons_its_actor_on_both_engines() {
    // spec §6.6: a fault in a turn poisons its actor; the system stays live and later sends to it
    // drop and are counted. A divide-by-zero traps on the WASM engine exactly where the
    // interpreter's checked division faults — so the poison accounting matches.
    let src = "module bomb\n\
        actor Bomb {\n\
        var n: Int\n\
        new() { self.n = 0 }\n\
        be boom(d: Int) ! {Async} { self.n = 100 / d }\n\
        be tick() ! {Async} { self.n = self.n + 1 }\n\
        }\n\
        fn main(root: Root) ! {Async} {\n\
        let b = spawn Bomb()\n\
        b.boom(0)\n\
        b.tick()\n\
        b.tick()\n\
        }\n";
    assert_actor_parity(src);
    let (_, w) = wasm_report(src);
    assert_eq!(w.dead_actors, 1, "the bomb died");
    assert_eq!(w.dropped_sends, 2, "the two later ticks dropped after poisoning");
    assert_eq!(w.surviving_actors, 0);
    assert_eq!(w.total_turns, 2, "1 create + the faulting boom (the dropped ticks are not turns)");
}

// ----- the subset boundary stays honest (DL1201 outside Int/actor-ref) ----------------------

#[test]
fn a_str_field_actor_refuses_as_dl1201() {
    // A `Str` actor field is outside the Int/actor-ref subset: the module must not compile to WASM
    // (so it runs on the interpreter), and the refusal is the interpreter-fallback code DL1201.
    let src = "module s\n\
        actor Named {\n\
        var label: Str\n\
        new(l: Str) { self.label = l }\n\
        be rename(l: Str) ! {Async} { self.label = l }\n\
        }\n\
        fn main(root: Root) ! {Async} {\n\
        let a = spawn Named(\"hi\")\n\
        a.rename(\"bye\")\n\
        }\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    match compile_module(&checked.module) {
        Err(e) => assert_eq!(e.code(), "DL1201", "an out-of-subset actor field must be DL1201, got {}", e.message()),
        Ok(_) => panic!("a Str-field actor must not compile to WASM (it belongs on the interpreter)"),
    }
}

#[test]
fn a_list_param_behavior_refuses_as_dl1201() {
    // A behaviour parameter outside the subset (`List[Int]`) likewise refuses honestly.
    let src = "module l\n\
        actor Sink {\n\
        var n: Int\n\
        new() { self.n = 0 }\n\
        be take(xs: List[Int]) ! {Async} { self.n = self.n + 1 }\n\
        }\n\
        fn main(root: Root) ! {Async} {\n\
        let s = spawn Sink()\n\
        s.take([1, 2, 3])\n\
        }\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    assert!(
        matches!(compile_module(&checked.module), Err(ref e) if e.code() == "DL1201"),
        "an out-of-subset behaviour parameter must refuse as DL1201"
    );
}
