//! RW 3.4 — records on the WASM engine, held to the interpreter: a literal's fields computed in the order written, a
//! field read, a field written (`p.x = v`, `l.a.x = v`) with the value computed before the place, the cell shared as the
//! interpreter's is, records nested, in lists, in options and in function signatures. What stays outside —
//! `str` and `==` of a whole record, a record with a field the backend cannot hold — is refused, never miscompiled.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// Run `main` on both engines with the console and return `(wasm, interp)`: the output, or the fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
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

/// Outside the fragment, and refused: `str` and `==` of a whole record, and a record whose field holds a type the
/// backend does not (an `Option`).
#[test]
fn what_the_record_fragment_leaves_out_is_refused() {
    for src in [
        "module m\ntype P {\n  x: Int,\n}\nfn f() -> Str {\n  str(P { x: 1 })\n}\n",
        "module m\ntype P {\n  x: Int,\n}\nfn f() -> Bool {\n  P { x: 1 } == P { x: 1 }\n}\n",
        "module m\ntype N {\n  v: Int,\n  next: Option[Int],\n}\nfn f() -> Int {\n  let n = N { v: 1, next: None }\n  n.v\n}\n",
    ] {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        assert!(compile_checked(&checked.module, &checked.result).is_err(), "stays on the interpreter:\n{src}");
    }
}
