//! RW 3.4 — `Float` on the WASM engine, held to the interpreter: arithmetic, comparison and negation (IEEE 754 on both, so
//! the results agree to the bit), `%` (the host's, as the interpreter's), `float` and `int` (Rust's `as` casts: rounding
//! to nearest, truncating and saturating), `str` of a `Float` (the interpreter's own display) and `parse_float` (its own
//! rule) — in lists, options, variables and function signatures.

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

/// The arithmetic at its edges: a sum that is not exact, a quotient, a negative zero, an overflow to infinity, a NaN and
/// its comparisons, `%` with signs and by zero, negation, and the casts at the ends of their ranges.
#[test]
fn float_arithmetic_matches_the_interpreter() {
    let src = r#"module m

fn b(x: Bool) -> Str {
  if x { "1" } else { "0" }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let third = 1.0 / 3.0
  out.println(str(0.1 + 0.2) + " " + str(third) + " " + str(2.5 * 4.0) + " " + str(0.0 - 0.0) + " " + str(-0.0))
  let huge = 1.0e300 * 1.0e10
  let nan = 0.0 / 0.0
  out.println(str(huge) + " " + str(-huge) + " " + str(nan) + " " + b(nan == nan) + b(nan != nan) + b(nan < 1.0) + b(nan >= 1.0))
  out.println(str(5.5 % 2.0) + " " + str(-5.5 % 2.0) + " " + str(5.5 % -2.0) + " " + str(1.0 % 0.0) + " " + str(-(3.25)))
  out.println(b(1.5 < 2.5) + b(2.5 <= 2.5) + b(3.0 > 2.0) + b(2.0 >= 3.0) + b(0.0 == -0.0) + b(1.0 != 1.0))
  out.println(str(float(7)) + " " + str(float(9007199254740993)) + " " + str(float(0 - 3)))
  out.println(str(int(2.7)) + " " + str(int(-2.7)) + " " + str(int(1.0e30)) + " " + str(int(-1.0e30)) + " " + str(int(nan)) + " " + str(int(huge)))
}
"#;
    assert_eq!(
        assert_parity(src),
        "0.30000000000000004 0.3333333333333333 10.0 0.0 -0.0\ninf -inf NaN 0100\n1.5 -1.5 1.5 NaN -3.25\n111010\n\
         7.0 9007199254740992.0 -3.0\n2 -2 9223372036854775807 -9223372036854775808 0 9223372036854775807\n"
    );
}

/// `parse_float` by the interpreter's rule — padded, signed, an exponent — and `None` for what a `Float` literal may not
/// be: `inf`, `nan`, a magnitude past the range, a word, the empty string.
#[test]
fn parse_float_matches_the_interpreter() {
    let src = r#"module m

fn show(s: Str) -> Str {
  match parse_float(s) {
    Some(x) => str(x),
    None => "none",
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(show("3.5") + " " + show(" -2.25 ") + " " + show("1e3") + " " + show("0.1") + " " + show("7"))
  out.println(show("inf") + " " + show("nan") + " " + show("1e999") + " " + show("") + " " + show("abc") + " " + show("1.2.3"))
}
"#;
    assert_eq!(assert_parity(src), "3.5 -2.25 1000.0 0.1 7.0\nnone none none none none none\n");
}

/// `Float`s held where the fragment holds values: a function's parameters and result, a `var` updated in a loop, a list
/// pushed to, indexed, walked and mapped, an `Option` matched.
#[test]
fn floats_are_held_as_the_interpreter_holds_them() {
    let src = r#"module m

fn mean(xs: List[Float]) -> Float {
  var total = 0.0
  for x in xs { total = total + x }
  total / float(xs.len())
}

fn hottest(xs: List[Float]) -> Option[Float] {
  if xs.is_empty() {
    None
  } else {
    var best = xs[0]
    for x in xs {
      if x > best { best = x }
    }
    Some(best)
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var xs = [21.5, -3.0, 37.25]
  xs.push(19.75)
  out.println(str(mean(xs)) + " " + str(xs[2]) + " " + str(xs.len()))
  let shown = match hottest(xs) {
    Some(c) => "hottest " + str(c),
    None => "none",
  }
  let empty: List[Float] = []
  let nothing = match hottest(empty) {
    Some(c) => str(c),
    None => "none",
  }
  var line = ""
  for y in xs.map(fn(x: Float) -> Float { x * 2.0 - 0.5 }) { line = line + str(y) + "," }
  out.println(shown + " " + nothing + " " + line)
}
"#;
    assert_eq!(assert_parity(src), "18.875 37.25 4\nhottest 37.25 none 42.5,-6.5,74.0,39.0,\n");
}

/// Only a module that makes or takes a `Float` imports `delulu:float`, so every other module is unchanged; one with the
/// foreign interface keeps its own `float_to_str` and imports only `rem`.
#[test]
fn only_a_module_with_a_float_imports_the_float_interface() {
    let imports = |src: &str| {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile: {e:?}\n{src}"));
        wasm.windows(b"delulu:float".len()).any(|w| w == b"delulu:float")
    };
    assert!(!imports("module m\nfn f(a: Int) -> Int {\n  a * 2\n}\n"));
    assert!(imports("module m\nfn f(a: Int) -> Int {\n  int(float(a) * 1.5)\n}\n"));
    assert!(imports("module m\nfn f(x: Float) -> Bool {\n  x > 0.0\n}\n"));
    assert!(imports(
        "module m\nfn f(s: Str) -> Int {\n  match parse_float(s) {\n    Some(x) => int(x),\n    None => 0,\n  }\n}\n"
    ));
}
