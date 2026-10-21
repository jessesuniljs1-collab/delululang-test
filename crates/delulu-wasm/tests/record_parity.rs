//! RW 3.4 — records on the WASM engine, held to the interpreter: a literal's fields computed in the order written, a
//! field read, a field written (`p.x = v`, `l.a.x = v`) with the value computed before the place, the cell shared as the
//! interpreter's is, records nested, in lists, in options and in function signatures; a field that holds an `Option`, a
//! `Result` or a `List` (a linked list, a tree); a constructor with no written type. What stays outside — `str` and `==`
//! of a whole record, a record with a field the backend cannot hold — is refused, never miscompiled.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, set_fixed_clock_ms, set_rand_seed, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// Run `main` on both engines with the console and return `(wasm, interp)`: the output, or the fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    run_both_with(src, HostConfig { console: true, ..HostConfig::default() })
}

/// The same under `cfg`'s grants, its fixed clock and its seed, given to both engines.
fn run_both_with(src: &str, cfg: HostConfig) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &cfg).map_err(|e| e.message());
    set_fixed_clock_ms(cfg.fixed_clock_ms);
    if let Some(s) = cfg.rand_seed {
        set_rand_seed(s);
    }
    set_capture(true);
    let g = Grants { console: cfg.console, clock: cfg.clock, rand: cfg.rand, ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    set_fixed_clock_ms(None);
    (w, i.map(|_| out))
}

fn assert_parity(src: &str) -> String {
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    w.unwrap_or_else(|e| panic!("both engines failed: {e}\n{src}"))
}

/// Fields of every scalar kind; a literal written out of declaration order; a record passed (`box`), returned (`ref`) and
/// shared (`var q = p` names one cell, so `q.n = 70` is read through `p`); nested records written through two levels
/// (`l.a.x = 9.5`, seen through `p`); records in a list, walked, indexed and read with `get`.
#[test]
fn records_match_the_interpreter() {
    let src = r#"module m

type Color = Red | Green

type P {
  x: Float,
  n: Int,
  name: Str,
  on: Bool,
  c: Color,
}

type Line {
  a: ref P,
  b: ref P,
}

fn shifted(p: box P, by: Float) -> ref P {
  P { name: p.name + "'", x: p.x + by, n: p.n + 1, on: !p.on, c: p.c }
}

fn color(c: Color) -> Str {
  match c {
    Red => "red",
    Green => "green",
  }
}

fn show(p: box P) -> Str {
  p.name + "(" + str(p.x) + "," + str(p.n) + "," + str(p.on) + "," + color(p.c) + ")"
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var p = P { x: 1.5, n: 7, name: "p", on: true, c: Green }
  var q = p
  q.n = 70
  q.on = false
  out.println(show(p) + " " + show(shifted(p, 0.25)))
  var l = Line { a: p, b: shifted(p, 1.0) }
  l.a.x = 9.5
  l.b.name = "bee"
  out.println(show(l.a) + " " + show(l.b) + " " + str(p.x))
  var ps: ref List[ref P] = [p, shifted(p, 2.0)]
  var total = 0
  for r in ps { total = total + r.n }
  out.println(str(total) + " " + ps[1].name + " " + str(ps.len()))
  let found = match ps.get(1) {
    Some(r) => r.name + str(r.n),
    None => "none",
  }
  out.println(found)
}
"#;
    assert_eq!(
        assert_parity(src),
        "p(1.5,70,false,green) p'(1.75,71,true,green)
p(9.5,70,false,green) bee(2.5,71,true,green) 9.5
141 p' 2
p'71
"
    );
}

