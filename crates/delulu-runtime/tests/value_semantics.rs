//! `==` and `str` on the values a program builds, held to what the checker admits and to the rule that a runtime fault is
//! a diagnostic, never a host crash:
//! - EQ-RECORD-1: a record and a map compare as their contents. `Value::eq` had no arm for either, so `p == p` was
//!   `false` and `p != p` `true`, and `List.contains` and `assert_eq` answered the same wrong way — though the checker
//!   admits `==` on both;
//! - DISPLAY-CYCLE-1: a value that holds itself (`n.next = Some(n)`; a record in a list held by that record) renders in
//!   finite text and compares in finite time. `str` of one recursed until the stack overflowed and the process died.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};

/// Run `main` on the interpreter with the console; its output.
fn run(src: &str) -> String {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let r = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root())));
    let out = take_capture().unwrap_or_default();
    r.unwrap_or_else(|e| panic!("the program faulted: {e:?}\n{src}"));
    out
}

/// A record equals itself and one of equal fields, not one that differs; a list of records contains an equal one and is
/// equal to an equal list in the same order only; a map equals one with the same keys and equal values; a NaN inside a
/// list keeps the list unequal to itself, as `nan == nan` is false.
#[test]
fn records_and_maps_compare_as_their_contents() {
    let src = r#"module m

type P {
  x: Int,
  name: Str,
}

fn b(x: Bool) -> Str {
  if x { "1" } else { "0" }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let p = P { x: 1, name: "a" }
  let same = P { x: 1, name: "a" }
  let other = P { x: 2, name: "a" }
  out.println(b(p == p) + b(p == same) + b(p != same) + b(p == other) + b(p != other))
  let ps = [p, other]
  out.println(b(ps.contains(same)) + b(ps.contains(P { x: 3, name: "a" })) + b(ps == [same, other]) + b(ps == [other, same]))
  let m = Map()
  m.insert("k", p)
  let n = Map()
  n.insert("k", same)
  out.println(b(m == m) + b(m == n))
  n.insert("j", same)
  out.println(b(m == n))
  let nan = 0.0 / 0.0
  let f = [nan]
  out.println(b(f == f))
}
"#;
    assert_eq!(run(src), "11001\n1010\n11\n0\n0\n");
}

/// A record that holds itself, through an `Option` field or through a list it holds, renders with `...` where it meets
/// itself, and compares in finite time: equal to itself and to another that holds itself the same way, unequal to one
/// that differs.
#[test]
fn a_value_that_holds_itself_renders_and_compares_in_finite_time() {
    let src = r#"module m

type Node {
  v: Int,
  next: Option[Node],
}

type Bag {
  items: List[Bag],
  label: Str,
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var n = Node { v: 1, next: None }
  n.next = Some(n)
  out.println(str(n))
  var m = Node { v: 1, next: None }
  m.next = Some(m)
  var k = Node { v: 2, next: None }
  k.next = Some(k)
  out.println(str(n == n) + " " + str(n == m) + " " + str(n == k))
  var items: ref List[Bag] = []
  let bag = Bag { items: items, label: "b" }
  items.push(bag)
  out.println(str(bag) + " " + str(bag == bag))
}
"#;
    assert_eq!(
        run(src),
        "Node { v: 1, next: Some(Node { ... }) }\ntrue true false\nBag { items: [Bag { ... }], label: b } true\n"
    );
}
