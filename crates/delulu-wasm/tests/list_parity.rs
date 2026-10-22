//! RW 3.4 — `List` on the WASM engine, held to the interpreter: literals, `len`, `is_empty`, `get`, `push`, indexing,
//! `for`, `map` and `Str.split`, with the interpreter's sharing (`let ys = xs` names one list), its snapshot `for`, its
//! `None` outside the list and its DL0903 for an index outside it, word for word. Two defects found while building it
//! are witnessed here too: WASM-IFTY-1 (an `if` or `match` of `Str` variables compiled to a module wasmtime refused) and
//! WASM-INTERN-1 (a string literal inside a lambda panicked the compiler).

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, compile_module, run_main, HostConfig};

/// Run `main` on both engines with the console and return `(wasm, interp)`: the output, or the fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
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

#[test]
fn lists_of_ints_strs_and_bools_match_the_interpreter() {
    let src = r#"module m

fn total(xs: List[Int]) -> Int {
  var t = 0
  for x in xs { t = t + x }
  t
}

fn names() -> List[Str] {
  ["ada", "grace", "edsger"]
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs = [3, 1, 4, 1, 5]
  out.println(str(xs.len()) + " " + str(total(xs)) + " " + str(xs[0]) + str(xs[4]) + " " + str(xs.is_empty()))
  let ns = names()
  var line = ""
  for n in ns { line = line + n + "/" }
  out.println(line + str(ns[1].len()))
  let flags = [true, false, true]
  var yes = 0
  for f in flags {
    if f { yes = yes + 1 }
  }
  out.println(str(yes) + " " + str(flags[1]))
  let empty: List[Int] = []
  out.println(str(empty.len()) + " " + str(empty.is_empty()) + " " + str(total(empty)))
}
"#;
    assert_eq!(assert_parity(src), "5 14 35 false\nada/grace/edsger/5\n2 false\n0 true 0\n");
}

/// `get` answers `Some` inside the list and `None` outside it, a negative index included; a `match` on it binds a `Str`
/// (WASM-IFTY-1's `match` half: the arm's type was guessed `Int` from the variable's syntax and refused).
#[test]
fn get_answers_none_outside_the_list() {
    let src = r#"module m

fn show(o: Option[Str]) -> Str {
  match o {
    Some(s) => s,
    None => "-",
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs = ["a", "b"]
  out.println(show(xs.get(0)) + show(xs.get(1)) + show(xs.get(2)) + show(xs.get(0 - 1)))
  let ns = [10, 20]
  let v = match ns.get(1) {
    Some(n) => n * 2,
    None => 0,
  }
  out.println(str(v))
}
"#;
    assert_eq!(assert_parity(src), "ab--\n40\n");
}

/// `push` grows a list in place, through every name for it; the value is computed before the list grows; and a `for`
/// walks the list as it was when the loop began, though its body pushes past the slots it started with.
#[test]
fn push_is_shared_and_for_walks_a_snapshot() {
    let src = r#"module m

fn add(xs: ref List[Int], v: Int) {
  xs.push(v)
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs = [1, 2]
  let ys = xs
  ys.push(3)
  add(xs, 4)
  xs.push(xs.len())
  out.println(str(xs.len()) + " " + str(ys.len()) + " " + str(xs[4]))
  var seen = 0
  for x in xs {
    xs.push(x * 10)
    seen = seen + 1
  }
  out.println(str(seen) + " " + str(xs.len()) + " " + str(xs[9]))
  let big: ref List[Int] = []
  var i = 0
  while i < 100 {
    big.push(i)
    i = i + 1
  }
  var t = 0
  for b in big { t = t + b }
  out.println(str(big.len()) + " " + str(t) + " " + str(big[99]))
}
"#;
    assert_eq!(assert_parity(src), "5 5 4\n5 10 40\n100 4950 99\n");
}

#[test]
fn break_and_continue_in_a_for_match_the_interpreter() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xs = [1, 2, 3, 4, 5, 6, 7, 8, 9]
  var t = 0
  for x in xs {
    if x % 2 == 0 { continue }
    if x > 7 { break }
    t = t + x
  }
  var grid = ""
  for a in [1, 2, 3] {
    for b in ["x", "yy", "z"] {
      if b.len() == 2 { continue }
      grid = grid + str(a) + b
    }
    if a == 2 { break }
  }
  out.println(str(t) + " " + grid)
}
"#;
    assert_eq!(assert_parity(src), "16 1x1z2x2z\n");
}

