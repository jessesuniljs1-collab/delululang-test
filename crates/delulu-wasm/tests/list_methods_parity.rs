//! RW 3.4 — the rest of `List` on the WASM engine, held to the interpreter: `pop`, `reverse`, `concat`, `slice`,
//! `contains`, `join`, `sort`, and the three that take a function, `filter`, `find` and `fold`. Each walks the list as it
//! was when the call began, as the interpreter's do; `slice` clamps as the interpreter clamps; `sort` is stable and orders
//! by the interpreter's rule, host-side; `find` stops at its first match, so a predicate with an effect runs once per
//! element examined; and the lists they make hold the elements themselves, so a record in two lists is one record.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

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

/// The seven first-order methods on lists of `Int`, `Str`, `Bool` and `Float`: `pop` to empty and once more, `reverse`,
/// `concat`, `slice` at every clamp, `contains`, `join` (and of an empty list), and a stable `sort` — `Bool` by its
/// number, `Str` by its bytes.
#[test]
fn the_first_order_list_methods_match_the_interpreter() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs: ref List[Int] = [5, 3, 9, 1]
  out.println(str(xs.pop()) + " " + str(xs) + " " + str(xs.pop()) + str(xs.pop()) + str(xs.pop()) + " " + str(xs.pop()) + " " + str(xs))
  let ys = [4, 8, 15, 16, 23, 42]
  out.println(str(ys.reverse()) + " " + str(ys) + " " + str(ys.concat([1, 2])) + " " + str([1, 2].concat(ys)))
  out.println(str(ys.slice(1, 3)) + str(ys.slice(-3, 2)) + str(ys.slice(2, 100)) + str(ys.slice(4, 1)) + str(ys.slice(10, 20)) + str(ys.slice(0, 6)))
  out.println(str(ys.contains(15)) + " " + str(ys.contains(7)) + " " + str(ys.contains(4)) + " " + str(ys.contains(42)))
  let words = ["pear", "Apple", "fig", "", "éclair", "apple"]
  out.println(words.join(", ") + "|" + words.join("") + "|" + words.slice(0, 0).join("-") + "|" + ["one"].join("+"))
  out.println(str(words.sort()) + " " + str(words) + " " + str(words.contains("fig")) + " " + str(words.contains("Fig")))
  out.println(str([3, -1, 2, -1, 0].sort()) + " " + str([true, false, true, false].sort()) + " " + str([1.5, 0.25].reverse()))
  out.println(str([1.5, 0.25].contains(0.25)) + " " + str([1.5, 0.25].contains(2.0)) + " " + str([true].concat([false])))
}
"#;
    let out = assert_parity(src);
    assert!(out.starts_with("Some(1) [5, 3, 9] Some(9)Some(3)Some(5) None []"), "{out}");
    assert!(out.contains("[, Apple, apple, fig, pear, éclair]"), "by bytes: {out}");
}

/// The three that take a function: `filter` keeping what its predicate holds for, `find` stopping at its first match —
/// its printing predicate prints once per element examined, the same on both engines — and `fold` with an `Int` and a `Str`
/// accumulator; each over the list as it was when the call began, a predicate that pushes onto it included.
#[test]
fn filter_find_and_fold_match_the_interpreter() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs: ref List[Int] = [3, 8, 1, 9, 4, 7]
  out.println(str(xs.filter(fn(x: Int) -> Bool { x > 4 })) + " " + str(xs.filter(fn(x: Int) -> Bool { x > 100 })))
  let hit = xs.find(fn(x: Int) -> Bool {
    out.println("look " + str(x))
    x % 3 == 0 && x > 3
  })
  out.println(str(hit) + " " + str(xs.find(fn(x: Int) -> Bool { x > 100 })))
  out.println(str(xs.fold(0, fn(acc: Int, x: Int) -> Int { acc + x })) + " " + xs.fold("", fn(acc: Str, x: Int) -> Str { acc + str(x) }))
  let grown = xs.filter(fn(x: Int) -> Bool {
    xs.push(x)
    true
  })
  out.println(str(grown.len()) + " " + str(xs.len()))
  let names = ["ada", "grace", "edsger", "barbara"]
  let long = names.filter(fn(n: Str) -> Bool { n.len() > 4 })
  out.println(str(long) + " " + str(names.find(fn(n: Str) -> Bool { n.starts_with("e") })) + " " + names.fold(">", fn(a: Str, n: Str) -> Str { a + n.slice(0, 1) }))
}
"#;
    let out = assert_parity(src);
    assert!(out.contains("look 3\nlook 8\nlook 1\nlook 9\nSome(9)"), "find stops at its first match: {out}");
    assert!(out.contains("6 12"), "the walk is the list as it began: {out}");
}

