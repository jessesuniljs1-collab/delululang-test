//! A record's width costs nothing per field read, field write or literal built (RW 2.10, finding C55):
//! - a literal written in another order than its type declares is put in order by each field's declared place, looked
//!   up once. Until 2026-10-23 every comparison of the sort scanned the declaration for both fields' places, so a
//!   1,600-field literal written out of order took 39 ms to build and 6,400 fields 1.1 s — CPU a plugin's fuel, one step
//!   per expression, never charged;
//! - a field of a record wider than a short scan is found at its declared place, and the name there is checked before it
//!   is used: a record whose fields are not where this module's declaration puts them is scanned, never misread.
//!   Reading or writing the last of 1,600 fields cost 4x reading the first.
//!
//! The two timing tests judge a cost against a control measured beside it in the same run (D-V2-47), best of three each,
//! never against a wall-clock bound.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};

/// A module declaring `W`, `width` Int fields `f0..`, with `extra` appended; field `fi` holds `i` in every literal.
fn module_with_wide(width: usize, extra: &str) -> String {
    let decl: String = (0..width).map(|i| format!("  f{i}: Int,\n")).collect();
    format!("module m\n\ntype W {{\n{decl}}}\n\n{extra}")
}

/// The fields of a `W` literal in declared order, or in a fixed shuffle that leaves almost none in place (7919 is prime
/// and divides no width used here, so `i * 7919 % width` is a permutation).
fn literal(width: usize, shuffled: bool) -> String {
    let at = |i: usize| if shuffled { i * 7919 % width } else { i };
    let fields: Vec<String> = (0..width).map(|i| format!("f{}: {}", at(i), at(i))).collect();
    format!("W {{ {} }}", fields.join(", "))
}

fn interp(src: &str) -> Interp {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{:?}", checked.diagnostics);
    Interp::new(&checked.module)
}

/// Run `main` on the interpreter with the console; its output.
fn run(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{:?}", checked.diagnostics);
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let r = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root())));
    let out = take_capture().unwrap_or_default();
    r.unwrap_or_else(|e| panic!("the program faulted: {e:?}"));
    out
}

/// The best of three timed runs of `f`.
fn best_of_three(mut f: impl FnMut()) -> Duration {
    (0..3)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed()
        })
        .min()
        .expect("three timings")
}

/// A `W` of `width` fields in declared order, as a literal builds one, built here so that a timing reads it without
/// building it.
fn wide_value(width: usize) -> Value {
    let fields: Vec<(String, Value)> = (0..width).map(|k| (format!("f{k}"), Value::Int(k as i64))).collect();
    Value::Record { name: Rc::from("W"), fields: Rc::new(RefCell::new(fields)) }
}

/// A 40-field record (wider than the scan) written in a shuffled order is held, printed and compared as one written in
/// declared order; a field read and a field written land on that field, directly, through a function, a list and a
/// record field.
#[test]
fn a_wide_record_is_held_read_and_written_at_its_declared_places() {
    let w = 40;
    let main = format!(
        r#"type Holder {{
  w: W,
}}

fn mid(r: W) -> Int {{
  r.f20
}}

fn main(root: Root) ! {{Write}} {{
  let out = root.console()
  var p = {shuffled}
  let q = {ordered}
  out.println(str(str(p) == str(q)) + " " + str(p == q))
  out.println(str(p.f0) + " " + str(p.f17) + " " + str(mid(q)) + " " + str(p.f39))
  p.f39 = 100
  p.f17 = 200
  out.println(str(p.f39) + " " + str(p.f17) + " " + str(p.f38) + " " + str(p == q))
  let h = Holder {{ w: {shuffled} }}
  let ws = [p]
  out.println(str(h.w.f25) + " " + str(ws[0].f17))
}}
"#,
        shuffled = literal(w, true),
        ordered = literal(w, false),
    );
    let out = run(&module_with_wide(w, &main));
    assert_eq!(out, "true true\n0 17 20 39\n100 200 38 false\n25 200\n");
}

