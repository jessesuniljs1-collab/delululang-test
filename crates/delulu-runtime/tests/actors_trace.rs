//! Stage 7 phase 7i â€” causal trace attribution (spec Â§6.3) and the `--debug-rcaps`
//! uniqueness detector (spec Â§6.4). The causal law (invariant 35, executable): every traced
//! effect belongs to the EXECUTING member's static row AND the send site's static row via
//! the cause chain.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use delulu_runtime::actors::{assert_unique_graph, ActorSystem};
use delulu_runtime::{assert_trace_causal, Cause, Interp, TraceRecord, Value};

const SRC: &str = "module t\n\
    actor Logger {\n\
    var out: Cap[Console]\n\
    new(out: Cap[Console]) { self.out = out }\n\
    be log(msg: Str) ! {Write} { self.out.println(msg) }\n\
    }\n\
    fn main(root: Root) ! {Async, Write} {\n\
    let l = spawn Logger(root.console())\n\
    l.log(\"hello\")\n\
    l.log(\"world\")\n\
    }\n";

#[test]
fn traced_effects_carry_actor_member_turn_and_cause() {
    let checked = delulu_check::check_source(0, SRC);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);

    let collector: Arc<Mutex<Vec<TraceRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let system = ActorSystem::start_with(&checked.module, 2, false, Some(collector.clone()), None, None, false);
    let interp = Interp::new(&checked.module).with_actors(system.host());
    let root = Value::Root(std::rc::Rc::new(delulu_runtime::RootVal {
        console: true,
        ..Default::default()
    }));
    delulu_runtime::set_capture(true); // keep test output clean; println captures
    interp.run_main(root).expect("main runs clean");
    let report = system.finish();
    delulu_runtime::set_capture(false);
    assert_eq!(report.dead_actors, 0);

    let records = collector.lock().unwrap().clone();
    let writes: Vec<&TraceRecord> = records.iter().filter(|r| r.effect == "Write").collect();
    assert_eq!(writes.len(), 2, "two println turns traced: {records:?}");
    for r in &writes {
        assert!(r.actor.as_deref().is_some_and(|a| a.starts_with("Logger#")), "{r:?}");
        assert_eq!(r.member.as_deref(), Some("Logger.log"), "{r:?}");
        assert!(r.turn.is_some(), "turn id stamped: {r:?}");
        let c = r.cause.as_ref().expect("cause stamped");
        assert_eq!(c.sender, "main#0", "sent from the main line: {c:?}");
        assert_eq!(c.sender_member, None);
        assert!(c.send_span.is_some(), "the send site's span travels: {c:?}");
    }

    // The causal law holds on the real witness.
    let main_row: BTreeSet<String> = ["Async", "Write"].iter().map(|s| s.to_string()).collect();
    let member_rows: HashMap<String, BTreeSet<String>> = checked
        .result
        .facts
        .iter()
        .filter(|(k, _)| k.contains('.'))
        .map(|(k, f)| (k.clone(), f.effects.iter().map(|e| e.name().to_string()).collect()))
        .collect();
    assert!(member_rows["Logger.log"].contains("Write"));
    let violations = assert_trace_causal(&main_row, &member_rows, &records);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn the_causal_law_flags_an_effect_outside_the_executing_members_row() {
    // Synthetic (a checked program can never produce this â€” that is the point: this is the
    // compiler-bug detector's own witness). The behavior's static row lacks Net.
    let main_row: BTreeSet<String> = ["Async", "Net"].iter().map(|s| s.to_string()).collect();
    let mut member_rows: HashMap<String, BTreeSet<String>> = HashMap::new();
    member_rows.insert("A.quiet".into(), ["Write"].iter().map(|s| s.to_string()).collect());
    let record = TraceRecord {
        seq: 0,
        effect: "Net".into(),
        op: "get".into(),
        cap_kind: "Http".into(),
        actor: Some("A#1".into()),
        member: Some("A.quiet".into()),
        turn: Some(3),
        cause: Some(Cause { sender: "main#0".into(), sender_member: None, send_span: None }),
        ..Default::default()
    };
    let violations = assert_trace_causal(&main_row, &member_rows, &[record]);
    assert!(
        violations.iter().any(|v| v.contains("executing member")),
        "the executing-member half fires: {violations:?}"
    );
}

#[test]
fn the_causal_law_flags_a_send_site_that_could_not_have_carried_the_effect() {
    // The effect IS in the executing member's row, but the SEND SITE's row lacks it â€” the
    // T-Send containment replayed on the witness.
    let main_row: BTreeSet<String> = ["Async"].iter().map(|s| s.to_string()).collect();
    let mut member_rows: HashMap<String, BTreeSet<String>> = HashMap::new();
    member_rows.insert("A.log".into(), ["Write"].iter().map(|s| s.to_string()).collect());
    let record = TraceRecord {
        seq: 0,
        effect: "Write".into(),
        op: "println".into(),
        cap_kind: "Console".into(),
        actor: Some("A#1".into()),
        member: Some("A.log".into()),
        turn: Some(1),
        cause: Some(Cause { sender: "main#0".into(), sender_member: None, send_span: None }),
        ..Default::default()
    };
    let violations = assert_trace_causal(&main_row, &member_rows, &[record]);
    assert!(
        violations.iter().any(|v| v.contains("SEND SITE")),
        "the send-site half fires: {violations:?}"
    );
}

// ----- --debug-rcaps: the Â§7.4 uniqueness detector --------------------------------------

#[test]
fn an_unaliased_iso_graph_passes_the_uniqueness_walk() {
    let inner = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![Value::Int(1)])));
    let outer = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![inner])));
    assert!(assert_unique_graph(&outer).is_ok());
}

#[test]
fn an_aliased_nested_node_is_a_dl1610_class_violation() {
    // Two paths to the same mutable list inside a "unique" graph â€” exactly what the static
    // proof forbids; the detector must see it.
    let shared = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![Value::Int(1)])));
    let outer = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![
        shared.clone(),
        shared,
    ])));
    let err = assert_unique_graph(&outer).expect_err("aliasing must be detected");
    assert!(err.contains("strong refs"), "{err}");
}

#[test]
fn a_checked_iso_send_under_debug_rcaps_runs_clean() {
    let src = "module d\n\
        actor Sink { var n: Int\nnew() { self.n = 0 }\n\
        be take(vs: iso List[Int]) ! {} { self.n = vs.len() } }\n\
        fn main(root: Root) ! {Async} {\n\
        let s = spawn Sink()\n\
        let xs: iso List[Int] = recover { [1, 2, 3] }\n\
        s.take(consume xs)\n\
        }\n";
    let checked = delulu_check::check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    assert!(!checked.result.iso_moves.is_empty(), "the checker exported the iso move");
    let debug = Arc::new(checked.result.iso_moves.clone());
    let system = ActorSystem::start_with(&checked.module, 2, false, None, Some(debug.clone()), None, false);
    let interp = Interp::new(&checked.module).with_actors(system.host()).with_debug_rcaps(debug);
    let root = Value::Root(std::rc::Rc::new(delulu_runtime::RootVal::default()));
    interp.run_main(root).expect("a proven-unique move passes the debug lane");
    let report = system.finish();
    assert_eq!(report.dead_actors, 0);
}