/// `map` inlines its function per element, the function reading the names where it was written.
#[test]
fn map_inlines_its_function_per_element() {
    let src = r#"module m

fn main(root: Root) ! {Write} {
  let out = root.console()
  let k = 3
  let xs = [1, 2, 3]
  let ys = xs.map(fn(x: Int) -> Int { x * k })
  let ss = xs.map(fn(x: Int) -> Str { "<" + str(x) + ">" })
  let scale = fn(x: Int) -> Int { x + k }
  let zs = ys.map(scale)
  var line = ""
  for s in ss { line = line + s }
  out.println(str(ys[2]) + " " + line + " " + str(zs[0]) + str(zs.len()) + " " + str(xs[0]))
}
"#;
    assert_eq!(assert_parity(src), "9 <1><2><3> 63 1\n");
}

/// `Str.split` by the interpreter's rule — every part between separators, empty ones included, and one per character
/// for an empty separator, multi-byte characters whole.
#[test]
fn split_matches_the_interpreter() {
    let src = r#"module m

fn show(xs: List[Str]) -> Str {
  var s = str(xs.len()) + ":"
  for x in xs { s = s + "[" + x + "]" }
  s
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(show("the quick brown fox".split(" ")))
  out.println(show("a,,b,".split(",")))
  out.println(show("héllo ✓".split("")))
  out.println(show("no separator here".split("|")))
  out.println(show("".split(",")))
  out.println(show("ab--cd--".split("--")))
}
"#;
    assert_eq!(
        assert_parity(src),
        "4:[the][quick][brown][fox]\n4:[a][][b][]\n7:[h][é][l][l][o][ ][✓]\n1:[no separator here]\n1:[]\n3:[ab][cd][]\n"
    );
}

/// `Str == Str` and `!=` compare the bytes on both engines: one interned literal against itself, a built string against a
/// literal, differing lengths, a differing byte, multi-byte characters, empty strings — and words split from a line.
#[test]
fn str_equality_matches_the_interpreter() {
    let src = r#"module m

fn b(x: Bool) -> Str {
  if x { "1" } else { "0" }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let abc = "ab" + "c"
  out.println(b("abc" == "abc") + b(abc == "abc") + b(abc != "abc") + b("abc" == "abcd") + b("abd" == "abc"))
  out.println(b("é" == "e") + b("é" == "é") + b("" == "") + b("" == "a") + b("✓x" != "✓y"))
  var hits = 0
  for w in "the cat and the hat".split(" ") {
    if w == "the" { hits = hits + 1 }
    if w != "cat" && w.len() == 3 { hits = hits + 10 }
  }
  out.println(str(hits))
}
"#;
    assert_eq!(assert_parity(src), "11000\n01101\n42\n");
}

/// An index outside the list faults DL0903 on both engines, in the interpreter's words; what was printed before stays.
#[test]
fn an_index_outside_the_list_is_the_interpreters_dl0903() {
    for (index, len) in [("3", "3"), ("0 - 1", "3"), ("9223372036854775807", "3")] {
        let src = format!(
            "module m\nfn main(root: Root) ! {{Write}} {{\n  let out = root.console()\n  let xs = [7, 8, 9]\n  \
             out.println(\"before\")\n  out.println(str(xs[{index}]))\n}}\n"
        );
        let (w, i) = run_both(&src);
        let (w, i) = (w.expect_err("WASM faults"), i.expect_err("the interpreter faults"));
        let n = if index == "0 - 1" { "-1" } else { index };
        let words = format!("index {n} out of bounds (len {len})");
        assert!(i.contains("DL0903") && i.contains(&words), "the interpreter: {i}");
        assert!(w.contains("DL0903") && w.contains(&words), "the WASM engine, in the interpreter's words: {w}");
    }
}