/// The lists these methods make hold the elements themselves: a record in a reversed, concatenated, sliced, filtered or
/// sorted-by-key list is the record the first list holds, so a write through one is seen through all.
#[test]
fn the_lists_they_make_hold_the_records_themselves() {
    let src = r#"module m

type Cell {
  v: Int,
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let a: ref Cell = Cell { v: 1 }
  let b: ref Cell = Cell { v: 2 }
  let cells: ref List[ref Cell] = [a, b]
  let r = cells.reverse()
  let c = cells.concat([b])
  let s = cells.slice(1, 2)
  let f = cells.filter(fn(x: ref Cell) -> Bool { x.v > 1 })
  b.v = 20
  out.println(str(r) + " " + str(c) + " " + str(s) + " " + str(f) + " " + str(cells.pop()) + " " + str(cells))
  out.println(str(cells.fold(0, fn(acc: Int, x: ref Cell) -> Int { acc + x.v })))
}
"#;
    let out = assert_parity(src);
    assert!(out.starts_with("[Cell { v: 20 }, Cell { v: 1 }]"), "{out}");
}

/// `sort` and `join` run host-side over the guest's memory, read as untrusted (a `.dwx` can be written by hand): a header
/// whose length passes its capacity, slots outside the memory, and a string outside it are each refused in words (DL0903),
/// never a panic and never a read past the memory.
#[test]
fn a_hand_written_list_outside_the_memory_is_refused_in_words() {
    let module = |body: &str, data: &str| {
        format!(
            r#"(module
  (import "delulu:list" "sort" (func $sort (param i32 i32) (result i32)))
  (import "delulu:list" "join" (func $join (param i32 i32) (result i32)))
  (memory (export "memory") 1)
  (global (export "__heap") (mut i32) (i32.const 1024))
  (data (i32.const 0) "{data}")
  (func (export "main") (param i32) {body}))"#
        )
    };
    let cases = [
        // len 2, cap 1.
        (
            "a length past its capacity",
            module("(drop (call $sort (i32.const 0) (i32.const 0)))", "\\02\\00\\00\\00\\01\\00\\00\\00\\40\\00\\00\\00"),
        ),
        // One slot, its data at 0xFFFF0000.
        (
            "slots past the memory",
            module("(drop (call $sort (i32.const 0) (i32.const 0)))", "\\01\\00\\00\\00\\01\\00\\00\\00\\00\\00\\ff\\ff"),
        ),
        // One `Str` slot at 12, naming a string at 0xFFFFFFF0.
        (
            "a string past the memory",
            module(
                "(drop (call $sort (i32.const 0) (i32.const 2)))",
                "\\01\\00\\00\\00\\01\\00\\00\\00\\0c\\00\\00\\00\\f0\\ff\\ff\\ff\\00\\00\\00\\00",
            ),
        ),
        // One `Str` slot naming a cell inside the memory whose length runs past its end.
        (
            "a string running past the memory",
            module(
                "(drop (call $sort (i32.const 0) (i32.const 2)))",
                "\\01\\00\\00\\00\\01\\00\\00\\00\\0c\\00\\00\\00\\14\\00\\00\\00\\00\\00\\00\\00\\00\\ff\\ff\\ff",
            ),
        ),
        // The same list joined, by a separator at 0 (its length the header's first word).
        (
            "a joined string past the memory",
            module(
                "(drop (call $join (i32.const 0) (i32.const 0)))",
                "\\01\\00\\00\\00\\01\\00\\00\\00\\0c\\00\\00\\00\\f0\\ff\\ff\\ff\\00\\00\\00\\00",
            ),
        ),
    ];
    for (what, wat) in cases {
        let got = run_main(wat.as_bytes(), &HostConfig { console: true, ..HostConfig::default() });
        let err = got.expect_err(what).message();
        assert!(err.contains("DL0903") && err.contains("a list's elements lie outside the guest's memory"), "{what}: {err}");
    }
}

