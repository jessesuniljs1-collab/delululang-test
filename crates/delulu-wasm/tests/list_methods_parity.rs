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
/// its printing predicate prints once per element examined, the same on both engines — and `fold` with an `Int`, a `Str`
/// and a `List` accumulator; each over the list as it was when the call began, a predicate that pushes onto it included.
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
        ("a length past its capacity", module("(drop (call $sort (i32.const 0) (i32.const 0)))", "\\02\\00\\00\\00\\01\\00\\00\\00\\40\\00\\00\\00")),
        // One slot, its data at 0xFFFF0000.
        ("slots past the memory", module("(drop (call $sort (i32.const 0) (i32.const 0)))", "\\01\\00\\00\\00\\01\\00\\00\\00\\00\\00\\ff\\ff")),
        // One `Str` slot at 12, naming a string at 0xFFFFFFF0.
        ("a string past the memory", module("(drop (call $sort (i32.const 0) (i32.const 2)))", "\\01\\00\\00\\00\\01\\00\\00\\00\\0c\\00\\00\\00\\f0\\ff\\ff\\ff\\00\\00\\00\\00")),
        // The same list joined, by a separator at 0 (its length the header's first word).
        ("a joined string past the memory", module("(drop (call $join (i32.const 0) (i32.const 0)))", "\\01\\00\\00\\00\\01\\00\\00\\00\\0c\\00\\00\\00\\f0\\ff\\ff\\ff\\00\\00\\00\\00")),
    ];
    for (what, wat) in cases {
        let got = run_main(wat.as_bytes(), &HostConfig { console: true, ..HostConfig::default() });
        let err = got.expect_err(what).message();
        assert!(err.contains("DL0903") && err.contains("a list's elements lie outside the guest's memory"), "{what}: {err}");
    }
}
