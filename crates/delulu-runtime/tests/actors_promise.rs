//! Stage 7 phase 7j — acceptance criterion 9: `std.actors.Promise[T]` end-to-end. A library
//! ACTOR, not a language feature (spec §5/§8): first fulfill wins, later fulfills drop;
//! callbacks registered before fulfilment run at fulfilment, callbacks registered after run
//! immediately — all as turns of the Promise actor, witnessed through the causal trace.

use std::sync::{Arc, Mutex};

use delulu_runtime::actors::ActorSystem;
use delulu_runtime::{Interp, TraceRecord, Value};

const SRC: &str = "module p\n\
fn main(root: Root) ! {Async, Write} {\n\
  let out = root.console()\n\
  let pr = spawn Promise()\n\
  pr.then(fn(v: val Str) ! {Write} { out.println(v) })\n\
  pr.then(fn(v: val Str) ! {Write} { out.println(v) })\n\
  pr.fulfill(\"first\")\n\
  pr.fulfill(\"second\")\n\
  pr.then(fn(v: val Str) ! {Write} { out.println(v) })\n\
}\n";

#[test]
fn criterion9_promise_first_fulfill_wins_and_callbacks_run_as_promise_turns() {
    let checked = delulu_check::check_source(0, SRC);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

    let collector: Arc<Mutex<Vec<TraceRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let system = ActorSystem::start_with(&checked.module, 2, false, Some(collector.clone()), None);
    let interp = Interp::new(&checked.module).with_actors(system.host());
    delulu_runtime::set_capture(true);
    let root = Value::Root(std::rc::Rc::new(delulu_runtime::RootVal {
        console: true,
        ..Default::default()
    }));
    interp.run_main(root).expect("main runs clean");
    let report = system.finish();
    delulu_runtime::set_capture(false);
    assert_eq!(report.dead_actors, 0, "the promise survives");

    let records = collector.lock().unwrap().clone();
    let writes: Vec<&TraceRecord> = records.iter().filter(|r| r.effect == "Write").collect();
    // Two queued callbacks fire at fulfilment + one late registration fires immediately.
    assert_eq!(writes.len(), 3, "three callback runs: {records:?}");
    for w in &writes {
        // First fulfill wins: every callback observed "first", never "second".
        assert_eq!(w.detail.as_deref(), Some("first"), "{w:?}");
        // Callbacks run as turns of the Promise actor (spec §8), inside its members.
        assert!(w.actor.as_deref().is_some_and(|a| a.starts_with("Promise#")), "{w:?}");
        assert!(
            matches!(w.member.as_deref(), Some("Promise.fulfill") | Some("Promise.then")),
            "{w:?}"
        );
    }
    // The queued pair ran in fulfill's turn; the late one in then's.
    let in_fulfill = writes.iter().filter(|w| w.member.as_deref() == Some("Promise.fulfill")).count();
    let in_then = writes.iter().filter(|w| w.member.as_deref() == Some("Promise.then")).count();
    assert_eq!((in_fulfill, in_then), (2, 1), "{records:?}");
}

#[test]
fn on_actor_death_abort_reports_aborted() {
    // spec §6.6: `--on-actor-death abort` opts into whole-program abort — the report says so.
    let src = "module f\n\
        actor Bomb { var n: Int\nnew() { self.n = 0 }\n\
        be boom(xs: List[Int]) ! {Async} { let v = xs[9] } }\n\
        fn main(root: Root) ! {Async} {\n\
        let b = spawn Bomb()\n\
        b.boom([1])\n\
        }\n";
    let checked = delulu_check::check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    let system = ActorSystem::start(&checked.module, 2, true);
    let interp = Interp::new(&checked.module).with_actors(system.host());
    let root = Value::Root(std::rc::Rc::new(delulu_runtime::RootVal::default()));
    interp.run_main(root).expect("main itself is clean");
    let report = system.finish();
    assert_eq!(report.dead_actors, 1);
    assert!(report.aborted, "abort_on_death surfaces in the report");
}