/// A module whose only list is one a map made (`keys()`) — no literal, no `push`, no index — still carries the list
/// helpers its methods need: the module is read for a method on a receiver the checker typed a `List`, not only for the
/// three names it was read for before.
#[test]
fn a_list_no_literal_made_still_has_its_methods() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let m: ref Map[Str, Int] = Map()
  m.insert("b", 2)
  m.insert("a", 1)
  m.insert("c", 3)
  out.println(str(m.keys().reverse()) + " " + m.keys().join("+") + " " + str(m.values().fold(0, fn(t: Int, v: Int) -> Int { t + v })))
}
"#;
    assert_eq!(assert_parity(src), "[c, b, a] a+b+c 6\n");
}

/// A `pop` and then a `push` reuse the popped slot. Every walk that runs code per element — `for`, `map`, `filter`,
/// `find`, `fold` — walks the list as it was when it began, as the interpreter's walk of a copy does, so the code it runs
/// popping and pushing changes the list and not the walk. (This is why `pop` waited: a walk of the list's own slots read
/// the pushed value where the interpreter read the popped one.)
#[test]
fn a_walk_is_not_moved_by_a_pop_and_a_push_in_its_own_code() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs: ref List[Int] = [1, 2, 3, 4]
  var seen = ""
  for x in xs {
    if x == 1 {
      let p = xs.pop()
      xs.push(9)
    }
    seen = seen + str(x)
  }
  out.println(seen + " " + str(xs))
  let ys: ref List[Int] = [1, 2, 3, 4]
  let doubled = ys.map(fn(y: Int) -> Int {
    if y == 1 {
      let p = ys.pop()
      ys.push(50)
    }
    y * 2
  })
  let kept = ys.filter(fn(y: Int) -> Bool {
    if y == 1 {
      let p = ys.pop()
      ys.push(70)
    }
    y > 2
  })
  let total = ys.fold(0, fn(t: Int, y: Int) -> Int {
    if y == 1 {
      let p = ys.pop()
      ys.push(1000)
    }
    t + y
  })
  let found = ys.find(fn(y: Int) -> Bool {
    if y == 1 {
      let p = ys.pop()
      ys.push(3)
    }
    y > 999
  })
  out.println(str(doubled) + " " + str(kept) + " " + str(total) + " " + str(found) + " " + str(ys))
}
"#;
    let out = assert_parity(src);
    assert!(out.starts_with("1234 [1, 2, 3, 9]\n"), "the walk is the list as it began: {out}");
}

/// `contains` on a list of records or sums compares as `==` of two of them does — by the interpreter's own `eq`, host-side
/// (`reify.rs`): a record equal by its contents is contained though it is another record, and a sum by its constructor and
/// payload.
#[test]
fn contains_on_records_and_sums_compares_as_equality_does() {
    let src = r#"module m

type P {
  x: Int,
  name: Str,
}

type T = A | B(Int)

fn main(root: Root) ! {Write} {
  let out = root.console()
  let ps = [P { x: 1, name: "a" }, P { x: 2, name: "b" }]
  out.println(str(ps.contains(P { x: 2, name: "b" })) + " " + str(ps.contains(P { x: 2, name: "c" })) + " " + str(ps.slice(0, 0).contains(P { x: 1, name: "a" })))
  let ts = [A, B(3)]
  out.println(str(ts.contains(B(3))) + " " + str(ts.contains(B(4))) + " " + str(ts.contains(A)) + " " + str([B(1)].contains(A)))
}
"#;
    assert_eq!(assert_parity(src), "true false false\ntrue false true false\n");
}
