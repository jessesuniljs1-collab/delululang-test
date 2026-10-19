//! RW 3.4 — a function named as a value on the WASM engine, held to the interpreter: passed to a generic function
//! (`apply(double, 21)`), to `map`, bound with `let` and aliased, generic or not, recursive and returning early. A
//! compilable function named as a value is a call to it, so its recursion and its `return` stay its own; a generic one is
//! inlined as a call to it is. A local or a function value of the same name hides it, as on the interpreter.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_module, run_main, HostConfig};

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

/// Named functions as values: to a generic function once and twice, a recursive one that returns early, one on `Str`,
/// bound with `let` and aliased, mapped over a list, a generic one, and one named inside a generic function's body.
#[test]
fn a_named_function_is_a_value_as_on_the_interpreter() {
    let src = r#"module m

fn apply[T, U](f: fn(T) -> U, x: T) -> U {
  f(x)
}

fn twice[T](f: fn(T) -> T, x: T) -> T {
  f(f(x))
}

fn identity[T](x: T) -> T {
  x
}

fn double(n: Int) -> Int {
  n * 2
}

fn fact(n: Int) -> Int {
  if n < 2 {
    return 1
  }
  n * fact(n - 1)
}

fn shout(s: Str) -> Str {
  s.to_upper() + "!"
}

fn via_generic[T](x: T, n: Int) -> Int {
  apply(double, n)
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(str(apply(double, 21)) + " " + str(twice(double, 5)) + " " + str(apply(fact, 10)) + " " + apply(shout, "hey"))
  let h = double
  let g = h
  out.println(str(h(4)) + " " + str(g(g(1))) + " " + str(apply(g, 0 - 3)))
  let xs = [1, 2, 3, 4]
  var line = ""
  for y in xs.map(double) { line = line + str(y) + "," }
  out.println(line + " " + str(apply(identity, 7)) + " " + str(via_generic("x", 6)))
}
"#;
    assert_eq!(assert_parity(src), "42 20 3628800 HEY!\n8 4 -6\n2,4,6,8, 7 12\n");
}

/// What hides a function's name: a function value of that name (a lambda bound in a block, a generic function's
/// parameter) while it is in scope, and a local of that name — `let y = double` then copies the local.
#[test]
fn a_function_value_or_a_local_of_the_same_name_hides_the_function() {
    let src = r#"module m

fn apply[T](f: fn(T) -> T, x: T) -> T {
  f(x)
}

fn double(n: Int) -> Int {
  n * 2
}

fn inc(n: Int) -> Int {
  n + 1
}

fn pick[T](double: fn(T) -> T, x: T) -> T {
  apply(double, x)
}

fn shadowed() -> Int {
  let double = 7
  let y = double
  y + 1
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(str(apply(double, 5)) + " " + str(pick(inc, 5)))
  if true {
    let double = fn(n: Int) -> Int { n * 3 }
    out.println(str(apply(double, 5)))
  }
  out.println(str(apply(double, 5)) + " " + str(apply(fn(n: Int) -> Int { double(inc(n)) }, 5)) + " " + str(shadowed()))
}
"#;
    assert_eq!(assert_parity(src), "10 6\n15\n10 12 8\n");
}

/// A generic function's body is inlined into the functions that call it, so what it needs, the module carries: the
/// `Str` methods' import, `__str_eq`, the list helpers, its own string literals and the console. Before, the module was
/// built from the compilable functions' bodies alone, and each of these was refused (`DL1201`) for want of it.
#[test]
fn a_generic_body_brings_what_it_needs() {
    let src = r#"module m

fn tidy[T](x: T, s: Str) -> Str {
  s.trim()
}

fn same[T](x: T, a: Str, b: Str) -> Bool {
  a == b
}

fn second[T](x: T) -> Int {
  let xs = [4, 5, 6]
  xs[1]
}

fn label[T](x: T) -> Str {
  "only-in-a-generic"
}

fn say[T](out: Cap[Console], x: T) ! {Write} {
  out.println("said")
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(tidy(1, "  hi  ") + " " + str(same(1, "a", "a")) + " " + str(second(1)) + " " + label(true))
  say(out, 0)
}
"#;
    assert_eq!(assert_parity(src), "hi true 5 only-in-a-generic\nsaid\n");
}