/// A record value whose fields are not where this module's declaration of its type puts them — the declaration says
/// `f3` is the fourth field; this value holds `f36` there — is read and written by name. A place is used only when the
/// name at it is the field asked for; without that check `f3` would read 36.
#[test]
fn a_field_is_found_by_name_when_its_record_is_not_laid_out_as_declared() {
    let w = 40;
    let i = interp(&module_with_wide(
        w,
        "fn get3(r: W) -> Int {\n  r.f3\n}\n\nfn set3(r: ref W) -> Int {\n  r.f3 = 99\n  r.f3 + r.f36\n}\n",
    ));
    let reversed = || {
        let Value::Record { fields, .. } = wide_value(w) else { unreachable!() };
        fields.borrow_mut().reverse();
        Value::Record { name: Rc::from("W"), fields }
    };
    assert!(matches!(i.call_with("get3", vec![reversed()]), Ok(Value::Int(3))), "f3 is read by its name");
    assert!(matches!(i.call_with("set3", vec![reversed()]), Ok(Value::Int(135))), "f3 is written by its name, f36 kept");
}

/// Building a 2,000-field literal written out of order costs a constant times building a 2,000-item list: each field
/// clones its name and looks up its place once (17x the list here, in a debug build). The sort that put it in order looked
/// up both fields' places by scanning the declaration at every comparison, which made it quadratic in the width.
#[test]
#[cfg_attr(miri, ignore)] // the witness is the wall clock
fn building_a_wide_literal_out_of_order_costs_a_constant_times_a_list_of_its_width() {
    let w = 2000;
    let items: Vec<String> = (0..w).map(|k| k.to_string()).collect();
    let fns = format!(
        "fn shuffled(n: Int) -> Int {{\n  var i = 0\n  var s = 0\n  while i < n {{\n    let r = {lit}\n    s = s + r.f1\n    i = i + 1\n  }}\n  s\n}}\n\n\
         fn list(n: Int) -> Int {{\n  var i = 0\n  var s = 0\n  while i < n {{\n    let r = [{items}]\n    s = s + r[1]\n    i = i + 1\n  }}\n  s\n}}\n",
        lit = literal(w, true),
        items = items.join(", "),
    );
    let i = interp(&module_with_wide(w, &fns));
    assert_eq!(i.call_int_fn("shuffled", &[3]).expect("shuffled"), 3);
    let timed = |f: &str| {
        best_of_three(|| {
            i.call_int_fn(f, &[10]).unwrap_or_else(|e| panic!("`{f}` faulted: {e:?}"));
        })
    };
    let record = timed("shuffled");
    let list = timed("list");
    let ratio = record.as_secs_f64() / list.as_secs_f64();
    eprintln!("record_width: 10 shuffled {w}-field literals {record:?}, 10 {w}-item lists {list:?}, ratio {ratio:.1}");
    assert!(ratio < 100.0, "a {w}-field literal out of order cost {ratio:.1}x a {w}-item list ({record:?} vs {list:?})");
}

/// Reading the last field of an 8,000-field record costs about what reading the first does. The record is built once,
/// outside the timing, so only the reads are timed; a scan made the last cost about 20x the first here.
#[test]
#[cfg_attr(miri, ignore)] // the witness is the wall clock
fn reading_the_last_field_of_a_wide_record_costs_about_what_the_first_does() {
    let w = 8000;
    let read = |f: &str, field: usize| {
        format!(
            "fn {f}(r: W, n: Int) -> Int {{
  var i = 0
  var s = 0
  while i < n {{
    s = s + r.f{field}
    i = i + 1
  }}
  s
}}

"
        )
    };
    let i = interp(&module_with_wide(w, &(read("first", 0) + &read("last", w - 1))));
    let r = wide_value(w);
    let reads = |f: &str, n: i64| match i.call_with(f, vec![r.clone(), Value::Int(n)]) {
        Ok(Value::Int(s)) => s,
        other => panic!("`{f}` answered {other:?}"),
    };
    assert_eq!(reads("last", 2), 2 * (w as i64 - 1));
    let n = 5_000;
    let last = best_of_three(|| {
        reads("last", n);
    });
    let first = best_of_three(|| {
        reads("first", n);
    });
    let ratio = last.as_secs_f64() / first.as_secs_f64();
    eprintln!("record_width: {n} reads of f{} {last:?}, of f0 {first:?}, ratio {ratio:.2}", w - 1);
    assert!(ratio < 3.0, "reading the last of {w} fields cost {ratio:.2}x reading the first ({last:?} vs {first:?})");
}