/// A literal's fields are computed in the order written, as the interpreter computes them — made visible by which fault
/// comes first: `y` before `x`, so the index fault and not the division.
#[test]
fn a_literal_computes_its_fields_in_the_order_written() {
    let literal = "module m
type P {
  x: Int,
  y: Int,
}
fn main(root: Root) ! {Write} {
  let out = root.console()
                     let xs = [1]
  let z = 0
  out.println(\"before\")
  let p = P { y: xs[5], x: 10 / z }
                     out.println(str(p.x))
}
";
    let (w, i) = run_both(literal);
    let (w, i) = (w.expect_err("WASM faults"), i.expect_err("the interpreter faults"));
    assert!(
        i.contains("DL0903") && w.contains("DL0903"),
        "the index first, on both:
{i}
{w}"
    );
}

/// A field may hold an `Option`, a `Result` or a `List`: a linked list built by prepending, walked to its end and cut
/// (`head.next = None`, RCAP-CTOR-1's shape); a tree whose children are a list field, grown through the field
/// (`t.kids.push(…)`), summed recursively over a `Result` field; an `Option[Str]` field written both ways. Each cell is
/// shared as the interpreter's is: the list pushed through `t.kids` is the one `t` holds.
#[test]
fn records_holding_options_results_and_lists_match_the_interpreter() {
    let src = r#"module m

type Node {
  v: Int,
  next: Option[ref Node],
}

type Tree {
  label: Str,
  kids: ref List[ref Tree],
  weight: Result[Int, Str],
  nick: Option[Str],
}

fn total(t: box Tree) -> Int {
  var n = match t.weight {
    Ok(w) => w,
    Err(e) => 0 - e.len(),
  }
  for k in t.kids { n = n + total(k) }
  n
}

fn name(t: box Tree) -> Str {
  match t.nick {
    Some(s) => s,
    None => t.label,
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var head = Node { v: 1, next: None }
  var i = 2
  while i <= 4 {
    head = Node { v: i, next: Some(head) }
    i = i + 1
  }
  var sum = 0
  var cur = Some(head)
  var walking = true
  while walking {
    match cur {
      Some(n) => {
        sum = sum * 10 + n.v
        cur = n.next
      }
      None => { walking = false }
    }
  }
  out.println(str(sum))
  head.next = None
  out.println(match head.next { Some(n) => "more", None => "cut" })
  let leaf = Tree { label: "a", kids: [], weight: Ok(5), nick: None }
  var t = Tree { label: "root", kids: [leaf], weight: Err("bad"), nick: Some("r") }
  t.kids.push(Tree { label: "b", kids: [leaf], weight: Ok(7), nick: None })
  out.println(str(total(t)) + " " + str(t.kids.len()) + " " + t.kids[1].label + " " + name(t) + " " + name(leaf))
  t.nick = None
  leaf.nick = Some("leaf")
  out.println(name(t) + " " + name(t.kids[0]) + " " + name(t.kids[1].kids[0]))
}
"#;
    assert_eq!(
        assert_parity(src),
        "4321
cut
14 2 b r a
root leaf leaf
"
    );
}

/// A constructor where no type is written: a sum's own (`Red`, `Circle(1.5)`), `Some(x)` of its payload's type, and
/// `None`, `Ok`, `Err` of the type the checker inferred from their later use.
#[test]
fn constructors_with_no_written_type_match_the_interpreter() {
    let src = r#"module m

type Color = Red | Green

type Shape = Circle(Float) | Dot

fn area(s: Shape) -> Float {
  match s {
    Circle(r) => r * r * 3.0,
    Dot => 0.0,
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let c = Red
  let s = Circle(1.5)
  let d = Dot
  out.println(str(area(s)) + " " + str(area(d)) + " " + match c { Red => "red", Green => "green" })
  var o = None
  out.println(match o { Some(v) => str(v), None => "none" })
  o = Some(2)
  let w = Some("w")
  let r = Ok(1)
  let e = Err("no")
  let z: Result[Int, Str] = r
  let y: Result[Int, Str] = e
  out.println(match o { Some(v) => str(v), None => "none" } + " " + match w { Some(x) => x, None => "-" })
  out.println(match z { Ok(v) => str(v), Err(m) => m } + " " + match y { Ok(v) => str(v), Err(m) => m })
  let q: Option[Color] = None
  let k: Result[Color, Str] = Ok(Green)
  out.println(match q { Some(x) => "some", None => "none" } + " " + match k { Ok(Green) => "green", _ => "other" })
}
"#;
    assert_eq!(assert_parity(src), "6.75 0.0 red\nnone\n2 w\n1 no\nnone green\n");
}

/// WASM-SCAN-1: a capability used only inside a record literal — `Stamp { at: root.clock().now_ms() }` — or a field's
/// receiver had no import: the scan that decides a module's imports passed over both, and the module the backend wrote
/// called an import it did not have ("invalid WASM module" on master). The clock is fixed and the seed given, so both
/// engines print the same.
#[test]
fn a_capability_used_inside_a_record_literal_brings_its_import() {
    let src = r#"module m

type Stamp {
  at: Int,
  roll: Int,
}

fn stamp(at: Int, roll: Int) -> ref Stamp {
  Stamp { at: at, roll: roll }
}

fn main(root: Root) ! {Write, Clock, Rand} {
  let s = Stamp { at: root.clock().now_ms(), roll: root.rand().int(1, 7) }
  let r = stamp(0, root.rand().int(10, 20)).roll
  root.console().println(str(s.at) + " " + str(s.roll) + " " + str(r))
}
"#;
    let cfg = HostConfig {
        console: true,
        clock: true,
        rand: true,
        fixed_clock_ms: Some(1_700_000_000_000),
        rand_seed: Some(7),
        ..HostConfig::default()
    };
    let (w, i) = run_both_with(src, cfg);
    assert_eq!(w, i, "two-engine parity broken");
    assert!(w.as_deref().is_ok_and(|o| o.starts_with("1700000000000 ")), "the fixed clock, read in the literal: {w:?}");
}

/// Outside the fragment, and refused: a record whose field holds a type the backend does not (a function, a capability).
/// (`str` and `==` of a whole record compile since the host's value interface, `value_parity.rs`; a field holding an
/// `Option` of a list or a map since a slot holds a pointer to any of them, `nested_parity.rs`.)
#[test]
fn what_the_record_fragment_leaves_out_is_refused() {
    for src in [
        "module m\ntype N {\n  v: Int,\n  h: fn(Int) -> Int,\n}\nfn f() -> Int {\n  let n = N { v: 1, h: fn(x: Int) -> Int { x } }\n  n.v\n}\n",
        "module m\ntype N {\n  v: Int,\n  c: Cap[Console],\n}\nfn f(c: Cap[Console]) -> Int {\n  let n = N { v: 1, c: c }\n  n.v\n}\n",
    ] {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        assert!(compile_checked(&checked.module, &checked.result).is_err(), "stays on the interpreter:\n{src}");
    }
}