/// WASM-IFTY-1 and WASM-INTERN-1, each red on the backend before this change: an `if` choosing between two `Str`
/// variables compiled to an invalid module, and a string literal inside a lambda panicked `compile_module`.
#[test]
fn an_if_of_strs_and_a_literal_in_a_lambda_compile() {
    let src = r#"module m

fn apply[T](g: fn(T) -> T, x: T) -> T { g(x) }

fn pick(a: Str, b: Str, c: Bool) -> Str {
  if c { a } else { b }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let x = "left"
  let y = "right"
  let z = if x.len() > 2 { x } else { y }
  out.println(pick("a", "b", false) + " " + z + " " + apply(fn(s: Str) -> Str { s + " and a literal" }, "a value"))
}
"#;
    assert_eq!(assert_parity(src), "b left a value and a literal\n");
}

/// What stays outside the fragment is refused, never miscompiled: a list of functions, and a list of the root, which no
/// slot holds (it enters only at `main`) — by `compile_checked` too, which knows every node's type. (`pop`
/// compiles since D-V2-171, its reuse of a slot kept from every walk: `list_methods_parity.rs`; a list of lists and a
/// list holding a map since a slot holds a pointer to any list, map, option or result: `nested_parity.rs`; a list of the
/// capabilities a variable may hold since a slot holds a handle, D-V2-177: `handle_parity.rs`.)
#[test]
fn what_the_list_fragment_leaves_out_is_refused() {
    for src in [
        "module m\nfn f() -> Int {\n  let fs = [fn(x: Int) -> Int { x }]\n  fs.len()\n}\n",
        "module m\nfn f(root: Root) -> Int {\n  let xs = [root]\n  xs.len()\n}\n",
    ] {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        assert!(compile_module(&checked.module).is_err(), "stays on the interpreter:\n{src}");
        assert!(compile_checked(&checked.module, &checked.result).is_err(), "stays on the interpreter:\n{src}");
    }
}

/// An empty list whose element type no `let` writes takes the type the checker inferred — through `compile_checked`,
/// which the command line compiles with: `let evens = []` filled by `push` later, a function's `let out = []`, an empty
/// literal passed straight to a function, lists of `Bool` and `Float` begun empty. Without the checker's types
/// (`compile_module`) the same program is refused, never guessed.
#[test]
fn an_empty_list_takes_the_type_the_checker_inferred() {
    let src = r#"module m

fn total(xs: List[Int]) -> Int {
  var t = 0
  for x in xs { t = t + x }
  t
}

fn words(n: Int) -> List[Str] {
  let out = []
  var i = 0
  while i < n {
    out.push("w" + str(i))
    i = i + 1
  }
  out
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let evens = []
  var i = 0
  while i <= 10 {
    if i % 2 == 0 { evens.push(i) }
    i = i + 1
  }
  let flags = []
  flags.push(true)
  let fs = []
  fs.push(2.5)
  out.println(str(evens.len()) + " " + str(total(evens)) + " " + str(total([])) + " " + str(words(3).len()) + words(3)[2])
  out.println(str(flags[0]) + " " + str(fs[0]) + " " + str(words(0).len()))
}
"#;
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    assert!(compile_module(&checked.module).is_err(), "without the checker's types, an empty list is refused");
    let wasm = compile_checked(&checked.module, &checked.result).expect("with them, it compiles");
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    assert_eq!(w, i.map(|_| out), "two-engine parity broken");
    assert_eq!(w.unwrap(), "6 30 0 3w2\ntrue 2.5 0\n");
}
