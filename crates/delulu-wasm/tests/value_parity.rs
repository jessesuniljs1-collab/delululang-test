//! RW 3.4 — `str` and `==` of a list, an option, a result, a sum or a record on the WASM engine, held to the interpreter.
//! The host rebuilds the guest's value as the interpreter's `Value` by its type's descriptor and answers with the
//! interpreter's own `display` and `eq` (`reify.rs`), so the two engines agree by construction on what the interpreter
//! decides: the text of a `Float`, a `NaN` unequal to itself inside a list, a value that holds itself rendered with
//! `...`, a record's fields in the order its type declares (RECORD-ORDER-1).

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// Run `main` on both engines with the console; both outputs, or each fault's text.
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

/// `str` and `==`/`!=` of each kind: a list of `Int`s and of `Float`s, an option, a result, a sum with and without a
/// payload, a record holding a sum, a list and a result — equal to one whose literal wrote its fields in another order,
/// unequal once its list grows through a field.
#[test]
fn str_and_equality_of_compound_values_match_the_interpreter() {
    let src = r#"module m

type Color = Red | Green | Rgb(Int, Int, Float)

type P {
  x: Float,
  name: Str,
  on: Bool,
  c: Color,
  tags: ref List[Str],
  r: Result[Int, Str],
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs = [1, 2, 3]
  out.println(str(xs) + " " + str(xs == [1, 2, 3]) + " " + str(xs != [1, 2]) + " " + str([1.5, 0.0]))
  out.println(str(Some(4)) + " " + str(None == Some(1)))
  let e: Result[Int, Str] = Err("bad")
  out.println(str(e) + " " + str(Rgb(1, 2, 0.5)) + " " + str(Red) + " " + str(Rgb(1, 2, 0.5) == Rgb(1, 2, 0.5)))
  var p = P { name: "p", x: 1.0, on: true, c: Green, tags: ["a", "b"], r: Ok(3) }
  let q = P { x: 1.0, name: "p", on: true, c: Green, tags: ["a", "b"], r: Ok(3) }
  out.println(str(p))
  out.println(str(p == q) + " " + str(p == p) + " " + str([p] == [q]) + " " + str(p != q))
  p.tags.push("c")
  out.println(str(p == q) + " " + str(p.tags) + " " + str(Rgb(1, 2, 0.5) == Rgb(1, 3, 0.5)))
}
"#;
    assert_eq!(
        assert_parity(src),
        "[1, 2, 3] true true [1.5, 0.0]
Some(4) false
Err(bad) Rgb(1, 2, 0.5) Red true
P { x: 1.0, name: p, on: true, c: Green, tags: [a, b], r: Ok(3) }
true true true false
false [a, b, c] false
"
    );
}

/// What the interpreter decides, decided once: a record that holds itself renders with `...` where it meets itself and
/// equals itself in finite time; a `NaN` inside a list or an option keeps it unequal to itself; a record held twice in a
/// list is one cell, so a write through one is seen through the other.
#[test]
fn a_value_that_holds_itself_and_a_nan_match_the_interpreter() {
    let src = r#"module m

type Node {
  v: Int,
  next: Option[ref Node],
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var n = Node { v: 1, next: None }
  n.next = Some(n)
  out.println(str(n) + " " + str(n == n))
  let nan = 0.0 / 0.0
  out.println(str([nan] == [nan]) + " " + str(Some(nan) == Some(nan)) + " " + str([nan]))
  var k = Node { v: 7, next: None }
  let twice = [k, k]
  k.v = 8
  out.println(str(twice) + " " + str(twice[0] == twice[1]))
}
"#;
    assert_eq!(
        assert_parity(src),
        "Node { v: 1, next: Some(Node { ... }) } true
false false [NaN]
[Node { v: 8, next: None }, Node { v: 8, next: None }] true
"
    );
}

/// Without the checker's types (`compile_module`) the backend cannot know a `str` or `==` needs the host's value
/// interface, and refuses rather than guess; and what stays outside — `str` of a map, a list of options — is refused with
/// the types too.
#[test]
fn what_the_value_interface_leaves_out_is_refused() {
    let src = "module m\nfn f() -> Str {\n  str([1, 2])\n}\n";
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    assert!(compile_module(&checked.module).is_err(), "no types, no `delulu:value`: refused");
    assert!(compile_checked(&checked.module, &checked.result).is_ok(), "with the checker's types it compiles");
    for src in [
        "module m\nfn f() -> Str {\n  let m: Map[Str, Int] = Map()\n  str(m)\n}\n",
        "module m\nfn f() -> Str {\n  str([Some(1)])\n}\n",
    ] {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        assert!(compile_checked(&checked.module, &checked.result).is_err(), "stays on the interpreter:\n{src}");
    }
}

/// Inside a generic body the checker's type is a variable, so the module carries the value interface for it, and each
/// inlined copy describes the type it was given.
#[test]
fn str_and_equality_inside_a_generic_body_match_the_interpreter() {
    let src = r#"module m

fn show[T](x: T) -> Str {
  "<" + str(x) + ">"
}

fn same[T](a: T, b: T) -> Bool {
  a == b
}

fn main(root: Root) ! {Write} {
  root.console().println(show([1, 2]) + show(Some("a")) + " " + str(same([1], [1])) + str(same(Some(2), None)))
}
"#;
    assert_eq!(assert_parity(src), "<[1, 2]><Some(a)> truefalse\n");
}
